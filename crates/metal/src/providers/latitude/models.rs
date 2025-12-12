//! Latitude.sh API request and response models.
//!
//! Based on the JSON:API specification used by Latitude.sh.

use serde::{Deserialize, Serialize};

// ============================================================================
// Common JSON:API wrapper types
// ============================================================================

/// JSON:API response wrapper.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data.
    pub data: T,
}

/// JSON:API error response.
#[derive(Debug, Deserialize)]
pub struct ApiError {
    /// Error details.
    pub errors: Vec<ApiErrorDetail>,
}

/// Individual error detail.
#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    /// Error status code.
    pub status: Option<String>,
    /// Error title.
    pub title: Option<String>,
    /// Error detail message.
    pub detail: Option<String>,
}

// ============================================================================
// Server types
// ============================================================================

/// Server resource from API.
#[derive(Debug, Deserialize)]
pub struct ServerResource {
    /// Server ID.
    pub id: String,
    /// Resource type (always "servers").
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Server attributes.
    pub attributes: ServerAttributes,
}

/// Server attributes.
#[derive(Debug, Deserialize)]
pub struct ServerAttributes {
    /// Server hostname.
    pub hostname: String,
    /// Server status.
    pub status: String,
    /// Primary IPv4 address.
    pub primary_ipv4: Option<String>,
    /// Primary IPv6 address.
    pub primary_ipv6: Option<String>,
    /// IPMI status.
    pub ipmi_status: Option<String>,
    /// Server creation timestamp.
    pub created_at: Option<String>,
    /// Server specs.
    pub specs: Option<ServerSpecs>,
    /// Server plan.
    pub plan: Option<ServerPlan>,
}

/// Server hardware specs.
#[derive(Debug, Deserialize)]
pub struct ServerSpecs {
    /// CPU description.
    pub cpu: Option<String>,
    /// Disk description.
    pub disk: Option<String>,
    /// RAM description.
    pub ram: Option<String>,
    /// NIC description.
    pub nic: Option<String>,
}

/// Server plan info.
#[derive(Debug, Deserialize)]
pub struct ServerPlan {
    /// Plan ID.
    pub id: Option<String>,
    /// Plan name.
    pub name: Option<String>,
    /// Plan slug.
    pub slug: Option<String>,
}

// ============================================================================
// Create Server request
// ============================================================================

/// Request body for creating a server.
#[derive(Debug, Serialize)]
pub struct CreateServerBody {
    /// Request data.
    pub data: CreateServerData,
}

/// Create server data wrapper.
#[derive(Debug, Serialize)]
pub struct CreateServerData {
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Server attributes.
    pub attributes: CreateServerAttributes,
}

/// Attributes for creating a server.
#[derive(Debug, Serialize)]
pub struct CreateServerAttributes {
    /// Project ID or slug.
    pub project: String,
    /// Plan slug (e.g., "c2-small-x86").
    pub plan: String,
    /// Site/region slug (e.g., "MIA2").
    pub site: String,
    /// Operating system slug.
    pub operating_system: String,
    /// Server hostname.
    pub hostname: String,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_keys: Vec<String>,
}

// ============================================================================
// Reinstall Server request
// ============================================================================

/// Request body for reinstalling a server.
#[derive(Debug, Serialize)]
pub struct ReinstallServerBody {
    /// Request data.
    pub data: ReinstallServerData,
}

/// Reinstall server data wrapper.
#[derive(Debug, Serialize)]
pub struct ReinstallServerData {
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Reinstall attributes.
    pub attributes: ReinstallServerAttributes,
}

/// Attributes for reinstalling a server.
#[derive(Debug, Serialize)]
pub struct ReinstallServerAttributes {
    /// Operating system slug (use "ipxe" for custom iPXE).
    pub operating_system: String,
    /// Server hostname.
    pub hostname: String,
    /// iPXE script URL (required when `operating_system` is "ipxe").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipxe: Option<String>,
}

// ============================================================================
// SSH Key types
// ============================================================================

/// SSH key resource from API.
#[derive(Debug, Deserialize)]
pub struct SshKeyResource {
    /// SSH key ID.
    pub id: String,
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// SSH key attributes.
    pub attributes: SshKeyAttributes,
}

/// SSH key attributes.
#[derive(Debug, Deserialize)]
pub struct SshKeyAttributes {
    /// Key name.
    pub name: String,
    /// Public key content.
    pub public_key: Option<String>,
}

/// Request body for creating an SSH key.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyBody {
    /// Request data.
    pub data: CreateSshKeyData,
}

