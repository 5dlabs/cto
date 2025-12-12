//! Cilium `ClusterMesh` utilities for multi-cluster connectivity.
//!
//! This module provides functions for setting up and managing Cilium
//! `ClusterMesh` connections between Kubernetes clusters.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use tracing::info;

/// `ClusterMesh` connection status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterMeshStatus {
    /// `ClusterMesh` is not enabled.
    Disabled,
    /// `ClusterMesh` is enabled but not connected to any clusters.
    Enabled,
    /// `ClusterMesh` is connected to remote clusters.
    Connected { cluster_count: usize },
}

/// Enable `ClusterMesh` on a cluster.
///
/// This deploys the clustermesh-apiserver and enables the cluster
/// to participate in `ClusterMesh` connections.
///
/// # Errors
///
/// Returns an error if the cilium CLI fails.
pub fn enable_clustermesh(kubeconfig: &Path) -> Result<()> {
    info!("Enabling ClusterMesh...");

    let output = Command::new("cilium")
        .env("KUBECONFIG", kubeconfig)
        .args(["clustermesh", "enable"])
        .output()
        .context("Failed to run cilium clustermesh enable")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if already enabled
        if stderr.contains("already enabled") {
            info!("ClusterMesh is already enabled");
            return Ok(());
        }
        bail!("Failed to enable ClusterMesh: {stderr}");
    }

    info!("ClusterMesh enabled successfully");
    Ok(())
}

/// Connect two clusters via `ClusterMesh`.
///
/// This creates a bidirectional connection between two clusters,
/// allowing pods to communicate across cluster boundaries.
///
/// # Arguments
///
/// * `source_kubeconfig` - Path to the kubeconfig for the source cluster
/// * `destination_kubeconfig` - Path to the kubeconfig for the destination cluster
///
/// # Errors
///
/// Returns an error if the connection fails.
pub fn connect_clusters(source_kubeconfig: &Path, destination_kubeconfig: &Path) -> Result<()> {
    info!("Connecting clusters via ClusterMesh...");

    let output = Command::new("cilium")
        .env("KUBECONFIG", source_kubeconfig)
        .args(["clustermesh", "connect", "--destination-kubeconfig"])
        .arg(destination_kubeconfig)
        .output()
        .context("Failed to run cilium clustermesh connect")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if already connected
        if stderr.contains("already connected") {
            info!("Clusters are already connected");
            return Ok(());
        }
        bail!("Failed to connect clusters: {stderr}");
    }

    info!("Clusters connected successfully");
    Ok(())
}

/// Connect clusters using kubectl context names.
///
/// This is useful when you have multiple clusters configured in your
/// kubeconfig and want to connect them by context name.
///
/// # Errors
///
/// Returns an error if the connection fails.
pub fn connect_clusters_by_context(source_context: &str, destination_context: &str) -> Result<()> {
    info!(
        "Connecting clusters via ClusterMesh: {} -> {}",
        source_context, destination_context
    );

    let output = Command::new("cilium")
        .args([
            "clustermesh",
            "connect",
            "--context",
            source_context,
            "--destination-context",
            destination_context,
        ])
        .output()
        .context("Failed to run cilium clustermesh connect")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already connected") {
            info!("Clusters are already connected");
            return Ok(());
        }
        bail!("Failed to connect clusters: {stderr}");
    }

    info!("Clusters connected successfully");
    Ok(())
}

/// Get `ClusterMesh` status for a cluster.
///
/// # Errors
///
/// Returns an error if the cilium CLI fails.
pub fn get_clustermesh_status(kubeconfig: &Path) -> Result<String> {
    let output = Command::new("cilium")
        .env("KUBECONFIG", kubeconfig)
        .args(["clustermesh", "status"])
        .output()
        .context("Failed to run cilium clustermesh status")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get ClusterMesh status: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Wait for `ClusterMesh` to be ready.
///
/// # Errors
///
/// Returns an error if `ClusterMesh` does not become ready within the timeout.
pub fn wait_for_clustermesh_ready(kubeconfig: &Path) -> Result<()> {
    info!("Waiting for ClusterMesh to be ready...");

    let output = Command::new("cilium")
        .env("KUBECONFIG", kubeconfig)
        .args(["clustermesh", "status", "--wait"])
        .output()
        .context("Failed to run cilium clustermesh status --wait")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ClusterMesh is not ready: {stderr}");
    }

    info!("ClusterMesh is ready");
    Ok(())
}

/// Disconnect a cluster from `ClusterMesh`.
///
/// # Errors
///
/// Returns an error if the disconnection fails.
pub fn disconnect_cluster(source_kubeconfig: &Path, destination_cluster_name: &str) -> Result<()> {
    info!(
        "Disconnecting cluster '{}' from ClusterMesh...",
        destination_cluster_name
    );

    let output = Command::new("cilium")
        .env("KUBECONFIG", source_kubeconfig)
        .args(["clustermesh", "disconnect", destination_cluster_name])
        .output()
        .context("Failed to run cilium clustermesh disconnect")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to disconnect cluster: {stderr}");
    }

    info!("Cluster disconnected successfully");
    Ok(())
}

/// Disable `ClusterMesh` on a cluster.
///
/// # Errors
///
/// Returns an error if disabling fails.
pub fn disable_clustermesh(kubeconfig: &Path) -> Result<()> {
    info!("Disabling ClusterMesh...");

    let output = Command::new("cilium")
        .env("KUBECONFIG", kubeconfig)
        .args(["clustermesh", "disable"])
        .output()
        .context("Failed to run cilium clustermesh disable")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to disable ClusterMesh: {stderr}");
    }

    info!("ClusterMesh disabled successfully");
    Ok(())
}

/// Set up a full `ClusterMesh` between multiple clusters.
///
/// This helper function enables `ClusterMesh` on all provided clusters
/// and connects them in a full mesh topology.
///
/// # Arguments
///
/// * `kubeconfigs` - Paths to kubeconfigs for all clusters to mesh
///
/// # Errors
///
/// Returns an error if any step fails.
pub fn setup_full_mesh(kubeconfigs: &[&Path]) -> Result<()> {
    if kubeconfigs.len() < 2 {
        bail!("At least 2 clusters are required for ClusterMesh");
    }

    info!(
        "Setting up full ClusterMesh with {} clusters...",
        kubeconfigs.len()
    );

    // Enable ClusterMesh on all clusters
    for kubeconfig in kubeconfigs {
        enable_clustermesh(kubeconfig)?;
    }

    // Wait for all to be ready
    for kubeconfig in kubeconfigs {
        wait_for_clustermesh_ready(kubeconfig)?;
    }

    // Connect all clusters to each other (full mesh)
    for (i, source) in kubeconfigs.iter().enumerate() {
        for destination in kubeconfigs.iter().skip(i + 1) {
            connect_clusters(source, destination)?;
        }
    }

    info!("Full ClusterMesh setup complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clustermesh_status_enum() {
        let disabled = ClusterMeshStatus::Disabled;
        let enabled = ClusterMeshStatus::Enabled;
        let connected = ClusterMeshStatus::Connected { cluster_count: 2 };

        assert_eq!(disabled, ClusterMeshStatus::Disabled);
        assert_eq!(enabled, ClusterMeshStatus::Enabled);
        assert_eq!(connected, ClusterMeshStatus::Connected { cluster_count: 2 });
    }
}
