//! Cilium CNI installation and configuration utilities.
//!
//! This module provides functions for installing Cilium as the CNI
//! and configuring it for `ClusterMesh` multi-cluster networking.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use tracing::info;

/// Cilium configuration for a cluster.
#[derive(Debug, Clone)]
pub struct CiliumConfig {
    /// Unique cluster name (used in `ClusterMesh`).
    pub cluster_name: String,
    /// Unique cluster ID (1-255) for `ClusterMesh`.
    pub cluster_id: u8,
    /// Pod CIDR for this cluster (must not overlap with other clusters).
    pub pod_cidr: String,
    /// Enable `WireGuard` encryption for pod-to-pod traffic.
    pub enable_wireguard: bool,
    /// Enable Hubble for network observability.
    pub enable_hubble: bool,
    /// Cilium version to install (e.g., "1.16.4").
    pub version: String,
}

impl Default for CiliumConfig {
    fn default() -> Self {
        Self {
            cluster_name: "default".to_string(),
            cluster_id: 1,
            pod_cidr: "10.0.0.0/16".to_string(),
            enable_wireguard: true,
            enable_hubble: true,
            version: "1.16.4".to_string(),
        }
    }
}

impl CiliumConfig {
    /// Create a new Cilium configuration for a cluster.
    #[must_use]
    pub fn new(cluster_name: impl Into<String>, cluster_id: u8) -> Self {
        Self {
            cluster_name: cluster_name.into(),
            cluster_id,
            ..Default::default()
        }
    }

    /// Set the Pod CIDR for `ClusterMesh` compatibility.
    #[must_use]
    pub fn with_pod_cidr(mut self, cidr: impl Into<String>) -> Self {
        self.pod_cidr = cidr.into();
        self
    }

    /// Enable or disable `WireGuard` encryption.
    #[must_use]
    pub fn with_wireguard(mut self, enabled: bool) -> Self {
        self.enable_wireguard = enabled;
        self
    }

    /// Enable or disable Hubble observability.
    #[must_use]
    pub fn with_hubble(mut self, enabled: bool) -> Self {
        self.enable_hubble = enabled;
        self
    }

