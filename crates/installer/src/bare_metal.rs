//! Bare metal orchestrator.
//!
//! This module wraps the metal library for infrastructure operations,
//! providing higher-level abstractions for the installer.

use anyhow::{Context, Result};
use std::env;
use tracing::{info, warn};

use crate::config::{BareMetalProvider, InstallConfig};
use crate::ui;

use metal::inventory::InventoryManager;
use metal::providers::latitude::Latitude;
use metal::providers::{CreateServerRequest, Provider, ReinstallIpxeRequest};
use metal::talos::{TalosVersion, DEFAULT_SCHEMATIC_ID};

/// Result of server creation.
#[derive(Debug, Clone)]
pub struct CreatedServer {
    /// Server ID from the provider.
    pub id: String,
    /// Server IP address.
    pub ip: String,
    /// Server hostname.
    pub hostname: String,
}

/// Bare metal orchestrator for server provisioning.
pub struct BareMetalOrchestrator {
    /// The provider implementation.
    provider: Box<dyn Provider>,
    /// Installation configuration.
    config: InstallConfig,
}

impl BareMetalOrchestrator {
    /// Create a new bare metal orchestrator.
    ///
    /// # Errors
    ///
    /// Returns an error if credentials are missing or provider initialization fails.
    pub async fn new(config: &InstallConfig) -> Result<Self> {
        let provider: Box<dyn Provider> = match config.provider {
            BareMetalProvider::Latitude => {
                let (api_key, project_id) = get_latitude_credentials()?;
                let latitude = Latitude::new(&api_key, &project_id)
                    .context("Failed to create Latitude provider")?;
                Box::new(latitude)
            }
        };

        Ok(Self {
            provider,
            config: config.clone(),
        })
    }

    /// Select the best region based on configuration.
    ///
    /// If auto_region is enabled, queries inventory for stock availability.
    /// Otherwise, uses the configured region.
    ///
    /// # Errors
    ///
    /// Returns an error if no suitable region is available.
    pub async fn select_region(&self) -> Result<String> {
        if !self.config.auto_region {
            ui::print_info(&format!("Using configured region: {}", self.config.region));
            return Ok(self.config.region.clone());
        }

        ui::print_info("Auto-selecting region based on stock availability...");

        // Get Latitude provider for inventory queries
        let (api_key, project_id) = get_latitude_credentials()?;
        let latitude = Latitude::new(&api_key, &project_id)
            .context("Failed to create Latitude provider for inventory")?;

        let mut inventory = InventoryManager::new(latitude);

        // Try to find a region with stock for our plan
        let preferred: Vec<&str> = self
            .config
            .fallback_regions
            .iter()
            .map(String::as_str)
            .collect();

        match inventory
            .find_best_region_from_preferred(&self.config.cp_plan, &preferred, true)
            .await
        {
            Ok(region) => {
                ui::print_success(&format!("Auto-selected region: {region}"));
                Ok(region)
            }
            Err(e) => {
                ui::print_warning(&format!("Auto-region selection failed: {e}"));
                ui::print_info(&format!(
                    "Falling back to configured region: {}",
                    self.config.region
                ));
                Ok(self.config.region.clone())
            }
        }
    }

