//! On-premises / Colocation provider implementation.
//!
//! This provider manages servers through a local inventory file and
//! uses IPMI/BMC for power management operations.

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use tokio::fs;
use tracing::{debug, info};

use super::models::{BmcConfig, Inventory, PowerAction, ServerEntry, ServerState};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Default inventory file path.
const DEFAULT_INVENTORY_PATH: &str = "~/.cto/onprem-inventory.yaml";

/// Default timeout for BMC operations.
const DEFAULT_BMC_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// On-premises bare metal provider.
#[derive(Clone)]
pub struct OnPrem {
    /// Path to inventory file.
    inventory_path: PathBuf,
    /// Default timeout for operations.
    #[allow(dead_code)]
    timeout_secs: u64,
}

impl OnPrem {
    /// Create a new on-premises provider.
    ///
    /// # Arguments
    /// * `inventory_path` - Path to the inventory YAML file
    ///
    /// # Errors
    /// Returns error if the path is invalid.
    pub fn new(inventory_path: Option<PathBuf>) -> Result<Self, ProviderError> {
        let path = inventory_path.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            PathBuf::from(DEFAULT_INVENTORY_PATH.replace('~', &home))
        });

        Ok(Self {
            inventory_path: path,
            timeout_secs: DEFAULT_BMC_TIMEOUT_SECS,
        })
    }

    /// Load inventory from file.
    async fn load_inventory(&self) -> Result<Inventory, ProviderError> {
        if !self.inventory_path.exists() {
            return Ok(Inventory::default());
        }

        let contents = fs::read_to_string(&self.inventory_path)
            .await
            .map_err(|e| ProviderError::Config(format!("Failed to read inventory: {e}")))?;

        serde_yaml::from_str(&contents)
            .map_err(|e| ProviderError::Config(format!("Failed to parse inventory: {e}")))
    }

    /// Save inventory to file.
    async fn save_inventory(&self, inventory: &Inventory) -> Result<(), ProviderError> {
        // Ensure parent directory exists
        if let Some(parent) = self.inventory_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ProviderError::Config(format!("Failed to create directory: {e}")))?;
        }

        let mut inventory = inventory.clone();
        inventory.updated_at = Some(chrono::Utc::now());

        let contents = serde_yaml::to_string(&inventory)
            .map_err(|e| ProviderError::Config(format!("Failed to serialize inventory: {e}")))?;

        fs::write(&self.inventory_path, contents)
            .await
            .map_err(|e| ProviderError::Config(format!("Failed to write inventory: {e}")))?;

        Ok(())
    }

    /// Execute IPMI command.
    async fn ipmi_command(&self, bmc: &BmcConfig, command: &str) -> Result<String, ProviderError> {
        let port = bmc.port.unwrap_or(623);
        let port_str = port.to_string();

        // Build ipmitool command
        let password = bmc.password.as_deref().unwrap_or("");
        let mut args = vec![
            "-I",
            "lanplus",
            "-H",
            &bmc.address,
            "-p",
            &port_str,
            "-U",
            &bmc.username,
            "-P",
            password,
        ];

        args.extend(command.split_whitespace());

        debug!(
            bmc_address = %bmc.address,
            command = %command,
            "Executing IPMI command"
        );

        // Execute the command
        let output = tokio::process::Command::new("ipmitool")
            .args(&args)
            .output()
            .await
            .map_err(|e| ProviderError::Config(format!("Failed to execute ipmitool: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ProviderError::Api {
                status: 500,
                message: format!("IPMI command failed: {stderr}"),
            })
        }
    }

    /// Execute power action via BMC.
    async fn power_action(
        &self,
        bmc: &BmcConfig,
        action: PowerAction,
    ) -> Result<(), ProviderError> {
        let command = match action {
            PowerAction::PowerOn => "power on",
            PowerAction::PowerOff => "power off",
            PowerAction::Reset => "power reset",
            PowerAction::Cycle => "power cycle",
            PowerAction::Status => "power status",
        };

        self.ipmi_command(bmc, command).await?;
        Ok(())
    }

    /// Set boot device via BMC (for PXE boot).
    async fn set_boot_device(&self, bmc: &BmcConfig, device: &str) -> Result<(), ProviderError> {
        // Set boot device for next boot only
        let command = format!("chassis bootdev {device} options=efiboot");
        self.ipmi_command(bmc, &command).await?;
        Ok(())
    }

    /// Convert inventory entry to Server type.
    fn to_server(entry: &ServerEntry) -> Server {
        let status = match entry.status {
            ServerState::Ready => ServerStatus::On,
            ServerState::Provisioning => ServerStatus::Deploying,
            ServerState::PoweredOff | ServerState::Maintenance => ServerStatus::Off,
            ServerState::Decommissioning => ServerStatus::Deleting,
            ServerState::Unknown => ServerStatus::Unknown,
        };

        Server {
            id: entry.id.clone(),
            hostname: entry.hostname.clone(),
            status,
            ipv4: entry.ipv4.clone(),
            ipv6: entry.ipv6.clone(),
            plan: entry.plan.clone(),
            region: entry.location.clone(),
            created_at: entry.created_at,
        }
    }
}

