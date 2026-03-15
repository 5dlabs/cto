//! Installation configuration types.
//!
//! This module defines the configuration types for bare metal cluster installation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ============================================================================
// Cluster Size Configuration (Tier 2 Managed)
// ============================================================================

/// Cluster size options for Tier 2 Managed provisioning.
///
/// These map to predefined configurations for node counts and instance plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClusterSize {
    /// Small: 2 nodes (1 CP + 1 worker), c2-small-x86
    /// Best for: Development, testing, POCs
    #[default]
    Small,

    /// Medium: 4 nodes (1 CP + 3 workers), c2-medium-x86
    /// Best for: Small teams, staging environments
    Medium,

    /// Large: 8 nodes (3 CP HA + 5 workers), c2-large-x86
    /// Best for: Production workloads requiring HA
    Large,
}

impl ClusterSize {
    /// Get the number of control plane nodes for this size.
    #[must_use]
    pub const fn control_plane_count(self) -> u8 {
        match self {
            Self::Small | Self::Medium => 1,
            Self::Large => 3, // HA configuration
        }
    }

    /// Get the number of worker nodes for this size.
    #[must_use]
    pub const fn worker_count(self) -> u8 {
        match self {
            Self::Small => 1,
            Self::Medium => 3,
            Self::Large => 5,
        }
    }

    /// Get the total node count for this size.
    #[must_use]
    pub const fn total_nodes(self) -> u8 {
        self.control_plane_count() + self.worker_count()
    }

    /// Get the recommended instance plan for control plane nodes.
    #[must_use]
    pub const fn cp_plan(self) -> &'static str {
        match self {
            Self::Small => "c2-small-x86",
            Self::Medium => "c2-medium-x86",
            Self::Large => "c2-large-x86",
        }
    }

    /// Get the recommended instance plan for worker nodes.
    #[must_use]
    pub const fn worker_plan(self) -> &'static str {
        match self {
            Self::Small => "c2-small-x86",
            Self::Medium => "c2-medium-x86",
            Self::Large => "c2-large-x86",
        }
    }

    /// Get the install disk path for this size's instance plan.
    ///
    /// Based on Latitude.sh disk types:
    /// - c2-small-x86: SATA SSD → /dev/sda
    /// - c2-medium-x86: NVMe SSD → /dev/nvme0n1
    /// - c2-large-x86: NVMe SSD → /dev/nvme0n1
    #[must_use]
    pub const fn install_disk(self) -> &'static str {
        match self {
            Self::Small => "/dev/sda",
            Self::Medium | Self::Large => "/dev/nvme0n1",
        }
    }

    /// Get the recommended storage replica count for this size.
    #[must_use]
    pub const fn storage_replicas(self) -> u8 {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
        }
    }

    /// Whether this size uses HA control plane (3 CPs).
    #[must_use]
    pub const fn is_ha(self) -> bool {
        matches!(self, Self::Large)
    }

    /// Get estimated monthly cost (approximate, varies by provider/region).
    #[must_use]
    #[allow(dead_code)]
    pub const fn estimated_monthly_cost_usd(self) -> u32 {
        match self {
            Self::Small => 300,  // ~$150/node × 2 nodes
            Self::Medium => 800, // ~$200/node × 4 nodes
            Self::Large => 2400, // ~$300/node × 8 nodes
        }
    }
}

impl std::fmt::Display for ClusterSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Small => write!(f, "small"),
            Self::Medium => write!(f, "medium"),
            Self::Large => write!(f, "large"),
        }
    }
}

impl std::str::FromStr for ClusterSize {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "small" | "s" => Ok(Self::Small),
            "medium" | "m" => Ok(Self::Medium),
            "large" | "l" => Ok(Self::Large),
            _ => Err(anyhow::anyhow!(
                "Unknown cluster size: {s}. Supported: small (s), medium (m), large (l)"
            )),
        }
    }
}

// ============================================================================
// Bare Metal Provider
// ============================================================================

/// Bare metal provider for cluster provisioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BareMetalProvider {
    /// Latitude.sh bare metal provider.
    #[default]
    Latitude,
    /// Hetzner dedicated servers.
    Hetzner,
    /// OVHcloud dedicated servers.
    Ovh,
    /// Vultr bare metal.
    Vultr,
    /// Scaleway Elastic Metal.
    Scaleway,
    /// Cherry Servers.
    Cherry,
    /// On-premises / colocation with IPMI.
    OnPrem,
}

impl std::fmt::Display for BareMetalProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latitude => write!(f, "latitude"),
            Self::Hetzner => write!(f, "hetzner"),
            Self::Ovh => write!(f, "ovh"),
            Self::Vultr => write!(f, "vultr"),
            Self::Scaleway => write!(f, "scaleway"),
            Self::Cherry => write!(f, "cherry"),
            Self::OnPrem => write!(f, "onprem"),
        }
    }
}

