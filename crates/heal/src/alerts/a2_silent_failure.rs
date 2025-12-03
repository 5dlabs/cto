//! A2: Silent Agent Failure
//!
//! Detects when an agent container has failed but the pod is still in "Running"
//! phase (due to sidecar containers keeping the pod alive).
//!
//! Uses multiple detection methods with priority-based deduplication:
//! 1. Exit code: Container terminated with non-zero exit code (highest priority)
//! 2. Ready status: Container ready=false while pod Running
//! 3. Pod conditions: ContainersReady=False condition
//! 4. Restart count: High restart count indicates instability
//! 5. Terminated duration: Container dead for extended period

use super::types::{Alert, AlertContext, AlertHandler, AlertId, Severity};
use crate::github::GitHubState;
use crate::k8s::{ContainerState, K8sEvent, Pod};
use chrono::{Duration, Utc};

/// Restart count threshold for crash loop detection
const RESTART_THRESHOLD: i32 = 3;

/// Seconds before terminated container is flagged by duration check
const TERMINATED_DURATION_SECS: i64 = 60;

/// Startup grace period in seconds.
/// Containers not ready within this period after starting are not flagged as silent failures.
/// This prevents false positives during normal pod initialization when readiness probes
/// haven't passed yet.
const STARTUP_GRACE_PERIOD_SECS: i64 = 30;

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

/// Detection method attribution for debugging and context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    ExitCode,
    ReadyStatus,
    PodCondition,
    RestartCount,
    Duration,
}

impl Method {
    fn priority(self) -> u8 {
        match self {
            Self::ExitCode => 1,     // Most definitive
            Self::ReadyStatus => 2,  // Strong signal
            Self::PodCondition => 3, // Good fallback
            Self::RestartCount => 4, // Indicates instability
            Self::Duration => 5,     // Catch-all for stale failures
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ExitCode => "exit_code",
            Self::ReadyStatus => "ready_status",
            Self::PodCondition => "pod_condition",
            Self::RestartCount => "restart_count",
            Self::Duration => "terminated_duration",
        }
    }
}

struct Detection {
    method: Method,
    container: String,
    message: String,
    exit_code: Option<i32>,
    reason: Option<String>,
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

        // Run all detection methods and pick highest priority
        let detection = detect_silent_failure(pod)?;

        let mut alert = Alert::new(AlertId::A2, detection.message)
            .with_severity(Severity::Critical)
            .with_context("pod_name", pod.name.clone())
            .with_context("container_name", detection.container)
            .with_context("detection_method", detection.method.as_str())
            .with_context("pod_phase", pod.phase.clone());

        if let Some(code) = detection.exit_code {
            alert = alert.with_context("exit_code", code.to_string());
        }
        if let Some(reason) = detection.reason {
            alert = alert.with_context("reason", reason);
        }

        Some(alert)
    }
}

/// Run all detection methods and return the highest-priority detection.
fn detect_silent_failure(pod: &Pod) -> Option<Detection> {
    let mut detections = Vec::new();

    if let Some(d) = detect_by_exit_code(pod) {
        detections.push(d);
    }
    if let Some(d) = detect_by_ready_status(pod) {
        detections.push(d);
    }
    if let Some(d) = detect_by_pod_conditions(pod) {
        detections.push(d);
    }
    if let Some(d) = detect_by_restart_count(pod) {
        detections.push(d);
    }
    if let Some(d) = detect_by_terminated_duration(pod) {
        detections.push(d);
    }

    // Return highest priority (lowest number)
    detections.into_iter().min_by_key(|d| d.method.priority())
}

/// Method 1: Container terminated with non-zero exit code
fn detect_by_exit_code(pod: &Pod) -> Option<Detection> {
    for container in &pod.container_statuses {
        if let ContainerState::Terminated {
            exit_code, reason, ..
        } = &container.state
        {
            if *exit_code != 0 {
                return Some(Detection {
                    method: Method::ExitCode,
                    container: container.name.clone(),
                    message: format!(
                        "Container '{}' terminated with exit code {} but pod still Running",
                        container.name, exit_code
                    ),
                    exit_code: Some(*exit_code),
                    reason: reason.clone(),
                });
            }
        }
    }
    None
}

