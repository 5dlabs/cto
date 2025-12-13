//! Installation state persistence.
//!
//! This module provides state tracking for cluster installation operations,
//! allowing automatic recovery from failures and resumption of interrupted processes.

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
            Self::CreatingServers => Self::WaitingServersReady,
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
            Self::ConfiguringStorage => Self::ConfiguringKubeconfig,
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
            Self::WaitingServersReady => 3,
            Self::BootingTalos => 4,
            Self::WaitingTalosMaintenance => 5,
            Self::GeneratingConfigs => 6,
            Self::ApplyingCPConfig => 7,
            Self::WaitingCPInstall => 8,
            Self::Bootstrapping => 9,
            Self::DeployingCilium => 10,
            Self::WaitingKubernetes => 11,
            Self::ApplyingWorkerConfig => 12,
            Self::WaitingWorkerJoin => 13,
            Self::DeployingBootstrapResources => 14,
            Self::DeployingLocalPathProvisioner => 15,
            Self::DeployingArgoCD => 16,
            Self::WaitingArgoCDReady => 17,
            Self::ApplyingAppOfApps => 18,
            Self::WaitingGitOpsSync => 19,
            Self::ConfiguringStorage => 20,
            Self::ConfiguringKubeconfig => 21,
            Self::Complete => 22,
        }
    }

    /// Total number of steps.
    pub const TOTAL_STEPS: u8 = 22;
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
}
