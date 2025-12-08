//! Vultr API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// Server (Bare Metal) types
// ============================================================================

/// Bare metal instance from API.
#[derive(Debug, Clone, Deserialize)]
pub struct BareMetalInstance {
    /// Instance ID.
    pub id: String,
    /// Instance label.
    pub label: String,
    /// Main IP address.
    pub main_ip: String,
    /// IPv6 network.
    pub v6_network: Option<String>,
    /// CPU count.
    pub cpu_count: i32,
    /// RAM in MB.
    pub ram: String,
    /// Disk size.
    pub disk: String,
    /// Region ID.
    pub region: String,
    /// Plan ID.
    pub plan: String,
    /// Status: "active", "pending", "suspended", "resizing".
    pub status: String,
    /// Power status: "running", "stopped".
    pub power_status: String,
    /// Server state: "ok", "locked", "installingbooting", etc.
    pub server_state: String,
    /// Operating system.
    pub os: String,
    /// Date created.
    pub date_created: String,
    /// Netmask v4.
    pub netmask_v4: Option<String>,
    /// Gateway v4.
    pub gateway_v4: Option<String>,
    /// MAC address.
    pub mac_address: Option<String>,
    /// OS ID.
    pub os_id: i32,
    /// App ID (if from marketplace).
    pub app_id: i32,
    /// Image ID (if custom image).
    pub image_id: Option<String>,
    /// Tags.
    pub tags: Vec<String>,
}

/// Bare metal list response.
#[derive(Debug, Deserialize)]
pub struct BareMetalListResponse {
    /// List of instances.
    pub bare_metals: Vec<BareMetalInstance>,
    /// Pagination metadata.
    pub meta: Option<PaginationMeta>,
}

/// Single bare metal response.
#[derive(Debug, Deserialize)]
pub struct BareMetalResponse {
    /// Instance details.
    pub bare_metal: BareMetalInstance,
}

/// Pagination metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationMeta {
    /// Total count.
    pub total: i32,
    /// Links.
    pub links: Option<PaginationLinks>,
}

/// Pagination links.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationLinks {
    /// Next page URL.
    pub next: Option<String>,
    /// Previous page URL.
    pub prev: Option<String>,
}

// ============================================================================
// Create Server types
// ============================================================================

/// Request body for creating a bare metal instance.
#[derive(Debug, Serialize)]
pub struct CreateBareMetalRequest {
    /// Region ID.
    pub region: String,
    /// Plan ID.
    pub plan: String,
    /// OS ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_id: Option<i32>,
    /// Image ID (for custom images).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
    /// Instance label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sshkey_id: Vec<String>,
    /// Enable IPv6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_ipv6: Option<bool>,
    /// User data for cloud-init.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
    /// Tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Script ID for startup script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<String>,
    /// App ID for marketplace app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<i32>,
    /// Hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

// ============================================================================
// Reinstall types
// ============================================================================

/// Request for reinstalling a bare metal instance.
#[derive(Debug, Serialize)]
pub struct ReinstallRequest {
    /// OS ID (or use "iPXE" for iPXE boot).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_id: Option<i32>,
    /// Hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

// ============================================================================
// iPXE types
// ============================================================================

/// iPXE chain URL request.
#[derive(Debug, Serialize)]
pub struct IpxeChainRequest {
    /// iPXE chain URL.
    pub chain_url: String,
}

// ============================================================================
// Power types
// ============================================================================

/// Halt/reboot request.
#[derive(Debug, Serialize)]
pub struct HaltRequest {}

/// Reboot request.
#[derive(Debug, Serialize)]
pub struct RebootRequest {}

/// Start request.
#[derive(Debug, Serialize)]
pub struct StartRequest {}

// ============================================================================
// Region types
// ============================================================================

/// Region information.
#[derive(Debug, Clone, Deserialize)]
pub struct Region {
    /// Region ID.
    pub id: String,
    /// Region city.
    pub city: String,
    /// Region country.
    pub country: String,
    /// Continent.
    pub continent: String,
    /// Available features.
    pub options: Vec<String>,
}

// ============================================================================
// Plan types
// ============================================================================

/// Bare metal plan.
#[derive(Debug, Clone, Deserialize)]
pub struct Plan {
    /// Plan ID.
    pub id: String,
    /// CPU count.
    pub cpu_count: i32,
    /// CPU model.
    pub cpu_model: String,
    /// CPU threads.
    pub cpu_threads: i32,
    /// RAM in MB.
    pub ram: i64,
    /// Disk size.
    pub disk: String,
    /// Disk count.
    pub disk_count: i32,
    /// Bandwidth in GB.
    pub bandwidth: i64,
    /// Monthly cost.
    pub monthly_cost: f64,
    /// Plan type.
    #[serde(rename = "type")]
    pub plan_type: String,
    /// Locations where available.
    pub locations: Vec<String>,
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
    pub ssh_key: String,
    /// Date created.
    pub date_created: String,
}

/// Create SSH key request.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyRequest {
    /// Key name.
    pub name: String,
    /// Public key content.
    pub ssh_key: String,
}

// ============================================================================
// OS types
// ============================================================================

/// Operating system.
#[derive(Debug, Clone, Deserialize)]
pub struct OperatingSystem {
    /// OS ID.
    pub id: i32,
    /// OS name.
    pub name: String,
    /// Architecture.
    pub arch: String,
    /// OS family.
    pub family: String,
}


