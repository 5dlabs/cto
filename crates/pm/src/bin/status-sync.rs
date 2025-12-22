//! Enhanced sidecar for Linear integration with 2-way communication.
//!
//! This sidecar runs alongside agent pods and provides:
//! - Status file monitoring and sync to Linear service
//! - Log file streaming to Linear agent dialog (`emit_thought`)
//! - Input polling from Linear and forwarding to agent FIFO
//! - HTTP server for local input injection
//! - **Whip Cracking**: Progress monitoring with escalating nudges when agents stall

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

    // Argo workflow external URL (shown in Linear UI)
    pub argo_workflow_url: Option<String>,

    // Task info for plan updates
    pub task_id: Option<String>,
    pub task_description: Option<String>,

    // File paths
    pub status_file: String,
    pub log_file: String,
    pub input_fifo: String,
    pub claude_stream_file: String,

    // Service URLs
    pub linear_service_url: String,
    pub linear_api_url: String,

    // Intervals
    pub status_poll_interval_ms: u64,
    pub log_post_interval_ms: u64,
    pub input_poll_interval_ms: u64,
    pub stream_poll_interval_ms: u64,

    // HTTP server
    pub http_port: u16,

    // Whip cracking (progress monitoring with escalating nudges)
    pub whip_crack_enabled: bool,
    pub stall_threshold_secs: u64,
    pub nudge_interval_secs: u64,
    pub max_nudge_level: u8,
    pub nudge_messages: Vec<String>,
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

            // Argo workflow URL for external link in Linear
            argo_workflow_url: std::env::var("ARGO_WORKFLOW_URL")
                .ok()
                .filter(|s| !s.is_empty()),

            // Task info for plan updates
            task_id: std::env::var("TASK_ID").ok().filter(|s| !s.is_empty()),
            task_description: std::env::var("TASK_DESCRIPTION")
                .ok()
                .filter(|s| !s.is_empty()),

            status_file: std::env::var("STATUS_FILE")
                .unwrap_or_else(|_| "/workspace/status.json".to_string()),
            log_file: std::env::var("LOG_FILE_PATH")
                .unwrap_or_else(|_| "/workspace/agent.log".to_string()),
            input_fifo: std::env::var("INPUT_FIFO_PATH")
                .unwrap_or_else(|_| "/workspace/agent-input.jsonl".to_string()),
            claude_stream_file: std::env::var("CLAUDE_STREAM_FILE")
                .unwrap_or_else(|_| "/workspace/claude-stream.jsonl".to_string()),

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
            stream_poll_interval_ms: std::env::var("STREAM_POLL_INTERVAL_MS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .unwrap_or(500),

            http_port: std::env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),

            // Whip cracking configuration
            whip_crack_enabled: std::env::var("WHIP_CRACK_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            stall_threshold_secs: std::env::var("STALL_THRESHOLD_SECS")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap_or(120),
            nudge_interval_secs: std::env::var("NUDGE_INTERVAL_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            max_nudge_level: std::env::var("MAX_NUDGE_LEVEL")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            nudge_messages: Self::parse_nudge_messages(),
        }
    }

    /// Parse nudge messages from environment or use defaults.
    fn parse_nudge_messages() -> Vec<String> {
        if let Ok(json) = std::env::var("NUDGE_MESSAGES") {
            if let Ok(messages) = serde_json::from_str::<Vec<String>>(&json) {
                if !messages.is_empty() {
                    return messages;
                }
            }
        }

        // Default escalating messages (based on "whip cracking" concept)
        vec![
            "📊 Checking in - how's progress? Let me know if you need clarification on the task."
                .to_string(),
            "⏰ I notice things have slowed down. Please focus on completing the current step. What's blocking you?"
                .to_string(),
            "⚠️ FOCUS: Stop exploring and execute the next concrete action NOW. We need results, not investigation."
                .to_string(),
            "🚨 CRITICAL: You appear stuck. Complete the current task immediately or report what's blocking you. Time is limited."
                .to_string(),
        ]
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

/// Activity content types for Linear Agent API.
/// Follows Linear's agent activity specification.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum LinearActivityContent {
    /// A thought or internal note
    Thought { thought: ThoughtBody },
    /// A tool invocation or action
    Action { action: ActionBody },
    /// Request for user input
    Elicitation { elicitation: ThoughtBody },
    /// Final response/completion
    Response { response: ThoughtBody },
    /// Error report
    Error { error: ThoughtBody },
}

