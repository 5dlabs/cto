//! Adapter Factory Pattern
//!
//! Manages CLI adapter lifecycle, registration, and creation with health monitoring
//! and dynamic adapter discovery.

use crate::cli::adapter::{AdapterError, AdapterResult, CliAdapter, HealthState, HealthStatus};
use crate::cli::adapters::{
    ClaudeAdapter, CodexAdapter, CursorAdapter, FactoryAdapter, GeminiAdapter, OpenCodeAdapter,
};
use crate::cli::base_adapter::AdapterConfig;
use crate::cli::types::CLIType;
#[cfg(test)]
use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Factory for creating and managing CLI adapters
#[derive(Debug)]
pub struct AdapterFactory {
    /// Registered adapters by CLI type
    adapters: Arc<DashMap<CLIType, Arc<dyn CliAdapter>>>,
    /// Configuration registry for adapters
    config_registry: Arc<RwLock<ConfigRegistry>>,
    /// Health monitor for tracking adapter health
    health_monitor: Arc<HealthMonitor>,
    /// Factory configuration
    config: FactoryConfig,
}

/// Configuration for the adapter factory
#[derive(Debug, Clone)]
pub struct FactoryConfig {
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum unhealthy duration before adapter is disabled
    pub max_unhealthy_duration: Duration,
    /// Enable automatic health monitoring
    pub enable_health_monitoring: bool,
    /// Maximum concurrent adapter creations
    pub max_concurrent_creations: usize,
}

impl Default for FactoryConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(60),
            max_unhealthy_duration: Duration::from_secs(300), // 5 minutes
            enable_health_monitoring: true,
            max_concurrent_creations: 10,
        }
    }
}

/// Configuration registry for adapter settings
#[derive(Debug, Default)]
pub struct ConfigRegistry {
    /// Default configurations for each CLI type
    defaults: HashMap<CLIType, AdapterConfig>,
    /// Override configurations
    overrides: HashMap<String, AdapterConfig>,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default configuration for a CLI type
    pub fn set_default(&mut self, cli_type: CLIType, config: AdapterConfig) {
        self.defaults.insert(cli_type, config);
    }

    /// Get default configuration for a CLI type
    #[must_use]
    pub fn get_default(&self, cli_type: CLIType) -> Option<&AdapterConfig> {
        self.defaults.get(&cli_type)
    }

    /// Set override configuration
    pub fn set_override(&mut self, key: String, config: AdapterConfig) {
        self.overrides.insert(key, config);
    }

    /// Get override configuration
    #[must_use]
    pub fn get_override(&self, key: &str) -> Option<&AdapterConfig> {
        self.overrides.get(key)
    }
}

/// Health monitor for tracking adapter health over time
#[derive(Debug)]
pub struct HealthMonitor {
    /// Health history for each adapter
    health_history: Arc<DashMap<CLIType, Vec<HealthCheckRecord>>>,
    /// Configuration for health monitoring
    config: HealthMonitorConfig,
}

/// Configuration for health monitoring
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Maximum number of health records to keep
    pub max_history_size: usize,
    /// Minimum consecutive failures before marking as unhealthy
    pub failure_threshold: usize,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            failure_threshold: 3,
        }
    }
}

/// Record of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckRecord {
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Health status at the time
    pub status: HealthState,
    /// Duration of the health check
    pub check_duration: Duration,
    /// Optional error message
    pub error: Option<String>,
}

