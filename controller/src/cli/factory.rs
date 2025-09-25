//! Adapter Factory Pattern
//!
//! Manages CLI adapter lifecycle, registration, and provides a factory interface
//! for creating and accessing CLI adapters with health monitoring.

use crate::cli::base_adapter::{AdapterMetrics, BaseAdapter, HealthCheckResult};
use crate::cli::trait_adapter::*;
use crate::cli::types::CLIType;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{error, info, warn, debug, instrument};
use async_trait::async_trait;

/// Factory for creating and managing CLI adapters
#[derive(Debug)]
pub struct AdapterFactory {
    /// Registered adapters by CLI type
    adapters: RwLock<HashMap<CLIType, Arc<dyn CliAdapter>>>,
    /// Configuration registry for adapters
    config_registry: ConfigRegistry,
    /// Health monitor for adapters
    health_monitor: HealthMonitor,
    /// Factory configuration
    config: FactoryConfig,
}

/// Configuration for the adapter factory
#[derive(Debug, Clone)]
pub struct FactoryConfig {
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum concurrent health checks
    pub max_concurrent_health_checks: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Enable automatic adapter discovery
    pub enable_auto_discovery: bool,
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
}

impl Default for FactoryConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(60),
            max_concurrent_health_checks: 10,
            health_check_timeout: Duration::from_secs(30),
            enable_auto_discovery: false,
            enable_health_monitoring: true,
        }
    }
}

/// Configuration registry for managing adapter configurations
#[derive(Debug)]
pub struct ConfigRegistry {
    /// CLI configurations by type
    cli_configs: RwLock<HashMap<CLIType, CLIConfiguration>>,
}

/// CLI configuration metadata
#[derive(Debug, Clone)]
pub struct CLIConfiguration {
    /// CLI type
    pub cli_type: CLIType,
    /// Human-readable name
    pub name: String,
    /// Executable name
    pub executable: String,
    /// Default model
    pub default_model: String,
    /// Supported models
    pub supported_models: Vec<String>,
    /// Configuration schema
    pub config_schema: serde_json::Value,
    /// Default configuration
    pub default_config: serde_json::Value,
}

/// Health monitoring system for adapters
#[derive(Debug)]
pub struct HealthMonitor {
    /// Health check results by CLI type
    health_results: RwLock<HashMap<CLIType, HealthCheckResult>>,
    /// Health check history
    health_history: RwLock<HashMap<CLIType, Vec<HealthCheckResult>>>,
    /// Monitor configuration
    config: HealthMonitorConfig,
}

/// Health monitor configuration
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Maximum history entries per CLI
    pub max_history_entries: usize,
    /// Health check retry attempts
    pub max_retry_attempts: u32,
    /// Retry backoff duration
    pub retry_backoff: Duration,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            max_history_entries: 100,
            max_retry_attempts: 3,
            retry_backoff: Duration::from_secs(5),
        }
    }
}

impl ConfigRegistry {
    pub fn new() -> Self {
        Self {
            cli_configs: RwLock::new(HashMap::new()),
        }
    }

    /// Register CLI configuration
    pub async fn register_cli_config(&self, config: CLIConfiguration) {
        let mut cli_configs = self.cli_configs.write().await;
        cli_configs.insert(config.cli_type, config);
    }

    /// Get CLI configuration
    pub async fn get_cli_config(&self, cli_type: CLIType) -> Option<CLIConfiguration> {
        let cli_configs = self.cli_configs.read().await;
        cli_configs.get(&cli_type).cloned()
    }

