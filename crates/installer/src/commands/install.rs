//! Install command implementation.
//!
//! Provides the CLI interface for bare metal cluster installation.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::{BareMetalProvider, InstallConfig, InstallProfile};
use crate::orchestrator::Installer;
use crate::ui;
use crate::validator::PrerequisitesValidator;

/// Install command arguments.
#[derive(Args, Debug)]
pub struct InstallCommand {
    /// Bare metal provider (currently only "latitude" supported).
    #[arg(long, default_value = "latitude")]
    provider: String,

    /// Cluster name (required).
    #[arg(long)]
    cluster_name: String,

    /// Region/site for server provisioning (e.g., "DAL", "MIA2").
    /// If --auto-region is set, this is ignored.
    /// LESSON LEARNED: DAL has excellent value with c2-large-x86 (20 cores, 128GB @ $0.39/hr)
    #[arg(long, default_value = "DAL")]
    region: String,

    /// Automatically select the best available region based on stock.
    #[arg(long)]
    auto_region: bool,

    /// Fallback regions to try if --auto-region is set (comma-separated).
    /// LESSON LEARNED: DAL first for best value, then MIA2 for c1-tiny availability.
    #[arg(long, value_delimiter = ',', default_value = "DAL,MIA2,LAX,ASH")]
    fallback_regions: Vec<String>,

    /// Control plane server plan.
    /// For production, c2-small-x86 (4 cores, 32GB) is sufficient for control plane.
    #[arg(long, default_value = "c2-small-x86")]
    cp_plan: String,

    /// Worker server plan.
    /// LESSON LEARNED: c2-large-x86 in DAL is excellent value (20 cores, 128GB @ $0.39/hr).
    /// Use smaller plans like c1-tiny-x86 or c2-small-x86 for dev clusters.
    #[arg(long, default_value = "c2-large-x86")]
    worker_plan: String,

    /// Total node count (1 control plane + N-1 workers).
    #[arg(long, default_value = "2")]
    nodes: u8,

    /// SSH key IDs for initial boot (comma-separated).
    #[arg(long, value_delimiter = ',')]
    ssh_keys: Vec<String>,

    /// Talos Linux version.
    #[arg(long, default_value = "v1.9.0")]
    talos_version: String,

    /// Install disk path (e.g., "/dev/sda", "/dev/nvme0n1").
    #[arg(long, default_value = "/dev/sda")]
    install_disk: String,

    /// Storage disk for Mayastor (e.g., "/dev/nvme0n1"). Defaults to install_disk.
    #[arg(long)]
    storage_disk: Option<String>,

    /// Number of Mayastor replicas (1-3). Defaults to min(2, node_count).
    #[arg(long, default_value = "2")]
    storage_replicas: u8,

    /// Output directory for configs and state (defaults to /tmp/{cluster-name}).
    #[arg(long)]
    output_dir: Option<PathBuf>,

    /// GitOps sync timeout in minutes.
    #[arg(long, default_value = "30")]
    sync_timeout: u32,

    /// Installation profile (standard or production).
    #[arg(long, default_value = "standard")]
    profile: String,

    /// GitOps repository URL.
    #[arg(long, default_value = "https://github.com/5dlabs/cto")]
    gitops_repo: String,

    /// GitOps branch to deploy from.
    #[arg(long, default_value = "develop")]
    gitops_branch: String,

    /// Enable VLAN private networking for node-to-node communication.
    /// Creates a Latitude VLAN and configures Talos with private IPs.
    #[arg(long, default_value_t = true)]
    enable_vlan: bool,

    /// Private network subnet for VLAN (e.g., "10.8.0.0/24").
    #[arg(long, default_value = "10.8.0.0/24")]
    vlan_subnet: String,

    /// Parent NIC for VLAN interface (secondary NIC on Latitude servers).
    /// Latitude c2/c3 servers use "enp1s0f1" as the secondary NIC.
    #[arg(long, default_value = "enp1s0f1")]
    vlan_interface: String,

    /// Enable Talos Ingress Firewall for host-level security.
    #[arg(long, default_value_t = true)]
    enable_firewall: bool,
}

impl InstallCommand {
    /// Run the install command.
    ///
    /// # Errors
    ///
    /// Returns an error if installation fails.
    pub async fn run(&self) -> Result<()> {
        ui::print_banner();
        ui::print_section("CTO Platform Installation");

        // Validate prerequisites first
        ui::print_step("Checking prerequisites...");
        let validator = PrerequisitesValidator::new();
        validator.validate().context("Prerequisites check failed")?;
        ui::print_success("All prerequisites met");

        // Build config from args
        let config = self.build_config()?;
        ui::print_info(&format!("Cluster: {}", config.cluster_name));
        ui::print_info(&format!("Provider: {}", config.provider));
        ui::print_info(&format!("Region: {}", config.region));
        ui::print_info(&format!("Nodes: {}", config.node_count));
        ui::print_info(&format!("Output: {}", config.output_dir.display()));

        // Create or resume installer
        let mut installer = Installer::new_or_resume(config).await?;

        // Run to completion (handles all retry/resume internally)
        installer.run_to_completion().await
    }

    /// Build installation config from CLI arguments.
    fn build_config(&self) -> Result<InstallConfig> {
        let provider: BareMetalProvider = self
            .provider
            .parse()
            .context("Invalid provider specified")?;

        let profile: InstallProfile = self.profile.parse().context("Invalid profile specified")?;

        let output_dir = self
            .output_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("/tmp").join(&self.cluster_name));

        if self.nodes < 1 {
            anyhow::bail!("Node count must be at least 1");
        }

        if self.cluster_name.is_empty() {
            anyhow::bail!("Cluster name is required");
        }

        // Validate cluster name format (DNS-compatible)
        if !self
            .cluster_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            anyhow::bail!("Cluster name must contain only alphanumeric characters and hyphens");
        }

        Ok(InstallConfig {
            cluster_name: self.cluster_name.clone(),
            provider,
            region: self.region.clone(),
            auto_region: self.auto_region,
            fallback_regions: self.fallback_regions.clone(),
            cp_plan: self.cp_plan.clone(),
            worker_plan: self.worker_plan.clone(),
            node_count: self.nodes,
            ssh_keys: self.ssh_keys.clone(),
            talos_version: self.talos_version.clone(),
            install_disk: self.install_disk.clone(),
            storage_disk: self.storage_disk.clone(),
            storage_replicas: self.storage_replicas.clamp(1, 3),
            output_dir,
            gitops_repo: self.gitops_repo.clone(),
            gitops_branch: self.gitops_branch.clone(),
            sync_timeout_minutes: self.sync_timeout,
            profile,
            enable_vlan: self.enable_vlan,
            vlan_subnet: self.vlan_subnet.clone(),
            vlan_parent_interface: self.vlan_interface.clone(),
            enable_firewall: self.enable_firewall,
        })
    }
}
