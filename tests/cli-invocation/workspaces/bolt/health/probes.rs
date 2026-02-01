use crate::health::types::{HealthResponse, HealthStatus};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Service state tracking for health checks
pub struct ServiceState {
    start_time: Instant,
    version: String,
    ready: Arc<std::sync::atomic::AtomicBool>,
}

impl ServiceState {
    /// Creates a new service state tracker
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            start_time: Instant::now(),
            version: version.into(),
            ready: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Sets the readiness state
    pub fn set_ready(&self, ready: bool) {
        self.ready
            .store(ready, std::sync::atomic::Ordering::SeqCst);
    }

    /// Checks if the service is ready
    pub fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Gets the uptime in seconds
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Gets the version string
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Liveness probe handler
///
/// Returns 200 OK when the service is alive (basic process check).
/// This endpoint should return healthy unless the process is deadlocked or crashed.
pub fn liveness_probe(state: &ServiceState) -> (u16, HealthResponse) {
    let response = HealthResponse::healthy(state.version().to_string(), state.uptime());
    (200, response)
}

/// Readiness probe handler
///
/// Returns 200 OK when the service is ready to handle requests.
/// This checks if dependencies are available and the service is fully initialized.
pub fn readiness_probe(state: &ServiceState) -> (u16, HealthResponse) {
    if state.is_ready() {
        let response = HealthResponse::healthy(state.version().to_string(), state.uptime());
        (200, response)
    } else {
        let response = HealthResponse::unhealthy(state.version().to_string(), state.uptime());
        (503, response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveness_probe_returns_healthy() {
        let state = ServiceState::new("1.0.0");
        let (status, response) = liveness_probe(&state);

        assert_eq!(status, 200);
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.version, "1.0.0");
    }

    #[test]
    fn test_readiness_probe_when_ready() {
        let state = ServiceState::new("1.0.0");
        state.set_ready(true);
        let (status, response) = readiness_probe(&state);

        assert_eq!(status, 200);
        assert_eq!(response.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_readiness_probe_when_not_ready() {
        let state = ServiceState::new("1.0.0");
        state.set_ready(false);
        let (status, response) = readiness_probe(&state);

        assert_eq!(status, 503);
        assert_eq!(response.status, HealthStatus::Unhealthy);
    }
}
