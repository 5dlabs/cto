//! Installation state persistence.
//!
//! This module provides state tracking for cluster installation operations,
//! allowing automatic recovery from failures and resumption of interrupted processes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::InstallConfig;

/// Extended provisioning steps (includes infrastructure + GitOps).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStep {
    // Initial
    /// Not yet started.
    NotStarted,
    /// Validating prerequisites.
    ValidatingPrerequisites,

    // Infrastructure (server provisioning)
    /// Creating bare metal servers.
    CreatingServers,
    /// Creating VLAN for private networking.
    CreatingVLAN,
    /// Waiting for servers to be ready.
    WaitingServersReady,
    /// Triggering Talos iPXE boot.
    BootingTalos,
    /// Waiting for Talos maintenance mode.
    WaitingTalosMaintenance,

    // Talos bootstrap
    /// Generating Talos secrets and configs.
    GeneratingConfigs,
    /// Applying config to control plane.
    ApplyingCPConfig,
    /// Waiting for control plane installation.
    WaitingCPInstall,
    /// Bootstrapping etcd and Kubernetes.
    Bootstrapping,
    /// Deploying Cilium CNI (required for nodes to be Ready).
    DeployingCilium,
    /// Waiting for Kubernetes API and nodes to be ready.
    WaitingKubernetes,
    /// Applying config to worker nodes.
    ApplyingWorkerConfig,
    /// Waiting for workers to join cluster.
    WaitingWorkerJoin,

    // Platform stack
    /// Deploying pre-ArgoCD bootstrap resources.
    DeployingBootstrapResources,
    /// Deploying local-path-provisioner.
    DeployingLocalPathProvisioner,
    /// Deploying ArgoCD.
    DeployingArgoCD,
    /// Waiting for ArgoCD to be ready.
    WaitingArgoCDReady,
    /// Applying app-of-apps manifest.
    ApplyingAppOfApps,
    /// Waiting for all GitOps applications to sync.
    WaitingGitOpsSync,

    // Post-GitOps configuration
    /// Configuring Mayastor storage (DiskPools + StorageClass).
    ConfiguringStorage,
    /// Bootstrapping OpenBao (init, unseal, seed secrets from 1Password).
    BootstrappingOpenBao,
    /// Configuring local kubeconfig for kubectl and Lens access.
    ConfiguringKubeconfig,

    // Terminal states
    /// Installation complete.
    Complete,
}

