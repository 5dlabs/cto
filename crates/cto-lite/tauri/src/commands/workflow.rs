//! Workflow management commands

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::process::Command;

const CLUSTER_NAME: &str = "cto-lite";
const NAMESPACE: &str = "cto";

/// Workflow status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub message: Option<String>,
}

/// Workflow log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLogEntry {
    pub timestamp: String,
    pub pod: String,
    pub container: String,
    pub message: String,
}

/// Run argo CLI command
fn run_argo(args: &[&str]) -> Result<String, AppError> {
    let output = Command::new("argo")
        .args(args)
        .env("KUBECONFIG", get_kubeconfig_path())
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run argo: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(stderr.to_string()))
    }
}

/// Get kubeconfig path for kind cluster
fn get_kubeconfig_path() -> String {
    dirs::home_dir()
        .map(|h: std::path::PathBuf| h.join(".kube").join("config"))
        .and_then(|p: std::path::PathBuf| p.to_str().map(String::from))
        .unwrap_or_else(|| "~/.kube/config".to_string())
}

/// List all workflows
#[tauri::command]
pub async fn list_workflows() -> Result<Vec<WorkflowInfo>, AppError> {
    // Check if argo is installed
    if which::which("argo").is_err() {
        return Err(AppError::CommandFailed("argo CLI not found".to_string()));
    }

    let output = run_argo(&[
        "list",
        "-n",
        NAMESPACE,
        "--context",
        &format!("kind-{}", CLUSTER_NAME),
        "-o",
        "json",
    ])?;

    if output.is_empty() || output == "null" {
        return Ok(vec![]);
    }

    // Parse JSON output
    let workflows: Vec<ArgoWorkflow> =
        serde_json::from_str(&output).map_err(AppError::JsonError)?;

    Ok(workflows
        .into_iter()
        .map(|w| WorkflowInfo {
            name: w.metadata.name,
            namespace: w.metadata.namespace,
            phase: w.status.phase.unwrap_or_else(|| "Unknown".to_string()),
            started_at: w.status.started_at,
            finished_at: w.status.finished_at,
            message: w.status.message,
        })
        .collect())
}

/// Get workflow status
#[tauri::command]
pub async fn get_workflow_status(name: String) -> Result<WorkflowInfo, AppError> {
    let output = run_argo(&[
        "get",
        &name,
        "-n",
        NAMESPACE,
        "--context",
        &format!("kind-{}", CLUSTER_NAME),
        "-o",
        "json",
    ])?;

    let workflow: ArgoWorkflow = serde_json::from_str(&output).map_err(AppError::JsonError)?;

    Ok(WorkflowInfo {
        name: workflow.metadata.name,
        namespace: workflow.metadata.namespace,
        phase: workflow
            .status
            .phase
            .unwrap_or_else(|| "Unknown".to_string()),
        started_at: workflow.status.started_at,
        finished_at: workflow.status.finished_at,
        message: workflow.status.message,
    })
}

/// Get workflow logs
#[tauri::command]
pub async fn get_workflow_logs(name: String) -> Result<Vec<WorkflowLogEntry>, AppError> {
    let output = run_argo(&[
        "logs",
        &name,
        "-n",
        NAMESPACE,
        "--context",
        &format!("kind-{}", CLUSTER_NAME),
        "--no-color",
    ])?;

    // Parse log lines
    // Format: pod-name/container-name: timestamp message
    let entries: Vec<WorkflowLogEntry> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() != 2 {
                return None;
            }

            let prefix = parts[0];
            let message = parts[1];

            // Parse prefix: pod-name/container-name timestamp
            let prefix_parts: Vec<&str> = prefix.splitn(2, ' ').collect();
            if prefix_parts.len() != 2 {
                return Some(WorkflowLogEntry {
                    timestamp: String::new(),
                    pod: prefix.to_string(),
                    container: String::new(),
                    message: message.to_string(),
                });
            }

            let pod_container: Vec<&str> = prefix_parts[0].splitn(2, '/').collect();
            let (pod, container) = if pod_container.len() == 2 {
                (pod_container[0].to_string(), pod_container[1].to_string())
            } else {
                (prefix_parts[0].to_string(), String::new())
            };

            Some(WorkflowLogEntry {
                timestamp: prefix_parts[1].to_string(),
                pod,
                container,
                message: message.to_string(),
            })
        })
        .collect();

    Ok(entries)
}

/// Cancel a workflow
#[tauri::command]
pub async fn cancel_workflow(name: String) -> Result<(), AppError> {
    run_argo(&[
        "stop",
        &name,
        "-n",
        NAMESPACE,
        "--context",
        &format!("kind-{}", CLUSTER_NAME),
    ])?;

    tracing::info!("Cancelled workflow: {}", name);
    Ok(())
}

// Argo workflow JSON structure (minimal)
#[derive(Debug, Deserialize)]
struct ArgoWorkflow {
    metadata: ArgoMetadata,
    status: ArgoStatus,
}

#[derive(Debug, Deserialize)]
struct ArgoMetadata {
    name: String,
    namespace: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArgoStatus {
    phase: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    message: Option<String>,
}
