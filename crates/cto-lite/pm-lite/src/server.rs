//! HTTP server for GitHub webhooks

use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::{
    config::Config,
    github::{
        extract_prompt, is_trigger_command, should_trigger_workflow, verify_signature, GitHubEvent,
        IssueCommentPayload, IssuePayload, PullRequestPayload,
    },
    workflow::{get_default_stack, trigger_workflow, WorkflowParams},
};

/// Server state
#[derive(Clone)]
pub struct ServerState {
    pub config: Arc<Config>,
}

/// HTTP Server
pub struct Server {
    config: Config,
}

impl Server {
    /// Create a new server with the given config
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the server
    ///
    /// # Errors
    /// Returns error if server fails to start
    pub async fn run(self) -> Result<()> {
        let state = ServerState {
            config: Arc::new(self.config.clone()),
        };

        let app = Router::new()
            .route("/health", get(health))
            .route("/ready", get(ready))
            .route("/webhook/github", post(github_webhook))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.config.port);
        info!("PM Lite starting on {addr}");

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

async fn ready() -> impl IntoResponse {
    Json(json!({"status": "ready"}))
}

async fn github_webhook(
    State(state): State<ServerState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Get event type
    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .map_or(GitHubEvent::Unknown("missing".to_string()), GitHubEvent::from);

    info!("Received GitHub webhook: {event_type:?}");

    // Verify signature if secret is configured
    if let Some(ref secret) = state.config.github.webhook_secret {
        if let Some(signature) = headers
            .get("x-hub-signature-256")
            .and_then(|v| v.to_str().ok())
        {
            if let Err(e) = verify_signature(secret, signature, &body) {
                error!("Webhook signature verification failed: {e}");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid signature"})),
                );
            }
        } else {
            error!("Missing webhook signature");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing signature"})),
            );
        }
    }

    // Handle event
    let result = match event_type {
        GitHubEvent::Issues => handle_issue_event(&state, &body).await,
        GitHubEvent::IssueComment => handle_comment_event(&state, &body).await,
        GitHubEvent::PullRequest => handle_pr_event(&state, &body).await,
        _ => {
            info!("Ignoring event type: {event_type:?}");
            Ok(None)
        }
    };

    match result {
        Ok(Some(workflow_name)) => (StatusCode::OK, Json(json!({"workflow": workflow_name}))),
        Ok(None) => (StatusCode::OK, Json(json!({"status": "ignored"}))),
        Err(e) => {
            error!("Error handling webhook: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    }
}

async fn handle_issue_event(state: &ServerState, body: &[u8]) -> Result<Option<String>> {
    let payload: IssuePayload = serde_json::from_slice(body)?;

    // Only handle opened issues with "cto" label
    if payload.action != "opened" && payload.action != "labeled" {
        return Ok(None);
    }

    if !should_trigger_workflow(&payload.issue) {
        info!(
            "Issue {} doesn't have 'cto' label, skipping",
            payload.issue.number
        );
        return Ok(None);
    }

    info!(
        "Triggering workflow for issue #{} in {}",
        payload.issue.number, payload.repository.full_name
    );

    let params = WorkflowParams {
        repo: payload.repository.full_name,
        branch: payload.repository.default_branch,
        issue_number: Some(payload.issue.number),
        pr_number: None,
        prompt: extract_prompt(payload.issue.body.as_deref().unwrap_or("")),
        stack: get_default_stack(),
    };

    let name = trigger_workflow(&state.config.namespace, params).await?;
    Ok(Some(name))
}

async fn handle_comment_event(state: &ServerState, body: &[u8]) -> Result<Option<String>> {
    let payload: IssueCommentPayload = serde_json::from_slice(body)?;

    // Only handle new comments
    if payload.action != "created" {
        return Ok(None);
    }

    // Check if it's a trigger command
    if !is_trigger_command(&payload.comment.body) {
        return Ok(None);
    }

    // Don't respond to bot comments
    if payload.sender.user_type.as_deref() == Some("Bot") {
        return Ok(None);
    }

    info!(
        "Triggering workflow from comment on issue #{} in {}",
        payload.issue.number, payload.repository.full_name
    );

    let params = WorkflowParams {
        repo: payload.repository.full_name,
        branch: payload.repository.default_branch,
        issue_number: Some(payload.issue.number),
        pr_number: None,
        prompt: extract_prompt(&payload.comment.body),
        stack: get_default_stack(),
    };

    let name = trigger_workflow(&state.config.namespace, params).await?;
    Ok(Some(name))
}

async fn handle_pr_event(state: &ServerState, body: &[u8]) -> Result<Option<String>> {
    let payload: PullRequestPayload = serde_json::from_slice(body)?;

    // Only handle opened PRs (for now)
    if payload.action != "opened" {
        return Ok(None);
    }

    // Check if PR body has trigger command
    let body_text = payload.pull_request.body.as_deref().unwrap_or("");
    if !is_trigger_command(body_text) {
        return Ok(None);
    }

    info!(
        "Triggering workflow for PR #{} in {}",
        payload.pull_request.number, payload.repository.full_name
    );

    let params = WorkflowParams {
        repo: payload.repository.full_name,
        branch: payload.pull_request.head.ref_name,
        issue_number: None,
        pr_number: Some(payload.pull_request.number),
        prompt: extract_prompt(body_text),
        stack: get_default_stack(),
    };

    let name = trigger_workflow(&state.config.namespace, params).await?;
    Ok(Some(name))
}
