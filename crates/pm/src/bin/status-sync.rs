//! Enhanced sidecar for Linear integration with 2-way communication.
//!
//! This sidecar runs alongside agent pods and provides:
//! - Status file monitoring and sync to Linear service
//! - Log file streaming to Linear agent dialog (`emit_thought`)
//! - Input polling from Linear and forwarding to agent FIFO
//! - HTTP server for local input injection

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, SeekFrom};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info, warn};

#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

// =============================================================================
// Configuration
// =============================================================================

/// Sidecar configuration from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    // Linear session info
    pub linear_session_id: String,
    pub linear_issue_id: String,
    pub linear_team_id: String,
    pub linear_oauth_token: Option<String>,
    pub workflow_name: String,

    // File paths
    pub status_file: String,
    pub log_file: String,
    pub input_fifo: String,

    // Service URLs
    pub linear_service_url: String,
    pub linear_api_url: String,

    // Intervals
    pub status_poll_interval_ms: u64,
    pub log_post_interval_ms: u64,
    pub input_poll_interval_ms: u64,

    // HTTP server
    pub http_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            linear_session_id: std::env::var("LINEAR_SESSION_ID").unwrap_or_default(),
            linear_issue_id: std::env::var("LINEAR_ISSUE_ID").unwrap_or_default(),
            linear_team_id: std::env::var("LINEAR_TEAM_ID").unwrap_or_default(),
            linear_oauth_token: std::env::var("LINEAR_OAUTH_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
            workflow_name: std::env::var("WORKFLOW_NAME").unwrap_or_else(|_| "unknown".to_string()),

            status_file: std::env::var("STATUS_FILE")
                .unwrap_or_else(|_| "/workspace/status.json".to_string()),
            log_file: std::env::var("LOG_FILE_PATH")
                .unwrap_or_else(|_| "/workspace/agent.log".to_string()),
            input_fifo: std::env::var("INPUT_FIFO_PATH")
                .unwrap_or_else(|_| "/workspace/agent-input.jsonl".to_string()),

            linear_service_url: std::env::var("LINEAR_SERVICE_URL")
                .unwrap_or_else(|_| "http://pm-svc.cto.svc.cluster.local:8081".to_string()),
            linear_api_url: std::env::var("LINEAR_API_URL")
                .unwrap_or_else(|_| "https://api.linear.app/graphql".to_string()),

            status_poll_interval_ms: std::env::var("STATUS_POLL_INTERVAL_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            log_post_interval_ms: std::env::var("LOG_POST_INTERVAL_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            input_poll_interval_ms: std::env::var("INPUT_POLL_INTERVAL_MS")
                .unwrap_or_else(|_| "2000".to_string())
                .parse()
                .unwrap_or(2000),

            http_port: std::env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
        }
    }

    /// Check if Linear session is configured.
    #[must_use]
    pub fn has_linear_session(&self) -> bool {
        !self.linear_session_id.is_empty()
    }

    /// Check if direct Linear API access is available.
    #[must_use]
    pub fn has_linear_api(&self) -> bool {
        self.linear_oauth_token.is_some() && self.has_linear_session()
    }
}

// =============================================================================
// Linear API Client (Lightweight)
// =============================================================================

/// Lightweight Linear API client for sidecar operations.
#[derive(Clone)]
pub struct LinearApiClient {
    client: reqwest::Client,
    api_url: String,
}

