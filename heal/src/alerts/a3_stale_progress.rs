//! A3: Stale Progress
//!
//! Detects when there are no new commits to the feature branch
//! for a threshold duration while an agent pod is running.

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::K8sEvent;
use chrono::{Duration, Utc};

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
        AlertId::A3
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        github: &GitHubState,
        ctx: &AlertContext,
    ) -> Option<Alert> {
        // Only check when a pod is running
        let pod = match event {
            K8sEvent::PodRunning(pod) => pod,
            _ => return None,
        };

        // Get the last commit time
        let last_commit = github.commits.last()?;
        let elapsed = Utc::now() - last_commit.committed_at;
        let threshold = Duration::minutes(ctx.config.stale_progress_threshold_mins as i64);

        if elapsed > threshold {
            return Some(
                Alert::new(
                    AlertId::A3,
                    format!(
                        "No commits for {} minutes while {} is running",
                        elapsed.num_minutes(),
                        pod.labels.get("agent").unwrap_or(&"unknown".to_string())
                    ),
                )
                .with_severity(Severity::Warning)
                .with_context("pod_name", pod.name.clone())
                .with_context("last_commit_sha", last_commit.sha.clone())
                .with_context("elapsed_minutes", elapsed.num_minutes().to_string())
                .with_context(
                    "threshold_minutes",
                    ctx.config.stale_progress_threshold_mins.to_string(),
                ),
            );
        }

        None
    }
}