    /// Get all registered CLI types
    pub async fn get_registered_clis(&self) -> Vec<CLIType> {
        let cli_configs = self.cli_configs.read().await;
        cli_configs.keys().cloned().collect()
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            health_results: RwLock::new(HashMap::new()),
            health_history: RwLock::new(HashMap::new()),
            config: HealthMonitorConfig::default(),
        }
    }

    pub fn with_config(config: HealthMonitorConfig) -> Self {
        Self {
            health_results: RwLock::new(HashMap::new()),
            health_history: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Record health check result
    #[instrument(skip(self))]
    pub async fn record_health_check(&self, result: HealthCheckResult) {
        let cli_type = result.cli_type;

        // Update latest result
        {
            let mut health_results = self.health_results.write().await;
            health_results.insert(cli_type, result.clone());
        }

        // Add to history
        {
            let mut health_history = self.health_history.write().await;
            let history = health_history.entry(cli_type).or_insert_with(Vec::new);
            history.push(result);

            // Trim history to max entries
            if history.len() > self.config.max_history_entries {
                history.drain(0..history.len() - self.config.max_history_entries);
            }
        }

        debug!(
            cli_type = ?cli_type,
            "Recorded health check result"
        );
    }

    /// Get latest health status for a CLI
    pub async fn get_health_status(&self, cli_type: CLIType) -> Option<HealthStatus> {
        let health_results = self.health_results.read().await;
        health_results.get(&cli_type).map(|result| result.status.clone())
    }

    /// Get health check history for a CLI
    pub async fn get_health_history(&self, cli_type: CLIType) -> Vec<HealthCheckResult> {
        let health_history = self.health_history.read().await;
        health_history.get(&cli_type).cloned().unwrap_or_default()
    }

    /// Check if CLI is healthy
    pub async fn is_healthy(&self, cli_type: CLIType) -> bool {
        match self.get_health_status(cli_type).await {
            Some(HealthStatus::Healthy) => true,
            Some(HealthStatus::Degraded(_)) => true, // Consider degraded as healthy for now
            Some(HealthStatus::Unhealthy(_)) | None => false,
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterFactory {
    /// Create a new adapter factory
    pub async fn new() -> Result<Self, AdapterError> {
        let config = FactoryConfig::default();
        Self::with_config(config).await
    }

    /// Create adapter factory with custom configuration
    pub async fn with_config(config: FactoryConfig) -> Result<Self, AdapterError> {
        let mut factory = Self {
            adapters: RwLock::new(HashMap::new()),
            config_registry: ConfigRegistry::new(),
            health_monitor: HealthMonitor::new(),
            config,
        };

        // Register built-in CLI configurations
        factory.register_builtin_configs().await;

        // Start health monitoring if enabled
        if factory.config.enable_health_monitoring {
            factory.start_health_monitoring().await;
        }

        Ok(factory)
    }

    /// Register built-in CLI configurations
    async fn register_builtin_configs(&mut self) {
        // Claude configuration
        let claude_config = CLIConfiguration {
            cli_type: CLIType::Claude,
            name: "Anthropic Claude Code".to_string(),
            executable: "claude-code".to_string(),
            default_model: "claude-3-5-sonnet".to_string(),
            supported_models: vec![
                "claude-3-5-sonnet".to_string(),
                "claude-3-opus".to_string(),
                "claude-3-haiku".to_string(),
                "opus".to_string(),
                "sonnet".to_string(),
                "haiku".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "model": {"type": "string"},
                    "max_tokens": {"type": "integer", "minimum": 1},
                    "temperature": {"type": "number", "minimum": 0.0, "maximum": 2.0}
                },
                "required": ["model"]
            }),
            default_config: serde_json::json!({
                "model": "claude-3-5-sonnet",
                "max_tokens": 4096,
                "temperature": 0.7
            }),
        };

        self.config_registry.register_cli_config(claude_config).await;

        // Codex configuration
        let codex_config = CLIConfiguration {
            cli_type: CLIType::Codex,
            name: "OpenAI Codex CLI".to_string(),
            executable: "codex".to_string(),
            default_model: "gpt-4".to_string(),
            supported_models: vec![
                "gpt-4".to_string(),
                "gpt-4o".to_string(),
                "o3".to_string(),
                "gpt-4-turbo".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "model": {"type": "string"},
                    "max_tokens": {"type": "integer", "minimum": 1},
                    "temperature": {"type": "number", "minimum": 0.0, "maximum": 2.0},
                    "api_key": {"type": "string"}
                },
                "required": ["model", "api_key"]
            }),
            default_config: serde_json::json!({
                "model": "gpt-4",
                "max_tokens": 8192,
                "temperature": 0.7
            }),
        };

        self.config_registry.register_cli_config(codex_config).await;

        // Add configurations for other CLIs as stubs
        for cli_type in [
            CLIType::OpenCode,
            CLIType::Gemini,
            CLIType::Grok,
            CLIType::Qwen,
            CLIType::Cursor,
            CLIType::OpenHands,
        ] {
            let stub_config = CLIConfiguration {
                cli_type,
                name: format!("{:?} CLI", cli_type),
                executable: cli_type.to_string().to_lowercase(),
                default_model: "default".to_string(),
                supported_models: vec!["default".to_string()],
                config_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "model": {"type": "string"}
                    },
                    "required": ["model"]
                }),
                default_config: serde_json::json!({
                    "model": "default"
                }),
            };

            self.config_registry.register_cli_config(stub_config).await;
        }
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) {
        // In a real implementation, this would start a background task
        // For now, we'll implement synchronous health checking
        info!("Health monitoring enabled with interval: {:?}", self.config.health_check_interval);
    }

    /// Register an adapter with the factory
    #[instrument(skip(self, adapter))]
    pub async fn register_adapter(
        &self,
        cli_type: CLIType,
        adapter: Arc<dyn CliAdapter>,
    ) -> Result<(), AdapterError> {
        // Validate the adapter before registration
        self.validate_adapter(&adapter).await?;

        // Register the adapter
        {
            let mut adapters = self.adapters.write().await;
            adapters.insert(cli_type, adapter.clone());
        }

        info!(cli_type = ?cli_type, "Registered CLI adapter");

        // Perform initial health check
        self.perform_health_check(cli_type, &adapter).await;

        Ok(())
    }

    /// Create an adapter instance for the given CLI type
    #[instrument(skip(self))]
    pub async fn create(&self, cli_type: CLIType) -> Result<Arc<dyn CliAdapter>, AdapterError> {
        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&cli_type)
            .ok_or_else(|| AdapterError::Generic(format!("Unsupported CLI type: {:?}", cli_type)))?
            .clone();

        // Check health before returning
        if self.config.enable_health_monitoring {
            let is_healthy = self.health_monitor.is_healthy(cli_type).await;
            if !is_healthy {
                return Err(AdapterError::HealthCheck(format!(
                    "CLI adapter {:?} is not healthy",
                    cli_type
                )));
            }
        }

        debug!(cli_type = ?cli_type, "Created CLI adapter instance");
        Ok(adapter)
    }

    /// Get all supported CLI types
    pub async fn get_supported_clis(&self) -> Vec<CLIType> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }

    /// Check if a CLI type is supported
    pub async fn supports_cli(&self, cli_type: CLIType) -> bool {
        let adapters = self.adapters.read().await;
        adapters.contains_key(&cli_type)
    }

    /// Get CLI configuration
    pub async fn get_cli_config(&self, cli_type: CLIType) -> Option<CLIConfiguration> {
        self.config_registry.get_cli_config(cli_type).await
    }

    /// Perform health check for all registered adapters
    #[instrument(skip(self))]
    pub async fn health_check_all(&self) -> HashMap<CLIType, HealthStatus> {
        let adapters = self.adapters.read().await;
        let mut results = HashMap::new();

        for (cli_type, adapter) in adapters.iter() {
            match tokio::time::timeout(self.config.health_check_timeout, adapter.health_check()).await {
                Ok(Ok(status)) => {
                    results.insert(*cli_type, status.clone());
                    self.record_health_check_result(*cli_type, status, Duration::from_secs(0)).await;
                }
                Ok(Err(error)) => {
                    let status = HealthStatus::Unhealthy(error.to_string());
                    results.insert(*cli_type, status.clone());
                    self.record_health_check_result(*cli_type, status, Duration::from_secs(0)).await;
                }
                Err(_) => {
                    let status = HealthStatus::Unhealthy("Health check timeout".to_string());
                    results.insert(*cli_type, status.clone());
                    self.record_health_check_result(*cli_type, status, Duration::from_secs(0)).await;
                }
            }
        }

        results
    }

    /// Validate an adapter before registration
    async fn validate_adapter(&self, adapter: &Arc<dyn CliAdapter>) -> Result<(), AdapterError> {
        // Check that the adapter can perform basic operations
        let capabilities = adapter.get_capabilities();
        if capabilities.max_context_tokens == 0 {
            return Err(AdapterError::Generic(
                "Adapter must specify max context tokens".to_string(),
            ));
        }

        if adapter.get_executable_name().is_empty() {
            return Err(AdapterError::Generic(
                "Adapter must specify executable name".to_string(),
            ));
        }

        if adapter.get_memory_filename().is_empty() {
            return Err(AdapterError::Generic(
                "Adapter must specify memory filename".to_string(),
            ));
        }

        Ok(())
    }

    /// Perform health check for a specific adapter
    async fn perform_health_check(&self, cli_type: CLIType, adapter: &Arc<dyn CliAdapter>) {
        let start_time = std::time::Instant::now();

        match tokio::time::timeout(self.config.health_check_timeout, adapter.health_check()).await {
            Ok(Ok(status)) => {
                let duration = start_time.elapsed();
                self.record_health_check_result(cli_type, status, duration).await;
            }
            Ok(Err(error)) => {
                let duration = start_time.elapsed();
                let status = HealthStatus::Unhealthy(error.to_string());
                self.record_health_check_result(cli_type, status, duration).await;
            }
            Err(_) => {
                let duration = start_time.elapsed();
                let status = HealthStatus::Unhealthy("Health check timeout".to_string());
                self.record_health_check_result(cli_type, status, duration).await;
            }
        }
    }

    /// Record health check result
    async fn record_health_check_result(&self, cli_type: CLIType, status: HealthStatus, duration: Duration) {
        let result = HealthCheckResult {
            timestamp: SystemTime::now(),
            cli_type,
            status,
            duration,
            context: HashMap::new(),
        };

        self.health_monitor.record_health_check(result).await;
    }

    /// Get health status for a specific CLI
    pub async fn get_health_status(&self, cli_type: CLIType) -> Option<HealthStatus> {
        self.health_monitor.get_health_status(cli_type).await
    }

    /// Get comprehensive factory statistics
    pub async fn get_factory_stats(&self) -> FactoryStats {
        let adapters = self.adapters.read().await;
        let supported_clis = adapters.keys().cloned().collect();
        let total_adapters = adapters.len();

        let mut healthy_adapters = 0;
        let mut unhealthy_adapters = 0;
        let mut degraded_adapters = 0;

        for &cli_type in &supported_clis {
            match self.health_monitor.get_health_status(cli_type).await {
                Some(HealthStatus::Healthy) => healthy_adapters += 1,
                Some(HealthStatus::Degraded(_)) => degraded_adapters += 1,
                Some(HealthStatus::Unhealthy(_)) | None => unhealthy_adapters += 1,
            }
        }

        FactoryStats {
            total_adapters,
            healthy_adapters,
            degraded_adapters,
            unhealthy_adapters,
            supported_clis,
        }
    }
}

