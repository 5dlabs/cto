//! Kind cluster management commands

use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::error::{AppError, AppResult};

const CLUSTER_NAME: &str = "cto-lite";

/// Cluster status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub name: String,
    pub exists: bool,
    pub running: bool,
    pub nodes: Vec<NodeStatus>,
    pub kubeconfig_path: Option<String>,
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub name: String,
    pub role: String,
    pub status: String,
}

/// Run a kind command and return stdout
fn run_kind(args: &[&str]) -> AppResult<String> {
    let output = Command::new("kind")
        .args(args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kind: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::ClusterError(stderr.to_string()))
    }
}

/// Run kubectl command
fn run_kubectl(args: &[&str]) -> AppResult<String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::ClusterError(stderr.to_string()))
    }
}

/// Check if kind is installed
fn is_kind_installed() -> bool {
    which::which("kind").is_ok()
}

/// Check if kubectl is installed
fn is_kubectl_installed() -> bool {
    which::which("kubectl").is_ok()
}

/// Get list of kind clusters
fn list_kind_clusters() -> AppResult<Vec<String>> {
    let output = run_kind(&["get", "clusters"])?;
    Ok(output.lines().map(String::from).filter(|s| !s.is_empty()).collect())
}

/// Check if our cluster exists
fn cluster_exists() -> bool {
    list_kind_clusters()
        .map(|clusters| clusters.contains(&CLUSTER_NAME.to_string()))
        .unwrap_or(false)
}

/// Get node status from kubectl
fn get_node_status() -> AppResult<Vec<NodeStatus>> {
    let output = run_kubectl(&[
        "get", "nodes",
        "--context", &format!("kind-{}", CLUSTER_NAME),
        "-o", "custom-columns=NAME:.metadata.name,ROLE:.metadata.labels.node-role\\.kubernetes\\.io/control-plane,STATUS:.status.conditions[-1].type",
        "--no-headers",
    ])?;

    let nodes = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            NodeStatus {
                name: parts.first().unwrap_or(&"").to_string(),
                role: if parts.get(1).map(|s| !s.is_empty()).unwrap_or(false) {
                    "control-plane".to_string()
                } else {
                    "worker".to_string()
                },
                status: parts.get(2).unwrap_or(&"Unknown").to_string(),
            }
        })
        .collect();

    Ok(nodes)
}

/// Create the CTO Lite Kind cluster
#[tauri::command]
pub async fn create_cluster() -> Result<ClusterStatus, AppError> {
    // Check prerequisites
    if !is_kind_installed() {
        return Err(AppError::CommandFailed(
            "kind is not installed. Please install it first.".to_string()
        ));
    }
    
    if !is_kubectl_installed() {
        return Err(AppError::CommandFailed(
            "kubectl is not installed. Please install it first.".to_string()
        ));
    }

    // Check if cluster already exists
    if cluster_exists() {
        tracing::info!("Cluster {} already exists", CLUSTER_NAME);
        return get_cluster_status().await;
    }

    tracing::info!("Creating Kind cluster: {}", CLUSTER_NAME);

    // Create cluster with a configuration that supports ingress
    let config = r#"
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
  - containerPort: 8080
    hostPort: 8080
    protocol: TCP
"#;

    // Write config to temp file
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("cto-lite-kind-config.yaml");
    std::fs::write(&config_path, config)?;

    // Create cluster
    let output = Command::new("kind")
        .args([
            "create", "cluster",
            "--name", CLUSTER_NAME,
            "--config", config_path.to_str().unwrap(),
            "--wait", "300s",
        ])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to create cluster: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&config_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::ClusterError(format!("Failed to create cluster: {}", stderr)));
    }

    tracing::info!("Cluster {} created successfully", CLUSTER_NAME);
    
    get_cluster_status().await
}

/// Delete the CTO Lite Kind cluster
#[tauri::command]
pub async fn delete_cluster() -> Result<(), AppError> {
    if !cluster_exists() {
        tracing::info!("Cluster {} does not exist", CLUSTER_NAME);
        return Ok(());
    }

    tracing::info!("Deleting Kind cluster: {}", CLUSTER_NAME);

    run_kind(&["delete", "cluster", "--name", CLUSTER_NAME])?;

    tracing::info!("Cluster {} deleted", CLUSTER_NAME);
    Ok(())
}

/// Get the status of the CTO Lite cluster
#[tauri::command]
pub async fn get_cluster_status() -> Result<ClusterStatus, AppError> {
    let exists = cluster_exists();
    
    if !exists {
        return Ok(ClusterStatus {
            name: CLUSTER_NAME.to_string(),
            exists: false,
            running: false,
            nodes: vec![],
            kubeconfig_path: None,
        });
    }

    // Get kubeconfig path
    let kubeconfig = dirs::home_dir()
        .map(|h: std::path::PathBuf| h.join(".kube").join("config"))
        .and_then(|p: std::path::PathBuf| p.to_str().map(String::from));

    // Try to get node status - if this fails, cluster exists but isn't running
    let (running, nodes) = match get_node_status() {
        Ok(nodes) => {
            let all_ready = nodes.iter().all(|n| n.status == "Ready");
            (all_ready, nodes)
        }
        Err(_) => (false, vec![]),
    };

    Ok(ClusterStatus {
        name: CLUSTER_NAME.to_string(),
        exists,
        running,
        nodes,
        kubeconfig_path: kubeconfig,
    })
}

/// List all Kind clusters
#[tauri::command]
pub async fn list_clusters() -> Result<Vec<String>, AppError> {
    list_kind_clusters()
}
