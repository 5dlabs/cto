//! Installation configuration types.
//!
//! This module defines the configuration types for bare metal cluster installation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Bare metal provider (Latitude for now, extensible).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BareMetalProvider {
    /// Latitude.sh bare metal provider.
    #[default]
    Latitude,
    // Future: Cherry, Hetzner, DigitalOcean, etc.
}

impl std::fmt::Display for BareMetalProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latitude => write!(f, "latitude"),
        }
    }
}

impl std::str::FromStr for BareMetalProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "latitude" => Ok(Self::Latitude),
            _ => Err(anyhow::anyhow!(
                "Unknown provider: {s}. Supported: latitude"
            )),
        }
    }
}

/// Installation profile for resource sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum InstallProfile {
    /// Standard deployment (most use cases).
    #[default]
    Standard,
    /// Production with HA considerations.
    Production,
}

impl std::fmt::Display for InstallProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Production => write!(f, "production"),
        }
    }
}

impl std::str::FromStr for InstallProfile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            "production" => Ok(Self::Production),
            _ => Err(anyhow::anyhow!(
                "Unknown profile: {s}. Supported: standard, production"
            )),
        }
    }
}

/// Full installation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    // Cluster identity
    /// Name of the cluster being provisioned.
    pub cluster_name: String,
    /// Bare metal provider to use.
    pub provider: BareMetalProvider,

    // Infrastructure
    /// Region/site for server provisioning (e.g., "MIA2", "DAL").
    pub region: String,
    /// Whether to auto-select region based on stock availability.
    pub auto_region: bool,
    /// Fallback regions to try if auto_region is enabled.
    pub fallback_regions: Vec<String>,
    /// Server plan for control plane node.
    pub cp_plan: String,
    /// Server plan for worker nodes.
    pub worker_plan: String,
    /// Total node count (1 control plane + N-1 workers).
    pub node_count: u8,
    /// SSH key IDs for initial Ubuntu boot.
    pub ssh_keys: Vec<String>,

    // Talos
    /// Talos Linux version (e.g., "v1.9.0").
    pub talos_version: String,
    /// Install disk path (e.g., "/dev/sda", "/dev/nvme0n1").
    pub install_disk: String,

    // Storage
    /// NVMe disk for Mayastor storage (e.g., "/dev/nvme0n1").
    /// If not specified, defaults to install_disk (not recommended for production).
    pub storage_disk: Option<String>,
    /// Number of Mayastor replicas (1-3). Default: 2 for 2+ nodes, 1 for single node.
    pub storage_replicas: u8,

    // Paths
    /// Output directory for generated configs and state.
    pub output_dir: PathBuf,

    // GitOps
    /// GitOps repository URL.
    pub gitops_repo: String,
    /// GitOps branch to deploy from.
    pub gitops_branch: String,
    /// Timeout in minutes for GitOps sync.
    pub sync_timeout_minutes: u32,

    // Profile
    /// Installation profile for resource sizing.
    pub profile: InstallProfile,

    // Networking
    /// Enable VLAN private networking for node-to-node communication.
    /// When enabled, creates a Latitude VLAN and configures Talos with a
    /// VLAN sub-interface for private cluster traffic (etcd, kubelet, Cilium VXLAN).
    /// Public IPs are still used for ingress traffic.
    pub enable_vlan: bool,
    /// Private network subnet for VLAN (e.g., "10.8.0.0/24").
    /// Node private IPs are allocated from this subnet.
    pub vlan_subnet: String,
    /// Parent NIC for VLAN interface (e.g., "eth1", "eno2").
    /// This is the internal/PXE NIC on Latitude servers.
    pub vlan_parent_interface: String,
    /// Enable Talos Ingress Firewall for host-level traffic control.
    /// Blocks all ingress by default, allowing only necessary cluster traffic.
    pub enable_firewall: bool,
}

impl InstallConfig {
    /// Create config with sensible defaults for a given cluster name.
    #[must_use]
    #[allow(dead_code)]
    pub fn with_defaults(cluster_name: String) -> Self {
        let output_dir = PathBuf::from("/tmp").join(&cluster_name);
        Self {
            cluster_name,
            provider: BareMetalProvider::default(),
            region: "MIA2".into(),
            auto_region: false,
            fallback_regions: vec!["MIA2".into(), "DAL".into(), "ASH".into(), "LAX".into()],
            cp_plan: "c2-small-x86".into(),
            worker_plan: "c2-small-x86".into(),
            node_count: 2,
            ssh_keys: vec![],
            talos_version: "v1.9.0".into(),
            install_disk: "/dev/sda".into(),
            storage_disk: None, // Use separate NVMe if available
            storage_replicas: 2,
            output_dir,
            gitops_repo: "https://github.com/5dlabs/cto".into(),
            gitops_branch: "develop".into(),
            sync_timeout_minutes: 30,
            profile: InstallProfile::default(),
            enable_vlan: true, // Recommended for bare metal
            vlan_subnet: "10.8.0.0/24".into(),
            vlan_parent_interface: "eth1".into(), // Secondary NIC on Latitude
            enable_firewall: true,                // Recommended for security
        }
    }

    /// Get the control plane hostname.
    #[must_use]
    pub fn cp_hostname(&self) -> String {
        format!("{}-cp1", self.cluster_name)
    }

    /// Get worker hostnames.
    #[must_use]
    pub fn worker_hostnames(&self) -> Vec<String> {
        (1..self.node_count)
            .map(|i| format!("{}-worker{i}", self.cluster_name))
            .collect()
    }

    /// Get the state file path.
    #[must_use]
    #[allow(dead_code)]
    pub fn state_file(&self) -> PathBuf {
        self.output_dir.join("install-state.json")
    }

    /// Get the kubeconfig path.
    #[must_use]
    pub fn kubeconfig_path(&self) -> PathBuf {
        self.output_dir.join("kubeconfig")
    }

    /// Get the talosconfig path.
    #[must_use]
    pub fn talosconfig_path(&self) -> PathBuf {
        self.output_dir.join("talosconfig")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = InstallConfig::with_defaults("test-cluster".into());
        assert_eq!(config.cluster_name, "test-cluster");
        assert_eq!(config.provider, BareMetalProvider::Latitude);
        assert_eq!(config.region, "MIA2");
        assert_eq!(config.node_count, 2);
        assert_eq!(config.output_dir, PathBuf::from("/tmp/test-cluster"));
    }

    #[test]
    fn test_hostnames() {
        let config = InstallConfig::with_defaults("prod".into());
        assert_eq!(config.cp_hostname(), "prod-cp1");

        let workers = config.worker_hostnames();
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0], "prod-worker1");
    }

    #[test]
    fn test_provider_parsing() {
        assert_eq!(
            "latitude".parse::<BareMetalProvider>().unwrap(),
            BareMetalProvider::Latitude
        );
        assert!("unknown".parse::<BareMetalProvider>().is_err());
    }

    #[test]
    fn test_profile_parsing() {
        assert_eq!(
            "standard".parse::<InstallProfile>().unwrap(),
            InstallProfile::Standard
        );
        assert_eq!(
            "production".parse::<InstallProfile>().unwrap(),
            InstallProfile::Production
        );
        assert!("invalid".parse::<InstallProfile>().is_err());
    }
}
