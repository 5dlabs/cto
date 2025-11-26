use crate::errors::{BridgeError, BridgeResult, ErrorContext, RecoveryStrategy};
use crate::health_monitor::{HealthCheckConfig, HealthMonitor};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};

/// Configuration for recovery strategies
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum time to wait for a server restart
    pub restart_timeout: Duration,
    /// Base delay before first restart attempt
    pub restart_base_delay: Duration,
    /// Maximum delay between restart attempts
    pub restart_max_delay: Duration,
    /// Exponential backoff multiplier
    pub restart_backoff_multiplier: f32,
    /// Circuit breaker threshold - stop trying after this many failures
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker reset time - try again after this duration
    pub circuit_breaker_reset: Duration,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            restart_timeout: Duration::from_secs(30),
            restart_base_delay: Duration::from_secs(1),
            restart_max_delay: Duration::from_secs(60),
            restart_backoff_multiplier: 2.0,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Server connection information for recovery
#[derive(Debug, Clone)]
pub struct ServerConnectionInfo {
    pub server_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub process_id: Option<u32>,
    pub last_restart: Option<Instant>,
    pub circuit_breaker_trips: u32,
    pub circuit_breaker_opened_at: Option<Instant>,
}

/// Recovery actions that can be taken
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// No action needed
    None,
    /// Retry the operation with delay
    RetryWithDelay { delay: Duration },
    /// Restart the server
    RestartServer { server_name: String },
    /// Switch to fallback server
    SwitchToFallback { primary: String, fallback: String },
    /// Mark server as failed and stop trying
    MarkAsFailed { server_name: String, reason: String },
    /// Manual intervention required
    RequireManualIntervention { message: String },
}

