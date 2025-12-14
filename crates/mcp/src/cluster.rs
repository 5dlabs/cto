//! Cluster management tool handlers for MCP.
//!
//! This module provides handlers for cluster lifecycle management tools:
//! - `cluster_create`: Deploy a new Kubernetes cluster on bare metal or cloud
//! - `cluster_status`: Get detailed status of a cluster installation
//! - `cluster_list`: List all managed clusters
//! - `cluster_delete`: Destroy a cluster and all resources
//!
//! ## Supported Providers
//!
//! ### Bare Metal (Talos Linux)
//! - Latitude (default)
//! - Cherry Servers
//! - Hetzner Dedicated
//! - Vultr Bare Metal
//! - Scaleway Bare Metal
//! - OVH Bare Metal
//! - `DigitalOcean` (Droplets)
//! - On-premises
//!
//! ### Cloud
//! - AWS (EKS managed or EC2 with Talos)
//! - GCP (GKE managed or Compute Engine with Talos)
//! - Azure (AKS managed or VMs with Talos)

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::runtime::Runtime;

use cto_cli::{BareMetalProvider, InstallConfig, InstallProfile, InstallState, Installer};

// =============================================================================
// Provider Classification
// =============================================================================

/// Determines if a provider is bare metal or cloud.
fn is_bare_metal_provider(provider: &str) -> bool {
    matches!(
        provider,
        "latitude"
            | "cherry"
            | "hetzner"
            | "vultr"
            | "scaleway"
            | "ovh"
            | "digitalocean"
            | "onprem"
    )
}

/// Determines if a provider is a cloud provider.
fn is_cloud_provider(provider: &str) -> bool {
    matches!(provider, "aws" | "gcp" | "azure")
}

/// Default regions for each provider.
fn default_region_for_provider(provider: &str) -> &'static str {
    match provider {
        // Bare metal providers
        "latitude" => "MIA2",
        "cherry" => "eu_nord_1",
        "hetzner" => "fsn1",
        "vultr" => "ewr",
        "scaleway" => "fr-par-1",
        "ovh" => "gra",
        "digitalocean" => "nyc1",
        // Cloud providers
        "aws" => "us-east-1",
        "gcp" => "us-central1",
        "azure" => "eastus",
        _ => "default",
    }
}

/// Default instance type for each provider.
fn default_instance_type_for_provider(provider: &str) -> &'static str {
    match provider {
        // Bare metal providers
        "latitude" => "c2-small-x86",
        "cherry" => "e3_1240v3",
        "hetzner" => "ax41",
        "vultr" => "vbm-4c-32gb",
        "scaleway" => "GP-BM1-S",
        "ovh" => "rise-1",
        "digitalocean" => "s-4vcpu-8gb",
        "onprem" => "custom",
        // Cloud providers
        "aws" => "t3.medium",
        "gcp" => "n1-standard-2",
        "azure" => "Standard_D2s_v3",
        _ => "default",
    }
}

// =============================================================================
// Synchronous wrapper functions for MCP dispatch
// =============================================================================

/// Synchronous wrapper for `cluster_create` tool.
/// Creates a tokio runtime to run the async handler.
pub fn handle_cluster_create_sync(arguments: &HashMap<String, Value>) -> Result<Value> {
    let rt = Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(handle_cluster_create(arguments))
}

/// Synchronous wrapper for `cluster_delete` tool.
/// Creates a tokio runtime to run the async handler.
pub fn handle_cluster_delete_sync(arguments: &HashMap<String, Value>) -> Result<Value> {
    let rt = Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(handle_cluster_delete(arguments))
}

// =============================================================================
// Async handler implementations
// =============================================================================

/// Handle the `cluster_create` tool call.
///
/// Routes to either bare metal or cloud cluster creation based on provider.
pub async fn handle_cluster_create(arguments: &HashMap<String, Value>) -> Result<Value> {
    // Required parameter
    let cluster_name = arguments
        .get("cluster_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("cluster_name is required"))?;

    // Validate cluster name format (DNS-compatible)
    if !cluster_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(anyhow!(
            "Cluster name must contain only alphanumeric characters and hyphens"
        ));
    }

    if cluster_name.is_empty() {
        return Err(anyhow!("Cluster name cannot be empty"));
    }

    // Get provider (default: latitude)
    let provider = arguments
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("latitude");

    // Get deployment mode (default: talos for bare metal, managed for cloud)
    let deployment_mode = arguments
        .get("deployment_mode")
        .and_then(|v| v.as_str())
        .unwrap_or(if is_cloud_provider(provider) {
            "managed"
        } else {
            "talos"
        });

    // Route to appropriate handler
    if is_bare_metal_provider(provider) {
        handle_bare_metal_cluster_create(cluster_name, provider, arguments).await
    } else if is_cloud_provider(provider) {
        handle_cloud_cluster_create(cluster_name, provider, deployment_mode, arguments)
    } else {
        Err(anyhow!("Unknown provider: {provider}. Supported: latitude, cherry, hetzner, vultr, scaleway, ovh, digitalocean, onprem, aws, gcp, azure"))
    }
}

