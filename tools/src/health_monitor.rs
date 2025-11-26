use crate::errors::{BridgeError, BridgeResult, ErrorContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};

/// Health status of a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerHealth {
    /// Server is healthy and responsive
    Healthy,
    /// Server is experiencing issues but still responsive
    Degraded { reason: String },
    /// Server is unresponsive
    Unresponsive {
        last_response: chrono::DateTime<chrono::Utc>,
    },
    /// Server has crashed and needs restart
    Crashed { exit_code: Option<i32> },
    /// Server is being restarted
    Restarting { attempt: u32 },
    /// Server is unknown/not monitored
    Unknown,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Timeout for health check requests
    pub check_timeout: Duration,
    /// Number of failed checks before marking as unhealthy
    pub failure_threshold: u32,
    /// Number of successful checks before marking as healthy again
    pub recovery_threshold: u32,
    /// Maximum restart attempts before giving up
    pub max_restart_attempts: u32,
    /// Backoff multiplier for restart attempts
    pub restart_backoff_multiplier: f32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            max_restart_attempts: 5,
            restart_backoff_multiplier: 2.0,
        }
    }
}

/// Server health status and metrics
#[derive(Debug, Clone)]
pub struct ServerHealthStatus {
    pub server_name: String,
    pub health: ServerHealth,
    pub last_check: Option<Instant>,
    pub last_successful_check: Option<Instant>,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub restart_attempts: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub uptime_start: Option<Instant>,
}

impl ServerHealthStatus {
    pub fn new(server_name: String) -> Self {
        Self {
            server_name,
            health: ServerHealth::Unknown,
            last_check: None,
            last_successful_check: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
            restart_attempts: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::from_millis(0),
            uptime_start: None,
        }
    }

    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate uptime duration
    pub fn uptime(&self) -> Option<Duration> {
        self.uptime_start.map(|start| start.elapsed())
    }

    /// Update health status based on a successful operation
    pub fn record_success(&mut self, response_time: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.consecutive_failures = 0;
        self.consecutive_successes += 1;
        self.last_successful_check = Some(Instant::now());

        // Update rolling average response time
        if self.average_response_time.is_zero() {
            self.average_response_time = response_time;
        } else {
            let total_time =
                self.average_response_time.as_millis() * (self.successful_requests - 1) as u128;
            let new_average =
                (total_time + response_time.as_millis()) / self.successful_requests as u128;
            self.average_response_time = Duration::from_millis(new_average as u64);
        }
    }

    /// Update health status based on a failed operation
    pub fn record_failure(&mut self, _error: &str) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.consecutive_successes = 0;
        self.consecutive_failures += 1;

        // Update health status based on failure pattern
        if self.consecutive_failures >= 3 {
            self.health = ServerHealth::Unresponsive {
                last_response: self
                    .last_successful_check
                    .map(|instant| {
                        chrono::Utc::now()
                            - chrono::Duration::from_std(instant.elapsed()).unwrap_or_default()
                    })
                    .unwrap_or_else(chrono::Utc::now),
            };
        }
    }
}

