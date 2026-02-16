//! `OpenClaw` gateway commands
//!
//! These commands provide the IPC bridge between the Tauri frontend and the
//! `OpenClaw` agent gateway. The gateway handles agent orchestration, `Lobster`
//! workflow execution, and CLI proxying.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// URL for the `OpenClaw` gateway. Matches OpenClaw's default port (18789).
/// Override with `OPENCLAW_GATEWAY_URL` (e.g. `http://localhost:3100`) if you run the gateway on another port.
const DEFAULT_GATEWAY_URL: &str = "http://localhost:18789";

/// Whether the gateway has been connected at least once.
static GATEWAY_CONNECTED: AtomicBool = AtomicBool::new(false);

// ─── Types ──────────────────────────────────────────────────────────────────

/// Response from the `OpenClaw` agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawResponse {
    pub content: String,
    pub action: Option<OpenClawAction>,
}

/// A structured action the agent requests the user to take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub label: String,
    pub description: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "workflowId")]
    pub workflow_id: Option<String>,
    pub completed: Option<bool>,
}

/// Result of starting a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStartResult {
    #[serde(rename = "workflowId")]
    pub workflow_id: String,
    pub status: String,
}

/// Gateway status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawStatus {
    pub connected: bool,
    pub version: Option<String>,
    pub agents: Vec<String>,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn gateway_url() -> String {
    std::env::var("OPENCLAW_GATEWAY_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_URL.to_string())
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_default()
}

// ─── Commands ───────────────────────────────────────────────────────────────

/// Send a message to the PM agent (`Morgan`) via the `OpenClaw` gateway.
#[tauri::command]
pub async fn openclaw_send_message(
    session_id: String,
    message: String,
) -> Result<OpenClawResponse, AppError> {
    let url = format!("{}/api/chat", gateway_url());

    let body = serde_json::json!({
        "sessionId": session_id,
        "message": message,
    });

    let resp = http_client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("OpenClaw gateway request failed: {}", e);
            AppError::CommandFailed(format!("Failed to reach OpenClaw gateway: {e}"))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "OpenClaw gateway returned {status}: {text}"
        )));
    }

    let result: OpenClawResponse = resp.json().await.map_err(|e| {
        AppError::CommandFailed(format!("Invalid response from OpenClaw gateway: {e}"))
    })?;

    GATEWAY_CONNECTED.store(true, Ordering::Relaxed);
    Ok(result)
}

/// Start a `Lobster` workflow via the `OpenClaw` gateway.
#[tauri::command]
pub async fn openclaw_start_workflow(
    workflow_type: String,
    params: HashMap<String, String>,
) -> Result<WorkflowStartResult, AppError> {
    let url = format!("{}/api/workflows", gateway_url());

    let body = serde_json::json!({
        "type": workflow_type,
        "params": params,
    });

    let resp = http_client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            AppError::CommandFailed(format!("Failed to start workflow via OpenClaw: {e}"))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "OpenClaw workflow start failed ({status}): {text}"
        )));
    }

    let result: WorkflowStartResult = resp
        .json()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Invalid workflow start response: {e}")))?;

    tracing::info!(
        "Started workflow {} (type={})",
        result.workflow_id,
        workflow_type
    );
    Ok(result)
}

/// Approve a pending workflow step.
#[tauri::command]
pub async fn openclaw_approve(workflow_id: String) -> Result<(), AppError> {
    let url = format!("{}/api/workflows/{}/approve", gateway_url(), workflow_id);

    let resp = http_client()
        .post(&url)
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to approve workflow: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "Workflow approval failed ({status}): {text}"
        )));
    }

    tracing::info!("Approved workflow {}", workflow_id);
    Ok(())
}

/// Get `OpenClaw` gateway connection status.
#[tauri::command]
pub async fn openclaw_get_status() -> Result<OpenClawStatus, AppError> {
    let url = format!("{}/api/status", gateway_url());

    match http_client().get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let status: OpenClawStatus = resp
                .json()
                .await
                .map_err(|e| AppError::CommandFailed(format!("Invalid status response: {e}")))?;
            GATEWAY_CONNECTED.store(true, Ordering::Relaxed);
            Ok(status)
        }
        Ok(resp) => {
            let code = resp.status();
            Err(AppError::CommandFailed(format!(
                "OpenClaw gateway returned {code}"
            )))
        }
        Err(_) => Ok(OpenClawStatus {
            connected: false,
            version: None,
            agents: vec![],
        }),
    }
}

/// Get message history for a session.
#[tauri::command]
pub async fn openclaw_get_messages(session_id: String) -> Result<Vec<OpenClawResponse>, AppError> {
    let url = format!("{}/api/chat/{}/messages", gateway_url(), session_id);

    let resp = http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to get messages: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "Get messages failed ({status}): {text}"
        )));
    }

    let messages: Vec<OpenClawResponse> = resp
        .json()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Invalid messages response: {e}")))?;

    Ok(messages)
}

/// Reject a pending workflow approval gate.
#[tauri::command]
pub async fn openclaw_reject(workflow_id: String, reason: String) -> Result<(), AppError> {
    let url = format!("{}/api/workflows/{}/reject", gateway_url(), workflow_id);

    let body = serde_json::json!({ "reason": reason });

    let resp = http_client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to reject workflow: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "Workflow rejection failed ({status}): {text}"
        )));
    }

    tracing::info!("Rejected workflow {} (reason: {})", workflow_id, reason);
    Ok(())
}

/// Get the status of a running workflow.
#[tauri::command]
pub async fn openclaw_get_workflow_status(
    workflow_id: String,
) -> Result<WorkflowStartResult, AppError> {
    let url = format!("{}/api/workflows/{}", gateway_url(), workflow_id);

    let resp = http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to get workflow status: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "Workflow status failed ({status}): {text}"
        )));
    }

    let result: WorkflowStartResult = resp
        .json()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Invalid workflow status response: {e}")))?;

    Ok(result)
}

/// Execute a CLI command through the `OpenClaw` agent proxy.
///
/// This allows the frontend to run any platform CLI (e.g. `intake`,
/// `agent-controller`, `pm-server`) through the `OpenClaw` agent, which
/// handles environment setup and error recovery.
#[tauri::command]
pub async fn openclaw_exec_cli(cli: String, args: Vec<String>) -> Result<String, AppError> {
    let url = format!("{}/api/cli", gateway_url());

    let body = serde_json::json!({
        "cli": cli,
        "args": args,
    });

    let resp = http_client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("CLI proxy request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::CommandFailed(format!(
            "CLI execution failed ({status}): {text}"
        )));
    }

    let output = resp
        .text()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to read CLI output: {e}")))?;

    Ok(output)
}