impl LinearApiClient {
    /// Create a new Linear API client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client fails to build or the access token is invalid.
    pub fn new(access_token: &str, api_url: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();

        // Linear API keys (lin_api_*) should NOT use Bearer prefix
        let auth_value = if access_token.starts_with("lin_api_") {
            access_token.to_string()
        } else {
            format!("Bearer {access_token}")
        };

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).context("Invalid access token")?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_url: api_url.to_string(),
        })
    }

    /// Emit a thought to the Linear agent session.
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_thought(&self, session_id: &str, body: &str) -> Result<()> {
        #[derive(Serialize)]
        struct Variables<'a> {
            input: ActivityInput<'a>,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ActivityInput<'a> {
            agent_session_id: &'a str,
            content: ActivityContent<'a>,
        }

        #[derive(Serialize)]
        struct ActivityContent<'a> {
            thought: ThoughtContent<'a>,
        }

        #[derive(Serialize)]
        struct ThoughtContent<'a> {
            body: &'a str,
        }

        #[derive(Serialize)]
        struct Request<'a> {
            query: &'static str,
            variables: Variables<'a>,
        }

        const MUTATION: &str = r"
            mutation EmitActivity($input: AgentActivityCreateInput!) {
                agentActivityCreate(input: $input) {
                    success
                }
            }
        ";

        let request = Request {
            query: MUTATION,
            variables: Variables {
                input: ActivityInput {
                    agent_session_id: session_id,
                    content: ActivityContent {
                        thought: ThoughtContent { body },
                    },
                },
            },
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send emit_thought request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Linear API emit_thought failed");
        }

        Ok(())
    }

    /// Get agent session activities (for polling user input).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails or the response is invalid.
    pub async fn get_session_activities(&self, session_id: &str) -> Result<Vec<SessionActivity>> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
        }

        #[derive(Serialize)]
        struct Request<'a> {
            query: &'static str,
            variables: Variables<'a>,
        }

        #[derive(Deserialize)]
        struct Response {
            data: Option<ResponseData>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ResponseData {
            agent_session: Option<AgentSession>,
        }

        #[derive(Deserialize)]
        struct AgentSession {
            activities: ActivitiesConnection,
        }

        #[derive(Deserialize)]
        struct ActivitiesConnection {
            nodes: Vec<SessionActivity>,
        }

        const QUERY: &str = r"
            query GetSessionActivities($id: String!) {
                agentSession(id: $id) {
                    activities(first: 50, orderBy: createdAt) {
                        nodes {
                            id
                            createdAt
                            content
                        }
                    }
                }
            }
        ";

        let request = Request {
            query: QUERY,
            variables: Variables { id: session_id },
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to get session activities")?;

        let response: Response = response
            .json()
            .await
            .context("Failed to parse session activities response")?;

        Ok(response
            .data
            .and_then(|d| d.agent_session)
            .map(|s| s.activities.nodes)
            .unwrap_or_default())
    }
}

/// Activity from Linear session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionActivity {
    pub id: String,
    pub created_at: String,
    pub content: serde_json::Value,
}

impl SessionActivity {
    /// Check if this is a user message.
    #[must_use]
    pub fn is_user_message(&self) -> bool {
        self.content.get("userMessage").is_some()
    }

    /// Get the user message body if this is a user message.
    #[must_use]
    pub fn user_message_body(&self) -> Option<&str> {
        self.content
            .get("userMessage")
            .and_then(|m| m.get("body"))
            .and_then(|b| b.as_str())
    }
}

// =============================================================================
// Status Sync (Existing functionality)
// =============================================================================

/// Status file content structure.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TaskStatus {
    pub status: String,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub progress: Option<u8>,
    #[serde(default)]
    pub activity: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Payload sent to Linear service.
#[derive(Debug, Clone, Serialize)]
pub struct StatusUpdate {
    pub linear_session_id: String,
    pub linear_issue_id: String,
    pub linear_team_id: String,
    pub status: TaskStatus,
    pub workflow_name: String,
}

