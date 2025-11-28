/*
 * 5D Labs Agent Platform - Controller Service
 * Copyright (C) 2025 5D Labs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Controller Service - Kubernetes Controller for `CodeRun` and `DocsRun` CRDs
//!
//! This service manages the lifecycle of AI agent jobs by:
//! - Watching for `CodeRun` and `DocsRun` custom resources
//! - Creating and managing Kubernetes Jobs for agent execution
//! - Handling resource cleanup and status updates
//! - Providing health and metrics endpoints

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use controller::remediation::RemediationStateManager;
use controller::tasks::label::client::GitHubLabelClient;
use controller::tasks::{
    config::ControllerConfig,
    label::{override_detector::OverrideDetector, schema::WorkflowState, LabelOrchestrator},
    run_task_controller,
    types::Context as TaskContext,
};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::Api;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
#[allow(dead_code)] // Fields are read via axum State extractor
struct AppState {
    client: kube::Client,
    namespace: String,
    config: Arc<ControllerConfig>,
    remediation_state_manager: Arc<RemediationStateManager>,
}

/// Verify that all required agent template `ConfigMaps` exist and are healthy
///
/// This health check runs at controller startup to fail-fast if `ConfigMaps` are missing.
/// Prevents 8-hour silent retry loops when template generation cannot succeed.
async fn verify_required_configmaps(
    client: &kube::Client,
    namespace: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);

    // Get the ConfigMap prefix from environment (set by Helm based on release name)
    // Defaults to "controller" for backward compatibility
    let cm_prefix = std::env::var("CONFIGMAP_PREFIX").unwrap_or_else(|_| "controller".to_string());

    let required_configmaps = vec![
        (
            format!("{cm_prefix}-agent-templates-claude-code"),
            "Claude code templates",
        ),
        (
            format!("{cm_prefix}-agent-templates-claude-docs"),
            "Claude docs templates",
        ),
        (
            format!("{cm_prefix}-agent-templates-codex"),
            "Codex agent templates",
        ),
        (
            format!("{cm_prefix}-agent-templates-cursor"),
            "Cursor agent templates",
        ),
        (
            format!("{cm_prefix}-agent-templates-factory"),
            "Factory agent templates",
        ),
        (
            format!("{cm_prefix}-agent-templates-integration"),
            "Integration agent templates",
        ),
        (
            format!("{cm_prefix}-agent-templates-shared"),
            "Shared agent utilities",
        ),
        (
            format!("{cm_prefix}-agent-templates-watch"),
            "E2E Watch agent templates",
        ),
    ];

    let mut missing = Vec::new();
    let mut empty = Vec::new();

    for (cm_name, description) in &required_configmaps {
        match configmaps.get(cm_name).await {
            Ok(cm) => {
                // Check if ConfigMap has data (could be in .data or .binaryData fields)
                let data_count = cm.data.as_ref().map_or(0, std::collections::BTreeMap::len);
                let binary_count = cm
                    .binary_data
                    .as_ref()
                    .map_or(0, std::collections::BTreeMap::len);
                let total_files = data_count + binary_count;

                if total_files == 0 {
                    empty.push(format!("{cm_name} ({description})"));
                    error!("❌ ConfigMap {} exists but is EMPTY", cm_name);
                } else {
                    info!("  ✓ {} - {} files", description, total_files);
                }
            }
            Err(e) => {
                missing.push(format!("{cm_name} ({description})"));
                error!("❌ ConfigMap {} NOT FOUND: {}", cm_name, e);
            }
        }
    }

    if !missing.is_empty() || !empty.is_empty() {
        error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        error!("❌ CRITICAL: Required ConfigMaps are unavailable");
        error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if !missing.is_empty() {
            error!("Missing ConfigMaps:");
            for cm in &missing {
                error!("  - {}", cm);
            }
        }

        if !empty.is_empty() {
            error!("Empty ConfigMaps:");
            for cm in &empty {
                error!("  - {}", cm);
            }
        }

        error!("");
        error!("Controller cannot start without these ConfigMaps.");
        error!("They contain agent templates required for job creation.");
        error!("");
        error!("Possible causes:");
        error!("  1. ArgoCD hasn't synced yet (check: kubectl get app controller -n argocd)");
        error!("  2. Helm chart not deployed properly");
        error!("  3. ConfigMaps were manually deleted");
        error!("");
        error!("To fix:");
        error!("  1. Check ArgoCD sync status");
        error!("  2. Verify Helm values.yaml has agent templates enabled");
        error!("  3. Re-run: helm upgrade controller ./charts/controller");
        error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        return Err(format!(
            "Missing {} ConfigMaps, {} empty ConfigMaps",
            missing.len(),
            empty.len()
        )
        .into());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,core=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting 5D Labs Controller Service v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize Kubernetes client and controller
    let client = kube::Client::try_default().await?;
    info!("Connected to Kubernetes cluster");

    let namespace = "cto".to_string();
    let controller_config = Arc::new(load_controller_config());

    // Verify required ConfigMaps exist before starting controller
    // This prevents 8-hour silent retry loops when ConfigMaps are broken
    info!("Verifying required ConfigMaps are available...");
    verify_required_configmaps(&client, &namespace).await?;
    info!("✅ All required ConfigMaps verified");

    // Create shared remediation state manager for webhook reuse
    let task_context = TaskContext {
        client: client.clone(),
        namespace: namespace.clone(),
        config: controller_config.clone(),
    };
    let remediation_state_manager = Arc::new(RemediationStateManager::new(&task_context));

    let state = AppState {
        client: client.clone(),
        namespace: namespace.clone(),
        config: controller_config.clone(),
        remediation_state_manager,
    };

    // Start the controller in the background
    let controller_handle = {
        let client = client.clone();
        let namespace = namespace.clone();
        tokio::spawn(async move {
            if let Err(e) = run_task_controller(client, namespace).await {
                tracing::error!("Controller error: {}", e);
            }
        })
    };

    // Build the HTTP router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics))
        .route("/webhook", post(webhook_handler))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(CorsLayer::permissive())
                .layer(TimeoutLayer::new(Duration::from_secs(60))),
        )
        .with_state(state);

    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("Controller HTTP server listening on 0.0.0.0:8080");

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Wait for controller to finish
    controller_handle.abort();
    info!("Controller service stopped");

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "controller",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn readiness_check(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Check if controller is ready (basic check)
    Json(json!({
        "status": "ready",
        "service": "controller",
        "version": env!("CARGO_PKG_VERSION")
    }))
    .pipe(Ok)
}

async fn metrics() -> Json<Value> {
    // Basic metrics endpoint - can be extended with prometheus metrics
    Json(json!({
        "service": "controller",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

fn load_controller_config() -> ControllerConfig {
    let override_path = std::env::var("CONTROLLER_CONFIG_PATH").ok();
    let config_path = override_path
        .as_deref()
        .filter(|path| Path::new(path).exists())
        .unwrap_or("/config/config.yaml");

    match ControllerConfig::from_mounted_file(config_path) {
        Ok(cfg) => {
            info!("Loaded controller configuration from {}", config_path);
            cfg
        }
        Err(err) => {
            warn!(
                "Failed to load configuration from {}: {}. Using defaults.",
                config_path, err
            );
            ControllerConfig::default()
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    const LABEL_NEEDS_FIXES: &str = "needs-fixes";
    const LABEL_FIXING_IN_PROGRESS: &str = "fixing-in-progress";
    const LABEL_NEEDS_CLEO: &str = "needs-cleo";
    const LABEL_NEEDS_TESS: &str = "needs-tess";
    const LABEL_APPROVED: &str = "approved";
    const LABEL_FAILED: &str = "failed-remediation";

    let event = headers
        .get("X-GitHub-Event")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if event != "pull_request" {
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "unsupported_event"
        })));
    }

    let payload: Value = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    if action != "labeled" {
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "non_labeled_action"
        })));
    }

    let label_name = payload
        .get("label")
        .and_then(|label| label.get("name"))
        .and_then(|name| name.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let pr_number = payload
        .get("pull_request")
        .and_then(|pr| pr.get("number"))
        .and_then(serde_json::Value::as_i64)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let repo_owner = payload
        .get("repository")
        .and_then(|repo| repo.get("owner"))
        .and_then(|owner| owner.get("login"))
        .and_then(|login| login.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let repo_name = payload
        .get("repository")
        .and_then(|repo| repo.get("name"))
        .and_then(|name| name.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let target_state = match label_name {
        LABEL_NEEDS_FIXES => Some(WorkflowState::NeedsFixes),
        LABEL_FIXING_IN_PROGRESS => Some(WorkflowState::FixingInProgress),
        LABEL_NEEDS_CLEO => Some(WorkflowState::NeedsCleo),
        LABEL_NEEDS_TESS => Some(WorkflowState::NeedsTess),
        LABEL_APPROVED => Some(WorkflowState::Approved),
        LABEL_FAILED => Some(WorkflowState::Failed),
        _ => None,
    };

    if target_state.is_none() {
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "non_state_label"
        })));
    }

    let task_label = payload
        .get("pull_request")
        .and_then(|pr| pr.get("labels"))
        .and_then(|labels| labels.as_array())
        .and_then(|labels| {
            labels.iter().find_map(|label| {
                label
                    .get("name")
                    .and_then(|name| name.as_str())
                    .filter(|name| name.starts_with("task-"))
            })
        })
        .ok_or(StatusCode::ACCEPTED)?;

    let task_id = task_label.to_string();

    let Ok(token) = std::env::var("GITHUB_TOKEN") else {
        warn!(
            "GITHUB_TOKEN not set; skipping orchestrator update for label '{}'",
            label_name
        );
        return Ok(Json(json!({
            "status": "skipped",
            "reason": "missing_token"
        })));
    };

    let label_client =
        GitHubLabelClient::with_token(token, repo_owner.to_string(), repo_name.to_string());

    // Reuse the shared state manager from AppState instead of creating new one per webhook
    let override_detector = OverrideDetector::new(label_client.clone());
    let mut orchestrator = LabelOrchestrator::new(
        label_client,
        state.remediation_state_manager.clone(),
        override_detector,
    );

    match state
        .remediation_state_manager
        .load_state(u32::try_from(pr_number).unwrap_or(0), &task_id)
        .await
    {
        Ok(None) => {
            if let Err(err) = state
                .remediation_state_manager
                .initialize_state(u32::try_from(pr_number).unwrap_or(0), task_id.clone(), None)
                .await
            {
                warn!(
                    "Failed to initialize remediation state for PR #{} (task {}): {}",
                    pr_number, task_id, err
                );
            }
        }
        Err(err) => {
            warn!(
                "Failed to load remediation state for PR #{} (task {}): {}",
                pr_number, task_id, err
            );
        }
        _ => {}
    }

    if let Err(err) = orchestrator
        .force_state(
            i32::try_from(pr_number).unwrap_or(0),
            &task_id,
            target_state.unwrap(),
        )
        .await
    {
        error!(
            "Failed to update remediation state for PR #{} (task {}): {}",
            pr_number, task_id, err
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(json!({
        "status": "ok",
        "label": label_name,
        "task": task_id,
        "pr": pr_number
    })))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully");
        },
        () = terminate => {
            info!("Received SIGTERM, shutting down gracefully");
        },
    }
}

// Helper trait for more ergonomic Result handling
trait Pipe<T> {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R;
}

impl<T> Pipe<T> for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        f(self)
    }
}
