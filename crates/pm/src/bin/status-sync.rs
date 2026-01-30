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
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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
    pub progress_file: String,

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

    // Context engineering: artifact summary injection
    pub artifact_injection_enabled: bool,
    pub artifact_injection_interval_turns: u32,

    // Main container exit detection (for graceful sidecar shutdown)
    pub agent_done_file: String,
    pub main_exit_watch_enabled: bool,
    pub main_exit_watch_interval_ms: u64,
}

impl Config {
    /// Load configuration from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            linear_session_id: std::env::var("LINEAR_SESSION_ID").unwrap_or_default(),
            linear_issue_id: std::env::var("LINEAR_ISSUE_ID").unwrap_or_default(),
            linear_team_id: std::env::var("LINEAR_TEAM_ID").unwrap_or_default(),
            // Check LINEAR_OAUTH_TOKEN first, fall back to LINEAR_API_KEY
            // Note: We filter empty strings BEFORE or_else, because Kubernetes
            // secrets can set an env var to empty string (not missing)
            linear_oauth_token: std::env::var("LINEAR_OAUTH_TOKEN")
                .ok()
                .filter(|s| !s.is_empty())
                .or_else(|| {
                    std::env::var("LINEAR_API_KEY")
                        .ok()
                        .filter(|s| !s.is_empty())
                }),
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
            progress_file: std::env::var("PROGRESS_FILE")
                .unwrap_or_else(|_| "/workspace/progress.jsonl".to_string()),

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

