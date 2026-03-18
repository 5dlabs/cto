//! `OpenClaw` gateway commands
//!
//! These commands provide the IPC bridge between the Tauri frontend and the
//! `OpenClaw` agent gateway. The gateway handles agent orchestration, `Lobster`
//! workflow execution, and CLI proxying.

use crate::error::AppError;
use crate::runtime;
use acp_runtime::{run_oneshot_prompt, AcpClientProfile, AcpPermissionPolicy, AcpPromptRequest};
use agent_client_protocol::{ContentBlock, SessionNotification, SessionUpdate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::State;

/// Default ingress endpoint for a local Morgan gateway in kind.
const DEFAULT_LOCAL_INGRESS_URL: &str = "http://morgan.localhost";
/// URL for the `OpenClaw` gateway when a local bridge is used.
const DEFAULT_LOCAL_BRIDGE_URL: &str = "http://127.0.0.1:18789";
/// Static auth token used by the local Morgan gateway.
const DEFAULT_GATEWAY_TOKEN: &str = "openclaw-internal";
const DEFAULT_AGENT_ID: &str = "morgan";
const DEFAULT_GATEWAY_PORT: u16 = 18789;
/// Dedicated local kind context that CTO manages for Morgan.
const LOCAL_KIND_CONTEXT: &str = "kind-cto-lite";

/// Whether the gateway has been connected at least once.
static GATEWAY_CONNECTED: AtomicBool = AtomicBool::new(false);
const IN_CLUSTER_ACP_GATEWAY_URL: &str = "ws://127.0.0.1:18789";
const AGENT_DEPLOYMENT_NAMESPACE: &str = "openclaw";
const AGENT_CONTAINER_NAME: &str = "agent";
const AVATAR_USER_PREFIX: &str = "morgan-avatar";

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

#[derive(Debug, Clone, Default)]
struct ConversationSessionRecord {
    messages: Vec<OpenClawMessage>,
    last_acp_session_id: Option<String>,
    gateway_session_key: String,
}

/// In-process conversation state used to preserve the desktop-visible message
/// history while Morgan itself owns the authoritative session in OpenClaw.
pub struct ConversationState {
    sessions: Mutex<HashMap<String, ConversationSessionRecord>>,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for ConversationState {
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
    #[serde(rename = "latencyMs")]
    pub latency_ms: Option<u64>,
    #[serde(rename = "gatewayUrl")]
    pub gateway_url: Option<String>,
    #[serde(rename = "gatewaySessionKey")]
    pub gateway_session_key: Option<String>,
    #[serde(rename = "acpSessionId")]
    pub acp_session_id: Option<String>,
    #[serde(rename = "stopReason")]
    pub stop_reason: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMorganHealth {
    pub expected_context: String,
    pub active_context: Option<String>,
    pub docker_available: bool,
    pub kind_context_configured: bool,
    pub kind_cluster_exists: bool,
    pub kind_context_reachable: bool,
    pub ingress_controller_ready: bool,
    pub morgan_deployment_ready: bool,
    pub morgan_service_present: bool,
    pub morgan_ingress_host: Option<String>,
    pub cto_tools_ready: bool,
    pub cto_openmemory_ready: bool,
    pub nats_ready: bool,
    pub gateway_reachable: bool,
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MorganDiagnostics {
    pub healthy: bool,
    pub model_primary: Option<String>,
    pub model_fallbacks: Vec<String>,
    pub catalog_source: Option<String>,
    pub catalog_generated_at: Option<String>,
    pub catalog_provider_count: usize,
    pub catalog_model_count: usize,
    pub recent_errors: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawMessage {
    role: String,
    content: String,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn gateway_url() -> String {
    std::env::var("OPENCLAW_GATEWAY_URL").unwrap_or_else(|_| DEFAULT_LOCAL_INGRESS_URL.to_string())
}

fn local_bridge_url() -> String {
    DEFAULT_LOCAL_BRIDGE_URL.to_string()
}

fn local_ingress_url() -> String {
    DEFAULT_LOCAL_INGRESS_URL.to_string()
}

fn gateway_auth_token() -> String {
    std::env::var("OPENCLAW_GATEWAY_TOKEN").unwrap_or_else(|_| DEFAULT_GATEWAY_TOKEN.to_string())
}

fn gateway_ws_url(base_url: &str) -> String {
    if let Some(stripped) = base_url.strip_prefix("http://") {
        format!("ws://{stripped}")
    } else if let Some(stripped) = base_url.strip_prefix("https://") {
        format!("wss://{stripped}")
    } else {
        base_url.to_string()
    }
}

fn gateway_request(client: &reqwest::Client, url: &str) -> reqwest::RequestBuilder {
    client
        .get(url)
        .bearer_auth(gateway_auth_token())
        .header("x-openclaw-agent-id", DEFAULT_AGENT_ID)
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

fn sanitize_session_component(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn normalize_agent_id(agent_id: Option<String>) -> String {
    let candidate = agent_id
        .unwrap_or_else(|| DEFAULT_AGENT_ID.to_string())
        .trim()
        .to_lowercase();
    if candidate.is_empty() {
        DEFAULT_AGENT_ID.to_string()
    } else {
        sanitize_session_component(&candidate)
    }
}

fn project_gateway_session_key(agent_id: &str, session_id: &str) -> String {
    format!(
        "agent:{agent_id}:openai-user:{AVATAR_USER_PREFIX}:{}",
        sanitize_session_component(session_id)
    )
}

fn agent_deployment_name(agent_id: &str) -> String {
    format!("openclaw-{agent_id}")
}

fn desktop_acp_cwd() -> PathBuf {
    std::env::var("OPENCLAW_ACP_CWD")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("/workspace"))
}

fn local_agent_acp_runtime(
    session_key: &str,
    gateway_url: &str,
) -> Result<cto_config::AcpRuntimeConfig, AppError> {
    let openclaw = which::which("openclaw")
        .map_err(|_| AppError::CommandFailed("openclaw is required for local ACP".to_string()))?;

    let mut runtime = cto_config::AcpRuntimeConfig::stdio(
        openclaw.to_string_lossy().to_string(),
        [
            "acp".to_string(),
            "--url".to_string(),
            gateway_ws_url(gateway_url),
            "--token".to_string(),
            gateway_auth_token(),
            "--session".to_string(),
            session_key.to_string(),
            "--no-prefix-cwd".to_string(),
        ],
    );
    runtime.env.insert(
        "OPENCLAW_ALLOW_INSECURE_PRIVATE_WS".to_string(),
        "1".to_string(),
    );
    Ok(runtime)
}

fn in_cluster_agent_acp_runtime(
    session_key: &str,
    agent_id: &str,
) -> Result<cto_config::AcpRuntimeConfig, AppError> {
    let kubectl = which::which("kubectl")
        .map_err(|_| AppError::CommandFailed("kubectl is required for agent ACP".to_string()))?;
    let deployment_name = agent_deployment_name(agent_id);

    Ok(cto_config::AcpRuntimeConfig::stdio(
        kubectl.to_string_lossy().to_string(),
        [
            "--context".to_string(),
            LOCAL_KIND_CONTEXT.to_string(),
            "exec".to_string(),
            "-i".to_string(),
            "-n".to_string(),
            AGENT_DEPLOYMENT_NAMESPACE.to_string(),
            format!("deploy/{deployment_name}"),
            "-c".to_string(),
            AGENT_CONTAINER_NAME.to_string(),
            "--".to_string(),
            "openclaw".to_string(),
            "acp".to_string(),
            "--url".to_string(),
            IN_CLUSTER_ACP_GATEWAY_URL.to_string(),
            "--token".to_string(),
            gateway_auth_token(),
            "--session".to_string(),
            session_key.to_string(),
            "--no-prefix-cwd".to_string(),
        ],
    ))
}

fn agent_acp_runtime(
    session_key: &str,
    agent_id: &str,
    gateway_url: Option<&str>,
) -> Result<cto_config::AcpRuntimeConfig, AppError> {
    if let Some(url) = gateway_url.filter(|value| !value.trim().is_empty()) {
        return local_agent_acp_runtime(session_key, url);
    }

    in_cluster_agent_acp_runtime(session_key, agent_id)
}

fn chunk_text(content: &ContentBlock) -> Option<&str> {
    match content {
        ContentBlock::Text(text) => Some(text.text.as_str()),
        _ => None,
    }
}

fn extract_agent_reply(notifications: &[SessionNotification]) -> String {
    notifications
        .iter()
        .filter_map(|notification| match &notification.update {
            SessionUpdate::AgentMessageChunk(chunk) => chunk_text(&chunk.content),
            _ => None,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

async fn run_morgan_acp_turn(
    runtime: cto_config::AcpRuntimeConfig,
    request: AcpPromptRequest,
    profile: AcpClientProfile,
) -> Result<acp_runtime::AcpPromptResult, AppError> {
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| {
                AppError::CommandFailed(format!("Failed to create ACP runtime: {error}"))
            })?;

        rt.block_on(run_oneshot_prompt(&runtime, request, profile))
            .map_err(|error| {
                AppError::CommandFailed(format!("Failed to run Morgan via ACP: {error:#}"))
            })
    })
    .await
    .map_err(|error| AppError::CommandFailed(format!("Morgan ACP task failed: {error}")))?
}

async fn fetch_gateway_status_payload(base_url: &str) -> Result<OpenClawStatus, AppError> {
    let api_status_url = format!("{}/api/status", base_url);
    if let Ok(response) = gateway_request(&fast_http_client(), &api_status_url)
        .send()
        .await
    {
        if response.status().is_success() {
            let status: OpenClawStatus = response
                .json()
                .await
                .map_err(|e| AppError::CommandFailed(format!("Invalid status response: {e}")))?;
            return Ok(status);
        }
    }

    let health_url = format!("{}/health", base_url);
    let response = gateway_request(&fast_http_client(), &health_url)
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

async fn gateway_url_is_reachable(base_url: &str) -> bool {
    fetch_gateway_status_payload(base_url)
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

fn discover_agent_service(agent_id: &str) -> Result<MorganServiceTarget, AppError> {
    if let Some(target) = configured_bridge_target() {
        return Ok(target);
    }

    let kubectl = which::which("kubectl").map_err(|_| {
        AppError::CommandFailed("kubectl is required to resolve Morgan".to_string())
    })?;

    let output = Command::new(kubectl)
        .args([
            "--context",
            LOCAL_KIND_CONTEXT,
            "get",
            "svc",
            "-A",
            "-l",
            &format!("openclaw.io/agent={agent_id}"),
            "-o",
            "json",
        ])
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
            item.spec.ports.iter().any(|port| {
                port.port == DEFAULT_GATEWAY_PORT || port.name.as_deref() == Some("gateway")
            })
        })
        .or_else(|| services.items.first())
        .ok_or_else(|| {
            AppError::CommandFailed(format!(
                "No Kubernetes service with label openclaw.io/agent={agent_id} was found"
            ))
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

fn kubectl_context_exists(context: &str) -> bool {
    let Ok(stdout) = kubectl_output(&["config", "get-contexts", context, "-o", "name"]) else {
        return false;
    };

    String::from_utf8_lossy(&stdout).trim() == context
}

fn kubectl_current_context() -> Option<String> {
    let stdout = kubectl_output(&["config", "current-context"]).ok()?;
    let value = String::from_utf8_lossy(&stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn kubectl_context_reachable(context: &str) -> bool {
    kubectl_output(&[
        "--context",
        context,
        "get",
        "namespace",
        "default",
        "-o",
        "name",
    ])
    .is_ok()
}

fn kind_cluster_exists(name: &str) -> bool {
    let Ok(kind) = which::which("kind") else {
        return false;
    };

    let Ok(output) = Command::new(kind).args(["get", "clusters"]).output() else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == name)
}

fn kind_ready_replicas(namespace: &str, deployment: &str) -> u32 {
    let Ok(stdout) = kubectl_output(&[
        "--context",
        LOCAL_KIND_CONTEXT,
        "get",
        "deployment",
        deployment,
        "-n",
        namespace,
        "-o",
        "jsonpath={.status.readyReplicas}",
    ]) else {
        return 0;
    };

    String::from_utf8_lossy(&stdout)
        .trim()
        .parse::<u32>()
        .unwrap_or(0)
}

fn kind_service_exists(namespace: &str, service: &str) -> bool {
    kubectl_output(&[
        "--context",
        LOCAL_KIND_CONTEXT,
        "get",
        "service",
        service,
        "-n",
        namespace,
        "-o",
        "name",
    ])
    .is_ok()
}

fn kind_ingress_host(namespace: &str, ingress: &str) -> Option<String> {
    let stdout = kubectl_output(&[
        "--context",
        LOCAL_KIND_CONTEXT,
        "get",
        "ingress",
        ingress,
        "-n",
        namespace,
        "-o",
        "jsonpath={.spec.rules[0].host}",
    ])
    .ok()?;

    let value = String::from_utf8_lossy(&stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

async fn collect_local_morgan_health() -> LocalMorganHealth {
    let docker_available = runtime::scan_runtime_environment().docker_available;
    let active_context = kubectl_current_context();
    let kind_context_configured = kubectl_context_exists(LOCAL_KIND_CONTEXT);
    let kind_cluster_exists = kind_cluster_exists("cto-lite");
    let kind_context_reachable =
        kind_context_configured && kubectl_context_reachable(LOCAL_KIND_CONTEXT);
    let ingress_controller_ready = kind_context_reachable
        && kind_ready_replicas("ingress-nginx", "ingress-nginx-controller") > 0;
    let morgan_deployment_ready =
        kind_context_reachable && kind_ready_replicas("openclaw", "openclaw-morgan") > 0;
    let morgan_service_present =
        kind_context_reachable && kind_service_exists("openclaw", "openclaw-morgan");
    let morgan_ingress_host = if kind_context_reachable {
        kind_ingress_host("openclaw", "openclaw-morgan")
    } else {
        None
    };
    let cto_tools_ready = kind_context_reachable && kind_ready_replicas("cto", "cto-tools") > 0;
    let cto_openmemory_ready =
        kind_context_reachable && kind_ready_replicas("cto", "cto-openmemory") > 0;
    let nats_ready = kind_context_reachable && kind_ready_replicas("messaging", "nats") > 0;
    let gateway_reachable = if kind_context_reachable {
        gateway_url_is_reachable(&local_ingress_url()).await
    } else {
        false
    };

    let mut problems = Vec::new();
    if !docker_available {
        problems.push("Docker-compatible runtime unavailable".to_string());
    }
    if !kind_cluster_exists {
        problems.push("kind cluster 'cto-lite' does not exist".to_string());
    }
    if !kind_context_configured {
        problems.push(format!("kubectl context '{LOCAL_KIND_CONTEXT}' is missing"));
    } else if !kind_context_reachable {
        problems.push(format!(
            "kubectl context '{LOCAL_KIND_CONTEXT}' is not reachable"
        ));
    }
    if kind_context_reachable && !ingress_controller_ready {
        problems.push("ingress-nginx controller is not ready in kind".to_string());
    }
    if kind_context_reachable && !morgan_deployment_ready {
        problems.push("Morgan deployment is not ready in kind".to_string());
    }
    if kind_context_reachable && !morgan_service_present {
        problems.push("Morgan service is missing in kind".to_string());
    }
    if kind_context_reachable && morgan_ingress_host.as_deref() != Some("morgan.localhost") {
        problems.push("Morgan ingress is missing or not bound to morgan.localhost".to_string());
    }
    if kind_context_reachable && !cto_tools_ready {
        problems.push("cto-tools deployment is not ready".to_string());
    }
    if kind_context_reachable && !cto_openmemory_ready {
        problems.push("cto-openmemory deployment is not ready".to_string());
    }
    if kind_context_reachable && !nats_ready {
        problems.push("nats deployment is not ready".to_string());
    }
    if kind_context_reachable && !gateway_reachable {
        problems.push("morgan.localhost is not reachable".to_string());
    }

    LocalMorganHealth {
        expected_context: LOCAL_KIND_CONTEXT.to_string(),
        active_context,
        docker_available,
        kind_context_configured,
        kind_cluster_exists,
        kind_context_reachable,
        ingress_controller_ready,
        morgan_deployment_ready,
        morgan_service_present,
        morgan_ingress_host,
        cto_tools_ready,
        cto_openmemory_ready,
        nats_ready,
        gateway_reachable,
        problems,
    }
}

fn ensure_local_morgan_exec_ready(health: &LocalMorganHealth) -> Result<(), AppError> {
    let mut blockers = Vec::new();
    if !health.docker_available {
        blockers.push("Docker-compatible runtime unavailable");
    }
    if !health.kind_cluster_exists {
        blockers.push("kind cluster 'cto-lite' missing");
    }
    if !health.kind_context_configured {
        blockers.push("kubectl context 'kind-cto-lite' missing");
    }
    if !health.kind_context_reachable {
        blockers.push("kubectl context 'kind-cto-lite' unreachable");
    }
    if !health.morgan_deployment_ready {
        blockers.push("Morgan deployment not ready");
    }

    if blockers.is_empty() {
        Ok(())
    } else {
        Err(AppError::CommandFailed(format!(
            "Local Morgan is not ready: {}",
            blockers.join("; ")
        )))
    }
}

fn kubectl_output(args: &[&str]) -> Result<Vec<u8>, AppError> {
    let kubectl = which::which("kubectl").map_err(|_| {
        AppError::CommandFailed("kubectl is required to inspect Morgan".to_string())
    })?;

    let output = Command::new(kubectl)
        .args(args)
        .output()
        .map_err(|error| AppError::CommandFailed(format!("Failed to run kubectl: {error}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!(
            "kubectl command failed: {}",
            stderr.trim()
        )));
    }

    Ok(output.stdout)
}

fn recent_backend_errors(logs: &str) -> Vec<String> {
    logs.lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("rate limit")
                || lower.contains("failovererror")
                || lower.contains("lane task error")
                || lower.contains("timed out")
                || lower.contains("error=")
        })
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

async fn get_local_bridge_status_inner(
    state: &LocalBridgeState,
    agent_id: &str,
) -> Result<OpenClawBridgeStatus, AppError> {
    let target = discover_agent_service(agent_id).ok();
    let pid = {
        let mut process = state
            .process
            .lock()
            .map_err(|e| AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}")))?;
        bridge_pid(&mut *process)?
    };
    let connected = gateway_url_is_reachable(&local_ingress_url()).await
        || gateway_url_is_reachable(&local_bridge_url()).await;

    Ok(build_bridge_status(
        target.as_ref(),
        pid.is_some(),
        connected,
        pid,
    ))
}

async fn start_local_bridge_inner(
    state: &LocalBridgeState,
    agent_id: &str,
) -> Result<OpenClawBridgeStatus, AppError> {
    let _startup = state.startup.lock().await;

    if gateway_url_is_reachable(&local_bridge_url()).await {
        return get_local_bridge_status_inner(state, agent_id).await;
    }

    let bridge_running = {
        let mut process = state
            .process
            .lock()
            .map_err(|e| AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}")))?;
        bridge_pid(&mut *process)?.is_some()
    };

    if bridge_running {
        return get_local_bridge_status_inner(state, agent_id).await;
    }

    let target = discover_agent_service(agent_id)?;
    let kubectl = which::which("kubectl")
        .map_err(|_| AppError::CommandFailed("kubectl is required to start Morgan".to_string()))?;

    tracing::info!(
        "Starting Morgan bridge via kubectl port-forward for {}/{}",
        target.namespace,
        target.service
    );

    let mut child = Command::new(kubectl)
        .args([
            "--context",
            LOCAL_KIND_CONTEXT,
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
        if gateway_url_is_reachable(&local_bridge_url()).await {
            let mut process = state.process.lock().map_err(|e| {
                AppError::CommandFailed(format!("Failed to acquire bridge lock: {e}"))
            })?;
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

async fn resolve_gateway_url(
    state: &LocalBridgeState,
    agent_id: &str,
    allow_start_bridge: bool,
) -> Result<String, AppError> {
    if let Ok(url) = std::env::var("OPENCLAW_GATEWAY_URL") {
        return Ok(url);
    }

    let ingress = local_ingress_url();
    if gateway_url_is_reachable(&ingress).await {
        return Ok(ingress);
    }

    let bridge = local_bridge_url();
    if gateway_url_is_reachable(&bridge).await {
        return Ok(bridge);
    }

    let allow_debug_port_forward = std::env::var("OPENCLAW_ALLOW_PORT_FORWARD")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);

    if allow_start_bridge && allow_debug_port_forward {
        start_local_bridge_inner(state, agent_id).await?;
        return Ok(bridge);
    }

    Ok(gateway_url())
}

fn load_session_messages(
    conversations: &ConversationState,
    session_id: &str,
) -> Result<Vec<OpenClawMessage>, AppError> {
    let sessions = conversations.sessions.lock().map_err(|e| {
        AppError::CommandFailed(format!("Failed to access conversation state: {e}"))
    })?;
    Ok(sessions
        .get(session_id)
        .map(|session| session.messages.clone())
        .unwrap_or_default())
}

fn update_session_record(
    conversations: &ConversationState,
    session_id: &str,
    gateway_session_key: &str,
    messages: &[OpenClawMessage],
    acp_session_id: Option<String>,
) -> Result<(), AppError> {
    let mut sessions = conversations.sessions.lock().map_err(|e| {
        AppError::CommandFailed(format!("Failed to update conversation state: {e}"))
    })?;
    let entry =
        sessions
            .entry(session_id.to_string())
            .or_insert_with(|| ConversationSessionRecord {
                gateway_session_key: gateway_session_key.to_string(),
                ..ConversationSessionRecord::default()
            });
    entry.gateway_session_key = gateway_session_key.to_string();
    entry.last_acp_session_id = acp_session_id;
    entry.messages.extend_from_slice(messages);
    Ok(())
}

// ─── Commands ───────────────────────────────────────────────────────────────

/// Start the local Morgan bridge.
#[tauri::command]
pub async fn openclaw_start_local_bridge(
    agent_id: Option<String>,
    state: State<'_, LocalBridgeState>,
) -> Result<OpenClawBridgeStatus, AppError> {
    let agent_id = normalize_agent_id(agent_id);
    start_local_bridge_inner(state.inner(), &agent_id).await
}

/// Stop the local Morgan bridge.
#[tauri::command]
pub async fn openclaw_stop_local_bridge(
    agent_id: Option<String>,
    state: State<'_, LocalBridgeState>,
) -> Result<OpenClawBridgeStatus, AppError> {
    let _startup = state.startup.lock().await;
    let agent_id = normalize_agent_id(agent_id);
    let target = discover_agent_service(&agent_id).ok();

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

    let connected = gateway_url_is_reachable(&local_ingress_url()).await
        || gateway_url_is_reachable(&local_bridge_url()).await;
    Ok(build_bridge_status(target.as_ref(), false, connected, pid))
}

/// Get local Morgan bridge status.
#[tauri::command]
pub async fn openclaw_get_local_bridge_status(
    agent_id: Option<String>,
    state: State<'_, LocalBridgeState>,
) -> Result<OpenClawBridgeStatus, AppError> {
    let agent_id = normalize_agent_id(agent_id);
    get_local_bridge_status_inner(state.inner(), &agent_id).await
}

#[tauri::command]
pub async fn openclaw_get_local_health() -> Result<LocalMorganHealth, AppError> {
    Ok(collect_local_morgan_health().await)
}

#[tauri::command]
pub async fn openclaw_get_morgan_diagnostics(
    agent_id: Option<String>,
    bridge: State<'_, LocalBridgeState>,
) -> Result<MorganDiagnostics, AppError> {
    let agent_id = normalize_agent_id(agent_id);
    let deployment_name = agent_deployment_name(&agent_id);
    let health = collect_local_morgan_health().await;
    let healthy = if health.gateway_reachable {
        true
    } else if let Ok(url) = resolve_gateway_url(bridge.inner(), &agent_id, false).await {
        gateway_url_is_reachable(&url).await
    } else {
        false
    };

    let mut model_primary = None;
    let mut model_fallbacks = Vec::new();
    let mut catalog_source = None;
    let mut catalog_generated_at = None;
    let mut catalog_provider_count = 0;
    let mut catalog_model_count = 0;
    let mut recent_errors = health.problems.clone();

    if health.kind_context_reachable && health.morgan_deployment_ready {
        if let Ok(config_bytes) = kubectl_output(&[
            "--context",
            LOCAL_KIND_CONTEXT,
            "-n",
            AGENT_DEPLOYMENT_NAMESPACE,
            "exec",
            &format!("deploy/{deployment_name}"),
            "-c",
            AGENT_CONTAINER_NAME,
            "--",
            "sh",
            "-lc",
            "cat /workspace/.openclaw/openclaw.json",
        ]) {
            if let Ok(config) = serde_json::from_slice::<serde_json::Value>(&config_bytes) {
                model_primary = config
                    .pointer("/agents/defaults/model/primary")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned);
                model_fallbacks = config
                    .pointer("/agents/defaults/model/fallbacks")
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                catalog_source = config
                    .pointer("/models/catalog/source")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned);
                catalog_generated_at = config
                    .pointer("/models/catalog/generatedAt")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned);
                let counts = config
                    .pointer("/models/providers")
                    .and_then(|value| value.as_object())
                    .map(|providers| {
                        let model_count = providers
                            .values()
                            .filter_map(|provider| {
                                provider.get("models").and_then(|models| models.as_array())
                            })
                            .map(std::vec::Vec::len)
                            .sum::<usize>();
                        (providers.len(), model_count)
                    })
                    .unwrap_or((0, 0));
                catalog_provider_count = counts.0;
                catalog_model_count = counts.1;
            } else {
                recent_errors.push("Unable to parse live Morgan config".to_string());
            }
        } else {
            recent_errors.push("Unable to read live Morgan config".to_string());
        }

        match kubectl_output(&[
            "--context",
            LOCAL_KIND_CONTEXT,
            "-n",
            AGENT_DEPLOYMENT_NAMESPACE,
            "logs",
            &format!("deploy/{deployment_name}"),
            "-c",
            AGENT_CONTAINER_NAME,
            "--since=10m",
            "--tail=200",
        ]) {
            Ok(log_bytes) => {
                if let Ok(logs) = String::from_utf8(log_bytes) {
                    recent_errors.extend(recent_backend_errors(&logs));
                } else {
                    recent_errors.push("Unable to decode Morgan logs".to_string());
                }
            }
            Err(_) => {
                recent_errors.push("Unable to read recent Morgan logs".to_string());
            }
        }
    }

    recent_errors = recent_errors
        .into_iter()
        .filter(|entry| !entry.trim().is_empty())
        .collect::<Vec<_>>();

    Ok(MorganDiagnostics {
        healthy,
        model_primary,
        model_fallbacks,
        catalog_source,
        catalog_generated_at,
        catalog_provider_count,
        catalog_model_count,
        recent_errors,
    })
}

/// Send a message to the PM agent (`Morgan`) via the `OpenClaw` gateway.
#[tauri::command]
pub async fn openclaw_send_message(
    session_id: String,
    message: String,
    agent_id: Option<String>,
    bridge: State<'_, LocalBridgeState>,
    conversations: State<'_, ConversationState>,
) -> Result<OpenClawResponse, AppError> {
    let started_at = Instant::now();
    let agent_id = normalize_agent_id(agent_id);
    let health = collect_local_morgan_health().await;
    ensure_local_morgan_exec_ready(&health)?;
    let deployment_name = agent_deployment_name(&agent_id);
    let gateway_session_key = project_gateway_session_key(&agent_id, &session_id);
    let gateway_base_url = match start_local_bridge_inner(bridge.inner(), &agent_id).await {
        Ok(status) if status.running => Some(local_bridge_url()),
        _ => None,
    };
    let runtime = agent_acp_runtime(&gateway_session_key, &agent_id, gateway_base_url.as_deref())?;
    let user_message = OpenClawMessage {
        role: "user".to_string(),
        content: message.clone(),
    };
    let result = run_morgan_acp_turn(
        runtime,
        AcpPromptRequest {
            runtime_id: agent_id.clone(),
            cwd: desktop_acp_cwd(),
            prompt: message,
            // We intentionally create a fresh ACP session per turn and bind it
            // to a stable OpenClaw session key. That keeps Morgan's context in
            // the Gateway without depending on bridge-local ACP session ids.
            session_id: None,
        },
        AcpClientProfile {
            name: "cto-morgan-desktop".to_string(),
            title: "CTO Morgan Desktop".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            permission_policy: AcpPermissionPolicy::AllowAll,
        },
    )
    .await?;

    let assistant_text = extract_agent_reply(&result.notifications);
    if assistant_text.is_empty() {
        return Err(AppError::CommandFailed(format!(
            "Morgan ACP session ended without assistant text ({:?})",
            result.stop_reason
        )));
    }

    let assistant_message = OpenClawMessage {
        role: "assistant".to_string(),
        content: assistant_text.clone(),
    };

    update_session_record(
        conversations.inner(),
        &session_id,
        &gateway_session_key,
        &[user_message, assistant_message],
        Some(result.session_id.clone()),
    )?;

    let latency_ms = started_at.elapsed().as_millis() as u64;
    tracing::info!(
        session_id = %session_id,
        gateway_session_key = %gateway_session_key,
        latency_ms,
        stop_reason = ?result.stop_reason,
        "Morgan ACP turn finished"
    );

    GATEWAY_CONNECTED.store(true, Ordering::Relaxed);
    Ok(OpenClawResponse {
        content: assistant_text,
        action: None,
        latency_ms: Some(latency_ms),
        gateway_url: gateway_base_url.or_else(|| {
            Some(format!(
                "k8s://{}/{}/{}",
                LOCAL_KIND_CONTEXT, AGENT_DEPLOYMENT_NAMESPACE, deployment_name
            ))
        }),
        gateway_session_key: Some(gateway_session_key),
        acp_session_id: Some(result.session_id),
        stop_reason: Some(format!("{:?}", result.stop_reason)),
    })
}

/// Inject pasted context into the active Morgan avatar room session.
#[tauri::command]
pub async fn openclaw_send_avatar_context(
    room_name: String,
    content: String,
    agent_id: Option<String>,
    bridge: State<'_, LocalBridgeState>,
    conversations: State<'_, ConversationState>,
) -> Result<OpenClawResponse, AppError> {
    let trimmed_room = room_name.trim();
    let trimmed_content = content.trim();

    if trimmed_room.is_empty() {
        return Err(AppError::CommandFailed(
            "Avatar room name is required before sending context".to_string(),
        ));
    }

    if trimmed_content.is_empty() {
        return Err(AppError::CommandFailed(
            "Context cannot be empty".to_string(),
        ));
    }

    let started_at = Instant::now();
    let agent_id = normalize_agent_id(agent_id);
    let health = collect_local_morgan_health().await;
    ensure_local_morgan_exec_ready(&health)?;
    let deployment_name = agent_deployment_name(&agent_id);
    let gateway_session_key = project_gateway_session_key(&agent_id, trimmed_room);
    let gateway_base_url = match start_local_bridge_inner(bridge.inner(), &agent_id).await {
        Ok(status) if status.running => Some(local_bridge_url()),
        _ => None,
    };
    let runtime = agent_acp_runtime(&gateway_session_key, &agent_id, gateway_base_url.as_deref())?;
    let prompt = format!(
        "The user has pasted supporting context for the active voice call. \
Treat it as additional material for the current task and use it in the next reply.\n\n\
Context:\n{trimmed_content}\n\nReply exactly: CONTEXT_STORED"
    );

    let result = run_morgan_acp_turn(
        runtime,
        AcpPromptRequest {
            runtime_id: agent_id.clone(),
            cwd: desktop_acp_cwd(),
            prompt,
            session_id: None,
        },
        AcpClientProfile {
            name: "cto-morgan-avatar-context".to_string(),
            title: "CTO Morgan Avatar Context".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            permission_policy: AcpPermissionPolicy::AllowAll,
        },
    )
    .await?;

    let assistant_text = extract_agent_reply(&result.notifications);
    if assistant_text.is_empty() {
        return Err(AppError::CommandFailed(
            "Morgan did not acknowledge the pasted context".to_string(),
        ));
    }

    let user_message = OpenClawMessage {
        role: "user".to_string(),
        content: format!("[Shared context]\n{trimmed_content}"),
    };
    let assistant_message = OpenClawMessage {
        role: "assistant".to_string(),
        content: assistant_text.clone(),
    };

    update_session_record(
        conversations.inner(),
        trimmed_room,
        &gateway_session_key,
        &[user_message, assistant_message],
        Some(result.session_id.clone()),
    )?;

    Ok(OpenClawResponse {
        content: assistant_text,
        action: None,
        latency_ms: Some(started_at.elapsed().as_millis() as u64),
        gateway_url: gateway_base_url.or_else(|| {
            Some(format!(
                "k8s://{}/{}/{}",
                LOCAL_KIND_CONTEXT, AGENT_DEPLOYMENT_NAMESPACE, deployment_name
            ))
        }),
        gateway_session_key: Some(gateway_session_key),
        acp_session_id: Some(result.session_id),
        stop_reason: Some(format!("{:?}", result.stop_reason)),
    })
}

/// Start a `Lobster` workflow via the `OpenClaw` gateway.
#[tauri::command]
pub async fn openclaw_start_workflow(
    workflow_type: String,
    params: HashMap<String, String>,
    bridge: State<'_, LocalBridgeState>,
) -> Result<WorkflowStartResult, AppError> {
    let base_url = resolve_gateway_url(bridge.inner(), DEFAULT_AGENT_ID, false).await?;
    let url = format!("{}/api/workflows", base_url);

    let body = serde_json::json!({
        "type": workflow_type,
        "params": params,
    });

    let resp = http_client()
        .post(&url)
        .bearer_auth(gateway_auth_token())
        .header("x-openclaw-agent-id", DEFAULT_AGENT_ID)
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
    let base_url = resolve_gateway_url(bridge.inner(), DEFAULT_AGENT_ID, false).await?;
    let url = format!("{}/api/workflows/{}/approve", base_url, workflow_id);

    let resp = http_client()
        .post(&url)
        .bearer_auth(gateway_auth_token())
        .header("x-openclaw-agent-id", DEFAULT_AGENT_ID)
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
    let base_url = match resolve_gateway_url(bridge.inner(), DEFAULT_AGENT_ID, false).await {
        Ok(url) => url,
        Err(_) => gateway_url(),
    };

    match fetch_gateway_status_payload(&base_url).await {
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
    conversations: State<'_, ConversationState>,
) -> Result<Vec<OpenClawMessage>, AppError> {
    load_session_messages(conversations.inner(), &session_id)
}

/// Reject a pending workflow approval gate.
#[tauri::command]
pub async fn openclaw_reject(
    workflow_id: String,
    reason: String,
    bridge: State<'_, LocalBridgeState>,
) -> Result<(), AppError> {
    let base_url = resolve_gateway_url(bridge.inner(), DEFAULT_AGENT_ID, false).await?;
    let url = format!("{}/api/workflows/{}/reject", base_url, workflow_id);

    let body = serde_json::json!({ "reason": reason });

    let resp = http_client()
        .post(&url)
        .bearer_auth(gateway_auth_token())
        .header("x-openclaw-agent-id", DEFAULT_AGENT_ID)
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
    let base_url = resolve_gateway_url(bridge.inner(), DEFAULT_AGENT_ID, false).await?;
    let url = format!("{}/api/workflows/{}", base_url, workflow_id);

    let resp = http_client()
        .get(&url)
        .bearer_auth(gateway_auth_token())
        .header("x-openclaw-agent-id", DEFAULT_AGENT_ID)
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
    let base_url = resolve_gateway_url(bridge.inner(), DEFAULT_AGENT_ID, false).await?;
    let url = format!("{}/api/cli", base_url);

    let body = serde_json::json!({
        "cli": cli,
        "args": args,
    });

    let resp = http_client()
        .post(&url)
        .bearer_auth(gateway_auth_token())
        .header("x-openclaw-agent-id", DEFAULT_AGENT_ID)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn morgan_acp_test_lock() -> &'static tokio::sync::Mutex<()> {
        static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    #[tokio::test]
    #[ignore = "requires a running local kind cluster with Morgan deployed"]
    async fn smoke_morgan_acp_turn_returns_text() {
        let _guard = morgan_acp_test_lock().lock().await;
        let runtime = agent_acp_runtime("agent:morgan:test-acp-smoke", DEFAULT_AGENT_ID, None)
            .expect("runtime");
        let result = run_morgan_acp_turn(
            runtime,
            AcpPromptRequest {
                runtime_id: DEFAULT_AGENT_ID.to_string(),
                cwd: desktop_acp_cwd(),
                prompt: "Reply with exactly CTO_ACP_SMOKE.".to_string(),
                session_id: None,
            },
            AcpClientProfile {
                name: "cto-acp-smoke".to_string(),
                title: "CTO ACP Smoke".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                permission_policy: AcpPermissionPolicy::AllowAll,
            },
        )
        .await
        .expect("acp turn should succeed");

        let assistant_text = extract_agent_reply(&result.notifications);
        assert!(
            !assistant_text.trim().is_empty(),
            "expected assistant text, stop_reason={:?}",
            result.stop_reason
        );
    }

    #[tokio::test]
    #[ignore = "requires a running local kind cluster with Morgan deployed"]
    async fn smoke_morgan_acp_turn_via_local_bridge_returns_text() {
        let _guard = morgan_acp_test_lock().lock().await;
        let bridge = LocalBridgeState::new();
        let bridge_status = start_local_bridge_inner(&bridge, DEFAULT_AGENT_ID)
            .await
            .expect("bridge should start");
        assert!(
            bridge_status.connected,
            "expected local bridge connectivity, status={bridge_status:?}"
        );

        let runtime = agent_acp_runtime(
            "agent:morgan:test-acp-bridge-smoke",
            DEFAULT_AGENT_ID,
            Some(DEFAULT_LOCAL_BRIDGE_URL),
        )
        .expect("runtime");
        let result = run_morgan_acp_turn(
            runtime,
            AcpPromptRequest {
                runtime_id: DEFAULT_AGENT_ID.to_string(),
                cwd: desktop_acp_cwd(),
                prompt: "Reply with exactly CTO_ACP_BRIDGE_SMOKE.".to_string(),
                session_id: None,
            },
            AcpClientProfile {
                name: "cto-acp-bridge-smoke".to_string(),
                title: "CTO ACP Bridge Smoke".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                permission_policy: AcpPermissionPolicy::AllowAll,
            },
        )
        .await
        .expect("bridge ACP turn should succeed");

        if let Ok(mut process) = bridge.process.lock() {
            if let Some(mut child) = process.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }

        let assistant_text = extract_agent_reply(&result.notifications);
        assert!(
            !assistant_text.trim().is_empty(),
            "expected assistant text, stop_reason={:?}",
            result.stop_reason
        );
    }

    #[tokio::test]
    #[ignore = "requires a running local kind cluster with Morgan deployed"]
    async fn smoke_morgan_acp_session_persists_across_turns() {
        let _guard = morgan_acp_test_lock().lock().await;
        let session_key = format!(
            "agent:morgan:test-acp-session-persistence-{}",
            uuid::Uuid::new_v4()
        );
        let runtime = agent_acp_runtime(&session_key, DEFAULT_AGENT_ID, None).expect("runtime");

        let _first = run_morgan_acp_turn(
            runtime.clone(),
            AcpPromptRequest {
                runtime_id: DEFAULT_AGENT_ID.to_string(),
                cwd: desktop_acp_cwd(),
                prompt: "Remember the codeword RIVERSTONE and reply exactly READY.".to_string(),
                session_id: None,
            },
            AcpClientProfile {
                name: "cto-acp-session-smoke".to_string(),
                title: "CTO ACP Session Smoke".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                permission_policy: AcpPermissionPolicy::AllowAll,
            },
        )
        .await
        .expect("first ACP turn should succeed");

        let _second = run_morgan_acp_turn(
            runtime,
            AcpPromptRequest {
                runtime_id: DEFAULT_AGENT_ID.to_string(),
                cwd: desktop_acp_cwd(),
                prompt: "What codeword did I ask you to remember? Reply with only the codeword."
                    .to_string(),
                session_id: None,
            },
            AcpClientProfile {
                name: "cto-acp-session-smoke".to_string(),
                title: "CTO ACP Session Smoke".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                permission_policy: AcpPermissionPolicy::AllowAll,
            },
        )
        .await
        .expect("second ACP turn should succeed");

        let session_log = kubectl_output(&[
            "--context",
            LOCAL_KIND_CONTEXT,
            "-n",
            AGENT_DEPLOYMENT_NAMESPACE,
            "exec",
            &format!("deploy/{}", agent_deployment_name(DEFAULT_AGENT_ID)),
            "-c",
            AGENT_CONTAINER_NAME,
            "--",
            "sh",
            "-lc",
            &format!(
                "rg -n '{}' /workspace/.openclaw/logs/openclaw.log | tail -n 40",
                session_key
            ),
        ])
        .expect("session log should be readable");
        let session_text =
            String::from_utf8(session_log).expect("session log should be valid utf-8");
        let counts = regex::Regex::new(r"\[context-diag\] pre-prompt:.*messages=(\d+)")
            .expect("regex")
            .captures_iter(&session_text)
            .filter_map(|capture| {
                capture
                    .get(1)
                    .and_then(|value| value.as_str().parse::<u32>().ok())
            })
            .collect::<Vec<_>>();

        assert!(
            counts.len() >= 2,
            "expected at least two context snapshots for session, log={session_text}"
        );
        assert!(
            counts.iter().copied().max().unwrap_or_default() > 0,
            "expected later turn to include prior messages, counts={counts:?}, log={session_text}"
        );
    }
}