/// Factory statistics
#[derive(Debug, Clone)]
pub struct FactoryStats {
    pub total_adapters: usize,
    pub healthy_adapters: usize,
    pub degraded_adapters: usize,
    pub unhealthy_adapters: usize,
    pub supported_clis: Vec<CLIType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock adapter for testing
    #[derive(Debug)]
    struct MockAdapter {
        cli_type: CLIType,
        health_status: HealthStatus,
    }

    impl MockAdapter {
        fn new(cli_type: CLIType) -> Self {
            Self {
                cli_type,
                health_status: HealthStatus::Healthy,
            }
        }

        fn with_health_status(mut self, status: HealthStatus) -> Self {
            self.health_status = status;
            self
        }
    }

    #[async_trait]
    impl CliAdapter for MockAdapter {
        async fn validate_model(&self, _model: &str) -> Result<bool, AdapterError> {
            Ok(true)
        }

        async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String, AdapterError> {
            Ok("{}".to_string())
        }

        fn format_prompt(&self, prompt: &str) -> String {
            prompt.to_string()
        }

        async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError> {
            Ok(ParsedResponse {
                content: response.to_string(),
                tool_calls: vec![],
                metadata: ResponseMetadata {
                    id: None,
                    usage: None,
                    model: None,
                    timing: None,
                },
                finish_reason: FinishReason::Stop,
                streaming_delta: None,
            })
        }

