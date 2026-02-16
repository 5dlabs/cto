// Kind cluster management commands
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize, Clone)]
pub struct ClusterStatus {
    pub name: String,
    pub status: String,
    pub nodes: usize,
    pub kubernetes_version: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ClusterInfo {
    pub name: String,
    pub status: String,
    pub nodes: Vec<NodeInfo>,
    pub kubernetes_version: Option<String>,
}

fn check_command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("which {} 2>/dev/null", cmd))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn start_kind_cluster(name: &str) -> Result<String, String> {
    if !check_command_exists("kind") {
        return Err("kind is not installed. Run install_kind() first.".to_string());
    }

    // Check if cluster already exists
    let check_output = Command::new("kind")
        .arg("get")
        .arg("clusters")
        .output()
        .map_err(|e| format!("Failed to check clusters: {}", e))?;

    let clusters = String::from_utf8_lossy(&check_output.stdout);
    if clusters.lines().any(|l| l.trim() == name) {
        return Ok(format!("Cluster '{}' already exists", name));
    }

    // Create the cluster
    let output = Command::new("kind")
        .arg("create")
        .arg("cluster")
        .arg("--name")
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to create cluster: {}", e))?;

    if output.status.success() {
        Ok(format!("Cluster '{}' created successfully", name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn delete_kind_cluster(name: &str) -> Result<String, String> {
    if !check_command_exists("kind") {
        return Err("kind is not installed".to_string());
    }

    let output = Command::new("kind")
        .arg("delete")
        .arg("cluster")
        .arg("--name")
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to delete cluster: {}", e))?;

    if output.status.success() {
        Ok(format!("Cluster '{}' deleted successfully", name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn get_cluster_status(name: &str) -> Result<ClusterStatus, String> {
    if !check_command_exists("kind") {
        return Err("kind is not installed".to_string());
    }

    // Get cluster list to check if it exists
    let list_output = Command::new("kind")
        .arg("get")
        .arg("clusters")
        .output()
        .map_err(|e| format!("Failed to get clusters: {}", e))?;

    let clusters = String::from_utf8_lossy(&list_output.stdout);
    let cluster_exists = clusters.lines().any(|l| l.trim() == name);

    if !cluster_exists {
        return Err(format!("Cluster '{}' not found", name));
    }

    // Get cluster nodes
    let nodes_output = Command::new("kind")
        .arg("get")
        .arg("nodes")
        .arg("--name")
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to get nodes: {}", e))?;

    let nodes_str = String::from_utf8_lossy(&nodes_output.stdout);
    let nodes: Vec<&str> = nodes_str.lines().filter(|l| !l.trim().is_empty()).collect();

    Ok(ClusterStatus {
        name: name.to_string(),
        status: "Running".to_string(),
        nodes: nodes.len(),
        kubernetes_version: None,
    })
}

#[tauri::command]
pub fn list_clusters() -> Result<Vec<ClusterStatus>, String> {
    if !check_command_exists("kind") {
        return Err("kind is not installed".to_string());
    }

    let output = Command::new("kind")
        .arg("get")
        .arg("clusters")
        .output()
        .map_err(|e| format!("Failed to get clusters: {}", e))?;

    let clusters_str = String::from_utf8_lossy(&output.stdout);
    let mut statuses: Vec<ClusterStatus> = Vec::new();

    for line in clusters_str.lines() {
        let name = line.trim();
        if !name.is_empty() {
            // Get node count for each cluster
            let nodes_output = Command::new("kind")
                .arg("get")
                .arg("nodes")
                .arg("--name")
                .arg(name)
                .output()
                .ok();

            let node_count = nodes_output
                .and_then(|o| {
                    Some(
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .filter(|l| !l.trim().is_empty())
                            .count(),
                    )
                })
                .unwrap_or(0);

            statuses.push(ClusterStatus {
                name: name.to_string(),
                status: "Running".to_string(),
                nodes: node_count,
                kubernetes_version: None,
            });
        }
    }

    Ok(statuses)
}

#[tauri::command]
pub fn start_cluster(name: &str) -> Result<String, String> {
    start_kind_cluster(name)
}

#[tauri::command]
pub fn stop_cluster(_name: &str) -> Result<String, String> {
    Err("Stopping a kind cluster is not supported (delete it instead)".to_string())
}

#[tauri::command]
pub fn restart_cluster(_name: &str) -> Result<String, String> {
    Err("Restarting a kind cluster is not supported (delete and recreate)".to_string())
}

#[tauri::command]
pub fn delete_cluster(name: &str) -> Result<String, String> {
    delete_kind_cluster(name)
}

#[tauri::command]
pub fn get_cluster_info(name: &str) -> Result<ClusterInfo, String> {
    let status = get_cluster_status(name)?;

    Ok(ClusterInfo {
        name: status.name,
        status: status.status,
        nodes: (0..status.nodes)
            .map(|i| NodeInfo {
                name: format!("node-{i}"),
                status: "ready".to_string(),
            })
            .collect(),
        kubernetes_version: status.kubernetes_version,
    })
}

#[tauri::command]
pub fn get_clusters_status() -> Result<Vec<ClusterStatus>, String> {
    list_clusters()
}
