//! Unified GitHub webhook handler.
//!
//! Receives ALL GitHub webhook events directly (replacing Argo Events).
//! Handles:
//! - PR opened/synchronize/reopened → Stitch code review (CodeRun)
//! - PR merged to main → Task completion / play trigger
//! - Check run requested_action → Remediation (forward to existing handler)
//! - Check run completed+failure → CI failure buttons (forward to existing handler)
//! - Issue/PR comments with @mentions → Agent response (forward to existing handler)

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use hmac::{Hmac, Mac};
use kube::{api::PostParams, Api};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::{debug, error, info, warn};

use super::callbacks::CallbackState;

// =============================================================================
// Types
// =============================================================================

/// Minimal pull_request payload for routing decisions.
#[derive(Debug, Deserialize)]
struct PrEventPayload {
    action: String,
    pull_request: PrData,
    repository: RepoData,
    #[serde(default)]
    sender: Option<SenderData>,
}

#[derive(Debug, Deserialize)]
struct PrData {
    number: u64,
    title: String,
    #[serde(default)]
    body: Option<String>,
    head: RefData,
    base: RefData,
    #[serde(default)]
    merged: bool,
    html_url: String,
    #[serde(default)]
    merge_commit_sha: Option<String>,
    #[serde(default)]
    merged_by: Option<SenderData>,
    #[serde(default)]
    labels: Vec<LabelData>,
    user: UserData,
}

#[derive(Debug, Deserialize)]
struct RefData {
    #[serde(rename = "ref")]
    ref_name: String,
    sha: String,
}

