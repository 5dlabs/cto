//! Workflow management for CTO Lite
//!
//! Manages Argo Workflows running in the local Kind cluster.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Workflow status from Argo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStatus {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub progress: Option<String>,
    pub message: Option<String>,
}

/// Workflow node (step) status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub node_type: String,
    pub phase: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub message: Option<String>,
}

/// Workflow detail with nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDetail {
    pub status: WorkflowStatus,
    pub nodes: Vec<WorkflowNode>,
}

/// Parameters for triggering a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowParams {
    pub repo_url: String,
    pub branch: Option<String>,
    pub prompt: String,
    pub stack: Option<String>,
}

/// Check if Argo Workflows is available
pub async fn check_argo() -> Result<bool> {
    let output = Command::new("kubectl")
        .args([
            "get",
            "deployment",
            "-n",
            "cto-lite",
            "-l",
            "app.kubernetes.io/name=argo-workflows-server",
            "-o",
            "name",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to check Argo Workflows")?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// List all workflows in the cto-lite namespace
pub async fn list_workflows() -> Result<Vec<WorkflowStatus>> {
    let output = Command::new("kubectl")
        .args([
            "get",
            "workflows",
            "-n",
            "cto-lite",
            "-o",
            "json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to list workflows")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No resources found") {
            return Ok(vec![]);
        }
        anyhow::bail!("kubectl failed: {}", stderr);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse workflow list")?;

    let empty_items = vec![];
    let items = json["items"].as_array().unwrap_or(&empty_items);
    let mut workflows = Vec::new();

    for item in items {
        let status = parse_workflow_status(item)?;
        workflows.push(status);
    }

    // Sort by start time, newest first
    workflows.sort_by(|a, b| {
        b.started_at
            .as_ref()
            .unwrap_or(&String::new())
            .cmp(a.started_at.as_ref().unwrap_or(&String::new()))
    });

    Ok(workflows)
}

/// Get detailed status of a specific workflow
pub async fn get_workflow(name: &str) -> Result<WorkflowDetail> {
    let output = Command::new("kubectl")
        .args([
            "get",
            "workflow",
            name,
            "-n",
            "cto-lite",
            "-o",
            "json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to get workflow")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to get workflow: {}", stderr);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse workflow")?;

    let status = parse_workflow_status(&json)?;
    let nodes = parse_workflow_nodes(&json)?;

    Ok(WorkflowDetail { status, nodes })
}

/// Trigger a new workflow using the play-workflow-lite template
pub async fn trigger_workflow(params: &WorkflowParams) -> Result<String> {
    info!("Triggering workflow for repo: {}", params.repo_url);

    // Generate a unique workflow name
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let workflow_name = format!("play-{}", timestamp);

    let stack = params.stack.as_deref().unwrap_or("grizz");
    let branch = params.branch.as_deref().unwrap_or("main");

    let output = Command::new("kubectl")
        .args([
            "create",
            "-n",
            "cto-lite",
            "-f",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to create workflow")?;

    // Create workflow from template
    let workflow_yaml = format!(
        r#"apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: {}
  namespace: cto-lite
spec:
  workflowTemplateRef:
    name: play-workflow-lite
  arguments:
    parameters:
      - name: repo-url
        value: "{}"
      - name: branch
        value: "{}"
      - name: prompt
        value: "{}"
      - name: stack
        value: "{}"
"#,
        workflow_name,
        params.repo_url,
        branch,
        params.prompt.replace('"', r#"\""#),
        stack
    );

    let output = Command::new("kubectl")
        .args(["apply", "-n", "cto-lite", "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    // Use argo submit instead if kubectl fails
    match output {
        Ok(out) if out.status.success() => {
            info!("Workflow {} created", workflow_name);
            Ok(workflow_name)
        }
        _ => {
            // Try using argo CLI as fallback
            let output = Command::new("argo")
                .args([
                    "submit",
                    "--from",
                    "workflowtemplate/play-workflow-lite",
                    "-n",
                    "cto-lite",
                    "--name",
                    &workflow_name,
                    "-p",
                    &format!("repo-url={}", params.repo_url),
                    "-p",
                    &format!("branch={}", branch),
                    "-p",
                    &format!("prompt={}", params.prompt),
                    "-p",
                    &format!("stack={}", stack),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .context("Failed to submit workflow via argo CLI")?;

            if output.status.success() {
                info!("Workflow {} submitted via argo CLI", workflow_name);
                Ok(workflow_name)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to submit workflow: {}", stderr)
            }
        }
    }
}

/// Get logs for a workflow node
pub async fn get_workflow_logs(workflow_name: &str, node_name: Option<&str>) -> Result<String> {
    let mut args = vec![
        "logs".to_string(),
        workflow_name.to_string(),
        "-n".to_string(),
        "cto-lite".to_string(),
    ];

    if let Some(node) = node_name {
        args.push(node.to_string());
    }

    let output = Command::new("argo")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        }
        Ok(out) => {
            // Try kubectl logs as fallback
            let stderr = String::from_utf8_lossy(&out.stderr);
            warn!("argo logs failed: {}, trying kubectl", stderr);
            get_workflow_logs_kubectl(workflow_name, node_name).await
        }
        Err(_) => get_workflow_logs_kubectl(workflow_name, node_name).await,
    }
}

/// Get logs using kubectl (fallback)
async fn get_workflow_logs_kubectl(workflow_name: &str, node_name: Option<&str>) -> Result<String> {
    let selector = if let Some(node) = node_name {
        format!("workflows.argoproj.io/workflow={},workflows.argoproj.io/node-name={}", workflow_name, node)
    } else {
        format!("workflows.argoproj.io/workflow={}", workflow_name)
    };

    let output = Command::new("kubectl")
        .args([
            "logs",
            "-n",
            "cto-lite",
            "-l",
            &selector,
            "--all-containers",
            "--tail",
            "1000",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to get logs")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to get logs: {}", stderr)
    }
}

/// Delete a workflow
pub async fn delete_workflow(name: &str) -> Result<()> {
    let output = Command::new("kubectl")
        .args([
            "delete",
            "workflow",
            name,
            "-n",
            "cto-lite",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to delete workflow")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to delete workflow: {}", stderr);
    }

    info!("Deleted workflow {}", name);
    Ok(())
}

/// Stop a running workflow
pub async fn stop_workflow(name: &str) -> Result<()> {
    let output = Command::new("argo")
        .args(["stop", name, "-n", "cto-lite"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            info!("Stopped workflow {}", name);
            Ok(())
        }
        _ => {
            // Fallback: patch the workflow to terminate
            let output = Command::new("kubectl")
                .args([
                    "patch",
                    "workflow",
                    name,
                    "-n",
                    "cto-lite",
                    "--type",
                    "merge",
                    "-p",
                    r#"{"spec":{"shutdown":"Stop"}}"#,
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .context("Failed to stop workflow")?;

            if output.status.success() {
                info!("Stopped workflow {} via kubectl patch", name);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to stop workflow: {}", stderr)
            }
        }
    }
}

// Helper functions

fn parse_workflow_status(json: &serde_json::Value) -> Result<WorkflowStatus> {
    Ok(WorkflowStatus {
        name: json["metadata"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        namespace: json["metadata"]["namespace"]
            .as_str()
            .unwrap_or("cto-lite")
            .to_string(),
        phase: json["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
        started_at: json["status"]["startedAt"]
            .as_str()
            .map(|s| s.to_string()),
        finished_at: json["status"]["finishedAt"]
            .as_str()
            .map(|s| s.to_string()),
        progress: json["status"]["progress"]
            .as_str()
            .map(|s| s.to_string()),
        message: json["status"]["message"]
            .as_str()
            .map(|s| s.to_string()),
    })
}

fn parse_workflow_nodes(json: &serde_json::Value) -> Result<Vec<WorkflowNode>> {
    let nodes_obj = match json["status"]["nodes"].as_object() {
        Some(obj) => obj,
        None => return Ok(vec![]),
    };

    let mut nodes = Vec::new();
    for (_id, node) in nodes_obj {
        nodes.push(WorkflowNode {
            id: node["id"].as_str().unwrap_or("").to_string(),
            name: node["name"].as_str().unwrap_or("").to_string(),
            display_name: node["displayName"].as_str().unwrap_or("").to_string(),
            node_type: node["type"].as_str().unwrap_or("").to_string(),
            phase: node["phase"].as_str().unwrap_or("").to_string(),
            started_at: node["startedAt"].as_str().map(|s| s.to_string()),
            finished_at: node["finishedAt"].as_str().map(|s| s.to_string()),
            message: node["message"].as_str().map(|s| s.to_string()),
        });
    }

    // Sort by start time
    nodes.sort_by(|a, b| {
        a.started_at
            .as_ref()
            .unwrap_or(&String::new())
            .cmp(b.started_at.as_ref().unwrap_or(&String::new()))
    });

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workflow_status() {
        let json = serde_json::json!({
            "metadata": {
                "name": "test-workflow",
                "namespace": "cto-lite"
            },
            "status": {
                "phase": "Running",
                "startedAt": "2024-01-30T10:00:00Z",
                "progress": "2/5"
            }
        });

        let status = parse_workflow_status(&json).unwrap();
        assert_eq!(status.name, "test-workflow");
        assert_eq!(status.phase, "Running");
        assert_eq!(status.progress, Some("2/5".to_string()));
    }
}
