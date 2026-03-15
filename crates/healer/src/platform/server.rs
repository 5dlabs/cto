//! HTTP server for platform alert webhooks.

use crate::acp::{MonitorEventStore, StakpakMonitorEvent};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};

use super::alerts::PlatformAlertHandler;
use super::types::AlertmanagerPayload;
use super::workflow::WorkflowRemediator;

/// Server state for platform monitoring.
pub struct PlatformServerState {
    /// Platform alert handler
    pub platform_handler: PlatformAlertHandler,
    /// Workflow remediator
    pub workflow_handler: WorkflowRemediator,
    /// Ingested Stakpak monitor events
    pub monitor_events: MonitorEventStore,
}

impl PlatformServerState {
    /// Create new server state.
    #[must_use]
    pub fn new(namespace: &str, repository: &str) -> Self {
        Self {
            platform_handler: PlatformAlertHandler::new(namespace, repository),
            workflow_handler: WorkflowRemediator::new(namespace, repository),
            monitor_events: MonitorEventStore::default(),
        }
    }
}

/// Build the platform monitoring router.
pub fn build_platform_router(state: Arc<PlatformServerState>) -> Router {
    Router::new()
        .route("/api/alerts/platform", post(platform_alert_handler))
        .route("/api/alerts/workflows", post(workflow_alert_handler))
        .route("/api/v1/monitor-events/stakpak", post(stakpak_monitor_event_handler))
        .route("/api/platform/status", get(platform_status_handler))
        .route("/api/platform/remediations", get(remediations_handler))
        .with_state(state)
}

/// Response for alert handlers.
#[derive(Debug, Serialize)]
struct AlertResponse {
    status: &'static str,
    alerts_processed: usize,
    coderuns_spawned: Vec<String>,
    errors: Vec<String>,
}

/// Response for Stakpak monitor event ingestion.
#[derive(Debug, Serialize)]
struct MonitorEventResponse {
    status: &'static str,
    duplicate: bool,
    stored_events: usize,
}

/// Platform alert webhook handler.
async fn platform_alert_handler(
    State(state): State<Arc<PlatformServerState>>,
    Json(payload): Json<AlertmanagerPayload>,
) -> impl IntoResponse {
    let alert_count = payload.alerts.len();
    info!(
        "Received platform alert webhook: {} alerts, status={}",
        alert_count, payload.status
    );

    let mut coderuns = Vec::new();
    let mut errors = Vec::new();

    for alert in payload.alerts {
        let alert_name = alert.name().to_string();
        match state.platform_handler.process_alert(alert).await {
            Ok(Some(coderun_name)) => {
                coderuns.push(coderun_name);
            }
            Ok(None) => {
                // Alert was skipped (resolved, duplicate, etc.)
            }
            Err(e) => {
                error!("Failed to process platform alert {}: {e}", alert_name);
                errors.push(format!("{alert_name}: {e}"));
            }
        }
    }

    let status = if errors.is_empty() { "ok" } else { "partial" };

    (
        StatusCode::OK,
        Json(AlertResponse {
            status,
            alerts_processed: alert_count,
            coderuns_spawned: coderuns,
            errors,
        }),
    )
}

/// Workflow alert webhook handler.
async fn workflow_alert_handler(
    State(state): State<Arc<PlatformServerState>>,
    Json(payload): Json<AlertmanagerPayload>,
) -> impl IntoResponse {
    let alert_count = payload.alerts.len();
    info!(
        "Received workflow alert webhook: {} alerts, status={}",
        alert_count, payload.status
    );

    let mut coderuns = Vec::new();
    let mut errors = Vec::new();

    for alert in payload.alerts {
        let alert_name = alert.name().to_string();
        match state.workflow_handler.process_alert(alert).await {
            Ok(Some(coderun_name)) => {
                coderuns.push(coderun_name);
            }
            Ok(None) => {
                // Alert was skipped
            }
            Err(e) => {
                error!("Failed to process workflow alert {}: {e}", alert_name);
                errors.push(format!("{alert_name}: {e}"));
            }
        }
    }

    let status = if errors.is_empty() { "ok" } else { "partial" };

    (
        StatusCode::OK,
        Json(AlertResponse {
            status,
            alerts_processed: alert_count,
            coderuns_spawned: coderuns,
            errors,
        }),
    )
}

/// Stakpak monitor event ingestion endpoint.
async fn stakpak_monitor_event_handler(
    State(state): State<Arc<PlatformServerState>>,
    Json(payload): Json<StakpakMonitorEvent>,
) -> impl IntoResponse {
    let fingerprint = payload.fingerprint.clone();
    let duplicate = state.monitor_events.upsert(payload).await;
    let stored_events = state.monitor_events.len().await;

    info!(
        fingerprint = %fingerprint,
        duplicate,
        stored_events,
        "Processed Stakpak monitor event"
    );

    (
        StatusCode::ACCEPTED,
        Json(MonitorEventResponse {
            status: "accepted",
            duplicate,
            stored_events,
        }),
    )
}

/// Platform status response.
#[derive(Debug, Serialize)]
struct PlatformStatusResponse {
    status: &'static str,
    active_remediations: usize,
    components: ComponentStatus,
}

/// Component status.
#[derive(Debug, Serialize)]
struct ComponentStatus {
    prometheus_healthy: bool,
    loki_healthy: bool,
}

/// Platform status handler.
async fn platform_status_handler(
    State(state): State<Arc<PlatformServerState>>,
) -> impl IntoResponse {
    let prometheus_healthy = state
        .platform_handler
        .prometheus()
        .health_check()
        .await
        .unwrap_or(false);

    let loki_healthy = state
        .platform_handler
        .loki()
        .health_check()
        .await
        .unwrap_or(false);

    let remediations = state.platform_handler.get_remediations().await;
    let active = remediations
        .iter()
        .filter(|r| {
            matches!(
                r.status,
                super::types::RemediationStatus::Pending
                    | super::types::RemediationStatus::InProgress
            )
        })
        .count();

    let status = if prometheus_healthy && loki_healthy {
        "healthy"
    } else {
        "degraded"
    };

    Json(PlatformStatusResponse {
        status,
        active_remediations: active,
        components: ComponentStatus {
            prometheus_healthy,
            loki_healthy,
        },
    })
}

/// Remediations list handler.
async fn remediations_handler(
    State(state): State<Arc<PlatformServerState>>,
) -> impl IntoResponse {
    let remediations = state.platform_handler.get_remediations().await;
    Json(remediations)
}
