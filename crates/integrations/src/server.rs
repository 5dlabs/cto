//! HTTP server for Linear webhooks.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::handlers::callbacks::{
    handle_intake_complete, handle_play_complete, handle_status_sync, handle_tasks_json_callback,
    CallbackState,
};
use crate::handlers::github::handle_github_webhook;
use crate::handlers::intake::{extract_intake_request, submit_intake_workflow};
use crate::handlers::play::{cancel_play_workflow, extract_play_request, submit_play_workflow};
use crate::webhooks::{
    validate_webhook_timestamp, verify_webhook_signature, WebhookAction, WebhookPayload,
    WebhookType,
};
use crate::LinearClient;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Configuration.
    pub config: Config,
    /// Kubernetes client.
    pub kube_client: kube::Client,
    /// Linear API client.
    pub linear_client: Option<LinearClient>,
}

/// Build the HTTP router for the Linear service.
pub fn build_router(state: AppState) -> Router {
    let callback_state = Arc::new(CallbackState {
        linear_client: state.linear_client.clone(),
        github_token: state.config.github_token.clone(),
    });

    Router::new()
        // Webhook endpoints
        .route("/webhooks/linear", post(linear_webhook_handler))
        .route(
            "/webhooks/github",
            post(handle_github_webhook).with_state(callback_state.clone()),
        )
        // Callback endpoints for Argo workflows
        .route(
            "/callbacks/intake-complete",
            post(handle_intake_complete).with_state(callback_state.clone()),
        )
        .route(
            "/callbacks/tasks-json",
            post(handle_tasks_json_callback).with_state(callback_state.clone()),
        )
        .route(
            "/callbacks/play-complete",
            post(handle_play_complete).with_state(callback_state.clone()),
        )
        // Status sync endpoint for sidecar
        .route(
            "/status/linear-sync",
            post(handle_status_sync).with_state(callback_state),
        )
        // Manual trigger endpoints for testing
        .route("/trigger/intake", post(trigger_intake))
        // Health check
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(readiness_check))
        .with_state(state)
}

/// Request body for manual intake trigger.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TriggerIntakeRequest {
    /// Linear issue ID or identifier (e.g., "CTOPA-21" or UUID)
    issue_id: String,
    /// Optional session ID for activity updates (generates one if not provided)
    #[serde(default)]
    session_id: Option<String>,
}

/// Manually trigger intake workflow for an issue.
///
/// This endpoint fetches the issue from Linear and triggers the intake workflow
/// without requiring an agent session webhook.
async fn trigger_intake(
    State(state): State<AppState>,
    Json(request): Json<TriggerIntakeRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!(issue_id = %request.issue_id, "Manual intake trigger requested");

    let Some(client) = &state.linear_client else {
        error!("Linear client not configured");
        return Ok(Json(json!({
            "status": "error",
            "error": "Linear client not configured"
        })));
    };

    // Fetch the issue from Linear
    let issue = match client.get_issue(&request.issue_id).await {
        Ok(issue) => issue,
        Err(e) => {
            error!(error = %e, "Failed to fetch issue from Linear");
            return Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to fetch issue: {e}")
            })));
        }
    };

    info!(
        issue_id = %issue.id,
        identifier = %issue.identifier,
        title = %issue.title,
        "Fetched issue from Linear"
    );

    // Generate session ID if not provided
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("manual-intake-{}", chrono::Utc::now().timestamp()));

    // Extract intake request
    let intake_request = match extract_intake_request(&session_id, &issue) {
        Ok(req) => req,
        Err(e) => {
            error!(error = %e, "Failed to extract intake request");
            return Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to extract intake request: {e}")
            })));
        }
    };

    // Submit the workflow
    let namespace = &state.config.namespace;
    match submit_intake_workflow(
        &state.kube_client,
        namespace,
        &intake_request,
        &state.config.intake,
    )
    .await
    {
        Ok(result) => {
            info!(
                workflow_name = %result.workflow_name,
                configmap_name = %result.configmap_name,
                "Intake workflow submitted via manual trigger"
            );
            Ok(Json(json!({
                "status": "accepted",
                "workflow": "intake",
                "workflow_name": result.workflow_name,
                "configmap_name": result.configmap_name,
                "session_id": session_id,
                "issue": {
                    "id": issue.id,
                    "identifier": issue.identifier,
                    "title": issue.title
                }
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to submit intake workflow");
            Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to submit workflow: {e}")
            })))
        }
    }
}

