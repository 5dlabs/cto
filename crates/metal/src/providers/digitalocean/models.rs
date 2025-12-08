//! `DigitalOcean` API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// Droplet types
// ============================================================================

/// Droplet (instance) from API.
#[derive(Debug, Clone, Deserialize)]
pub struct Droplet {
    /// Droplet ID.
    pub id: i64,
    /// Droplet name.
    pub name: String,
    /// Memory in MB.
    pub memory: i64,
    /// vCPU count.
    pub vcpus: i32,
    /// Disk size in GB.
    pub disk: i64,
    /// Status: "new", "active", "off", "archive".
    pub status: String,
    /// Region info.
    pub region: Region,
    /// Size (plan) slug.
    pub size_slug: String,
    /// Networks.
    pub networks: Networks,
    /// Image info.
    pub image: Image,
    /// Tags.
    pub tags: Vec<String>,
    /// Created at.
    pub created_at: String,
}

/// Droplet list response.
#[derive(Debug, Deserialize)]
pub struct DropletListResponse {
    /// List of droplets.
    pub droplets: Vec<Droplet>,
    /// Links for pagination.
    pub links: Option<Links>,
    /// Metadata.
    pub meta: Option<Meta>,
}

/// Single droplet response.
#[derive(Debug, Deserialize)]
pub struct DropletResponse {
    /// Droplet details.
    pub droplet: Droplet,
}

/// Network configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Networks {
    /// IPv4 addresses.
    pub v4: Vec<NetworkAddress>,
    /// IPv6 addresses.
    pub v6: Vec<NetworkAddress>,
}

/// Network address.
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkAddress {
    /// IP address.
    pub ip_address: String,
    /// Netmask.
    pub netmask: Option<String>,
    /// Gateway.
    pub gateway: Option<String>,
    /// Type: "public" or "private".
    #[serde(rename = "type")]
    pub address_type: String,
}

/// Region information.
#[derive(Debug, Clone, Deserialize)]
pub struct Region {
    /// Region slug.
    pub slug: String,
    /// Region name.
    pub name: String,
    /// Available features.
    pub features: Vec<String>,
    /// Available.
    pub available: bool,
}

/// Image information.
#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    /// Image ID.
    pub id: i64,
    /// Image name.
    pub name: String,
    /// Image slug.
    pub slug: Option<String>,
    /// Distribution.
    pub distribution: String,
    /// Public image.
    pub public: bool,
}

/// Pagination links.
#[derive(Debug, Clone, Deserialize)]
pub struct Links {
    /// Pages.
    pub pages: Option<Pages>,
}

/// Page links.
#[derive(Debug, Clone, Deserialize)]
pub struct Pages {
    /// First page.
    pub first: Option<String>,
    /// Previous page.
    pub prev: Option<String>,
    /// Next page.
    pub next: Option<String>,
    /// Last page.
    pub last: Option<String>,
}

/// Response metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
    /// Total count.
    pub total: i32,
}

// ============================================================================
// Create Droplet types
// ============================================================================

/// Request body for creating a droplet.
#[derive(Debug, Serialize)]
pub struct CreateDropletRequest {
    /// Droplet name.
    pub name: String,
    /// Region slug.
    pub region: String,
    /// Size (plan) slug.
    pub size: String,
    /// Image slug or ID.
    pub image: ImageIdentifier,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_keys: Vec<String>,
    /// Enable IPv6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,
    /// Enable monitoring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<bool>,
    /// User data for cloud-init.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
    /// Tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// VPC UUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_uuid: Option<String>,
}

/// Image identifier (can be slug or ID).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageIdentifier {
    /// Image slug.
    Slug(String),
    /// Image ID.
    Id(i64),
}

// ============================================================================
// Action types
// ============================================================================

/// Droplet action.
#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    /// Action ID.
    pub id: i64,
    /// Action status: "in-progress", "completed", "errored".
    pub status: String,
    /// Action type.
    #[serde(rename = "type")]
    pub action_type: String,
    /// Started at.
    pub started_at: String,
    /// Completed at.
    pub completed_at: Option<String>,
}

/// Action response.
#[derive(Debug, Deserialize)]
pub struct ActionResponse {
    /// Action details.
    pub action: Action,
}

/// Rebuild request.
#[derive(Debug, Serialize)]
pub struct RebuildRequest {
    /// Action type.
    #[serde(rename = "type")]
    pub action_type: String,
    /// Image slug or ID.
    pub image: ImageIdentifier,
}

/// Power action request.
#[derive(Debug, Serialize)]
pub struct PowerActionRequest {
    /// Action type: `power_on`, `power_off`, `reboot`, `power_cycle`.
    #[serde(rename = "type")]
    pub action_type: String,
}

// ============================================================================
// SSH Key types
// ============================================================================

/// SSH key.
#[derive(Debug, Clone, Deserialize)]
pub struct SshKey {
    /// Key ID.
    pub id: i64,
    /// Key fingerprint.
    pub fingerprint: String,
    /// Key name.
    pub name: String,
    /// Public key.
    pub public_key: String,
}

/// Create SSH key request.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyRequest {
    /// Key name.
    pub name: String,
    /// Public key content.
    pub public_key: String,
}

// ============================================================================
// Size (Plan) types
// ============================================================================

/// Instance size (plan).
#[derive(Debug, Clone, Deserialize)]
pub struct Size {
    /// Size slug.
    pub slug: String,
    /// Memory in MB.
    pub memory: i64,
    /// vCPU count.
    pub vcpus: i32,
    /// Disk size in GB.
    pub disk: i64,
    /// Transfer in TB.
    pub transfer: f64,
    /// Price per month.
    pub price_monthly: f64,
    /// Price per hour.
    pub price_hourly: f64,
    /// Available regions.
    pub regions: Vec<String>,
    /// Available.
    pub available: bool,
    /// Description.
    pub description: String,
}
