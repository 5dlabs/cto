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