            // Context engineering: artifact injection
            artifact_injection_enabled: std::env::var("ARTIFACT_INJECTION_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true), // Enabled by default
            artifact_injection_interval_turns: std::env::var("ARTIFACT_INJECTION_INTERVAL_TURNS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),

            // Main container exit detection
            agent_done_file: std::env::var("AGENT_DONE_FILE")
                .unwrap_or_else(|_| "/workspace/.agent_done".to_string()),
            main_exit_watch_enabled: std::env::var("MAIN_EXIT_WATCH_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true), // Enabled by default for graceful shutdown
            main_exit_watch_interval_ms: std::env::var("MAIN_EXIT_WATCH_INTERVAL_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
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
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LinearActivityContent {
    /// A thought or internal note
    Thought { body: String },
    /// A tool invocation or action
    Action {
        action: String,
        parameter: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    /// Request for user input
    Elicitation { body: String },
    /// Final response/completion
    Response { body: String },
    /// Error report
    Error { body: String },
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

    /// Resolve an issue identifier (e.g., "CTOPA-123") to a UUID.
    ///
    /// Linear's session creation API now requires the issue UUID, not the identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the issue is not found or the request fails.
    async fn resolve_issue_id(&self, identifier: &str) -> Result<String> {
        #[derive(Serialize)]
        struct Variables {
            identifier: String,
        }

        #[derive(Serialize)]
        struct Request {
            query: &'static str,
            variables: Variables,
        }

        const QUERY: &str = r#"
            query GetIssue($identifier: String!) {
                issue(id: $identifier) {
                    id
                }
            }
        "#;

        let request = Request {
            query: QUERY,
            variables: Variables {
                identifier: identifier.to_string(),
            },
        };

        debug!(identifier = %identifier, "Resolving issue identifier to UUID");

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to resolve issue ID")?;

        let body = response.text().await.unwrap_or_default();
        let json: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse issue resolution response")?;

        if let Some(errors) = json.get("errors") {
            anyhow::bail!("Failed to resolve issue: {}", errors);
        }

        let issue_id = json["data"]["issue"]["id"]
            .as_str()
            .context("Issue not found")?
            .to_string();

        info!(identifier = %identifier, issue_id = %issue_id, "Resolved issue identifier");
        Ok(issue_id)
    }

    /// Create a new agent session on a Linear issue.
    ///
    /// This is used when running in standalone mode (e.g., docker-compose)
    /// where no session is pre-created by the controller.
    ///
    /// # Errors
    ///
    /// Returns an error if the session creation fails.
    pub async fn create_session_on_issue(
        &self,
        issue_identifier: &str,
        model: &str,
        provider: &str,
    ) -> Result<String> {
        #[derive(Serialize)]
        struct Variables {
            input: SessionCreateInput,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SessionCreateInput {
            issue_id: String,
        }

        #[derive(Serialize)]
        struct Request {
            query: &'static str,
            variables: Variables,
        }

        // First, resolve issue identifier to UUID
        let issue_id = self.resolve_issue_id(issue_identifier).await?;

        const MUTATION: &str = r#"
            mutation CreateAgentSession($input: AgentSessionCreateOnIssue!) {
                agentSessionCreateOnIssue(input: $input) {
                    success
                    agentSession {
                        id
                    }
                }
            }
        "#;

        let request = Request {
            query: MUTATION,
            variables: Variables {
                input: SessionCreateInput {
                    issue_id,
                },
            },
        };
        
        // Log model/provider for debugging (no longer sent to Linear)
        debug!(model = %model, provider = %provider, "Session metadata (for reference)");

        info!(
            issue = %issue_identifier,
            model = %model,
            "Creating Linear agent session"
        );

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send session creation request")?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("Session creation failed: {} - {}", status, body);
        }

        let json: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse session creation response")?;

        if let Some(errors) = json.get("errors") {
            anyhow::bail!("GraphQL errors: {}", errors);
        }

        let session_id = json["data"]["agentSessionCreateOnIssue"]["agentSession"]["id"]
            .as_str()
            .context("Missing session ID in response")?
            .to_string();

        info!(session_id = %session_id, "Created Linear agent session");
        Ok(session_id)
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

        debug!(session_id = %session_id, "Emitting Linear activity");
        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send activity request")?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        // Check for GraphQL errors in response body
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(errors) = json.get("errors") {
                warn!(errors = %errors, "Linear GraphQL errors");
            }
            if let Some(data) = json.get("data") {
                if let Some(create) = data.get("agentActivityCreate") {
                    if create.get("success") == Some(&serde_json::Value::Bool(true)) {
                        debug!(session_id = %session_id, "Activity emitted successfully");
                    } else {
                        warn!(response = %body, "Activity creation returned success=false");
                    }
                }
            }
        }

        if !status.is_success() {
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
                body: body.to_string(),
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
                body: body.to_string(),
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
                action: action.to_string(),
                parameter: parameter.to_string(),
                result: None,
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
                action: action.to_string(),
                parameter: parameter.to_string(),
                result: Some(result.to_string()),
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
                body: body.to_string(),
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
                body: body.to_string(),
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
            external_link: String,
        }

        #[derive(Serialize)]
        struct Request {
            query: &'static str,
            variables: Variables,
        }

        const MUTATION: &str = r"
            mutation SetExternalLink($id: String!, $input: AgentSessionUpdateInput!) {
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
                    external_link: url.to_string(),
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

    /// Check if this activity contains a stop signal.
    /// Linear Agent API sends signals via metadata: { "signal": "stop" }
    #[must_use]
    pub fn is_stop_signal(&self) -> bool {
        self.content
            .get("signal")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s == "stop")
    }

    /// Check if this activity contains an auth signal.
    /// Linear Agent API sends auth signals via metadata: { "signal": "auth" }
    #[must_use]
    pub fn is_auth_signal(&self) -> bool {
        self.content
            .get("signal")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s == "auth")
    }

    /// Check if this activity contains a select signal.
    /// Linear Agent API sends select signals via metadata: { "signal": "select" }
    #[must_use]
    pub fn is_select_signal(&self) -> bool {
        self.content
            .get("signal")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s == "select")
    }

    /// Get the signal type if present.
    #[must_use]
    pub fn signal_type(&self) -> Option<&str> {
        self.content.get("signal").and_then(|s| s.as_str())
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

// =============================================================================
// Artifact Trail (Context Engineering)
// =============================================================================

/// Artifact trail for tracking file operations during agent sessions.
///
/// This addresses the "artifact trail problem" identified in context compression research,
/// where file tracking scores 2.2-2.5/5.0 across all compression methods. Explicit tracking
/// ensures agents know which files were created, modified, or read.
///
/// Reference: Agent-Skills-for-Context-Engineering/skills/context-compression
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactTrail {
    /// Files created during this session
    pub files_created: Vec<String>,
    /// Files modified with change summaries (path -> summary)
    pub files_modified: std::collections::HashMap<String, String>,
    /// Files read but not modified
    pub files_read: Vec<String>,
    /// Key decisions made during the session
    pub decisions_made: Vec<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

impl ArtifactTrail {
    /// Record a file creation
    pub fn record_create(&mut self, path: &str) {
        let path = path.to_string();
        if !self.files_created.contains(&path) {
            self.files_created.push(path);
            self.update_timestamp();
        }
    }

    /// Record a file modification with summary
    pub fn record_modify(&mut self, path: &str, summary: &str) {
        self.files_modified
            .insert(path.to_string(), summary.to_string());
        // Remove from files_read if present (it's now modified)
        self.files_read.retain(|p| p != path);
        self.update_timestamp();
    }

    /// Record a file read (only if not already modified)
    pub fn record_read(&mut self, path: &str) {
        let path = path.to_string();
        if !self.files_modified.contains_key(&path) && !self.files_read.contains(&path) {
            self.files_read.push(path);
            self.update_timestamp();
        }
    }

    /// Record a decision
    pub fn record_decision(&mut self, decision: &str) {
        self.decisions_made.push(decision.to_string());
        self.update_timestamp();
    }

    fn update_timestamp(&mut self) {
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Generate a structured summary for context compression
    #[must_use]
    pub fn to_summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.files_created.is_empty() {
            parts.push(format!(
                "## Files Created\n{}",
                self.files_created
                    .iter()
                    .map(|f| format!("- {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !self.files_modified.is_empty() {
            parts.push(format!(
                "## Files Modified\n{}",
                self.files_modified
                    .iter()
                    .map(|(path, summary)| format!("- {path}: {summary}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !self.files_read.is_empty() {
            parts.push(format!(
                "## Files Read\n{}",
                self.files_read
                    .iter()
                    .map(|f| format!("- {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !self.decisions_made.is_empty() {
            parts.push(format!(
                "## Decisions Made\n{}",
                self.decisions_made
                    .iter()
                    .map(|d| format!("- {d}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        parts.join("\n\n")
    }
}

/// Extract file path from tool input JSON
fn extract_file_path(input: &serde_json::Value) -> Option<String> {
    // Try common field names for file paths
    input
        .get("path")
        .or_else(|| input.get("file_path"))
        .or_else(|| input.get("filepath"))
        .or_else(|| input.get("file"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Extract change summary from tool input (for edit operations)
fn extract_change_summary(input: &serde_json::Value) -> String {
    // Try to get a summary of the change
    if let Some(old) = input.get("old_string").and_then(|v| v.as_str()) {
        let preview: String = old.chars().take(30).collect();
        return format!("replaced '{preview}...'");
    }
    if let Some(content) = input.get("content").and_then(|v| v.as_str()) {
        let lines = content.lines().count();
        return format!("{lines} lines written");
    }
    "modified".to_string()
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

/// Check if a log line is internal intake/sidecar noise that should be filtered.
/// These are implementation details, not user-visible progress.
fn is_internal_noise(line: &str) -> bool {
    // Intake binary internal messages
    line.contains("Streaming output to file for sidecar")
        || line.contains("cli_type=")
        || line.contains("prompt_len=")
        || line.contains("extended_thinking=")
        || line.contains("force_disable_thinking=")
        || line.contains("thinking_budget=")
        || line.contains("mcp_config=")
        // Sidecar internal logs
        || line.contains("Starting Linear sidecar")
        || line.contains("Sidecar configured")
        || line.contains("Linear API client initialized")
        || line.contains("Setting initial agent plan")
        || line.contains("Whip cracker starting")
        || line.contains("Starting main container exit watch")
        || line.contains("Starting HTTP server")
        || line.contains("Initial process count")
        || line.contains("Initialized with existing activities")
        || line.contains("Starting log file streaming")
        || line.contains("Starting Claude stream parsing")
        || line.contains("Starting intake progress stream")
        // Other noise
        || line.contains("Generating text via CLI")
        || line.contains("[DEBUG]")
        || line.starts_with("Fresh ")
        || line.starts_with("   Compiling ")
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
    // Start from beginning of file (not end) to catch any output written before
    // the sidecar started watching. This is important for short-lived tasks like
    // intake where the agent may fail quickly before sidecar starts streaming.
    // Note: SeekFrom::Start(0) is the default, but explicit for clarity.

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
                // Filter out internal noise - don't add to buffer
                if is_internal_noise(&line) {
                    continue;
                }

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
        skills: Option<Vec<String>>,
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

// =============================================================================
// Intake Progress Events (for intake workflow monitoring)
// =============================================================================

/// Step status for tracking workflow progress.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntakeStepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
    Failed,
}

/// Progress events emitted by intake workflow.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IntakeProgressEvent {
    /// Initial configuration summary.
    Config {
        model: String,
        cli: String,
        target_tasks: u32,
        acceptance: u32,
    },
    /// Workflow step progress update.
    Step {
        step: u8,
        total: u8,
        name: String,
        status: IntakeStepStatus,
        #[serde(default)]
        details: Option<String>,
    },
    /// Retry attempt notification.
    Retry {
        step: u8,
        attempt: u8,
        max: u8,
        reason: String,
    },
    /// Task generation progress.
    TaskProgress { generated: u32, target: u32 },
    /// Workflow completion summary.
    Complete {
        tasks: u32,
        duration_secs: f64,
        success: bool,
        #[serde(default)]
        error: Option<String>,
    },
}

/// Artifact trail file path
const ARTIFACT_TRAIL_FILE: &str = "/workspace/artifact-trail.json";

/// Claude stream parsing task - reads stream-json and emits structured activities.
/// Maps Claude CLI output to proper Linear Agent API activity types.
/// Also maintains an artifact trail for context engineering.
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

    // Initialize artifact trail (context engineering)
    let mut artifact_trail = ArtifactTrail::default();
    let mut last_artifact_save = Instant::now();
    let artifact_save_interval = Duration::from_secs(10);

    // Turn counter for artifact injection
    let mut turn_count: u32 = 0;
    let mut last_injection_turn: u32 = 0;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // No new data - persist artifact trail periodically
                if last_artifact_save.elapsed() >= artifact_save_interval {
                    persist_artifact_trail(&artifact_trail).await;
                    last_artifact_save = Instant::now();
                }
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
                        // Count turns (Result events indicate turn completion)
                        if matches!(event, ClaudeStreamEvent::Result { .. }) {
                            turn_count += 1;

                            // Check if we should inject artifact summary
                            if config.artifact_injection_enabled
                                && turn_count - last_injection_turn
                                    >= config.artifact_injection_interval_turns
                                && inject_artifact_summary(&config.input_fifo, &artifact_trail)
                                    .await
                            {
                                last_injection_turn = turn_count;
                                debug!(
                                    turn = turn_count,
                                    interval = config.artifact_injection_interval_turns,
                                    "Artifact summary injected"
                                );
                            }
                        }

                        if let Err(e) = process_stream_event(
                            &client,
                            &config.linear_session_id,
                            &event,
                            &mut tool_state,
                            &mut total_cost,
                            &mut artifact_trail,
                        )
                        .await
                        {
                            warn!(error = %e, line = processed_lines, "Failed to process stream event");
                        }

                        // Persist artifact trail on completion
                        if matches!(event, ClaudeStreamEvent::Result { .. }) {
                            persist_artifact_trail(&artifact_trail).await;
                            info!(
                                files_created = artifact_trail.files_created.len(),
                                files_modified = artifact_trail.files_modified.len(),
                                files_read = artifact_trail.files_read.len(),
                                turn = turn_count,
                                "Artifact trail saved"
                            );
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

/// Persist artifact trail to file
async fn persist_artifact_trail(trail: &ArtifactTrail) {
    match serde_json::to_string_pretty(trail) {
        Ok(json) => {
            if let Err(e) = fs::write(ARTIFACT_TRAIL_FILE, json).await {
                warn!(error = %e, "Failed to persist artifact trail");
            } else {
                debug!("Artifact trail persisted to {}", ARTIFACT_TRAIL_FILE);
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to serialize artifact trail");
        }
    }
}

/// Generate artifact summary message for context injection.
///
/// This addresses the "artifact trail problem" where file tracking accuracy
/// degrades in long sessions. Periodic injection of the summary helps
/// maintain context accuracy.
fn generate_artifact_summary_message(trail: &ArtifactTrail) -> String {
    let mut summary =
        String::from("## Session Artifact Summary (Auto-injected for context retention)\n\n");

    if !trail.files_created.is_empty() {
        summary.push_str("### Files Created\n");
        for file in &trail.files_created {
            use std::fmt::Write;
            let _ = writeln!(summary, "- {file}");
        }
        summary.push('\n');
    }

    if !trail.files_modified.is_empty() {
        summary.push_str("### Files Modified\n");
        for (file, change) in &trail.files_modified {
            use std::fmt::Write;
            let _ = writeln!(summary, "- {file}: {change}");
        }
        summary.push('\n');
    }

    if !trail.decisions_made.is_empty() {
        summary.push_str("### Key Decisions\n");
        for decision in &trail.decisions_made {
            use std::fmt::Write;
            let _ = writeln!(summary, "- {decision}");
        }
        summary.push('\n');
    }

    summary.push_str("---\n\nPlease continue with the task, keeping these artifacts in mind.\n");
    summary
}

/// Inject artifact summary into the agent's input FIFO.
///
/// This implements context compression from the context engineering principles,
/// helping maintain file tracking accuracy in long-running sessions.
async fn inject_artifact_summary(input_fifo: &str, trail: &ArtifactTrail) -> bool {
    // Don't inject if nothing to report
    if trail.files_created.is_empty()
        && trail.files_modified.is_empty()
        && trail.decisions_made.is_empty()
    {
        debug!("No artifacts to inject, skipping");
        return false;
    }

    let message = generate_artifact_summary_message(trail);

    // Write directly to FIFO
    match OpenOptions::new().write(true).open(input_fifo).await {
        Ok(mut file) => {
            let line = format!("{message}\n");
            if let Err(e) = file.write_all(line.as_bytes()).await {
                warn!(error = %e, "Failed to inject artifact summary");
                false
            } else {
                info!(
                    files_created = trail.files_created.len(),
                    files_modified = trail.files_modified.len(),
                    "Injected artifact summary into agent context"
                );
                true
            }
        }
        Err(e) => {
            debug!(error = %e, "FIFO not available for artifact injection");
            false
        }
    }
}

/// Intake progress stream task - reads progress.jsonl and updates Linear plan.
/// Maps intake progress events to Linear activities for better visibility.
async fn progress_stream_task(config: Arc<Config>, linear_client: Option<LinearApiClient>) {
    let Some(client) = linear_client else {
        info!("Linear API not configured, progress stream parsing disabled");
        return;
    };

    // Wait for progress file to exist
    loop {
        if fs::metadata(&config.progress_file).await.is_ok() {
            break;
        }
        debug!(path = %config.progress_file, "Waiting for progress file to exist");
        sleep(Duration::from_secs(2)).await;
    }

    info!(path = %config.progress_file, "Starting intake progress stream parsing");

    // Open file and track position
    let file = match File::open(&config.progress_file).await {
        Ok(f) => f,
        Err(e) => {
            error!(error = %e, "Failed to open progress file");
            return;
        }
    };

    let mut reader = BufReader::new(file);
    let mut current_plan: Vec<PlanStep> = vec![
        PlanStep {
            content: "Parse PRD and generate tasks".to_string(),
            status: PlanStepStatus::Pending,
        },
        PlanStep {
            content: "Analyze task complexity".to_string(),
            status: PlanStepStatus::Pending,
        },
        PlanStep {
            content: "Expand tasks into subtasks".to_string(),
            status: PlanStepStatus::Pending,
        },
        PlanStep {
            content: "Generate documentation".to_string(),
            status: PlanStepStatus::Pending,
        },
    ];

    // Set initial plan
    if let Err(e) = client
        .update_plan(&config.linear_session_id, &current_plan)
        .await
    {
        warn!(error = %e, "Failed to set initial intake plan");
    }

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                sleep(Duration::from_millis(config.stream_poll_interval_ms)).await;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Parse the JSON event
                match serde_json::from_str::<IntakeProgressEvent>(line) {
                    Ok(event) => {
                        if let Err(e) = process_progress_event(
                            &client,
                            &config.linear_session_id,
                            &event,
                            &mut current_plan,
                        )
                        .await
                        {
                            warn!(error = %e, "Failed to process progress event");
                        }
                    }
                    Err(e) => {
                        debug!(error = %e, line = %line, "Failed to parse progress event");
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Error reading progress file");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

/// Process a single intake progress event and update Linear.
#[allow(clippy::ptr_arg)] // plan needs to be Vec for update_plan API
async fn process_progress_event(
    client: &LinearApiClient,
    session_id: &str,
    event: &IntakeProgressEvent,
    plan: &mut Vec<PlanStep>,
) -> Result<()> {
    match event {
        IntakeProgressEvent::Config {
            model,
            cli,
            target_tasks,
            acceptance,
        } => {
            info!(model = %model, cli = %cli, target = %target_tasks, "Processing config event");
            let msg = format!(
                "**Intake starting** | Model: `{model}` | CLI: `{cli}` | Target: ~{target_tasks} tasks | Acceptance: {acceptance}%",
            );
            client.emit_thought(session_id, &msg).await?;
        }
        IntakeProgressEvent::Step {
            step,
            name,
            status,
            details,
            ..
        } => {
            info!(step = %step, name = %name, status = ?status, "Processing step event");
            let step_idx = (*step as usize).saturating_sub(1);
            if step_idx < plan.len() {
                // Update plan step status
                plan[step_idx].status = match status {
                    IntakeStepStatus::Pending => PlanStepStatus::Pending,
                    IntakeStepStatus::InProgress => PlanStepStatus::InProgress,
                    IntakeStepStatus::Completed => PlanStepStatus::Completed,
                    IntakeStepStatus::Skipped | IntakeStepStatus::Failed => {
                        PlanStepStatus::Canceled
                    }
                };

                // Update content with details if completed
                if matches!(status, IntakeStepStatus::Completed) {
                    if let Some(d) = details {
                        plan[step_idx].content = format!("{name} ({d})");
                    }
                }

                client.update_plan(session_id, plan).await?;
            }
        }
        IntakeProgressEvent::Retry {
            step,
            attempt,
            max,
            reason,
        } => {
            let msg = format!("⚠️ **Retry {attempt}/{max}** (Step {step}): {reason}");
            client.emit_ephemeral_thought(session_id, &msg).await?;
        }
        IntakeProgressEvent::TaskProgress { generated, target } => {
            let msg = format!("📊 Generated {generated}/{target} tasks");
            client.emit_ephemeral_thought(session_id, &msg).await?;
        }
        IntakeProgressEvent::Complete {
            tasks,
            duration_secs,
            success,
            error,
        } => {
            if *success {
                let msg = format!("✅ **Intake complete** | {tasks} tasks | {duration_secs:.1}s",);
                client.emit_response(session_id, &msg).await?;
            } else {
                let err_msg = error.as_deref().unwrap_or("Unknown error");
                let msg = format!("❌ **Intake failed** | {err_msg}");
                client.emit_error(session_id, &msg).await?;
            }
        }
    }
    Ok(())
}

/// Track tool invocation state for proper action activities
#[derive(Default)]
#[allow(clippy::struct_field_names)] // current_ prefix is intentional for state tracking clarity
struct ToolState {
    current_tool: Option<String>,
    current_input: Option<String>,
    /// Raw input JSON for artifact extraction
    current_input_json: Option<serde_json::Value>,
}

/// Process a single Claude stream event and emit appropriate Linear activity.
/// Maps Claude events to proper Linear Agent API activity types:
/// - Tool invocations → `action` activities
/// - Tool completions → `action` with result
/// - Errors → `error` activities
/// - Completion → `response` activities
/// - Transient status → ephemeral `thought` activities
///
/// Also tracks file operations for the artifact trail (context engineering best practice).
#[allow(clippy::too_many_lines)] // Complex function not easily split
async fn process_stream_event(
    client: &LinearApiClient,
    session_id: &str,
    event: &ClaudeStreamEvent,
    tool_state: &mut ToolState,
    total_cost: &mut f64,
    artifact_trail: &mut ArtifactTrail,
) -> Result<()> {
    match event {
        ClaudeStreamEvent::System { model, tools, skills, .. } => {
            // System init - emit as thought (it's informational, not an action)
            let tool_count = tools.as_ref().map_or(0, Vec::len);
            let skill_count = skills.as_ref().map_or(0, Vec::len);
            let model_name = model.as_deref().unwrap_or("unknown");
            
            // Build sections for verbose init message
            let mut sections = vec![format!("**Model:** {model_name}")];
            
            // MCP Tools section (show first few names if available)
            if tool_count > 0 {
                if let Some(tool_list) = tools {
                    // Separate MCP tools from native tools
                    let mcp_tools: Vec<_> = tool_list.iter()
                        .filter(|t| t.starts_with("mcp__") || t.contains("__"))
                        .take(10)
                        .cloned()
                        .collect();
                    let native_count = tool_count - mcp_tools.len();
                    
                    if !mcp_tools.is_empty() {
                        let mcp_preview = mcp_tools.iter()
                            .map(|t| t.replace("mcp__cto-tools__", ""))
                            .take(5)
                            .collect::<Vec<_>>()
                            .join(", ");
                        sections.push(format!("**MCP Tools ({}):** {} (+{} more)", mcp_tools.len(), mcp_preview, mcp_tools.len().saturating_sub(5)));
                    }
                    sections.push(format!("**Native Tools:** {native_count}"));
                } else {
                    sections.push(format!("**Tools:** {tool_count}"));
                }
            }
            
            // Skills section
            if skill_count > 0 {
                if let Some(skill_list) = skills {
                    let skill_preview: Vec<_> = skill_list.iter().take(10).cloned().collect();
                    let skill_str = if skill_preview.len() < skill_count {
                        format!("{} (+{} more)", skill_preview.join(", "), skill_count - skill_preview.len())
                    } else {
                        skill_preview.join(", ")
                    };
                    sections.push(format!("**Skills ({skill_count}):** {skill_str}"));
                }
            }
            
            let msg = format!("🚀 **Agent Initialized**\n\n{}", sections.join("\n"));
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
                            tool_state.current_input_json.clone_from(input);

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
            let tool_input_json = tool_state.current_input_json.take();

            // Track file operations for artifact trail (context engineering)
            let is_success = tool_use_result
                .as_ref()
                .is_none_or(|r| !r.contains("error") && !r.contains("Error"));

            if is_success {
                if let Some(ref input_json) = tool_input_json {
                    let tool_lower = tool_name.to_lowercase();

                    // Track file operations based on tool name
                    if tool_lower.contains("write_file") || tool_lower.contains("create") {
                        if let Some(path) = extract_file_path(input_json) {
                            artifact_trail.record_create(&path);
                            debug!(path = %path, "Artifact: file created");
                        }
                    } else if tool_lower.contains("edit_file")
                        || tool_lower.contains("str_replace")
                        || tool_lower.contains("modify")
                    {
                        if let Some(path) = extract_file_path(input_json) {
                            let summary = extract_change_summary(input_json);
                            artifact_trail.record_modify(&path, &summary);
                            debug!(path = %path, summary = %summary, "Artifact: file modified");
                        }
                    } else if tool_lower.contains("read_file")
                        || tool_lower.contains("read_multiple")
                    {
                        if let Some(path) = extract_file_path(input_json) {
                            artifact_trail.record_read(&path);
                            debug!(path = %path, "Artifact: file read");
                        }
                    }
                }
            }

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
            #[allow(clippy::cast_precision_loss)] // Precision loss acceptable for duration display
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
/// Also monitors for stop signals from Linear UI.
async fn input_poll_task(
    config: Arc<Config>,
    linear_client: Option<LinearApiClient>,
    fifo_tx: mpsc::Sender<String>,
    shutdown: Arc<AtomicBool>,
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
        // Check shutdown flag
        if shutdown.load(Ordering::Relaxed) {
            info!("Input polling task shutting down");
            return;
        }

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

                    // Check for stop signal from Linear
                    if activity.is_stop_signal() {
                        warn!(activity_id = %activity.id, "🛑 STOP SIGNAL received from Linear");

                        // Emit response to Linear acknowledging stop
                        let stop_msg = "🛑 Stopped as requested. No further changes were made.";
                        if let Err(e) = client
                            .emit_response(&config.linear_session_id, stop_msg)
                            .await
                        {
                            warn!(error = %e, "Failed to emit stop response to Linear");
                        }

                        // Trigger graceful shutdown
                        shutdown.store(true, Ordering::SeqCst);
                        info!("Shutdown triggered by Linear stop signal");
                        return;
                    }

                    // Check for auth signal (future: handle OAuth flow)
                    if activity.is_auth_signal() {
                        info!(activity_id = %activity.id, "Auth signal received (not yet implemented)");
                        // Future: Trigger OAuth flow
                        continue;
                    }

                    // Check for select signal (future: handle selection)
                    if activity.is_select_signal() {
                        info!(activity_id = %activity.id, "Select signal received (not yet implemented)");
                        // Future: Handle selection choice
                        continue;
                    }

                    // Handle user messages
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
#[allow(clippy::too_many_lines)] // Complex function not easily split
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
                    "⚠️ Agent appears stalled (level {current_nudge_level}). Please continue with the task."
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
            let linear_msg = format!("🔥 **Nudge (level {current_nudge_level})**: {nudge_message}");
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
// Main Container Exit Watch
// =============================================================================

/// Watch for main container exit and trigger graceful sidecar shutdown.
///
/// This task monitors two signals that indicate the main container has exited:
/// 1. The `.agent_done` file appearing (explicit completion signal)
/// 2. Main container process exit via `/proc` (implicit - uses `shareProcessNamespace`)
///
/// When either condition is detected, the shutdown flag is set to trigger
/// graceful termination of all sidecar tasks.
async fn main_exit_watch_task(config: Arc<Config>, shutdown: Arc<AtomicBool>) {
    if !config.main_exit_watch_enabled {
        info!("Main exit watch disabled");
        // When disabled, block forever instead of returning immediately.
        // Returning would cause the select! to trigger shutdown.
        loop {
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            sleep(Duration::from_secs(3600)).await; // Check hourly for shutdown
        }
    }

    info!(
        agent_done_file = %config.agent_done_file,
        interval_ms = config.main_exit_watch_interval_ms,
        "👁️ Starting main container exit watch"
    );

    let check_interval = Duration::from_millis(config.main_exit_watch_interval_ms);

    // Track initial process count to detect when main container processes exit
    let initial_proc_count = count_non_sidecar_processes().await;
    info!(initial_proc_count, "Initial process count recorded");

    loop {
        if shutdown.load(Ordering::Relaxed) {
            debug!("Exit watch: shutdown already requested");
            return;
        }

        sleep(check_interval).await;

        // Check 1: Look for explicit completion signal file
        if fs::metadata(&config.agent_done_file).await.is_ok() {
            info!(
                file = %config.agent_done_file,
                "🏁 Agent done file detected - main container completed"
            );
            shutdown.store(true, Ordering::SeqCst);
            return;
        }

        // Check 2: Look for main container process exit via /proc
        // With shareProcessNamespace, we can see all processes in the pod
        let current_proc_count = count_non_sidecar_processes().await;

        // If process count dropped significantly and we had processes before,
        // the main container likely exited
        if initial_proc_count > 0 && current_proc_count == 0 {
            info!(
                initial = initial_proc_count,
                current = current_proc_count,
                "🏁 Main container processes exited - triggering shutdown"
            );
            shutdown.store(true, Ordering::SeqCst);
            return;
        }
    }
}

/// Count non-sidecar processes visible in /proc.
///
/// With `shareProcessNamespace: true`, we can see processes from other containers.
/// We filter out our own sidecar processes to detect when main container exits.
async fn count_non_sidecar_processes() -> usize {
    let Ok(mut entries) = tokio::fs::read_dir("/proc").await else {
        return 0;
    };

    let mut count = 0;
    let my_pid = std::process::id();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };

        // Only look at numeric directories (PIDs)
        let Ok(pid) = name_str.parse::<u32>() else {
            continue;
        };

        // Skip our own process and kernel threads (PID 1, 2)
        if pid == my_pid || pid <= 2 {
            continue;
        }

        // Try to read the process cmdline to identify it
        let cmdline_path = format!("/proc/{pid}/cmdline");
        if let Ok(cmdline) = tokio::fs::read_to_string(&cmdline_path).await {
            // Skip sidecar-related processes (status-sync, pause container)
            let cmdline_lower = cmdline.to_lowercase();
            if cmdline_lower.contains("status-sync")
                || cmdline_lower.contains("pause")
                || cmdline_lower.contains("/pause")
            {
                continue;
            }

            // Skip docker daemon sidecar processes
            if cmdline_lower.contains("dockerd") || cmdline_lower.contains("containerd") {
                continue;
            }

            // This looks like a main container process
            count += 1;
        }
    }

    count
}

// =============================================================================
// HTTP Server
// =============================================================================

/// Shared state for HTTP handlers.
#[derive(Clone)]
struct AppState {
    fifo_tx: mpsc::Sender<String>,
    shutdown: Arc<AtomicBool>,
    config: Arc<Config>,
    linear_client: Option<LinearApiClient>,
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

/// Stop endpoint - handles Linear stop signal.
///
/// This endpoint is called when a user clicks "Stop" in Linear.
/// It emits a response activity confirming the stop and triggers shutdown.
async fn handle_stop(State(state): State<AppState>) -> impl IntoResponse {
    info!("Stop signal received via HTTP");

    // Emit response activity to Linear
    if let Some(ref client) = state.linear_client {
        if state.config.has_linear_session() {
            let stop_msg = "Stopped as requested. No further changes were made.";
            if let Err(e) = client
                .emit_response(&state.config.linear_session_id, stop_msg)
                .await
            {
                warn!(error = %e, "Failed to emit stop response to Linear");
            } else {
                info!("Emitted stop response to Linear");
            }
        }
    }

    // Trigger graceful shutdown
    state.shutdown.store(true, Ordering::SeqCst);
    (StatusCode::OK, "Agent stopped")
}

// =============================================================================
// FluentD Ingest Endpoint
// =============================================================================

/// Ingest request body from FluentD.
/// Supports both raw log lines and structured JSON.
#[derive(Debug, Deserialize)]
struct IngestRequest {
    /// Raw log line (if not structured)
    #[serde(default)]
    log: Option<String>,
    /// Structured event type
    #[serde(rename = "type", default)]
    event_type: Option<String>,
    /// Container/CLI name from FluentD labels
    #[serde(default)]
    cli: Option<String>,
    /// Agent name from FluentD labels
    #[serde(default)]
    agent: Option<String>,
    /// Catch-all for other fields (full event data)
    #[serde(flatten)]
    extra: serde_json::Value,
}

/// Ingest endpoint - receives logs from FluentD and emits to Linear.
///
/// FluentD sends logs to this endpoint which are then parsed and
/// emitted to the Linear agent dialog in real-time.
async fn handle_ingest(
    State(state): State<AppState>,
    Json(payload): Json<IngestRequest>,
) -> impl IntoResponse {
    debug!(?payload, "Received ingest from FluentD");

    let Some(ref client) = state.linear_client else {
        return (StatusCode::OK, "No Linear client configured");
    };

    if !state.config.has_linear_session() {
        return (StatusCode::OK, "No Linear session configured");
    }

    let session_id = &state.config.linear_session_id;

    // Try to parse as ClaudeStreamEvent
    let event_json = if let Some(log) = &payload.log {
        // FluentD wraps the log line in a "log" field
        log.clone()
    } else {
        // Direct JSON structure
        serde_json::to_string(&payload.extra).unwrap_or_default()
    };

    // Attempt to parse as Claude stream event
    match serde_json::from_str::<ClaudeStreamEvent>(&event_json) {
        Ok(event) => {
            // Use a static tool state for ingest (stateless per-request)
            // In production, this should be shared state
            let mut tool_state = ToolState::default();
            let mut total_cost = 0.0;
            let mut artifact_trail = ArtifactTrail::default();

            if let Err(e) = process_stream_event(
                client,
                session_id,
                &event,
                &mut tool_state,
                &mut total_cost,
                &mut artifact_trail,
            )
            .await
            {
                warn!(error = %e, "Failed to process stream event");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process event");
            }
        }
        Err(_) => {
            // Not a structured Claude event - emit as raw thought
            let msg = payload
                .log
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| event_json.trim());

            // Skip empty or boilerplate messages
            if msg.is_empty() || msg.len() < 10 {
                return (StatusCode::OK, "Skipped empty message");
            }

            // Emit as thought
            if let Err(e) = client.emit_thought(session_id, msg).await {
                warn!(error = %e, "Failed to emit thought from ingest");
            }
        }
    }

    (StatusCode::OK, "Ingested")
}

/// Batch ingest endpoint - receives array of logs from FluentD.
async fn handle_ingest_batch(
    State(state): State<AppState>,
    Json(payloads): Json<Vec<IngestRequest>>,
) -> impl IntoResponse {
    debug!(count = payloads.len(), "Received batch ingest from FluentD");

    for payload in payloads {
        let _ = handle_ingest(State(state.clone()), Json(payload)).await;
    }

    (StatusCode::OK, "Batch ingested")
}

/// HTTP server task.
async fn http_server_task(config: Arc<Config>, state: AppState) {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/input", post(handle_input))
        .route("/stop", post(handle_stop))
        .route("/shutdown", post(handle_shutdown))
        // FluentD ingest endpoints
        .route("/ingest", post(handle_ingest))
        .route("/ingest/batch", post(handle_ingest_batch))
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

    // Auto-create session if issue identifier provided but no session ID
    // This enables standalone mode (docker-compose) without pre-created sessions
    let config = if !config.has_linear_session() {
        if let Some(ref client) = linear_client {
            // Check for LINEAR_ISSUE_IDENTIFIER env var
            if let Ok(issue_id) = std::env::var("LINEAR_ISSUE_IDENTIFIER") {
                if !issue_id.is_empty() {
                    info!(issue = %issue_id, "No session configured, attempting to create one");
                    match client
                        .create_session_on_issue(&issue_id, "claude-sonnet-4", "anthropic")
                        .await
                    {
                        Ok(session_id) => {
                            info!(session_id = %session_id, "Auto-created Linear session");
                            // Create new config with the session ID
                            let mut new_config = (*config).clone();
                            new_config.linear_session_id = session_id;
                            Arc::new(new_config)
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to auto-create session, running in minimal mode");
                            config
                        }
                    }
                } else {
                    info!("No Linear session or issue configured, running in minimal mode (HTTP only)");
                    config
                }
            } else {
                info!("No Linear session or issue configured, running in minimal mode (HTTP only)");
                config
            }
        } else {
            info!("No Linear API client, running in minimal mode (HTTP only)");
            config
        }
    } else {
        config
    };

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
        config: config.clone(),
        linear_client: linear_client.clone(),
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

    // Intake progress stream parsing task (plan updates from progress.jsonl)
    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let progress_handle = tokio::spawn(async move {
        if config_clone.has_linear_api() {
            progress_stream_task(config_clone, linear_client_clone).await;
        }
    });

    // Clone fifo_tx for both input polling and progress monitor
    let fifo_tx_for_input = fifo_tx.clone();
    let fifo_tx_for_whip = fifo_tx;

    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let shutdown_clone = shutdown.clone();
    let input_handle = tokio::spawn(async move {
        if config_clone.has_linear_api() {
            input_poll_task(
                config_clone,
                linear_client_clone,
                fifo_tx_for_input,
                shutdown_clone,
            )
            .await;
        }
    });

    // Whip cracking (progress monitor with escalating nudges)
    let config_clone = config.clone();
    let linear_client_clone = linear_client.clone();
    let shutdown_clone = shutdown.clone();
    let whip_handle = tokio::spawn(async move {
        progress_monitor_task(
            config_clone,
            fifo_tx_for_whip,
            shutdown_clone,
            linear_client_clone,
        )
        .await;
    });

    // Main container exit watch (triggers graceful shutdown when main container exits)
    let config_clone = config.clone();
    let shutdown_clone = shutdown.clone();
    let exit_watch_handle = tokio::spawn(async move {
        main_exit_watch_task(config_clone, shutdown_clone).await;
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
        result = progress_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Progress stream task panicked");
            }
            info!("Progress stream task exited");
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
        result = exit_watch_handle => {
            if let Err(e) = result {
                warn!(error = %e, "Exit watch task panicked");
            }
            info!("Exit watch task detected main container exit");
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that `IntakeProgressEvent::Step` deserializes correctly when `details` field is missing.
    /// This is critical because the producer uses `#[serde(skip_serializing_if = "Option::is_none")]`
    /// to omit the field when `None`, and without `#[serde(default)]` on the consumer side,
    /// deserialization would fail.
    #[test]
    fn test_step_event_deserialize_without_details() {
        let json =
            r#"{"type":"step","step":1,"total":4,"name":"Parse PRD","status":"in_progress"}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        match event {
            IntakeProgressEvent::Step {
                step,
                total,
                name,
                status,
                details,
            } => {
                assert_eq!(step, 1);
                assert_eq!(total, 4);
                assert_eq!(name, "Parse PRD");
                assert!(matches!(status, IntakeStepStatus::InProgress));
                assert!(details.is_none());
            }
            _ => panic!("Expected Step variant"),
        }
    }

    /// Test that `IntakeProgressEvent::Step` deserializes correctly when `details` field is present.
    #[test]
    fn test_step_event_deserialize_with_details() {
        let json = r#"{"type":"step","step":2,"total":4,"name":"Generate Tasks","status":"completed","details":"Generated 50 tasks"}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        match event {
            IntakeProgressEvent::Step {
                step,
                total,
                name,
                status,
                details,
            } => {
                assert_eq!(step, 2);
                assert_eq!(total, 4);
                assert_eq!(name, "Generate Tasks");
                assert!(matches!(status, IntakeStepStatus::Completed));
                assert_eq!(details.as_deref(), Some("Generated 50 tasks"));
            }
            _ => panic!("Expected Step variant"),
        }
    }

    /// Test that `IntakeProgressEvent::Complete` deserializes correctly when `error` field is missing.
    /// This is the success case - workflows complete successfully without an error field.
    #[test]
    fn test_complete_event_deserialize_without_error() {
        let json = r#"{"type":"complete","tasks":50,"duration_secs":120.5,"success":true}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        match event {
            IntakeProgressEvent::Complete {
                tasks,
                duration_secs,
                success,
                error,
            } => {
                assert_eq!(tasks, 50);
                assert!((duration_secs - 120.5).abs() < f64::EPSILON);
                assert!(success);
                assert!(error.is_none());
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    /// Test that `IntakeProgressEvent::Complete` deserializes correctly when `error` field is present.
    /// This is the failure case - workflows fail with an error message.
    #[test]
    fn test_complete_event_deserialize_with_error() {
        let json = r#"{"type":"complete","tasks":0,"duration_secs":30.0,"success":false,"error":"Failed to parse PRD"}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        match event {
            IntakeProgressEvent::Complete {
                tasks,
                duration_secs,
                success,
                error,
            } => {
                assert_eq!(tasks, 0);
                assert!((duration_secs - 30.0).abs() < f64::EPSILON);
                assert!(!success);
                assert_eq!(error.as_deref(), Some("Failed to parse PRD"));
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    /// Test deserialization of all other event types to ensure they still work.
    #[test]
    fn test_other_event_types_deserialize() {
        // Config event
        let json = r#"{"type":"config","model":"claude-opus-4-5","cli":"claude","target_tasks":50,"acceptance":90}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, IntakeProgressEvent::Config { .. }));

        // Retry event
        let json = r#"{"type":"retry","step":1,"attempt":2,"max":3,"reason":"Extended thinking disabled"}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, IntakeProgressEvent::Retry { .. }));

        // TaskProgress event
        let json = r#"{"type":"task_progress","generated":25,"target":50}"#;
        let event: IntakeProgressEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, IntakeProgressEvent::TaskProgress { .. }));
    }
}