impl AdapterFactory {
    /// Create a new adapter factory
    ///
    /// # Errors
    /// Returns error if adapter creation fails
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(FactoryConfig::default()).await
    }

    /// Create a new adapter factory with custom configuration
    ///
    /// # Errors
    /// Returns error if adapter creation fails
    pub async fn with_config(config: FactoryConfig) -> AdapterResult<Self> {
        let health_monitor = Arc::new(HealthMonitor::new(HealthMonitorConfig::default()));

        let factory = Self {
            adapters: Arc::new(DashMap::new()),
            config_registry: Arc::new(RwLock::new(ConfigRegistry::default())),
            health_monitor,
            config,
        };

        // Register built-in adapters
        factory.register_default_adapters().await?;

        // Start health monitoring if enabled
        if factory.config.enable_health_monitoring {
            factory.start_health_monitoring();
        }

        info!(
            "Adapter factory initialized with {} CLI types supported",
            factory.get_supported_clis().len()
        );

        Ok(factory)
    }

    /// Register built-in adapters that are always available
    async fn register_default_adapters(&self) -> AdapterResult<()> {
        let claude_adapter = Arc::new(ClaudeAdapter::new()?);
        self.register_adapter(CLIType::Claude, claude_adapter)
            .await?;

        let codex_adapter = Arc::new(CodexAdapter::new()?);
        self.register_adapter(CLIType::Codex, codex_adapter).await?;

        let cursor_adapter = Arc::new(CursorAdapter::new()?);
        self.register_adapter(CLIType::Cursor, cursor_adapter)
            .await?;

        let factory_adapter = Arc::new(FactoryAdapter::new()?);
        self.register_adapter(CLIType::Factory, factory_adapter)
            .await?;

        let gemini_adapter = Arc::new(GeminiAdapter::new()?);
        self.register_adapter(CLIType::Gemini, gemini_adapter)
            .await?;

        let opencode_adapter = Arc::new(OpenCodeAdapter::new()?);
        self.register_adapter(CLIType::OpenCode, opencode_adapter)
            .await?;

        Ok(())
    }

    /// Register a CLI adapter
    #[instrument(skip(self, adapter))]
    pub async fn register_adapter(
        &self,
        cli_type: CLIType,
        adapter: Arc<dyn CliAdapter>,
    ) -> AdapterResult<()> {
        info!(cli_type = %cli_type, "Registering CLI adapter");

        // Validate adapter before registration
        self.validate_adapter(&adapter).await?;

        // Register the adapter
        self.adapters.insert(cli_type, adapter.clone());

        // Initialize health monitoring for this adapter
        if self.config.enable_health_monitoring {
            self.health_monitor.initialize_monitoring(cli_type);
        }

        info!(cli_type = %cli_type, "CLI adapter registered successfully");
        Ok(())
    }

    /// Create an adapter instance
    #[instrument(skip(self))]
    pub async fn create(&self, cli_type: CLIType) -> AdapterResult<Arc<dyn CliAdapter>> {
        debug!(cli_type = %cli_type, "Creating CLI adapter instance");

        // Get the registered adapter
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or_else(|| {
                error!(cli_type = %cli_type, "No adapter registered for CLI type");
                AdapterError::UnsupportedCliType(cli_type.to_string())
            })?
            .clone();

        // Check adapter health before returning
        match adapter.health_check().await {
            Ok(health) => match health.status {
                HealthState::Healthy | HealthState::Warning => {
                    debug!(cli_type = %cli_type, status = ?health.status, "Adapter health check passed");
                }
                HealthState::Unhealthy => {
                    warn!(
                        cli_type = %cli_type,
                        message = ?health.message,
                        "Adapter is unhealthy but will be returned anyway"
                    );
                }
                HealthState::Unknown => {
                    debug!(cli_type = %cli_type, "Adapter health is unknown");
                }
            },
            Err(e) => {
                warn!(
                    cli_type = %cli_type,
                    error = %e,
                    "Health check failed for adapter, returning anyway"
                );
            }
        }

        info!(cli_type = %cli_type, "CLI adapter instance created successfully");
        Ok(adapter)
    }

    /// Get all supported CLI types
    #[must_use]
    pub fn get_supported_clis(&self) -> Vec<CLIType> {
        self.adapters.iter().map(|entry| *entry.key()).collect()
    }

    /// Check if a CLI type is supported
    #[must_use]
    pub fn supports_cli(&self, cli_type: CLIType) -> bool {
        self.adapters.contains_key(&cli_type)
    }

    /// Get health status for all adapters
    #[instrument(skip(self))]
    pub async fn get_health_summary(&self) -> HashMap<CLIType, HealthStatus> {
        let mut health_summary = HashMap::new();

        for entry in self.adapters.iter() {
            let cli_type = *entry.key();
            let adapter = entry.value();

            match adapter.health_check().await {
                Ok(health) => {
                    health_summary.insert(cli_type, health);
                }
                Err(e) => {
                    let error_health = HealthStatus {
                        status: HealthState::Unhealthy,
                        message: Some(format!("Health check failed: {e}")),
                        checked_at: chrono::Utc::now(),
                        details: HashMap::new(),
                    };
                    health_summary.insert(cli_type, error_health);
                }
            }
        }

        health_summary
    }

    /// Get configuration registry (for configuration management)
    #[must_use]
    pub fn get_config_registry(&self) -> Arc<RwLock<ConfigRegistry>> {
        self.config_registry.clone()
    }

    /// Get detailed factory statistics
    pub async fn get_factory_stats(&self) -> FactoryStats {
        let health_summary = self.get_health_summary().await;
        let supported_clis = self.get_supported_clis();

        let healthy_count = health_summary
            .values()
            .filter(|h| matches!(h.status, HealthState::Healthy))
            .count();

        let warning_count = health_summary
            .values()
            .filter(|h| matches!(h.status, HealthState::Warning))
            .count();

        let unhealthy_count = health_summary
            .values()
            .filter(|h| matches!(h.status, HealthState::Unhealthy))
            .count();

        FactoryStats {
            total_adapters: supported_clis.len(),
            healthy_adapters: healthy_count,
            warning_adapters: warning_count,
            unhealthy_adapters: unhealthy_count,
            supported_cli_types: supported_clis,
            health_monitoring_enabled: self.config.enable_health_monitoring,
            last_health_check: chrono::Utc::now(),
        }
    }

    /// Validate an adapter before registration
    #[instrument(skip(self, adapter))]
    async fn validate_adapter(&self, adapter: &Arc<dyn CliAdapter>) -> AdapterResult<()> {
        debug!("Validating adapter before registration");

        // Check that adapter can provide basic information
        let executable_name = adapter.get_executable_name();
        if executable_name.is_empty() {
            return Err(AdapterError::ValidationError(
                "Adapter executable name cannot be empty".to_string(),
            ));
        }

        let memory_filename = adapter.get_memory_filename();
        if memory_filename.is_empty() {
            return Err(AdapterError::ValidationError(
                "Adapter memory filename cannot be empty".to_string(),
            ));
        }

        // Verify adapter capabilities are reasonable
        let capabilities = adapter.get_capabilities();
        if capabilities.max_context_tokens == 0 {
            return Err(AdapterError::ValidationError(
                "Adapter max context tokens must be greater than 0".to_string(),
            ));
        }

        // Test basic health check
        match adapter.health_check().await {
            Ok(_) => debug!("Adapter validation passed"),
            Err(e) => {
                warn!(error = %e, "Adapter health check failed during validation, but allowing registration");
            }
        }

        Ok(())
    }

    /// Start background health monitoring
    fn start_health_monitoring(&self) {
        let adapters = self.adapters.clone();
        let health_monitor = self.health_monitor.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                for entry in adapters.iter() {
                    let cli_type = *entry.key();
                    let adapter = entry.value().clone();
                    let monitor = health_monitor.clone();

                    // Spawn health check for each adapter
                    tokio::spawn(async move {
                        let start_time = Instant::now();
                        match adapter.health_check().await {
                            Ok(health) => {
                                let record = HealthCheckRecord {
                                    timestamp: chrono::Utc::now(),
                                    status: health.status,
                                    check_duration: start_time.elapsed(),
                                    error: None,
                                };
                                monitor.record_health_check(cli_type, &record);
                            }
                            Err(e) => {
                                let record = HealthCheckRecord {
                                    timestamp: chrono::Utc::now(),
                                    status: HealthState::Unhealthy,
                                    check_duration: start_time.elapsed(),
                                    error: Some(e.to_string()),
                                };
                                monitor.record_health_check(cli_type, &record);
                            }
                        }
                    });
                }
            }
        });

        debug!("Health monitoring background task started");
    }
}

