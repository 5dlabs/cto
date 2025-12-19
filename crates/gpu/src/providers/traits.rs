//! GPU provider trait and common types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during GPU provider operations.
#[derive(Error, Debug)]
pub enum GpuProviderError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Operation timed out.
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// No GPU plans available.
    #[error("No GPU plans available in region: {0}")]
    NoPlansAvailable(String),
}

/// GPU VM status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuVmStatus {
    /// VM is being scheduled.
    Scheduling,
    /// VM is scheduled, waiting to start.
    Scheduled,
    /// VM is starting up.
    Starting,
    /// VM is configuring network.
    ConfiguringNetwork,
    /// VM is running and ready.
    Running,
    /// VM is stopped.
    Stopped,
    /// VM is being deleted.
    Deleting,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for GpuVmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scheduling => write!(f, "scheduling"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Starting => write!(f, "starting"),
            Self::ConfiguringNetwork => write!(f, "configuring_network"),
            Self::Running => write!(f, "running"),
            Self::Stopped => write!(f, "stopped"),
            Self::Deleting => write!(f, "deleting"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// GPU hardware specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpecs {
    /// GPU model (e.g., "H100", "L40S", "RTX 6000 Pro").
    pub gpu_model: String,
    /// Number of GPUs.
    pub gpu_count: u32,
    /// GPU memory in GB.
    pub gpu_memory_gb: Option<u32>,
    /// Number of virtual CPUs.
    pub vcpus: u32,
    /// RAM in GB.
    pub ram_gb: u32,
    /// Storage in GB.
    pub storage_gb: u32,
}

/// A GPU VM instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuVm {
    /// Unique VM identifier.
    pub id: String,
    /// VM name.
    pub name: String,
    /// Current status.
    pub status: GpuVmStatus,
    /// SSH host/IP address.
    pub host: Option<String>,
    /// SSH username.
    pub username: Option<String>,
    /// Plan ID.
    pub plan_id: String,
    /// GPU specifications.
    pub specs: Option<GpuSpecs>,
    /// When the VM was created.
    pub created_at: Option<DateTime<Utc>>,
}

/// A GPU VM plan (hardware configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPlan {
    /// Plan ID.
    pub id: String,
    /// Plan name.
    pub name: String,
    /// GPU specifications.
    pub specs: GpuSpecs,
    /// Hourly price in USD.
    pub price_per_hour: f64,
    /// Monthly price in USD.
    pub price_per_month: f64,
    /// Available regions/locations.
    pub available_regions: Vec<String>,
    /// Stock level (high, medium, low, unavailable).
    pub stock_level: String,
}

/// Request to create a new GPU VM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGpuVmRequest {
    /// VM name.
    pub name: String,
    /// Plan ID.
    pub plan_id: String,
    /// SSH key IDs to configure.
    pub ssh_keys: Vec<String>,
}

/// Trait for GPU cloud providers.
#[async_trait]
pub trait GpuProvider: Send + Sync {
    /// List all available GPU plans.
    async fn list_gpu_plans(&self) -> Result<Vec<GpuPlan>, GpuProviderError>;

    /// Create a new GPU VM.
    async fn create_gpu_vm(&self, req: CreateGpuVmRequest) -> Result<GpuVm, GpuProviderError>;

    /// Get a GPU VM by ID.
    async fn get_gpu_vm(&self, id: &str) -> Result<GpuVm, GpuProviderError>;

    /// List all GPU VMs.
    async fn list_gpu_vms(&self) -> Result<Vec<GpuVm>, GpuProviderError>;

    /// Delete a GPU VM.
    async fn delete_gpu_vm(&self, id: &str) -> Result<(), GpuProviderError>;

    /// Run a power action on a GPU VM.
    ///
    /// Actions: `power_on`, `power_off`, `reboot`
    async fn gpu_vm_action(&self, id: &str, action: &str) -> Result<(), GpuProviderError>;

    /// Wait for a GPU VM to reach running status.
    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<GpuVm, GpuProviderError>;
}




