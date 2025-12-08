//! HTTP server for CI remediation.
//!
//! Provides REST API endpoints for:
//! - Health checks
//! - Receiving CI failure events from sensors
//! - Querying remediation status

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

use super::{
    context::ContextGatherer,
    router::CiRouter,
    spawner::CodeRunSpawner,
    types::{CiFailure, RemediationConfig, RemediationContext},
};

/// Server state shared across handlers.
pub struct ServerState {
    /// CI router for failure classification
    pub router: CiRouter,
    /// Context gatherer
    pub gatherer: ContextGatherer,
    /// `CodeRun` spawner
    pub spawner: RwLock<CodeRunSpawner>,
    /// Configuration
    pub config: RemediationConfig,
    /// Repository
    pub repository: String,
    /// Namespace
    pub namespace: String,
}

impl ServerState {
    /// Create a new server state.
    ///
    /// # Errors
    ///
    /// Returns an error if the `CodeRunSpawner` cannot be created or templates cannot be loaded.
    pub fn new(config: RemediationConfig, repository: &str, namespace: &str) -> Result<Self> {
        let router = CiRouter::new();
        let gatherer = ContextGatherer::new(repository, namespace);
        let mut spawner = CodeRunSpawner::new(config.clone(), namespace, repository)?;

        // Load CI prompt templates from standard locations
        // Try /app/prompts/ci first (production), then crates/healer/prompts/ci (dev)
        let template_dirs = ["/app/prompts/ci", "crates/healer/prompts/ci", "prompts/ci"];
        let mut loaded = false;
        for dir in &template_dirs {
            if std::path::Path::new(dir).exists() {
                if let Err(e) = spawner.load_templates(dir) {
                    warn!("Failed to load templates from {dir}: {e}");
                } else {
                    info!("Loaded CI templates from {dir}");
                    loaded = true;
                    break;
                }
            }
        }
        if !loaded {
            warn!("No CI templates found, using generic prompts");
        }

        Ok(Self {
            router,
            gatherer,
            spawner: RwLock::new(spawner),
            config,
            repository: repository.to_string(),
            namespace: namespace.to_string(),
        })
    }
}

/// Build the HTTP router.
pub fn build_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/remediate/ci-failure", post(ci_failure_handler))
        .route(
            "/api/remediate/security-alert",
            post(security_alert_handler),
        )
        .route("/api/status", get(status_handler))
        .route("/api/status/{task_id}", get(task_status_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Start the HTTP server.
///
/// # Errors
///
/// Returns an error if the server fails to start or bind to the address.
pub async fn run_server(state: Arc<ServerState>, addr: &str) -> Result<()> {
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Healer CI remediation server listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// Request/Response types
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// CI failure event from webhook/sensor.
#[derive(Debug, Deserialize)]
pub struct CiFailureRequest {
    /// GitHub `workflow_job` or `check_run` event
    #[serde(flatten)]
    pub event: serde_json::Value,
}

/// Response to CI failure request.
#[derive(Debug, Serialize)]
pub struct CiFailureResponse {
    /// Request status
    pub status: ResponseStatus,
    /// `CodeRun` name (if created)
    pub coderun_name: Option<String>,
    /// Agent selected
    pub agent: Option<String>,
    /// Failure type classification
    pub failure_type: Option<String>,
    /// Reason for skipping (if applicable)
    pub reason: Option<String>,
}

/// Response status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    /// Request accepted, remediation started
    Accepted,
    /// Request skipped (duplicate, excluded, etc.)
    Skipped,
    /// Request failed
    Failed,
}

/// Overall server status.
#[derive(Debug, Serialize)]
struct ServerStatus {
    status: &'static str,
    active_remediations: usize,
    total_processed: u64,
    config: ConfigStatus,
}

/// Configuration status.
#[derive(Debug, Serialize)]
struct ConfigStatus {
    cli: String,
    model: String,
    max_attempts: u32,
    time_window_mins: u32,
    memory_enabled: bool,
}

/// Task status response.
#[derive(Debug, Serialize)]
struct TaskStatus {
    task_id: String,
    found: bool,
    status: Option<String>,
    agent: Option<String>,
    attempts: Option<u32>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check handler.
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// CI failure handler - main entry point for CI remediation.
async fn ci_failure_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<CiFailureRequest>,
) -> impl IntoResponse {
    info!("Received CI failure event");

    // Parse the event
    let Some(failure) = parse_ci_failure(&request.event) else {
        warn!("Could not parse CI failure event");
        return (
            StatusCode::BAD_REQUEST,
            Json(CiFailureResponse {
                status: ResponseStatus::Failed,
                coderun_name: None,
                agent: None,
                failure_type: None,
                reason: Some("Could not parse event".to_string()),
            }),
        );
    };

    // Validate the event
    if !should_process(&failure, &state.config) {
        info!("Skipping CI failure (filtered out)");
        return (
            StatusCode::OK,
            Json(CiFailureResponse {
                status: ResponseStatus::Skipped,
                coderun_name: None,
                agent: None,
                failure_type: None,
                reason: Some("Event filtered out".to_string()),
            }),
        );
    }

    // Gather context
    let mut ctx = match state.gatherer.gather(&failure) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to gather context: {e}");
            // Continue with minimal context
            RemediationContext {
                failure: Some(failure.clone()),
                ..Default::default()
            }
        }
    };

    // Classify the failure
    let failure_type = state.router.classify_failure(&failure, &ctx.workflow_logs);
    ctx.failure_type = Some(failure_type.clone());

    // Route to agent
    let agent = state.router.route(&ctx);

    info!(
        "Routing {} ({}) to {:?}",
        failure.workflow_name,
        failure_type.short_name(),
        agent
    );

    // Spawn CodeRun
    let spawner = state.spawner.read().await;
    match spawner.spawn(agent, &ctx) {
        Ok(coderun_name) => {
            info!("Spawned CodeRun: {coderun_name}");
            (
                StatusCode::ACCEPTED,
                Json(CiFailureResponse {
                    status: ResponseStatus::Accepted,
                    coderun_name: Some(coderun_name),
                    agent: Some(agent.name().to_string()),
                    failure_type: Some(failure_type.short_name().to_string()),
                    reason: None,
                }),
            )
        }
        Err(e) => {
            // Check if it's a deduplication skip
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("Recent remediation") {
                info!("Skipping (dedup): {msg}");
                (
                    StatusCode::OK,
                    Json(CiFailureResponse {
                        status: ResponseStatus::Skipped,
                        coderun_name: None,
                        agent: Some(agent.name().to_string()),
                        failure_type: Some(failure_type.short_name().to_string()),
                        reason: Some(msg),
                    }),
                )
            } else {
                error!("Failed to spawn CodeRun: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CiFailureResponse {
                        status: ResponseStatus::Failed,
                        coderun_name: None,
                        agent: Some(agent.name().to_string()),
                        failure_type: Some(failure_type.short_name().to_string()),
                        reason: Some(msg),
                    }),
                )
            }
        }
    }
}