/// Comprehensive health monitoring system
pub struct HealthMonitor {
    /// Health status for each server
    server_health: Arc<RwLock<HashMap<String, ServerHealthStatus>>>,
    /// Health check configuration
    config: HealthCheckConfig,
    /// Active monitoring tasks
    monitoring_tasks: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
    /// Shutdown signal
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl HealthMonitor {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            server_health: Arc::new(RwLock::new(HashMap::new())),
            config,
            monitoring_tasks: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Start monitoring a server
    pub async fn start_monitoring_server(&mut self, server_name: String) -> BridgeResult<()> {
        // Initialize health status
        {
            let mut health_map = self.server_health.write().await;
            health_map.insert(
                server_name.clone(),
                ServerHealthStatus::new(server_name.clone()),
            );
        }

        // Create shutdown channel if not exists
        if self.shutdown_tx.is_none() {
            let (tx, _) = tokio::sync::broadcast::channel(1);
            self.shutdown_tx = Some(tx);
        }

        let shutdown_rx = self.shutdown_tx.as_ref().unwrap().subscribe();

        // Start monitoring task
        let health_map = Arc::clone(&self.server_health);
        let config = self.config.clone();
        let server_name_clone = server_name.clone();

        let task = tokio::spawn(async move {
            Self::monitor_server_loop(server_name_clone, health_map, config, shutdown_rx).await;
        });

        // Store the task handle
        let mut tasks = self.monitoring_tasks.lock().await;
        tasks.insert(server_name.clone(), task);

        tracing::info!("🔍 Started health monitoring for server: {server_name}");
        Ok(())
    }

    /// Stop monitoring a server
    pub async fn stop_monitoring_server(&mut self, server_name: &str) -> BridgeResult<()> {
        let mut tasks = self.monitoring_tasks.lock().await;
        if let Some(task) = tasks.remove(server_name) {
            task.abort();
            tracing::info!("🛑 Stopped health monitoring for server: {server_name}");
        }

        // Remove from health map
        let mut health_map = self.server_health.write().await;
        health_map.remove(server_name);

        Ok(())
    }

    /// Get health status for a specific server
    pub async fn get_server_health(&self, server_name: &str) -> Option<ServerHealthStatus> {
        let health_map = self.server_health.read().await;
        health_map.get(server_name).cloned()
    }

    /// Get health status for all monitored servers
    pub async fn get_all_health_status(&self) -> HashMap<String, ServerHealthStatus> {
        let health_map = self.server_health.read().await;
        health_map.clone()
    }

    /// Record a successful operation for a server
    pub async fn record_success(&self, server_name: &str, response_time: Duration) {
        let mut health_map = self.server_health.write().await;
        if let Some(status) = health_map.get_mut(server_name) {
            status.record_success(response_time);

            // Update health status if recovering
            if status.consecutive_successes >= self.config.recovery_threshold {
                match status.health {
                    ServerHealth::Degraded { .. } | ServerHealth::Unresponsive { .. } => {
                        status.health = ServerHealth::Healthy;
                        status.uptime_start = Some(Instant::now());
                        tracing::info!(
                            "✅ Server '{server_name}' has recovered and is now healthy"
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    /// Record a failed operation for a server
    pub async fn record_failure(&self, server_name: &str, error: &str) {
        let mut health_map = self.server_health.write().await;
        if let Some(status) = health_map.get_mut(server_name) {
            status.record_failure(error);

            // Check if we need to trigger recovery
            if status.consecutive_failures >= self.config.failure_threshold {
                tracing::info!(
                    "⚠️ Server '{server_name}' marked as unhealthy after {} consecutive failures",
                    status.consecutive_failures
                );
            }
        }
    }

    /// Check if a server should be restarted based on health
    pub async fn should_restart_server(&self, server_name: &str) -> bool {
        let health_map = self.server_health.read().await;
        if let Some(status) = health_map.get(server_name) {
            match &status.health {
                ServerHealth::Crashed { .. } => {
                    status.restart_attempts < self.config.max_restart_attempts
                }
                ServerHealth::Unresponsive { .. } => {
                    status.consecutive_failures >= self.config.failure_threshold
                        && status.restart_attempts < self.config.max_restart_attempts
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Mark a server as crashed
    pub async fn mark_server_crashed(&self, server_name: &str, exit_code: Option<i32>) {
        let mut health_map = self.server_health.write().await;
        if let Some(status) = health_map.get_mut(server_name) {
            status.health = ServerHealth::Crashed { exit_code };
            status.uptime_start = None;
            tracing::info!(
                "💥 Server '{server_name}' marked as crashed (exit code: {exit_code:?})"
            );
        }
    }

    /// Mark a server as being restarted
    pub async fn mark_server_restarting(&self, server_name: &str) {
        let mut health_map = self.server_health.write().await;
        if let Some(status) = health_map.get_mut(server_name) {
            status.restart_attempts += 1;
            status.health = ServerHealth::Restarting {
                attempt: status.restart_attempts,
            };
            tracing::info!(
                "🔄 Server '{server_name}' restart attempt #{}",
                status.restart_attempts
            );
        }
    }

    /// Mark a server as healthy after successful restart
    pub async fn mark_server_healthy(&self, server_name: &str) {
        let mut health_map = self.server_health.write().await;
        if let Some(status) = health_map.get_mut(server_name) {
            status.health = ServerHealth::Healthy;
            status.consecutive_failures = 0;
            status.consecutive_successes = 1;
            status.uptime_start = Some(Instant::now());
            tracing::info!("✅ Server '{server_name}' marked as healthy after restart");
        }
    }

    /// Get servers that need attention (unhealthy, restarting, etc.)
    pub async fn get_servers_needing_attention(&self) -> Vec<(String, ServerHealthStatus)> {
        let health_map = self.server_health.read().await;
        health_map
            .iter()
            .filter(|(_, status)| {
                matches!(
                    status.health,
                    ServerHealth::Degraded { .. }
                        | ServerHealth::Unresponsive { .. }
                        | ServerHealth::Crashed { .. }
                        | ServerHealth::Restarting { .. }
                )
            })
            .map(|(name, status)| (name.clone(), status.clone()))
            .collect()
    }

    /// Internal monitoring loop for a server
    async fn monitor_server_loop(
        server_name: String,
        health_map: Arc<RwLock<HashMap<String, ServerHealthStatus>>>,
        config: HealthCheckConfig,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) {
        let mut interval = tokio::time::interval(config.check_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Perform health check
                    let start_time = Instant::now();
                    let health_check_result = Self::perform_health_check(&server_name, config.check_timeout).await;
                    let response_time = start_time.elapsed();

                    // Update health status
                    let mut health_map = health_map.write().await;
                    if let Some(status) = health_map.get_mut(&server_name) {
                        status.last_check = Some(Instant::now());

                        match health_check_result {
                            Ok(()) => {
                                status.record_success(response_time);
                            }
                            Err(error) => {
                                status.record_failure(&error.error.to_string());
                            }
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("🛑 Shutting down health monitor for server: {server_name}");
                    break;
                }
            }
        }
    }

    /// Perform a health check on a server
    async fn perform_health_check(server_name: &str, timeout: Duration) -> BridgeResult<()> {
        // This is a simple ping-style health check
        // In a real implementation, this would send a tools/list request to the server
        // For now, we'll simulate a basic health check

        let check_future = async {
            // Simulate health check logic
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Simulate occasional failures for testing
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            server_name.hash(&mut hasher);
            let hash = hasher.finish();

            if hash.is_multiple_of(50) {
                // 2% failure rate for simulation
                Err(Box::new(ErrorContext::new(
                    BridgeError::HealthCheckFailed {
                        server: server_name.to_string(),
                        reason: "Simulated health check failure".to_string(),
                    },
                )))
            } else {
                Ok(())
            }
        };

        match tokio::time::timeout(timeout, check_future).await {
            Ok(result) => result,
            Err(_) => Err(Box::new(ErrorContext::new(BridgeError::ServerTimeout {
                name: server_name.to_string(),
                timeout_secs: timeout.as_secs(),
            }))),
        }
    }

    /// Shutdown all monitoring
    pub async fn shutdown(&mut self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Wait for all tasks to complete
        let mut tasks = self.monitoring_tasks.lock().await;
        for (server_name, task) in tasks.drain() {
            task.abort();
            let _ = task.await;
            tracing::info!("🛑 Shutdown monitoring for server: {server_name}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_health_status_success_rate() {
        let mut status = ServerHealthStatus::new("test".to_string());

        status.record_success(Duration::from_millis(100));
        status.record_success(Duration::from_millis(200));
        status.record_failure("test error");

        assert!((status.success_rate() - 66.66666666666667).abs() < 0.00001);
        assert_eq!(status.total_requests, 3);
        assert_eq!(status.successful_requests, 2);
        assert_eq!(status.failed_requests, 1);
    }

    #[tokio::test]
    async fn test_health_monitor_basic_operations() {
        let config = HealthCheckConfig::default();
        let mut monitor = HealthMonitor::new(config);

        // Start monitoring
        monitor
            .start_monitoring_server("test_server".to_string())
            .await
            .unwrap();

        // Record some operations
        monitor
            .record_success("test_server", Duration::from_millis(50))
            .await;
        monitor.record_failure("test_server", "test error").await;

        // Check health status
        let health = monitor.get_server_health("test_server").await;
        assert!(health.is_some());

        let status = health.unwrap();
        assert_eq!(status.server_name, "test_server");
        assert_eq!(status.total_requests, 2);

        // Stop monitoring
        monitor.stop_monitoring_server("test_server").await.unwrap();

        // Shutdown
        monitor.shutdown().await;
    }
}
