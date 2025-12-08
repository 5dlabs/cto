//! On-premises inventory models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Inventory types
// ============================================================================

/// Server inventory file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    /// List of servers.
    pub servers: Vec<ServerEntry>,
    /// Last updated timestamp.
    pub updated_at: Option<DateTime<Utc>>,
}

/// Server entry in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    /// Unique server identifier.
    pub id: String,
    /// Server hostname.
    pub hostname: String,
    /// Server status.
    pub status: ServerState,
    /// Primary IPv4 address.
    pub ipv4: Option<String>,
    /// Primary IPv6 address.
    pub ipv6: Option<String>,
    /// Server plan/type (custom label).
    pub plan: String,
    /// Location (rack, room, site).
    pub location: String,
    /// BMC/IPMI configuration.
    pub bmc: Option<BmcConfig>,
    /// Hardware specifications.
    pub specs: Option<HardwareSpecs>,
    /// SSH access configuration.
    pub ssh: Option<SshConfig>,
    /// Network configuration.
    pub network: Option<NetworkConfig>,
    /// Tags/labels.
    #[serde(default)]
    pub tags: Vec<String>,
    /// When the server was added to inventory.
    pub created_at: Option<DateTime<Utc>>,
    /// Additional notes.
    pub notes: Option<String>,
}

/// Server state in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerState {
    /// Server is available and running.
    Ready,
    /// Server is being provisioned.
    Provisioning,
    /// Server is powered off.
    PoweredOff,
    /// Server is in maintenance mode.
    Maintenance,
    /// Server is being decommissioned.
    Decommissioning,
    /// Unknown state.
    Unknown,
}

// ============================================================================
// BMC/IPMI types
// ============================================================================

/// BMC (IPMI/iDRAC/iLO) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmcConfig {
    /// BMC IP address or hostname.
    pub address: String,
    /// BMC port (default: 623 for IPMI).
    pub port: Option<u16>,
    /// Username for BMC access.
    pub username: String,
    /// Password for BMC access (should be encrypted/vaulted in production).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// BMC type: "ipmi", "idrac", "ilo", "redfish".
    pub bmc_type: String,
    /// Web interface URL (if available).
    pub web_url: Option<String>,
}

// ============================================================================
// Hardware types
// ============================================================================

/// Hardware specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    /// CPU model.
    pub cpu: Option<String>,
    /// Number of CPU cores.
    pub cores: Option<i32>,
    /// Number of CPU threads.
    pub threads: Option<i32>,
    /// Memory in GB.
    pub memory_gb: Option<i64>,
    /// Storage configuration.
    pub storage: Option<Vec<StorageDevice>>,
    /// Network interfaces.
    pub network_interfaces: Option<Vec<NetworkInterface>>,
    /// Manufacturer.
    pub manufacturer: Option<String>,
    /// Model.
    pub model: Option<String>,
    /// Serial number.
    pub serial_number: Option<String>,
}

/// Storage device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    /// Device name/path (e.g., "/dev/sda").
    pub device: String,
    /// Device type: "ssd", "nvme", "hdd".
    pub device_type: String,
    /// Capacity in GB.
    pub capacity_gb: i64,
    /// Model.
    pub model: Option<String>,
}

/// Network interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "eno1").
    pub name: String,
    /// MAC address.
    pub mac_address: String,
    /// Speed in Mbps.
    pub speed_mbps: Option<i64>,
    /// Link state.
    pub link_up: Option<bool>,
}

// ============================================================================
// SSH types
// ============================================================================

/// SSH access configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    /// SSH port (default: 22).
    pub port: Option<u16>,
    /// SSH user.
    pub user: String,
    /// SSH key path.
    pub key_path: Option<String>,
    /// Known host fingerprint.
    pub host_key_fingerprint: Option<String>,
}

// ============================================================================
// Network types
// ============================================================================

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// VLAN ID.
    pub vlan_id: Option<i32>,
    /// Subnet.
    pub subnet: Option<String>,
    /// Gateway.
    pub gateway: Option<String>,
    /// DNS servers.
    pub dns_servers: Option<Vec<String>>,
}

// ============================================================================
// Action types
// ============================================================================

/// Power action to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerAction {
    /// Power on the server.
    PowerOn,
    /// Power off the server.
    PowerOff,
    /// Reset/reboot the server.
    Reset,
    /// Power cycle (off then on).
    Cycle,
    /// Check current power status.
    Status,
}

/// Boot source for PXE/iPXE.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootSource {
    /// Boot from disk (default).
    Disk,
    /// Boot from PXE/network.
    Pxe,
    /// Boot from BIOS setup.
    Bios,
    /// Boot from CD/DVD.
    Cdrom,
}

/// Provisioning request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionRequest {
    /// Server ID.
    pub server_id: String,
    /// iPXE URL to boot from.
    pub ipxe_url: String,
    /// New hostname.
    pub hostname: String,
    /// SSH keys to install.
    #[serde(default)]
    pub ssh_keys: Vec<String>,
}


