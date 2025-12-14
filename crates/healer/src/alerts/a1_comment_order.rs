//! A1: Agent Comment Order Mismatch
//!
//! Detects when an agent is running but the previous agent in the pipeline
//! hasn't posted a comment to the PR yet.
//!
//! Expected order: Rex/Blaze → Cleo → Tess → Cipher → Atlas

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::K8sEvent;

/// Expected agent comment order: Rex/Blaze → Cleo → Tess → Cipher → Atlas
const AGENT_ORDER: &[&str] = &[
    "5DLabs-Rex",
    "5DLabs-Blaze",
    "5DLabs-Cleo",
    "5DLabs-Tess",
    "5DLabs-Cipher",
    "5DLabs-Atlas",
];

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    /// Get the expected previous agents for a given agent based on `AGENT_ORDER`
    fn expected_previous_agents(current_agent: &str) -> Vec<&'static str> {
        // Find position of current agent in the order
        let current_pos = AGENT_ORDER.iter().position(|&a| current_agent.contains(a));

        match current_pos {
            // Unknown agent or Rex/Blaze (first agents) - no previous required
            None | Some(0 | 1) => vec![],
            Some(pos) => {
                // Return all agents that should come before this one
                // For Cleo (pos 2), return Rex and Blaze (pos 0, 1)
                // For Tess (pos 3), return Cleo (pos 2) - immediate predecessor
                // etc.
                if pos == 2 {
                    // Cleo needs Rex OR Blaze
                    vec![AGENT_ORDER[0], AGENT_ORDER[1]]
                } else {
                    // Others need their immediate predecessor
                    vec![AGENT_ORDER[pos - 1]]
                }
            }
        }
    }

    /// Check if any of the expected previous agents have commented
    fn has_previous_agent_commented(github: &GitHubState, expected: &[&str]) -> bool {
        if expected.is_empty() {
            return true; // No previous agent required
        }

        github
            .comments
            .iter()
            .any(|c| expected.iter().any(|agent| c.author.contains(agent)))
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertHandler for Handler {
    fn id(&self) -> AlertId {
        AlertId::A1
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        github: &GitHubState,
        _ctx: &AlertContext,
    ) -> Option<Alert> {
        // Only check when a pod starts running
        let K8sEvent::PodRunning(pod) = event else {
            return None;
        };

        // Get the agent from pod labels
        let current_agent = pod.labels.get("agent")?;

        // Get expected previous agents
        let expected = Self::expected_previous_agents(current_agent);
        if expected.is_empty() {
            return None; // First agent in pipeline, no check needed
        }

        // Check if previous agent has commented
        if Self::has_previous_agent_commented(github, &expected) {
            return None; // All good
        }

        // Alert: Previous agent hasn't commented
        let expected_str = expected.join(" or ");
        Some(
            Alert::new(
                AlertId::A1,
                format!("{current_agent} is running but no PR comment from {expected_str}"),
            )
            .with_severity(Severity::Warning)
            .with_context("current_agent", current_agent.clone())
            .with_context("expected_previous", expected_str)
            .with_context("pod_name", pod.name.clone())
            .with_context("pr_comments_count", github.comments.len().to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::types::AlertConfig;
    use crate::github::Comment;
    use crate::k8s::Pod;

    #[test]
    fn test_cleo_running_without_rex_comment() {
        let handler = Handler::new();

        let event = K8sEvent::PodRunning(Pod {
            name: "cleo-pod-123".into(),
            labels: [("agent".into(), "5DLabs-Cleo".into())]
                .into_iter()
                .collect(),
            ..Default::default()
        });

        let github = GitHubState {
            comments: vec![], // No comments yet
            ..Default::default()
        };

        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "agent-platform".into(),
            pr_number: Some(123),
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().id, AlertId::A1);
    }

    #[test]
    fn test_cleo_running_with_rex_comment() {
        let handler = Handler::new();

        let event = K8sEvent::PodRunning(Pod {
            name: "cleo-pod-123".into(),
            labels: [("agent".into(), "5DLabs-Cleo".into())]
                .into_iter()
                .collect(),
            ..Default::default()
        });

        let github = GitHubState {
            comments: vec![Comment {
                author: "5DLabs-Rex".into(),
                body: "PR created successfully".into(),
                created_at: chrono::Utc::now(),
            }],
            ..Default::default()
        };

        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "agent-platform".into(),
            pr_number: Some(123),
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_none()); // No alert - Rex already commented
    }
}