#[derive(Debug, Deserialize)]
struct RepoData {
    name: String,
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct UserData {
    login: String,
    #[serde(rename = "type", default)]
    user_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SenderData {
    login: String,
}

#[derive(Debug, Deserialize)]
struct LabelData {
    name: String,
}

// =============================================================================
// Signature Verification
// =============================================================================

type HmacSha256 = Hmac<Sha256>;

/// Verify GitHub webhook signature (HMAC-SHA256).
fn verify_github_signature(secret: &str, signature_header: &str, body: &[u8]) -> bool {
    // GitHub sends: sha256=<hex-digest>
    let Some(hex_sig) = signature_header.strip_prefix("sha256=") else {
        warn!("GitHub signature missing sha256= prefix");
        return false;
    };

    let Ok(expected) = hex::decode(hex_sig) else {
        warn!("GitHub signature is not valid hex");
        return false;
    };

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        error!("Failed to create HMAC from webhook secret");
        return false;
    };

    mac.update(body);
    let computed = mac.finalize().into_bytes();

    // Constant-time comparison to prevent timing attacks
    computed.as_slice().ct_eq(&expected).into()
}

// =============================================================================
// Unified Webhook Handler
// =============================================================================

/// Unified GitHub webhook endpoint.
///
/// Receives all GitHub webhook events, verifies the signature, and routes
/// to the appropriate handler based on the event type and action.
#[allow(clippy::too_many_lines)]
pub async fn handle_github_events(
    State(state): State<Arc<CallbackState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!(
        event_type = %event_type,
        delivery_id = %delivery_id,
        "Received unified GitHub webhook"
    );

    // Verify signature if secret is configured
    if let Some(ref secret) = state.github_webhook_secret {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !verify_github_signature(secret, signature, &body) {
            warn!(
                delivery_id = %delivery_id,
                "GitHub webhook signature verification failed"
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
        debug!("GitHub webhook signature verified");
    } else {
        debug!("No GITHUB_WEBHOOK_SECRET configured, skipping signature verification");
    }

    // Route based on event type
    match event_type {
        "pull_request" => handle_pr_event(&state, &body).await,

        "check_run" => handle_check_run_event(&state, &body).await,

        "issue_comment" | "pull_request_review_comment" => {
            handle_comment_event(&state, &body).await
        }

        "ping" => {
            info!("Received GitHub ping event");
            Ok(Json(json!({ "status": "pong" })))
        }

        _ => {
            debug!(event_type = %event_type, "Ignoring unhandled GitHub event type");
            Ok(Json(json!({
                "status": "ignored",
                "reason": format!("unhandled_event_type: {event_type}")
            })))
        }
    }
}

// =============================================================================
// Pull Request Event Router
// =============================================================================

/// Route pull_request events to the appropriate handler.
async fn handle_pr_event(state: &CallbackState, body: &[u8]) -> Result<Json<Value>, StatusCode> {
    let payload: PrEventPayload = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse pull_request payload");
        StatusCode::BAD_REQUEST
    })?;

    match payload.action.as_str() {
        // PR opened, new commits pushed, or reopened → Stitch review
        "opened" | "synchronize" | "reopened" => handle_pr_review_trigger(state, &payload).await,

        // PR closed + merged → Task completion or play trigger
        // Pass raw bytes so intake PRs can delegate to the existing handler
        "closed" if payload.pull_request.merged => handle_pr_merged(state, body, &payload).await,

        _ => {
            debug!(
                action = %payload.action,
                pr = %payload.pull_request.number,
                "Ignoring pull_request action"
            );
            Ok(Json(json!({
                "status": "ignored",
                "reason": format!("unhandled_pr_action: {}", payload.action)
            })))
        }
    }
}

// =============================================================================
// Stitch PR Review (replaces stitch-pr-review-sensor)
// =============================================================================

/// Allowed repositories for Stitch code review.
const REVIEW_ALLOWED_REPOS: &[&str] = &["5dlabs/cto", "5dlabs/web"];

/// Handle PR opened/synchronize/reopened → Create Stitch review CodeRun.
async fn handle_pr_review_trigger(
    state: &CallbackState,
    payload: &PrEventPayload,
) -> Result<Json<Value>, StatusCode> {
    let repo = &payload.repository.full_name;
    let pr_number = payload.pull_request.number;

    // Filter: only allowed repos
    if !REVIEW_ALLOWED_REPOS.contains(&repo.as_str()) {
        debug!(repo = %repo, "Repo not in review allowlist, skipping");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "repo_not_in_allowlist"
        })));
    }

    // Filter: only User PRs (skip bots like dependabot, renovate)
    let user_type = payload
        .pull_request
        .user
        .user_type
        .as_deref()
        .unwrap_or("User");
    if user_type != "User" {
        debug!(
            user_type = %user_type,
            author = %payload.pull_request.user.login,
            "Skipping bot PR"
        );
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "bot_pr"
        })));
    }

    let head_sha = &payload.pull_request.head.sha;
    let sha7 = &head_sha[..7.min(head_sha.len())];

    // Deterministic name for deduplication (same as Argo sensor used)
    let run_name = format!("review-pr{pr_number}-{sha7}");

    info!(
        pr = %pr_number,
        repo = %repo,
        action = %payload.action,
        run_name = %run_name,
        "Creating Stitch review CodeRun"
    );

    let coderun = json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": run_name,
            "namespace": "cto",
            "labels": {
                "workflow-type": "stitch-review",
                "trigger": "webhook",
                "pr-number": pr_number.to_string(),
                "repository": repo.replace('/', "-")
            }
        },
        "spec": {
            "runType": "review",
            "service": payload.repository.name,
            "repositoryUrl": format!("https://github.com/{}.git", repo),
            "docsRepositoryUrl": format!("https://github.com/{}.git", repo),
            "docsProjectDirectory": ".",
            "workingDirectory": ".",
            "githubApp": "5DLabs-Stitch",
            "model": "claude-opus-4-5-20251101",
            "env": {
                "PR_NUMBER": pr_number.to_string(),
                "PR_URL": payload.pull_request.html_url,
                "PR_TITLE": payload.pull_request.title,
                "PR_BRANCH": payload.pull_request.head.ref_name,
                "PR_BASE_BRANCH": payload.pull_request.base.ref_name,
                "PR_AUTHOR": payload.pull_request.user.login,
                "REPO_FULL_NAME": repo,
                "HEAD_SHA": head_sha,
                "REVIEW_MODE": "review",
                "TRIGGER": "webhook"
            }
        }
    });

    create_coderun_from_json(&state.kube_client, coderun)
        .await
        .map_err(|e| {
            // Duplicate name → already exists (idempotent)
            if e.contains("already exists") {
                info!(run_name = %run_name, "CodeRun already exists (duplicate webhook)");
                return StatusCode::OK;
            }
            error!(error = %e, "Failed to create Stitch review CodeRun");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(run_name = %run_name, "Stitch review CodeRun created");

    Ok(Json(json!({
        "status": "created",
        "action": "stitch_review",
        "run_name": run_name,
        "pr_number": pr_number,
        "repository": repo
    })))
}

// =============================================================================
// PR Merged Handler (replaces play-workflow-pr-merged-sensor)
// =============================================================================

/// Handle PR merged to main → detect intake or task PR, take action.
///
/// `body` is the raw webhook payload bytes, needed for delegation to the
/// intake handler which re-parses them into its own `PullRequestEvent` type.
async fn handle_pr_merged(
    state: &CallbackState,
    body: &[u8],
    payload: &PrEventPayload,
) -> Result<Json<Value>, StatusCode> {
    let pr = &payload.pull_request;
    let repo = &payload.repository.full_name;

    // Only handle merges to main branch
    if pr.base.ref_name != "main" {
        debug!(
            base = %pr.base.ref_name,
            "PR not merged to main, skipping"
        );
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not_merged_to_main"
        })));
    }

    info!(
        pr = %pr.number,
        repo = %repo,
        title = %pr.title,
        "Processing PR merged to main"
    );

    // Detect PR type from labels and title
    let is_intake =
        pr.labels.iter().any(|l| l.name == "cto-intake") || pr.title.contains("🚀 Intake:");
    let task_id = extract_task_id(&pr.title, &pr.labels);

    if is_intake {
        info!(pr = %pr.number, "Detected INTAKE PR merge");
        // Delegate to the full intake handler in github.rs which handles
        // Linear project creation + play CodeRun trigger
        super::github::handle_github_webhook_inner(state, body).await
    } else if let Some(task_id) = task_id {
        info!(pr = %pr.number, task_id = %task_id, "Detected TASK PR merge");
        handle_task_pr_merged(state, payload, &task_id).await
    } else {
        debug!(pr = %pr.number, "PR is neither intake nor task, ignoring");
        Ok(Json(json!({
            "status": "ignored",
            "reason": "not_intake_or_task_pr"
        })))
    }
}

