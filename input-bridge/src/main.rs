use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    fifo_writer: Arc<Mutex<std::fs::File>>, // Persistent writer to keep FIFO open and serialize writes
}

#[derive(Deserialize)]
struct InputMessage {
    text: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum StreamJsonEvent<'a> {
    #[serde(rename = "user")]
    User { #[serde(borrow)] message: StreamJsonUserMessage<'a> },
}

#[derive(Serialize)]
struct StreamJsonUserMessage<'a> {
    role: &'a str,
    content: Vec<StreamJsonContent<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum StreamJsonContent<'a> {
    #[serde(rename = "text")]
    Text { text: &'a str },
}

async fn handle_input(
    State(state): State<AppState>,
    Json(payload): Json<InputMessage>,
) -> impl IntoResponse {
    // Lock the persistent writer to serialize writes and keep the FIFO open
    let file_guard = state.fifo_writer.lock().await;

    let message = StreamJsonEvent::User {
        message: StreamJsonUserMessage {
            role: "user",
            content: vec![StreamJsonContent::Text { text: &payload.text }],
        },
    };

    match write_to_fifo(&file_guard, &message).await {
        Ok(_) => (StatusCode::OK, "Message sent successfully"),
        Err(e) => {
            error!("Failed to write to FIFO: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send message")
        }
    }
}

async fn write_to_fifo(file: &std::fs::File, message: &StreamJsonEvent<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let json_line = serde_json::to_string(message)? + "\n";
    // Clone the file handle so we can move it into the blocking task
    let mut writer = file.try_clone()?;
    tokio::task::spawn_blocking(move || {
        writer.write_all(json_line.as_bytes())?;
        writer.flush()?;
        Ok::<_, std::io::Error>(())
    })
    .await??;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let fifo_path = std::env::var("FIFO_PATH").unwrap_or_else(|_| "/workspace/agent-input.jsonl".to_string());
    let fifo_path = PathBuf::from(fifo_path);

    info!("Starting input-bridge server, FIFO path: {:?}", fifo_path);

    let mut attempts = 0;
    while !fifo_path.exists() && attempts < 60 {
        warn!("Waiting for FIFO to be created at {:?}...", fifo_path);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        attempts += 1;
    }

    if !fifo_path.exists() {
        error!("FIFO not found after 2 minutes, exiting");
        std::process::exit(1);
    }

    // Open a persistent writer (read-write to avoid open() blocking) and keep it for the lifetime of the process
    // Retry to avoid race with chmod in main container
    let persistent_file = {
        let p = fifo_path.clone();
        let mut tries: u32 = 0;
        loop {
            match tokio::task::spawn_blocking({
                let p2 = p.clone();
                move || OpenOptions::new().read(true).write(true).append(true).open(p2)
            })
            .await
            {
                Ok(Ok(f)) => break f,
                Ok(Err(e)) => {
                    tries += 1;
                    if tries % 5 == 0 {
                        warn!("Retrying opening FIFO (attempt {}): {}", tries, e);
                    }
                    if tries > 120 {
                        error!("Failed to open FIFO after {} attempts: {}", tries, e);
                        panic!("Failed to open FIFO for persistent writer: {}", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
                Err(join_err) => {
                    tries += 1;
                    if tries > 120 {
                        panic!("Join error repeatedly when opening FIFO writer: {}", join_err);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            }
        }
    };

    let state = AppState { fifo_writer: Arc::new(Mutex::new(persistent_file)) };

    let app = Router::new().route("/input", post(handle_input)).route("/health", get(health_check)).with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Input bridge listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}