impl InstallStep {
    /// Get the next step in the sequence.
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::NotStarted => Self::ValidatingPrerequisites,
            Self::ValidatingPrerequisites => Self::CreatingServers,
            Self::CreatingServers => Self::CreatingVLAN,
            Self::CreatingVLAN => Self::WaitingServersReady,
            Self::WaitingServersReady => Self::BootingTalos,
            Self::BootingTalos => Self::WaitingTalosMaintenance,
            Self::WaitingTalosMaintenance => Self::GeneratingConfigs,
            Self::GeneratingConfigs => Self::ApplyingCPConfig,
            Self::ApplyingCPConfig => Self::WaitingCPInstall,
            Self::WaitingCPInstall => Self::Bootstrapping,
            Self::Bootstrapping => Self::DeployingCilium,
            Self::DeployingCilium => Self::WaitingKubernetes,
            Self::WaitingKubernetes => Self::ApplyingWorkerConfig,
            Self::ApplyingWorkerConfig => Self::WaitingWorkerJoin,
            Self::WaitingWorkerJoin => Self::DeployingBootstrapResources,
            Self::DeployingBootstrapResources => Self::DeployingLocalPathProvisioner,
            Self::DeployingLocalPathProvisioner => Self::DeployingArgoCD,
            Self::DeployingArgoCD => Self::WaitingArgoCDReady,
            Self::WaitingArgoCDReady => Self::ApplyingAppOfApps,
            Self::ApplyingAppOfApps => Self::WaitingGitOpsSync,
            Self::WaitingGitOpsSync => Self::ConfiguringStorage,
            Self::ConfiguringStorage => Self::BootstrappingOpenBao,
            Self::BootstrappingOpenBao => Self::ConfiguringKubeconfig,
            Self::ConfiguringKubeconfig | Self::Complete => Self::Complete,
        }
    }

    /// Get a human-readable description of the step.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::NotStarted => "Not started",
            Self::ValidatingPrerequisites => "Validating prerequisites",
            Self::CreatingServers => "Creating bare metal servers",
            Self::CreatingVLAN => "Creating VLAN for private networking",
            Self::WaitingServersReady => "Waiting for servers to be ready",
            Self::BootingTalos => "Triggering Talos iPXE boot",
            Self::WaitingTalosMaintenance => "Waiting for Talos maintenance mode",
            Self::GeneratingConfigs => "Generating Talos configs",
            Self::ApplyingCPConfig => "Applying control plane config",
            Self::WaitingCPInstall => "Waiting for control plane installation",
            Self::Bootstrapping => "Bootstrapping Kubernetes cluster",
            Self::DeployingCilium => "Deploying Cilium CNI",
            Self::WaitingKubernetes => "Waiting for nodes to be Ready",
            Self::ApplyingWorkerConfig => "Applying worker node configs",
            Self::WaitingWorkerJoin => "Waiting for workers to join",
            Self::DeployingBootstrapResources => "Deploying bootstrap resources",
            Self::DeployingLocalPathProvisioner => "Deploying local-path-provisioner",
            Self::DeployingArgoCD => "Deploying ArgoCD",
            Self::WaitingArgoCDReady => "Waiting for ArgoCD to be ready",
            Self::ApplyingAppOfApps => "Applying app-of-apps manifest",
            Self::WaitingGitOpsSync => "Waiting for GitOps sync",
            Self::ConfiguringStorage => "Configuring Mayastor storage",
            Self::BootstrappingOpenBao => "Bootstrapping OpenBao secrets",
            Self::ConfiguringKubeconfig => "Configuring kubeconfig for Lens",
            Self::Complete => "Complete",
        }
    }

    /// Get the step number for progress display.
    #[must_use]
    pub fn step_number(&self) -> u8 {
        match self {
            Self::NotStarted => 0,
            Self::ValidatingPrerequisites => 1,
            Self::CreatingServers => 2,
            Self::CreatingVLAN => 3,
            Self::WaitingServersReady => 4,
            Self::BootingTalos => 5,
            Self::WaitingTalosMaintenance => 6,
            Self::GeneratingConfigs => 7,
            Self::ApplyingCPConfig => 8,
            Self::WaitingCPInstall => 9,
            Self::Bootstrapping => 10,
            Self::DeployingCilium => 11,
            Self::WaitingKubernetes => 12,
            Self::ApplyingWorkerConfig => 13,
            Self::WaitingWorkerJoin => 14,
            Self::DeployingBootstrapResources => 15,
            Self::DeployingLocalPathProvisioner => 16,
            Self::DeployingArgoCD => 17,
            Self::WaitingArgoCDReady => 18,
            Self::ApplyingAppOfApps => 19,
            Self::WaitingGitOpsSync => 20,
            Self::ConfiguringStorage => 21,
            Self::BootstrappingOpenBao => 22,
            Self::ConfiguringKubeconfig => 23,
            Self::Complete => 24,
        }
    }

    /// Total number of steps.
    pub const TOTAL_STEPS: u8 = 24;
}

impl std::fmt::Display for InstallStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Server state during provisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    /// Server ID from provider.
    pub id: String,
    /// Server IP address.
    pub ip: String,
    /// Server hostname.
    pub hostname: String,
    /// Whether Talos is ready on this server.
    pub talos_ready: bool,
    /// Whether config has been applied.
    pub config_applied: bool,
}

/// Full installation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallState {
    /// Installation configuration.
    pub config: InstallConfig,
    /// Current installation step.
    pub step: InstallStep,
    /// Selected region (may differ from config if auto-selected).
    pub selected_region: Option<String>,
    /// Control plane server state.
    pub control_plane: Option<ServerState>,
    /// Worker server states.
    pub workers: Vec<ServerState>,
    /// Path to kubeconfig (once available).
    pub kubeconfig_path: Option<PathBuf>,
    /// ArgoCD admin password (once retrieved).
    pub argocd_password: Option<String>,
    /// Timestamp of last state update.
    pub updated_at: String,
    /// Number of retry attempts for current step.
    pub attempt_count: u32,
    /// Last error message (if any).
    pub last_error: Option<String>,

    // VLAN state
    /// VLAN resource ID from Latitude (e.g., `vlan_xxx`).
    #[serde(default)]
    pub vlan_id: Option<String>,
    /// VLAN ID (VID) for OS configuration (e.g., 2063).
    #[serde(default)]
    pub vlan_vid: Option<u16>,
    /// Private IP addresses for each server (server_id -> private_ip).
    #[serde(default)]
    pub private_ips: HashMap<String, String>,
}

