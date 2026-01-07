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
use std::fmt::Write as _;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::activities::{PlanStep, PlanStepStatus};
use crate::config::Config;
use crate::emitter::{AgentActivityEmitter, LinearAgentEmitter};
use crate::handlers::callbacks::{
    handle_agent_work_started, handle_intake_complete, handle_play_complete, handle_pr_created,
    handle_status_sync, handle_tasks_json_callback, CallbackState,
};
use crate::handlers::github::handle_github_webhook;
use crate::handlers::intake::{extract_intake_request, submit_intake_coderun};
use crate::handlers::play::{cancel_play_workflow, extract_play_request, submit_play_workflow};
use crate::webhooks::{
    identify_agent_or_legacy, validate_webhook_timestamp, WebhookAction, WebhookPayload,
    WebhookType,
};
use crate::LinearClient;

/// Get an agent-specific Linear client using the agent's OAuth access token.
///
/// For agent activities, Linear requires the OAuth token obtained when the agent
/// app was installed, not a personal API key. This function creates a new client
/// with the agent's access token if available.
///
/// Returns `None` if:
/// - The agent is not configured
/// - The agent doesn't have an access token (OAuth not completed)
/// - Failed to create the client
fn get_agent_client(config: &Config, agent_name: &str) -> Option<LinearClient> {
    let token = config.linear.get_access_token(agent_name)?;
    match LinearClient::new(token) {
        Ok(client) => {
            debug!(agent = %agent_name, "Created agent-specific Linear client");
            Some(client)
        }
        Err(e) => {
            warn!(
                agent = %agent_name,
                error = %e,
                "Failed to create agent-specific Linear client"
            );
            None
        }
    }
}

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
    // Create HTTP client for external API calls
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    // Get GitHub token from environment (optional)
    let github_token = std::env::var("GITHUB_TOKEN").ok();

    let callback_state = Arc::new(CallbackState {
        linear_client: state.linear_client.clone(),
        http_client,
        github_token,
        namespace: state.config.namespace.clone(),
        play_config: state.config.play.clone(),
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
            post(handle_status_sync).with_state(callback_state.clone()),
        )
        // PR created callback from agents
        .route(
            "/callbacks/pr-created",
            post(handle_pr_created).with_state(callback_state.clone()),
        )
        // Agent work started callback
        .route(
            "/callbacks/agent-work-started",
            post(handle_agent_work_started).with_state(callback_state),
        )
        // Manual trigger endpoints for testing
        .route("/trigger/intake", post(trigger_intake))
        // Intake setup endpoint - create Linear project + PRD issue
        .route("/api/intake/setup", post(handle_intake_setup))
        // Input routing endpoint - send messages to running agents
        .route("/api/sessions/{session_id}/input", post(send_session_input))
        // OAuth endpoints for Linear agent apps
        .route(
            "/oauth/callback",
            axum::routing::get(crate::handlers::oauth::handle_oauth_callback),
        )
        .route(
            "/oauth/start",
            axum::routing::get(crate::handlers::oauth::handle_oauth_start),
        )
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

/// Request body for intake setup (Linear project + PRD issue creation).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IntakeSetupRequest {
    /// Project name (used for Linear project name and derived identifiers)
    project_name: String,
    /// PRD content (markdown)
    prd_content: String,
    /// Optional architecture content (markdown)
    #[serde(default)]
    architecture_content: Option<String>,
    /// Optional repository URL (if not provided, a new repo will be created)
    #[serde(default)]
    repository_url: Option<String>,
    /// Optional team ID override (uses config default if not provided)
    #[serde(default)]
    team_id: Option<String>,
}

