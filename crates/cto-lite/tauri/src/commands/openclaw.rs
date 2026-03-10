//! `OpenClaw` gateway commands
//!
//! These commands provide the IPC bridge between the Tauri frontend and the
//! `OpenClaw` agent gateway. The gateway handles agent orchestration, `Lobster`
//! workflow execution, and CLI proxying.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::State;

/// URL for the `OpenClaw` gateway. Matches OpenClaw's default port (18789).
/// Override with `OPENCLAW_GATEWAY_URL` (e.g. `http://localhost:3100`) if you run the gateway on another port.
const DEFAULT_GATEWAY_URL: &str = "http://localhost:18789";
const DEFAULT_LOCAL_BRIDGE_URL: &str = "http://127.0.0.1:18789";
const DEFAULT_GATEWAY_PORT: u16 = 18789;

/// Whether the gateway has been connected at least once.
static GATEWAY_CONNECTED: AtomicBool = AtomicBool::new(false);

/// Local bridge state for `kubectl port-forward`.
pub struct LocalBridgeState {
    process: Mutex<Option<Child>>,
    startup: tokio::sync::Mutex<()>,
}

impl LocalBridgeState {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
            startup: tokio::sync::Mutex::new(()),
        }
    }
}

impl Default for LocalBridgeState {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Local bridge status information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawBridgeStatus {
    pub running: bool,
    pub connected: bool,
    pub pid: Option<u32>,
    pub namespace: Option<String>,
    pub service: Option<String>,
    pub local_url: String,
}

#[derive(Debug, Clone)]
struct MorganServiceTarget {
    namespace: String,
    service: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct ServiceList {
    items: Vec<ServiceItem>,
}

#[derive(Debug, Deserialize)]
struct ServiceItem {
    metadata: ServiceMetadata,
    spec: ServiceSpec,
}

#[derive(Debug, Deserialize)]
struct ServiceMetadata {
    name: String,
    namespace: String,
}

#[derive(Debug, Deserialize)]
struct ServiceSpec {
    ports: Vec<ServicePort>,
}

#[derive(Debug, Deserialize)]
struct ServicePort {
    port: u16,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenClawHealthPayload {
    ok: Option<bool>,
    status: Option<String>,
    version: Option<String>,
    agents: Option<Vec<String>>,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn gateway_url() -> String {
    std::env::var("OPENCLAW_GATEWAY_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_URL.to_string())
}

fn local_bridge_url() -> String {
    DEFAULT_LOCAL_BRIDGE_URL.to_string()
}

fn is_local_gateway_url() -> bool {
    matches!(
        gateway_url().as_str(),
        DEFAULT_GATEWAY_URL | DEFAULT_LOCAL_BRIDGE_URL
    )
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap_or_default()
}

fn fast_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap_or_default()
}

async fn fetch_gateway_status_payload() -> Result<OpenClawStatus, AppError> {
    let api_status_url = format!("{}/api/status", gateway_url());
    if let Ok(response) = fast_http_client().get(&api_status_url).send().await {
        if response.status().is_success() {
            let status: OpenClawStatus = response
                .json()
                .await
                .map_err(|e| AppError::CommandFailed(format!("Invalid status response: {e}")))?;
            return Ok(status);
        }
    }

    let health_url = format!("{}/health", gateway_url());
    let response = fast_http_client()
        .get(&health_url)
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to reach OpenClaw gateway: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::CommandFailed(format!(
            "OpenClaw gateway returned {}",
            response.status()
        )));
    }

    let payload: OpenClawHealthPayload = response
        .json()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Invalid health response: {e}")))?;

