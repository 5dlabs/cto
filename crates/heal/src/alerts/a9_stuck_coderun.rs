//! A9: Stuck `CodeRun`
//!
//! Detects when a `CodeRun` has been in a non-terminal state (Running/Pending)
//! for longer than the configured threshold. This catches `CodeRuns` where the
//! agent process has stopped making progress without transitioning to
//! Succeeded or Failed.

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
        AlertId::A9
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        _github: &GitHubState,
        ctx: &AlertContext,
    ) -> Option<Alert> {
        // Only check CodeRun events
        let K8sEvent::CodeRunChanged(coderun) = event else {
            return None;
        };

        // Only alert on non-terminal phases
        let phase = coderun.phase.as_str();
        if phase == "Succeeded" || phase == "Failed" {
            return None;
        }

        // Check if we have a creation timestamp to calculate elapsed time
        // For CodeRuns, we track the time since we first observed them in a non-terminal state
        // This is handled by the caller tracking first-seen timestamps
        //
        // For now, we emit the alert immediately when we see a CodeRun in Running/Pending
        // that has been running longer than the threshold. The caller (run_alert_watch)
        // tracks timestamps and only calls evaluate when the threshold has passed.

        #[allow(clippy::cast_possible_wrap)]
        let threshold_mins = ctx.config.stuck_coderun_threshold_mins as i64;

        Some(
            Alert::new(
                AlertId::A9,
                format!(
                    "CodeRun {} has been in '{}' state for over {} minutes without completing",
                    coderun.name, phase, threshold_mins
                ),
            )
            .with_severity(Severity::Warning)
            .with_context("coderun_name", coderun.name.clone())
            .with_context("coderun_phase", phase.to_string())
            .with_context("agent", coderun.agent.clone())
            .with_context("task_id", coderun.task_id.clone())
            .with_context(
                "threshold_minutes",
                ctx.config.stuck_coderun_threshold_mins.to_string(),
            ),
        )
    }
}

/// Tracks `CodeRun` timestamps for detecting stuck `CodeRuns`.
#[derive(Default)]
pub struct CodeRunTracker {
    /// Map of `CodeRun` name to first-seen timestamp.
    first_seen: std::collections::HashMap<String, chrono::DateTime<Utc>>,
}

impl CodeRunTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record when we first saw a `CodeRun` (if not already tracked).
    pub fn record_first_seen(&mut self, name: &str) {
        self.first_seen
            .entry(name.to_string())
            .or_insert_with(Utc::now);
    }

    /// Check if a `CodeRun` has exceeded the threshold duration.
    pub fn exceeds_threshold(&self, name: &str, threshold_mins: u64) -> bool {
        if let Some(first_seen) = self.first_seen.get(name) {
            let elapsed = Utc::now() - *first_seen;
            #[allow(clippy::cast_possible_wrap)]
            let threshold = Duration::minutes(threshold_mins as i64);
            return elapsed > threshold;
        }
        false
    }

    /// Remove a `CodeRun` from tracking (when it completes).
    pub fn remove(&mut self, name: &str) {
        self.first_seen.remove(name);
    }
}
