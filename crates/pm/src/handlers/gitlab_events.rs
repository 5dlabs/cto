//! GitLab webhook event handler.
//!
//! Handles GitLab webhook events for dual SCM support.
//! Routes:
//! - Merge Request Hook → intake PR processing (maps to GitHub pull_request)
//! - Pipeline Hook → CI status (maps to GitHub check_run)
//! - Note Hook → comments (maps to GitHub issue_comment)

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::{debug, error, info, warn};

use super::callbacks::CallbackState;
use crate::config::WebhookDispatchMode;
use crate::morgan_hooks::{dispatch_to_morgan, MorganWebhookDispatch};

// =============================================================================
// Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct MergeRequestEvent {
    object_kind: String,
    object_attributes: MrAttributes,
    project: GitLabProject,
    user: GitLabUser,
}

#[derive(Debug, Deserialize)]
struct MrAttributes {
    iid: u64,
    title: String,
    #[serde(default)]
    description: Option<String>,
    source_branch: String,
    target_branch: String,
    action: Option<String>,
    state: String,
    url: String,
    #[serde(default)]
    merge_commit_sha: Option<String>,
    #[serde(default)]
    last_commit: Option<GitLabCommit>,
    #[serde(default)]
    labels: Vec<GitLabLabel>,
}

#[derive(Debug, Deserialize)]
struct GitLabProject {
    id: u64,
    name: String,
    path_with_namespace: String,
    web_url: String,
    #[serde(default)]
    default_branch: Option<String>,
    http_url_to_repo: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabUser {
    username: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabCommit {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GitLabLabel {
    title: String,
}

#[derive(Debug, Deserialize)]
struct PipelineEvent {
    object_kind: String,
    object_attributes: PipelineAttributes,
    project: GitLabProject,
    #[serde(default)]
    builds: Vec<PipelineBuild>,
}

#[derive(Debug, Deserialize)]
struct PipelineAttributes {
    id: u64,
    #[serde(rename = "ref")]
    ref_name: String,
    status: String,
    #[serde(default)]
    detailed_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PipelineBuild {
    id: u64,
    name: String,
    status: String,
    stage: String,
}

#[derive(Debug, Deserialize)]
struct NoteEvent {
    object_kind: String,
    object_attributes: NoteAttributes,
    project: GitLabProject,
    user: GitLabUser,
    #[serde(default)]
    merge_request: Option<NoteMrRef>,
    #[serde(default)]
    issue: Option<NoteIssueRef>,
}

#[derive(Debug, Deserialize)]
struct NoteAttributes {
    #[serde(default)]
    note: String,
    noteable_type: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct NoteMrRef {
    iid: u64,
    source_branch: String,
    target_branch: String,
}

#[derive(Debug, Deserialize)]
struct NoteIssueRef {
    iid: u64,
    title: String,
}

// =============================================================================
// Handler
// =============================================================================

async fn maybe_dispatch_gitlab_webhook_to_morgan(
    state: &CallbackState,
    event_type: &str,
    payload: &Value,
) -> Result<Option<Json<Value>>, StatusCode> {
    let dispatch_mode = state.morgan_dispatch.mode;
    if !dispatch_mode.dispatches_to_morgan() {
        return Ok(None);
    }

    let mut labels = vec![("gitlab_event", event_type.to_string())];
    if let Some(kind) = payload.get("object_kind").and_then(Value::as_str) {
        labels.push(("object_kind", kind.to_string()));
    }
    if let Some(project) = payload
        .pointer("/project/path_with_namespace")
        .and_then(Value::as_str)
    {
        labels.push(("project", project.to_string()));
    }

    let dispatch = MorganWebhookDispatch {
        source: "gitlab",
        route: "/webhooks/gitlab/events",
        event_type: event_type.to_string(),
        delivery_id: payload
            .pointer("/object_attributes/id")
            .and_then(Value::as_u64)
            .map(|id| id.to_string()),
        verified: state.gitlab_webhook_secret.is_some(),
        labels,
        payload: payload.clone(),
    };

    match dispatch_to_morgan(&state.http_client, &state.morgan_dispatch, &dispatch).await {
        Ok(accepted) => {
            info!(
                session_key = %accepted.session_key,
                agent_id = %accepted.agent_id,
                "Forwarded GitLab webhook to Morgan"
            );

            if dispatch_mode == WebhookDispatchMode::Morgan {
                return Ok(Some(Json(json!({
                    "status": "accepted",
                    "dispatch": "morgan",
                    "source": "gitlab",
                    "event_type": event_type,
                    "session_key": accepted.session_key,
                    "agent_id": accepted.agent_id
                }))));
            }

            Ok(None)
        }
        Err(e) => {
            if dispatch_mode == WebhookDispatchMode::Shadow {
                warn!(error = %e, "Failed to shadow-dispatch GitLab webhook to Morgan");
                Ok(None)
            } else {
                error!(error = %e, "Failed to dispatch GitLab webhook to Morgan");
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    }
}

/// Handle incoming GitLab webhook events.
pub async fn handle_gitlab_events(
    State(state): State<Arc<CallbackState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    let event_type = headers
        .get("X-Gitlab-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!(event_type = %event_type, "Received GitLab webhook");

    // Verify token (simple string comparison)
    if let Some(ref expected_secret) = state.gitlab_webhook_secret {
        let provided_token = headers
            .get("X-Gitlab-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !bool::from(provided_token.as_bytes().ct_eq(expected_secret.as_bytes())) {
            warn!("GitLab webhook token verification failed");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let payload: Value = serde_json::from_slice(&body).map_err(|e| {
        error!(error = %e, "Failed to parse GitLab webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    if let Some(response) =
        maybe_dispatch_gitlab_webhook_to_morgan(&state, event_type, &payload).await?
    {
        return Ok(response);
    }

    // Route by event type
    match event_type {
        "Merge Request Hook" => handle_merge_request_event(&state, &body).await,
        "Pipeline Hook" => handle_pipeline_event(&state, &body),
        "Note Hook" => handle_note_event(&state, &body),
        _ => {
            debug!(event_type = %event_type, "Ignoring unhandled GitLab event type");
            Ok(Json(json!({
                "status": "ignored",
                "reason": format!("unhandled event type: {event_type}")
            })))
        }
    }
}

/// Handle GitLab merge request events (parallel to GitHub pull_request).
async fn handle_merge_request_event(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let event: MergeRequestEvent = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse GitLab merge request event");
        StatusCode::BAD_REQUEST
    })?;

    let action = event
        .object_attributes
        .action
        .as_deref()
        .unwrap_or("unknown");
    info!(
        action = %action,
        mr_iid = event.object_attributes.iid,
        project = %event.project.path_with_namespace,
        source_branch = %event.object_attributes.source_branch,
        "Processing GitLab merge request event"
    );

    match action {
        // MR merged → check if intake MR, trigger project creation
        "merge" => {
            if is_intake_mr(&event) {
                info!(
                    mr_iid = event.object_attributes.iid,
                    "Detected merged intake MR — triggering project creation"
                );
                handle_intake_mr_merged(state, &event).await
            } else {
                Ok(Json(json!({
                    "status": "ignored",
                    "reason": "not_intake_mr"
                })))
            }
        }
        // MR opened/updated → could trigger Stitch review (future)
        "open" | "reopen" | "update" => {
            info!(
                action = %action,
                mr_iid = event.object_attributes.iid,
                "MR opened/updated — review trigger placeholder"
            );
            Ok(Json(json!({
                "status": "acknowledged",
                "action": action,
                "mr_iid": event.object_attributes.iid
            })))
        }
        _ => Ok(Json(json!({
            "status": "ignored",
            "reason": format!("unhandled MR action: {action}")
        }))),
    }
}

/// Check if a MR is an intake MR (mirrors is_intake_pr logic from github.rs).
fn is_intake_mr(event: &MergeRequestEvent) -> bool {
    let has_intake_label = event
        .object_attributes
        .labels
        .iter()
        .any(|l| l.title == "cto-intake" || l.title == "intake");

    let branch = &event.object_attributes.source_branch;
    let is_intake_branch = branch.starts_with("intake/")
        || branch.starts_with("intake-")
        || branch.starts_with("cto-intake/")
        || branch.contains("-intake-");

    let title = event.object_attributes.title.to_lowercase();
    let has_intake_title = title.contains("[intake]") || title.starts_with("intake:");

    has_intake_label || is_intake_branch || has_intake_title
}

/// Handle merged intake MR → create Linear project + play CodeRun.
async fn handle_intake_mr_merged(
    state: &CallbackState,
    event: &MergeRequestEvent,
) -> Result<Json<Value>, StatusCode> {
    // Convert GitLab MR event to the GitHub PullRequestEvent shape
    // so we can reuse the existing handle_github_webhook_inner logic.
    let github_shaped = serde_json::json!({
        "action": "closed",
        "pull_request": {
            "number": event.object_attributes.iid,
            "title": event.object_attributes.title,
            "body": event.object_attributes.description,
            "head": {
                "ref": event.object_attributes.source_branch,
                "sha": event
                    .object_attributes
                    .last_commit
                    .as_ref()
                    .map_or("", |c| c.id.as_str())
            },
            "base": {
                "ref": event.object_attributes.target_branch,
                "sha": ""
            },
            "merged": true,
            "merge_commit_sha": event.object_attributes.merge_commit_sha,
            "html_url": event.object_attributes.url,
            "state": "closed",
            "labels": event.object_attributes.labels.iter().map(|l| {
                serde_json::json!({"name": l.title})
            }).collect::<Vec<_>>()
        },
        "repository": {
            "id": event.project.id,
            "name": event.project.name,
            "full_name": event.project.path_with_namespace,
            "clone_url": event.project.http_url_to_repo.as_deref().unwrap_or(""),
            "html_url": event.project.web_url,
            "default_branch": event.project.default_branch
        },
        "sender": {
            "login": event.user.username,
            "id": 0
        }
    });

    let body_bytes = serde_json::to_vec(&github_shaped).map_err(|e| {
        error!(error = %e, "Failed to serialize normalized MR event");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Delegate to existing intake handler
    super::github::handle_github_webhook_inner(state, &body_bytes).await
}

/// Handle GitLab pipeline events (parallel to GitHub check_run).
fn handle_pipeline_event(_state: &CallbackState, body: &[u8]) -> Result<Json<Value>, StatusCode> {
    let event: PipelineEvent = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse GitLab pipeline event");
        StatusCode::BAD_REQUEST
    })?;

    info!(
        pipeline_id = event.object_attributes.id,
        status = %event.object_attributes.status,
        ref_name = %event.object_attributes.ref_name,
        project = %event.project.path_with_namespace,
        build_count = event.builds.len(),
        "GitLab pipeline event"
    );

    // Pipeline failure handling (future: trigger CI remediation like github_events.rs)
    if event.object_attributes.status == "failed" {
        let failed_jobs: Vec<&str> = event
            .builds
            .iter()
            .filter(|b| b.status == "failed")
            .map(|b| b.name.as_str())
            .collect();
        warn!(
            pipeline_id = event.object_attributes.id,
            failed_jobs = ?failed_jobs,
            "Pipeline failed — CI remediation placeholder"
        );
    }

    Ok(Json(json!({
        "status": "acknowledged",
        "pipeline_id": event.object_attributes.id,
        "pipeline_status": event.object_attributes.status
    })))
}

/// Handle GitLab note (comment) events (parallel to GitHub issue_comment).
fn handle_note_event(_state: &CallbackState, body: &[u8]) -> Result<Json<Value>, StatusCode> {
    let event: NoteEvent = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse GitLab note event");
        StatusCode::BAD_REQUEST
    })?;

    info!(
        noteable_type = %event.object_attributes.noteable_type,
        user = %event.user.username,
        project = %event.project.path_with_namespace,
        "GitLab note event"
    );

    // @mention handling (future: route to agent like github_events.rs does)
    if event.object_attributes.note.contains('@') {
        debug!(
            note = %event.object_attributes.note,
            "Note contains @mention — agent routing placeholder"
        );
    }

    Ok(Json(json!({
        "status": "acknowledged",
        "noteable_type": event.object_attributes.noteable_type,
        "user": event.user.username
    })))
}
