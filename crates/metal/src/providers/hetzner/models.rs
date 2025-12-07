//! Hetzner Robot API request and response models.
//!
//! Based on the Hetzner Robot API documentation.

use serde::{Deserialize, Serialize};

// ============================================================================
// Server types
// ============================================================================

/// Server information from Hetzner Robot API.
#[derive(Debug, Deserialize)]
pub struct HetznerServer {
    /// Server details wrapper.
    pub server: ServerDetails,
}

/// Server details.
#[derive(Debug, Deserialize)]
pub struct ServerDetails {
    /// Server number (unique identifier).
    pub server_number: i64,
    /// Server name/label.
    pub server_name: String,
    /// Server IP address.
    pub server_ip: String,
    /// IPv6 network.
    pub server_ipv6_net: Option<String>,
    /// Product name (plan).
    pub product: String,
    /// Data center location.
    pub dc: String,
    /// Server status.
    pub status: String,
    /// Paid until date.
    pub paid_until: Option<String>,
    /// Cancellation allowed.
    pub cancelled: bool,
}

/// List of servers response.
#[derive(Debug, Deserialize)]
pub struct ServerListResponse {
    /// List of server wrappers.
    #[serde(default)]
    pub servers: Vec<HetznerServer>,
}

// ============================================================================
// Transaction/Order types
// ============================================================================