/// Status sync task - monitors status file and sends updates.
async fn status_sync_task(config: Arc<Config>, http_client: reqwest::Client) {
    let mut last_status: Option<TaskStatus> = None;

    loop {
        if let Ok(content) = fs::read_to_string(&config.status_file).await {
            if let Ok(status) = serde_json::from_str::<TaskStatus>(&content) {
                if Some(&status) != last_status.as_ref() {
                    info!(
                        status = %status.status,
                        stage = ?status.stage,
                        progress = ?status.progress,
                        "Status changed, sending update"
                    );

                    let update = StatusUpdate {
                        linear_session_id: config.linear_session_id.clone(),
                        linear_issue_id: config.linear_issue_id.clone(),
                        linear_team_id: config.linear_team_id.clone(),
                        status: status.clone(),
                        workflow_name: config.workflow_name.clone(),
                    };

                    let url = format!("{}/status/linear-sync", config.linear_service_url);
                    match http_client
                        .post(&url)
                        .json(&update)
                        .timeout(Duration::from_secs(10))
                        .send()
                        .await
                    {
                        Ok(resp) if resp.status().is_success() => {
                            debug!("Status update sent successfully");
                            last_status = Some(status);
                        }
                        Ok(resp) => {
                            warn!(status = %resp.status(), "Status update failed");
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to send status update");
                        }
                    }
                }
            }
        }

        sleep(Duration::from_millis(config.status_poll_interval_ms)).await;
    }
}

// =============================================================================
// Log Streaming
// =============================================================================

/// Strip ANSI escape codes from a string.
fn strip_ansi_codes(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(s, "").to_string()
}