impl InstallState {
    /// Create a new installation state.
    #[must_use]
    pub fn new(config: InstallConfig) -> Self {
        Self {
            config,
            step: InstallStep::NotStarted,
            selected_region: None,
            control_plane: None,
            workers: Vec::new(),
            kubeconfig_path: None,
            argocd_password: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
            attempt_count: 0,
            last_error: None,
            vlan_id: None,
            vlan_vid: None,
            private_ips: HashMap::new(),
        }
    }

    /// Get the state file path for a given output directory.
    #[must_use]
    pub fn state_file(output_dir: &Path) -> PathBuf {
        output_dir.join("install-state.json")
    }

    /// Load state from file if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load(output_dir: &Path) -> Result<Option<Self>> {
        let path = Self::state_file(output_dir);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path).context("Failed to read state file")?;
        let state: Self = serde_json::from_str(&content).context("Failed to parse state file")?;

        info!(
            "Loaded installation state: step={:?}, attempt={}",
            state.step, state.attempt_count
        );
        Ok(Some(state))
    }

    /// Save state to file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&mut self) -> Result<()> {
        self.updated_at = chrono::Utc::now().to_rfc3339();

        let path = Self::state_file(&self.config.output_dir);
        std::fs::create_dir_all(&self.config.output_dir)
            .context("Failed to create output directory")?;

        let content = serde_json::to_string_pretty(self).context("Failed to serialize state")?;
        std::fs::write(&path, content).context("Failed to write state file")?;

        Ok(())
    }

    /// Update the current step and save.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_step(&mut self, step: InstallStep) -> Result<()> {
        info!("Step: {} -> {}", self.step, step);
        self.step = step;
        self.attempt_count = 0; // Reset attempts for new step
        self.last_error = None;
        self.save()
    }

    /// Advance to the next step.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn advance(&mut self) -> Result<()> {
        let next = self.step.next();
        self.set_step(next)
    }

    /// Record an error for the current step.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn record_error(&mut self, error: &str) -> Result<()> {
        self.last_error = Some(error.to_string());
        self.attempt_count += 1;
        self.save()
    }

    /// Clear the last error.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn clear_error(&mut self) -> Result<()> {
        self.last_error = None;
        self.save()
    }

    /// Set control plane server info.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    /// Set the selected region.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_selected_region(&mut self, region: String) -> Result<()> {
        self.selected_region = Some(region);
        self.save()
    }

    pub fn set_control_plane(&mut self, id: String, ip: String, hostname: String) -> Result<()> {
        self.control_plane = Some(ServerState {
            id,
            ip,
            hostname,
            talos_ready: false,
            config_applied: false,
        });
        self.save()
    }

    /// Add a worker server.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn add_worker(&mut self, id: String, ip: String, hostname: String) -> Result<()> {
        self.workers.push(ServerState {
            id,
            ip,
            hostname,
            talos_ready: false,
            config_applied: false,
        });
        self.save()
    }

    /// Mark control plane Talos as ready.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_cp_talos_ready(&mut self) -> Result<()> {
        if let Some(ref mut cp) = self.control_plane {
            cp.talos_ready = true;
        }
        self.save()
    }

    /// Mark a worker's Talos as ready.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_worker_talos_ready(&mut self, index: usize) -> Result<()> {
        if let Some(worker) = self.workers.get_mut(index) {
            worker.talos_ready = true;
        }
        self.save()
    }

    /// Set the kubeconfig path.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_kubeconfig(&mut self, path: PathBuf) -> Result<()> {
        self.kubeconfig_path = Some(path);
        self.save()
    }

    /// Set the ArgoCD password.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_argocd_password(&mut self, password: String) -> Result<()> {
        self.argocd_password = Some(password);
        self.save()
    }

    /// Check if the installation is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.step == InstallStep::Complete
    }

    /// Check if we can resume from this state.
    #[must_use]
    pub fn can_resume(&self) -> bool {
        self.step != InstallStep::NotStarted && self.step != InstallStep::Complete
    }

    /// Set VLAN info.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_vlan(&mut self, vlan_id: String, vid: u16) -> Result<()> {
        self.vlan_id = Some(vlan_id);
        self.vlan_vid = Some(vid);
        self.save()
    }

    /// Set a private IP for a server.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn set_private_ip(&mut self, server_id: &str, private_ip: String) -> Result<()> {
        self.private_ips.insert(server_id.to_string(), private_ip);
        self.save()
    }

    /// Get the private IP for a server.
    #[must_use]
    #[allow(dead_code)] // Will be used when generating configs with VLAN
    pub fn get_private_ip(&self, server_id: &str) -> Option<&String> {
        self.private_ips.get(server_id)
    }

    /// Allocate the next private IP from the VLAN subnet.
    ///
    /// Returns IPs in sequence, skipping network (.0) and broadcast (.255) addresses
    /// within each /24 block:
    /// - /24: 10.8.0.1, 10.8.0.2, ..., 10.8.0.254 (254 hosts)
    /// - /23: 10.8.0.1, ..., 10.8.0.254, 10.8.1.1, ..., 10.8.1.254 (508 hosts)
    /// - /16: 10.8.0.1, ..., 10.8.0.254, 10.8.1.1, ..., 10.8.255.254 (65024 hosts)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The subnet format is invalid
    /// - The subnet is exhausted
    pub fn allocate_next_private_ip(&self) -> Result<String> {
        let subnet = &self.config.vlan_subnet;
        let base = subnet.split('/').next().unwrap_or("10.8.0.0");

        // Parse CIDR prefix to determine available hosts
        let prefix_len: u8 = subnet
            .split('/')
            .nth(1)
            .and_then(|p| p.parse().ok())
            .unwrap_or(24);

        // Calculate max usable hosts, accounting for .0 and .255 in each /24 block
        // Each /24 block has 254 usable hosts (1-254), skipping 0 (network) and 255 (broadcast)
        let max_hosts = if prefix_len >= 31 {
            // /31 and /32 are special cases (point-to-point or single host)
            anyhow::bail!("Subnet /{prefix_len} too small for cluster networking");
        } else if prefix_len >= 24 {
            // Single /24 or smaller: 2^(32-prefix) - 2
            (1u32 << (32 - prefix_len)) - 2
        } else {
            // Multiple /24 blocks: 254 hosts per /24 block
            let num_slash24_blocks = 1u32 << (24 - prefix_len);
            num_slash24_blocks * 254
        };

        let allocation_index = self.private_ips.len();

        if allocation_index >= max_hosts as usize {
            anyhow::bail!(
                "VLAN subnet {subnet} exhausted: cannot allocate host {} (max {max_hosts} hosts)",
                allocation_index + 1
            );
        }

        // Parse base IP octets
        let parts: Vec<&str> = base.split('.').collect();
        if parts.len() != 4 {
            anyhow::bail!("Invalid subnet base address: {base}");
        }

        let octets: Vec<u8> = parts
            .iter()
            .map(|p| p.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| anyhow::anyhow!("Invalid subnet base address: {base}"))?;

        // Convert base to u32
        let base_ip = u32::from(octets[0]) << 24
            | u32::from(octets[1]) << 16
            | u32::from(octets[2]) << 8
            | u32::from(octets[3]);

        // Calculate the IP by mapping allocation index to valid host addresses
        // Each /24 block has 254 usable addresses (1-254), skipping .0 and .255
        let allocation_u32 = u32::try_from(allocation_index)
            .map_err(|_| anyhow::anyhow!("Allocation index {allocation_index} too large"))?;

        // Which /24 block and which host within that block?
        let block_index = allocation_u32 / 254; // Which /24 block (0, 1, 2, ...)
        let host_in_block = allocation_u32 % 254; // Host within block (0-253 -> 1-254)

        // Calculate offset from base: (block * 256) + (host + 1)
        // block * 256 moves to the next /24, host + 1 skips the .0 address
        let offset = block_index * 256 + host_in_block + 1;
        let result_ip = base_ip + offset;

        // Convert back to octets
        let o1 = ((result_ip >> 24) & 0xFF) as u8;
        let o2 = ((result_ip >> 16) & 0xFF) as u8;
        let o3 = ((result_ip >> 8) & 0xFF) as u8;
        let o4 = (result_ip & 0xFF) as u8;

        Ok(format!("{o1}.{o2}.{o3}.{o4}"))
    }
}