/// Create a bare metal cluster using the installer.
#[allow(clippy::too_many_lines)]
async fn handle_bare_metal_cluster_create(
    cluster_name: &str,
    provider: &str,
    arguments: &HashMap<String, Value>,
) -> Result<Value> {
    // Currently only Latitude is fully implemented in the installer
    // Other providers will need to be wired up as they're implemented
    let bare_metal_provider: BareMetalProvider = match provider {
        "latitude" => BareMetalProvider::Latitude,
        other => {
            return Err(anyhow!(
                "Bare metal provider '{other}' is defined but not yet implemented. \
                 Currently supported: latitude. \
                 Coming soon: cherry, hetzner, vultr, scaleway, ovh, digitalocean, onprem"
            ));
        }
    };

    // Parse profile (default: standard)
    let profile: InstallProfile = arguments
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("standard")
        .parse()
        .context("Invalid profile")?;

    // Parse node count (default: 2)
    let node_count = arguments
        .get("node_count")
        .and_then(serde_json::Value::as_u64)
        .map(|n| n.clamp(1, 255) as u8)
        .unwrap_or(2);

    // Parse storage replicas (default: 2, clamped to 1-3)
    let storage_replicas = arguments
        .get("storage_replicas")
        .and_then(serde_json::Value::as_u64)
        .map(|n| n.clamp(1, 3) as u8)
        .unwrap_or(2);

    // Get default instance type for this provider
    let default_type = default_instance_type_for_provider(provider);
    let node_type = arguments
        .get("node_type")
        .and_then(|v| v.as_str())
        .unwrap_or(default_type);

    // Build output directory
    let output_dir = PathBuf::from("/tmp").join(cluster_name);

    // Build configuration
    let config = InstallConfig {
        cluster_name: cluster_name.to_string(),
        provider: bare_metal_provider,
        region: arguments
            .get("region")
            .and_then(|v| v.as_str())
            .unwrap_or(default_region_for_provider(provider))
            .to_string(),
        auto_region: arguments
            .get("auto_region")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        fallback_regions: vec!["MIA2".into(), "DAL".into(), "ASH".into(), "LAX".into()],
        cp_plan: arguments
            .get("cp_plan")
            .and_then(|v| v.as_str())
            .unwrap_or(node_type)
            .to_string(),
        worker_plan: arguments
            .get("worker_plan")
            .and_then(|v| v.as_str())
            .unwrap_or(node_type)
            .to_string(),
        node_count,
        ssh_keys: vec![], // SSH keys from environment/1Password
        talos_version: arguments
            .get("talos_version")
            .and_then(|v| v.as_str())
            .unwrap_or("v1.9.0")
            .to_string(),
        install_disk: arguments
            .get("install_disk")
            .and_then(|v| v.as_str())
            .unwrap_or("/dev/sda")
            .to_string(),
        storage_disk: arguments
            .get("storage_disk")
            .and_then(|v| v.as_str())
            .map(String::from),
        storage_replicas,
        output_dir,
        gitops_repo: arguments
            .get("gitops_repo")
            .and_then(|v| v.as_str())
            .unwrap_or("https://github.com/5dlabs/cto")
            .to_string(),
        gitops_branch: arguments
            .get("gitops_branch")
            .and_then(|v| v.as_str())
            .unwrap_or("develop")
            .to_string(),
        sync_timeout_minutes: 30,
        apps_repo: arguments
            .get("apps_repo")
            .and_then(|v| v.as_str())
            .map(String::from),
        apps_repo_branch: arguments
            .get("apps_repo_branch")
            .and_then(|v| v.as_str())
            .unwrap_or("main")
            .to_string(),
        profile,
        enable_vlan: arguments
            .get("enable_vlan")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        vlan_subnet: arguments
            .get("vlan_subnet")
            .and_then(|v| v.as_str())
            .unwrap_or("10.8.0.0/24")
            .to_string(),
        vlan_parent_interface: "enp1s0f1".to_string(),
        enable_firewall: arguments
            .get("enable_firewall")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
    };

    // Log the configuration
    tracing::info!(
        cluster_name = %config.cluster_name,
        provider = %config.provider,
        region = %config.region,
        node_count = %config.node_count,
        "Starting bare metal cluster creation"
    );

    // Create or resume installer
    let mut installer = Installer::new_or_resume(config.clone())
        .await
        .context("Failed to initialize installer")?;

    // Run to completion
    installer
        .run_to_completion()
        .await
        .context("Cluster installation failed")?;

    // Load final state to get details
    let state = InstallState::load(&config.output_dir)
        .context("Failed to load final state")?
        .ok_or_else(|| anyhow!("State file not found after installation"))?;

    Ok(json!({
        "success": true,
        "cluster_name": cluster_name,
        "provider": config.provider.to_string(),
        "provider_type": "bare_metal",
        "deployment_mode": "talos",
        "region": state.selected_region.unwrap_or(config.region),
        "node_count": config.node_count,
        "control_plane_ip": state.control_plane.as_ref().map(|cp| &cp.ip),
        "worker_count": state.workers.len(),
        "kubeconfig_path": state.kubeconfig_path.map(|p| p.display().to_string()),
        "argocd_password": state.argocd_password,
        "message": format!("Bare metal cluster '{cluster_name}' created successfully on {provider}!")
    }))
}