/// Security alert handler.
async fn security_alert_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Received security alert event");

    // Parse the security alert
    let alert = parse_security_alert(&request);

    // Security alerts always go to Cipher
    let agent = super::types::Agent::Cipher;

    // Build context
    let ctx = RemediationContext {
        security_alert: Some(alert.clone()),
        failure_type: Some(super::types::CiFailureType::SecurityCodeScan),
        ..Default::default()
    };

    info!(
        "Routing security alert {} ({}) to {:?}",
        alert.alert_type, alert.severity, agent
    );

    // Spawn CodeRun
    let spawner = state.spawner.read().await;
    match spawner.spawn(agent, &ctx) {
        Ok(coderun_name) => {
            info!("Spawned CodeRun for security alert: {coderun_name}");
            (
                StatusCode::ACCEPTED,
                Json(CiFailureResponse {
                    status: ResponseStatus::Accepted,
                    coderun_name: Some(coderun_name),
                    agent: Some(agent.name().to_string()),
                    failure_type: Some("security".to_string()),
                    reason: None,
                }),
            )
        }
        Err(e) => {
            error!("Failed to spawn CodeRun for security alert: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CiFailureResponse {
                    status: ResponseStatus::Failed,
                    coderun_name: None,
                    agent: Some(agent.name().to_string()),
                    failure_type: Some("security".to_string()),
                    reason: Some(e.to_string()),
                }),
            )
        }
    }
}

/// Server status handler.
async fn status_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    Json(ServerStatus {
        status: "running",
        active_remediations: 0, // TODO: Track active remediations
        total_processed: 0,     // TODO: Track total processed
        config: ConfigStatus {
            cli: state.config.cli.clone(),
            model: state.config.model.clone(),
            max_attempts: state.config.max_attempts,
            time_window_mins: state.config.time_window_mins,
            memory_enabled: state.config.memory_enabled,
        },
    })
}

