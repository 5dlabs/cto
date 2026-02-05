//! Tauri commands exposed to the frontend

use crate::docker::{self, DockerInfo};
use crate::helm::{self, HelmRelease, HelmValues};
use crate::keychain::{self, ApiKeyType};
use crate::kind::{self, ClusterInfo, KindInfo};
use crate::state::{AppState, SetupState};
use crate::workflows::{self, WorkflowDetail, WorkflowParams, WorkflowStatus};
use serde_json::{json, Value};
use tauri::State;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use uuid::Uuid;

// ============================================================================
// MCP Server Management
// ============================================================================

/// Spawn the MCP server as a child process and initialize it
#[tauri::command]
pub async fn spawn_mcp_server(state: State<'_, AppState>) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();

    // Build the path to the mcp-lite binary
    let binary_path = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Could not determine executable directory")?
        .to_path_buf();

    // Try to find the mcp-lite binary (with .exe on Windows)
    #[cfg(target_os = "windows")]
    let mcp_binary = binary_path.join("mcp-lite.exe");

    #[cfg(not(target_os = "windows"))]
    let mcp_binary = binary_path.join("mcp-lite");

    let mut child = Command::new(&mcp_binary)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to spawn MCP server: {}", e))?;

    // Create buffered reader for stdout
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture MCP server stdout")?;
    let reader = BufReader::new(stdout);

    // Store the child process with its reader
    {
        let mut sessions = state.mcp_sessions.lock().await;
        sessions.insert(
            session_id.clone(),
            crate::state::McpSession { child, reader },
        );
    }

    Ok(session_id)
}

/// Send a request to the MCP server and get the response
#[tauri::command]
pub async fn mcp_call(
    state: State<'_, AppState>,
    session_id: String,
    method: String,
    params: Value,
) -> Result<Value, String> {
    let mut sessions = state.mcp_sessions.lock().await;

    let session = sessions.get_mut(&session_id).ok_or("Session not found")?;

    let stdin = session
        .child
        .stdin
        .as_mut()
        .ok_or("MCP server stdin not available")?;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let request_str = serde_json::to_string(&request).map_err(|e| e.to_string())?;

    stdin
        .write_all((request_str + "\n").as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    // Flush stdin to ensure the request is sent before reading response
    stdin.flush().await.map_err(|e| e.to_string())?;

    // Use the persistent reader to avoid losing buffered data
    let mut response = String::new();

    // Add timeout to prevent deadlock if MCP server hangs
    let read_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        session.reader.read_line(&mut response),
    )
    .await;

    // If timeout occurs, remove the session to prevent stale data issues
    let read_result = match read_result {
        Err(_) => {
            drop(sessions); // Drop the lock before removing
            let mut sessions = state.mcp_sessions.lock().await;
            sessions.remove(&session_id);
            return Err("MCP server request timed out after 30 seconds".to_string());
        }
        Ok(result) => result.map_err(|e| e.to_string())?,
    };

    if read_result == 0 {
        return Err("MCP server closed connection".to_string());
    }

    let response_value: Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;

    Ok(response_value)
}