/// Create SSH key data wrapper.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyData {
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// SSH key attributes.
    pub attributes: CreateSshKeyAttributes,
}

/// Attributes for creating an SSH key.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyAttributes {
    /// Key name.
    pub name: String,
    /// Public key content.
    pub public_key: String,
}

// ============================================================================
// Plan types
// ============================================================================

/// Plan resource from API.
#[derive(Debug, Deserialize)]
pub struct PlanResource {
    /// Plan ID.
    pub id: String,
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Plan attributes.
    pub attributes: PlanAttributes,
}

/// Plan attributes.
#[derive(Debug, Deserialize)]
pub struct PlanAttributes {
    /// Plan name.
    pub name: Option<String>,
    /// Plan slug (e.g., "m4-metal-small").
    pub slug: Option<String>,
    /// Plan features (list of feature strings like "ssh", "raid", "`user_data`").
    pub features: Option<Vec<String>>,
    /// Plan specs.
    pub specs: Option<PlanSpecs>,
    /// Available regions.
    pub regions: Option<Vec<PlanRegion>>,
}

/// Plan hardware specs.
#[derive(Debug, Deserialize)]
pub struct PlanSpecs {
    /// CPU info.
    pub cpu: Option<PlanCpu>,
    /// Memory info.
    pub memory: Option<PlanMemory>,
    /// Disk info.
    pub drives: Option<Vec<PlanDrive>>,
    /// Network info.
    pub nics: Option<Vec<PlanNic>>,
}

/// Plan CPU specs.
#[derive(Debug, Deserialize)]
pub struct PlanCpu {
    /// CPU type/model.
    #[serde(rename = "type")]
    pub cpu_type: Option<String>,
    /// CPU description.
    pub description: Option<String>,
    /// Number of cores.
    pub cores: Option<u32>,
    /// Clock speed in GHz.
    pub clock: Option<f32>,
}

/// Plan memory specs.
#[derive(Debug, Deserialize)]
pub struct PlanMemory {
    /// Total RAM in GB.
    pub total: Option<u32>,
}

/// Plan drive specs.
#[derive(Debug, Deserialize)]
pub struct PlanDrive {
    /// Number of drives.
    pub count: Option<u32>,
    /// Drive size.
    pub size: Option<String>,
    /// Drive type (e.g., "`NVMe`").
    #[serde(rename = "type")]
    pub drive_type: Option<String>,
}

/// Plan NIC specs.
#[derive(Debug, Deserialize)]
pub struct PlanNic {
    /// Number of NICs.
    pub count: Option<u32>,
    /// NIC type/description.
    #[serde(rename = "type")]
    pub nic_type: Option<String>,
}

/// Plan region availability.
#[derive(Debug, Deserialize)]
pub struct PlanRegion {
    /// Region name (e.g., "United States", "Japan").
    pub name: Option<String>,
    /// Location availability info.
    pub locations: Option<PlanLocations>,
    /// Stock level (e.g., "high", "medium", "low", "unavailable").
    pub stock_level: Option<String>,
    /// Pricing info.
    pub pricing: Option<PlanPricing>,
}

/// Plan location availability.
#[derive(Debug, Deserialize)]
pub struct PlanLocations {
    /// Available site slugs (e.g., `["DAL", "LAX", "NYC"]`).
    pub available: Option<Vec<String>>,
    /// In-stock site slugs (e.g., `["DAL", "NYC"]`).
    pub in_stock: Option<Vec<String>>,
}

/// Plan pricing by currency.
#[derive(Debug, Deserialize)]
pub struct PlanPricing {
    /// USD pricing.
    #[serde(rename = "USD")]
    pub usd: Option<PlanPrice>,
}

/// Plan price details.
#[derive(Debug, Deserialize)]
pub struct PlanPrice {
    /// Hourly price.
    pub hour: Option<f64>,
    /// Monthly price.
    pub month: Option<f64>,
}

// ============================================================================
// Region types
// ============================================================================

/// Region resource from API.
#[derive(Debug, Deserialize)]
pub struct RegionResource {
    /// Region ID.
    pub id: String,
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    /// Region attributes.
    pub attributes: RegionAttributes,
}

/// Region attributes.
#[derive(Debug, Deserialize)]
pub struct RegionAttributes {
    /// Region name.
    pub name: Option<String>,
    /// Region slug.
    pub slug: Option<String>,
    /// Country info.
    pub country: Option<RegionCountry>,
}

/// Region country info.
#[derive(Debug, Deserialize)]
pub struct RegionCountry {
    /// Country name.
    pub name: Option<String>,
    /// Country slug.
    pub slug: Option<String>,
}