/// Factory statistics
#[derive(Debug, Clone)]
pub struct FactoryStats {
    pub total_adapters: usize,
    pub healthy_adapters: usize,
    pub warning_adapters: usize,
    pub unhealthy_adapters: usize,
    pub supported_cli_types: Vec<CLIType>,
    pub health_monitoring_enabled: bool,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

impl HealthMonitor {
    /// Create a new health monitor
    #[must_use]
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            health_history: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Initialize monitoring for a CLI type
    pub fn initialize_monitoring(&self, cli_type: CLIType) {
        self.health_history.insert(cli_type, Vec::new());
        debug!(cli_type = %cli_type, "Initialized health monitoring");
    }

    /// Record a health check result
    pub fn record_health_check(&self, cli_type: CLIType, record: &HealthCheckRecord) {
        if let Some(mut history) = self.health_history.get_mut(&cli_type) {
            // Add new record
            history.push(record.clone());

            // Trim history to max size
            let current_len = history.len();
            if current_len > self.config.max_history_size {
                history.drain(0..current_len - self.config.max_history_size);
            }

            debug!(
                cli_type = %cli_type,
                status = ?record.status,
                duration_ms = record.check_duration.as_millis(),
                "Recorded health check"
            );
        }
    }

    /// Get health history for a CLI type
    #[must_use]
    pub fn get_health_history(&self, cli_type: CLIType) -> Vec<HealthCheckRecord> {
        self.health_history
            .get(&cli_type)
            .map(|history| history.clone())
            .unwrap_or_default()
    }