#[derive(Debug, Clone, Serialize)]
pub struct ThoughtBody {
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionBody {
    pub action: String,
    pub parameter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

/// Agent plan step status (per Linear API)
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Canceled,
}

/// Agent plan step
#[derive(Debug, Clone, Serialize)]
pub struct PlanStep {
    pub content: String,
    pub status: PlanStepStatus,
}

/// Lightweight Linear API client for sidecar operations.
/// Implements the full Linear Agent Activity API.
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

    /// Generic activity emission helper
    async fn emit_activity(
        &self,
        session_id: &str,
        content: LinearActivityContent,
        ephemeral: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Variables {
            input: ActivityInput,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ActivityInput {
            agent_session_id: String,
            content: LinearActivityContent,
            #[serde(skip_serializing_if = "Option::is_none")]
            ephemeral: Option<bool>,
        }

        #[derive(Serialize)]
        struct Request {
            query: &'static str,
            variables: Variables,
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
                    agent_session_id: session_id.to_string(),
                    content,
                    ephemeral: if ephemeral { Some(true) } else { None },
                },
            },
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send activity request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Linear API activity emission failed");
        }

        Ok(())
    }

    /// Emit a thought to the Linear agent session.
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_thought(&self, session_id: &str, body: &str) -> Result<()> {
        self.emit_activity(
            session_id,
            LinearActivityContent::Thought {
                thought: ThoughtBody {
                    body: body.to_string(),
                },
            },
            false,
        )
        .await
    }

    /// Emit an ephemeral thought (replaced by next activity).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_ephemeral_thought(&self, session_id: &str, body: &str) -> Result<()> {
        self.emit_activity(
            session_id,
            LinearActivityContent::Thought {
                thought: ThoughtBody {
                    body: body.to_string(),
                },
            },
            true,
        )
        .await
    }

    /// Emit an action activity (tool invocation in progress).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_action(&self, session_id: &str, action: &str, parameter: &str) -> Result<()> {
        self.emit_activity(
            session_id,
            LinearActivityContent::Action {
                action: ActionBody {
                    action: action.to_string(),
                    parameter: parameter.to_string(),
                    result: None,
                },
            },
            false,
        )
        .await
    }

    /// Emit an action activity with result (completed tool call).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_action_complete(
        &self,
        session_id: &str,
        action: &str,
        parameter: &str,
        result: &str,
    ) -> Result<()> {
        self.emit_activity(
            session_id,
            LinearActivityContent::Action {
                action: ActionBody {
                    action: action.to_string(),
                    parameter: parameter.to_string(),
                    result: Some(result.to_string()),
                },
            },
            false,
        )
        .await
    }

    /// Emit an error activity.
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_error(&self, session_id: &str, body: &str) -> Result<()> {
        self.emit_activity(
            session_id,
            LinearActivityContent::Error {
                error: ThoughtBody {
                    body: body.to_string(),
                },
            },
            false,
        )
        .await
    }

    /// Emit a response activity (final completion).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn emit_response(&self, session_id: &str, body: &str) -> Result<()> {
        self.emit_activity(
            session_id,
            LinearActivityContent::Response {
                response: ThoughtBody {
                    body: body.to_string(),
                },
            },
            false,
        )
        .await
    }

    /// Update the agent session plan (visual checklist in Linear UI).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn update_plan(&self, session_id: &str, steps: &[PlanStep]) -> Result<()> {
        #[derive(Serialize)]
        struct Variables {
            id: String,
            input: SessionUpdateInput,
        }

        #[derive(Serialize)]
        struct SessionUpdateInput {
            plan: Vec<PlanStep>,
        }

        #[derive(Serialize)]
        struct Request {
            query: &'static str,
            variables: Variables,
        }

        const MUTATION: &str = r"
            mutation UpdatePlan($id: String!, $input: AgentSessionUpdateInput!) {
                agentSessionUpdate(id: $id, input: $input) {
                    success
                }
            }
        ";

        let request = Request {
            query: MUTATION,
            variables: Variables {
                id: session_id.to_string(),
                input: SessionUpdateInput {
                    plan: steps.to_vec(),
                },
            },
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to update plan")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Linear API plan update failed");
        }

        Ok(())
    }

    /// Set the session external URL (links to Argo workflow in Linear UI).
    ///
    /// # Errors
    ///
    /// Returns an error if the GraphQL request fails.
    pub async fn set_external_url(&self, session_id: &str, url: &str) -> Result<()> {
        #[derive(Serialize)]
        struct Variables {
            id: String,
            input: SessionUpdateInput,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SessionUpdateInput {
            external_url: String,
        }

        #[derive(Serialize)]
        struct Request {
            query: &'static str,
            variables: Variables,
        }

        const MUTATION: &str = r"
            mutation SetExternalUrl($id: String!, $input: AgentSessionUpdateInput!) {
                agentSessionUpdate(id: $id, input: $input) {
                    success
                }
            }
        ";

        let request = Request {
            query: MUTATION,
            variables: Variables {
                id: session_id.to_string(),
                input: SessionUpdateInput {
                    external_url: url.to_string(),
                },
            },
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to set external URL")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Linear API set external URL failed");
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
// Claude Stream Parsing
// =============================================================================

/// Truncate a string to a maximum number of characters (Unicode-safe).
/// Avoids panics from byte-level slicing on multi-byte UTF-8 characters.
fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}

/// Claude stream event types from stream-json output.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeStreamEvent {
    /// System initialization event
    System {
        subtype: Option<String>,
        model: Option<String>,
        tools: Option<Vec<String>>,
        session_id: Option<String>,
    },
    /// Assistant message (may contain text or `tool_use`)
    Assistant {
        message: Option<AssistantMessage>,
        session_id: Option<String>,
    },
    /// User message (usually tool results)
    User {
        message: Option<UserMessage>,
        tool_use_result: Option<String>,
        session_id: Option<String>,
    },
    /// Final result with stats
    Result {
        subtype: Option<String>,
        duration_ms: Option<u64>,
        total_cost_usd: Option<f64>,
        num_turns: Option<u32>,
        result: Option<String>,
        session_id: Option<String>,
    },
}