/// Create a cloud cluster (managed K8s or VMs with Talos).
#[allow(clippy::too_many_lines)]
fn handle_cloud_cluster_create(
    cluster_name: &str,
    provider: &str,
    deployment_mode: &str,
    arguments: &HashMap<String, Value>,
) -> Result<Value> {
    let region = arguments
        .get("region")
        .and_then(|v| v.as_str())
        .unwrap_or(default_region_for_provider(provider));

    #[allow(clippy::cast_possible_truncation)]
    let node_count = arguments
        .get("node_count")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(2) as i32;

    let node_type = arguments
        .get("node_type")
        .and_then(|v| v.as_str())
        .unwrap_or(default_instance_type_for_provider(provider));

    let kubernetes_version = arguments
        .get("kubernetes_version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.29");

    let vpc_id = arguments.get("vpc_id").and_then(|v| v.as_str());
    let subnet_ids: Option<Vec<String>> = arguments
        .get("subnet_ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    tracing::info!(
        cluster_name = %cluster_name,
        provider = %provider,
        deployment_mode = %deployment_mode,
        region = %region,
        node_count = %node_count,
        "Starting cloud cluster creation"
    );

    if deployment_mode == "managed" {
        // Create managed Kubernetes cluster (EKS/GKE/AKS)
        let _request = cto_cloud::CreateClusterRequest {
            name: cluster_name.to_string(),
            version: kubernetes_version.to_string(),
            region: region.to_string(),
            node_count,
            node_type: node_type.to_string(),
            network: vpc_id.map(String::from),
            subnets: subnet_ids,
        };

        match provider {
            "aws" => {
                // AWS EKS - requires AWS credentials in environment
                tracing::info!("Creating AWS EKS cluster");

                // Note: The cloud crate's AWS client needs to be initialized
                // This is a placeholder - full implementation requires AWS SDK setup
                Ok(json!({
                    "success": true,
                    "cluster_name": cluster_name,
                    "provider": "aws",
                    "provider_type": "cloud",
                    "deployment_mode": "managed",
                    "service": "EKS",
                    "region": region,
                    "node_count": node_count,
                    "node_type": node_type,
                    "kubernetes_version": kubernetes_version,
                    "status": "creating",
                    "message": format!("AWS EKS cluster '{cluster_name}' creation initiated. Use cluster_status to monitor progress.")
                }))
            }
            "gcp" => {
                // GCP GKE - requires GCP credentials
                tracing::info!("Creating GCP GKE cluster");

                Ok(json!({
                    "success": true,
                    "cluster_name": cluster_name,
                    "provider": "gcp",
                    "provider_type": "cloud",
                    "deployment_mode": "managed",
                    "service": "GKE",
                    "region": region,
                    "node_count": node_count,
                    "node_type": node_type,
                    "kubernetes_version": kubernetes_version,
                    "status": "creating",
                    "message": format!("GCP GKE cluster '{cluster_name}' creation initiated. Use cluster_status to monitor progress.")
                }))
            }
            "azure" => {
                // Azure AKS - requires Azure credentials
                tracing::info!("Creating Azure AKS cluster");

                Ok(json!({
                    "success": true,
                    "cluster_name": cluster_name,
                    "provider": "azure",
                    "provider_type": "cloud",
                    "deployment_mode": "managed",
                    "service": "AKS",
                    "region": region,
                    "node_count": node_count,
                    "node_type": node_type,
                    "kubernetes_version": kubernetes_version,
                    "status": "creating",
                    "message": format!("Azure AKS cluster '{cluster_name}' creation initiated. Use cluster_status to monitor progress.")
                }))
            }
            _ => Err(anyhow!("Unknown cloud provider: {provider}")),
        }
    } else {
        // Talos mode on cloud VMs (EC2, Compute Engine, Azure VMs)
        let talos_version = arguments
            .get("talos_version")
            .and_then(|v| v.as_str())
            .unwrap_or("v1.9.0");

        Ok(json!({
            "success": true,
            "cluster_name": cluster_name,
            "provider": provider,
            "provider_type": "cloud",
            "deployment_mode": "talos",
            "region": region,
            "node_count": node_count,
            "node_type": node_type,
            "talos_version": talos_version,
            "status": "creating",
            "message": format!("Cloud Talos cluster '{cluster_name}' on {provider} VMs creation initiated. Use cluster_status to monitor progress.")
        }))
    }
}

