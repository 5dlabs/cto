//! Tauri commands exposed to the frontend

use crate::docker::{self, DockerInfo};
use crate::helm::{self, HelmRelease, HelmValues};
use crate::keychain::{self, ApiKeyType};
use crate::kind::{self, ClusterInfo, KindInfo};
use crate::state::{AppState, SetupState};
use crate::workflows::{self, WorkflowDetail, WorkflowNode, WorkflowParams, WorkflowStatus};
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
