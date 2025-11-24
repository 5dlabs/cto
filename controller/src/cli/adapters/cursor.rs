//! Cursor CLI Adapter Implementation
//!
//! Provides a placeholder implementation that wires Cursor into the
//! controller. Behaviour will be fleshed out in later tasks; for now we
//! surface clear TODOs so the variant can be selected without panicking.

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

/// Cursor CLI adapter skeleton.
///
/// The Cursor CLI shares much of the Codex flow but executes via the
/// `cursor-agent` binary and uses `.cursor` configuration assets. The
/// implementation here simply integrates the variant into the factory
/// while leaving detailed behaviour for follow-up tasks.
#[derive(Debug)]
pub struct CursorAdapter {
    base: Arc<BaseAdapter>,
}

impl CursorAdapter {
    /// Create a new Cursor adapter using default configuration.
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Cursor))
    }

    /// Create a new Cursor adapter with a custom base configuration.
    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Cursor adapter (skeleton)");
        let base = Arc::new(BaseAdapter::new(config)?);
        Ok(Self { base })
    }

    fn unsupported(feature: &str) -> AdapterError {
        match feature {
            "configuration generation" => AdapterError::ConfigGenerationError(format!(
                "Cursor adapter placeholder: {feature} is not implemented yet"
            )),
            "response parsing" => AdapterError::ResponseParsingError(format!(
                "Cursor adapter placeholder: {feature} is not implemented yet"
            )),
            _ => AdapterError::ValidationError(format!(
                "Cursor adapter placeholder: {feature} is not implemented yet"
            )),
        }
    }
}

#[async_trait]
impl CliAdapter for CursorAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool> {
        // Cursor will validate models at execution time; accept all for now.
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

    fn get_memory_filename(&self) -> &'static str {
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "cursor-agent"
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
            "Cursor adapter placeholder initialize"
        );
        self.base
            .base_initialize(container)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(())
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        self.base
            .base_cleanup(container)
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        // Reuse base health check but warn that functionality is pending.
        warn!("Cursor adapter health check returning placeholder status");
        let mut health = self
            .base
            .base_health_check(&ContainerContext {
                pod: None,
                container_name: "cursor-health".to_string(),
                working_dir: "/tmp".to_string(),
                env_vars: HashMap::new(),
                namespace: "default".to_string(),
            })
            .await?;

        health.details.insert(
            "implementation_status".to_string(),
            Value::String("cursor adapter skeleton".to_string()),
        );
        Ok(health)
    }
}
