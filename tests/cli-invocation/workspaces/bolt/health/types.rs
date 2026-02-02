use serde::{Deserialize, Serialize};

/// Health status of the service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy and ready
    Healthy,
    /// Service is unhealthy or not ready
    Unhealthy,
}

/// Health check response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Current health status
    pub status: HealthStatus,
    /// Service version
    pub version: String,
    /// Service uptime in seconds
    pub uptime: u64,
}

impl HealthResponse {
    /// Creates a new healthy response
    pub fn healthy(version: String, uptime: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            version,
            uptime,
        }
    }

    /// Creates a new unhealthy response
    pub fn unhealthy(version: String, uptime: u64) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            version,
            uptime,
        }
    }
}
