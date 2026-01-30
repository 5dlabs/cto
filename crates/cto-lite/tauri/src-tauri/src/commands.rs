//! Tauri commands exposed to the frontend

use crate::docker::{self, DockerInfo};
use crate::helm::{self, HelmRelease, HelmValues};
use crate::keychain::{self, ApiKeyType};
use crate::kind::{self, ClusterInfo, KindInfo};
use crate::state::{AppState, SetupState};
use serde::{Deserialize, Serialize};
use tauri::State;

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
    let key_type = ApiKeyType::from_str(&key_type)
        .ok_or_else(|| format!("Unknown key type: {}", key_type))?;
    
    keychain::store_key(key_type, &value).map_err(|e| e.to_string())
}

/// Get an API key from the system keychain
#[tauri::command]
pub async fn get_api_key(key_type: String) -> Result<Option<String>, String> {
    let key_type = ApiKeyType::from_str(&key_type)
        .ok_or_else(|| format!("Unknown key type: {}", key_type))?;
    
    keychain::get_key(key_type).map_err(|e| e.to_string())
}

/// Delete an API key from the system keychain
#[tauri::command]
pub async fn delete_api_key(key_type: String) -> Result<(), String> {
    let key_type = ApiKeyType::from_str(&key_type)
        .ok_or_else(|| format!("Unknown key type: {}", key_type))?;
    
    keychain::delete_key(key_type).map_err(|e| e.to_string())
}

/// Check if an API key exists in the keychain
#[tauri::command]
pub async fn has_api_key(key_type: String) -> Result<bool, String> {
    let key_type = ApiKeyType::from_str(&key_type)
        .ok_or_else(|| format!("Unknown key type: {}", key_type))?;
    
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

/// Workflow trigger request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTriggerRequest {
    pub repo_url: String,
    pub branch: Option<String>,
    pub prompt: String,
}

/// Workflow status response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub message: Option<String>,
}

/// Trigger a new workflow
#[tauri::command]
pub async fn trigger_workflow(request: WorkflowTriggerRequest) -> Result<String, String> {
    // TODO: Implement workflow triggering via PM-Lite
    tracing::info!("Triggering workflow for repo: {}", request.repo_url);
    
    // For now, return a placeholder
    Ok("workflow-placeholder-id".to_string())
}

/// Get workflow status
#[tauri::command]
pub async fn get_workflow_status(workflow_id: String) -> Result<WorkflowStatus, String> {
    // TODO: Implement workflow status retrieval
    tracing::info!("Getting status for workflow: {}", workflow_id);
    
    Ok(WorkflowStatus {
        id: workflow_id,
        name: "placeholder-workflow".to_string(),
        status: "Pending".to_string(),
        started_at: None,
        finished_at: None,
        message: None,
    })
}

/// List all workflows
#[tauri::command]
pub async fn list_workflows() -> Result<Vec<WorkflowStatus>, String> {
    // TODO: Implement workflow listing
    Ok(vec![])
}

/// Get workflow logs
#[tauri::command]
pub async fn get_workflow_logs(
    workflow_id: String,
    node_name: Option<String>,
) -> Result<String, String> {
    // TODO: Implement log retrieval
    tracing::info!("Getting logs for workflow: {}, node: {:?}", workflow_id, node_name);
    
    Ok("Workflow logs will appear here...".to_string())
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
