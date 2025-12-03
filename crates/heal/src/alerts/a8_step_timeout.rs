//! A8: Workflow Step Timeout
//!
//! Detects when a workflow step has been running longer than
//! the configured threshold for that agent type.
//! Excludes infrastructure pods that restart during deployments.

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::{is_excluded_pod, K8sEvent};
use chrono::{Duration, Utc};

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    fn get_timeout_for_agent(agent: &str, ctx: &AlertContext) -> Duration {
        let mins = match agent {
            a if a.contains("Rex") || a.contains("Blaze") => {
                ctx.config.step_timeouts.implementation_mins
            }
            a if a.contains("Cleo") => ctx.config.step_timeouts.quality_mins,
            a if a.contains("Tess") => ctx.config.step_timeouts.testing_mins,
            a if a.contains("Cipher") => ctx.config.step_timeouts.security_mins,
            a if a.contains("Atlas") => ctx.config.step_timeouts.integration_mins,
            _ => ctx.config.step_timeouts.default_mins,
        };
        #[allow(clippy::cast_possible_wrap)] // timeout mins are small config values, won't wrap
        Duration::minutes(mins as i64)
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertHandler for Handler {
    fn id(&self) -> AlertId {
        AlertId::A8
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        _github: &GitHubState,
        ctx: &AlertContext,
    ) -> Option<Alert> {
        let (K8sEvent::PodRunning(pod) | K8sEvent::PodModified(pod)) = event else {
            return None;
        };

        // Check if pod is still running
        if pod.phase != "Running" {
            return None;
        }

        // Skip excluded infrastructure pods (heal, cto-tools, etc.)
        if is_excluded_pod(&pod.name) {
            return None;
        }

        // Get pod start time
        let started_at = pod.started_at?;
        let elapsed = Utc::now() - started_at;

        // Get agent and task_id from labels
        let agent = pod.labels.get("agent").cloned().unwrap_or_default();
        let task_id = pod.labels.get("task-id").cloned().unwrap_or_default();
        let threshold = Self::get_timeout_for_agent(&agent, ctx);

        if elapsed > threshold {
            return Some(
                Alert::new(
                    AlertId::A8,
                    format!(
                        "Step {} has been running for {} minutes (threshold: {} min)",
                        pod.name,
                        elapsed.num_minutes(),
                        threshold.num_minutes()
                    ),
                )
                .with_severity(Severity::Warning)
                .with_context("pod_name", pod.name.clone())
                .with_context("agent", agent)
                .with_context("task_id", task_id)
                .with_context("elapsed_minutes", elapsed.num_minutes().to_string())
                .with_context("threshold_minutes", threshold.num_minutes().to_string()),
            );
        }

        None
    }
}