/// Method 2: Container not ready while pod is Running
fn detect_by_ready_status(pod: &Pod) -> Option<Detection> {
    let now = Utc::now();

    for container in &pod.container_statuses {
        if !container.ready {
            // Skip if container is in waiting state (startup phase)
            if matches!(container.state, ContainerState::Waiting { .. }) {
                continue;
            }

            // Skip if container is in running state but within startup grace period.
            // This prevents false positives during normal pod initialization when
            // readiness probes haven't passed yet.
            if let ContainerState::Running {
                started_at: Some(start_time),
            } = &container.state
            {
                let running_duration = now.signed_duration_since(*start_time);
                if running_duration < Duration::seconds(STARTUP_GRACE_PERIOD_SECS) {
                    continue;
                }
            }

            return Some(Detection {
                method: Method::ReadyStatus,
                container: container.name.clone(),
                message: format!("Container '{}' not ready while pod Running", container.name),
                exit_code: None,
                reason: None,
            });
        }
    }
    None
}

/// Method 3: Pod condition ContainersReady=False
fn detect_by_pod_conditions(pod: &Pod) -> Option<Detection> {
    // Skip if pod is within startup grace period.
    // This prevents false positives during normal pod initialization.
    if let Some(started_at) = pod.started_at {
        let now = Utc::now();
        let pod_age = now.signed_duration_since(started_at);
        if pod_age < Duration::seconds(STARTUP_GRACE_PERIOD_SECS) {
            return None;
        }
    }

    // Check ContainersReady condition first (more specific)
    for condition in &pod.conditions {
        if condition.condition_type == "ContainersReady" && condition.status == "False" {
            let container_name = condition
                .message
                .as_ref()
                .and_then(|m| {
                    // Parse "containers with unready status: [factory-claude-opus-4-5-20251101]"
                    m.strip_prefix("containers with unready status: [")
                        .and_then(|s| s.strip_suffix(']'))
                        .map(String::from)
                })
                .unwrap_or_else(|| "unknown".into());

            return Some(Detection {
                method: Method::PodCondition,
                container: container_name,
                message: format!(
                    "Pod condition ContainersReady=False: {}",
                    condition.reason.as_deref().unwrap_or("unknown")
                ),
                exit_code: None,
                reason: condition.reason.clone(),
            });
        }
    }

    // Fall back to Ready condition
    for condition in &pod.conditions {
        if condition.condition_type == "Ready" && condition.status == "False" {
            return Some(Detection {
                method: Method::PodCondition,
                container: "unknown".into(),
                message: format!(
                    "Pod condition Ready=False: {}",
                    condition.reason.as_deref().unwrap_or("unknown")
                ),
                exit_code: None,
                reason: condition.reason.clone(),
            });
        }
    }

    None
}

/// Method 4: High restart count indicates crash loop
fn detect_by_restart_count(pod: &Pod) -> Option<Detection> {
    for container in &pod.container_statuses {
        if container.restart_count >= RESTART_THRESHOLD {
            return Some(Detection {
                method: Method::RestartCount,
                container: container.name.clone(),
                message: format!(
                    "Container '{}' has restarted {} times (threshold: {})",
                    container.name, container.restart_count, RESTART_THRESHOLD
                ),
                exit_code: None,
                reason: None,
            });
        }
    }
    None
}