/// Retry configuration for operations.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(120),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a given attempt number.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exp = i32::try_from(attempt.min(10)).unwrap_or(10);
        let multiplier = self.backoff_multiplier.powi(exp);
        let delay_secs = self.initial_delay.as_secs_f64() * multiplier;
        let capped = delay_secs.min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(capped)
    }

    /// Check if we should retry given the current attempt count.
    #[must_use]
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_progression() {
        let mut step = InstallStep::NotStarted;
        let mut steps = vec![step.clone()];

        while step != InstallStep::Complete {
            step = step.next();
            steps.push(step.clone());
        }

        // Should end at Complete
        assert_eq!(steps.last().unwrap(), &InstallStep::Complete);
        // Complete.next() should stay Complete
        assert_eq!(InstallStep::Complete.next(), InstallStep::Complete);
    }

    #[test]
    fn test_step_numbers() {
        assert_eq!(InstallStep::NotStarted.step_number(), 0);
        assert_eq!(
            InstallStep::Complete.step_number(),
            InstallStep::TOTAL_STEPS
        );
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();

        // First attempt: 5s
        assert_eq!(config.delay_for_attempt(0), Duration::from_secs(5));
        // Second attempt: 10s
        assert_eq!(config.delay_for_attempt(1), Duration::from_secs(10));
        // Third attempt: 20s
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(20));

        // Should cap at max_delay
        let huge_delay = config.delay_for_attempt(100);
        assert!(huge_delay <= config.max_delay);
    }

    #[test]
    fn test_should_retry() {
        let config = RetryConfig::default();
        assert!(config.should_retry(0));
        assert!(config.should_retry(9));
        assert!(!config.should_retry(10));
        assert!(!config.should_retry(100));
    }

    /// Helper to create a minimal state for testing IP allocation.
    fn create_test_state_with_subnet(subnet: &str) -> InstallState {
        use crate::config::{BareMetalProvider, InstallConfig, InstallProfile};

        let config = InstallConfig {
            cluster_name: "test".into(),
            provider: BareMetalProvider::Latitude,
            region: "MIA2".into(),
            auto_region: false,
            fallback_regions: vec![],
            cp_plan: "test".into(),
            worker_plan: "test".into(),
            node_count: 3,
            ssh_keys: vec![],
            talos_version: "v1.0.0".into(),
            install_disk: "/dev/sda".into(),
            storage_disk: None,
            storage_replicas: 2,
            output_dir: PathBuf::from("/tmp/test"),
            gitops_repo: "test".into(),
            gitops_branch: "main".into(),
            sync_timeout_minutes: 30,
            profile: InstallProfile::default(),
            enable_vlan: true,
            vlan_subnet: subnet.into(),
            vlan_parent_interface: "eth1".into(),
            enable_firewall: true,
        };

        InstallState {
            config,
            step: InstallStep::NotStarted,
            selected_region: None,
            control_plane: None,
            workers: vec![],
            kubeconfig_path: None,
            argocd_password: None,
            updated_at: String::new(),
            attempt_count: 0,
            last_error: None,
            vlan_id: None,
            vlan_vid: None,
            private_ips: HashMap::new(),
        }
    }

    #[test]
    fn test_allocate_private_ip_slash24_sequential() {
        let mut state = create_test_state_with_subnet("10.8.0.0/24");

        // First 3 allocations should be .1, .2, .3
        let ip1 = state.allocate_next_private_ip().unwrap();
        assert_eq!(ip1, "10.8.0.1");
        state.private_ips.insert("server1".into(), ip1);

        let ip2 = state.allocate_next_private_ip().unwrap();
        assert_eq!(ip2, "10.8.0.2");
        state.private_ips.insert("server2".into(), ip2);

        let ip3 = state.allocate_next_private_ip().unwrap();
        assert_eq!(ip3, "10.8.0.3");
        state.private_ips.insert("server3".into(), ip3);
    }

    #[test]
    fn test_allocate_private_ip_skips_broadcast() {
        let mut state = create_test_state_with_subnet("10.8.0.0/24");

        // Pre-fill 253 IPs (indices 0-252 -> addresses .1-.253)
        for i in 0..253 {
            state
                .private_ips
                .insert(format!("server{i}"), format!("10.8.0.{}", i + 1));
        }

        // Next allocation (index 253) should be .254, NOT .255 (broadcast)
        let ip254 = state.allocate_next_private_ip().unwrap();
        assert_eq!(ip254, "10.8.0.254");
        state.private_ips.insert("server253".into(), ip254);

        // Subnet should now be exhausted (254 hosts max in /24)
        let result = state.allocate_next_private_ip();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exhausted"));
    }

    #[test]
    fn test_allocate_private_ip_slash23_wraps_blocks() {
        let mut state = create_test_state_with_subnet("10.8.0.0/23");

        // Pre-fill first /24 block (254 hosts: .0.1 through .0.254)
        for i in 0..254 {
            state
                .private_ips
                .insert(format!("server{i}"), format!("10.8.0.{}", i + 1));
        }

        // Next allocation should wrap to .1.1, skipping .0.255 (broadcast) and .1.0 (network)
        let ip = state.allocate_next_private_ip().unwrap();
        assert_eq!(ip, "10.8.1.1");
    }

    #[test]
    fn test_allocate_private_ip_small_subnet_error() {
        let state = create_test_state_with_subnet("10.8.0.0/31");

        // /31 should fail as too small
        let result = state.allocate_next_private_ip();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