/// Response from intake setup.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IntakeSetupResponse {
    status: String,
    project: IntakeSetupProject,
    prd_issue: IntakeSetupIssue,
    /// Architecture document (created if architecture content was provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    architecture_document: Option<IntakeSetupDocument>,
    /// CTO config document (created for agent settings)
    #[serde(skip_serializing_if = "Option::is_none")]
    cto_config_document: Option<IntakeSetupDocument>,
    next_step: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IntakeSetupProject {
    id: String,
    name: String,
    url: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IntakeSetupIssue {
    id: String,
    identifier: String,
    title: String,
    url: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IntakeSetupDocument {
    id: String,
    title: String,
    url: Option<String>,
}

/// Request body for sending input to a session.
#[derive(Debug, Deserialize)]
struct SessionInputRequest {
    /// Message text to send to the agent.
    text: String,
    /// Optional issue identifier for context.
    #[serde(default)]
    issue_identifier: Option<String>,
}

/// Send input to a running agent session.
///
/// `POST /api/sessions/:session_id/input`
///
/// Routes the message to the agent's sidecar via HTTP.
async fn send_session_input(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Json(request): Json<SessionInputRequest>,
) -> Result<Json<Value>, StatusCode> {
    use crate::handlers::agent_comms::broadcast_to_session;

    info!(
        session_id = %session_id,
        text_len = request.text.len(),
        "Received input for session"
    );

    // Try to send to running agents
    match broadcast_to_session(
        &state.kube_client,
        &state.config.namespace,
        &session_id,
        &request.text,
        request.issue_identifier.as_deref(),
    )
    .await
    {
        Ok(sent_count) => {
            info!(
                session_id = %session_id,
                sent_count = sent_count,
                "Message routed to agents"
            );
            Ok(Json(json!({
                "status": "ok",
                "session_id": session_id,
                "agents_notified": sent_count,
                "message": "Message sent successfully"
            })))
        }
        Err(e) => {
            warn!(
                session_id = %session_id,
                error = %e,
                "Failed to route message"
            );
            Ok(Json(json!({
                "status": "error",
                "session_id": session_id,
                "error": format!("Failed to route message: {e}")
            })))
        }
    }
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

    // Extract intake request (reads PRD/arch from ConfigMap if available)
    let intake_request = match extract_intake_request(&state.kube_client, &session_id, &issue).await
    {
        Ok(req) => req,
        Err(e) => {
            error!(error = %e, "Failed to extract intake request");
            return Ok(Json(json!({
                "status": "error",
                "error": format!("Failed to extract intake request: {e}")
            })));
        }
    };

    // Submit the CodeRun (new architecture - direct CodeRun creation)
    let namespace = &state.config.namespace;
    match submit_intake_coderun(
        &state.kube_client,
        namespace,
        &intake_request,
        &state.config.intake,
    )
    .await
    {
        Ok(result) => {
            info!(
                coderun_name = %result.workflow_name,
                configmap_name = %result.configmap_name,
                "Intake CodeRun submitted via manual trigger"
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

/// Create Linear project and PRD issue for intake.
///
/// `POST /api/intake/setup`
///
/// This is called by the MCP tool to set up Linear before the user triggers intake.
#[allow(clippy::too_many_lines)]
async fn handle_intake_setup(
    State(state): State<AppState>,
    Json(request): Json<IntakeSetupRequest>,
) -> Result<Json<IntakeSetupResponse>, (StatusCode, Json<Value>)> {
    info!(
        project_name = %request.project_name,
        prd_len = request.prd_content.len(),
        has_arch = request.architecture_content.is_some(),
        "Intake setup requested"
    );

    let Some(client) = &state.linear_client else {
        error!("Linear client not configured");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "error",
                "error": "Linear client not configured on server"
            })),
        ));
    };

    // Get team ID/key from request or config (empty strings treated as "not provided")
    let team_id_or_key = request
        .team_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(state.config.linear_team_id.as_deref())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            error!("No Linear team ID configured");
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "error": "No Linear team ID configured. Set LINEAR_TEAM_ID or provide team_id in request."
                })),
            )
        })?;

    // Resolve team key to UUID if needed (team keys are short like "CTOPA", UUIDs contain hyphens)
    let team_id = if team_id_or_key.contains('-') {
        // Already a UUID
        team_id_or_key.to_string()
    } else {
        // Look up team by key
        info!(team_key = %team_id_or_key, "Looking up team by key");
        let team = client.get_team_by_key(team_id_or_key).await.map_err(|e| {
            error!(error = %e, team_key = %team_id_or_key, "Failed to look up team");
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "error": format!("Team '{}' not found: {}", team_id_or_key, e)
                })),
            )
        })?;
        info!(team_id = %team.id, team_name = %team.name, "Resolved team");
        team.id
    };

    // Ensure play workflow states exist for the team's board view
    if let Err(e) = crate::handlers::intake::ensure_play_workflow_states(client, &team_id).await {
        warn!(
            error = %e,
            "Failed to ensure play workflow states (continuing with project creation)"
        );
    }

    // Ensure required labels exist for CLI/model config, status tracking, etc.
    if let Err(e) = client.ensure_required_labels(&team_id).await {
        warn!(
            error = %e,
            "Failed to ensure required labels (continuing with project creation)"
        );
    }

    // Try to find "Planned" project status for initial project state
    let status_id = match client.find_project_status_by_type("planned").await {
        Ok(Some(status)) => {
            info!(status_id = %status.id, status_name = %status.name, "Using 'Planned' project status");
            Some(status.id)
        }
        Ok(None) => {
            debug!("No 'planned' type project status found, project will use default status");
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to look up project status, continuing without");
            None
        }
    };

    // Create project
    let project_description = format!(
        "## Project Overview\n\n\
         Generated from PRD: **{}**\n\n\
         Switch to **Board view** to track progress through play workflow phases.\n\n\
         ---\n\n\
         *Created by CTO Agent intake*",
        request.project_name
    );

    info!(team_id = %team_id, "Creating Linear project: {}", request.project_name);

    let project = client
        .create_project(crate::models::ProjectCreateInput {
            name: request.project_name.clone(),
            description: Some(project_description),
            team_ids: Some(vec![team_id.clone()]),
            lead_id: None,
            target_date: None,
            template_id: None,
            status_id,
        })
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create Linear project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "error": format!("Failed to create Linear project: {e}")
                })),
            )
        })?;

    info!(
        project_id = %project.id,
        project_name = %project.name,
        "Created Linear project"
    );

    // Store PRD, architecture, and repository URL in project ConfigMap (source of truth)
    // This makes the workflow Linear-independent - we don't need to re-fetch from Linear
    if let Err(e) = crate::handlers::document::store_intake_content(
        &state.kube_client,
        &project.id,
        &request.prd_content,
        request.architecture_content.as_deref(),
        request.repository_url.as_deref(),
    )
    .await
    {
        warn!(
            error = %e,
            project_id = %project.id,
            "Failed to store PRD/architecture in ConfigMap (continuing with Linear documents)"
        );
    } else {
        info!(
            project_id = %project.id,
            has_repo = request.repository_url.is_some(),
            "Stored PRD and architecture in project ConfigMap"
        );
    }

    // Create architecture document if provided (as a separate Linear Document)
    let mut architecture_doc: Option<IntakeSetupDocument> = None;
    if let Some(arch) = &request.architecture_content {
        if !arch.is_empty() {
            info!("Creating architecture document for project");
            let arch_input = crate::models::DocumentCreateInput {
                title: "Architecture".to_string(),
                content: Some(arch.clone()),
                project_id: Some(project.id.clone()),
                issue_id: None,
                icon: None, // Linear API doesn't support icon field
                color: None,
            };
            match client.create_document(arch_input).await {
                Ok(doc) => {
                    info!(
                        document_id = %doc.id,
                        document_url = ?doc.url,
                        project_id = %project.id,
                        "Created architecture document for project"
                    );
                    architecture_doc = Some(IntakeSetupDocument {
                        id: doc.id,
                        title: doc.title,
                        url: doc.url,
                    });
                }
                Err(e) => {
                    warn!(
                        project_id = %project.id,
                        error = %e,
                        "Failed to create architecture document (continuing without)"
                    );
                }
            }
        }
    }

    // Create cto-config.json document EARLY so user can configure agent settings
    // before assigning Morgan. This triggers a webhook -> ConfigMap sync.
    let mut cto_config_doc: Option<IntakeSetupDocument> = None;
    {
        info!("Creating cto-config.json document for project");

        // Generate config using shared crate with minimal data
        // Repository URL will be updated by intake-complete callback after repo is created
        let config_input = cto_config::ProjectConfigInput {
            repository_url: None, // Not known yet - repo will be created during intake
            project_name: Some(request.project_name.clone()),
            team_id: team_id.clone(),
            source_branch: None, // Default to main
            docs_repository: None,
            docs_project_directory: None,
        };
        let config = cto_config::generate_project_config(&config_input);
        let config_json = config.to_json().unwrap_or_else(|_| "{}".to_string());

        // Derive service name for display
        let service_name = cto_config::derive_service_name(&request.project_name);

        // Wrap JSON in markdown code fence for better display in Linear
        let document_content = format!(
            "# CTO Configuration\n\n\
             Project-specific configuration for Play workflows.\n\n\
             **Service:** {service_name}\n\n\
             > **Note:** Repository URL will be set after intake workflow creates/validates the repo.\n\n\
             ```json\n{config_json}\n```"
        );

        let config_input = crate::models::DocumentCreateInput {
            title: "cto-config.json".to_string(),
            content: Some(document_content),
            project_id: Some(project.id.clone()),
            issue_id: None,
            icon: None, // Linear API validation issues with emoji icons
            color: None,
        };

        match client.create_document(config_input).await {
            Ok(doc) => {
                info!(
                    document_id = %doc.id,
                    document_url = ?doc.url,
                    project_id = %project.id,
                    "Created cto-config.json document for project (early setup)"
                );
                cto_config_doc = Some(IntakeSetupDocument {
                    id: doc.id,
                    title: doc.title,
                    url: doc.url,
                });
            }
            Err(e) => {
                warn!(
                    project_id = %project.id,
                    error = %e,
                    "Failed to create cto-config.json document (continuing without)"
                );
            }
        }
    }

    // Get or create task:intake label for PRD issues
    let intake_label = client
        .get_or_create_label(&team_id, "task:intake")
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to create task:intake label, continuing without");
            e
        })
        .ok();

    // Build issue description (PRD content only, architecture is a separate document)
    let mut issue_description = format!("## PRD Content\n\n{}", request.prd_content);

    // Add links to supporting documents
    let has_docs = architecture_doc.is_some() || cto_config_doc.is_some();
    if has_docs {
        issue_description.push_str("\n\n---\n\n## 📎 Project Documents\n");
    }
    if let Some(ref arch) = architecture_doc {
        if let Some(ref arch_url) = arch.url {
            let _ = write!(
                issue_description,
                "\n- 📐 **Architecture:** [View Architecture]({arch_url})"
            );
        }
    }
    if let Some(ref config) = cto_config_doc {
        if let Some(ref config_url) = config.url {
            let _ = write!(
                issue_description,
                "\n- ⚙️ **Agent Configuration:** [View cto-config.json]({config_url})"
            );
        }
    }
    issue_description.push_str("\n\n---\n\n*Assign to @Morgan to start intake workflow*");

    // Create PRD issue
    let issue_title = format!("[PRD] {}", request.project_name);
    info!(title = %issue_title, "Creating PRD issue");

    let mut label_ids = Vec::new();
    if let Some(label) = &intake_label {
        label_ids.push(label.id.clone());
    }

    let issue = client
        .create_issue(crate::models::IssueCreateInput {
            team_id: team_id.clone(),
            title: issue_title.clone(),
            description: Some(issue_description),
            parent_id: None,
            priority: Some(2), // High priority
            label_ids: if label_ids.is_empty() {
                None
            } else {
                Some(label_ids)
            },
            project_id: Some(project.id.clone()),
            state_id: None,
            delegate_id: None,
        })
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create PRD issue");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "error": format!("Failed to create PRD issue: {e}")
                })),
            )
        })?;

    let issue_url = issue
        .url
        .clone()
        .unwrap_or_else(|| format!("https://linear.app/team/issue/{}", issue.identifier));

    info!(
        issue_id = %issue.id,
        identifier = %issue.identifier,
        url = %issue_url,
        "Created PRD issue"
    );

    Ok(Json(IntakeSetupResponse {
        status: "success".to_string(),
        project: IntakeSetupProject {
            id: project.id,
            name: project.name,
            url: project.url,
        },
        prd_issue: IntakeSetupIssue {
            id: issue.id,
            identifier: issue.identifier,
            title: issue.title,
            url: issue_url,
        },
        architecture_document: architecture_doc,
        cto_config_document: cto_config_doc,
        next_step: "Assign Morgan to the PRD issue in Linear to start intake workflow".to_string(),
    }))
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

    // Identify agent and verify signature using multi-app configuration
    let agent_id = if let Some(sig) = &signature {
        // Try multi-app identification first, fall back to legacy single secret
        let id = identify_agent_or_legacy(
            &body,
            sig,
            &state.config.linear,
            state.config.webhook_secret.as_deref(),
        );

        if let Some(ref agent_id) = id {
            debug!(
                agent = %agent_id.agent,
                "Webhook signature verified for agent"
            );
        } else {
            // No valid signature found
            warn!("Invalid webhook signature - no matching agent or legacy secret");
            return Err(StatusCode::UNAUTHORIZED);
        }
        id
    } else if state.config.webhook_secret.is_some() || !state.config.linear.apps.is_empty() {
        // Signature required but missing
        warn!("Missing Linear-Signature header");
        return Err(StatusCode::UNAUTHORIZED);
    } else {
        // No secrets configured, allow unsigned webhooks (dev mode)
        debug!("No webhook secrets configured, accepting unsigned webhook");
        None
    };

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

    // Extract agent name for logging
    let agent_name = agent_id.as_ref().map_or("unknown", |id| id.agent.as_str());

    // Route based on event type
    match (&payload.event_type, &payload.action) {
        (WebhookType::AgentSessionEvent, WebhookAction::Created) => {
            info!(agent = %agent_name, "Routing to session created handler");
            handle_session_created(&state, &payload, agent_name).await
        }
        (WebhookType::AgentSessionEvent, WebhookAction::Prompted) => {
            info!(agent = %agent_name, "Routing to session prompted handler");
            handle_session_prompted(&state, &payload, agent_name).await
        }
        (WebhookType::Document, WebhookAction::Create | WebhookAction::Update) => {
            info!(agent = %agent_name, action = ?payload.action, "Routing to document handler");
            handle_document_event(&state, &payload).await
        }
        _ => {
            debug!(
                agent = %agent_name,
                event_type = ?payload.event_type,
                action = ?payload.action,
                "Ignoring unhandled webhook event"
            );
            Ok(Json(json!({
                "status": "ignored",
                "reason": "unhandled_event_type",
                "agent": agent_name
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
    agent_name: &str,
) -> Result<Json<Value>, StatusCode> {
    let session_id = payload.get_session_id().ok_or_else(|| {
        warn!("Missing session ID in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    let webhook_issue = payload.get_issue().ok_or_else(|| {
        warn!("Missing issue in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    // Linear webhooks often don't include labels in the embedded issue.
    // If labels are empty, fetch the full issue from the API.
    let issue = if webhook_issue.labels.is_empty() {
        if let Some(client) = &state.linear_client {
            match client.get_issue(&webhook_issue.id).await {
                Ok(full_issue) => {
                    debug!(
                        issue_id = %webhook_issue.id,
                        label_count = full_issue.labels.len(),
                        "Fetched full issue with labels"
                    );
                    full_issue
                }
                Err(e) => {
                    warn!(error = %e, "Failed to fetch full issue, using webhook data");
                    webhook_issue.clone()
                }
            }
        } else {
            webhook_issue.clone()
        }
    } else {
        webhook_issue.clone()
    };

    // Get current state name for workflow detection
    let state_name = issue.state.as_ref().map_or("unknown", |s| s.name.as_str());

    info!(
        session_id = %session_id,
        issue_id = %issue.id,
        issue_identifier = %issue.identifier,
        title = %issue.title,
        state = %state_name,
        label_count = issue.labels.len(),
        "Processing new agent session"
    );

    // Check issue labels to determine workflow type
    let labels: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();

    // Check for intake/PRD labels (legacy: prd, intake, product-requirement; new: task:intake)
    let is_prd = labels.iter().any(|l| {
        *l == "prd"
            || *l == "intake"
            || *l == "product-requirement"
            || *l == "task:intake"
            || l.eq_ignore_ascii_case("task:intake")
    });
    // Check for task/play labels (legacy: task, cto-task, task-*; new: task:play)
    let is_task = labels.iter().any(|l| {
        *l == "task"
            || *l == "cto-task"
            || *l == "task:play"
            || l.eq_ignore_ascii_case("task:play")
            || l.starts_with("task-")
    });

    if is_prd {
        info!(
            session_id = %session_id,
            "Detected PRD issue - triggering intake workflow"
        );

        // Create emitter for activity emission using agent's OAuth token
        // (not the workspace API key, which Linear rejects for agent activities)
        let agent_client = get_agent_client(&state.config, agent_name);
        let emitter = agent_client
            .as_ref()
            .map(|client| LinearAgentEmitter::new(client.clone(), session_id));

        if emitter.is_none() {
            warn!(
                agent = %agent_name,
                "Agent OAuth token not available - activities will not be emitted. \
                 Complete OAuth authorization at /oauth/start?agent={agent_name}"
            );
        }

        // Emit initial thought and plan
        if let Some(ref emitter) = emitter {
            if let Err(e) = emitter
                .emit_thought("Processing PRD and generating tasks...", false)
                .await
            {
                warn!(error = %e, "Failed to emit thought activity");
            }

            // Set initial plan
            match emitter
                .update_plan(&[
                    PlanStep {
                        content: "Extract PRD content".to_string(),
                        status: PlanStepStatus::InProgress,
                    },
                    PlanStep::pending("Submit intake workflow"),
                    PlanStep::pending("Parse PRD with AI"),
                    PlanStep::pending("Generate tasks"),
                    PlanStep::pending("Create Linear issues"),
                ])
                .await
            {
                Ok(success) => {
                    info!(success, "Updated initial plan");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to update initial plan");
                }
            }
        }

        // Extract intake request from issue (reads PRD/arch from ConfigMap if available)
        let intake_request =
            match extract_intake_request(&state.kube_client, session_id, &issue).await {
                Ok(req) => req,
                Err(e) => {
                    error!(error = %e, "Failed to extract intake request");
                    if let Some(ref emitter) = emitter {
                        let _ = emitter
                            .emit_error(&format!("Failed to extract PRD: {e}"))
                            .await;
                    }
                    return Ok(Json(json!({
                        "status": "error",
                        "error": format!("Failed to extract intake request: {e}"),
                        "session_id": session_id
                    })));
                }
            };

        // Update plan - extraction complete
        if let Some(ref emitter) = emitter {
            if let Err(e) = emitter
                .update_plan(&[
                    PlanStep::completed("Extract PRD content"),
                    PlanStep {
                        content: "Submit intake workflow".to_string(),
                        status: PlanStepStatus::InProgress,
                    },
                    PlanStep::pending("Parse PRD with AI"),
                    PlanStep::pending("Generate tasks"),
                    PlanStep::pending("Create Linear issues"),
                ])
                .await
            {
                warn!(error = %e, "Failed to update plan after extraction");
            }
        }

        // Submit intake CodeRun (new architecture - direct CodeRun creation)
        match submit_intake_coderun(
            &state.kube_client,
            &state.config.namespace,
            &intake_request,
            &state.config.intake,
        )
        .await
        {
            Ok(result) => {
                info!(
                    coderun_name = %result.workflow_name,
                    configmap_name = %result.configmap_name,
                    "Intake CodeRun submitted"
                );

                // Emit action activity and update plan
                if let Some(ref emitter) = emitter {
                    match emitter
                        .emit_action("Submitted workflow", &result.workflow_name)
                        .await
                    {
                        Ok(id) => info!(activity_id = %id, "Emitted action activity"),
                        Err(e) => warn!(error = %e, "Failed to emit action activity"),
                    }

                    if let Err(e) = emitter
                        .update_plan(&[
                            PlanStep::completed("Extract PRD content"),
                            PlanStep::completed("Submit intake workflow"),
                            PlanStep {
                                content: "Parse PRD with AI".to_string(),
                                status: PlanStepStatus::InProgress,
                            },
                            PlanStep::pending("Generate tasks"),
                            PlanStep::pending("Create Linear issues"),
                        ])
                        .await
                    {
                        warn!(error = %e, "Failed to update plan after workflow submission");
                    }
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
                if let Some(ref emitter) = emitter {
                    let _ = emitter
                        .emit_error(&format!("Failed to start intake: {e}"))
                        .await;
                }
                Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to submit intake workflow: {e}"),
                    "session_id": session_id
                })))
            }
        }
    } else if is_task {
        info!(
            session_id = %session_id,
            "Detected task issue - triggering play workflow"
        );

        // Create emitter for activity emission using agent's OAuth token
        let agent_client = get_agent_client(&state.config, agent_name);
        let emitter = agent_client
            .as_ref()
            .map(|client| LinearAgentEmitter::new(client.clone(), session_id));

        if emitter.is_none() {
            warn!(
                agent = %agent_name,
                "Agent OAuth token not available - activities will not be emitted"
            );
        }

        // Emit initial thought and plan
        if let Some(ref emitter) = emitter {
            if let Err(e) = emitter
                .emit_thought("Starting task implementation...", false)
                .await
            {
                warn!(error = %e, "Failed to emit thought activity");
            }

            // Set initial plan for play workflow
            if let Err(e) = emitter
                .update_plan(&[
                    PlanStep {
                        content: "Extract task details".to_string(),
                        status: PlanStepStatus::InProgress,
                    },
                    PlanStep::pending("Submit play workflow"),
                    PlanStep::pending("Implementation"),
                    PlanStep::pending("Quality review"),
                    PlanStep::pending("Security audit"),
                    PlanStep::pending("Testing"),
                    PlanStep::pending("Create PR"),
                ])
                .await
            {
                warn!(error = %e, "Failed to set initial play plan");
            }
        }

        // Extract play request from issue
        let play_request = match extract_play_request(session_id, &issue) {
            Ok(req) => req,
            Err(e) => {
                error!(error = %e, "Failed to extract play request");
                if let Some(ref emitter) = emitter {
                    let _ = emitter
                        .emit_error(&format!("Failed to start play: {e}"))
                        .await;
                }
                return Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to extract play request: {e}"),
                    "session_id": session_id
                })));
            }
        };

        // Update plan - extraction complete
        if let Some(ref emitter) = emitter {
            if let Err(e) = emitter
                .update_plan(&[
                    PlanStep::completed("Extract task details"),
                    PlanStep {
                        content: "Submit play workflow".to_string(),
                        status: PlanStepStatus::InProgress,
                    },
                    PlanStep::pending("Implementation"),
                    PlanStep::pending("Quality review"),
                    PlanStep::pending("Security audit"),
                    PlanStep::pending("Testing"),
                    PlanStep::pending("Create PR"),
                ])
                .await
            {
                warn!(error = %e, "Failed to update play plan after extraction");
            }
        }

        // Submit play workflow
        match submit_play_workflow(&state.config.namespace, &play_request, &state.config.play).await
        {
            Ok(result) => {
                info!(
                    workflow_name = %result.workflow_name,
                    task_id = result.task_id,
                    "Play workflow submitted"
                );

                // Emit action activity and update plan
                if let Some(ref emitter) = emitter {
                    match emitter
                        .emit_action("Started play workflow", &result.workflow_name)
                        .await
                    {
                        Ok(id) => info!(activity_id = %id, "Emitted play action activity"),
                        Err(e) => warn!(error = %e, "Failed to emit play action activity"),
                    }

                    if let Err(e) = emitter
                        .update_plan(&[
                            PlanStep::completed("Extract task details"),
                            PlanStep::completed("Submit play workflow"),
                            PlanStep {
                                content: "Implementation".to_string(),
                                status: PlanStepStatus::InProgress,
                            },
                            PlanStep::pending("Quality review"),
                            PlanStep::pending("Security audit"),
                            PlanStep::pending("Testing"),
                            PlanStep::pending("Create PR"),
                        ])
                        .await
                    {
                        warn!(error = %e, "Failed to update play plan after submission");
                    }
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
                if let Some(ref emitter) = emitter {
                    let _ = emitter
                        .emit_error(&format!("Failed to start play: {e}"))
                        .await;
                }
                Ok(Json(json!({
                    "status": "error",
                    "error": format!("Failed to submit play workflow: {e}"),
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

        // Provide helpful guidance using agent's client
        if let Some(client) = &get_agent_client(&state.config, agent_name) {
            let _ = client
                .emit_response(
                    session_id,
                    "I couldn't determine the workflow type for this issue.\n\n\
                    **To trigger a workflow, add one of these labels:**\n\
                    - `task:intake` (or legacy: `prd`, `intake`) → PRD processing\n\
                    - `task:play` (or legacy: `task`, `cto-task`) → Task implementation",
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

/// Handle prompted session (follow-up message or stop signal).
#[allow(clippy::too_many_lines)]
async fn handle_session_prompted(
    state: &AppState,
    payload: &WebhookPayload,
    agent_name: &str,
) -> Result<Json<Value>, StatusCode> {
    use crate::handlers::agent_comms::{broadcast_to_session, AgentMessage};

    let session_id = payload.get_session_id().ok_or_else(|| {
        warn!("Missing session ID in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    let issue_identifier = payload.get_issue().map(|i| i.identifier.clone());

    // Get agent-specific client for activities
    let agent_client = get_agent_client(&state.config, agent_name);

    // Check for stop signal
    if payload.has_stop_signal() {
        info!(
            session_id = %session_id,
            "Received stop signal - cancelling workflow"
        );

        if let Some(client) = &agent_client {
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
                if let Some(client) = &agent_client {
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
                if let Some(client) = &agent_client {
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
        if let Some(client) = &agent_client {
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

                if let Some(client) = &agent_client {
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

                if let Some(client) = &agent_client {
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

/// Handle Document events (create/update).
///
/// Syncs `cto-config.json` documents to Kubernetes `ConfigMaps` for project-specific
/// Play workflow configuration.
#[allow(clippy::too_many_lines)]
async fn handle_document_event(
    state: &AppState,
    payload: &WebhookPayload,
) -> Result<Json<Value>, StatusCode> {
    use crate::handlers::document::{sync_architecture_to_configmap, sync_document_to_configmap};

    let document = payload.get_document_data().ok_or_else(|| {
        warn!("Missing document data in webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    info!(
        document_id = %document.id,
        document_title = %document.title,
        project_id = ?document.project_id,
        action = ?payload.action,
        "Received Document webhook"
    );

    // Must have a project association
    let Some(project_id) = document.project_id.as_deref() else {
        debug!(
            document_id = %document.id,
            "Document has no project association, ignoring"
        );
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "no_project",
            "document_title": document.title
        })));
    };

    // Handle different document types
    match document.title.as_str() {
        "cto-config.json" => {
            // Sync CTO config to ConfigMap (existing behavior)
            match sync_document_to_configmap(&document, project_id).await {
                Ok(configmap_name) => {
                    info!(
                        document_id = %document.id,
                        project_id = %project_id,
                        configmap_name = %configmap_name,
                        "Synced CTO config document to ConfigMap"
                    );
                    Ok(Json(json!({
                        "status": "synced",
                        "document_type": "cto-config",
                        "document_id": document.id,
                        "project_id": project_id,
                        "configmap_name": configmap_name
                    })))
                }
                Err(e) => {
                    error!(
                        document_id = %document.id,
                        project_id = %project_id,
                        error = %e,
                        "Failed to sync CTO config document to ConfigMap"
                    );
                    Ok(Json(json!({
                        "status": "error",
                        "error": format!("Failed to sync document: {e}"),
                        "document_id": document.id,
                        "project_id": project_id
                    })))
                }
            }
        }
        "Architecture" => {
            // Sync Architecture document to ConfigMap
            let content = document.content.as_deref().unwrap_or("");
            match sync_architecture_to_configmap(&state.kube_client, project_id, content).await {
                Ok(configmap_name) => {
                    info!(
                        document_id = %document.id,
                        project_id = %project_id,
                        configmap_name = %configmap_name,
                        "Synced Architecture document to ConfigMap"
                    );
                    Ok(Json(json!({
                        "status": "synced",
                        "document_type": "architecture",
                        "document_id": document.id,
                        "project_id": project_id,
                        "configmap_name": configmap_name
                    })))
                }
                Err(e) => {
                    error!(
                        document_id = %document.id,
                        project_id = %project_id,
                        error = %e,
                        "Failed to sync Architecture document to ConfigMap"
                    );
                    Ok(Json(json!({
                        "status": "error",
                        "error": format!("Failed to sync architecture: {e}"),
                        "document_id": document.id,
                        "project_id": project_id
                    })))
                }
            }
        }
        _ => {
            debug!(
                document_title = %document.title,
                "Ignoring unrecognized document type"
            );
            Ok(Json(json!({
                "status": "ignored",
                "reason": "unrecognized_document_type",
                "document_title": document.title
            })))
        }
    }
}