/// Extract task ID from PR title or labels.
fn extract_task_id(title: &str, labels: &[LabelData]) -> Option<String> {
    // Check title for "Task X" or "task-X" pattern
    let re = regex::Regex::new(r"(?i)task[- ]?(\d+)").ok()?;
    if let Some(caps) = re.captures(title) {
        return caps.get(1).map(|m| m.as_str().to_string());
    }

    // Check labels for "task-X" pattern
    for label in labels {
        if let Some(id) = label.name.strip_prefix("task-") {
            if id.chars().all(|c| c.is_ascii_digit()) {
                return Some(id.to_string());
            }
        }
    }

    None
}

/// Handle task PR merged → move task to completed, notify orchestration.
///
/// Uses a deterministic CodeRun name (`task-complete-{task_id}-pr{number}`)
/// so duplicate webhook deliveries are idempotent.
async fn handle_task_pr_merged(
    state: &CallbackState,
    payload: &PrEventPayload,
    task_id: &str,
) -> Result<Json<Value>, StatusCode> {
    let pr = &payload.pull_request;
    let repo = &payload.repository.full_name;
    let merge_sha7 = pr
        .merge_commit_sha
        .as_deref()
        .map_or("unknown", |s| &s[..7.min(s.len())]);

    info!(
        pr = %pr.number,
        task_id = %task_id,
        repo = %repo,
        "Processing task PR merge"
    );

    // Deterministic name for deduplication: same PR merge always yields
    // the same CodeRun name so Kubernetes rejects duplicates.
    let run_name = format!("task-complete-{task_id}-pr{}-{merge_sha7}", pr.number);

    let coderun = json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": run_name,
            "namespace": &state.namespace,
            "labels": {
                "workflow-type": "task-complete",
                "task-id": task_id,
                "trigger": "pr-merge",
                "pr-number": pr.number.to_string(),
                "repository": repo.replace('/', "-")
            }
        },
        "spec": {
            "runType": "task-complete",
            "service": payload.repository.name,
            "repositoryUrl": format!("https://github.com/{repo}"),
            "docsRepositoryUrl": format!("https://github.com/{repo}"),
            "docsProjectDirectory": ".",
            "workingDirectory": ".",
            "githubApp": "5DLabs-Morgan",
            "model": "claude-sonnet-4-20250514",
            "env": {
                "TASK_ID": task_id,
                "PR_NUMBER": pr.number.to_string(),
                "PR_TITLE": pr.title,
                "PR_URL": pr.html_url,
                "MERGE_SHA": pr.merge_commit_sha.as_deref().unwrap_or(""),
                "MERGED_BY": pr.merged_by.as_ref().map_or("unknown", |u| u.login.as_str()),
                "REPOSITORY": repo,
                "TRIGGER": "pr-merge"
            }
        }
    });

    create_coderun_from_json(&state.kube_client, coderun)
        .await
        .map_err(|e| {
            // Duplicate name → already exists (idempotent)
            if e.contains("already exists") {
                info!(run_name = %run_name, "Task-complete CodeRun already exists (duplicate webhook)");
                return StatusCode::OK;
            }
            error!(error = %e, "Failed to create task-complete CodeRun");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        run_name = %run_name,
        task_id = %task_id,
        "Task-complete CodeRun created"
    );

    Ok(Json(json!({
        "status": "created",
        "action": "task_complete",
        "run_name": run_name,
        "task_id": task_id,
        "pr_number": pr.number,
        "repository": repo
    })))
}

// =============================================================================
// Check Run Event Router
// =============================================================================