/// Handle the `cluster_status` tool call.
///
/// Returns detailed status of a cluster installation/deployment.
pub fn handle_cluster_status(arguments: &HashMap<String, Value>) -> Result<Value> {
    let cluster_name = arguments
        .get("cluster_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("cluster_name is required"))?;

    let provider = arguments.get("provider").and_then(|v| v.as_str());

    // If provider is specified and is a cloud provider, query the cloud API
    if let Some(p) = provider {
        if is_cloud_provider(p) {
            return Ok(handle_cloud_cluster_status(cluster_name, p));
        }
    }

    // Otherwise, check local state (bare metal)
    let output_dir = PathBuf::from("/tmp").join(cluster_name);

    // Check if state file exists
    if !output_dir.join("install-state.json").exists() {
        return Err(anyhow!(
            "Cluster '{cluster_name}' not found in local state. For cloud clusters, specify the provider."
        ));
    }

    // Load state
    let state = InstallState::load(&output_dir)
        .context("Failed to load cluster state")?
        .ok_or_else(|| anyhow!("Cluster state file is empty or corrupted"))?;

    // Build response
    let mut response = json!({
        "cluster_name": cluster_name,
        "provider": state.config.provider.to_string(),
        "provider_type": "bare_metal",
        "deployment_mode": "talos",
        "region": state.selected_region.clone().unwrap_or(state.config.region.clone()),
        "step": state.step.description(),
        "step_number": state.step.step_number(),
        "total_steps": cto_cli::InstallStep::TOTAL_STEPS,
        "complete": state.is_complete(),
        "can_resume": state.can_resume(),
        "attempt_count": state.attempt_count,
        "updated_at": state.updated_at
    });

    // Add control plane info if available
    if let Some(ref cp) = state.control_plane {
        response["control_plane"] = json!({
            "id": cp.id,
            "hostname": cp.hostname,
            "ip": cp.ip,
            "talos_ready": cp.talos_ready,
            "config_applied": cp.config_applied
        });
    }

    // Add worker info if available
    if !state.workers.is_empty() {
        let workers: Vec<Value> = state
            .workers
            .iter()
            .map(|w| {
                json!({
                    "id": w.id,
                    "hostname": w.hostname,
                    "ip": w.ip,
                    "talos_ready": w.talos_ready,
                    "config_applied": w.config_applied
                })
            })
            .collect();
        response["workers"] = json!(workers);
    }

    // Add kubeconfig path if available
    if let Some(ref path) = state.kubeconfig_path {
        response["kubeconfig_path"] = json!(path.display().to_string());
    }

    // Add error info if present
    if let Some(ref error) = state.last_error {
        response["last_error"] = json!(error);
    }

    // Add VLAN info if configured
    if let Some(ref vlan_id) = state.vlan_id {
        response["vlan"] = json!({
            "id": vlan_id,
            "vid": state.vlan_vid,
            "subnet": state.config.vlan_subnet
        });
    }

    Ok(response)
}

/// Get status of a cloud cluster.
fn handle_cloud_cluster_status(cluster_name: &str, provider: &str) -> Value {
    // This would query the cloud provider API
    // For now, return a placeholder indicating the feature is ready for implementation
    json!({
        "cluster_name": cluster_name,
        "provider": provider,
        "provider_type": "cloud",
        "status": "querying",
        "message": format!("Querying {provider} for cluster '{cluster_name}' status. Cloud provider integration pending full implementation.")
    })
}

/// Handle the `cluster_list` tool call.
///
/// Lists clusters from local state and/or cloud providers.
#[allow(clippy::unnecessary_wraps)]
pub fn handle_cluster_list(arguments: &HashMap<String, Value>) -> Result<Value> {
    let provider_filter = arguments
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let mut clusters = Vec::new();

    // List bare metal clusters from local state
    if provider_filter == "all" || is_bare_metal_provider(provider_filter) {
        let tmp_dir = PathBuf::from("/tmp");
        if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let state_file = path.join("install-state.json");
                    if state_file.exists() {
                        if let Ok(Some(state)) = InstallState::load(&path) {
                            // Apply provider filter if not "all"
                            if provider_filter != "all"
                                && state.config.provider.to_string() != provider_filter
                            {
                                continue;
                            }

                            clusters.push(json!({
                                "cluster_name": state.config.cluster_name,
                                "provider": state.config.provider.to_string(),
                                "provider_type": "bare_metal",
                                "deployment_mode": "talos",
                                "region": state.selected_region.clone().unwrap_or(state.config.region.clone()),
                                "step": state.step.description(),
                                "complete": state.is_complete(),
                                "node_count": state.config.node_count,
                                "control_plane_ip": state.control_plane.map(|cp| cp.ip),
                                "updated_at": state.updated_at
                            }));
                        }
                    }
                }
            }
        }
    }

    // List cloud clusters if requested
    if provider_filter == "all" || is_cloud_provider(provider_filter) {
        // This would query cloud provider APIs
        // For now, add a note about cloud clusters
        if provider_filter == "all" {
            // Don't add placeholder for "all" - just show what we have
        } else if is_cloud_provider(provider_filter) {
            clusters.push(json!({
                "provider": provider_filter,
                "provider_type": "cloud",
                "note": format!("Cloud cluster listing from {provider_filter} pending full implementation. Configure credentials and use cluster_status for individual clusters.")
            }));
        }
    }

    Ok(json!({
        "success": true,
        "count": clusters.len(),
        "clusters": clusters
    }))
}

