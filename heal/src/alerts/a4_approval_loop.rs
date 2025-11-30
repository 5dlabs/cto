//! A4: Repeated Approval Loop
//!
//! Detects when the same agent posts multiple approval comments
//! without the workflow advancing, indicating a potential infinite loop.

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::K8sEvent;
use std::collections::HashMap;

/// Keywords that indicate an approval
const APPROVAL_KEYWORDS: &[&str] = &[
    "approved",
    "lgtm",
    "looks good",
    "passing",
    "✅",
    "all checks pass",
];

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    fn is_approval_comment(body: &str) -> bool {
        let lower = body.to_lowercase();
        APPROVAL_KEYWORDS.iter().any(|kw| lower.contains(kw))
    }

    fn count_approvals_by_author(github: &GitHubState) -> HashMap<String, u32> {
        let mut counts = HashMap::new();

        for comment in &github.comments {
            if Self::is_approval_comment(&comment.body) {
                *counts.entry(comment.author.clone()).or_insert(0) += 1;
            }
        }

        counts
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertHandler for Handler {
    fn id(&self) -> AlertId {
        AlertId::A4
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        github: &GitHubState,
        ctx: &AlertContext,
    ) -> Option<Alert> {
        // Check on any GitHub state update or pod event
        match event {
            K8sEvent::GitHubUpdate | K8sEvent::PodRunning(_) | K8sEvent::PodModified(_) => {}
            _ => return None,
        }

        let approval_counts = Self::count_approvals_by_author(github);

        for (agent, count) in &approval_counts {
            if *count > ctx.config.approval_loop_threshold {
                return Some(
                    Alert::new(
                        AlertId::A4,
                        format!(
                            "{} has posted {} approvals - possible infinite loop",
                            agent, count
                        ),
                    )
                    .with_severity(Severity::Warning)
                    .with_context("agent", agent.clone())
                    .with_context("approval_count", count.to_string())
                    .with_context("threshold", ctx.config.approval_loop_threshold.to_string()),
                );
            }
        }

        None
    }
}