    Ok(OpenClawStatus {
        connected: payload.ok.unwrap_or(true),
        version: payload.version.or(payload.status),
        agents: payload.agents.unwrap_or_default(),
    })
}

async fn gateway_is_reachable() -> bool {
    fetch_gateway_status_payload()
        .await
        .map(|status| status.connected)
        .unwrap_or(false)
}

fn configured_bridge_target() -> Option<MorganServiceTarget> {
    let service = std::env::var("OPENCLAW_BRIDGE_SERVICE").ok()?;
    let namespace =
        std::env::var("OPENCLAW_BRIDGE_NAMESPACE").unwrap_or_else(|_| "openclaw".to_string());
    let port = std::env::var("OPENCLAW_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_GATEWAY_PORT);

    Some(MorganServiceTarget {
        namespace,
        service,
        port,
    })
}

fn discover_morgan_service() -> Result<MorganServiceTarget, AppError> {
    if let Some(target) = configured_bridge_target() {
        return Ok(target);
    }

    let kubectl = which::which("kubectl")
        .map_err(|_| AppError::CommandFailed("kubectl is required to resolve Morgan".to_string()))?;

    let output = Command::new(kubectl)
        .args(["get", "svc", "-A", "-l", "openclaw.io/agent=morgan", "-o", "json"])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to inspect Morgan service: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!(
            "Failed to inspect Morgan service: {}",
            stderr.trim()
        )));
    }

    let services: ServiceList = serde_json::from_slice(&output.stdout)?;

    let service = services
        .items
        .iter()
        .find(|item| {
            item.spec
                .ports
                .iter()
                .any(|port| port.port == DEFAULT_GATEWAY_PORT || port.name.as_deref() == Some("gateway"))
        })
        .or_else(|| services.items.first())
        .ok_or_else(|| {
            AppError::CommandFailed(
                "No Kubernetes service with label openclaw.io/agent=morgan was found".to_string(),
            )
        })?;

    let port = service
        .spec
        .ports
        .iter()
        .find(|port| port.port == DEFAULT_GATEWAY_PORT || port.name.as_deref() == Some("gateway"))
        .map(|port| port.port)
        .unwrap_or(DEFAULT_GATEWAY_PORT);

    Ok(MorganServiceTarget {
        namespace: service.metadata.namespace.clone(),
        service: service.metadata.name.clone(),
        port,
    })
}

fn bridge_pid(process: &mut Option<Child>) -> Result<Option<u32>, AppError> {
    if let Some(child) = process.as_mut() {
        match child.try_wait() {
            Ok(None) => Ok(Some(child.id())),
            Ok(Some(_)) => {
                *process = None;
                Ok(None)
            }
            Err(e) => Err(AppError::CommandFailed(format!(
                "Failed to inspect Morgan bridge process: {e}"
            ))),
        }
    } else {
        Ok(None)
    }
}

fn build_bridge_status(
    target: Option<&MorganServiceTarget>,
    running: bool,
    connected: bool,
    pid: Option<u32>,
) -> OpenClawBridgeStatus {
    OpenClawBridgeStatus {
        running,
        connected,
        pid,
        namespace: target.map(|value| value.namespace.clone()),
        service: target.map(|value| value.service.clone()),
        local_url: local_bridge_url(),
    }
}

async fn get_local_bridge_status_inner(
    state: &LocalBridgeState,
) -> Result<OpenClawBridgeStatus, AppError> {
    let target = discover_morgan_service().ok();
    let pid = {
        let mut process = state
            .process
            .lock()
            .map_err(|e| AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}")))?;
        bridge_pid(&mut *process)?
    };
    let connected = gateway_is_reachable().await;

    Ok(build_bridge_status(
        target.as_ref(),
        pid.is_some(),
        connected,
        pid,
    ))
}

async fn start_local_bridge_inner(
    state: &LocalBridgeState,
) -> Result<OpenClawBridgeStatus, AppError> {
    let _startup = state.startup.lock().await;

    if gateway_is_reachable().await {
        return get_local_bridge_status_inner(state).await;
    }

    let bridge_running = {
        let mut process = state
            .process
            .lock()
            .map_err(|e| AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}")))?;
        bridge_pid(&mut *process)?.is_some()
    };

    if bridge_running {
        return get_local_bridge_status_inner(state).await;
    }

    let target = discover_morgan_service()?;
    let kubectl = which::which("kubectl")
        .map_err(|_| AppError::CommandFailed("kubectl is required to start Morgan".to_string()))?;

    tracing::info!(
        "Starting Morgan bridge via kubectl port-forward for {}/{}",
        target.namespace,
        target.service
    );

    let mut child = Command::new(kubectl)
        .args([
            "port-forward",
            "-n",
            &target.namespace,
            &format!("svc/{}", target.service),
            &format!("{}:{}", DEFAULT_GATEWAY_PORT, target.port),
            "--address",
            "127.0.0.1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| AppError::CommandFailed(format!("Failed to start Morgan bridge: {e}")))?;

    let pid = child.id();
    let started_at = Instant::now();

    loop {
        if gateway_is_reachable().await {
            let mut process = state
                .process
                .lock()
                .map_err(|e| AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}")))?;
            *process = Some(child);
            return Ok(build_bridge_status(Some(&target), true, true, Some(pid)));
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(AppError::CommandFailed(format!(
                    "Morgan bridge exited before it became ready ({status})"
                )));
            }
            Ok(None) => {}
            Err(e) => {
                return Err(AppError::CommandFailed(format!(
                    "Failed to inspect Morgan bridge process: {e}"
                )));
            }
        }

        if started_at.elapsed() > Duration::from_secs(12) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AppError::CommandFailed(
                "Timed out waiting for Morgan bridge to become ready".to_string(),
            ));
        }

        tokio::time::sleep(Duration::from_millis(350)).await;
    }
}

