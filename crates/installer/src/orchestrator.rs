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

        let state = if let Some(mut existing) = InstallState::load(&config.output_dir)? {
            // Found existing state - resume from there
            if existing.can_resume() {
                ui::print_info(&format!("Resuming installation from: {}", existing.step));
                if let Some(ref err) = existing.last_error {
                    ui::print_warning(&format!("Previous error: {err}"));
                }

                // Check if config has changed - warn and update for certain fields
                Self::check_and_update_config(&mut existing, &config)?;

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

    /// Check if config has changed and update safe-to-update fields.
    ///
    /// Certain fields (like plan) cannot change mid-installation if servers exist.
    /// Other fields (like VLAN interface) can be updated when safe.
    fn check_and_update_config(
        existing: &mut InstallState,
        new_config: &InstallConfig,
    ) -> Result<()> {
        let servers_exist = existing.control_plane.is_some() || !existing.workers.is_empty();

        // Critical fields that can't change once servers exist
        if servers_exist {
            Self::warn_protected_field_changes(&existing.config, new_config)?;
        }

        // VLAN-related fields can only be updated if no servers exist yet
        // (these affect Talos machine config and private IP allocation)
        if !servers_exist {
            Self::update_vlan_config(&mut existing.config, new_config);
        }

        // Update GitOps settings (always safe to update)
        existing
            .config
            .gitops_repo
            .clone_from(&new_config.gitops_repo);
        existing
            .config
            .gitops_branch
            .clone_from(&new_config.gitops_branch);
        existing.config.sync_timeout_minutes = new_config.sync_timeout_minutes;

        existing.save()?;
        Ok(())
    }

    /// Warn about protected field changes when servers exist.
    fn warn_protected_field_changes(old: &InstallConfig, new: &InstallConfig) -> Result<()> {
        if old.cp_plan != new.cp_plan {
            ui::print_warning(&format!(
                "⚠️  Control plane plan changed from '{}' to '{}' but servers already exist!",
                old.cp_plan, new.cp_plan
            ));
            ui::print_info("   Delete state file to start fresh, or continue with existing plan.");
        }
        if old.worker_plan != new.worker_plan {
            ui::print_warning(&format!(
                "⚠️  Worker plan changed from '{}' to '{}' but servers already exist!",
                old.worker_plan, new.worker_plan
            ));
        }
        if old.cluster_name != new.cluster_name {
            anyhow::bail!(
                "Cluster name changed from '{}' to '{}'. Delete state file to start fresh.",
                old.cluster_name,
                new.cluster_name
            );
        }
        if old.vlan_parent_interface != new.vlan_parent_interface {
            ui::print_warning(&format!(
                "⚠️  VLAN interface changed from '{}' to '{}' but servers already exist!",
                old.vlan_parent_interface, new.vlan_parent_interface
            ));
            ui::print_info(
                "   The VLAN interface is hardware-specific. Continuing with existing value.",
            );
            ui::print_info(&format!(
                "   Delete state file to start fresh with interface '{}'.",
                new.vlan_parent_interface
            ));
        }
        if old.vlan_subnet != new.vlan_subnet {
            ui::print_warning(&format!(
                "⚠️  VLAN subnet changed from '{}' to '{}' but servers already exist!",
                old.vlan_subnet, new.vlan_subnet
            ));
            ui::print_info(
                "   Private IPs were allocated from the original subnet. Continuing with existing value.",
            );
            ui::print_info(&format!(
                "   Delete state file to start fresh with subnet '{}'.",
                new.vlan_subnet
            ));
        }
        if old.enable_vlan != new.enable_vlan {
            ui::print_warning(&format!(
                "⚠️  enable_vlan changed from '{}' to '{}' but servers already exist!",
                old.enable_vlan, new.enable_vlan
            ));
            ui::print_info(
                "   VLAN configuration was applied based on original setting. Continuing with existing value.",
            );
        }
        if old.enable_firewall != new.enable_firewall {
            ui::print_warning(&format!(
                "⚠️  enable_firewall changed from '{}' to '{}' but servers already exist!",
                old.enable_firewall, new.enable_firewall
            ));
            ui::print_info(
                "   Firewall rules were applied based on original setting. Continuing with existing value.",
            );
        }
        Ok(())
    }

    /// Update VLAN config fields when safe (no servers exist).
    fn update_vlan_config(config: &mut InstallConfig, new_config: &InstallConfig) {
        if config.vlan_parent_interface != new_config.vlan_parent_interface {
            ui::print_info(&format!(
                "Updating VLAN interface: {} -> {}",
                config.vlan_parent_interface, new_config.vlan_parent_interface
            ));
            config
                .vlan_parent_interface
                .clone_from(&new_config.vlan_parent_interface);
        }
        if config.vlan_subnet != new_config.vlan_subnet {
            ui::print_info(&format!(
                "Updating VLAN subnet: {} -> {}",
                config.vlan_subnet, new_config.vlan_subnet
            ));
            config.vlan_subnet.clone_from(&new_config.vlan_subnet);
        }
        if config.enable_vlan != new_config.enable_vlan {
            ui::print_info(&format!(
                "Updating enable_vlan: {} -> {}",
                config.enable_vlan, new_config.enable_vlan
            ));
            config.enable_vlan = new_config.enable_vlan;
        }
        if config.enable_firewall != new_config.enable_firewall {
            ui::print_info(&format!(
                "Updating enable_firewall: {} -> {}",
                config.enable_firewall, new_config.enable_firewall
            ));
            config.enable_firewall = new_config.enable_firewall;
        }
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
            InstallStep::CreatingVLAN => {
                self.create_vlan().await?;
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
            InstallStep::BootstrappingOpenBao => {
                self.bootstrap_openbao().await?;
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

    async fn create_vlan(&mut self) -> Result<()> {
        // Skip if VLAN is disabled
        if !self.state.config.enable_vlan {
            ui::print_info("VLAN private networking disabled, skipping");
            return Ok(());
        }

        // Idempotency: check if VLAN was already created (retry scenario)
        let vid = if let Some(ref existing_vlan_id) = self.state.vlan_id {
            ui::print_info(&format!(
                "VLAN already created ({existing_vlan_id}), skipping creation"
            ));
            self.state.vlan_vid.context("VLAN VID missing from state")?
        } else {
            let orchestrator = BareMetalOrchestrator::new(&self.state.config).await?;
            let region = self
                .state
                .selected_region
                .as_ref()
                .context("Region not selected")?;

            // Create VLAN
            ui::print_info("Creating VLAN for private networking...");
            let (vlan_id, vid) = orchestrator.create_vlan(region).await?;
            self.state.set_vlan(vlan_id, vid)?;
            vid
        };

        // Allocate private IPs for all servers
        // Idempotency: skip servers that already have a private IP allocated
        let cp_id = self.state.control_plane.as_ref().map(|cp| cp.id.clone());
        if let Some(cp_id) = cp_id {
            if self.state.get_private_ip(&cp_id).is_some() {
                ui::print_info("Control plane already has private IP, skipping allocation");
            } else {
                let private_ip = self.state.allocate_next_private_ip()?;
                ui::print_info(&format!(
                    "Allocated private IP {private_ip} for control plane"
                ));
                self.state.set_private_ip(&cp_id, private_ip)?;
            }
        }

        let worker_ids: Vec<_> = self.state.workers.iter().map(|w| w.id.clone()).collect();
        for (i, worker_id) in worker_ids.iter().enumerate() {
            if self.state.get_private_ip(worker_id).is_some() {
                ui::print_info(&format!(
                    "Worker {i} already has private IP, skipping allocation"
                ));
            } else {
                let private_ip = self.state.allocate_next_private_ip()?;
                ui::print_info(&format!("Allocated private IP {private_ip} for worker {i}"));
                self.state.set_private_ip(worker_id, private_ip)?;
            }
        }

        ui::print_success(&format!(
            "VLAN created (VID: {vid}) with {} private IPs",
            self.state.private_ips.len()
        ));

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

        let mut config = metal::talos::BootstrapConfig::new(
            self.state.config.cluster_name.clone(),
            cp.ip.clone(),
        )
        .with_install_disk(&self.state.config.install_disk)
        .with_output_dir(&self.state.config.output_dir)
        .with_talos_version(&self.state.config.talos_version)
        .with_kube_proxy_mode("none") // We use Cilium
        .with_cilium_cni();

        // Add VLAN configuration if enabled
        if self.state.config.enable_vlan {
            if let Some(vlan_vid) = self.state.vlan_vid {
                // Get control plane's private IP
                let cp_private_ip = self
                    .state
                    .get_private_ip(&cp.id)
                    .cloned()
                    .unwrap_or_else(|| "10.8.0.1".to_string());

                // Extract prefix length from vlan_subnet (e.g., "10.8.0.0/24" -> "24")
                let prefix = self
                    .state
                    .config
                    .vlan_subnet
                    .split('/')
                    .nth(1)
                    .unwrap_or("24");

                let private_ip_cidr = format!("{cp_private_ip}/{prefix}");

                ui::print_info(&format!(
                    "Adding VLAN config: VID={vlan_vid}, Interface={}, IP={private_ip_cidr}",
                    self.state.config.vlan_parent_interface
                ));

                config = config
                    .with_vlan_interface(
                        vlan_vid,
                        &private_ip_cidr,
                        &self.state.config.vlan_parent_interface,
                    )
                    .with_private_network(&self.state.config.vlan_subnet);
            }
        }

        // Add firewall configuration if enabled
        if self.state.config.enable_firewall {
            // Firewall requires VLAN for proper private network isolation
            // Without VLAN, nodes communicate via public IPs and host-level
            // firewall rules would need to allow arbitrary public IP ranges
            if self.state.config.enable_vlan {
                // Get control plane's private IP for etcd rules
                let cp_private_ip = self
                    .state
                    .get_private_ip(&cp.id)
                    .cloned()
                    .unwrap_or_else(|| "10.8.0.1".to_string());

                let cluster_subnet = self.state.config.vlan_subnet.clone();

                ui::print_info(&format!(
                    "Adding firewall config: cluster_subnet={cluster_subnet}"
                ));

                // Apply firewall rules:
                // - Common rules (kubelet, apid, trustd, cilium) → all nodes
                // - Control plane rules (K8s API, etcd) → only controlplane.yaml
                config = config.with_ingress_firewall(&cluster_subnet, &[&cp_private_ip]);
            } else {
                ui::print_warning(
                    "⚠️  Firewall enabled but VLAN disabled - skipping host firewall configuration.",
                );
                ui::print_info(
                    "   Host-level firewall requires VLAN for private network isolation.",
                );
                ui::print_info("   Use Kubernetes NetworkPolicies or cloud firewall for security.");
            }
        }

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

        // Wait for K8s API port to be reachable (NOT full health check)
        // Full health check requires CNI, which we deploy next step
        ui::print_info("Waiting for Kubernetes API port to be reachable...");
        let timeout = Duration::from_secs(300); // 5 minutes
        metal::talos::wait_for_kubernetes_api_port(&cp.ip, timeout)?;

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
        // Single-region by default; pass true for multi-region clusters
        metal::stack::deploy_local_path_provisioner(kubeconfig_path, false)?;

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

        // Apply ClusterIssuer for cert-manager after it's synced
        // This is needed by Jaeger, QuestDB, and other operators for webhook certs
        ui::print_info("Applying cert-manager ClusterIssuer for operator webhooks...");
        self.apply_cluster_issuer(kubeconfig_path).await?;

        Ok(())
    }

    /// Apply the self-signed ClusterIssuer for cert-manager.
    ///
    /// This is required by operators like Jaeger and QuestDB that use cert-manager
    /// to generate webhook certificates.
    async fn apply_cluster_issuer(&self, kubeconfig: &Path) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        const CLUSTER_ISSUER_YAML: &str = r"---
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: selfsigned-issuer
  labels:
    app.kubernetes.io/name: cert-manager
    app.kubernetes.io/component: issuer
spec:
  selfSigned: {}
";

        // Wait for cert-manager CRDs to be available
        let start = std::time::Instant::now();
        let crd_timeout = Duration::from_secs(120);

        loop {
            let output = Command::new("kubectl")
                .args([
                    "--kubeconfig",
                    kubeconfig.to_str().unwrap_or_default(),
                    "get",
                    "crd",
                    "clusterissuers.cert-manager.io",
                ])
                .output()
                .context("Failed to check for cert-manager CRDs")?;

            if output.status.success() {
                break;
            }

            if start.elapsed() > crd_timeout {
                anyhow::bail!("Timeout waiting for cert-manager CRDs to be available");
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        // Apply the ClusterIssuer
        let mut child = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "apply",
                "-f",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(CLUSTER_ISSUER_YAML.as_bytes())
                .context("Failed to write ClusterIssuer YAML")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for kubectl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to apply ClusterIssuer: {stderr}");
        }

        ui::print_success("ClusterIssuer 'selfsigned-issuer' created");
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

    async fn bootstrap_openbao(&mut self) -> Result<()> {
        let kubeconfig_path = self
            .state
            .kubeconfig_path
            .as_ref()
            .context("Kubeconfig not found in state")?;

        ui::print_info("Bootstrapping OpenBao secrets management...");

        let mut bootstrap = crate::openbao::OpenBaoBootstrap::new(kubeconfig_path);
        bootstrap.bootstrap().await?;

        ui::print_success("OpenBao secrets bootstrap complete!");

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

/// Wait for any control plane node to be Ready using kubectl.
///
/// Note: We can't rely on hostname matching because Talos may use the hardware
/// name (e.g., `hardware-212s000865`) instead of the configured hostname.
async fn wait_for_node_ready_kubectl(
    kubeconfig: &Path,
    _node_name: &str, // Ignored - we check for any control-plane node
    timeout: Duration,
) -> Result<()> {
    use std::process::Command;
    use std::time::Instant;

    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for control plane node to be Ready");
        }

        // Check if any node with control-plane role is Ready
        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "get",
                "nodes",
                "-l",
                "node-role.kubernetes.io/control-plane",
                "-o",
                "jsonpath={.items[*].status.conditions[?(@.type=='Ready')].status}",
            ])
            .output()
            .context("Failed to run kubectl get nodes")?;

        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.contains("True") {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
