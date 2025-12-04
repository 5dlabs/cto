//! Cluster provisioning state persistence.
//!
//! This module provides state tracking for cluster provisioning operations,
//! allowing recovery from failures and resumption of interrupted processes.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Cluster provisioning state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    /// Cluster name.
    pub name: String,
    /// Current provisioning step.
    pub step: ProvisionStep,
    /// Control plane server info.
    pub control_plane: Option<ServerState>,
    /// Worker server info.
    pub worker: Option<ServerState>,
    /// Output directory for configs.
    pub output_dir: PathBuf,
    /// Timestamp of last update.
    pub updated_at: String,
}

/// Individual server state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    /// Server ID from provider.
    pub id: String,
    /// Server IP address.
    pub ip: String,
    /// Server hostname.
    pub hostname: String,
    /// Whether Talos is ready.
    pub talos_ready: bool,
    /// Whether config has been applied.
    pub config_applied: bool,
}

/// Provisioning steps for tracking progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionStep {
    /// Initial state - no servers created.
    NotStarted,
    /// Servers are being created.
    CreatingServers,
    /// Waiting for servers to be ready.
    WaitingServersReady,
    /// iPXE boot triggered, waiting for Talos.
    WaitingTalos,
    /// Generating cluster configs.
    GeneratingConfigs,
    /// Applying control plane config.
    ApplyingCpConfig,
    /// Waiting for control plane install.
    WaitingCpInstall,
    /// Bootstrapping cluster.
    Bootstrapping,
    /// Waiting for Kubernetes API.
    WaitingKubernetes,
    /// Applying worker config.
    ApplyingWorkerConfig,
    /// Waiting for worker to join.
    WaitingWorkerJoin,
    /// Cluster is ready.
    Complete,
    /// Provisioning failed.
    Failed,
}

impl ClusterState {
    /// Create a new cluster state.
    #[must_use]
    pub fn new(name: impl Into<String>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            step: ProvisionStep::NotStarted,
            control_plane: None,
            worker: None,
            output_dir: output_dir.into(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Get the state file path.
    #[must_use]
    pub fn state_file(output_dir: &Path) -> PathBuf {
        output_dir.join("cluster-state.json")
    }

    /// Load state from file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(output_dir: &Path) -> Result<Option<Self>> {
        let path = Self::state_file(output_dir);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)
            .context("Failed to read state file")?;
        let state: Self = serde_json::from_str(&content)
            .context("Failed to parse state file")?;

        info!("Loaded cluster state: step={:?}", state.step);
        Ok(Some(state))
    }

    /// Save state to file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save(&mut self) -> Result<()> {
        self.updated_at = chrono::Utc::now().to_rfc3339();

        let path = Self::state_file(&self.output_dir);
        std::fs::create_dir_all(&self.output_dir)
            .context("Failed to create output directory")?;

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize state")?;
        std::fs::write(&path, content)
            .context("Failed to write state file")?;

