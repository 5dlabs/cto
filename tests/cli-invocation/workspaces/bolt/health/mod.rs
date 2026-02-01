//! Health check module for Kubernetes liveness and readiness probes.
//!
//! This module provides endpoints and types for Kubernetes health checks:
//! - `/healthz` - Liveness probe (indicates if the application is running)
//! - `/readyz` - Readiness probe (indicates if the application can serve traffic)
//!
//! # Example
//!
//! ```rust
//! use health::probes::{liveness_probe, readiness_probe};
//!
//! // Initialize the probe system at startup
//! health::probes::init();
//!
//! // Use in your HTTP handlers
//! let liveness = liveness_probe();
//! let readiness = readiness_probe();
//! ```

pub mod probes;
pub mod types;

// Re-export commonly used items
pub use probes::{liveness_probe, readiness_probe};
pub use types::{HealthResponse, HealthStatus};