#[async_trait]
impl Provider for OnPrem {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Adding server to inventory"
        );

        // For on-prem, "create" means adding an existing server to inventory
        let mut inventory = self.load_inventory().await?;

        // Check if server already exists
        if inventory.servers.iter().any(|s| s.id == req.hostname) {
            return Err(ProviderError::Config(format!(
                "Server '{}' already exists in inventory",
                req.hostname
            )));
        }

        let entry = ServerEntry {
            id: uuid::Uuid::new_v4().to_string(),
            hostname: req.hostname.clone(),
            status: ServerState::Provisioning,
            ipv4: None,
            ipv6: None,
            plan: req.plan,
            location: req.region,
            bmc: None,
            specs: None,
            ssh: None,
            network: None,
            tags: vec![],
            created_at: Some(chrono::Utc::now()),
            notes: Some("Added via CTO Platform".to_string()),
        };

        let server = Self::to_server(&entry);
        inventory.servers.push(entry);
        self.save_inventory(&inventory).await?;

        info!(
            server_id = %server.id,
            "Server added to inventory"
        );

        Ok(server)
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let inventory = self.load_inventory().await?;

        inventory
            .servers
            .iter()
            .find(|s| s.id == id || s.hostname == id)
            .map(Self::to_server)
            .ok_or_else(|| ProviderError::NotFound(format!("Server not found: {id}")))
    }

    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError> {
        info!(server_id = %id, timeout_secs, "Waiting for server to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let server = self.get_server(id).await?;

            debug!(
                server_id = %id,
                status = %server.status,
                elapsed_secs = start.elapsed().as_secs(),
                "Checking server status"
            );

            if server.status == ServerStatus::On {
                info!(server_id = %id, "Server is ready");
                return Ok(server);
            }

            if start.elapsed() > timeout {
                return Err(ProviderError::Timeout(timeout_secs));
            }

            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    async fn reinstall_ipxe(
        &self,
        id: &str,
        req: ReinstallIpxeRequest,
    ) -> Result<(), ProviderError> {
        info!(
            server_id = %id,
            ipxe_url = %req.ipxe_url,
            hostname = %req.hostname,
            "Triggering iPXE reinstall via BMC"
        );

        let inventory = self.load_inventory().await?;
        let entry = inventory
            .servers
            .iter()
            .find(|s| s.id == id || s.hostname == id)
            .ok_or_else(|| ProviderError::NotFound(format!("Server not found: {id}")))?;

        let bmc = entry.bmc.as_ref().ok_or_else(|| {
            ProviderError::Config(format!("No BMC configuration for server: {id}"))
        })?;

        // Set boot device to PXE
        self.set_boot_device(bmc, "pxe").await?;
        info!(server_id = %id, "Boot device set to PXE");

        // Reset the server to boot into PXE
        self.power_action(bmc, PowerAction::Reset).await?;
        info!(server_id = %id, "Server reset triggered");

        // Note: The iPXE URL would typically be served by a TFTP/HTTP server
        // configured in the PXE environment, not directly via IPMI

        Ok(())
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Removing server from inventory");

        let mut inventory = self.load_inventory().await?;
        let original_len = inventory.servers.len();

        inventory.servers.retain(|s| s.id != id && s.hostname != id);

        if inventory.servers.len() == original_len {
            return Err(ProviderError::NotFound(format!("Server not found: {id}")));
        }

        self.save_inventory(&inventory).await?;
        info!(server_id = %id, "Server removed from inventory");

        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let inventory = self.load_inventory().await?;
        Ok(inventory.servers.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_mapping() {
        let entry = ServerEntry {
            id: "server-001".to_string(),
            hostname: "node1.local".to_string(),
            status: ServerState::Ready,
            ipv4: Some("192.168.1.100".to_string()),
            ipv6: None,
            plan: "Dell PowerEdge R640".to_string(),
            location: "Rack 42, DC1".to_string(),
            bmc: None,
            specs: None,
            ssh: None,
            network: None,
            tags: vec![],
            created_at: None,
            notes: None,
        };

        let converted = OnPrem::to_server(&entry);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "server-001");
        assert_eq!(converted.hostname, "node1.local");
        assert_eq!(converted.ipv4, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_inventory_serialization() {
        let inventory = Inventory {
            servers: vec![ServerEntry {
                id: "test-1".to_string(),
                hostname: "test-host".to_string(),
                status: ServerState::Ready,
                ipv4: Some("10.0.0.1".to_string()),
                ipv6: None,
                plan: "Custom".to_string(),
                location: "Lab".to_string(),
                bmc: Some(BmcConfig {
                    address: "10.0.0.100".to_string(),
                    port: Some(623),
                    username: "admin".to_string(),
                    password: None,
                    bmc_type: "ipmi".to_string(),
                    web_url: None,
                }),
                specs: None,
                ssh: None,
                network: None,
                tags: vec!["kubernetes".to_string()],
                created_at: None,
                notes: None,
            }],
            updated_at: None,
        };

        let yaml = serde_yaml::to_string(&inventory).unwrap();
        assert!(yaml.contains("test-host"));
        assert!(yaml.contains("10.0.0.1"));
    }
}