        Ok(())
    }

    /// Update the current step and save.
    ///
    /// # Errors
    /// Returns an error if saving fails.
    pub fn set_step(&mut self, step: ProvisionStep) -> Result<()> {
        self.step = step;
        self.save()
    }

    /// Set control plane info and save.
    ///
    /// # Errors
    /// Returns an error if saving fails.
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

    /// Set worker info and save.
    ///
    /// # Errors
    /// Returns an error if saving fails.
    pub fn set_worker(&mut self, id: String, ip: String, hostname: String) -> Result<()> {
        self.worker = Some(ServerState {
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
    /// Returns an error if saving fails.
    pub fn set_cp_talos_ready(&mut self) -> Result<()> {
        if let Some(ref mut cp) = self.control_plane {
            cp.talos_ready = true;
        }
        self.save()
    }

    /// Mark worker Talos as ready.
    ///
    /// # Errors
    /// Returns an error if saving fails.
    pub fn set_worker_talos_ready(&mut self) -> Result<()> {
        if let Some(ref mut w) = self.worker {
            w.talos_ready = true;
        }
        self.save()
    }

    /// Check if we can resume from the current state.
    #[must_use]
    pub fn can_resume(&self) -> bool {
        !matches!(self.step, ProvisionStep::NotStarted | ProvisionStep::Complete | ProvisionStep::Failed)
    }

    /// Get the next step to execute based on current state.
    #[must_use]
    pub fn next_step(&self) -> ProvisionStep {
        match self.step {
            ProvisionStep::NotStarted => ProvisionStep::CreatingServers,
            ProvisionStep::CreatingServers => ProvisionStep::WaitingServersReady,
            ProvisionStep::WaitingServersReady => ProvisionStep::WaitingTalos,
            ProvisionStep::WaitingTalos => ProvisionStep::GeneratingConfigs,
            ProvisionStep::GeneratingConfigs => ProvisionStep::ApplyingCpConfig,
            ProvisionStep::ApplyingCpConfig => ProvisionStep::WaitingCpInstall,
            ProvisionStep::WaitingCpInstall => ProvisionStep::Bootstrapping,
            ProvisionStep::Bootstrapping => ProvisionStep::WaitingKubernetes,
            ProvisionStep::WaitingKubernetes => ProvisionStep::ApplyingWorkerConfig,
            ProvisionStep::ApplyingWorkerConfig => ProvisionStep::WaitingWorkerJoin,
            ProvisionStep::WaitingWorkerJoin
            | ProvisionStep::Complete
            | ProvisionStep::Failed => ProvisionStep::Complete,
        }
    }
}

/// Retry configuration for operations.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts.
    pub max_attempts: u32,
    /// Initial delay between retries.
    pub initial_delay: std::time::Duration,
    /// Maximum delay between retries.
    pub max_delay: std::time::Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: std::time::Duration::from_secs(5),
            max_delay: std::time::Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// Execute a function with retry logic.
///
/// # Errors
/// Returns an error if all attempts fail.
pub fn with_retry<T, F>(config: &RetryConfig, operation_name: &str, mut f: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_attempts {
                    return Err(e).context(format!(
                        "{operation_name} failed after {attempt} attempts"
                    ));
                }

                info!(
                    "{operation_name} failed (attempt {attempt}/{}): {e}. Retrying in {delay:?}...",
                    config.max_attempts
                );

                std::thread::sleep(delay);
                delay = std::cmp::min(
                    config.max_delay,
                    std::time::Duration::from_secs_f64(delay.as_secs_f64() * config.backoff_multiplier),
                );
            }
        }
    }
}

/// Execute an async function with retry logic.
///
/// # Errors
/// Returns an error if all attempts fail.
pub async fn with_retry_async<T, F, Fut>(
    config: &RetryConfig,
    operation_name: &str,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_attempts {
                    return Err(e).context(format!(
                        "{operation_name} failed after {attempt} attempts"
                    ));
                }

                info!(
                    "{operation_name} failed (attempt {attempt}/{}): {e}. Retrying in {delay:?}...",
                    config.max_attempts
                );

                tokio::time::sleep(delay).await;
                delay = std::cmp::min(
                    config.max_delay,
                    std::time::Duration::from_secs_f64(delay.as_secs_f64() * config.backoff_multiplier),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_state_new() {
        let state = ClusterState::new("test-cluster", "/tmp/test");
        assert_eq!(state.name, "test-cluster");
        assert_eq!(state.step, ProvisionStep::NotStarted);
        assert!(state.control_plane.is_none());
        assert!(state.worker.is_none());
    }

    #[test]
    fn test_provision_step_progression() {
        let mut state = ClusterState::new("test", "/tmp/test");
        assert_eq!(state.next_step(), ProvisionStep::CreatingServers);

        state.step = ProvisionStep::CreatingServers;
        assert_eq!(state.next_step(), ProvisionStep::WaitingServersReady);

        state.step = ProvisionStep::WaitingTalos;
        assert_eq!(state.next_step(), ProvisionStep::GeneratingConfigs);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
}

