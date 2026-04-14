//! MCP Tools for CTO Lite
//!
//! Available tools:
//! - `cto_trigger` - Trigger a workflow from a prompt
//! - `cto_status` - Get workflow status
//! - `cto_logs` - Get workflow logs
//! - `cto_jobs` - List recent workflows

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

use crate::k8s::K8sClient;

/// MCP Tool definition
#[derive(Debug, Serialize)]
pub struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// List all available tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "cto_trigger".to_string(),
            description: "Trigger a CTO workflow to implement a feature. \
                Provide a prompt describing what you want built. \
                Returns a workflow ID you can use to check status."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "repo": {
                        "type": "string",
                        "description": "GitHub repository (owner/repo)"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "Description of what to implement"
                    },
                    "issue_number": {
                        "type": "integer",
                        "description": "Optional GitHub issue number to reference"
                    },
                    "stack": {
                        "type": "string",
                        "enum": ["nova", "grizz"],
                        "description": "Backend stack: nova (TypeScript) or grizz (Rust/Go)",
                        "default": "nova"
                    }
                },
                "required": ["repo", "prompt"]
            }),
        },
        Tool {
            name: "cto_status".to_string(),
            description: "Get the status of a CTO workflow. \
                Returns the current phase and any error messages."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": {
                        "type": "string",
                        "description": "Workflow ID returned from cto_trigger"
                    }
                },
                "required": ["workflow_id"]
            }),
        },
        Tool {
            name: "cto_logs".to_string(),
            description: "Get logs from a CTO workflow. \
                Returns recent log output from the workflow execution."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": {
                        "type": "string",
                        "description": "Workflow ID returned from cto_trigger"
                    },
                    "tail": {
                        "type": "integer",
                        "description": "Number of log lines to return (default: 100)",
                        "default": 100
                    }
                },
                "required": ["workflow_id"]
            }),
        },
        Tool {
            name: "cto_jobs".to_string(),
            description: "List recent CTO workflows. \
                Shows workflow IDs, status, and creation time."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of workflows to return (default: 10)",
                        "default": 10
                    },
                    "repo": {
                        "type": "string",
                        "description": "Filter by repository (optional)"
                    }
                }
            }),
        },
    ]
}

/// Handle a tool call
pub async fn handle_tool_call(name: &str, arguments: Value) -> Result<Value> {
    match name {
        "cto_trigger" => handle_trigger(arguments).await,
        "cto_status" => handle_status(arguments).await,
        "cto_logs" => handle_logs(arguments).await,
        "cto_jobs" => handle_jobs(arguments).await,
        _ => Err(anyhow!("Unknown tool: {name}")),
    }
}

#[derive(Debug, Deserialize)]
struct TriggerArgs {
    repo: String,
    prompt: String,
    issue_number: Option<i64>,
    #[serde(default = "default_stack")]
    stack: String,
}

fn default_stack() -> String {
    "nova".to_string()
}

async fn handle_trigger(arguments: Value) -> Result<Value> {
    let args: TriggerArgs = serde_json::from_value(arguments)?;

    info!(
        "Triggering workflow for {} with prompt: {}",
        args.repo, args.prompt
    );

    let client = K8sClient::new().await?;
    let workflow_id = client
        .create_workflow(&args.repo, &args.prompt, args.issue_number, &args.stack)
        .await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "✅ Workflow triggered!\n\n\
                **Workflow ID:** `{}`\n\
                **Repository:** {}\n\
                **Stack:** {}\n\n\
                Use `cto_status` with this workflow ID to check progress.",
                workflow_id, args.repo, args.stack
            )
        }]
    }))
}

#[derive(Debug, Deserialize)]
struct StatusArgs {
    workflow_id: String,
}

async fn handle_status(arguments: Value) -> Result<Value> {
    let args: StatusArgs = serde_json::from_value(arguments)?;

    info!("Getting status for workflow: {}", args.workflow_id);

    let client = K8sClient::new().await?;
    let status = client.get_workflow_status(&args.workflow_id).await?;

    let status_emoji = match status.phase.as_str() {
        "Succeeded" => "✅",
        "Failed" => "❌",
        "Running" => "🔄",
        "Pending" => "⏳",
        _ => "❓",
    };

    let mut text = format!(
        "{} **{}**\n\n\
        **Workflow:** `{}`\n\
        **Started:** {}\n",
        status_emoji,
        status.phase,
        args.workflow_id,
        status
            .started_at
            .unwrap_or_else(|| "Not started".to_string()),
    );

    if let Some(finished) = status.finished_at {
        text.push_str(&format!("**Finished:** {finished}\n"));
    }

    if let Some(message) = status.message {
        text.push_str(&format!("\n**Message:** {message}\n"));
    }

    if !status.nodes.is_empty() {
        text.push_str("\n**Steps:**\n");
        for node in status.nodes {
            let node_emoji = match node.phase.as_str() {
                "Succeeded" => "✅",
                "Failed" => "❌",
                "Running" => "🔄",
                "Pending" => "⏳",
                "Skipped" => "⏭️",
                _ => "❓",
            };
            text.push_str(&format!("- {} {}\n", node_emoji, node.display_name));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

#[derive(Debug, Deserialize)]
struct LogsArgs {
    workflow_id: String,
    #[serde(default = "default_tail")]
    tail: i64,
}

fn default_tail() -> i64 {
    100
}

async fn handle_logs(arguments: Value) -> Result<Value> {
    let args: LogsArgs = serde_json::from_value(arguments)?;

    info!("Getting logs for workflow: {}", args.workflow_id);

    let client = K8sClient::new().await?;
    let logs = client
        .get_workflow_logs(&args.workflow_id, args.tail)
        .await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("**Logs for `{}`:**\n\n```\n{}\n```", args.workflow_id, logs)
        }]
    }))
}

#[derive(Debug, Deserialize)]
struct JobsArgs {
    #[serde(default = "default_limit")]
    limit: i64,
    repo: Option<String>,
}

fn default_limit() -> i64 {
    10
}

async fn handle_jobs(arguments: Value) -> Result<Value> {
    let args: JobsArgs = serde_json::from_value(arguments)?;

    info!("Listing workflows (limit: {})", args.limit);

    let client = K8sClient::new().await?;
    let jobs = client
        .list_workflows(args.limit, args.repo.as_deref())
        .await?;

    if jobs.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "No workflows found."
            }]
        }));
    }

    let mut text = "**Recent Workflows:**\n\n".to_string();
    for job in jobs {
        let status_emoji = match job.phase.as_str() {
            "Succeeded" => "✅",
            "Failed" => "❌",
            "Running" => "🔄",
            "Pending" => "⏳",
            _ => "❓",
        };
        text.push_str(&format!(
            "- {} `{}` - {} ({})\n",
            status_emoji, job.name, job.phase, job.created_at
        ));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}
