//! A2: Silent Agent Failure
//!
//! Detects when an agent container has terminated with a non-zero exit code
//! but the pod is still in "Running" phase (due to sidecar containers).

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::{ContainerState, K8sEvent};

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
        AlertId::A2
    }

    fn evaluate(
        &self,
        event: &K8sEvent,
        _github: &GitHubState,
        _ctx: &AlertContext,
    ) -> Option<Alert> {
        let (K8sEvent::PodModified(pod) | K8sEvent::PodRunning(pod)) = event else {
            return None;
        };

        // Only check pods that are still "Running"
        if pod.phase != "Running" {
            return None;
        }

        // Look for terminated containers with non-zero exit codes
        for container in &pod.container_statuses {
            if let ContainerState::Terminated {
                exit_code, reason, ..
            } = &container.state
            {
                if *exit_code != 0 {
                    return Some(
                        Alert::new(
                            AlertId::A2,
                            format!(
                                "Container '{}' terminated with exit code {} but pod still Running",
                                container.name, exit_code
                            ),
                        )
                        .with_severity(Severity::Critical)
                        .with_context("pod_name", pod.name.clone())
                        .with_context("container_name", container.name.clone())
                        .with_context("exit_code", exit_code.to_string())
                        .with_context("reason", reason.clone().unwrap_or_default())
                        .with_context("pod_phase", pod.phase.clone()),
                    );
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::types::AlertConfig;
    use crate::k8s::{ContainerStatus, Pod};

    #[test]
    fn test_detects_silent_failure() {
        let handler = Handler::new();

        let event = K8sEvent::PodModified(Pod {
            name: "rex-pod-123".into(),
            phase: "Running".into(),
            container_statuses: vec![
                ContainerStatus {
                    name: "factory-claude".into(),
                    state: ContainerState::Terminated {
                        exit_code: 1,
                        reason: Some("Error".into()),
                        finished_at: None,
                    },
                    restart_count: 0,
                },
                ContainerStatus {
                    name: "docker-daemon".into(),
                    state: ContainerState::Running,
                    restart_count: 0,
                },
            ],
            ..Default::default()
        });

        let github = GitHubState::default();
        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "agent-platform".into(),
            pr_number: None,
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(alert.id, AlertId::A2);
        assert!(alert.message.contains("factory-claude"));
        assert!(alert.message.contains("exit code 1"));
    }

    #[test]
    fn test_ignores_successful_termination() {
        let handler = Handler::new();

        let event = K8sEvent::PodModified(Pod {
            name: "rex-pod-123".into(),
            phase: "Running".into(),
            container_statuses: vec![ContainerStatus {
                name: "init-container".into(),
                state: ContainerState::Terminated {
                    exit_code: 0, // Success
                    reason: Some("Completed".into()),
                    finished_at: None,
                },
                restart_count: 0,
            }],
            ..Default::default()
        });

        let github = GitHubState::default();
        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "agent-platform".into(),
            pr_number: None,
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_none()); // Exit code 0 = no alert
    }
}
