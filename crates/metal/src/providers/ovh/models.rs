//! `OVHcloud` API request and response models.
//!
//! Based on the `OVHcloud` API documentation.

use serde::{Deserialize, Serialize};

// ============================================================================
// Server types
// ============================================================================

/// Dedicated server information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedServer {
    /// Server name (identifier).
    pub name: String,
    /// Server IP address.
    pub ip: Option<String>,
    /// Data center location.
    pub datacenter: Option<String>,
    /// Professional use.
    pub professional_use: bool,
    /// Commercial range.
    pub commercial_range: Option<String>,
    /// Operating system.
    pub os: Option<String>,
    /// Server state.
    pub state: String,
    /// Reverse DNS.
    pub reverse: Option<String>,
    /// Monitoring enabled.
    pub monitoring: bool,
    /// Root device.
    pub root_device: Option<String>,
    /// Rack location.
    pub rack: Option<String>,
    /// Server boot mode.
    pub boot_id: Option<i64>,
    /// Link speed in Mbps.
    pub link_speed: Option<i64>,
}

/// Server hardware specifications.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerHardware {
    /// Motherboard model.
    pub motherboard: Option<String>,
    /// CPU model.
    pub processor_architecture: Option<String>,
    /// CPU name.
    pub processor_name: Option<String>,
    /// Number of processors.
    pub number_of_processors: Option<i32>,
    /// Cores per processor.
    pub cores_per_processor: Option<i32>,
    /// Total memory in MB.
    pub memory_size: Option<i64>,
    /// Disk information.
    pub disk_groups: Option<Vec<DiskGroup>>,
}

/// Disk group information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskGroup {
    /// Disk type (SSD, HDD, `NVMe`).
    pub disk_type: Option<String>,
    /// Number of disks.
    pub number_of_disks: Option<i32>,
    /// Disk size in GB.
    pub disk_size: Option<i64>,
    /// RAID level.
    pub raid_controller: Option<String>,
}

// ============================================================================
// Installation types
// ============================================================================

/// OS installation request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationRequest {
    /// Template name (OS identifier).
    pub template_name: String,
    /// Partition scheme.
    pub partition_scheme_name: Option<String>,
    /// Custom hostname.
    pub details: Option<InstallationDetails>,
}

/// Installation details.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationDetails {
    /// Custom hostname.
    pub custom_hostname: Option<String>,
    /// SSH key name.
    pub ssh_key_name: Option<String>,
    /// Post installation script.
    pub post_installation_script_link: Option<String>,
    /// Post installation script return.
    pub post_installation_script_return: Option<String>,
}

/// Installation status.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationStatus {
    /// Current progress (0-100).
    pub progress: i32,
    /// Installation status.
    pub status: String,
    /// Elapsed time in seconds.
    pub elapsed_time: Option<i64>,
}

// ============================================================================
// Boot types
// ============================================================================

/// Boot configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootConfig {
    /// Boot ID.
    pub boot_id: i64,
    /// Boot type.
    pub boot_type: String,
    /// Description.
    pub description: Option<String>,
    /// Kernel.
    pub kernel: Option<String>,
}

/// Network boot (iPXE) request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkBootRequest {
    /// Boot type (must be "ipxeCustomerScript").
    pub boot_type: String,
    /// iPXE script content.
    pub kernel: String,
}

// ============================================================================
// Task types
// ============================================================================

/// Server task (async operation).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTask {
    /// Task ID.
    pub task_id: i64,
    /// Function name.
    pub function: String,
    /// Start date.
    pub start_date: Option<String>,
    /// Done date.
    pub done_date: Option<String>,
    /// Status.
    pub status: String,
    /// Comment.
    pub comment: Option<String>,
}

// ============================================================================
// Reboot types
// ============================================================================

/// Reboot request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebootRequest {
    /// Reboot type: "hardreset", "power".
    #[serde(rename = "type")]
    pub reboot_type: Option<String>,
}

// ============================================================================
// vRack/Network types
// ============================================================================

/// vRack (private network) configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VRack {
    /// vRack name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
}

/// vRack allowed services.
#[derive(Debug, Clone, Deserialize)]
pub struct VRackAllowedServices {
    /// List of dedicated servers.
    #[serde(default)]
    pub dedicated_server: Vec<String>,
}

// ============================================================================
// SSH Key types
// ============================================================================

/// SSH key.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshKey {
    /// Key name.
    pub key_name: String,
    /// Key content.
    pub key: String,
    /// Is default key.
    pub default: bool,
}

/// Create SSH key request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSshKeyRequest {
    /// Key name.
    pub key_name: String,
    /// Public key content.
    pub key: String,
}

// ============================================================================
// Order/Cart types
// ============================================================================

/// Create cart request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCartRequest {
    /// OVH subsidiary (e.g., "US", "EU", "FR").
    pub ovh_subsidiary: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Cart response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cart {
    /// Cart ID.
    pub cart_id: String,
    /// Cart description.
    #[serde(default)]
    pub description: Option<String>,
    /// Expiration date.
    pub expire: String,
    /// Read-only flag.
    pub read_only: bool,
}

/// Bare metal server product.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaremetalProduct {
    /// Plan code (e.g., `24rise01-us`).
    pub plan_code: String,
    /// Product ID.
    #[serde(default)]
    pub product_id: Option<String>,
    /// Product name.
    #[serde(default)]
    pub product_name: Option<String>,
    /// Available durations (e.g., `["P1M", "P12M"]`).
    pub duration: Vec<String>,
    /// Pricing mode.
    pub pricing_mode: String,
    /// Whether orderable.
    #[serde(default)]
    pub orderable: Option<bool>,
}

/// Add item to cart request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCartItemRequest {
    /// Plan code (e.g., `24rise01-us`).
    pub plan_code: String,
    /// Duration (e.g., "P1M").
    pub duration: String,
    /// Pricing mode (e.g., "default").
    pub pricing_mode: String,
    /// Quantity.
    pub quantity: i32,
}

/// Cart item response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CartItem {
    /// Item ID.
    pub item_id: i64,
    /// Duration.
    pub duration: String,
    /// Plan code.
    #[serde(default)]
    pub plan_code: Option<String>,
}

/// Item configuration request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemConfigurationRequest {
    /// Configuration label (e.g., `dedicated_datacenter`, `dedicated_os`, "region").
    pub label: String,
    /// Configuration value.
    pub value: String,
}

/// Configuration response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationResponse {
    /// Configuration ID.
    pub id: i64,
    /// Configuration label.
    pub label: String,
    /// Configuration value.
    pub value: String,
}

/// Checkout request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutRequest {
    /// Auto pay with preferred payment method.
    #[serde(default)]
    pub auto_pay_with_preferred_payment_method: bool,
    /// Waive retraction period.
    #[serde(default)]
    pub waive_retractation_period: bool,
}

/// Order response from checkout.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    /// Order ID.
    pub order_id: i64,
    /// Order URL for payment.
    pub url: String,
}

// ============================================================================
// Service termination types
// ============================================================================

/// Service termination confirmation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminationResponse {
    /// Message about termination.
    pub message: String,
}