    /// Create servers for the cluster.
    ///
    /// Returns control plane server and worker servers.
    ///
    /// # Errors
    ///
    /// Returns an error if server creation fails.
    pub async fn create_servers(
        &self,
        region: &str,
    ) -> Result<(CreatedServer, Vec<CreatedServer>)> {
        let cp_hostname = self.config.cp_hostname();
        let worker_hostnames = self.config.worker_hostnames();

        // Create control plane
        ui::print_info(&format!(
            "Creating control plane server: {cp_hostname} in {region}"
        ));
        let cp_server = self
            .provider
            .create_server(CreateServerRequest {
                hostname: cp_hostname.clone(),
                plan: self.config.cp_plan.clone(),
                region: region.to_string(),
                os: "ubuntu_24_04_x64_lts".to_string(),
                ssh_keys: self.config.ssh_keys.clone(),
            })
            .await
            .context("Failed to create control plane server")?;

        let cp = CreatedServer {
            id: cp_server.id,
            ip: cp_server.ipv4.unwrap_or_default(),
            hostname: cp_hostname,
        };

        // Create workers in parallel
        let mut workers = Vec::new();
        for hostname in &worker_hostnames {
            ui::print_info(&format!("Creating worker server: {hostname} in {region}"));
            let server = self
                .provider
                .create_server(CreateServerRequest {
                    hostname: hostname.clone(),
                    plan: self.config.worker_plan.clone(),
                    region: region.to_string(),
                    os: "ubuntu_24_04_x64_lts".to_string(),
                    ssh_keys: self.config.ssh_keys.clone(),
                })
                .await
                .with_context(|| format!("Failed to create worker server: {hostname}"))?;

            workers.push(CreatedServer {
                id: server.id,
                ip: server.ipv4.unwrap_or_default(),
                hostname: hostname.clone(),
            });
        }

        ui::print_success(&format!(
            "Created {} server(s) in {region}",
            1 + workers.len()
        ));

        Ok((cp, workers))
    }

    /// Wait for all servers to be ready.
    ///
    /// # Errors
    ///
    /// Returns an error if waiting times out.
    pub async fn wait_servers_ready(&self, cp_id: &str, worker_ids: &[String]) -> Result<()> {
        let timeout_secs = 1800; // 30 minutes

        // Wait for control plane
        ui::print_info("Waiting for control plane server to be ready...");
        let cp = self
            .provider
            .wait_ready(cp_id, timeout_secs)
            .await
            .context("Control plane server failed to become ready")?;
        ui::print_success(&format!(
            "Control plane ready: {} ({})",
            cp.hostname,
            cp.ipv4.as_deref().unwrap_or("no IP")
        ));

        // Wait for workers
        for (i, worker_id) in worker_ids.iter().enumerate() {
            ui::print_info(&format!("Waiting for worker {} to be ready...", i + 1));
            let worker = self
                .provider
                .wait_ready(worker_id, timeout_secs)
                .await
                .with_context(|| format!("Worker {} failed to become ready", i + 1))?;
            ui::print_success(&format!(
                "Worker {} ready: {} ({})",
                i + 1,
                worker.hostname,
                worker.ipv4.as_deref().unwrap_or("no IP")
            ));
        }

        Ok(())
    }

    /// Trigger Talos iPXE boot on all servers and wait for them to come back online.
    ///
    /// This triggers a reinstall with Talos iPXE, then polls the Latitude API
    /// until each server's status returns to "on" before proceeding.
    ///
    /// # Errors
    ///
    /// Returns an error if iPXE reinstall fails or servers don't come back online.
    pub async fn boot_talos(&self, cp_id: &str, worker_ids: &[String]) -> Result<()> {
        // Generate iPXE URL
        let talos_version = TalosVersion::new(&self.config.talos_version, DEFAULT_SCHEMATIC_ID);
        let ipxe_url = talos_version.ipxe_url_amd64();

        info!(ipxe_url = %ipxe_url, "Triggering Talos iPXE boot");
        ui::print_info(&format!("iPXE URL: {ipxe_url}"));

        // Boot control plane
        let cp_hostname = self.config.cp_hostname();
        ui::print_info(&format!(
            "Triggering Talos boot on control plane ({cp_hostname})..."
        ));
        self.provider
            .reinstall_ipxe(
                cp_id,
                ReinstallIpxeRequest {
                    hostname: cp_hostname.clone(),
                    ipxe_url: ipxe_url.clone(),
                },
            )
            .await
            .context("Failed to trigger Talos boot on control plane")?;

        // Boot workers
        let worker_hostnames = self.config.worker_hostnames();
        for (i, worker_id) in worker_ids.iter().enumerate() {
            let hostname = worker_hostnames
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("{}-worker{}", self.config.cluster_name, i + 1));

            ui::print_info(&format!(
                "Triggering Talos boot on worker {} ({hostname})...",
                i + 1
            ));
            self.provider
                .reinstall_ipxe(
                    worker_id,
                    ReinstallIpxeRequest {
                        hostname,
                        ipxe_url: ipxe_url.clone(),
                    },
                )
                .await
                .with_context(|| format!("Failed to trigger Talos boot on worker {}", i + 1))?;
        }