/// Route check_run events to remediation or CI failure handlers.
async fn handle_check_run_event(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let payload: Value = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse check_run payload");
        StatusCode::BAD_REQUEST
    })?;

    let action = payload.get("action").and_then(Value::as_str).unwrap_or("");

    match action {
        // Remediation button clicked
        "requested_action" => {
            info!("Routing check_run requested_action to remediation handler");
            crate::handlers::agent_interactions::handle_remediation_webhook_inner(state, body).await
        }

        // CI check completed with failure → create remediation buttons
        "completed" => {
            let conclusion = payload
                .pointer("/check_run/conclusion")
                .and_then(Value::as_str)
                .unwrap_or("");

            if conclusion == "failure" {
                info!("Routing check_run failure to CI failure handler");
                crate::handlers::agent_interactions::handle_ci_failure_webhook_inner(state, body)
                    .await
            } else {
                debug!(conclusion = %conclusion, "Ignoring non-failure check_run");
                Ok(Json(json!({
                    "status": "ignored",
                    "reason": format!("check_run_conclusion: {conclusion}")
                })))
            }
        }

        _ => {
            debug!(action = %action, "Ignoring check_run action");
            Ok(Json(json!({
                "status": "ignored",
                "reason": format!("unhandled_check_run_action: {action}")
            })))
        }
    }
}

// =============================================================================
// Comment Event Router
// =============================================================================

/// Route comment events to the @mention handler.
async fn handle_comment_event(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let payload: Value = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse comment payload");
        StatusCode::BAD_REQUEST
    })?;

    let action = payload.get("action").and_then(Value::as_str).unwrap_or("");

    if action != "created" {
        debug!(action = %action, "Ignoring non-created comment");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": format!("comment_action: {action}")
        })));
    }

    // Check for @agent mentions in the comment body
    let comment_body = payload
        .pointer("/comment/body")
        .and_then(Value::as_str)
        .unwrap_or("");

    // Quick check: does the comment contain any @5DLabs- mention?
    if !comment_body.contains("@5DLabs-") && !comment_body.contains("@5dlabs-") {
        debug!("Comment has no @agent mention, skipping");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "no_agent_mention"
        })));
    }

    info!("Routing comment with @agent mention to mention handler");
    crate::handlers::agent_interactions::handle_mention_webhook_inner(state, body).await
}

// =============================================================================
// Helpers
// =============================================================================

/// Create a CodeRun CRD from a JSON value.
async fn create_coderun_from_json(
    kube_client: &kube::Client,
    coderun_json: Value,
) -> Result<String, String> {
    let api: Api<kube::api::DynamicObject> = Api::namespaced_with(
        kube_client.clone(),
        "cto",
        &kube::api::ApiResource {
            group: "agents.platform".to_string(),
            version: "v1".to_string(),
            kind: "CodeRun".to_string(),
            api_version: "agents.platform/v1".to_string(),
            plural: "coderuns".to_string(),
        },
    );

    let run_name = coderun_json
        .pointer("/metadata/name")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let obj: kube::api::DynamicObject = serde_json::from_value(coderun_json)
        .map_err(|e| format!("Failed to serialize CodeRun: {e}"))?;

    api.create(&PostParams::default(), &obj)
        .await
        .map_err(|e| format!("Failed to create CodeRun: {e}"))?;

    Ok(run_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature_valid() {
        let secret = "test-secret";
        let body = b"hello world";

        // Compute expected signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let result = mac.finalize().into_bytes();
        let sig = format!("sha256={}", hex::encode(result));

        assert!(verify_github_signature(secret, &sig, body));
    }

    #[test]
    fn test_verify_signature_invalid() {
        let secret = "test-secret";
        let body = b"hello world";
        let bad_sig = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

        assert!(!verify_github_signature(secret, bad_sig, body));
    }

    #[test]
    fn test_verify_signature_missing_prefix() {
        let secret = "test-secret";
        let body = b"hello world";

        assert!(!verify_github_signature(secret, "invalid", body));
    }

    #[test]
    fn test_extract_task_id_from_title() {
        let labels = vec![];
        assert_eq!(
            extract_task_id("Task 3: Implement API", &labels),
            Some("3".to_string())
        );
        assert_eq!(
            extract_task_id("task-5 fix build", &labels),
            Some("5".to_string())
        );
        assert_eq!(
            extract_task_id("[Task 12] Add feature", &labels),
            Some("12".to_string())
        );
        assert_eq!(extract_task_id("Regular PR title", &labels), None);
    }

    #[test]
    fn test_extract_task_id_from_labels() {
        let labels = vec![
            LabelData {
                name: "enhancement".to_string(),
            },
            LabelData {
                name: "task-7".to_string(),
            },
        ];
        assert_eq!(
            extract_task_id("Some title without task", &labels),
            Some("7".to_string())
        );
    }
}