        fn get_memory_filename(&self) -> &str {
            match self.cli_type {
                CLIType::Claude => "CLAUDE.md",
                CLIType::Codex => "AGENTS.md",
                _ => "memory.md",
            }
        }

        fn get_executable_name(&self) -> &str {
            match self.cli_type {
                CLIType::Claude => "claude-code",
                CLIType::Codex => "codex",
                _ => "cli",
            }
        }

        fn get_capabilities(&self) -> CliCapabilities {
            CliCapabilities {
                supports_streaming: true,
                supports_multimodal: false,
                supports_function_calling: true,
                supports_system_prompts: true,
                max_context_tokens: 100_000,
                memory_strategy: MemoryStrategy::MarkdownFile("test.md".to_string()),
                config_format: ConfigFormat::Json,
                authentication_methods: vec![AuthMethod::SessionToken],
            }
        }

        async fn initialize(&self, _container: &Container) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn cleanup(&self, _container: &Container) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
            Ok(self.health_status.clone())
        }
    }

    #[tokio::test]
    async fn test_factory_creation() {
        let factory = AdapterFactory::new().await.unwrap();
        let supported_clis = factory.get_supported_clis().await;

        // Should have built-in configurations registered
        let cli_configs = factory.config_registry.get_registered_clis().await;
        assert!(cli_configs.contains(&CLIType::Claude));
        assert!(cli_configs.contains(&CLIType::Codex));
    }

    #[tokio::test]
    async fn test_adapter_registration() {
        let factory = AdapterFactory::new().await.unwrap();
        let mock_adapter = Arc::new(MockAdapter::new(CLIType::Claude));

        factory.register_adapter(CLIType::Claude, mock_adapter).await.unwrap();

        assert!(factory.supports_cli(CLIType::Claude).await);
        let adapter = factory.create(CLIType::Claude).await.unwrap();
        assert_eq!(adapter.get_executable_name(), "claude-code");
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        let factory = AdapterFactory::new().await.unwrap();
        let healthy_adapter = Arc::new(MockAdapter::new(CLIType::Claude));
        let unhealthy_adapter = Arc::new(
            MockAdapter::new(CLIType::Codex)
                .with_health_status(HealthStatus::Unhealthy("Test error".to_string()))
        );

        factory.register_adapter(CLIType::Claude, healthy_adapter).await.unwrap();
        factory.register_adapter(CLIType::Codex, unhealthy_adapter).await.unwrap();

        let health_results = factory.health_check_all().await;
        assert_eq!(health_results.get(&CLIType::Claude), Some(&HealthStatus::Healthy));
        assert!(matches!(
            health_results.get(&CLIType::Codex),
            Some(HealthStatus::Unhealthy(_))
        ));
    }

    #[tokio::test]
    async fn test_factory_stats() {
        let factory = AdapterFactory::new().await.unwrap();
        let adapter1 = Arc::new(MockAdapter::new(CLIType::Claude));
        let adapter2 = Arc::new(MockAdapter::new(CLIType::Codex));

        factory.register_adapter(CLIType::Claude, adapter1).await.unwrap();
        factory.register_adapter(CLIType::Codex, adapter2).await.unwrap();

        let stats = factory.get_factory_stats().await;
        assert_eq!(stats.total_adapters, 2);
        assert!(stats.supported_clis.contains(&CLIType::Claude));
        assert!(stats.supported_clis.contains(&CLIType::Codex));
    }

    #[tokio::test]
    async fn test_cli_configuration() {
        let factory = AdapterFactory::new().await.unwrap();

        let claude_config = factory.get_cli_config(CLIType::Claude).await.unwrap();
        assert_eq!(claude_config.cli_type, CLIType::Claude);
        assert_eq!(claude_config.executable, "claude-code");
        assert_eq!(claude_config.default_model, "claude-3-5-sonnet");

        let codex_config = factory.get_cli_config(CLIType::Codex).await.unwrap();
        assert_eq!(codex_config.cli_type, CLIType::Codex);
        assert_eq!(codex_config.executable, "codex");
        assert_eq!(codex_config.default_model, "gpt-4");
    }

    #[tokio::test]
    async fn test_adapter_validation() {
        let factory = AdapterFactory::new().await.unwrap();

        // Create invalid adapter with empty executable name
        #[derive(Debug)]
        struct InvalidAdapter;

        #[async_trait]
        impl CliAdapter for InvalidAdapter {
            async fn validate_model(&self, _model: &str) -> Result<bool, AdapterError> {
                Ok(true)
            }

            async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String, AdapterError> {
                Ok("{}".to_string())
            }

            fn format_prompt(&self, prompt: &str) -> String {
                prompt.to_string()
            }

            async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError> {
                Ok(ParsedResponse {
                    content: response.to_string(),
                    tool_calls: vec![],
                    metadata: ResponseMetadata {
                        id: None,
                        usage: None,
                        model: None,
                        timing: None,
                    },
                    finish_reason: FinishReason::Stop,
                    streaming_delta: None,
                })
            }

            fn get_memory_filename(&self) -> &str {
                ""  // Invalid: empty filename
            }

            fn get_executable_name(&self) -> &str {
                ""  // Invalid: empty executable
            }

            fn get_capabilities(&self) -> CliCapabilities {
                CliCapabilities {
                    supports_streaming: true,
                    supports_multimodal: false,
                    supports_function_calling: true,
                    supports_system_prompts: true,
                    max_context_tokens: 0,  // Invalid: zero tokens
                    memory_strategy: MemoryStrategy::MarkdownFile("test.md".to_string()),
                    config_format: ConfigFormat::Json,
                    authentication_methods: vec![AuthMethod::SessionToken],
                }
            }

            async fn initialize(&self, _container: &Container) -> Result<(), AdapterError> {
                Ok(())
            }

            async fn cleanup(&self, _container: &Container) -> Result<(), AdapterError> {
                Ok(())
            }

            async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
                Ok(HealthStatus::Healthy)
            }
        }

        let invalid_adapter = Arc::new(InvalidAdapter);
        let result = factory.register_adapter(CLIType::Claude, invalid_adapter).await;
        assert!(result.is_err());
    }
}