/// Get a list of available MCP tools
#[tauri::command]
pub async fn get_mcp_tools(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Value, String> {
    mcp_call(state, session_id, "tools/list", json!({})).await
}

/// Call an MCP tool
#[tauri::command]
pub async fn call_mcp_tool(
    state: State<'_, AppState>,
    session_id: String,
    tool_name: String,
    arguments: Value,
) -> Result<Value, String> {
    mcp_call(
        state,
        session_id,
        "tools/call",
        json!({
            "name": tool_name,
            "arguments": arguments
        }),
    )
    .await
}

/// Kill the MCP server for a session
#[tauri::command]
pub async fn kill_mcp_server(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let mut sessions = state.mcp_sessions.lock().await;

    if let Some(mut session) = sessions.remove(&session_id) {
        session
            .child
            .kill()
            .await
            .map_err(|e| format!("Failed to kill MCP server: {}", e))?;
    }

    Ok(())
}

// ============================================================================
// MCP Task Commands (convenience wrappers)
// ============================================================================

/// Get tasks from the MCP server (wraps cto_jobs)
#[tauri::command]
pub async fn get_tasks(
    state: State<'_, AppState>,
    session_id: String,
    limit: Option<i64>,
) -> Result<Value, String> {
    call_mcp_tool(
        state,
        session_id,
        "cto_jobs".to_string(),
        json!({
            "limit": limit.unwrap_or(10)
        }),
    )
    .await
}

/// Create a task by triggering a workflow (wraps cto_trigger)
#[tauri::command]
pub async fn create_task(
    state: State<'_, AppState>,
    session_id: String,
    repo: String,
    prompt: String,
    issue_number: Option<i64>,
    stack: Option<String>,
) -> Result<Value, String> {
    call_mcp_tool(
        state,
        session_id,
        "cto_trigger".to_string(),
        json!({
            "repo": repo,
            "prompt": prompt,
            "issue_number": issue_number,
            "stack": stack.unwrap_or_else(|| "nova".to_string())
        }),
    )
    .await
}

/// Update task status (wraps cto_status)
#[tauri::command]
pub async fn update_task_status(
    state: State<'_, AppState>,
    session_id: String,
    workflow_id: String,
) -> Result<Value, String> {
    call_mcp_tool(
        state,
        session_id,
        "cto_status".to_string(),
        json!({
            "workflow_id": workflow_id
        }),
    )
    .await
}

// ============================================================================
// Setup Commands
// ============================================================================

/// Check Docker installation and status
#[tauri::command]
pub async fn check_docker() -> Result<DockerInfo, String> {
    docker::check_docker().map_err(|e| e.to_string())
}

/// Check Kind installation
#[tauri::command]
pub async fn check_kind() -> Result<KindInfo, String> {
    kind::check_kind().map_err(|e| e.to_string())
}

/// Get current setup wizard state
#[tauri::command]
pub async fn get_setup_state(state: State<'_, AppState>) -> Result<SetupState, String> {
    let setup = state.setup.read().await;
    Ok(setup.clone())
}

/// Save setup wizard state
#[tauri::command]
pub async fn save_setup_state(
    state: State<'_, AppState>,
    setup_state: SetupState,
) -> Result<(), String> {
    let mut setup = state.setup.write().await;
    *setup = setup_state;
    Ok(())
}

/// Mark setup as complete
#[tauri::command]
pub async fn complete_setup(state: State<'_, AppState>) -> Result<(), String> {
    let mut setup = state.setup.write().await;
    setup.completed = true;
    Ok(())
}

// ============================================================================
// Keychain Commands
// ============================================================================

/// Store an API key in the system keychain
#[tauri::command]
pub async fn store_api_key(key_type: String, value: String) -> Result<(), String> {
    let key_type =
        ApiKeyType::from_str(&key_type).ok_or_else(|| format!("Unknown key type: {}", key_type))?;

    keychain::store_key(key_type, &value).map_err(|e| e.to_string())
}

/// Get an API key from the system keychain
#[tauri::command]
pub async fn get_api_key(key_type: String) -> Result<Option<String>, String> {
    let key_type =
        ApiKeyType::from_str(&key_type).ok_or_else(|| format!("Unknown key type: {}", key_type))?;

    keychain::get_key(key_type).map_err(|e| e.to_string())
}

/// Delete an API key from the system keychain
#[tauri::command]
pub async fn delete_api_key(key_type: String) -> Result<(), String> {
    let key_type =
        ApiKeyType::from_str(&key_type).ok_or_else(|| format!("Unknown key type: {}", key_type))?;

    keychain::delete_key(key_type).map_err(|e| e.to_string())
}

/// Check if an API key exists in the keychain
#[tauri::command]
pub async fn has_api_key(key_type: String) -> Result<bool, String> {
    let key_type =
        ApiKeyType::from_str(&key_type).ok_or_else(|| format!("Unknown key type: {}", key_type))?;

    keychain::has_key(key_type).map_err(|e| e.to_string())
}

// ============================================================================
// Cluster Commands
// ============================================================================

/// Create the CTO Lite Kind cluster
#[tauri::command]
pub async fn create_cluster() -> Result<(), String> {
    kind::create_cluster().map_err(|e| e.to_string())
}

/// Delete the CTO Lite Kind cluster
#[tauri::command]
pub async fn delete_cluster() -> Result<(), String> {
    kind::delete_cluster().map_err(|e| e.to_string())
}

/// Get cluster status
#[tauri::command]
pub async fn get_cluster_status() -> Result<ClusterInfo, String> {
    kind::get_cluster_status().map_err(|e| e.to_string())
}

/// List all Kind clusters
#[tauri::command]
pub async fn list_clusters() -> Result<Vec<String>, String> {
    kind::list_clusters().map_err(|e| e.to_string())
}

// ============================================================================
// Workflow Commands
// ============================================================================

/// Trigger a new workflow
#[tauri::command]
pub async fn trigger_workflow(
    repo_url: String,
    branch: Option<String>,
    prompt: String,
    stack: Option<String>,
) -> Result<String, String> {
    let params = WorkflowParams {
        repo_url,
        branch,
        prompt,
        stack,
    };
    workflows::trigger_workflow(&params)
        .await
        .map_err(|e| e.to_string())
}

/// Get workflow status
#[tauri::command]
pub async fn get_workflow_status(workflow_name: String) -> Result<WorkflowDetail, String> {
    workflows::get_workflow(&workflow_name)
        .await
        .map_err(|e| e.to_string())
}

/// List all workflows
#[tauri::command]
pub async fn list_workflows() -> Result<Vec<WorkflowStatus>, String> {
    workflows::list_workflows().await.map_err(|e| e.to_string())
}

/// Get workflow logs
#[tauri::command]
pub async fn get_workflow_logs(
    workflow_name: String,
    node_name: Option<String>,
) -> Result<String, String> {
    workflows::get_workflow_logs(&workflow_name, node_name.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Delete a workflow
#[tauri::command]
pub async fn delete_workflow(workflow_name: String) -> Result<(), String> {
    workflows::delete_workflow(&workflow_name)
        .await
        .map_err(|e| e.to_string())
}

/// Stop a running workflow
#[tauri::command]
pub async fn stop_workflow(workflow_name: String) -> Result<(), String> {
    workflows::stop_workflow(&workflow_name)
        .await
        .map_err(|e| e.to_string())
}

/// Check if Argo Workflows is available
#[tauri::command]
pub async fn check_argo() -> Result<bool, String> {
    workflows::check_argo().await.map_err(|e| e.to_string())
}

// ============================================================================
// Helm Commands
// ============================================================================

/// Check if Helm is installed
#[tauri::command]
pub async fn check_helm() -> Result<Option<String>, String> {
    helm::check_helm().await.map_err(|e| e.to_string())
}

/// Deploy the CTO Lite Helm chart
#[tauri::command]
pub async fn deploy_chart(values: HelmValues) -> Result<(), String> {
    helm::deploy_chart(&values).await.map_err(|e| e.to_string())
}

/// Get the status of the Helm release
#[tauri::command]
pub async fn get_release_status() -> Result<Option<HelmRelease>, String> {
    helm::get_release_status().await.map_err(|e| e.to_string())
}

/// Uninstall the CTO Lite Helm chart
#[tauri::command]
pub async fn uninstall_chart() -> Result<(), String> {
    helm::uninstall_chart().await.map_err(|e| e.to_string())
}

/// Update Helm dependencies
#[tauri::command]
pub async fn update_helm_dependencies() -> Result<(), String> {
    helm::update_dependencies().await.map_err(|e| e.to_string())
}