/// Assistant message content
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessage {
    pub model: Option<String>,
    pub content: Option<Vec<ContentBlock>>,
    pub usage: Option<UsageInfo>,
}

/// User message content
#[derive(Debug, Clone, Deserialize)]
pub struct UserMessage {
    pub content: Option<Vec<ContentBlock>>,
}

/// Content block (text or `tool_use`)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: Option<String>,
        name: String,
        input: Option<serde_json::Value>,
    },
    ToolResult {
        tool_use_id: Option<String>,
        content: Option<String>,
        is_error: Option<bool>,
    },
}

/// Usage information
#[derive(Debug, Clone, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// Claude stream parsing task - reads stream-json and emits structured activities.
/// Maps Claude CLI output to proper Linear Agent API activity types.
async fn claude_stream_task(config: Arc<Config>, linear_client: Option<LinearApiClient>) {
    let Some(client) = linear_client else {
        info!("Linear API not configured, Claude stream parsing disabled");
        return;
    };

    // Wait for stream file to exist
    loop {
        if fs::metadata(&config.claude_stream_file).await.is_ok() {
            break;
        }
        debug!(path = %config.claude_stream_file, "Waiting for Claude stream file to exist");
        sleep(Duration::from_secs(2)).await;
    }

    info!(path = %config.claude_stream_file, "Starting Claude stream parsing");

    // Open file and track position
    let file = match File::open(&config.claude_stream_file).await {
        Ok(f) => f,
        Err(e) => {
            error!(error = %e, "Failed to open Claude stream file");
            return;
        }
    };

    let mut reader = BufReader::new(file);
    let mut processed_lines = 0u64;
    let mut tool_state = ToolState::default();
    let mut total_cost: f64 = 0.0;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // No new data
                sleep(Duration::from_millis(config.stream_poll_interval_ms)).await;
            }
            Ok(_) => {
                processed_lines += 1;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Parse the JSON event
                match serde_json::from_str::<ClaudeStreamEvent>(line) {
                    Ok(event) => {
                        if let Err(e) = process_stream_event(
                            &client,
                            &config.linear_session_id,
                            &event,
                            &mut tool_state,
                            &mut total_cost,
                        )
                        .await
                        {
                            warn!(error = %e, line = processed_lines, "Failed to process stream event");
                        }
                    }
                    Err(e) => {
                        debug!(error = %e, line = %line, "Failed to parse stream event (may be partial)");
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Error reading Claude stream file");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

/// Track tool invocation state for proper action activities
#[derive(Default)]
struct ToolState {
    current_tool: Option<String>,
    current_input: Option<String>,
}

/// Process a single Claude stream event and emit appropriate Linear activity.
/// Maps Claude events to proper Linear Agent API activity types:
/// - Tool invocations → `action` activities
/// - Tool completions → `action` with result
/// - Errors → `error` activities
/// - Completion → `response` activities
/// - Transient status → ephemeral `thought` activities
#[allow(clippy::too_many_lines)]
async fn process_stream_event(
    client: &LinearApiClient,
    session_id: &str,
    event: &ClaudeStreamEvent,
    tool_state: &mut ToolState,
    total_cost: &mut f64,
) -> Result<()> {
    match event {
        ClaudeStreamEvent::System { model, tools, .. } => {
            // System init - emit as thought (it's informational, not an action)
            let tool_count = tools.as_ref().map_or(0, Vec::len);
            let model_name = model.as_deref().unwrap_or("unknown");
            let msg = format!("🚀 Starting with **{model_name}** | {tool_count} tools available");
            client.emit_thought(session_id, &msg).await?;
        }

        ClaudeStreamEvent::Assistant { message, .. } => {
            if let Some(msg) = message {
                for content in msg.content.as_ref().unwrap_or(&vec![]) {
                    match content {
                        ContentBlock::ToolUse { name, input, .. } => {
                            // Tool invocation → emit as ACTION (not thought)
                            let input_summary = input
                                .as_ref()
                                .map_or_else(String::new, |v| truncate_chars(&v.to_string(), 150));

                            // Store state for pairing with result
                            tool_state.current_tool = Some(name.clone());
                            tool_state.current_input = Some(input_summary.clone());

                            // Emit action activity (tool in progress)
                            client.emit_action(session_id, name, &input_summary).await?;
                        }
                        ContentBlock::Text { text } => {
                            // Significant text → thought activity
                            // Skip boilerplate phrases
                            if text.chars().count() > 50
                                && !text.starts_with("I'll")
                                && !text.starts_with("Let me")
                                && !text.starts_with("Now I")
                            {
                                let display_text = truncate_chars(text, 500);
                                client.emit_thought(session_id, &display_text).await?;
                            }
                        }
                        ContentBlock::ToolResult { .. } => {}
                    }
                }
            }
        }

        ClaudeStreamEvent::User {
            tool_use_result,
            message,
            ..
        } => {
            // Helper function to format result
            fn format_result(is_error: bool, result_text: &str) -> String {
                let status_emoji = if is_error { "❌" } else { "✅" };
                let result_preview = truncate_chars(result_text, 200);
                format!("{status_emoji} {result_preview}")
            }

            // Get tool info before consuming state
            let tool_name = tool_state
                .current_tool
                .take()
                .unwrap_or_else(|| "Tool".to_string());
            let tool_input = tool_state.current_input.take().unwrap_or_default();

            if let Some(result) = tool_use_result {
                let is_error = result.contains("error") || result.contains("Error");
                let result_str = format_result(is_error, result);

                if is_error {
                    // Error result → emit as ERROR activity
                    let msg = format!("**{tool_name}** failed: {result_str}");
                    client.emit_error(session_id, &msg).await?;
                } else {
                    // Success → emit ACTION with result
                    client
                        .emit_action_complete(session_id, &tool_name, &tool_input, &result_str)
                        .await?;
                }
            } else if let Some(msg) = message {
                for content in msg.content.as_ref().unwrap_or(&vec![]) {
                    if let ContentBlock::ToolResult {
                        content, is_error, ..
                    } = content
                    {
                        let result_text = content.as_deref().unwrap_or("completed");
                        let error = is_error.unwrap_or(false);
                        let result_str = format_result(error, result_text);

                        if error {
                            let msg = format!("**{tool_name}** failed: {result_str}");
                            client.emit_error(session_id, &msg).await?;
                        } else {
                            client
                                .emit_action_complete(
                                    session_id,
                                    &tool_name,
                                    &tool_input,
                                    &result_str,
                                )
                                .await?;
                        }
                    }
                }
            }
        }

        ClaudeStreamEvent::Result {
            duration_ms,
            total_cost_usd,
            num_turns,
            subtype,
            ..
        } => {
            *total_cost += total_cost_usd.unwrap_or(0.0);
            #[allow(clippy::cast_precision_loss)]
            let duration_secs = duration_ms.map(|ms| ms as f64 / 1000.0).unwrap_or(0.0);
            let turns = num_turns.unwrap_or(0);

            let is_error = subtype.as_deref() == Some("error");

            // Completion → emit as RESPONSE (final activity) or ERROR
            let summary = format!(
                "**Completed** | {duration_secs:.1}s | ${:.4} | {turns} turns",
                *total_cost
            );

            if is_error {
                client.emit_error(session_id, &summary).await?;
            } else {
                client.emit_response(session_id, &summary).await?;
            }
        }
    }

    Ok(())
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
// Whip Cracking (Progress Monitoring)
// =============================================================================

/// Progress monitor task - detects when agents stall and sends escalating nudges.
///
/// This implements the "whip cracking" pattern where an orchestrator monitors
/// a worker agent's progress and sends increasingly urgent prompts when the
/// agent appears stuck or slow.
///
/// The escalation levels are:
/// 0. Gentle check-in after initial stall threshold
/// 1. Firm reminder to focus
/// 2. Sharp directive to execute NOW
/// 3. Critical warning with timeout threat
async fn progress_monitor_task(
    config: Arc<Config>,
    fifo_tx: mpsc::Sender<String>,
    shutdown: Arc<AtomicBool>,
    linear_client: Option<LinearApiClient>,
) {
    if !config.whip_crack_enabled {
        info!("Whip cracking disabled, progress monitor not starting");
        return;
    }

    info!(
        stall_threshold = config.stall_threshold_secs,
        nudge_interval = config.nudge_interval_secs,
        max_level = config.max_nudge_level,
        "🔥 Whip cracker starting - monitoring agent progress"
    );

    let mut last_activity_time = Instant::now();
    let mut last_file_size: u64 = 0;
    let mut current_nudge_level: u8 = 0;
    let mut nudge_sent_at: Option<Instant> = None;

    let stall_threshold = Duration::from_secs(config.stall_threshold_secs);
    let nudge_interval = Duration::from_secs(config.nudge_interval_secs);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            info!("Whip cracker shutting down");
            break;
        }

        sleep(Duration::from_secs(10)).await;

        // Check if stream file has new content (indicates agent activity)
        let has_activity = match fs::metadata(&config.claude_stream_file).await {
            Ok(metadata) => {
                let current_size = metadata.len();
                if current_size > last_file_size {
                    last_file_size = current_size;
                    true
                } else {
                    false
                }
            }
            Err(_) => {
                // File doesn't exist yet, agent might not have started
                continue;
            }
        };

        if has_activity {
            // Agent is active - reset tracking
            if current_nudge_level > 0 {
                info!(
                    previous_level = current_nudge_level,
                    "✅ Agent activity detected, resetting nudge level"
                );

                // Optionally emit a positive acknowledgment
                if let Some(ref client) = linear_client {
                    let _ = client
                        .emit_ephemeral_thought(
                            &config.linear_session_id,
                            "✅ Progress detected, continuing to monitor",
                        )
                        .await;
                }
            }
            last_activity_time = Instant::now();
            current_nudge_level = 0;
            nudge_sent_at = None;
            continue;
        }

        // Check if we've exceeded the stall threshold
        let time_since_activity = last_activity_time.elapsed();
        if time_since_activity < stall_threshold {
            continue;
        }

        // Check if enough time has passed since last nudge
        if let Some(last_nudge) = nudge_sent_at {
            if last_nudge.elapsed() < nudge_interval {
                continue;
            }
        }

        // Don't exceed max nudge level
        if current_nudge_level > config.max_nudge_level {
            debug!("Max nudge level reached, not sending more nudges");
            continue;
        }

        // Get the appropriate nudge message
        let nudge_idx = current_nudge_level as usize;
        let nudge_message = config
            .nudge_messages
            .get(nudge_idx)
            .cloned()
            .unwrap_or_else(|| {
                format!(
                    "⚠️ Agent appears stalled (level {}). Please continue with the task.",
                    current_nudge_level
                )
            });

        warn!(
            level = current_nudge_level,
            stall_secs = time_since_activity.as_secs(),
            "🔥 WHIP CRACK: Agent stalled, sending nudge"
        );

        // Format as Claude input JSON
        let input = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [{"type": "text", "text": nudge_message}]
            }
        });

        if let Ok(json) = serde_json::to_string(&input) {
            if fifo_tx.send(json).await.is_err() {
                warn!("FIFO channel closed, stopping progress monitor");
                break;
            }
        }

        // Also emit to Linear so user can see the nudge
        if let Some(ref client) = linear_client {
            let linear_msg = format!(
                "🔥 **Nudge (level {})**: {}",
                current_nudge_level, nudge_message
            );
            let _ = client
                .emit_thought(&config.linear_session_id, &linear_msg)
                .await;
        }

        // Escalate for next time
        current_nudge_level = current_nudge_level.saturating_add(1);
        nudge_sent_at = Some(Instant::now());

        info!(
            next_level = current_nudge_level,
            "Nudge sent, escalated to next level"
        );
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
        claude_stream_file = %config.claude_stream_file,
        input_fifo = %config.input_fifo,
        http_port = config.http_port,
        argo_url = ?config.argo_workflow_url,
        task_id = ?config.task_id,
        whip_crack_enabled = config.whip_crack_enabled,
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

    // Initialize session with external URL and plan (if Linear API available)
    if let Some(ref client) = linear_client {
        // Set external URL to Argo workflow
        if let Some(ref url) = config.argo_workflow_url {
            info!(url = %url, "Setting session external URL to Argo workflow");
            if let Err(e) = client
                .set_external_url(&config.linear_session_id, url)
                .await
            {
                warn!(error = %e, "Failed to set session external URL");
            }
        }

        // Set initial plan based on task
        let task_desc = config
            .task_description
            .as_deref()
            .unwrap_or("Implementing task");
        let initial_plan = vec![
            PlanStep {
                content: format!("📋 {task_desc}"),
                status: PlanStepStatus::InProgress,
            },
            PlanStep {
                content: "🔧 Execute implementation".to_string(),
                status: PlanStepStatus::Pending,
            },
            PlanStep {
                content: "✅ Verify acceptance criteria".to_string(),
                status: PlanStepStatus::Pending,
            },
            PlanStep {
                content: "📤 Create pull request".to_string(),
                status: PlanStepStatus::Pending,
            },
        ];

        info!(steps = initial_plan.len(), "Setting initial agent plan");
        if let Err(e) = client
            .update_plan(&config.linear_session_id, &initial_plan)
            .await
        {
            warn!(error = %e, "Failed to set initial plan");
        }

        // Emit initial thought to indicate sidecar is active
        let init_msg = format!("🚀 Sidecar connected | Workflow: {}", config.workflow_name);
        if let Err(e) = client
            .emit_ephemeral_thought(&config.linear_session_id, &init_msg)
            .await
        {
            warn!(error = %e, "Failed to emit init thought");
        }
    }

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

    // Claude stream parsing task (structured activities from stream-json)
    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let stream_handle = tokio::spawn(async move {
        if config_clone.has_linear_api() {
            claude_stream_task(config_clone, linear_client_clone).await;
        }
    });

    // Clone fifo_tx for both input polling and progress monitor
    let fifo_tx_for_input = fifo_tx.clone();
    let fifo_tx_for_whip = fifo_tx;

    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let input_handle = tokio::spawn(async move {
        if config_clone.has_linear_api() {
            input_poll_task(config_clone, linear_client_clone, fifo_tx_for_input).await;
        }
    });

    // Whip cracking (progress monitor with escalating nudges)
    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let shutdown_clone = shutdown.clone();
    let whip_handle = tokio::spawn(async move {
        progress_monitor_task(config_clone, fifo_tx_for_whip, shutdown_clone, linear_client_clone)
            .await;
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
        result = stream_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Claude stream task panicked");
            }
            warn!("Claude stream task exited");
        }
        result = input_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Input poll task panicked");
            }
            warn!("Input poll task exited");
        }
        result = whip_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Whip cracker task panicked");
            }
            info!("Whip cracker task exited");
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
