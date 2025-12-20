//! Latitude.sh GPU VM API models.
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

// ============================================================================
// Virtual Machine types
// ============================================================================

/// Virtual Machine resource from API.
#[derive(Debug, Deserialize)]
pub struct VirtualMachineResource {
    /// VM ID.
    pub id: String,
    /// Resource type (always `virtual_machines`).
    #[serde(rename = "type")]
    pub resource_type: String,
    /// VM attributes.
    pub attributes: VirtualMachineAttributes,
}

/// Virtual Machine attributes.
#[derive(Debug, Deserialize)]
pub struct VirtualMachineAttributes {
    /// VM name.
    pub name: String,
    /// VM status.
    pub status: String,
    /// Operating system.
    pub operating_system: Option<String>,
    /// Creation timestamp.
    pub created_at: Option<String>,
    /// VM credentials (requires `extra_fields[virtual_machines]=credentials`).
    pub credentials: Option<VmCredentials>,
    /// VM plan info.
    pub plan: Option<VmPlan>,
    /// VM specs.
    pub specs: Option<VmSpecs>,
}

/// VM credentials.
#[derive(Debug, Deserialize)]
pub struct VmCredentials {
    /// Username for SSH access.
    pub username: Option<String>,
    /// Hostname/IP for access.
    pub host: Option<String>,
    /// Root password.
    pub password: Option<String>,
    /// SSH key IDs configured.
    pub ssh_keys: Option<Vec<String>>,
}

/// VM plan info.
#[derive(Debug, Deserialize)]
pub struct VmPlan {
    /// Plan ID.
    pub id: Option<String>,
    /// Plan name.
    pub name: Option<String>,
}

/// VM hardware specs.
#[derive(Debug, Deserialize)]
pub struct VmSpecs {
    /// Number of virtual CPUs.
    pub vcpu: Option<u32>,
    /// RAM description.
    pub ram: Option<String>,
    /// Storage description.
    pub storage: Option<String>,
    /// NIC description.
    pub nic: Option<String>,
    /// GPU description (if any).
    pub gpu: Option<String>,
}

/// Request body for creating a Virtual Machine.
#[derive(Debug, Serialize)]
pub struct CreateVirtualMachineBody {
    /// Request data.
    pub data: CreateVirtualMachineData,
}

/// Create Virtual Machine data wrapper.
#[derive(Debug, Serialize)]
pub struct CreateVirtualMachineData {
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// VM attributes.
    pub attributes: CreateVirtualMachineAttributes,
}

/// Attributes for creating a Virtual Machine.
#[derive(Debug, Serialize)]
pub struct CreateVirtualMachineAttributes {
    /// VM name.
    pub name: String,
    /// Plan slug or ID.
    pub plan: String,
    /// SSH key IDs to configure.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ssh_keys: Vec<String>,
    /// Project ID or slug.
    pub project: String,
}

/// Request body for VM actions.
#[derive(Debug, Serialize)]
pub struct VirtualMachineActionBody {
    /// VM ID.
    pub id: String,
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Action attributes.
    pub attributes: VirtualMachineActionAttributes,
}

/// VM action attributes.
#[derive(Debug, Serialize)]
pub struct VirtualMachineActionAttributes {
    /// Action to perform: `power_on`, `power_off`, or `reboot`.
    pub action: String,
}

// ============================================================================
// Virtual Machine Plan types
// ============================================================================

/// Virtual Machine plan resource.
#[derive(Debug, Deserialize)]
pub struct VirtualMachinePlanResource {
    /// Plan ID.
    pub id: String,
    /// Resource type (always `virtual_machine_plans`).
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Plan attributes.
    pub attributes: VirtualMachinePlanAttributes,
}

/// Virtual Machine plan attributes.
#[derive(Debug, Deserialize)]
pub struct VirtualMachinePlanAttributes {
    /// Plan name.
    pub name: Option<String>,
    /// Plan specs.
    pub specs: Option<VmPlanSpecs>,
    /// Available regions.
    pub regions: Option<Vec<VmPlanRegion>>,
    /// Stock level.
    pub stock_level: Option<String>,
}

/// VM plan specs.
#[derive(Debug, Deserialize)]
pub struct VmPlanSpecs {
    /// Memory in MB.
    pub memory: Option<u32>,
    /// Number of virtual CPUs.
    pub vcpus: Option<u32>,
    /// Disk info.
    pub disk: Option<VmPlanDisk>,
}

/// VM plan disk info.
#[derive(Debug, Deserialize)]
pub struct VmPlanDisk {
    /// Disk type (e.g., "local").
    #[serde(rename = "type")]
    pub disk_type: Option<String>,
    /// Disk size.
    pub size: Option<VmPlanDiskSize>,
}

/// VM plan disk size.
#[derive(Debug, Deserialize)]
pub struct VmPlanDiskSize {
    /// Size amount.
    pub amount: Option<u32>,
    /// Size unit (e.g., "gib").
    pub unit: Option<String>,
}

/// VM plan region info.
#[derive(Debug, Deserialize)]
pub struct VmPlanRegion {
    /// Region name.
    pub name: Option<String>,
    /// Available locations.
    pub available: Option<Vec<String>>,
    /// Pricing info.
    pub pricing: Option<VmPlanPricing>,
}

/// VM plan pricing.
#[derive(Debug, Deserialize)]
pub struct VmPlanPricing {
    /// USD pricing.
    #[serde(rename = "USD")]
    pub usd: Option<VmPlanPrice>,
}

/// VM plan price details.
#[derive(Debug, Deserialize)]
pub struct VmPlanPrice {
    /// Hourly price.
    pub hour: Option<f64>,
    /// Monthly price.
    pub month: Option<f64>,
    /// Yearly price.
    pub year: Option<f64>,
}

