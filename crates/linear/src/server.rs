//! HTTP server for Linear webhooks.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::handlers::callbacks::{
    handle_intake_complete, handle_tasks_json_callback, CallbackState,
};
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
    });

    Router::new()
        // Webhook endpoint
        .route("/webhooks/linear", post(linear_webhook_handler))
        // Callback endpoints for Argo workflows
        .route(
            "/callbacks/intake-complete",
            post(handle_intake_complete).with_state(callback_state.clone()),
        )
        .route(
            "/callbacks/tasks-json",
            post(handle_tasks_json_callback).with_state(callback_state),
        )
        // Health check
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(readiness_check))
        .with_state(state)
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

    info!(
        session_id = %session_id,
        issue_id = %issue.id,
        issue_identifier = %issue.identifier,
        title = %issue.title,
        "Processing new agent session"
    );

    // Check issue labels to determine workflow type
    let labels: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();

    let is_prd = labels
        .iter()
        .any(|l| *l == "prd" || *l == "intake" || *l == "product-requirement");
    let is_task = labels
        .iter()
        .any(|l| *l == "task" || *l == "cto-task" || l.starts_with("task-"));

    if is_prd {
        info!(
            session_id = %session_id,
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

        // Extract intake request from issue
        let intake_request = match extract_intake_request(session_id, issue) {
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
                        "id": issue.id,
                        "identifier": issue.identifier,
                        "title": issue.title
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
            labels = ?labels,
            "Issue does not have recognized workflow labels"
        );

        // Provide helpful guidance
        if let Some(client) = &state.linear_client {
            let _ = client
                .emit_response(
                    session_id,
                    "I couldn't determine the workflow type for this issue.\n\n\
                    Please add one of the following labels:\n\
                    - `prd` or `intake` for PRD processing\n\
                    - `task` or `cto-task` for task implementation",
                )
                .await;
        }

        Ok(Json(json!({
            "status": "ignored",
            "reason": "no_workflow_labels",
            "session_id": session_id,
            "available_labels": labels,
            "hint": "Add 'prd' or 'intake' label for intake workflow, or 'task'/'cto-task' for play workflow"
        })))
    }
}

/// Handle prompted session (follow-up message or stop signal).
async fn handle_session_prompted(
    state: &AppState,
    payload: &WebhookPayload,
) -> Result<Json<Value>, StatusCode> {
    let session_id = payload.get_session_id().ok_or_else(|| {
        warn!("Missing session ID in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

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

    // Get the prompt body
    let prompt_body = payload.get_prompt_body();

    info!(
        session_id = %session_id,
        has_prompt = prompt_body.is_some(),
        "Received prompted session event"
    );

    // Acknowledge the prompt
    if let Some(client) = &state.linear_client {
        if let Some(body) = &prompt_body {
            let _ = client
                .emit_thought(session_id, format!("Received follow-up: {body}"))
                .await;
        }
    }

    // TODO: Handle follow-up prompts (could trigger additional actions)
    Ok(Json(json!({
        "status": "accepted",
        "action": "prompted",
        "session_id": session_id,
        "prompt": prompt_body,
        "message": "Prompt received (handling not yet implemented)"
    })))
}
