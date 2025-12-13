//! Installer orchestration module.
//!
//! This module provides the main installation logic, coordinating:
//! - Bare metal server provisioning via the metal crate
//! - Talos Linux bootstrapping
//! - Platform stack deployment
//! - GitOps sync verification

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{error, info, warn};

use crate::bare_metal::BareMetalOrchestrator;
use crate::bootstrap::BootstrapResources;
use crate::config::InstallConfig;
use crate::gitops::GitOpsVerifier;
use crate::kubeconfig;
use crate::state::{InstallState, InstallStep, RetryConfig};
use crate::ui;

/// Main installer struct that orchestrates the full installation process.
pub struct Installer {
    /// Installation state (persisted across runs).
    state: InstallState,
    /// Retry configuration for transient errors.
    retry_config: RetryConfig,
}

impl Installer {
    /// Create a new installer or resume from existing state.
    ///
    /// If state exists in the output directory, it will be loaded and the
    /// installation will resume from where it left off.
    ///
    /// # Errors
    ///
    /// Returns an error if state loading fails.
    pub async fn new_or_resume(config: InstallConfig) -> Result<Self> {
        // Ensure output directory exists
        std::fs::create_dir_all(&config.output_dir).context("Failed to create output directory")?;

        let state = if let Some(existing) = InstallState::load(&config.output_dir)? {
            // Found existing state - resume from there
            if existing.can_resume() {
                ui::print_info(&format!("Resuming installation from: {}", existing.step));
                if let Some(ref err) = existing.last_error {
                    ui::print_warning(&format!("Previous error: {err}"));
                }
                existing
            } else if existing.is_complete() {
                ui::print_success("Installation already complete!");
                existing
            } else {
                // NotStarted state, use fresh config
                InstallState::new(config)
            }
        } else {
            // Fresh installation
            InstallState::new(config)
        };

        Ok(Self {
            state,
            retry_config: RetryConfig::default(),
        })
    }

