//! Proof-of-Concept: Silent Failure Detection Methods
//!
//! This module tests multiple approaches for detecting when the primary agent
//! container has failed but the pod is still "Running" (due to sidecars).
//!
//! Real cluster data observed:
//! - Pod phase: "Running"
//! - Container `factory-claude-opus-4-5-20251101`: terminated, exitCode=1, ready=false
//! - Container `docker-daemon`: running, ready=true
//! - Pod condition `Ready`: False, reason=ContainersNotReady
//! - Pod condition `ContainersReady`: False, reason=ContainersNotReady

#![allow(dead_code)] // POC module - types only used in tests

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

// ============================================================================
// Extended types for POC (will be merged into k8s.rs if validated)
// ============================================================================

/// Pod condition from Kubernetes status.conditions[]
#[derive(Debug, Clone, Default)]
pub struct PodCondition {
    pub condition_type: String, // "Ready", "ContainersReady", "Initialized", etc.
    pub status: String,         // "True", "False", "Unknown"
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition: Option<DateTime<Utc>>,
}

/// Extended container status with ready field
#[derive(Debug, Clone, Default)]
pub struct ContainerStatusExt {
    pub name: String,
    pub ready: bool,
    pub state: ContainerStateExt,
    pub last_state: Option<ContainerStateExt>,
    pub restart_count: i32,
}

#[derive(Debug, Clone)]
pub enum ContainerStateExt {
    Waiting { reason: Option<String> },
    Running { started_at: Option<DateTime<Utc>> },
    Terminated {
        exit_code: i32,
        reason: Option<String>,
        finished_at: Option<DateTime<Utc>>,
    },
}

impl Default for ContainerStateExt {
    fn default() -> Self {
        Self::Waiting { reason: None }
    }
}

/// Extended pod with conditions
#[derive(Debug, Clone, Default)]
pub struct PodExt {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub labels: HashMap<String, String>,
    pub conditions: Vec<PodCondition>,
    pub container_statuses: Vec<ContainerStatusExt>,
    pub started_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Detection Methods
// ============================================================================

/// Detection result with method attribution (for deduplication)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionMethod {
    /// Current method: container terminated with non-zero exit code
    ExitCode,
    /// Container ready=false while pod Running
    ReadyStatus,
    /// Pod condition ContainersReady=False
    PodCondition,
    /// High restart count indicates instability
    RestartCount,
    /// Container terminated for extended duration
    TerminatedDuration,
}

#[derive(Debug, Clone)]
pub struct Detection {
    pub method: DetectionMethod,
    pub container: String,
    pub message: String,
    pub priority: u8, // Lower = higher priority (for dedup)
}

// ============================================================================
// Method 1: Exit Code (current implementation)
// ============================================================================

pub fn detect_by_exit_code(pod: &PodExt) -> Option<Detection> {
    if pod.phase != "Running" {
        return None;
    }

    for container in &pod.container_statuses {
        if let ContainerStateExt::Terminated { exit_code, .. } = &container.state {
            if *exit_code != 0 {
                return Some(Detection {
                    method: DetectionMethod::ExitCode,
                    container: container.name.clone(),
                    message: format!(
                        "Container '{}' terminated with exit code {}",
                        container.name, exit_code
                    ),
                    priority: 1, // Highest priority - most definitive signal
                });
            }
        }
    }
    None
}

// ============================================================================
// Method 2: Ready Status
// ============================================================================

pub fn detect_by_ready_status(pod: &PodExt) -> Option<Detection> {
    if pod.phase != "Running" {
        return None;
    }

    for container in &pod.container_statuses {
        // Container not ready AND not in a normal waiting state (like pulling image)
        if !container.ready {
            // Skip if container is in waiting state (startup phase)
            if matches!(container.state, ContainerStateExt::Waiting { .. }) {
                continue;
            }

            return Some(Detection {
                method: DetectionMethod::ReadyStatus,
                container: container.name.clone(),
                message: format!(
                    "Container '{}' not ready while pod Running",
                    container.name
                ),
                priority: 2,
            });
        }
    }
    None
}

// ============================================================================
// Method 3: Pod Conditions
// ============================================================================

pub fn detect_by_pod_conditions(pod: &PodExt) -> Option<Detection> {
    if pod.phase != "Running" {
        return None;
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
                method: DetectionMethod::PodCondition,
                container: container_name,
                message: format!(
                    "Pod condition ContainersReady=False: {}",
                    condition.reason.as_deref().unwrap_or("unknown")
                ),
                priority: 3,
            });
        }
    }

    // Fall back to Ready condition
    for condition in &pod.conditions {
        if condition.condition_type == "Ready" && condition.status == "False" {
            return Some(Detection {
                method: DetectionMethod::PodCondition,
                container: "unknown".into(),
                message: format!(
                    "Pod condition Ready=False: {}",
                    condition.reason.as_deref().unwrap_or("unknown")
                ),
                priority: 4,
            });
        }
    }

    None
}

