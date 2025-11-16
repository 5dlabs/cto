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

    let namespace = "agent-platform".to_string();
    let controller_config = Arc::new(load_controller_config());

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

async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
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

    const LABEL_NEEDS_FIXES: &str = "needs-fixes";
    const LABEL_FIXING_IN_PROGRESS: &str = "fixing-in-progress";
    const LABEL_NEEDS_CLEO: &str = "needs-cleo";
    const LABEL_NEEDS_TESS: &str = "needs-tess";
    const LABEL_APPROVED: &str = "approved";
    const LABEL_FAILED: &str = "failed-remediation";

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

    let token = if let Ok(value) = std::env::var("GITHUB_TOKEN") {
        value
    } else {
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
        .load_state(pr_number as u32, &task_id)
        .await
    {
        Ok(None) => {
            if let Err(err) = state
                .remediation_state_manager
                .initialize_state(pr_number as u32, task_id.clone(), None)
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
        .force_state(pr_number as i32, &task_id, target_state.unwrap())
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
