use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
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
    fifo_path: PathBuf,
    fifo_writer: Arc<Mutex<()>>, // Mutex to ensure sequential writes
}

#[derive(Deserialize)]
struct InputMessage {
    text: String,
}

#[derive(Serialize)]
struct JsonLMessage {
    role: String,
    content: String,
}

async fn handle_input(
    State(state): State<AppState>,
    Json(payload): Json<InputMessage>,
) -> impl IntoResponse {
    let _lock = state.fifo_writer.lock().await;
    
    // Create JSONL message
    let message = JsonLMessage {
        role: "user".to_string(),
        content: payload.text,
    };
    
    // Write to FIFO
    match write_to_fifo(&state.fifo_path, &message).await {
        Ok(_) => {
            info!("Successfully wrote message to FIFO");
            (StatusCode::OK, "Message sent successfully")
        }
        Err(e) => {
            error!("Failed to write to FIFO: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send message")
        }
    }
}

async fn write_to_fifo(path: &PathBuf, message: &JsonLMessage) -> Result<(), Box<dyn std::error::Error>> {
    // Open FIFO in append mode
    let mut file = tokio::task::spawn_blocking({
        let path = path.clone();
        move || {
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(path)
        }
    })
    .await??;
    
    // Write JSONL line
    let json_line = serde_json::to_string(message)? + "\n";
    tokio::task::spawn_blocking(move || {
        file.write_all(json_line.as_bytes())?;
        file.flush()?;
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();
    
    // Get FIFO path from environment or use default
    let fifo_path = std::env::var("FIFO_PATH")
        .unwrap_or_else(|_| "/workspace/agent-input.jsonl".to_string());
    let fifo_path = PathBuf::from(fifo_path);
    
    info!("Starting input-bridge server, FIFO path: {:?}", fifo_path);
    
    // Wait for FIFO to exist (main container should create it)
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
    
    info!("FIFO found, starting HTTP server");
    
    let state = AppState {
        fifo_path,
        fifo_writer: Arc::new(Mutex::new(())),
    };
    
    // Build router
    let app = Router::new()
        .route("/input", post(handle_input))
        .route("/health", axum::routing::get(health_check))
        .with_state(state);
    
    // Get port from environment or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    info!("Input bridge listening on {}", addr);
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