    /// Check if an adapter is consistently unhealthy
    #[must_use]
    pub fn is_consistently_unhealthy(&self, cli_type: CLIType) -> bool {
        if let Some(history) = self.health_history.get(&cli_type) {
            if history.len() < self.config.failure_threshold {
                return false;
            }

            // Check last N records for consecutive failures
            let recent_records = &history[history.len() - self.config.failure_threshold..];
            recent_records
                .iter()
                .all(|record| matches!(record.status, HealthState::Unhealthy))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::{
        AgentConfig, AuthMethod, CliCapabilities, ConfigFormat, ContainerContext, MemoryStrategy,
    };
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock adapter for testing
    #[derive(Debug)]
    struct MockAdapter {
        cli_type: CLIType,
        healthy: bool,
    }

    #[async_trait]
    impl CliAdapter for MockAdapter {
        async fn validate_model(&self, _model: &str) -> Result<bool> {
            Ok(true)
        }

        async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String> {
            Ok("mock config".to_string())
        }

        fn format_prompt(&self, prompt: &str) -> String {
            format!("Mock: {prompt}")
        }

        async fn parse_response(
            &self,
            response: &str,
        ) -> Result<crate::cli::adapter::ParsedResponse> {
            Ok(crate::cli::adapter::ParsedResponse {
                content: response.to_string(),
                tool_calls: vec![],
                metadata: crate::cli::adapter::ResponseMetadata::default(),
                finish_reason: crate::cli::adapter::FinishReason::Stop,
                streaming_delta: None,
            })
        }

        fn get_memory_filename(&self) -> &str {
            match self.cli_type {
                CLIType::Claude => "CLAUDE.md",
                CLIType::Codex => "AGENTS.md",
                CLIType::Factory => "FACTORY.md",
                _ => "MOCK.md",
            }
        }

        fn get_executable_name(&self) -> &str {
            match self.cli_type {
                CLIType::Claude => "claude",
                CLIType::Codex => "codex",
                CLIType::Factory => "droid",
                _ => "mock",
            }
        }

        fn get_capabilities(&self) -> CliCapabilities {
            CliCapabilities {
                supports_streaming: false,
                supports_multimodal: false,
                supports_function_calling: true,
                supports_system_prompts: true,
                max_context_tokens: 4096,
                memory_strategy: MemoryStrategy::MarkdownFile("MOCK.md".to_string()),
                config_format: ConfigFormat::Json,
                authentication_methods: vec![AuthMethod::None],
            }
        }

        async fn initialize(&self, _container: &ContainerContext) -> Result<()> {
            Ok(())
        }

        async fn cleanup(&self, _container: &ContainerContext) -> Result<()> {
            Ok(())
        }

        async fn health_check(&self) -> Result<HealthStatus> {
            Ok(HealthStatus {
                status: if self.healthy {
                    HealthState::Healthy
                } else {
                    HealthState::Unhealthy
                },
                message: None,
                checked_at: chrono::Utc::now(),
                details: HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_factory_creation() {
        let factory = AdapterFactory::new().await.unwrap();
        assert_eq!(factory.get_supported_clis().len(), 6);
        assert!(factory.supports_cli(CLIType::Claude));
        assert!(factory.supports_cli(CLIType::Codex));
        assert!(factory.supports_cli(CLIType::Cursor));
        assert!(factory.supports_cli(CLIType::Factory));
        assert!(factory.supports_cli(CLIType::Gemini));
        assert!(factory.supports_cli(CLIType::OpenCode));
    }

    #[tokio::test]
    async fn test_adapter_registration() {
        let factory = AdapterFactory::new().await.unwrap();
        let mock_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Claude,
            healthy: true,
        });

        factory
            .register_adapter(CLIType::Claude, mock_adapter)
            .await
            .unwrap();

        assert!(factory.supports_cli(CLIType::Claude));
        assert_eq!(factory.get_supported_clis().len(), 6);
    }

    #[tokio::test]
    async fn test_adapter_creation() {
        let factory = AdapterFactory::new().await.unwrap();
        let mock_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Codex,
            healthy: true,
        });

        factory
            .register_adapter(CLIType::Codex, mock_adapter)
            .await
            .unwrap();

        let created_adapter = factory.create(CLIType::Codex).await.unwrap();
        assert_eq!(created_adapter.get_executable_name(), "codex");
    }

    #[tokio::test]
    async fn test_unsupported_cli_error() {
        let factory = AdapterFactory::new().await.unwrap();

        // Use Grok as it's not yet implemented
        let result = factory.create(CLIType::Grok).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AdapterError::UnsupportedCliType(_)
        ));
    }