/// Method 5: Container terminated for extended duration
fn detect_by_terminated_duration(pod: &Pod) -> Option<Detection> {
    let now = Utc::now();

    for container in &pod.container_statuses {
        if let ContainerState::Terminated {
            finished_at: Some(finished),
            exit_code,
            reason,
        } = &container.state
        {
            let duration = now.signed_duration_since(*finished);
            if duration > Duration::seconds(TERMINATED_DURATION_SECS) {
                return Some(Detection {
                    method: Method::Duration,
                    container: container.name.clone(),
                    message: format!(
                        "Container '{}' has been terminated for {}s (threshold: {}s)",
                        container.name,
                        duration.num_seconds(),
                        TERMINATED_DURATION_SECS
                    ),
                    exit_code: Some(*exit_code),
                    reason: reason.clone(),
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::types::AlertConfig;
    use crate::k8s::{ContainerStatus, PodCondition};

    #[test]
    fn test_detects_silent_failure_by_exit_code() {
        let handler = Handler::new();

        let event = K8sEvent::PodModified(Pod {
            name: "rex-pod-123".into(),
            phase: "Running".into(),
            container_statuses: vec![
                ContainerStatus {
                    name: "factory-claude".into(),
                    ready: false,
                    state: ContainerState::Terminated {
                        exit_code: 1,
                        reason: Some("Error".into()),
                        finished_at: None,
                    },
                    restart_count: 0,
                },
                ContainerStatus {
                    name: "docker-daemon".into(),
                    ready: true,
                    state: ContainerState::Running { started_at: None },
                    restart_count: 0,
                },
            ],
            ..Default::default()
        });

        let github = GitHubState::default();
        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "cto".into(),
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
        assert_eq!(
            alert.context.get("detection_method"),
            Some(&"exit_code".to_string())
        );
    }

    #[test]
    fn test_ignores_successful_termination() {
        let handler = Handler::new();

        let event = K8sEvent::PodModified(Pod {
            name: "rex-pod-123".into(),
            phase: "Running".into(),
            conditions: vec![PodCondition {
                condition_type: "Ready".into(),
                status: "True".into(),
                ..Default::default()
            }],
            container_statuses: vec![ContainerStatus {
                name: "init-container".into(),
                ready: true,
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
            namespace: "cto".into(),
            pr_number: None,
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_none()); // Exit code 0 = no alert
    }

    #[test]
    fn test_detects_by_pod_condition() {
        let handler = Handler::new();

        let event = K8sEvent::PodModified(Pod {
            name: "rex-pod-123".into(),
            phase: "Running".into(),
            // Pod started well before grace period (2 minutes ago)
            started_at: Some(Utc::now() - Duration::minutes(2)),
            conditions: vec![PodCondition {
                condition_type: "ContainersReady".into(),
                status: "False".into(),
                reason: Some("ContainersNotReady".into()),
                message: Some("containers with unready status: [factory-claude]".into()),
            }],
            container_statuses: vec![], // Empty - fallback to conditions
            ..Default::default()
        });

        let github = GitHubState::default();
        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "cto".into(),
            pr_number: None,
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(
            alert.context.get("detection_method"),
            Some(&"pod_condition".to_string())
        );
        assert_eq!(
            alert.context.get("container_name"),
            Some(&"factory-claude".to_string())
        );
    }

    #[test]
    fn test_detects_by_restart_count() {
        let handler = Handler::new();

        let event = K8sEvent::PodModified(Pod {
            name: "crashloop-pod".into(),
            phase: "Running".into(),
            conditions: vec![PodCondition {
                condition_type: "Ready".into(),
                status: "True".into(), // Ready is true (no condition trigger)
                ..Default::default()
            }],
            container_statuses: vec![ContainerStatus {
                name: "unstable".into(),
                ready: true, // Ready (no ready status trigger)
                state: ContainerState::Running { started_at: None }, // Running (no exit code trigger)
                restart_count: 5,                                    // High restart count
            }],
            ..Default::default()
        });

        let github = GitHubState::default();
        let ctx = AlertContext {
            task_id: "1".into(),
            repository: "5dlabs/test".into(),
            namespace: "cto".into(),
            pr_number: None,
            workflow_name: None,
            config: AlertConfig::default(),
        };

        let alert = handler.evaluate(&event, &github, &ctx);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(
            alert.context.get("detection_method"),
            Some(&"restart_count".to_string())
        );
    }

    #[test]
    fn test_priority_exit_code_over_ready_status() {
        // When multiple detection methods trigger, exit code should win
        let pod = Pod {
            name: "multi-signal".into(),
            phase: "Running".into(),
            conditions: vec![PodCondition {
                condition_type: "ContainersReady".into(),
                status: "False".into(),
                reason: Some("ContainersNotReady".into()),
                message: Some("containers with unready status: [main]".into()),
            }],
            container_statuses: vec![ContainerStatus {
                name: "main".into(),
                ready: false, // Would trigger ready_status
                state: ContainerState::Terminated {
                    exit_code: 1, // Would trigger exit_code
                    reason: Some("Error".into()),
                    finished_at: Some(Utc::now() - Duration::minutes(5)), // Would trigger duration
                },
                restart_count: 0,
            }],
            ..Default::default()
        };

        let detection = detect_silent_failure(&pod);
        assert!(detection.is_some());
        // Exit code has priority 1 (highest)
        assert_eq!(detection.unwrap().method, Method::ExitCode);
    }

    #[test]
    fn test_healthy_pod_no_detection() {
        let pod = Pod {
            name: "healthy".into(),
            phase: "Running".into(),
            conditions: vec![
                PodCondition {
                    condition_type: "Ready".into(),
                    status: "True".into(),
                    ..Default::default()
                },
                PodCondition {
                    condition_type: "ContainersReady".into(),
                    status: "True".into(),
                    ..Default::default()
                },
            ],
            container_statuses: vec![
                ContainerStatus {
                    name: "main".into(),
                    ready: true,
                    state: ContainerState::Running { started_at: None },
                    restart_count: 0,
                },
                ContainerStatus {
                    name: "sidecar".into(),
                    ready: true,
                    state: ContainerState::Running { started_at: None },
                    restart_count: 0,
                },
            ],
            ..Default::default()
        };

        assert!(detect_silent_failure(&pod).is_none());
    }

    #[test]
    fn test_waiting_container_not_flagged() {
        let pod = Pod {
            name: "starting".into(),
            phase: "Running".into(),
            container_statuses: vec![ContainerStatus {
                name: "main".into(),
                ready: false, // Not ready yet
                state: ContainerState::Waiting {
                    reason: Some("ContainerCreating".into()),
                },
                restart_count: 0,
            }],
            ..Default::default()
        };

        // Ready status check should NOT fire during startup
        assert!(detect_by_ready_status(&pod).is_none());
    }

    #[test]
    fn test_startup_grace_period_ready_status() {
        // Container just started (10 seconds ago) - should NOT trigger alert
        let recent_pod = Pod {
            name: "starting".into(),
            phase: "Running".into(),
            container_statuses: vec![ContainerStatus {
                name: "main".into(),
                ready: false, // Not ready yet
                state: ContainerState::Running {
                    started_at: Some(Utc::now() - Duration::seconds(10)),
                },
                restart_count: 0,
            }],
            ..Default::default()
        };
        assert!(detect_by_ready_status(&recent_pod).is_none());

        // Container started long ago (2 minutes) - SHOULD trigger alert
        let old_pod = Pod {
            name: "stale".into(),
            phase: "Running".into(),
            container_statuses: vec![ContainerStatus {
                name: "main".into(),
                ready: false, // Still not ready after 2 minutes
                state: ContainerState::Running {
                    started_at: Some(Utc::now() - Duration::minutes(2)),
                },
                restart_count: 0,
            }],
            ..Default::default()
        };
        assert!(detect_by_ready_status(&old_pod).is_some());
    }

    #[test]
    fn test_startup_grace_period_pod_conditions() {
        // Pod just started (10 seconds ago) - should NOT trigger alert via conditions
        let recent_pod = Pod {
            name: "starting".into(),
            phase: "Running".into(),
            started_at: Some(Utc::now() - Duration::seconds(10)),
            conditions: vec![PodCondition {
                condition_type: "ContainersReady".into(),
                status: "False".into(),
                reason: Some("ContainersNotReady".into()),
                message: Some("containers with unready status: [main]".into()),
            }],
            container_statuses: vec![],
            ..Default::default()
        };
        assert!(detect_by_pod_conditions(&recent_pod).is_none());

        // Pod started long ago (2 minutes) - SHOULD trigger alert via conditions
        let old_pod = Pod {
            name: "stale".into(),
            phase: "Running".into(),
            started_at: Some(Utc::now() - Duration::minutes(2)),
            conditions: vec![PodCondition {
                condition_type: "ContainersReady".into(),
                status: "False".into(),
                reason: Some("ContainersNotReady".into()),
                message: Some("containers with unready status: [main]".into()),
            }],
            container_statuses: vec![],
            ..Default::default()
        };
        assert!(detect_by_pod_conditions(&old_pod).is_some());
    }
}