// ============================================================================
// Method 4: Restart Count
// ============================================================================

const RESTART_THRESHOLD: i32 = 3;

pub fn detect_by_restart_count(pod: &PodExt) -> Option<Detection> {
    if pod.phase != "Running" {
        return None;
    }

    for container in &pod.container_statuses {
        if container.restart_count >= RESTART_THRESHOLD {
            return Some(Detection {
                method: DetectionMethod::RestartCount,
                container: container.name.clone(),
                message: format!(
                    "Container '{}' has restarted {} times (threshold: {})",
                    container.name, container.restart_count, RESTART_THRESHOLD
                ),
                priority: 5, // Lower priority - might be transient
            });
        }
    }
    None
}

// ============================================================================
// Method 5: Terminated Duration
// ============================================================================

const TERMINATED_DURATION_THRESHOLD_SECS: i64 = 60;

pub fn detect_by_terminated_duration(pod: &PodExt) -> Option<Detection> {
    if pod.phase != "Running" {
        return None;
    }

    let now = Utc::now();

    for container in &pod.container_statuses {
        if let ContainerStateExt::Terminated {
            finished_at: Some(finished),
            ..
        } = &container.state
        {
            let duration = now.signed_duration_since(*finished);
            if duration > Duration::seconds(TERMINATED_DURATION_THRESHOLD_SECS) {
                return Some(Detection {
                    method: DetectionMethod::TerminatedDuration,
                    container: container.name.clone(),
                    message: format!(
                        "Container '{}' has been terminated for {}s (threshold: {}s)",
                        container.name,
                        duration.num_seconds(),
                        TERMINATED_DURATION_THRESHOLD_SECS
                    ),
                    priority: 6,
                });
            }
        }
    }
    None
}

// ============================================================================
// Combined Detection with Deduplication
// ============================================================================

/// Run all detection methods and return the highest-priority detection.
/// This ensures we don't fire multiple alerts for the same failure.
pub fn detect_silent_failure(pod: &PodExt) -> Option<Detection> {
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

    // Return highest priority (lowest number) detection
    detections.into_iter().min_by_key(|d| d.priority)
}

