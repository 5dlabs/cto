use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs::{metadata, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    fifo_path: PathBuf,                    // Path to the FIFO
    write_lock: Arc<Mutex<()>>,            // Serialize writes to avoid interleaving
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>, // Signal for graceful shutdown
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
    // Serialize writes so multiple requests don't interleave JSON lines
    let _guard = state.write_lock.lock().await;

    let message = StreamJsonEvent::User {
        message: StreamJsonUserMessage {
            role: "user",
            content: vec![StreamJsonContent::Text { text: &payload.text }],
        },
    };

    match write_to_fifo(&state.fifo_path, &message).await {
        Ok(_) => (StatusCode::OK, "Message sent successfully"),
        Err(e) => {
            error!("Failed to write to FIFO: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send message")
        }
    }
}

async fn write_to_fifo(
    fifo_path: &Path,
    message: &StreamJsonEvent<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_line = serde_json::to_string(message)? + "\n";
    let path = fifo_path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        // Open FIFO for write-only; this will block until a reader is present (the main container)
        let mut writer = OpenOptions::new().write(true).open(path)?;
        writer.write_all(json_line.as_bytes())?;
        writer.flush()?; // Closing the file drops writer, allowing EOF when no other writers are open
        Ok::<_, std::io::Error>(())
    })
    .await??;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn stop_if_sentinel_present(fifo_dir: &Path) {
    let sentinel = fifo_dir.join(".agent_done");
    if metadata(&sentinel).is_ok() {
        // Sentinel exists; trigger shutdown by returning from server
        // This relies on an external /shutdown or graceful pod stop
        // We log to make it visible in sidecar logs
        tracing::info!("Sentinel detected at {:?}; awaiting shutdown signal", sentinel);
    }
}

async fn shutdown(State(state): State<AppState>) -> impl IntoResponse {
    let mut guard = state.shutdown_tx.lock().await;
    if let Some(tx) = guard.take() {
        let _ = tx.send(());
        (StatusCode::OK, "Shutting down")
    } else {
        (StatusCode::OK, "Already shutting down")
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let fifo_path = std::env::var("FIFO_PATH").unwrap_or_else(|_| "/workspace/agent-input.jsonl".to_string());
    let fifo_path = PathBuf::from(fifo_path);

    info!("Starting sidecar server, FIFO path: {:?}", fifo_path);

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

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let state = AppState {
        fifo_path: fifo_path.clone(),
        write_lock: Arc::new(Mutex::new(())),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    };

    let app = Router::new()
        .route("/input", post(handle_input))
        .route("/health", get(health_check))
        .route("/shutdown", post(shutdown))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Sidecar listening on {addr}");
    // Background watcher for sentinel file; if found and no further input, we still rely on /shutdown or pod stop.
    let fifo_dir = fifo_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/workspace"));
    tokio::spawn(async move {
        loop {
            stop_if_sentinel_present(&fifo_dir).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { let _ = shutdown_rx.await; })
        .await
        .unwrap();
}