    /// Set the Cilium version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

/// Run a helm command with the given kubeconfig.
fn helm(kubeconfig: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("helm")
        .env("KUBECONFIG", kubeconfig)
        .args(args)
        .output()
        .context("Failed to execute helm")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("helm failed: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check if Cilium CLI is installed.
///
/// # Errors
///
/// Returns an error if cilium CLI is not installed.
pub fn check_cilium_cli() -> Result<()> {
    let output = Command::new("cilium")
        .arg("version")
        .output()
        .context("Failed to run cilium CLI - is it installed?")?;

    if !output.status.success() {
        bail!("cilium CLI is not working properly");
    }

    Ok(())
}

/// Install Cilium using Helm.
///
/// This installs Cilium with the provided configuration, including
/// kube-proxy replacement and optional features like `WireGuard` and Hubble.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn install_cilium(kubeconfig: &Path, config: &CiliumConfig) -> Result<()> {
    info!(
        "Installing Cilium {} for cluster '{}' (ID: {})...",
        config.version, config.cluster_name, config.cluster_id
    );

    // Add Cilium Helm repo
    let _ = helm(
        kubeconfig,
        &["repo", "add", "cilium", "https://helm.cilium.io/"],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Build helm install arguments
    let mut args = vec![
        "upgrade",
        "--install",
        "cilium",
        "cilium/cilium",
        "--namespace",
        "kube-system",
        "--version",
        &config.version,
        "--set",
        "kubeProxyReplacement=true",
        "--set",
        "k8sServiceHost=localhost",
        "--set",
        "k8sServicePort=7445", // KubePrism port
    ];

    // Cluster identity for ClusterMesh
    let cluster_name_arg = format!("cluster.name={}", config.cluster_name);
    let cluster_id_arg = format!("cluster.id={}", config.cluster_id);
    args.extend_from_slice(&["--set", &cluster_name_arg, "--set", &cluster_id_arg]);

    // WireGuard encryption
    if config.enable_wireguard {
        args.extend_from_slice(&[
            "--set",
            "encryption.enabled=true",
            "--set",
            "encryption.type=wireguard",
        ]);
    }

    // Hubble observability
    if config.enable_hubble {
        args.extend_from_slice(&[
            "--set",
            "hubble.enabled=true",
            "--set",
            "hubble.relay.enabled=true",
            "--set",
            "hubble.ui.enabled=true",
        ]);
    }

    args.push("--wait");

    helm(kubeconfig, &args)?;

    info!(
        "Cilium installed successfully for cluster '{}'",
        config.cluster_name
    );
    Ok(())
}

/// Get Cilium status using the CLI.
///
/// # Errors
///
/// Returns an error if the cilium CLI fails.
pub fn get_cilium_status(kubeconfig: &Path) -> Result<String> {
    let output = Command::new("cilium")
        .env("KUBECONFIG", kubeconfig)
        .arg("status")
        .output()
        .context("Failed to run cilium status")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cilium status failed: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Wait for Cilium CNI to be functional (nodes can become Ready).
///
/// This is a lighter-weight check than `cilium status --wait`, which requires
/// ALL Cilium components (including hubble, operator replicas) to be fully healthy.
/// For single-node clusters or during initial bootstrap, we just need the
/// cilium agent daemonset to be running so nodes can become Ready.
///
/// # Errors
///
/// Returns an error if Cilium does not become functional within the timeout.
pub fn wait_for_cilium_healthy(kubeconfig: &Path) -> Result<()> {
    use std::time::{Duration, Instant};

    info!("Waiting for Cilium to be healthy...");

    let start = Instant::now();
    let timeout = Duration::from_secs(300); // 5 minutes

    loop {
        if start.elapsed() > timeout {
            bail!("Timeout waiting for Cilium to become functional");
        }

        // Check if cilium daemonset has at least one pod ready
        let output = Command::new("kubectl")
        .env("KUBECONFIG", kubeconfig)
            .args([
                "get",
                "daemonset",
                "cilium",
                "-n",
                "kube-system",
                "-o",
                "jsonpath={.status.numberReady}",
            ])
        .output()
            .context("Failed to check cilium daemonset status")?;

        if output.status.success() {
            let ready_count = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u32>()
                .unwrap_or(0);

            if ready_count > 0 {
                info!("Cilium CNI is functional ({} agent(s) ready)", ready_count);
                return Ok(());
    }
        }

        std::thread::sleep(Duration::from_secs(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cilium_config_defaults() {
        let config = CiliumConfig::default();
        assert_eq!(config.cluster_name, "default");
        assert_eq!(config.cluster_id, 1);
        assert!(config.enable_wireguard);
        assert!(config.enable_hubble);
    }

    #[test]
    fn test_cilium_config_builder() {
        let config = CiliumConfig::new("dal-cluster", 1)
            .with_pod_cidr("10.1.0.0/16")
            .with_wireguard(true)
            .with_hubble(true)
            .with_version("1.16.4");

        assert_eq!(config.cluster_name, "dal-cluster");
        assert_eq!(config.cluster_id, 1);
        assert_eq!(config.pod_cidr, "10.1.0.0/16");
        assert!(config.enable_wireguard);
        assert!(config.enable_hubble);
        assert_eq!(config.version, "1.16.4");
    }

    #[test]
    fn test_cilium_config_cluster_ids() {
        // Valid cluster IDs for ClusterMesh are 1-255
        let config1 = CiliumConfig::new("cluster-1", 1);
        let config2 = CiliumConfig::new("cluster-2", 255);

        assert_eq!(config1.cluster_id, 1);
        assert_eq!(config2.cluster_id, 255);
    }
}
