//! Provider trait and common types for bare metal providers.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during provider operations.
#[derive(Error, Debug)]
pub enum ProviderError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Server not found.
    #[error("Server not found: {0}")]
    NotFound(String),

    /// Operation timed out.
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),

    /// Server is stuck in a non-responsive state.
    /// LESSON LEARNED (Dec 2024): Servers can get stuck in "off" or "deploying"
    /// state indefinitely. When this happens, the server must be deleted and
    /// recreated.
    #[error("Server {id} stuck in '{status}' state for {duration_secs}s - delete and recreate")]
    ServerStuck {
        id: String,
        status: String,
        duration_secs: u64,
    },

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Server status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    /// Server is being deployed.
    Deploying,
    /// Server is on and running.
    On,
    /// Server is off.
    Off,
    /// Server disks are being erased (reinstall).
    DiskErasing,
    /// Server is being reinstalled.
    Reinstalling,
    /// Server is being deleted.
    Deleting,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deploying => write!(f, "deploying"),
            Self::On => write!(f, "on"),
            Self::Off => write!(f, "off"),
            Self::DiskErasing => write!(f, "disk_erasing"),
            Self::Reinstalling => write!(f, "reinstalling"),
            Self::Deleting => write!(f, "deleting"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A provisioned server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Unique server identifier.
    pub id: String,
    /// Server hostname.
    pub hostname: String,
    /// Current status.
    pub status: ServerStatus,
    /// Primary IPv4 address.
    pub ipv4: Option<String>,
    /// Primary IPv6 address.
    pub ipv6: Option<String>,
    /// Server plan/type.
    pub plan: String,
    /// Region/site where server is located.
    pub region: String,
    /// When the server was created.
    pub created_at: Option<DateTime<Utc>>,
}

/// Request to create a new server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    /// Hostname for the server.
    pub hostname: String,
    /// Plan/instance type (e.g., "c2-small-x86").
    pub plan: String,
    /// Region/site to deploy in (e.g., "MIA2").
    pub region: String,
    /// Operating system slug (e.g., `ubuntu_24_04_x64_lts`).
    pub os: String,
    /// SSH key IDs to add to the server.
    pub ssh_keys: Vec<String>,
}

/// Request to reinstall a server with iPXE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinstallIpxeRequest {
    /// Hostname for the server.
    pub hostname: String,
    /// URL to the iPXE script.
    pub ipxe_url: String,
}

/// Trait for bare metal cloud providers.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Create a new server.
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError>;

    /// Get server by ID.
    async fn get_server(&self, id: &str) -> Result<Server, ProviderError>;

    /// Wait for server to reach "on" status.
    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError>;

    /// Reinstall server with custom iPXE script.
    async fn reinstall_ipxe(
        &self,
        id: &str,
        req: ReinstallIpxeRequest,
    ) -> Result<(), ProviderError>;

    /// Delete a server.
    async fn delete_server(&self, id: &str) -> Result<(), ProviderError>;

    /// List all servers.
    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError>;
}