/// Log streaming task - tails log file and posts to Linear.
async fn log_stream_task(config: Arc<Config>, linear_client: Option<LinearApiClient>) {
    let Some(client) = linear_client else {
        info!("Linear API not configured, log streaming disabled");
        return;
    };

    // Wait for log file to exist
    loop {
        if fs::metadata(&config.log_file).await.is_ok() {
            break;
        }
        debug!(path = %config.log_file, "Waiting for log file to exist");
        sleep(Duration::from_secs(2)).await;
    }

    info!(path = %config.log_file, "Starting log file streaming");

    // Open file and seek to end
    let file = match File::open(&config.log_file).await {
        Ok(f) => f,
        Err(e) => {
            error!(error = %e, "Failed to open log file");
            return;
        }
    };

    let mut reader = BufReader::new(file);
    if let Err(e) = reader.seek(SeekFrom::End(0)).await {
        warn!(error = %e, "Failed to seek to end of log file");
    }

    let mut buffer = String::new();
    let mut last_post = Instant::now();
    let post_interval = Duration::from_millis(config.log_post_interval_ms);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // No new data, check if we should post buffered content
                if !buffer.is_empty() && last_post.elapsed() >= post_interval {
                    let cleaned = strip_ansi_codes(&buffer);
                    if !cleaned.trim().is_empty() {
                        if let Err(e) = client
                            .emit_thought(&config.linear_session_id, &cleaned)
                            .await
                        {
                            warn!(error = %e, "Failed to emit log thought");
                        } else {
                            debug!(len = cleaned.len(), "Posted log buffer to Linear");
                        }
                    }
                    buffer.clear();
                    last_post = Instant::now();
                }
                sleep(Duration::from_millis(100)).await;
            }
            Ok(_) => {
                buffer.push_str(&line);

                // Post immediately on important events or when buffer is large
                let should_post = line.contains("✓")
                    || line.contains("✗")
                    || line.contains("ERROR")
                    || line.contains("error:")
                    || line.contains("WARN")
                    || buffer.len() > 4096
                    || last_post.elapsed() >= post_interval;

                if should_post && !buffer.is_empty() {
                    let cleaned = strip_ansi_codes(&buffer);
                    if !cleaned.trim().is_empty() {
                        if let Err(e) = client
                            .emit_thought(&config.linear_session_id, &cleaned)
                            .await
                        {
                            warn!(error = %e, "Failed to emit log thought");
                        } else {
                            debug!(len = cleaned.len(), "Posted log buffer to Linear");
                        }
                    }
                    buffer.clear();
                    last_post = Instant::now();
                }
            }
            Err(e) => {
                warn!(error = %e, "Error reading log file");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// =============================================================================
// Input Polling
// =============================================================================

/// Input polling task - polls Linear for user messages and writes to FIFO.
async fn input_poll_task(
    config: Arc<Config>,
    linear_client: Option<LinearApiClient>,
    fifo_tx: mpsc::Sender<String>,
) {
    let Some(client) = linear_client else {
        info!("Linear API not configured, input polling disabled");
        return;
    };

    let mut seen_activity_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Initial fetch to populate seen IDs (don't forward old messages)
    match client
        .get_session_activities(&config.linear_session_id)
        .await
    {
        Ok(activities) => {
            for activity in activities {
                seen_activity_ids.insert(activity.id);
            }
            info!(
                count = seen_activity_ids.len(),
                "Initialized with existing activities"
            );
        }
        Err(e) => {
            warn!(error = %e, "Failed to get initial activities");
        }
    }

    loop {
        sleep(Duration::from_millis(config.input_poll_interval_ms)).await;

        match client
            .get_session_activities(&config.linear_session_id)
            .await
        {
            Ok(activities) => {
                for activity in activities {
                    if seen_activity_ids.contains(&activity.id) {
                        continue;
                    }

                    seen_activity_ids.insert(activity.id.clone());

                    if activity.is_user_message() {
                        if let Some(body) = activity.user_message_body() {
                            info!(activity_id = %activity.id, "Received user message from Linear");

                            // Format as Claude input JSON
                            let input = serde_json::json!({
                                "type": "user",
                                "message": {
                                    "role": "user",
                                    "content": [{"type": "text", "text": body}]
                                }
                            });

                            if let Ok(json) = serde_json::to_string(&input) {
                                if fifo_tx.send(json).await.is_err() {
                                    warn!("FIFO channel closed");
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!(error = %e, "Failed to poll activities");
            }
        }
    }
}

/// Check if a path is a FIFO (named pipe).
#[cfg(unix)]
async fn is_fifo(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path).await {
        metadata.file_type().is_fifo()
    } else {
        false
    }
}

/// Check if a path is a FIFO (named pipe) - non-Unix fallback.
#[cfg(not(unix))]
async fn is_fifo(path: &str) -> bool {
    // On non-Unix, just check if the file exists
    fs::metadata(path).await.is_ok()
}

/// FIFO writer task - writes messages to the agent input FIFO.
async fn fifo_writer_task(config: Arc<Config>, mut fifo_rx: mpsc::Receiver<String>) {
    loop {
        // Wait for FIFO to exist
        loop {
            if is_fifo(&config.input_fifo).await {
                break;
            }
            debug!(path = %config.input_fifo, "Waiting for FIFO");
            sleep(Duration::from_secs(1)).await;
        }

        info!(path = %config.input_fifo, "FIFO available, ready to write");

        while let Some(message) = fifo_rx.recv().await {
            match OpenOptions::new()
                .write(true)
                .open(&config.input_fifo)
                .await
            {
                Ok(mut file) => {
                    let line = format!("{message}\n");
                    if let Err(e) = file.write_all(line.as_bytes()).await {
                        warn!(error = %e, "Failed to write to FIFO");
                    } else {
                        info!("Wrote message to agent FIFO");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to open FIFO for writing");
                }
            }
        }
    }
}

// =============================================================================
// HTTP Server
// =============================================================================

/// Shared state for HTTP handlers.
#[derive(Clone)]
struct AppState {
    fifo_tx: mpsc::Sender<String>,
    shutdown: Arc<AtomicBool>,
}

/// Input request body.
#[derive(Debug, Deserialize)]
struct InputRequest {
    text: String,
}

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Input endpoint - accepts text and forwards to FIFO.
async fn handle_input(
    State(state): State<AppState>,
    Json(payload): Json<InputRequest>,
) -> impl IntoResponse {
    info!(len = payload.text.len(), "Received input via HTTP");

    // Format as Claude input JSON
    let input = serde_json::json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [{"type": "text", "text": payload.text}]
        }
    });

    match serde_json::to_string(&input) {
        Ok(json) => {
            if state.fifo_tx.send(json).await.is_ok() {
                (StatusCode::OK, "Message queued")
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "FIFO channel closed")
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "JSON serialization failed",
        ),
    }
}

/// Shutdown endpoint - triggers graceful shutdown.
async fn handle_shutdown(State(state): State<AppState>) -> impl IntoResponse {
    info!("Shutdown requested via HTTP");
    state.shutdown.store(true, Ordering::SeqCst);
    (StatusCode::OK, "Shutting down")
}

/// HTTP server task.
async fn http_server_task(config: Arc<Config>, state: AppState) {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/input", post(handle_input))
        .route("/shutdown", post(handle_shutdown))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.http_port);
    info!(addr = %addr, "Starting HTTP server");

    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                error!(error = %e, "HTTP server error");
            }
        }
        Err(e) => {
            error!(error = %e, addr = %addr, "Failed to bind HTTP server");
        }
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
#[allow(clippy::too_many_lines)] // Main orchestrates multiple async tasks
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    info!("Starting Linear sidecar v2...");

    let config = Arc::new(Config::from_env());

    // Log configuration
    info!(
        session_id = %config.linear_session_id,
        has_api = config.has_linear_api(),
        status_file = %config.status_file,
        log_file = %config.log_file,
        input_fifo = %config.input_fifo,
        http_port = config.http_port,
        "Sidecar configured"
    );

    // Skip if no Linear session configured
    if !config.has_linear_session() {
        info!("No Linear session configured, running in minimal mode (HTTP only)");
    }

    // Create Linear API client if token available
    let linear_client = config.linear_oauth_token.as_ref().and_then(|token| {
        match LinearApiClient::new(token, &config.linear_api_url) {
            Ok(client) => {
                info!("Linear API client initialized for sidecar");
                Some(client)
            }
            Err(e) => {
                warn!(error = %e, "Failed to create Linear API client, Linear API features disabled");
                None
            }
        }
    });

    // Create channels and shared state
    let (fifo_tx, fifo_rx) = mpsc::channel::<String>(100);
    let shutdown = Arc::new(AtomicBool::new(false));

    let http_state = AppState {
        fifo_tx: fifo_tx.clone(),
        shutdown: shutdown.clone(),
    };

    // Create HTTP client for status sync
    let http_client = reqwest::Client::new();

    // Spawn all tasks
    let config_clone = config.clone();
    let status_handle = tokio::spawn(async move {
        if config_clone.has_linear_session() {
            status_sync_task(config_clone, http_client).await;
        }
    });

    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let log_handle = tokio::spawn(async move {
        if config_clone.has_linear_api() {
            log_stream_task(config_clone, linear_client_clone).await;
        }
    });

    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let input_handle = tokio::spawn(async move {
        if config_clone.has_linear_api() {
            input_poll_task(config_clone, linear_client_clone, fifo_tx).await;
        }
    });

    let config_clone = config.clone();
    let fifo_handle = tokio::spawn(async move {
        fifo_writer_task(config_clone, fifo_rx).await;
    });

    let config_clone = config.clone();
    let http_handle = tokio::spawn(async move {
        http_server_task(config_clone, http_state).await;
    });

    // Wait for shutdown signal or any task to complete
    tokio::select! {
        () = async { tokio::signal::ctrl_c().await.ok(); } => {
            info!("Received SIGINT, shutting down");
        }
        () = async {
            while !shutdown.load(Ordering::SeqCst) {
                sleep(Duration::from_millis(100)).await;
            }
        } => {
            info!("Shutdown flag set, shutting down");
        }
        result = status_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Status sync task panicked");
            }
            warn!("Status sync task exited");
        }
        result = log_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Log stream task panicked");
            }
            warn!("Log stream task exited");
        }
        result = input_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Input poll task panicked");
            }
            warn!("Input poll task exited");
        }
        result = fifo_handle => {
            if let Err(e) = result {
                warn!(error = %e, "FIFO writer task panicked");
            }
            warn!("FIFO writer task exited");
        }
        result = http_handle => {
            if let Err(e) = result {
                warn!(error = %e, "HTTP server panicked");
            }
            warn!("HTTP server exited");
        }
    }

    info!("Sidecar shutdown complete");
    Ok(())
}