/// Health check endpoint.
async fn health_check() -> Json<Value> {
    Json(json!({ "status": "healthy" }))
}

/// Readiness check endpoint.
async fn readiness_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    if !state.config.enabled {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(Json(json!({ "status": "ready" })))
}

/// Handle incoming Linear webhooks.
///
/// This handler:
/// 1. Verifies webhook signature (if secret configured)
/// 2. Validates timestamp freshness
/// 3. Routes to appropriate handler based on event type
pub async fn linear_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    // Check if Linear integration is enabled
    if !state.config.enabled {
        debug!("Linear integration is disabled, ignoring webhook");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "linear_disabled"
        })));
    }

    // Extract headers
    let signature = headers
        .get("linear-signature")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let delivery_id = headers
        .get("linear-delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let event_type = headers
        .get("linear-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!(
        delivery_id = %delivery_id,
        event_type = %event_type,
        "Received Linear webhook"
    );

    // Verify signature if secret is configured
    if let Some(secret) = &state.config.webhook_secret {
        let Some(sig) = &signature else {
            warn!("Missing Linear-Signature header");
            return Err(StatusCode::UNAUTHORIZED);
        };

        if !verify_webhook_signature(&body, sig, secret) {
            warn!("Invalid webhook signature");
            return Err(StatusCode::UNAUTHORIZED);
        }
        debug!("Webhook signature verified");
    }

    // Parse webhook payload
    let payload: WebhookPayload = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse webhook payload: {e}");
        StatusCode::BAD_REQUEST
    })?;

    // Validate timestamp freshness
    if !validate_webhook_timestamp(payload.webhook_timestamp, state.config.max_timestamp_age_ms) {
        warn!(
            timestamp = payload.webhook_timestamp,
            "Webhook timestamp is stale"
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Route based on event type
    match (&payload.event_type, &payload.action) {
        (WebhookType::AgentSessionEvent, WebhookAction::Created) => {
            handle_session_created(&state, &payload).await
        }
        (WebhookType::AgentSessionEvent, WebhookAction::Prompted) => {
            handle_session_prompted(&state, &payload).await
        }
        (WebhookType::IssueAttachment, WebhookAction::Created) => {
            handle_attachment_added(&state, &payload).await
        }
        _ => {
            debug!(
                event_type = ?payload.event_type,
                action = ?payload.action,
                "Ignoring unhandled webhook event"
            );
            Ok(Json(json!({
                "status": "ignored",
                "reason": "unhandled_event_type"
            })))
        }
    }
}

/// Handle new agent session (delegation or mention).
///
/// Determines if this is an intake or play request based on issue labels/content.
#[allow(clippy::too_many_lines)]
async fn handle_session_created(
    state: &AppState,
    payload: &WebhookPayload,
) -> Result<Json<Value>, StatusCode> {
    let session_id = payload.get_session_id().ok_or_else(|| {
        warn!("Missing session ID in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    let issue = payload.get_issue().ok_or_else(|| {
        warn!("Missing issue in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    // Get current state name for workflow detection
    let state_name = issue.state.as_ref().map_or("unknown", |s| s.name.as_str());

    info!(
        session_id = %session_id,
        issue_id = %issue.id,
        issue_identifier = %issue.identifier,
        title = %issue.title,
        state = %state_name,
        "Processing new agent session"
    );

    // Log raw issue data from webhook
    info!(
        session_id = %session_id,
        issue_id = %issue.id,
        webhook_labels_count = issue.labels.len(),
        webhook_attachments_count = issue.attachments.len(),
        "Issue data from webhook payload"
    );

    // Webhook payloads often don't include full issue data (labels, attachments)
    // Fetch the full issue from the API to get accurate labels
    let full_issue = if let Some(client) = &state.linear_client {
        match client.get_issue(&issue.id).await {
            Ok(fetched) => {
                info!(
                    session_id = %session_id,
                    fetched_labels_count = fetched.labels.len(),
                    fetched_attachments_count = fetched.attachments.len(),
                    "Fetched full issue from API"
                );
                fetched
            }
            Err(e) => {
                warn!(
                    session_id = %session_id,
                    error = %e,
                    "Failed to fetch issue from API, using webhook data"
                );
                issue.clone()
            }
        }
    } else {
        warn!(session_id = %session_id, "No Linear client, using webhook issue data");
        issue.clone()
    };

    // Check issue labels to determine workflow type
    let labels: Vec<&str> = full_issue.labels.iter().map(|l| l.name.as_str()).collect();

    info!(
        session_id = %session_id,
        labels = ?labels,
        attachments_count = full_issue.attachments.len(),
        "Analyzing issue for workflow type"
    );

    // Log attachment details for debugging
    for (i, attachment) in full_issue.attachments.iter().enumerate() {
        info!(
            session_id = %session_id,
            attachment_index = i,
            attachment_id = %attachment.id,
            attachment_title = ?attachment.title,
            attachment_url = ?attachment.url,
            "Issue attachment"
        );
    }

    let is_prd = labels
        .iter()
        .any(|l| *l == "prd" || *l == "intake" || *l == "product-requirement");
    let is_task = labels
        .iter()
        .any(|l| *l == "task" || *l == "cto-task" || l.starts_with("task-"));

    info!(
        session_id = %session_id,
        is_prd = is_prd,
        is_task = is_task,
        labels = ?labels,
        "Workflow type detection results"
    );

    if is_prd {
        info!(
            session_id = %session_id,
            labels = ?labels,
            "Detected PRD issue - triggering intake workflow"
        );

        // Emit initial thought to Linear
        if let Some(client) = &state.linear_client {
            if let Err(e) = client
                .emit_thought(session_id, "Processing PRD and generating tasks...")
                .await
            {
                warn!(error = %e, "Failed to emit thought activity");
            }
        }

        // Extract intake request from issue (using full issue with labels/attachments)
        let intake_request = match extract_intake_request(session_id, &full_issue) {
            Ok(req) => req,
            Err(e) => {
                error!(error = %e, "Failed to extract intake request");
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_error(session_id, format!("Failed to extract PRD: {e}"))
                        .await;
                }
                return Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to extract intake request: {}", e),
                    "session_id": session_id
                })));
            }
        };

        // Log extracted intake request details
        debug!(
            session_id = %session_id,
            prd_issue_id = %intake_request.prd_issue_id,
            prd_identifier = %intake_request.prd_identifier,
            team_id = %intake_request.team_id,
            repository_url = ?intake_request.repository_url,
            source_branch = ?intake_request.source_branch,
            cli = ?intake_request.cto_config.cli,
            model = ?intake_request.cto_config.model,
            tech_stack_backend = ?intake_request.tech_stack.backend,
            tech_stack_frontend = ?intake_request.tech_stack.frontend,
            tech_stack_languages = ?intake_request.tech_stack.languages,
            "Extracted intake request"
        );

        // Validate repository URL is present
        if intake_request.repository_url.is_none() {
            warn!(
                issue_id = %issue.id,
                "No repository URL found in issue attachments or description"
            );
            if let Some(client) = &state.linear_client {
                let _ = client
                    .emit_error(
                        session_id,
                        "**Missing Repository URL**\n\n\
                        Please add a GitHub repository link to this issue:\n\n\
                        1. Click **Add Link** (or use the attachment icon)\n\
                        2. Paste your GitHub repository URL (e.g., `https://github.com/owner/repo`)\n\
                        3. Re-assign to the agent to retry\n\n\
                        The repository URL tells me where to commit the generated tasks and code.",
                    )
                    .await;
            }
            return Ok(Json(json!({
                "status": "error",
                "error": "Missing repository URL - please add a GitHub link attachment to the issue",
                "session_id": session_id
            })));
        }

        // Submit intake workflow
        match submit_intake_workflow(
            &state.kube_client,
            &state.config.namespace,
            &intake_request,
            &state.config.intake,
        )
        .await
        {
            Ok(result) => {
                info!(
                    workflow_name = %result.workflow_name,
                    configmap_name = %result.configmap_name,
                    "Intake workflow submitted"
                );

                // Emit action activity
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_action(
                            session_id,
                            "Submitted intake workflow",
                            &result.workflow_name,
                        )
                        .await;
                }

                Ok(Json(json!({
                    "status": "accepted",
                    "workflow": "intake",
                    "session_id": session_id,
                    "workflow_name": result.workflow_name,
                    "configmap_name": result.configmap_name,
                    "issue": {
                        "id": full_issue.id,
                        "identifier": full_issue.identifier,
                        "title": full_issue.title
                    }
                })))
            }
            Err(e) => {
                error!(error = %e, "Failed to submit intake workflow");
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_error(session_id, format!("Failed to start intake: {e}"))
                        .await;
                }
                Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to submit intake workflow: {}", e),
                    "session_id": session_id
                })))
            }
        }
    } else if is_task {
        info!(
            session_id = %session_id,
            "Detected task issue - triggering play workflow"
        );

        // Emit initial thought
        if let Some(client) = &state.linear_client {
            if let Err(e) = client
                .emit_thought(session_id, "Starting task implementation...")
                .await
            {
                warn!(error = %e, "Failed to emit thought activity");
            }
        }

        // Extract play request from issue
        let play_request = match extract_play_request(session_id, issue) {
            Ok(req) => req,
            Err(e) => {
                error!(error = %e, "Failed to extract play request");
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_error(session_id, format!("Failed to start play: {e}"))
                        .await;
                }
                return Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to extract play request: {}", e),
                    "session_id": session_id
                })));
            }
        };

        // Submit play workflow
        match submit_play_workflow(&state.config.namespace, &play_request, &state.config.play).await
        {
            Ok(result) => {
                info!(
                    workflow_name = %result.workflow_name,
                    task_id = result.task_id,
                    "Play workflow submitted"
                );

                // Emit action activity
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_action(session_id, "Started play workflow", &result.workflow_name)
                        .await;
                }

                Ok(Json(json!({
                    "status": "accepted",
                    "workflow": "play",
                    "session_id": session_id,
                    "workflow_name": result.workflow_name,
                    "task_id": result.task_id,
                    "issue": {
                        "id": issue.id,
                        "identifier": issue.identifier,
                        "title": issue.title
                    }
                })))
            }
            Err(e) => {
                error!(error = %e, "Failed to submit play workflow");
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_error(session_id, format!("Failed to start play: {e}"))
                        .await;
                }
                Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to submit play workflow: {}", e),
                    "session_id": session_id
                })))
            }
        }
    } else {
        warn!(
            session_id = %session_id,
            state = %state_name,
            labels = ?labels,
            "Issue does not have recognized labels for intake or play workflow"
        );

        // Provide helpful guidance
        if let Some(client) = &state.linear_client {
            let _ = client
                .emit_response(
                    session_id,
                    "I couldn't determine the workflow type for this issue.\n\n\
                    **To trigger a workflow, add one of these labels:**\n\
                    - `prd`, `intake`, or `product-requirement` → PRD processing (intake)\n\
                    - `task` or `cto-task` → Task implementation (play)",
                )
                .await;
        }

        Ok(Json(json!({
            "status": "ignored",
            "reason": "no_recognized_labels",
            "session_id": session_id,
            "current_state": state_name,
            "available_labels": labels,
            "hint": "Add 'prd' label for intake, or 'task' label for play workflow"
        })))
    }
}

