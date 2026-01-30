//! Kind (Kubernetes in Docker) cluster management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

const CLUSTER_NAME: &str = "cto-lite";

/// Kind cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KindInfo {
    pub installed: bool,
    pub version: Option<String>,
}

/// Kind cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfo {
    pub exists: bool,
    pub name: String,
    pub running: bool,
    pub nodes: Vec<NodeInfo>,
}

/// Kubernetes node information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub role: String,
}

/// Check if Kind is installed
pub fn check_kind() -> Result<KindInfo> {
    let kind_path = which::which("kind").ok();
    
    if kind_path.is_none() {
        return Ok(KindInfo {
            installed: false,
            version: None,
        });
    }

    let version_output = Command::new("kind")
        .args(["version"])
        .output()
        .context("Failed to run kind version")?;

    let version = if version_output.status.success() {
        let output = String::from_utf8_lossy(&version_output.stdout);
        // Parse "kind v0.20.0 go1.21.0 darwin/arm64"
        output.split_whitespace().nth(1).map(|s| s.to_string())
    } else {
        None
    };

    Ok(KindInfo {
        installed: true,
        version,
    })
}

/// Get cluster status
pub fn get_cluster_status() -> Result<ClusterInfo> {
    // Check if cluster exists
    let clusters_output = Command::new("kind")
        .args(["get", "clusters"])
        .output()
        .context("Failed to list kind clusters")?;

    let clusters = String::from_utf8_lossy(&clusters_output.stdout);
    let exists = clusters.lines().any(|l| l.trim() == CLUSTER_NAME);

    if !exists {
        return Ok(ClusterInfo {
            exists: false,
            name: CLUSTER_NAME.to_string(),
            running: false,
            nodes: vec![],
        });
    }

    // Get node status
    let nodes_output = Command::new("kubectl")
        .args([
            "--context", &format!("kind-{}", CLUSTER_NAME),
            "get", "nodes",
            "-o", "jsonpath={range .items[*]}{.metadata.name},{.status.conditions[?(@.type=='Ready')].status},{.metadata.labels['node-role\\.kubernetes\\.io/control-plane']}{\"\\n\"}{end}"
        ])
        .output();

    let nodes = if let Ok(output) = nodes_output {
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| {
                    let parts: Vec<&str> = l.split(',').collect();
                    NodeInfo {
                        name: parts.first().unwrap_or(&"").to_string(),
                        status: if parts.get(1) == Some(&"True") { "Ready" } else { "NotReady" }.to_string(),
                        role: if parts.get(2).is_some() { "control-plane" } else { "worker" }.to_string(),
                    }
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let running = nodes.iter().any(|n| n.status == "Ready");

    Ok(ClusterInfo {
        exists: true,
        name: CLUSTER_NAME.to_string(),
        running,
        nodes,
    })
}

/// Create a new Kind cluster
pub fn create_cluster() -> Result<()> {
    tracing::info!("Creating Kind cluster: {}", CLUSTER_NAME);

    // Kind config for CTO Lite
    let config = r#"
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: cto-lite
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
        hostPort: 8080
        protocol: TCP
      - containerPort: 443
        hostPort: 8443
        protocol: TCP
      - containerPort: 30000
        hostPort: 30000
        protocol: TCP
"#;

    // Write config to temp file
    let config_path = std::env::temp_dir().join("cto-lite-kind-config.yaml");
    std::fs::write(&config_path, config).context("Failed to write Kind config")?;

    // Create cluster
    let output = Command::new("kind")
        .args(["create", "cluster", "--config", config_path.to_str().unwrap()])
        .output()
        .context("Failed to create Kind cluster")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to create cluster: {}", stderr));
    }

    tracing::info!("Kind cluster created successfully");
    Ok(())
}

/// Delete the Kind cluster
pub fn delete_cluster() -> Result<()> {
    tracing::info!("Deleting Kind cluster: {}", CLUSTER_NAME);

    let output = Command::new("kind")
        .args(["delete", "cluster", "--name", CLUSTER_NAME])
        .output()
        .context("Failed to delete Kind cluster")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to delete cluster: {}", stderr));
    }

    tracing::info!("Kind cluster deleted successfully");
    Ok(())
}

/// List all Kind clusters
pub fn list_clusters() -> Result<Vec<String>> {
    let output = Command::new("kind")
        .args(["get", "clusters"])
        .output()
        .context("Failed to list Kind clusters")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let clusters = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(clusters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_kind() {
        let info = check_kind().unwrap();
        println!("Kind info: {:?}", info);
    }

    #[test]
    #[ignore] // Requires Docker running
    fn test_cluster_operations() {
        // Get initial status
        let status = get_cluster_status().unwrap();
        println!("Initial status: {:?}", status);
    }
}
