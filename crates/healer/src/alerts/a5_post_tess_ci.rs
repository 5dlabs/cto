//! A5: Post-Tess CI Failure
//!
//! Detects when Tess has approved the PR but CI checks are still failing.
//! This indicates either Tess approved incorrectly or something broke after approval.

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::{CheckConclusion, GitHubState};
use crate::k8s::K8sEvent;

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    fn tess_has_approved(github: &GitHubState) -> bool {
        github.comments.iter().any(|c| {
            c.author.contains("Tess")
                && (c.body.to_lowercase().contains("approved")
                    || c.body.to_lowercase().contains("lgtm")
                    || c.body.contains("✅"))
        })
    }

    fn ci_is_failing(github: &GitHubState) -> Vec<String> {
        github
            .checks
            .iter()
            .filter(|c| c.conclusion == CheckConclusion::Failure)
            .map(|c| c.name.clone())
            .collect()
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertHandler for Handler {
    fn id(&self) -> AlertId {
        AlertId::A5
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        github: &GitHubState,
        _ctx: &AlertContext,
    ) -> Option<Alert> {
        // Check on GitHub updates
        match event {
            K8sEvent::GitHubUpdate => {}
            _ => return None,
        }

        if !Self::tess_has_approved(github) {
            return None; // CI failures before Tess approval are expected
        }

        let failed_checks = Self::ci_is_failing(github);
        if failed_checks.is_empty() {
            return None; // CI is passing
        }

        Some(
            Alert::new(
                AlertId::A5,
                format!(
                    "Tess approved but {} CI checks are failing: {}",
                    failed_checks.len(),
                    failed_checks.join(", ")
                ),
            )
            .with_severity(Severity::Critical)
            .with_context("failed_checks", failed_checks.join(", "))
            .with_context("failed_count", failed_checks.len().to_string()),
        )
    }
}
