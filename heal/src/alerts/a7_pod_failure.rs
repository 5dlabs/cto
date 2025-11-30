//! A7: Pod Failure
//!
//! Detects when any CTO pod enters Failed, Error, or `CrashLoopBackOff` state.
//! Excludes infrastructure pods that restart during deployments.

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::K8sEvent;

/// Pod name prefixes to exclude from failure alerts.
/// These are infrastructure pods that may restart during deployments.
const EXCLUDED_POD_PREFIXES: &[&str] = &[
    "heal",
    "cto-tools",
    "cto-controller",
    "vault-mcp-server",
    "openmemory",
    "event-cleaner",
    "workspace-pvc-cleaner",
];

/// Check if a pod name should be excluded from alerts
fn is_excluded(pod_name: &str) -> bool {
    EXCLUDED_POD_PREFIXES
        .iter()
        .any(|prefix| pod_name.starts_with(prefix))
}

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertHandler for Handler {
    fn id(&self) -> AlertId {
        AlertId::A7
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        _github: &GitHubState,
        _ctx: &AlertContext,
    ) -> Option<Alert> {
        let pod = match event {
            K8sEvent::PodFailed(pod) => pod,
            K8sEvent::PodModified(pod) if pod.phase == "Failed" || pod.phase == "Error" => pod,
            _ => return None,
        };

        // Skip excluded infrastructure pods
        if is_excluded(&pod.name) {
            return None;
        }

        // Check for CrashLoopBackOff
        let restart_count: i32 = pod
            .container_statuses
            .iter()
            .map(|c| c.restart_count)
            .sum();

        let is_crash_loop = restart_count > 3;

        let severity = if is_crash_loop {
            Severity::Critical
        } else {
            Severity::Warning
        };

        let message = if is_crash_loop {
            format!(
                "Pod {} in CrashLoopBackOff ({} restarts)",
                pod.name, restart_count
            )
        } else {
            format!("Pod {} failed with phase: {}", pod.name, pod.phase)
        };

        Some(
            Alert::new(AlertId::A7, message)
                .with_severity(severity)
                .with_context("pod_name", pod.name.clone())
                .with_context("phase", pod.phase.clone())
                .with_context("restart_count", restart_count.to_string())
                .with_context(
                    "agent",
                    pod.labels.get("agent").cloned().unwrap_or_default(),
                )
                .with_context(
                    "task_id",
                    pod.labels.get("task-id").cloned().unwrap_or_default(),
                ),
        )
    }
}
