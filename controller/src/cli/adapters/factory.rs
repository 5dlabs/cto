//! Factory Droid CLI Adapter Implementation
//!
//! Provides a placeholder implementation so the controller can schedule
//! Factory CLI jobs without panicking. The heavier lifting (prompt
//! formatting, response parsing, configuration rendering) will be layered
//! in subsequent tasks.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, HealthStatus, MemoryStrategy, ParsedResponse,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Factory CLI adapter skeleton.
/// Mirrors the Cursor placeholder: we wire the variant into the
/// adapter factory while we build out end-to-end behaviour.
#[derive(Debug)]
pub struct FactoryAdapter {
    base: Arc<BaseAdapter>,
}

impl FactoryAdapter {
    /// Create a new Factory adapter using default configuration.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Factory)).await
    }

    /// Create a new Factory adapter with a custom base configuration.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Factory adapter (skeleton)");
        let base = Arc::new(BaseAdapter::new(config).await?);
        Ok(Self { base })
    }

    fn unsupported(feature: &str) -> AdapterError {
        match feature {
            "configuration generation" => AdapterError::ConfigGenerationError(format!(
                "Factory adapter placeholder: {feature} is not implemented yet"
            )),
            "response parsing" => AdapterError::ResponseParsingError(format!(
                "Factory adapter placeholder: {feature} is not implemented yet"
            )),
            _ => AdapterError::ValidationError(format!(
                "Factory adapter placeholder: {feature} is not implemented yet"
            )),
        }
    }
}

#[async_trait]
impl CliAdapter for FactoryAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool> {
        // Allow controllers to choose models freely; Factory CLI will enforce at runtime.
        Ok(true)
    }

    async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String> {
        Err(Self::unsupported("configuration generation").into())
    }

    fn format_prompt(&self, prompt: &str) -> String {
        prompt.to_string()
    }

    async fn parse_response(&self, _response: &str) -> Result<ParsedResponse> {
        Err(Self::unsupported("response parsing").into())
    }

    fn get_memory_filename(&self) -> &str {
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &str {
        "droid"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        }
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Factory adapter placeholder initialize"
        );
        self.base
            .base_initialize(container)
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        self.base
            .base_cleanup(container)
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        warn!("Factory adapter health check returning placeholder status");

        let mut health = self
            .base
            .base_health_check(&ContainerContext {
                pod: None,
                container_name: "factory-health".to_string(),
                working_dir: "/tmp".to_string(),
                env_vars: HashMap::new(),
                namespace: "default".to_string(),
            })
            .await?;

        health.details.insert(
            "implementation_status".to_string(),
            Value::String("factory adapter skeleton".to_string()),
        );
        Ok(health)
    }
}
