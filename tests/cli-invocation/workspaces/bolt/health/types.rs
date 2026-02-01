//! Health check response types for Kubernetes probes.

use serde::{Deserialize, Serialize};

/// Health status of the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy and operating normally
    Healthy,
    /// Service is unhealthy
    Unhealthy,
}

/// Health check response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Current health status
    pub status: HealthStatus,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime: u64,
}

impl HealthResponse {
    /// Creates a new healthy response.
    ///
    /// # Arguments
    ///
    /// * `version` - The service version string
    /// * `uptime` - The service uptime in seconds
    pub fn healthy(version: impl Into<String>, uptime: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            version: version.into(),
            uptime,
        }
    }

    /// Creates a new unhealthy response.
    ///
    /// # Arguments
    ///
    /// * `version` - The service version string
    /// * `uptime` - The service uptime in seconds
    pub fn unhealthy(version: impl Into<String>, uptime: u64) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            version: version.into(),
            uptime,
        }
    }
}