/// Server order request for POST /order/server/transaction.
#[derive(Debug, Serialize)]
pub struct OrderServerRequest {
    /// Product ID to order (e.g., "EX44").
    pub product_id: String,
    /// Data center location (e.g., "FSN1", "NBG1", "HEL1").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Authorized SSH key fingerprints.
    #[serde(rename = "authorized_key[]")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorized_key: Vec<String>,
    /// Distribution name for preinstallation (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dist: Option<String>,
    /// Language of preinstalled distribution (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Order comment (optional). Note: if supplied, order is processed manually.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Set to "true" for test order (won't be processed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

/// Server marketplace order request for `POST /order/server_market/transaction`.
#[derive(Debug, Serialize)]
pub struct OrderMarketServerRequest {
    /// Product ID from server market.
    pub product_id: i64,
    /// Authorized SSH key fingerprints.
    #[serde(rename = "authorized_key[]")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorized_key: Vec<String>,
    /// Distribution name for preinstallation (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dist: Option<String>,
    /// Language of preinstalled distribution (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Order comment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Set to "true" for test order (won't be processed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

/// Server order response.
#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    /// Order transaction.
    pub transaction: OrderTransaction,
}

/// Order transaction details.
#[derive(Debug, Deserialize)]
pub struct OrderTransaction {
    /// Transaction ID (e.g., "B20150121-344958-251479").
    pub id: String,
    /// Transaction date (ISO 8601).
    pub date: String,
    /// Transaction status: "ready", "in process", or "cancelled".
    pub status: String,
    /// Server number if allocated (only when status is "ready").
    pub server_number: Option<i64>,
    /// Server IP address if allocated (only when status is "ready").
    pub server_ip: Option<String>,
    /// Product details.
    #[serde(default)]
    pub product: Option<OrderedProduct>,
}

/// Product details in order transaction.
#[derive(Debug, Deserialize)]
pub struct OrderedProduct {
    /// Product ID.
    pub id: String,
    /// Product name.
    pub name: String,
    /// Data center location.
    #[serde(default)]
    pub location: Option<String>,
}

// ============================================================================
// Cancellation types
// ============================================================================

/// Server cancellation request for POST /server/{id}/cancellation.
#[derive(Debug, Serialize)]
pub struct CancellationRequest {
    /// Date to cancel: "YYYY-MM-DD" format or "now" for immediate.
    pub cancellation_date: String,
    /// Optional cancellation reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation_reason: Option<String>,
    /// Whether to reserve server location after cancellation.
    /// Required if `reservation_possible` is true, otherwise must be "false" or omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve_location: Option<String>,
}

/// Server cancellation response.
#[derive(Debug, Deserialize)]
pub struct CancellationResponse {
    /// Cancellation details.
    pub cancellation: CancellationDetails,
}

/// Cancellation details.
#[derive(Debug, Deserialize)]
pub struct CancellationDetails {
    /// Server IP address.
    pub server_ip: String,
    /// Server IPv6 network.
    #[serde(default)]
    pub server_ipv6_net: Option<String>,
    /// Server number.
    pub server_number: i64,
    /// Server name.
    pub server_name: String,
    /// Earliest possible cancellation date.
    pub earliest_cancellation_date: String,
    /// Whether server is cancelled.
    pub cancelled: bool,
    /// Cancellation date if active.
    pub cancellation_date: Option<String>,
}

// ============================================================================
// Product listing types
// ============================================================================

/// Server product for ordering.
#[derive(Debug, Deserialize)]
pub struct ServerProduct {
    /// Product details.
    pub product: ProductDetails,
}

/// Product details.
#[derive(Debug, Deserialize)]
pub struct ProductDetails {
    /// Product ID (e.g., "EX44").
    pub id: String,
    /// Product name.
    pub name: String,
    /// Product description lines.
    pub description: Vec<String>,
    /// Traffic quota.
    pub traffic: String,
    /// Available distributions.
    #[serde(default)]
    pub dist: Vec<String>,
    /// Available languages.
    #[serde(default)]
    pub lang: Vec<String>,
    /// Available locations.
    #[serde(default)]
    pub location: Vec<String>,
}

/// Server market product (auction).
#[derive(Debug, Deserialize)]
pub struct MarketProduct {
    /// Product details.
    pub product: MarketProductDetails,
}

/// Market product details.
#[derive(Debug, Deserialize)]
pub struct MarketProductDetails {
    /// Product ID (numeric for market products).
    pub id: i64,
    /// Product name (e.g., "SB34").
    pub name: String,
    /// Product description lines.
    pub description: Vec<String>,
    /// Traffic quota.
    pub traffic: String,
    /// CPU model.
    pub cpu: String,
    /// Memory size in GB.
    pub memory_size: i32,
    /// Data center.
    pub datacenter: String,
    /// Monthly price.
    pub price: String,
}

// ============================================================================
// Boot/Reinstall types
// ============================================================================

/// Boot configuration request.
#[derive(Debug, Serialize)]
pub struct BootConfigRequest {
    /// Operating system to install.
    pub os: String,
    /// Architecture.
    pub arch: Option<String>,
    /// SSH key fingerprints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorized_key: Vec<String>,
}

/// Linux boot activation (for custom image).
#[derive(Debug, Serialize)]
pub struct LinuxBootRequest {
    /// Distribution name.
    pub dist: String,
    /// Architecture (default: 64).
    pub arch: Option<i32>,
    /// Language.
    pub lang: Option<String>,
    /// SSH key fingerprints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorized_key: Vec<String>,
}

/// Rescue mode activation request.
#[derive(Debug, Serialize)]
pub struct RescueRequest {
    /// Operating system for rescue mode.
    pub os: String,
    /// SSH key fingerprints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorized_key: Vec<String>,
}

/// Rescue mode response.
#[derive(Debug, Deserialize)]
pub struct RescueResponse {
    /// Rescue mode details.
    pub rescue: RescueDetails,
}

/// Rescue mode details.
#[derive(Debug, Deserialize)]
pub struct RescueDetails {
    /// Server number.
    pub server_number: i64,
    /// Active status.
    pub active: bool,
    /// Root password for rescue mode.
    pub password: Option<String>,
}

// ============================================================================
// Reset types
// ============================================================================

/// Reset server request.
#[derive(Debug, Serialize)]
pub struct ResetRequest {
    /// Reset type: "hw" (hardware), "sw" (software), "power" (power cycle).
    #[serde(rename = "type")]
    pub reset_type: String,
}

/// Reset response.
#[derive(Debug, Deserialize)]
pub struct ResetResponse {
    /// Reset details.
    pub reset: ResetDetails,
}

/// Reset details.
#[derive(Debug, Deserialize)]
pub struct ResetDetails {
    /// Server number.
    pub server_number: i64,
    /// Reset type.
    #[serde(rename = "type")]
    pub reset_type: String,
}

// ============================================================================
// SSH Key types
// ============================================================================

/// SSH key resource.
#[derive(Debug, Deserialize)]
pub struct SshKey {
    /// Key details.
    pub key: SshKeyDetails,
}

/// SSH key details.
#[derive(Debug, Deserialize)]
pub struct SshKeyDetails {
    /// Key name.
    pub name: String,
    /// Key fingerprint.
    pub fingerprint: String,
    /// Key type.
    #[serde(rename = "type")]
    pub key_type: String,
    /// Key size.
    pub size: i32,
    /// Key data.
    pub data: String,
}

/// Create SSH key request.
#[derive(Debug, Serialize)]
pub struct CreateSshKeyRequest {
    /// Key name.
    pub name: String,
    /// Public key data.
    pub data: String,
}

// ============================================================================
// vSwitch/Network types
// ============================================================================

/// vSwitch (VLAN) resource.
#[derive(Debug, Deserialize)]
pub struct VSwitch {
    /// vSwitch ID.
    pub id: i64,
    /// vSwitch name.
    pub name: String,
    /// VLAN ID.
    pub vlan: i32,
    /// Cancelled status.
    pub cancelled: bool,
}

/// vSwitch list response.
#[derive(Debug, Deserialize)]
pub struct VSwitchListResponse {
    /// List of vSwitches.
    #[serde(default)]
    pub vswitches: Vec<VSwitch>,
}
