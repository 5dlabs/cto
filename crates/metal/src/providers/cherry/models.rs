//! Cherry Servers API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// Billing & Pricing types
// ============================================================================

/// Billing period for pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingPeriod {
    /// Hourly billing (pay-as-you-go).
    Hourly,
    /// Monthly billing (committed).
    Monthly,
}

impl BillingPeriod {
    /// Parse from CLI string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hourly" => Self::Hourly,
            "monthly" | _ => Self::Monthly,
        }
    }
}

/// Pricing information for a plan.
#[derive(Debug, Clone, Deserialize)]
pub struct Pricing {
    /// Pricing ID.
    pub id: i64,
    /// Billing unit (Hourly, Monthly, etc.).
    pub unit: String,
    /// Price in EUR.
    pub price: f64,
    /// Currency code.
    pub currency: String,
    /// Whether tax is included.
    pub taxed: bool,
}

/// Extended plan info with pricing.
#[derive(Debug, Clone)]
pub struct PlanWithPricing {
    /// Plan ID.
    pub id: i64,
    /// Plan href.
    pub href: String,
    /// Plan name.
    pub name: String,
    /// Plan slug.
    pub slug: String,
    /// Plan category (baremetal, cloud, etc.).
    pub category: String,
    /// CPU specs.
    pub cpus: Option<CpuSpec>,
    /// Memory specs.
    pub memory: Option<MemorySpec>,
    /// Storage specs.
    pub storage: Option<Vec<StorageSpec>>,
    /// NIC info.
    pub nics: Option<NicSpec>,
    /// Bandwidth info.
    pub bandwidth: Option<BandwidthSpec>,
    /// Hourly price in EUR.
    pub hourly_eur: f64,
    /// Monthly price in EUR.
    pub monthly_eur: f64,
}

/// NIC specification.
#[derive(Debug, Clone, Deserialize)]
pub struct NicSpec {
    /// NIC count and speed.
    pub name: String,
}

/// Bandwidth specification.
#[derive(Debug, Clone, Deserialize)]
pub struct BandwidthSpec {
    /// Bandwidth amount.
    pub name: String,
}

// ============================================================================
// Server types
// ============================================================================

/// Server resource from API.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerResource {
    /// Server ID.
    pub id: i64,
    /// Server name.
    pub name: Option<String>,
    /// Server hostname.
    pub hostname: String,
    /// Server status.
    pub status: String,
    /// Primary IP address info.
    pub ip_addresses: Vec<IpAddress>,
    /// Region info.
    pub region: Option<Region>,
    /// Plan info.
    pub plan: Option<Plan>,
    /// Project ID.
    pub project_id: Option<i64>,
    /// Created at timestamp.
    pub created_at: Option<String>,
}

/// IP address information.
#[derive(Debug, Clone, Deserialize)]
pub struct IpAddress {
    /// IP address.
    pub address: String,
    /// Address type (primary, floating, etc.).
    pub address_type: Option<String>,
    /// Address family (IPv4, IPv6).
    pub address_family: i32,
}

/// Region information.
#[derive(Debug, Clone, Deserialize)]
pub struct Region {
    /// Region ID.
    pub id: i64,
    /// Region name.
    pub name: String,
    /// Region slug.
    pub slug: String,
}

/// Plan information.
#[derive(Debug, Clone, Deserialize)]
pub struct Plan {
    /// Plan ID.
    pub id: i64,
    /// Plan name.
    pub name: String,
    /// Plan slug.
    pub slug: String,
    /// Plan specs.
    pub specs: Option<PlanSpecs>,
}

/// Plan specifications.
#[derive(Debug, Clone, Deserialize)]
pub struct PlanSpecs {
    /// CPU info.
    pub cpus: Option<CpuSpec>,
    /// Memory info.
    pub memory: Option<MemorySpec>,
    /// Storage info.
    pub storage: Option<Vec<StorageSpec>>,
    /// NIC info.
    pub nics: Option<NicSpec>,
    /// Bandwidth info.
    pub bandwidth: Option<BandwidthSpec>,
}

/// CPU specification.
#[derive(Debug, Clone, Deserialize)]
pub struct CpuSpec {
    /// Number of cores.
    pub cores: Option<i32>,
    /// CPU frequency.
    pub frequency: Option<f64>,
    /// CPU name.
    pub name: Option<String>,
}

/// Memory specification.
#[derive(Debug, Clone, Deserialize)]
pub struct MemorySpec {
    /// Total memory in GB.
    pub total: Option<i64>,
}

/// Storage specification.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageSpec {
    /// Number of disks.
    pub count: Option<i32>,
    /// Disk size in GB.
    pub size: Option<i64>,
    /// Disk type.
    #[serde(rename = "type")]
    pub storage_type: Option<String>,
}

// ============================================================================
// Create Server types
// ============================================================================

/// Request body for creating a server.
#[derive(Debug, Serialize)]
pub struct CreateServerRequest {
    /// Region slug.
    pub region: String,
    /// Plan slug.
    pub plan: String,
    /// Server hostname.
    pub hostname: String,
    /// Image slug (OS).
    pub image: String,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_keys: Vec<i64>,
    /// Optional user data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
    /// Tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<serde_json::Value>,
}

// ============================================================================
// Reinstall types
// ============================================================================

/// Request for reinstalling a server.
#[derive(Debug, Serialize)]
pub struct ReinstallRequest {
    /// Image slug (OS).
    pub image: String,
    /// Server hostname.
    pub hostname: String,
    /// SSH key IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_keys: Vec<i64>,
    /// Optional user data (for iPXE, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
}

// ============================================================================
// Power action types
// ============================================================================

/// Power action request.
#[derive(Debug, Serialize)]
pub struct PowerActionRequest {
    /// Action type: `power_on`, `power_off`, `reboot`.
    #[serde(rename = "type")]
    pub action_type: String,
}

// ============================================================================
// SSH Key types
// ============================================================================

/// SSH key resource.
#[derive(Debug, Clone, Deserialize)]
pub struct SshKey {
    /// Key ID.
    pub id: i64,
    /// Key label.
    pub label: String,
    /// Key fingerprint.
    pub fingerprint: String,
    /// Public key.
    pub key: String,
    /// Created timestamp.
    pub created: Option<String>,
    /// Updated timestamp.
    pub updated: Option<String>,
}

/// Create SSH key request.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyRequest {
    /// Key label.
    pub label: String,
    /// Public key content.
    pub key: String,
}

// ============================================================================
// Project types
// ============================================================================

/// Project resource.
#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    /// Project ID.
    pub id: i64,
    /// Project name.
    pub name: String,
    /// BGP enabled.
    pub bgp: Option<ProjectBgp>,
}

/// Project BGP settings.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectBgp {
    /// BGP enabled.
    pub enabled: bool,
    /// Local ASN.
    pub local_asn: Option<i64>,
}