async fn ensure_local_gateway(state: &LocalBridgeState) -> Result<(), AppError> {
    if !is_local_gateway_url() || gateway_is_reachable().await {
        return Ok(());
    }

    start_local_bridge_inner(state).await.map(|_| ())
}

// ─── Commands ───────────────────────────────────────────────────────────────

/// Start the local Morgan bridge.
#[tauri::command]
pub async fn openclaw_start_local_bridge(
    state: State<'_, LocalBridgeState>,
) -> Result<OpenClawBridgeStatus, AppError> {
    start_local_bridge_inner(state.inner()).await
}

/// Stop the local Morgan bridge.
#[tauri::command]
pub async fn openclaw_stop_local_bridge(
    state: State<'_, LocalBridgeState>,
) -> Result<OpenClawBridgeStatus, AppError> {
    let _startup = state.startup.lock().await;
    let target = discover_morgan_service().ok();

    let pid = {
        let mut process = state
            .process
            .lock()
            .map_err(|e| AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}")))?;

        if let Some(mut child) = process.take() {
            let pid = child.id();
            let _ = child.kill();
            let _ = child.wait();
            Some(pid)
        } else {
            None
        }
    };

    let connected = gateway_is_reachable().await;
    Ok(build_bridge_status(
        target.as_ref(),
        false,
        connected,
        pid,
    ))
}

/// Get local Morgan bridge status.
#[tauri::command]
pub async fn openclaw_get_local_bridge_status(
    state: State<'_, LocalBridgeState>,
) -> Result<OpenClawBridgeStatus, AppError> {
    get_local_bridge_status_inner(state.inner()).await
}

/// Send a message to the PM agent (`Morgan`) via the `OpenClaw` gateway.
#[tauri::command]
pub async fn openclaw_send_message(
    session_id: String,
    message: String,
    bridge: State<'_, LocalBridgeState>,
) -> Result<OpenClawResponse, AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
    bridge: State<'_, LocalBridgeState>,
) -> Result<WorkflowStartResult, AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
pub async fn openclaw_approve(
    workflow_id: String,
    bridge: State<'_, LocalBridgeState>,
) -> Result<(), AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
pub async fn openclaw_get_status(
    bridge: State<'_, LocalBridgeState>,
) -> Result<OpenClawStatus, AppError> {
    if is_local_gateway_url() {
        let _ = ensure_local_gateway(bridge.inner()).await;
    }

    let url = format!("{}/api/status", gateway_url());
    let _ = url;

    match fetch_gateway_status_payload().await {
        Ok(status) => {
            GATEWAY_CONNECTED.store(true, Ordering::Relaxed);
            Ok(status)
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
pub async fn openclaw_get_messages(
    session_id: String,
    bridge: State<'_, LocalBridgeState>,
) -> Result<Vec<OpenClawResponse>, AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
pub async fn openclaw_reject(
    workflow_id: String,
    reason: String,
    bridge: State<'_, LocalBridgeState>,
) -> Result<(), AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
    bridge: State<'_, LocalBridgeState>,
) -> Result<WorkflowStartResult, AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
pub async fn openclaw_exec_cli(
    cli: String,
    args: Vec<String>,
    bridge: State<'_, LocalBridgeState>,
) -> Result<String, AppError> {
    ensure_local_gateway(bridge.inner()).await?;

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