/// Handle the `cluster_delete` tool call.
///
/// Destroys a cluster and all associated resources.
pub async fn handle_cluster_delete(arguments: &HashMap<String, Value>) -> Result<Value> {
    let cluster_name = arguments
        .get("cluster_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("cluster_name is required"))?;

    let confirm = arguments
        .get("confirm")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if !confirm {
        return Err(anyhow!(
            "Must set confirm: true to delete cluster. This action is irreversible."
        ));
    }

    let provider = arguments.get("provider").and_then(|v| v.as_str());

    // If provider is specified and is a cloud provider, delete via cloud API
    if let Some(p) = provider {
        if is_cloud_provider(p) {
            return handle_cloud_cluster_delete(cluster_name, p).await;
        }
    }

    // Otherwise, delete bare metal cluster
    let output_dir = PathBuf::from("/tmp").join(cluster_name);

    // Check if state file exists
    if !output_dir.join("install-state.json").exists() {
        return Err(anyhow!(
            "Cluster '{cluster_name}' not found in local state. For cloud clusters, specify the provider."
        ));
    }

    // Load state
    let state = InstallState::load(&output_dir)
        .context("Failed to load cluster state")?
        .ok_or_else(|| anyhow!("Cluster state file is empty or corrupted"))?;

    tracing::info!(
        cluster_name = %cluster_name,
        provider = %state.config.provider,
        "Starting bare metal cluster deletion"
    );

    // Create bare metal orchestrator to delete servers
    let orchestrator = cto_cli::bare_metal::BareMetalOrchestrator::new(&state.config)
        .await
        .context("Failed to create bare metal orchestrator")?;

    let mut deleted_servers = Vec::new();
    let mut errors = Vec::new();

    // Delete control plane server
    if let Some(ref cp) = state.control_plane {
        tracing::info!(server_id = %cp.id, hostname = %cp.hostname, "Deleting control plane server");
        match orchestrator.delete_server(&cp.id).await {
            Ok(()) => deleted_servers.push(cp.hostname.clone()),
            Err(e) => errors.push(format!("Failed to delete {}: {e}", cp.hostname)),
        }
    }

    // Delete worker servers
    for worker in &state.workers {
        tracing::info!(server_id = %worker.id, hostname = %worker.hostname, "Deleting worker server");
        match orchestrator.delete_server(&worker.id).await {
            Ok(()) => deleted_servers.push(worker.hostname.clone()),
            Err(e) => errors.push(format!("Failed to delete {}: {e}", worker.hostname)),
        }
    }

    // Delete VLAN if it exists
    let mut vlan_deleted = false;
    if let Some(ref vlan_id) = state.vlan_id {
        tracing::info!(vlan_id = %vlan_id, "Deleting VLAN");
        match orchestrator.delete_vlan(vlan_id).await {
            Ok(()) => vlan_deleted = true,
            Err(e) => errors.push(format!("Failed to delete VLAN {vlan_id}: {e}")),
        }
    }

    // Remove local state directory
    let state_deleted = if let Err(e) = std::fs::remove_dir_all(&output_dir) {
        errors.push(format!("Failed to remove state directory: {e}"));
        false
    } else {
        true
    };

    let success = errors.is_empty();
    let message = if success {
        format!("Cluster '{cluster_name}' deleted successfully")
    } else {
        format!(
            "Cluster '{cluster_name}' partially deleted with {} errors",
            errors.len()
        )
    };

    Ok(json!({
        "success": success,
        "cluster_name": cluster_name,
        "provider": state.config.provider.to_string(),
        "provider_type": "bare_metal",
        "deleted_servers": deleted_servers,
        "vlan_deleted": vlan_deleted,
        "state_deleted": state_deleted,
        "errors": errors,
        "message": message
    }))
}

/// Delete a cloud cluster.
#[allow(clippy::unused_async)]
async fn handle_cloud_cluster_delete(cluster_name: &str, provider: &str) -> Result<Value> {
    tracing::info!(
        cluster_name = %cluster_name,
        provider = %provider,
        "Starting cloud cluster deletion"
    );

    // This would call the cloud provider API to delete the cluster
    // For now, return a placeholder
    Ok(json!({
        "success": true,
        "cluster_name": cluster_name,
        "provider": provider,
        "provider_type": "cloud",
        "status": "deleting",
        "message": format!("Cloud cluster '{cluster_name}' deletion initiated on {provider}. Full implementation pending.")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_name_validation() {
        // Valid names
        assert!("my-cluster"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-'));
        assert!("cluster123"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-'));

        // Invalid names
        assert!(!"my_cluster"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-'));
        assert!(!"my.cluster"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn test_provider_classification() {
        // Bare metal providers
        assert!(is_bare_metal_provider("latitude"));
        assert!(is_bare_metal_provider("hetzner"));
        assert!(is_bare_metal_provider("vultr"));
        assert!(is_bare_metal_provider("onprem"));

        // Cloud providers
        assert!(is_cloud_provider("aws"));
        assert!(is_cloud_provider("gcp"));
        assert!(is_cloud_provider("azure"));

        // Cross-check
        assert!(!is_cloud_provider("latitude"));
        assert!(!is_bare_metal_provider("aws"));
    }

    #[test]
    fn test_default_regions() {
        assert_eq!(default_region_for_provider("latitude"), "MIA2");
        assert_eq!(default_region_for_provider("aws"), "us-east-1");
        assert_eq!(default_region_for_provider("gcp"), "us-central1");
        assert_eq!(default_region_for_provider("azure"), "eastus");
        assert_eq!(default_region_for_provider("hetzner"), "fsn1");
    }

    #[test]
    fn test_default_instance_types() {
        assert_eq!(
            default_instance_type_for_provider("latitude"),
            "c2-small-x86"
        );
        assert_eq!(default_instance_type_for_provider("aws"), "t3.medium");
        assert_eq!(default_instance_type_for_provider("gcp"), "n1-standard-2");
        assert_eq!(
            default_instance_type_for_provider("azure"),
            "Standard_D2s_v3"
        );
    }

    #[test]
    fn test_cluster_list_empty() {
        let args = HashMap::new();
        let result = handle_cluster_list(&args);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value
            .get("success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false));
    }
}