        ui::print_success("Talos iPXE boot triggered on all servers");

        // Wait for ALL servers to return to "on" status via Latitude API
        // This ensures the reinstall/reboot has completed before we try to connect to Talos
        ui::print_info("Waiting for ALL servers to come back online (polling Latitude API)...");
        ui::print_info("This ensures reinstall is complete before attempting Talos connection.");
        let timeout_secs = 900; // 15 minutes for reinstall

        // Wait for control plane first
        ui::print_info(&format!("  → Control plane ({cp_id})..."));
        self.provider
            .wait_ready(cp_id, timeout_secs)
            .await
            .context("Control plane did not come back online after Talos reinstall")?;
        ui::print_success("  ✓ Control plane is online (status: on)");

        // Wait for all workers
        for (i, worker_id) in worker_ids.iter().enumerate() {
            ui::print_info(&format!("  → Worker {} ({worker_id})...", i + 1));
            self.provider
                .wait_ready(worker_id, timeout_secs)
                .await
                .with_context(|| {
                    format!(
                        "Worker {} did not come back online after Talos reinstall",
                        i + 1
                    )
                })?;
            ui::print_success(&format!("  ✓ Worker {} is online (status: on)", i + 1));
        }

        ui::print_success("All servers are online (Latitude API status: on)");
        ui::print_info("Now safe to proceed with Talos API polling...");

        Ok(())
    }

    /// Create a VLAN for private networking.
    ///
    /// Returns the VLAN ID and VID (for OS configuration).
    ///
    /// # Errors
    ///
    /// Returns an error if VLAN creation fails.
    pub async fn create_vlan(&self, region: &str) -> Result<(String, u16)> {
        // Currently only Latitude supports VLANs
        match self.config.provider {
            BareMetalProvider::Latitude => {
                let (api_key, project_id) = get_latitude_credentials()?;
                let latitude = Latitude::new(&api_key, &project_id)
                    .context("Failed to create Latitude provider for VLAN")?;

                let description = format!("{} Private Network", self.config.cluster_name);

                info!(region = %region, description = %description, "Creating VLAN");
                ui::print_info(&format!("Creating VLAN in {region}..."));

                let vlan = latitude
                    .create_virtual_network(region, &description)
                    .await
                    .context("Failed to create VLAN")?;

                let vlan_id = vlan.id;
                let vlan_vid = vlan.attributes.vid;

                ui::print_success(&format!(
                    "Created VLAN {vlan_id} (VID: {vlan_vid}) in {region}"
                ));

                // Note: Server assignment to VLAN via API is not currently exposed by Latitude.
                // The VLAN interface will be configured in Talos machine config with static IPs.
                // Manual assignment via Latitude dashboard may be needed for full L2 connectivity.
                ui::print_info("Note: Configure Talos with VLAN interface for private networking.");
                ui::print_info(&format!(
                    "      VLAN VID: {vlan_vid}, Subnet: {}",
                    self.config.vlan_subnet
                ));

                Ok((vlan_id, vlan_vid))
            }
        }
    }

    /// Delete a VLAN.
    ///
    /// # Errors
    ///
    /// Returns an error if VLAN deletion fails.
    #[allow(dead_code)] // Will be used when destroy command is added
    pub async fn delete_vlan(&self, vlan_id: &str) -> Result<()> {
        match self.config.provider {
            BareMetalProvider::Latitude => {
                let (api_key, project_id) = get_latitude_credentials()?;
                let latitude = Latitude::new(&api_key, &project_id)
                    .context("Failed to create Latitude provider for VLAN deletion")?;

                info!(vlan_id = %vlan_id, "Deleting VLAN");
                ui::print_info(&format!("Deleting VLAN {vlan_id}..."));

                latitude
                    .delete_virtual_network(vlan_id)
                    .await
                    .context("Failed to delete VLAN")?;

                ui::print_success(&format!("VLAN {vlan_id} deleted"));

                Ok(())
            }
        }
    }

    /// Delete a server.
    ///
    /// # Errors
    ///
    /// Returns an error if server deletion fails.
    #[allow(dead_code)] // Will be used when destroy command is added
    pub async fn delete_server(&self, server_id: &str) -> Result<()> {
        info!(server_id = %server_id, "Deleting server");
        ui::print_info(&format!("Deleting server {server_id}..."));

        self.provider
            .delete_server(server_id)
            .await
            .context("Failed to delete server")?;

        ui::print_success(&format!("Server {server_id} deleted"));
        Ok(())
    }
}

