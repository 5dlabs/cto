//! Kubernetes health probe handlers for liveness and readiness checks.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{HealthResponse, HealthStatus};

/// Global start time for tracking uptime
static START_TIME: AtomicU64 = AtomicU64::new(0);

/// Initialize the probe system with the current time.
/// Should be called once at application startup.
pub fn init() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    START_TIME.store(now, Ordering::SeqCst);
}

/// Get the current uptime in seconds.
fn get_uptime() -> u64 {
    let start = START_TIME.load(Ordering::SeqCst);
    if start == 0 {
        // If not initialized, initialize now
        init();
        return 0;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    now.saturating_sub(start)
}

/// Liveness probe handler for `/healthz` endpoint.
///
/// This probe indicates whether the application is running.
/// Kubernetes will restart the pod if this probe fails.
///
/// # Returns
///
/// A `HealthResponse` with status `Healthy` and current uptime.
pub fn liveness_probe() -> HealthResponse {
    HealthResponse::healthy(
        env!("CARGO_PKG_VERSION"),
        get_uptime(),
    )
}

/// Readiness probe handler for `/readyz` endpoint.
///
/// This probe indicates whether the application is ready to serve traffic.
/// Kubernetes will not send traffic to the pod if this probe fails.
///
/// # Returns
///
/// A `HealthResponse` with status `Healthy` and current uptime.
pub fn readiness_probe() -> HealthResponse {
    // In a real application, you would check:
    // - Database connections
    // - External service availability
    // - Cache readiness
    // - Any other dependencies

    HealthResponse::healthy(
        env!("CARGO_PKG_VERSION"),
        get_uptime(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveness_probe() {
        init();
        let response = liveness_probe();
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(!response.version.is_empty());
    }

    #[test]
    fn test_readiness_probe() {
        init();
        let response = readiness_probe();
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(!response.version.is_empty());
    }

    #[test]
    fn test_uptime_tracking() {
        init();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let uptime = get_uptime();
        assert!(uptime >= 0);
    }
}