/// Handle attachment added to an issue.
///
/// This allows the user to add a GitHub repository URL after initially delegating
/// the issue to the agent. If the attachment is a GitHub URL and the issue has
/// the appropriate labels, triggers the intake workflow.
#[allow(clippy::too_many_lines)]
async fn handle_attachment_added(
    state: &AppState,
    payload: &WebhookPayload,
) -> Result<Json<Value>, StatusCode> {
    use crate::handlers::intake::is_github_repo_url;

    // Get attachment URL
    let Some(attachment_url) = payload.get_attachment_url() else {
        debug!("Attachment event without URL - ignoring");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "no_attachment_url"
        })));
    };

    // Only process GitHub repository URLs
    if !is_github_repo_url(&attachment_url) {
        debug!(url = %attachment_url, "Attachment is not a GitHub repo URL - ignoring");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not_github_repo"
        })));
    }

    // Get the issue ID from the attachment event
    let Some(issue_id) = payload.get_attachment_issue_id() else {
        warn!("Attachment event without issue ID");
        return Err(StatusCode::BAD_REQUEST);
    };

    info!(
        issue_id = %issue_id,
        url = %attachment_url,
        "GitHub repository URL attached to issue"
    );

    // Fetch the full issue to check labels and active session
    let Some(client) = &state.linear_client else {
        error!("Linear client not configured");
        return Ok(Json(json!({
            "status": "error",
            "error": "Linear client not configured"
        })));
    };

    let full_issue = match client.get_issue(&issue_id).await {
        Ok(issue) => issue,
        Err(e) => {
            error!(error = %e, "Failed to fetch issue");
            return Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to fetch issue: {e}")
            })));
        }
    };

    // Check if the issue has PRD labels
    let labels: Vec<&str> = full_issue.labels.iter().map(|l| l.name.as_str()).collect();
    let is_intake = labels
        .iter()
        .any(|l| matches!(l.to_lowercase().as_str(), "prd" | "intake" | "product-requirement"));

    if !is_intake {
        debug!(
            issue_id = %issue_id,
            labels = ?labels,
            "Issue does not have intake labels - ignoring attachment"
        );
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "no_intake_labels"
        })));
    }

    // Check if there's an active agent session on this issue
    // We'll try to get the session from the issue's agent sessions
    let session_id = match client.get_active_session_for_issue(&issue_id).await {
        Ok(Some(session_id)) => session_id,
        Ok(None) => {
            debug!(
                issue_id = %issue_id,
                "No active agent session on issue - ignoring attachment"
            );
            return Ok(Json(json!({
                "status": "ignored",
                "reason": "no_active_session",
                "hint": "Assign the issue to the agent first"
            })));
        }
        Err(e) => {
            warn!(error = %e, "Failed to check for active session");
            return Ok(Json(json!({
                "status": "ignored",
                "reason": "session_check_failed",
                "error": format!("{e}")
            })));
        }
    };

    info!(
        issue_id = %issue_id,
        session_id = %session_id,
        "Found active session - triggering intake after URL attachment"
    );

    // Notify user
    let _ = client
        .emit_thought(
            &session_id,
            "GitHub repository URL detected! Starting PRD processing...",
        )
        .await;

    // Extract intake request
    let intake_request = match extract_intake_request(&session_id, &full_issue) {
        Ok(req) => req,
        Err(e) => {
            error!(error = %e, "Failed to extract intake request");
            let _ = client
                .emit_error(&session_id, format!("Failed to start intake: {e}"))
                .await;
            return Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to extract intake request: {e}"),
                "session_id": session_id
            })));
        }
    };

    // Submit intake workflow
    match submit_intake_workflow(
        &state.kube_client,
        &state.config.namespace,
        &intake_request,
        &state.config.intake,
    )
    .await
    {
        Ok(result) => {
            info!(
                workflow_name = %result.workflow_name,
                "Intake workflow submitted after URL attachment"
            );

            let _ = client
                .emit_action(&session_id, "Started intake workflow", &result.workflow_name)
                .await;

            Ok(Json(json!({
                "status": "accepted",
                "workflow": "intake",
                "trigger": "attachment_added",
                "session_id": session_id,
                "workflow_name": result.workflow_name,
                "issue_id": issue_id
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to submit intake workflow");
            let _ = client
                .emit_error(&session_id, format!("Failed to start intake: {e}"))
                .await;

            Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to submit intake workflow: {e}"),
                "session_id": session_id
            })))
        }
    }
}