impl std::str::FromStr for BareMetalProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "latitude" => Ok(Self::Latitude),
            "hetzner" => Ok(Self::Hetzner),
            "ovh" => Ok(Self::Ovh),
            "vultr" => Ok(Self::Vultr),
            "scaleway" => Ok(Self::Scaleway),
            "cherry" => Ok(Self::Cherry),
            "onprem" | "on-prem" | "on_prem" => Ok(Self::OnPrem),
            _ => Err(anyhow::anyhow!(
                "Unknown provider: {s}. Supported: latitude, hetzner, ovh, vultr, scaleway, cherry, onprem"
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
    ///
    /// ## LESSON LEARNED (Dec 2024)
    /// Different Latitude.sh server plans have different disk types:
    /// - **c1-tiny-x86**: SATA SSD → `/dev/sda`
    /// - **c2-small-x86**: SATA SSD → `/dev/sda`
    /// - **c2-medium-x86**: 2x NVMe SSD → `/dev/nvme0n1`
    /// - **c2-large-x86**: 2x NVMe SSD → `/dev/nvme0n1`
    /// - **c3-***: NVMe SSD → `/dev/nvme0n1`
    /// - **m4-***: NVMe SSD → `/dev/nvme0n1`
    ///
    /// Use `detect_install_disk()` to auto-detect based on plan type.
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
    /// Parent NIC for VLAN interface (e.g., "eno2", "eth1", "enp1s0f1").
    /// Latitude servers use "eno2" as the secondary NIC for VLAN.
    /// This may vary by server model and provider.
    pub vlan_parent_interface: String,
    /// Primary NIC for public IP (DHCP). Auto-detected in maintenance mode when VLAN is enabled.
    pub primary_interface: Option<String>,
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
            gitops_repo: "https://git.5dlabs.ai/5dlabs/cto.git".into(),
            gitops_branch: "develop".into(),
            sync_timeout_minutes: 30,
            profile: InstallProfile::default(),
            enable_vlan: true, // Recommended for bare metal
            vlan_subnet: "10.8.0.0/24".into(),
            vlan_parent_interface: "eno2".into(), // Secondary NIC on Latitude servers
            primary_interface: None,
            enable_firewall: true, // Recommended for security
        }
    }

    /// Create config from a `ClusterSize` preset (for Tier 2 Managed provisioning).
    ///
    /// This applies the recommended instance plans, node counts, and disk paths
    /// based on the selected cluster size.
    #[must_use]
    #[allow(dead_code)]
    pub fn from_cluster_size(cluster_name: String, size: ClusterSize, region: String) -> Self {
        let output_dir = PathBuf::from("/tmp").join(format!("tier2-{cluster_name}"));
        Self {
            cluster_name,
            provider: BareMetalProvider::default(),
            region,
            auto_region: false,
            fallback_regions: vec!["DAL".into(), "MIA2".into(), "ASH".into(), "LAX".into()],
            cp_plan: size.cp_plan().into(),
            worker_plan: size.worker_plan().into(),
            node_count: size.total_nodes(),
            ssh_keys: vec![],
            talos_version: "v1.9.0".into(),
            install_disk: size.install_disk().into(),
            storage_disk: None,
            storage_replicas: size.storage_replicas(),
            output_dir,
            gitops_repo: "https://git.5dlabs.ai/5dlabs/cto.git".into(),
            gitops_branch: "develop".into(),
            sync_timeout_minutes: if size.is_ha() { 45 } else { 30 }, // HA takes longer
            profile: if size.is_ha() {
                InstallProfile::Production
            } else {
                InstallProfile::Standard
            },
            enable_vlan: true,
            vlan_subnet: "10.8.0.0/24".into(),
            vlan_parent_interface: "eno2".into(),
            primary_interface: None,
            enable_firewall: true,
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

    /// Detect the correct install disk based on server plan type.
    ///
    /// ## LESSON LEARNED (Dec 2024)
    /// Latitude.sh server plans use different disk types:
    /// - c1-*/c2-small-x86: SATA SSD → /dev/sda
    /// - c2-medium-x86, c2-large-x86, c3-*, m4-*, s3-*: NVMe → /dev/nvme0n1
    ///
    /// Using the wrong disk path causes Talos installation to fail silently
    /// or the server to become unreachable after config apply.
    #[must_use]
    #[allow(dead_code)] // Utility function for future use
    pub fn detect_install_disk(plan: &str) -> String {
        // Plans that use NVMe storage
        let nvme_plans = [
            "c2-medium",
            "c2-large",
            "c3-",
            "m4-",
            "s2-",
            "s3-",
            "f4-",
            "rs4-",
            "g3-",
        ];

        for prefix in nvme_plans {
            if plan.starts_with(prefix) {
                return "/dev/nvme0n1".into();
            }
        }

        // Default to SATA for c1-tiny, c2-small, and unknown plans
        "/dev/sda".into()
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

    // ClusterSize tests
    #[test]
    fn test_cluster_size_small() {
        let size = ClusterSize::Small;
        assert_eq!(size.control_plane_count(), 1);
        assert_eq!(size.worker_count(), 1);
        assert_eq!(size.total_nodes(), 2);
        assert_eq!(size.cp_plan(), "c2-small-x86");
        assert_eq!(size.worker_plan(), "c2-small-x86");
        assert_eq!(size.install_disk(), "/dev/sda");
        assert_eq!(size.storage_replicas(), 1);
        assert!(!size.is_ha());
    }

    #[test]
    fn test_cluster_size_medium() {
        let size = ClusterSize::Medium;
        assert_eq!(size.control_plane_count(), 1);
        assert_eq!(size.worker_count(), 3);
        assert_eq!(size.total_nodes(), 4);
        assert_eq!(size.cp_plan(), "c2-medium-x86");
        assert_eq!(size.worker_plan(), "c2-medium-x86");
        assert_eq!(size.install_disk(), "/dev/nvme0n1");
        assert_eq!(size.storage_replicas(), 2);
        assert!(!size.is_ha());
    }

    #[test]
    fn test_cluster_size_large() {
        let size = ClusterSize::Large;
        assert_eq!(size.control_plane_count(), 3);
        assert_eq!(size.worker_count(), 5);
        assert_eq!(size.total_nodes(), 8);
        assert_eq!(size.cp_plan(), "c2-large-x86");
        assert_eq!(size.worker_plan(), "c2-large-x86");
        assert_eq!(size.install_disk(), "/dev/nvme0n1");
        assert_eq!(size.storage_replicas(), 3);
        assert!(size.is_ha());
    }

    #[test]
    fn test_cluster_size_parsing() {
        // Full names
        assert_eq!("small".parse::<ClusterSize>().unwrap(), ClusterSize::Small);
        assert_eq!(
            "medium".parse::<ClusterSize>().unwrap(),
            ClusterSize::Medium
        );
        assert_eq!("large".parse::<ClusterSize>().unwrap(), ClusterSize::Large);

        // Short names
        assert_eq!("s".parse::<ClusterSize>().unwrap(), ClusterSize::Small);
        assert_eq!("m".parse::<ClusterSize>().unwrap(), ClusterSize::Medium);
        assert_eq!("l".parse::<ClusterSize>().unwrap(), ClusterSize::Large);

        // Case insensitive
        assert_eq!("SMALL".parse::<ClusterSize>().unwrap(), ClusterSize::Small);
        assert_eq!(
            "Medium".parse::<ClusterSize>().unwrap(),
            ClusterSize::Medium
        );

        // Invalid
        assert!("xlarge".parse::<ClusterSize>().is_err());
    }

    #[test]
    fn test_cluster_size_display() {
        assert_eq!(format!("{}", ClusterSize::Small), "small");
        assert_eq!(format!("{}", ClusterSize::Medium), "medium");
        assert_eq!(format!("{}", ClusterSize::Large), "large");
    }

    #[test]
    fn test_config_from_cluster_size() {
        let config =
            InstallConfig::from_cluster_size("acme".into(), ClusterSize::Medium, "DAL".into());

        assert_eq!(config.cluster_name, "acme");
        assert_eq!(config.region, "DAL");
        assert_eq!(config.node_count, 4);
        assert_eq!(config.cp_plan, "c2-medium-x86");
        assert_eq!(config.worker_plan, "c2-medium-x86");
        assert_eq!(config.install_disk, "/dev/nvme0n1");
        assert_eq!(config.storage_replicas, 2);
        assert_eq!(config.profile, InstallProfile::Standard);
        assert_eq!(config.output_dir, PathBuf::from("/tmp/tier2-acme"));
    }

    #[test]
    fn test_config_from_cluster_size_large_ha() {
        let config =
            InstallConfig::from_cluster_size("bigcorp".into(), ClusterSize::Large, "FRA".into());

        assert_eq!(config.node_count, 8);
        assert_eq!(config.cp_plan, "c2-large-x86");
        assert_eq!(config.storage_replicas, 3);
        assert_eq!(config.profile, InstallProfile::Production);
        assert_eq!(config.sync_timeout_minutes, 45); // Longer for HA
    }
}
