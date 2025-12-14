//! Scaleway Elastic Metal API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// Server types
// ============================================================================

/// Elastic Metal server.
#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    /// Server ID.
    pub id: String,
    /// Server name.
    pub name: String,
    /// Organization ID.
    pub organization_id: String,
    /// Project ID.
    pub project_id: String,
    /// Zone.
    pub zone: String,
    /// Server status.
    pub status: String,
    /// Offer (plan) information.
    pub offer: Option<Offer>,
    /// IP addresses.
    pub ips: Vec<Ip>,
    /// Installation info.
    pub install: Option<InstallInfo>,
    /// Tags.
    pub tags: Vec<String>,
    /// Created at.
    pub created_at: Option<String>,
    /// Updated at.
    pub updated_at: Option<String>,
}

/// IP address information.
#[derive(Debug, Clone, Deserialize)]
pub struct Ip {
    /// IP ID.
    pub id: String,
    /// IP address.
    pub address: String,
    /// Reverse DNS.
    pub reverse: Option<String>,
    /// IP version (IPv4, IPv6).
    pub version: String,
}

/// Server offer (plan).
#[derive(Debug, Clone, Deserialize)]
pub struct Offer {
    /// Offer ID.
    pub id: String,
    /// Offer name.
    pub name: String,
    /// CPU specs.
    pub cpu: Option<CpuSpec>,
    /// Memory in bytes.
    pub memory: Option<i64>,
    /// Disk specs.
    pub disk: Option<Vec<DiskSpec>>,
    /// Bandwidth in bps.
    pub bandwidth: Option<i64>,
}

/// CPU specification.
#[derive(Debug, Clone, Deserialize)]
pub struct CpuSpec {
    /// CPU name.
    pub name: String,
    /// Core count.
    pub core_count: i32,
    /// Thread count.
    pub thread_count: i32,
    /// Frequency in MHz.
    pub frequency: i64,
}

/// Disk specification.
#[derive(Debug, Clone, Deserialize)]
pub struct DiskSpec {
    /// Disk type.
    #[serde(rename = "type")]
    pub disk_type: String,
    /// Disk capacity in bytes.
    pub capacity: i64,
}

/// Installation information.
#[derive(Debug, Clone, Deserialize)]
pub struct InstallInfo {
    /// OS ID.
    pub os_id: String,
    /// Hostname.
    pub hostname: String,
    /// SSH key IDs.
    pub ssh_key_ids: Vec<String>,
    /// Status.
    pub status: String,
}

/// Server list response.
#[derive(Debug, Deserialize)]
pub struct ServerListResponse {
    /// List of servers.
    pub servers: Vec<Server>,
    /// Total count.
    pub total_count: Option<i32>,
}

/// Single server response.
#[derive(Debug, Deserialize)]
pub struct ServerResponse {
    /// Server details.
    pub server: Server,
}

// ============================================================================
// Create Server types
// ============================================================================

/// Request body for creating a server.
#[derive(Debug, Serialize)]
pub struct CreateServerRequest {
    /// Offer ID (plan).
    pub offer_id: String,
    /// Server name.
    pub name: String,
    /// Project ID.
    pub project_id: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Install configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<InstallRequest>,
}

/// Installation request configuration.
#[derive(Debug, Serialize)]
pub struct InstallRequest {
    /// OS ID.
    pub os_id: String,
    /// Hostname.
    pub hostname: String,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_key_ids: Vec<String>,
    /// User data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Service user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_user: Option<String>,
    /// Service password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_password: Option<String>,
}

// ============================================================================
// Reinstall types
// ============================================================================

/// Request for reinstalling a server.
#[derive(Debug, Serialize)]
pub struct ReinstallRequest {
    /// OS ID.
    pub os_id: String,
    /// Hostname.
    pub hostname: String,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_key_ids: Vec<String>,
}

// ============================================================================
// Action types
// ============================================================================

/// Server action request.
#[derive(Debug, Serialize)]
pub struct ActionRequest {
    /// Action type: "poweron", "poweroff", "reboot".
    pub action: String,
}

// ============================================================================
// BMC (IPMI) types
// ============================================================================

/// BMC access information.
#[derive(Debug, Clone, Deserialize)]
pub struct BmcAccess {
    /// BMC URL.
    pub url: String,
    /// Login.
    pub login: String,
    /// Password.
    pub password: String,
    /// Expires at.
    pub expires_at: String,
}

/// Start BMC access request.
#[derive(Debug, Serialize)]
pub struct StartBmcAccessRequest {
    /// IP to allow access from.
    pub ip: String,
}

// ============================================================================
// OS types
// ============================================================================

/// Operating system.
#[derive(Debug, Clone, Deserialize)]
pub struct Os {
    /// OS ID.
    pub id: String,
    /// OS name.
    pub name: String,
    /// OS version.
    pub version: String,
}

/// OS list response.
#[derive(Debug, Deserialize)]
pub struct OsListResponse {
    /// List of operating systems.
    pub os: Vec<Os>,
    /// Total count.
    pub total_count: Option<i32>,
}

// ============================================================================
// SSH Key types
// ============================================================================

/// SSH key.
#[derive(Debug, Clone, Deserialize)]
pub struct SshKey {
    /// Key ID.
    pub id: String,
    /// Key name.
    pub name: String,
    /// Public key.
    pub public_key: String,
    /// Fingerprint.
    pub fingerprint: String,
    /// Created at.
    pub created_at: Option<String>,
}

/// Create SSH key request.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyRequest {
    /// Key name.
    pub name: String,
    /// Public key content.
    pub public_key: String,
    /// Project ID.
    pub project_id: String,
}