/// Handle prompted session (follow-up message or stop signal).
#[allow(clippy::too_many_lines)]
async fn handle_session_prompted(
    state: &AppState,
    payload: &WebhookPayload,
) -> Result<Json<Value>, StatusCode> {
    use crate::handlers::agent_comms::{broadcast_to_session, AgentMessage};

    let session_id = payload.get_session_id().ok_or_else(|| {
        warn!("Missing session ID in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    let issue_identifier = payload.get_issue().map(|i| i.identifier.clone());

    // Check for stop signal
    if payload.has_stop_signal() {
        info!(
            session_id = %session_id,
            "Received stop signal - cancelling workflow"
        );

        if let Some(client) = &state.linear_client {
            let _ = client
                .emit_thought(session_id, "Received stop signal. Cancelling workflow...")
                .await;
        }

        // First, try to send stop signal to running agents
        let stop_msg = AgentMessage::stop("User requested cancellation via Linear");
        if let Ok(agents) = crate::handlers::agent_comms::find_running_agents(
            &state.kube_client,
            &state.config.namespace,
            session_id,
        )
        .await
        {
            for agent in &agents {
                let _ = crate::handlers::agent_comms::send_message_to_agent(agent, &stop_msg).await;
            }
        }

        // Cancel running workflows for this session
        match cancel_play_workflow(&state.config.namespace, session_id).await {
            Ok(()) => {
                info!(session_id = %session_id, "Workflows cancelled");
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_response(session_id, "✅ Workflow cancelled successfully.")
                        .await;
                }
                return Ok(Json(json!({
                    "status": "accepted",
                    "action": "stop",
                    "session_id": session_id,
                    "message": "Workflow cancellation triggered"
                })));
            }
            Err(e) => {
                error!(error = %e, "Failed to cancel workflows");
                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_error(session_id, format!("Failed to cancel workflow: {e}"))
                        .await;
                }
                return Ok(Json(json!({
                    "status": "error",
                    "action": "stop",
                    "session_id": session_id,
                    "error": format!("Failed to cancel workflow: {}", e)
                })));
            }
        }
    }

    // Get the prompt body (user's follow-up message)
    let prompt_body = payload.get_prompt_body();

    info!(
        session_id = %session_id,
        has_prompt = prompt_body.is_some(),
        issue = ?issue_identifier,
        "Received prompted session event"
    );

    // If we have a prompt, forward it to running agents
    if let Some(body) = &prompt_body {
        // Emit ephemeral "processing" thought
        if let Some(client) = &state.linear_client {
            let _ = client
                .emit_ephemeral_thought(session_id, "💭 Processing your message...")
                .await;
        }

        // Try to forward to running agents
        match broadcast_to_session(
            &state.kube_client,
            &state.config.namespace,
            session_id,
            body,
            issue_identifier.as_deref(),
        )
        .await
        {
            Ok(sent_count) => {
                info!(
                    session_id = %session_id,
                    sent_count = sent_count,
                    "Forwarded message to running agents"
                );

                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_thought(
                            session_id,
                            format!("📨 Forwarded your message to {sent_count} running agent(s)"),
                        )
                        .await;
                }

                return Ok(Json(json!({
                    "status": "accepted",
                    "action": "forwarded",
                    "session_id": session_id,
                    "prompt": body,
                    "agents_notified": sent_count,
                    "message": "Message forwarded to running agents"
                })));
            }
            Err(e) => {
                // No running agents found - this is normal if workflow completed
                warn!(
                    session_id = %session_id,
                    error = %e,
                    "Could not forward message to agents"
                );

                if let Some(client) = &state.linear_client {
                    let _ = client
                        .emit_thought(
                            session_id,
                            "⚠️ No active agents found for this session. The workflow may have completed or not started yet.",
                        )
                        .await;
                }

                return Ok(Json(json!({
                    "status": "accepted",
                    "action": "no_agents",
                    "session_id": session_id,
                    "prompt": body,
                    "message": "No running agents found to forward message to"
                })));
            }
        }
    }

    // No prompt body - just acknowledge
    Ok(Json(json!({
        "status": "accepted",
        "action": "prompted",
        "session_id": session_id,
        "message": "Prompt event received (no message body)"
    })))
}