/// Get Latitude credentials from environment.
fn get_latitude_credentials() -> Result<(String, String)> {
    // First try direct environment variables
    if let (Ok(api_key), Ok(project_id)) = (
        env::var("LATITUDE_API_KEY"),
        env::var("LATITUDE_PROJECT_ID"),
    ) {
        info!("Using Latitude credentials from environment variables");
        return Ok((api_key, project_id));
    }

    // Try 1Password CLI
    if let Ok(credentials) = get_credentials_from_1password() {
        info!("Using Latitude credentials from 1Password");
        return Ok(credentials);
    }

    Err(anyhow::anyhow!(
        "Latitude credentials not found. Set LATITUDE_API_KEY and LATITUDE_PROJECT_ID \
         environment variables, or configure 1Password CLI with OP_VAULT and OP_LATITUDE_ITEM."
    ))
}

/// Try to get credentials from 1Password CLI.
fn get_credentials_from_1password() -> Result<(String, String)> {
    use std::process::Command;

    let vault = env::var("OP_VAULT").unwrap_or_else(|_| "Private".to_string());
    let item = env::var("OP_LATITUDE_ITEM").unwrap_or_else(|_| "Latitude".to_string());

    // Check if op is available
    let op_check = Command::new("op").arg("--version").output();
    if op_check.is_err() {
        return Err(anyhow::anyhow!("1Password CLI (op) not found"));
    }

    // Get API key
    let api_key_output = Command::new("op")
        .args(["read", &format!("op://{vault}/{item}/api_key")])
        .output()
        .context("Failed to run op read for api_key")?;

    if !api_key_output.status.success() {
        warn!("Failed to read Latitude API key from 1Password");
        return Err(anyhow::anyhow!("Failed to read API key from 1Password"));
    }

    let api_key = String::from_utf8(api_key_output.stdout)
        .context("Invalid UTF-8 in API key")?
        .trim()
        .to_string();

    // Get project ID
    let project_id_output = Command::new("op")
        .args(["read", &format!("op://{vault}/{item}/project_id")])
        .output()
        .context("Failed to run op read for project_id")?;

    if !project_id_output.status.success() {
        warn!("Failed to read Latitude project ID from 1Password");
        return Err(anyhow::anyhow!("Failed to read project ID from 1Password"));
    }

    let project_id = String::from_utf8(project_id_output.stdout)
        .context("Invalid UTF-8 in project ID")?
        .trim()
        .to_string();

    if api_key.is_empty() || project_id.is_empty() {
        return Err(anyhow::anyhow!("Empty credentials from 1Password"));
    }

    Ok((api_key, project_id))
}