/// Task status handler.
async fn task_status_handler(
    State(_state): State<Arc<ServerState>>,
    axum::extract::Path(task_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    // TODO: Query actual task status from ConfigMaps/CodeRuns
    Json(TaskStatus {
        task_id,
        found: false,
        status: None,
        agent: None,
        attempts: None,
    })
}

// ============================================================================
// Helper functions
// ============================================================================

/// Parse a CI failure from the webhook event.
fn parse_ci_failure(event: &serde_json::Value) -> Option<CiFailure> {
    // Try workflow_job format first
    if event.get("workflow_job").is_some() {
        return CiFailure::from_workflow_job(event);
    }

    // Try check_run format
    if event.get("check_run").is_some() {
        return CiFailure::from_check_run(event);
    }

    // Try direct body format (from sensor wrapping)
    if event.get("body").is_some() {
        let body = event.get("body")?;
        if body.get("workflow_job").is_some() {
            return CiFailure::from_workflow_job(body);
        }
        if body.get("check_run").is_some() {
            return CiFailure::from_check_run(body);
        }
    }

    None
}

/// Parse a security alert from webhook event.
fn parse_security_alert(event: &serde_json::Value) -> super::types::SecurityAlert {
    use chrono::Utc;

    // Get alert type from GitHub event header
    let event_type = event
        .get("X-GitHub-Event")
        .or_else(|| event.get("action"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Handle different alert types
    let (alert_type, severity, cve_id, package_name, description) =
        if let Some(alert) = event.get("alert") {
            // Dependabot or code scanning alert
            let severity = alert
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let cve = alert
                .get("security_advisory")
                .and_then(|sa| sa.get("cve_id"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let pkg = alert
                .get("security_vulnerability")
                .and_then(|sv| sv.get("package"))
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let desc = alert
                .get("security_advisory")
                .and_then(|sa| sa.get("summary"))
                .and_then(|v| v.as_str())
                .unwrap_or("Security alert");

            (
                "dependabot_alert".to_string(),
                severity.to_string(),
                cve,
                pkg,
                desc.to_string(),
            )
        } else if let Some(secret) = event.get("secret_scanning_alert") {
            // Secret scanning alert
            let secret_type = secret
                .get("secret_type_display_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown secret");
            (
                "secret_scanning_alert".to_string(),
                "critical".to_string(),
                None,
                None,
                format!("Exposed secret: {secret_type}"),
            )
        } else {
            // Generic security event
            (
                event_type.to_string(),
                "medium".to_string(),
                None,
                None,
                "Security alert detected".to_string(),
            )
        };

    let repository = event
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let html_url = event
        .get("alert")
        .and_then(|a| a.get("html_url"))
        .or_else(|| event.get("html_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com")
        .to_string();

    super::types::SecurityAlert {
        alert_type,
        severity,
        cve_id,
        package_name,
        description,
        repository,
        branch: None,
        html_url,
        detected_at: Utc::now(),
    }
}

/// Check if we should process this failure.
fn should_process(failure: &CiFailure, _config: &RemediationConfig) -> bool {
    // Skip if not a failure
    if failure.conclusion != "failure" {
        return false;
    }

    // Skip bot commits to prevent loops (unless explicitly allowed)
    if failure.sender.ends_with("[bot]") {
        return false;
    }

    // Skip if commit message contains skip flag
    if failure.commit_message.contains("[skip-healer]")
        || failure.commit_message.contains("[healer skip]")
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workflow_job_event() {
        let event = serde_json::json!({
            "workflow_job": {
                "run_id": 12345,
                "workflow_name": "Controller CI",
                "name": "clippy",
                "conclusion": "failure",
                "head_branch": "main",
                "head_sha": "abc123",
                "html_url": "https://github.com/5dlabs/cto/actions/runs/12345"
            },
            "repository": {
                "full_name": "5dlabs/cto"
            },
            "sender": {
                "login": "developer"
            }
        });

        let failure = parse_ci_failure(&event).expect("Should parse");
        assert_eq!(failure.workflow_run_id, 12345);
        assert_eq!(failure.workflow_name, "Controller CI");
        assert_eq!(failure.job_name, Some("clippy".to_string()));
        assert_eq!(failure.conclusion, "failure");
    }

    #[test]
    fn test_should_process() {
        let config = RemediationConfig::default();

        let failure = CiFailure {
            workflow_run_id: 123,
            workflow_name: "CI".to_string(),
            job_name: None,
            conclusion: "failure".to_string(),
            branch: "main".to_string(),
            head_sha: "abc".to_string(),
            commit_message: "test commit".to_string(),
            html_url: "https://github.com".to_string(),
            repository: "test/repo".to_string(),
            sender: "developer".to_string(),
            detected_at: chrono::Utc::now(),
            raw_event: None,
        };

        assert!(should_process(&failure, &config));

        // Skip bot commits
        let mut bot_failure = failure.clone();
        bot_failure.sender = "dependabot[bot]".to_string();
        assert!(!should_process(&bot_failure, &config));

        // Skip with flag
        let mut skip_failure = failure.clone();
        skip_failure.commit_message = "test [skip-healer]".to_string();
        assert!(!should_process(&skip_failure, &config));

        // Skip non-failures
        let mut success = failure.clone();
        success.conclusion = "success".to_string();
        assert!(!should_process(&success, &config));
    }
}