/// Run all detection methods and return all detections (for debugging/analysis)
pub fn detect_all_signals(pod: &PodExt) -> Vec<Detection> {
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

    detections
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a pod matching the real cluster state we observed
    fn create_real_world_failed_pod() -> PodExt {
        PodExt {
            name: "play-coderun-t3-rex-factory-c086f62a-v1-4l85r".into(),
            namespace: "cto".into(),
            phase: "Running".into(),
            labels: HashMap::new(),
            conditions: vec![
                PodCondition {
                    condition_type: "Ready".into(),
                    status: "False".into(),
                    reason: Some("ContainersNotReady".into()),
                    message: Some(
                        "containers with unready status: [factory-claude-opus-4-5-20251101]".into(),
                    ),
                    last_transition: None,
                },
                PodCondition {
                    condition_type: "ContainersReady".into(),
                    status: "False".into(),
                    reason: Some("ContainersNotReady".into()),
                    message: Some(
                        "containers with unready status: [factory-claude-opus-4-5-20251101]".into(),
                    ),
                    last_transition: None,
                },
                PodCondition {
                    condition_type: "Initialized".into(),
                    status: "True".into(),
                    reason: None,
                    message: None,
                    last_transition: None,
                },
            ],
            container_statuses: vec![
                ContainerStatusExt {
                    name: "factory-claude-opus-4-5-20251101".into(),
                    ready: false,
                    state: ContainerStateExt::Terminated {
                        exit_code: 1,
                        reason: Some("Error".into()),
                        finished_at: Some(Utc::now() - Duration::minutes(10)),
                    },
                    last_state: None,
                    restart_count: 0,
                },
                ContainerStatusExt {
                    name: "docker-daemon".into(),
                    ready: true,
                    state: ContainerStateExt::Running {
                        started_at: Some(Utc::now() - Duration::minutes(12)),
                    },
                    last_state: None,
                    restart_count: 0,
                },
            ],
            started_at: Some(Utc::now() - Duration::minutes(12)),
        }
    }

    #[test]
    fn test_real_world_pod_detected_by_all_methods() {
        let pod = create_real_world_failed_pod();

        // All relevant methods should detect this failure
        let all = detect_all_signals(&pod);
        println!("All detections: {all:#?}");

        assert!(!all.is_empty(), "Should detect failure");

        // Exit code should be detected
        assert!(
            all.iter()
                .any(|d| d.method == DetectionMethod::ExitCode),
            "Should detect by exit code"
        );

        // Ready status should be detected
        assert!(
            all.iter()
                .any(|d| d.method == DetectionMethod::ReadyStatus),
            "Should detect by ready status"
        );

        // Pod condition should be detected
        assert!(
            all.iter()
                .any(|d| d.method == DetectionMethod::PodCondition),
            "Should detect by pod condition"
        );

        // Terminated duration should be detected (>60s)
        assert!(
            all.iter()
                .any(|d| d.method == DetectionMethod::TerminatedDuration),
            "Should detect by terminated duration"
        );
    }

    #[test]
    fn test_combined_returns_highest_priority() {
        let pod = create_real_world_failed_pod();

        let detection = detect_silent_failure(&pod);
        assert!(detection.is_some());

        let d = detection.unwrap();
        // Exit code has priority 1 (highest)
        assert_eq!(d.method, DetectionMethod::ExitCode);
        assert!(d.container.contains("factory"));
    }

    #[test]
    fn test_healthy_pod_no_detection() {
        let pod = PodExt {
            name: "healthy-pod".into(),
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
                ContainerStatusExt {
                    name: "main".into(),
                    ready: true,
                    state: ContainerStateExt::Running {
                        started_at: Some(Utc::now()),
                    },
                    ..Default::default()
                },
                ContainerStatusExt {
                    name: "sidecar".into(),
                    ready: true,
                    state: ContainerStateExt::Running {
                        started_at: Some(Utc::now()),
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert!(detect_silent_failure(&pod).is_none());
        assert!(detect_all_signals(&pod).is_empty());
    }

    #[test]
    fn test_exit_code_zero_no_detection() {
        let pod = PodExt {
            name: "completed-init".into(),
            phase: "Running".into(),
            conditions: vec![PodCondition {
                condition_type: "Ready".into(),
                status: "True".into(),
                ..Default::default()
            }],
            container_statuses: vec![ContainerStatusExt {
                name: "init-container".into(),
                ready: true, // Init containers show ready after completion
                state: ContainerStateExt::Terminated {
                    exit_code: 0, // Success
                    reason: Some("Completed".into()),
                    finished_at: Some(Utc::now() - Duration::minutes(5)),
                },
                ..Default::default()
            }],
            ..Default::default()
        };

        // Exit code 0 should NOT trigger exit code detection
        assert!(detect_by_exit_code(&pod).is_none());
    }

    #[test]
    fn test_restart_count_threshold() {
        let pod = PodExt {
            name: "crashloop-pod".into(),
            phase: "Running".into(),
            conditions: vec![PodCondition {
                condition_type: "Ready".into(),
                status: "False".into(),
                reason: Some("ContainersNotReady".into()),
                ..Default::default()
            }],
            container_statuses: vec![ContainerStatusExt {
                name: "unstable-container".into(),
                ready: false,
                state: ContainerStateExt::Waiting {
                    reason: Some("CrashLoopBackOff".into()),
                },
                restart_count: 5,
                ..Default::default()
            }],
            ..Default::default()
        };

        let detection = detect_by_restart_count(&pod);
        assert!(detection.is_some());
        assert_eq!(detection.unwrap().method, DetectionMethod::RestartCount);
    }

    #[test]
    fn test_only_pod_condition_available() {
        // Scenario: We only have pod conditions, no detailed container state
        let pod = PodExt {
            name: "minimal-info-pod".into(),
            phase: "Running".into(),
            conditions: vec![PodCondition {
                condition_type: "ContainersReady".into(),
                status: "False".into(),
                reason: Some("ContainersNotReady".into()),
                message: Some("containers with unready status: [main]".into()),
                ..Default::default()
            }],
            container_statuses: vec![], // Empty - no container status available yet
            ..Default::default()
        };

        // Pod condition should still detect
        let detection = detect_by_pod_conditions(&pod);
        assert!(detection.is_some());
        assert_eq!(detection.unwrap().method, DetectionMethod::PodCondition);
    }

    #[test]
    fn test_recently_terminated_not_flagged_by_duration() {
        let pod = PodExt {
            name: "just-died".into(),
            phase: "Running".into(),
            container_statuses: vec![ContainerStatusExt {
                name: "main".into(),
                ready: false,
                state: ContainerStateExt::Terminated {
                    exit_code: 1,
                    reason: Some("Error".into()),
                    // Just terminated 10 seconds ago
                    finished_at: Some(Utc::now() - Duration::seconds(10)),
                },
                ..Default::default()
            }],
            ..Default::default()
        };

        // Duration check should NOT fire (under threshold)
        assert!(detect_by_terminated_duration(&pod).is_none());

        // But exit code should still catch it
        assert!(detect_by_exit_code(&pod).is_some());
    }

    #[test]
    fn test_waiting_container_not_flagged_by_ready() {
        let pod = PodExt {
            name: "starting-pod".into(),
            phase: "Running".into(),
            container_statuses: vec![ContainerStatusExt {
                name: "main".into(),
                ready: false, // Not ready yet
                state: ContainerStateExt::Waiting {
                    reason: Some("ContainerCreating".into()),
                },
                ..Default::default()
            }],
            ..Default::default()
        };

        // Ready status check should NOT fire during startup
        assert!(detect_by_ready_status(&pod).is_none());
    }
}

