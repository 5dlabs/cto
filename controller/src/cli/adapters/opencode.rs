//! Opencode Adapter Implementation (Stub)
//!
//! Stub implementation for the Opencode CLI.
//! This will be fully implemented in a future task.

use crate::cli::base_adapter::{BaseAdapter, AdapterConfig};
use crate::cli::trait_adapter::*;
use crate::cli::types::CLIType;
use async_trait::async_trait;
use tracing::{debug, warn};

/// Opencode CLI adapter (stub implementation)
#[derive(Debug)]
pub struct OpencodeAdapter {
    base: BaseAdapter,
}

impl OpencodeAdapter {
    pub async fn new() -> Result<Self, AdapterError> {
        let base = BaseAdapter::new(CLIType::OpenCode);
        warn!("OpencodeAdapter is a stub implementation");
        Ok(Self { base })
    }

    pub async fn with_config(config: AdapterConfig) -> Result<Self, AdapterError> {
        let base = BaseAdapter::with_config(CLIType::OpenCode, config);
        Ok(Self { base })
    }
}

#[async_trait]
impl CliAdapter for OpencodeAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool, AdapterError> {
        Ok(true) // Stub: accept all models
    }

    async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String, AdapterError> {
        Ok(r#"{"model": "default", "type": "stub"}"#.to_string())
    }

    fn format_prompt(&self, prompt: &str) -> String {
        prompt.to_string()
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError> {
        Ok(ParsedResponse {
            content: response.to_string(),
            tool_calls: vec![],
            metadata: ResponseMetadata { id: None, usage: None, model: None, timing: None },
            finish_reason: FinishReason::Stop,
            streaming_delta: None,
        })
    }

    fn get_memory_filename(&self) -> &str { "AGENTS.md" }
    fn get_executable_name(&self) -> &str { "opencode" }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 100_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::Custom("opencode-auth".to_string())],
        }
    }

    async fn initialize(&self, _container: &dyn Container) -> Result<(), AdapterError> { Ok(()) }
    async fn cleanup(&self, _container: &dyn Container) -> Result<(), AdapterError> { Ok(()) }
    async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
        Ok(HealthStatus::Degraded("Stub implementation".to_string()))
    }
}