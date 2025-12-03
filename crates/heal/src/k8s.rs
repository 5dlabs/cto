//! Kubernetes types and event handling for the monitor.

#![allow(dead_code)] // Public API - variants used for workflow/coderun monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Events from Kubernetes watches
#[derive(Debug, Clone)]
pub enum K8sEvent {
    /// Pod started running
    PodRunning(Pod),
    /// Pod was modified
    PodModified(Pod),
    /// Pod succeeded
    PodSucceeded(Pod),
    /// Pod failed
    PodFailed(Pod),
    /// Workflow phase changed
    WorkflowPhaseChanged(Workflow),
    /// `CodeRun` status changed
    CodeRunChanged(CodeRun),
    /// GitHub state was updated (from poller)
    GitHubUpdate,
}

/// Pod name prefixes to exclude from alerts and completion checks.
/// These are infrastructure pods that restart during deployments or `CronJobs`.
const EXCLUDED_POD_PREFIXES: &[&str] = &[
    "heal",
    "cto-tools",
    "cto-controller",
    "vault-mcp-server",
    "openmemory",
    "event-cleaner",
    "workspace-pvc-cleaner",
];

/// Check if a pod name should be excluded from alerts/completion checks.
/// Returns true for infrastructure pods that restart during deployments or `CronJobs`.
pub fn is_excluded_pod(pod_name: &str) -> bool {
    EXCLUDED_POD_PREFIXES
        .iter()
        .any(|prefix| pod_name.starts_with(prefix))
}

/// Simplified Pod representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub labels: HashMap<String, String>,
    pub conditions: Vec<PodCondition>,
    pub container_statuses: Vec<ContainerStatus>,
    pub started_at: Option<DateTime<Utc>>,
}

/// Pod condition from Kubernetes status.conditions[]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PodCondition {
    /// Condition type: "Ready", "ContainersReady", "Initialized", etc.
    pub condition_type: String,
    /// Status: "True", "False", "Unknown"
    pub status: String,
    /// Machine-readable reason for the condition
    pub reason: Option<String>,
    /// Human-readable message with details
    pub message: Option<String>,
}

/// Container status within a pod
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub name: String,
    pub ready: bool,
    pub state: ContainerState,
    pub restart_count: i32,
}

/// Container state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerState {
    Waiting { reason: Option<String> },
    Running,
    Terminated {
        exit_code: i32,
        reason: Option<String>,
        finished_at: Option<DateTime<Utc>>,
    },
}

impl Default for ContainerState {
    fn default() -> Self {
        Self::Waiting { reason: None }
    }
}

/// Simplified Workflow representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub nodes: Vec<WorkflowNode>,
}

/// A node/step within a workflow
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub name: String,
    pub phase: String,
    pub template_name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
}

/// Simplified `CodeRun` representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeRun {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub agent: String,
    pub task_id: String,
    pub labels: HashMap<String, String>,
}