/// Comprehensive server recovery system
pub struct ServerRecoveryManager {
    /// Health monitoring system
    health_monitor: Arc<Mutex<HealthMonitor>>,
    /// Recovery configuration
    recovery_config: RecoveryConfig,
    /// Server connection information
    server_connections: Arc<RwLock<HashMap<String, ServerConnectionInfo>>>,
    /// Active recovery tasks
    recovery_tasks: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
    /// Circuit breaker states
    circuit_breakers: Arc<RwLock<HashMap<String, bool>>>,
    /// Fallback server mappings
    fallback_mappings: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ServerRecoveryManager {
    pub fn new(recovery_config: RecoveryConfig, health_config: HealthCheckConfig) -> Self {
        let health_monitor = HealthMonitor::new(health_config);

        Self {
            health_monitor: Arc::new(Mutex::new(health_monitor)),
            recovery_config,
            server_connections: Arc::new(RwLock::new(HashMap::new())),
            recovery_tasks: Arc::new(Mutex::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            fallback_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a server for monitoring and recovery
    pub async fn register_server(
        &self,
        server_name: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> BridgeResult<()> {
        // Register with health monitor
        {
            let mut health_monitor = self.health_monitor.lock().await;
            health_monitor
                .start_monitoring_server(server_name.clone())
                .await?;
        }

        // Store connection info
        let connection_info = ServerConnectionInfo {
            server_name: server_name.clone(),
            command,
            args,
            env,
            process_id: None,
            last_restart: None,
            circuit_breaker_trips: 0,
            circuit_breaker_opened_at: None,
        };

        {
            let mut connections = self.server_connections.write().await;
            connections.insert(server_name.clone(), connection_info);
        }

        // Initialize circuit breaker
        {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers.insert(server_name.clone(), false);
        }

        tracing::info!("📋 Registered server '{server_name}' for monitoring and recovery");
        Ok(())
    }

    /// Unregister a server from monitoring
    pub async fn unregister_server(&self, server_name: &str) -> BridgeResult<()> {
        // Stop health monitoring
        {
            let mut health_monitor = self.health_monitor.lock().await;
            health_monitor.stop_monitoring_server(server_name).await?;
        }

        // Remove connection info
        {
            let mut connections = self.server_connections.write().await;
            connections.remove(server_name);
        }

        // Remove circuit breaker
        {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers.remove(server_name);
        }

        // Stop any active recovery tasks
        {
            let mut tasks = self.recovery_tasks.lock().await;
            if let Some(task) = tasks.remove(server_name) {
                task.abort();
            }
        }

        tracing::info!("📋 Unregistered server '{server_name}' from monitoring");
        Ok(())
    }

    /// Handle an error and determine recovery action
    pub async fn handle_error(&self, error_context: &ErrorContext) -> RecoveryAction {
        match &error_context.recovery_strategy {
            RecoveryStrategy::None => RecoveryAction::None,

            RecoveryStrategy::Retry {
                max_attempts: _,
                backoff_ms,
            } => {
                let delay = Duration::from_millis(*backoff_ms);
                RecoveryAction::RetryWithDelay { delay }
            }

            RecoveryStrategy::RestartServer { server_name } => {
                // Check circuit breaker
                if self.is_circuit_breaker_open(server_name).await {
                    return RecoveryAction::RequireManualIntervention {
                        message: format!(
                            "Server '{server_name}' circuit breaker is open. Too many failures detected."
                        ),
                    };
                }

                // Check if restart is needed and allowed
                let health_monitor = self.health_monitor.lock().await;
                if health_monitor.should_restart_server(server_name).await {
                    RecoveryAction::RestartServer {
                        server_name: server_name.clone(),
                    }
                } else {
                    RecoveryAction::MarkAsFailed {
                        server_name: server_name.clone(),
                        reason: "Maximum restart attempts exceeded".to_string(),
                    }
                }
            }

            RecoveryStrategy::UseFallback { fallback_servers } => {
                if let Some(fallback) = fallback_servers.first() {
                    // Try to find which server this error is for
                    if let BridgeError::ServerConnectionLost { name, .. }
                    | BridgeError::ServerCrashed { name, .. }
                    | BridgeError::ServerTimeout { name, .. } = &error_context.error
                    {
                        RecoveryAction::SwitchToFallback {
                            primary: name.clone(),
                            fallback: fallback.clone(),
                        }
                    } else {
                        RecoveryAction::RequireManualIntervention {
                            message: "Fallback requested but cannot determine primary server"
                                .to_string(),
                        }
                    }
                } else {
                    RecoveryAction::RequireManualIntervention {
                        message: "No fallback servers available".to_string(),
                    }
                }
            }

            RecoveryStrategy::ManualIntervention { message } => {
                RecoveryAction::RequireManualIntervention {
                    message: message.clone(),
                }
            }

            RecoveryStrategy::GracefulDegradation { feature } => {
                RecoveryAction::RequireManualIntervention {
                    message: format!("Graceful degradation: {feature} disabled"),
                }
            }
        }
    }

    /// Execute a recovery action
    pub async fn execute_recovery_action(&self, action: RecoveryAction) -> BridgeResult<()> {
        match action {
            RecoveryAction::None => {
                // No action needed
                Ok(())
            }

            RecoveryAction::RetryWithDelay { delay } => {
                tracing::info!("⏱️ Delaying retry for {delay:?}");
                tokio::time::sleep(delay).await;
                Ok(())
            }

            RecoveryAction::RestartServer { server_name } => {
                self.restart_server(&server_name).await
            }

            RecoveryAction::SwitchToFallback { primary, fallback } => {
                self.switch_to_fallback(&primary, &fallback).await
            }

            RecoveryAction::MarkAsFailed {
                server_name,
                reason,
            } => self.mark_server_as_failed(&server_name, &reason).await,

            RecoveryAction::RequireManualIntervention { message } => {
                tracing::info!("🚨 Manual intervention required: {message}");
                Ok(())
            }
        }
    }

    /// Restart a server with exponential backoff
    async fn restart_server(&self, server_name: &str) -> BridgeResult<()> {
        let connection_info = {
            let connections = self.server_connections.read().await;
            connections.get(server_name).cloned()
        };

        let mut connection_info = match connection_info {
            Some(info) => info,
            None => {
                return Err(Box::new(ErrorContext::new(BridgeError::ServerNotFound {
                    name: server_name.to_string(),
                })));
            }
        };

        // Calculate backoff delay
        let restart_attempt = {
            let health_monitor = self.health_monitor.lock().await;
            if let Some(status) = health_monitor.get_server_health(server_name).await {
                status.restart_attempts
            } else {
                0
            }
        };

        let delay_multiplier = self
            .recovery_config
            .restart_backoff_multiplier
            .powi(restart_attempt as i32);
        let delay = std::cmp::min(
            Duration::from_millis(
                (self.recovery_config.restart_base_delay.as_millis() as f32 * delay_multiplier)
                    as u64,
            ),
            self.recovery_config.restart_max_delay,
        );

        if delay > Duration::from_millis(0) {
            tracing::info!(
                "⏱️ Waiting {:?} before restart attempt #{} for server '{}'",
                delay,
                restart_attempt + 1,
                server_name
            );
            tokio::time::sleep(delay).await;
        }

        // Mark as restarting
        {
            let health_monitor = self.health_monitor.lock().await;
            health_monitor.mark_server_restarting(server_name).await;
        }

        tracing::info!("🔄 Attempting to restart server: {server_name}");

        // Spawn the new process
        let mut cmd = Command::new(&connection_info.command);
        cmd.args(&connection_info.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        cmd.envs(std::env::vars()); // Inherit parent environment
        for (key, value) in &connection_info.env {
            cmd.env(key, value);
        }

        match cmd.spawn() {
            Ok(process) => {
                connection_info.process_id = process.id();
                connection_info.last_restart = Some(Instant::now());

                // Update connection info
                {
                    let mut connections = self.server_connections.write().await;
                    connections.insert(server_name.to_string(), connection_info);
                }

                // Mark as healthy
                {
                    let health_monitor = self.health_monitor.lock().await;
                    health_monitor.mark_server_healthy(server_name).await;
                }

                tracing::info!(
                    "✅ Successfully restarted server '{}' (PID: {:?})",
                    server_name,
                    process.id()
                );
                Ok(())
            }
            Err(e) => {
                // Restart failed, increment circuit breaker
                self.increment_circuit_breaker(server_name).await;

                Err(Box::new(ErrorContext::new(
                    BridgeError::ServerRestartFailed {
                        server: server_name.to_string(),
                        reason: e.to_string(),
                    },
                )))
            }
        }
    }

    /// Switch to a fallback server
    async fn switch_to_fallback(&self, primary: &str, fallback: &str) -> BridgeResult<()> {
        tracing::info!("🔄 Switching from primary server '{primary}' to fallback '{fallback}'");

        // Update fallback mappings
        {
            let mut mappings = self.fallback_mappings.write().await;
            mappings
                .entry(primary.to_string())
                .or_insert_with(Vec::new)
                .push(fallback.to_string());
        }

        // For now, this is a placeholder - in a real implementation,
        // you'd need to reroute requests to the fallback server
        tracing::info!("✅ Configured fallback: {primary} -> {fallback}");
        Ok(())
    }

    /// Mark a server as permanently failed
    async fn mark_server_as_failed(&self, server_name: &str, reason: &str) -> BridgeResult<()> {
        tracing::info!("💀 Marking server '{server_name}' as failed: {reason}");

        // Open circuit breaker permanently
        {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers.insert(server_name.to_string(), true);
        }

        // Update connection info
        {
            let mut connections = self.server_connections.write().await;
            if let Some(connection_info) = connections.get_mut(server_name) {
                connection_info.circuit_breaker_opened_at = Some(Instant::now());
            }
        }

        Ok(())
    }

    /// Check if circuit breaker is open for a server
    async fn is_circuit_breaker_open(&self, server_name: &str) -> bool {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(&is_open) = circuit_breakers.get(server_name) {
            if is_open {
                // Check if it's time to reset
                let connections = self.server_connections.read().await;
                if let Some(connection_info) = connections.get(server_name) {
                    if let Some(opened_at) = connection_info.circuit_breaker_opened_at {
                        return opened_at.elapsed() < self.recovery_config.circuit_breaker_reset;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Increment circuit breaker failure count
    async fn increment_circuit_breaker(&self, server_name: &str) {
        let mut connections = self.server_connections.write().await;
        if let Some(connection_info) = connections.get_mut(server_name) {
            connection_info.circuit_breaker_trips += 1;

            if connection_info.circuit_breaker_trips
                >= self.recovery_config.circuit_breaker_threshold
            {
                connection_info.circuit_breaker_opened_at = Some(Instant::now());

                let mut circuit_breakers = self.circuit_breakers.write().await;
                circuit_breakers.insert(server_name.to_string(), true);

                tracing::info!(
                    "🔴 Circuit breaker opened for server '{}' after {} failures",
                    server_name,
                    connection_info.circuit_breaker_trips
                );
            }
        }
    }

    /// Record a successful operation (resets circuit breaker)
    pub async fn record_success(&self, server_name: &str, response_time: Duration) {
        // Reset circuit breaker on success
        {
            let mut connections = self.server_connections.write().await;
            if let Some(connection_info) = connections.get_mut(server_name) {
                connection_info.circuit_breaker_trips = 0;
                connection_info.circuit_breaker_opened_at = None;
            }
        }

        {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers.insert(server_name.to_string(), false);
        }

        // Record with health monitor
        {
            let health_monitor = self.health_monitor.lock().await;
            health_monitor
                .record_success(server_name, response_time)
                .await;
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self, server_name: &str, error: &str) {
        // Record with health monitor
        {
            let health_monitor = self.health_monitor.lock().await;
            health_monitor.record_failure(server_name, error).await;
        }
    }

    /// Get recovery status for all servers
    pub async fn get_recovery_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();

        let connections = self.server_connections.read().await;
        let circuit_breakers = self.circuit_breakers.read().await;

        for (server_name, connection_info) in connections.iter() {
            let circuit_open = circuit_breakers.get(server_name).copied().unwrap_or(false);

            let server_status = if circuit_open {
                format!(
                    "Circuit breaker open ({} trips)",
                    connection_info.circuit_breaker_trips
                )
            } else if let Some(last_restart) = connection_info.last_restart {
                format!("Healthy (last restart: {:?} ago)", last_restart.elapsed())
            } else {
                "Healthy".to_string()
            };

            status.insert(server_name.clone(), server_status);
        }

        status
    }

    /// Shutdown the recovery manager
    pub async fn shutdown(&mut self) {
        tracing::info!("🛑 Shutting down server recovery manager");

        // Stop all recovery tasks
        let mut tasks = self.recovery_tasks.lock().await;
        for (server_name, task) in tasks.drain() {
            task.abort();
            let _ = task.await;
            tracing::info!("🛑 Stopped recovery task for server: {server_name}");
        }

        // Shutdown health monitor
        let mut health_monitor = self.health_monitor.lock().await;
        health_monitor.shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_monitor::HealthCheckConfig;

    #[tokio::test]
    async fn test_recovery_manager_basic_operations() {
        let recovery_config = RecoveryConfig::default();
        let health_config = HealthCheckConfig::default();
        let mut manager = ServerRecoveryManager::new(recovery_config, health_config);

        // Register a server
        manager
            .register_server(
                "test_server".to_string(),
                "echo".to_string(),
                vec!["hello".to_string()],
                HashMap::new(),
            )
            .await
            .unwrap();

        // Record success and failure
        manager
            .record_success("test_server", Duration::from_millis(100))
            .await;
        manager.record_failure("test_server", "test error").await;

        // Get status
        let status = manager.get_recovery_status().await;
        assert!(status.contains_key("test_server"));

        // Unregister
        manager.unregister_server("test_server").await.unwrap();

        // Shutdown
        manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_error_handling() {
        let recovery_config = RecoveryConfig::default();
        let health_config = HealthCheckConfig::default();
        let manager = ServerRecoveryManager::new(recovery_config, health_config);

        let error = Box::new(ErrorContext::new(BridgeError::ServerTimeout {
            name: "test_server".to_string(),
            timeout_secs: 5,
        }));

        let action = manager.handle_error(&error).await;

        match action {
            RecoveryAction::RetryWithDelay { .. } => {
                // Expected for timeout errors
            }
            _ => panic!("Unexpected recovery action: {action:?}"),
        }
    }
}
