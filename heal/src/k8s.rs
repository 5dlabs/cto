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
    /// CodeRun status changed
    CodeRunChanged(CodeRun),
    /// GitHub state was updated (from poller)
    GitHubUpdate,
}

/// Simplified Pod representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub labels: HashMap<String, String>,
    pub container_statuses: Vec<ContainerStatus>,
    pub started_at: Option<DateTime<Utc>>,
}

/// Container status within a pod
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub name: String,
    pub state: ContainerState,
    pub restart_count: i32,
}

/// Container state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ContainerState {
    #[default]
    Waiting,
    Running,
    Terminated {
        exit_code: i32,
        reason: Option<String>,
        finished_at: Option<DateTime<Utc>>,
    },
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

/// Simplified CodeRun representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeRun {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub agent: String,
    pub task_id: String,
    pub labels: HashMap<String, String>,
}