    #[tokio::test]
    async fn test_health_summary() {
        let factory = AdapterFactory::new().await.unwrap();

        let healthy_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Claude,
            healthy: true,
        });

        let unhealthy_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Codex,
            healthy: false,
        });

        factory
            .register_adapter(CLIType::Claude, healthy_adapter)
            .await
            .unwrap();
        factory
            .register_adapter(CLIType::Codex, unhealthy_adapter)
            .await
            .unwrap();

        let health_summary = factory.get_health_summary().await;

        assert_eq!(health_summary.len(), 6);
        assert_eq!(
            health_summary[&CLIType::Claude].status,
            HealthState::Healthy
        );
        assert_eq!(
            health_summary[&CLIType::Codex].status,
            HealthState::Unhealthy
        );
        assert!(health_summary.contains_key(&CLIType::Cursor));
        assert!(health_summary.contains_key(&CLIType::OpenCode));
    }

    #[tokio::test]
    async fn test_factory_stats() {
        let factory = AdapterFactory::new().await.unwrap();

        let healthy_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Claude,
            healthy: true,
        });

        factory
            .register_adapter(CLIType::Claude, healthy_adapter)
            .await
            .unwrap();

        let stats = factory.get_factory_stats().await;

        assert_eq!(stats.total_adapters, 6);
        assert_eq!(stats.healthy_adapters, 6);
        assert_eq!(stats.warning_adapters, 0);
        assert_eq!(stats.unhealthy_adapters, 0);
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new(HealthMonitorConfig::default());

        monitor.initialize_monitoring(CLIType::Claude);

        let record = HealthCheckRecord {
            timestamp: chrono::Utc::now(),
            status: HealthState::Healthy,
            check_duration: Duration::from_millis(100),
            error: None,
        };

        monitor.record_health_check(CLIType::Claude, &record);

        let history = monitor.get_health_history(CLIType::Claude);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, HealthState::Healthy);
    }
}