    /// Run installation to completion with automatic retry/resume.
    ///
    /// This method will:
    /// 1. Execute each step in sequence
    /// 2. Automatically retry transient failures with exponential backoff
    /// 3. Save state after each step for recovery
    /// 4. Print progress updates throughout
    ///
    /// # Errors
    ///
    /// Returns an error if installation fails after all retries.
    pub async fn run_to_completion(&mut self) -> Result<()> {
        // Already complete?
        if self.state.is_complete() {
            self.print_success_summary();
            return Ok(());
        }

        ui::print_section("Starting Installation");
        ui::print_progress_step(
            self.state.step.step_number(),
            InstallStep::TOTAL_STEPS,
            self.state.step.description(),
        );

        loop {
            match self.execute_current_step().await {
                Ok(()) => {
                    self.state.clear_error()?;

                    if self.state.is_complete() {
                        self.print_success_summary();
                        return Ok(());
                    }

                    // Advance to next step
                    self.state.advance()?;
                    ui::print_progress_step(
                        self.state.step.step_number(),
                        InstallStep::TOTAL_STEPS,
                        self.state.step.description(),
                    );
                }
                Err(e) => {
                    self.state.record_error(&e.to_string())?;

                    if Self::is_transient_error(&e)
                        && self.retry_config.should_retry(self.state.attempt_count)
                    {
                        let delay = self
                            .retry_config
                            .delay_for_attempt(self.state.attempt_count);
                        warn!(
                            attempt = self.state.attempt_count,
                            error = %e,
                            delay_secs = delay.as_secs(),
                            "Transient error, retrying"
                        );
                        ui::print_warning(&format!(
                            "Transient error (attempt {}): {}",
                            self.state.attempt_count, e
                        ));
                        ui::print_info(&format!("Retrying in {} seconds...", delay.as_secs()));
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    // Hard failure - state is saved, can be resumed later
                    error!(
                        step = ?self.state.step,
                        error = %e,
                        "Installation failed"
                    );
                    ui::print_error(&format!(
                        "Installation failed at step '{}': {}",
                        self.state.step, e
                    ));
                    ui::print_info("State has been saved. Re-run the same command to resume.");
                    return Err(e);
                }
            }
        }
    }

    /// Execute the current step.
    async fn execute_current_step(&mut self) -> Result<()> {
        info!(step = ?self.state.step, "Executing step");

        match &self.state.step {
            InstallStep::NotStarted => {
                // Move to first real step
                self.state.set_step(InstallStep::ValidatingPrerequisites)?;
            }

            InstallStep::ValidatingPrerequisites | InstallStep::Complete => {
                // Already validated in run(), just advance / nothing to do
            }

            // Infrastructure steps (delegate to bare metal orchestrator)
            InstallStep::CreatingServers => {
                self.create_servers().await?;
            }
            InstallStep::WaitingServersReady => {
                self.wait_servers_ready().await?;
            }
            InstallStep::BootingTalos => {
                self.boot_talos().await?;
            }
            InstallStep::WaitingTalosMaintenance => {
                self.wait_talos_maintenance().await?;
            }

            // Talos bootstrap steps
            InstallStep::GeneratingConfigs => {
                self.generate_configs().await?;
            }
            InstallStep::ApplyingCPConfig => {
                self.apply_cp_config().await?;
            }
            InstallStep::WaitingCPInstall => {
                self.wait_cp_install().await?;
            }
            InstallStep::Bootstrapping => {
                self.bootstrap_cluster().await?;
            }
            // Cilium must be deployed before WaitingKubernetes because nodes
            // can't be Ready without CNI
            InstallStep::DeployingCilium => {
                self.deploy_cilium().await?;
            }
            InstallStep::WaitingKubernetes => {
                self.wait_kubernetes().await?;
            }
            InstallStep::ApplyingWorkerConfig => {
                self.apply_worker_configs().await?;
            }
            InstallStep::WaitingWorkerJoin => {
                self.wait_workers_join().await?;
            }

            // Platform stack steps
            InstallStep::DeployingBootstrapResources => {
                self.deploy_bootstrap_resources().await?;
            }
            InstallStep::DeployingLocalPathProvisioner => {
                self.deploy_local_path_provisioner().await?;
            }
            InstallStep::DeployingArgoCD => {
                self.deploy_argocd().await?;
            }
            InstallStep::WaitingArgoCDReady => {
                self.wait_argocd_ready().await?;
            }
            InstallStep::ApplyingAppOfApps => {
                self.apply_app_of_apps().await?;
            }
            InstallStep::WaitingGitOpsSync => {
                self.wait_gitops_sync().await?;
            }

            // Post-GitOps configuration
            InstallStep::ConfiguringStorage => {
                self.configure_storage().await?;
            }
            InstallStep::ConfiguringKubeconfig => {
                self.configure_kubeconfig().await?;
            }
        }

        Ok(())
    }

    // --- Infrastructure Steps ---

    async fn create_servers(&mut self) -> Result<()> {
        let orchestrator = BareMetalOrchestrator::new(&self.state.config).await?;

        // Select region (auto or configured)
        let region = orchestrator.select_region().await?;
        self.state.set_selected_region(region.clone())?;

        let (cp, workers) = orchestrator.create_servers(&region).await?;

        self.state.set_control_plane(cp.id, cp.ip, cp.hostname)?;
        for w in workers {
            self.state.add_worker(w.id, w.ip, w.hostname)?;
        }

        Ok(())
    }

    async fn wait_servers_ready(&mut self) -> Result<()> {
        let orchestrator = BareMetalOrchestrator::new(&self.state.config).await?;

        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let worker_ids: Vec<_> = self.state.workers.iter().map(|w| w.id.clone()).collect();

        orchestrator.wait_servers_ready(&cp.id, &worker_ids).await?;

        Ok(())
    }

    async fn boot_talos(&mut self) -> Result<()> {
        let orchestrator = BareMetalOrchestrator::new(&self.state.config).await?;

        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let worker_ids: Vec<_> = self.state.workers.iter().map(|w| w.id.clone()).collect();

        orchestrator.boot_talos(&cp.id, &worker_ids).await?;

        Ok(())
    }

    async fn wait_talos_maintenance(&mut self) -> Result<()> {
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let worker_ips: Vec<_> = self.state.workers.iter().map(|w| w.ip.clone()).collect();

        let timeout = Duration::from_secs(900); // 15 minutes

        // Wait for control plane
        ui::print_info(&format!("Waiting for Talos on control plane ({})", cp.ip));
        metal::talos::wait_for_talos(&cp.ip, timeout)?;
        self.state.set_cp_talos_ready()?;

        // Wait for workers
        for (i, ip) in worker_ips.iter().enumerate() {
            ui::print_info(&format!("Waiting for Talos on worker {} ({ip})", i + 1));
            metal::talos::wait_for_talos(ip, timeout)?;
            self.state.set_worker_talos_ready(i)?;
        }

        Ok(())
    }

    // --- Talos Bootstrap Steps ---

    async fn generate_configs(&mut self) -> Result<()> {
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let config = metal::talos::BootstrapConfig::new(
            self.state.config.cluster_name.clone(),
            cp.ip.clone(),
        )
        .with_install_disk(&self.state.config.install_disk)
        .with_output_dir(&self.state.config.output_dir)
        .with_talos_version(&self.state.config.talos_version)
        .with_kube_proxy_mode("none") // We use Cilium
        .with_cilium_cni();

        // Generate secrets
        metal::talos::generate_secrets(&self.state.config.output_dir)?;

        // Generate configs
        metal::talos::generate_config(&config)?;

        Ok(())
    }

    async fn apply_cp_config(&mut self) -> Result<()> {
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let config_path = self.state.config.output_dir.join("controlplane.yaml");

        metal::talos::apply_config(&cp.ip, &config_path)?;

        Ok(())
    }

    async fn wait_cp_install(&mut self) -> Result<()> {
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let talosconfig = self.state.config.talosconfig_path();
        let timeout = Duration::from_secs(600); // 10 minutes

        metal::talos::wait_for_install(&cp.ip, &talosconfig, timeout)?;

        Ok(())
    }

    async fn bootstrap_cluster(&mut self) -> Result<()> {
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let talosconfig = self.state.config.talosconfig_path();
        let kubeconfig_path = self.state.config.kubeconfig_path();

        metal::talos::bootstrap_cluster(&cp.ip, &talosconfig)?;

        // Wait briefly for K8s API to become available
        ui::print_info("Waiting for Kubernetes API to be available...");
        let timeout = Duration::from_secs(300); // 5 minutes
        metal::talos::wait_for_kubernetes(&cp.ip, &talosconfig, timeout)?;

        // Get kubeconfig (needed for Cilium deployment)
        ui::print_info("Getting kubeconfig...");
        metal::talos::get_kubeconfig(&cp.ip, &talosconfig, &kubeconfig_path)?;
        self.state.set_kubeconfig(kubeconfig_path)?;

        Ok(())
    }

    async fn wait_kubernetes(&mut self) -> Result<()> {
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        // Now that Cilium is deployed, wait for all nodes to be Ready
        ui::print_info("Waiting for all nodes to be Ready (CNI is now deployed)...");

        // Use kubectl to check node status instead of talosctl health
        // because talosctl health is too strict (requires all nodes ready)
        let timeout = Duration::from_secs(300); // 5 minutes
        wait_for_node_ready_kubectl(kubeconfig_path, &cp.hostname, timeout).await?;

        Ok(())
    }

    async fn apply_worker_configs(&mut self) -> Result<()> {
        let worker_config_path = self.state.config.output_dir.join("worker.yaml");

        for worker in &self.state.workers {
            ui::print_info(&format!(
                "Applying config to worker {} ({})",
                worker.hostname, worker.ip
            ));
            metal::talos::apply_config(&worker.ip, &worker_config_path)?;
        }

        Ok(())
    }

    async fn wait_workers_join(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        let timeout = Duration::from_secs(600); // 10 minutes

        // Wait for all nodes to be ready
        ui::print_info("Waiting for all nodes to join the cluster...");
        metal::talos::wait_for_node_ready(kubeconfig_path, timeout)?;

        Ok(())
    }

    // --- Platform Stack Steps ---

    async fn deploy_cilium(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Deploying Cilium CNI...");

        // Use cluster name and a default cluster ID of 1 for single-cluster deployments
        // For multi-cluster setups, cluster_id should be configurable (1-255)
        let cluster_name = &self.state.config.cluster_name;
        let cluster_id = 1_u8; // TODO: Make configurable for multi-cluster

        metal::stack::deploy_cilium(kubeconfig_path, cluster_name, cluster_id)?;

        // Wait for Cilium to be healthy before proceeding
        ui::print_info("Waiting for Cilium to be healthy...");
        metal::cilium::wait_for_cilium_healthy(kubeconfig_path)?;

        Ok(())
    }

    async fn deploy_bootstrap_resources(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Applying bootstrap resources...");
        BootstrapResources::apply(kubeconfig_path)?;

        Ok(())
    }

    async fn deploy_local_path_provisioner(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Deploying local-path-provisioner...");
        metal::stack::deploy_local_path_provisioner(kubeconfig_path)?;

        Ok(())
    }

    async fn deploy_argocd(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Deploying ArgoCD...");
        metal::stack::deploy_argocd(kubeconfig_path)?;

        Ok(())
    }

    async fn wait_argocd_ready(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Waiting for ArgoCD to be ready...");

        // Wait for the argocd-server deployment to be ready
        wait_for_deployment(
            kubeconfig_path,
            "argocd",
            "argocd-server",
            Duration::from_secs(300),
        )
        .await?;

        // Get ArgoCD password
        let password = metal::stack::get_argocd_password(kubeconfig_path)?;
        self.state.set_argocd_password(password)?;

        Ok(())
    }

    async fn apply_app_of_apps(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Applying app-of-apps manifest...");

        let verifier = GitOpsVerifier::from_kubeconfig(kubeconfig_path).await?;
        verifier
            .apply_app_of_apps(
                &self.state.config.gitops_repo,
                &self.state.config.gitops_branch,
            )
            .await?;

        Ok(())
    }

    async fn wait_gitops_sync(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        let timeout = Duration::from_secs(u64::from(self.state.config.sync_timeout_minutes) * 60);

        ui::print_info(&format!(
            "Waiting for GitOps sync (timeout: {} minutes)...",
            self.state.config.sync_timeout_minutes
        ));

        let verifier = GitOpsVerifier::from_kubeconfig(kubeconfig_path).await?;
        let report = verifier.wait_for_full_sync(timeout).await?;

        ui::print_success(&format!(
            "All {} applications synced and healthy!",
            report.total_count
        ));

        Ok(())
    }

    // --- Post-GitOps Configuration ---

    async fn configure_storage(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Configuring Mayastor storage...");

        // Wait for Mayastor to be fully deployed by ArgoCD
        ui::print_info("Waiting for Mayastor io-engine pods to be ready...");
        wait_for_deployment(
            kubeconfig_path,
            "mayastor",
            "mayastor-io-engine",
            Duration::from_secs(600), // 10 minutes - Mayastor can take a while
        )
        .await
        .context("Mayastor io-engine not ready - storage may need manual configuration")?;

        // Determine storage disk
        let storage_disk = self
            .state
            .config
            .storage_disk
            .clone()
            .unwrap_or_else(|| self.state.config.install_disk.clone());

        // Create DiskPool on each node
        let cp = self
            .state
            .control_plane
            .as_ref()
            .context("Control plane not found in state")?;

        // Control plane node pool
        let cp_pool_name = format!("{}-pool", cp.hostname);
        let disk_uri = format!("aio://{storage_disk}");

        ui::print_info(&format!(
            "Creating DiskPool {} on {} ({})",
            cp_pool_name, cp.hostname, disk_uri
        ));
        if let Err(e) = metal::stack::create_mayastor_diskpool(
            kubeconfig_path,
            "mayastor",
            &cp_pool_name,
            &cp.hostname,
            &disk_uri,
        ) {
            ui::print_warning(&format!(
                "Could not create DiskPool on {}: {e}",
                cp.hostname
            ));
            ui::print_info("You may need to configure storage manually with a different disk");
        }

        // Worker node pools
        for worker in &self.state.workers {
            let pool_name = format!("{}-pool", worker.hostname);
            ui::print_info(&format!(
                "Creating DiskPool {} on {} ({})",
                pool_name, worker.hostname, disk_uri
            ));
            if let Err(e) = metal::stack::create_mayastor_diskpool(
                kubeconfig_path,
                "mayastor",
                &pool_name,
                &worker.hostname,
                &disk_uri,
            ) {
                ui::print_warning(&format!(
                    "Could not create DiskPool on {}: {e}",
                    worker.hostname
                ));
            }
        }

        // Create StorageClass
        let replicas = self
            .state
            .config
            .storage_replicas
            .min(self.state.config.node_count);
        ui::print_info(&format!(
            "Creating mayastor-nvme StorageClass with {replicas} replicas"
        ));
        metal::stack::create_mayastor_storage_class(
            kubeconfig_path,
            "mayastor-nvme",
            replicas,
            true, // make default (removes local-path as default)
        )?;

        ui::print_success("Mayastor storage configured!");

        Ok(())
    }

    async fn configure_kubeconfig(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Merging kubeconfig for kubectl and Lens access...");

        match kubeconfig::merge_kubeconfig(kubeconfig_path, &self.state.config.cluster_name) {
            Ok(merged_path) => {
                ui::print_success(&format!("Kubeconfig merged into {}", merged_path.display()));
                ui::print_info(&format!(
                    "Current context set to: {}",
                    self.state.config.cluster_name
                ));
            }
            Err(e) => {
                ui::print_warning(&format!("Could not merge kubeconfig: {e}"));
                ui::print_info("You can manually merge it with:");
                ui::print_info(&format!(
                    "  KUBECONFIG=~/.kube/config:{} kubectl config view --flatten > ~/.kube/config.merged",
                    kubeconfig_path.display()
                ));
                ui::print_info("  mv ~/.kube/config.merged ~/.kube/config");
            }
        }

        Ok(())
    }

    // --- Utility Methods ---

    /// Check if an error is transient and worth retrying.
    fn is_transient_error(error: &anyhow::Error) -> bool {
        let msg = error.to_string().to_lowercase();
        msg.contains("timeout")
            || msg.contains("timed out")
            || msg.contains("connection refused")
            || msg.contains("connection reset")
            || msg.contains("rate limit")
            || msg.contains("503")
            || msg.contains("502")
            || msg.contains("504")
            || msg.contains("temporarily unavailable")
            || msg.contains("try again")
            || msg.contains("network")
    }

    /// Print success summary at the end.
    fn print_success_summary(&self) {
        ui::print_section("Installation Complete!");
        ui::print_success("Your CTO Platform cluster is ready.");

        ui::print_info(&format!("Cluster: {}", self.state.config.cluster_name));
        let region = self
            .state
            .selected_region
            .as_ref()
            .unwrap_or(&self.state.config.region);
        ui::print_info(&format!("Region: {region}"));

        if let Some(ref kc) = self.state.kubeconfig_path {
            ui::print_info(&format!("Kubeconfig: {}", kc.display()));
        }

        if let Some(ref cp) = self.state.control_plane {
            ui::print_info(&format!("Control Plane IP: {}", cp.ip));
        }

        // Print Lens instructions (kubeconfig already merged in ConfiguringKubeconfig step)
        kubeconfig::print_lens_instructions(&self.state.config.cluster_name);

        if let Some(ref password) = self.state.argocd_password {
            ui::print_section("ArgoCD Access");
            ui::print_info("URL: https://argocd.<your-domain>");
            ui::print_info("Username: admin");
            ui::print_info(&format!("Password: {password}"));
        }

        ui::print_section("Quick Start");
        ui::print_info("1. Verify cluster access:");
        ui::print_info("   kubectl get nodes");
        ui::print_info("");
        ui::print_info("2. Access ArgoCD dashboard:");
        ui::print_info("   kubectl port-forward svc/argocd-server -n argocd 8080:443");
        ui::print_info("   Open: https://localhost:8080");
        ui::print_info("");
        ui::print_info("3. Or use Lens for a full Kubernetes IDE experience!");
    }
}

/// Wait for a deployment to be ready.
async fn wait_for_deployment(
    kubeconfig: &Path,
    namespace: &str,
    name: &str,
    timeout: Duration,
) -> Result<()> {
    use std::process::Command;
    use std::time::Instant;

    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for deployment {namespace}/{name} to be ready");
        }

        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "rollout",
                "status",
                "deployment",
                name,
                "-n",
                namespace,
                "--timeout=30s",
            ])
            .output()
            .context("Failed to run kubectl rollout status")?;

        if output.status.success() {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

/// Wait for a specific node to be Ready using kubectl.
async fn wait_for_node_ready_kubectl(
    kubeconfig: &Path,
    node_name: &str,
    timeout: Duration,
) -> Result<()> {
    use std::process::Command;
    use std::time::Instant;

    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for node {node_name} to be Ready");
        }

        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "get",
                "node",
                node_name,
                "-o",
                "jsonpath={.status.conditions[?(@.type=='Ready')].status}",
            ])
            .output()
            .context("Failed to run kubectl get node")?;

        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.trim() == "True" {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
