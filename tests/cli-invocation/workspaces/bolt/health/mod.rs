//! Kubernetes health check module
//!
//! This module provides liveness and readiness probe implementations for Kubernetes deployments.
//!
//! # Examples
//!
//! ```no_run
//! use health::probes::{ServiceState, liveness_probe, readiness_probe};
//!
//! let state = ServiceState::new("1.0.0");
//!
//! // Liveness probe - checks if service is alive
//! let (status, response) = liveness_probe(&state);
//!
//! // Readiness probe - checks if service is ready to handle traffic
//! let (status, response) = readiness_probe(&state);
//! ```

pub mod probes;
pub mod types;

// Re-export commonly used items
pub use probes::{liveness_probe, readiness_probe, ServiceState};
pub use types::{HealthResponse, HealthStatus};
