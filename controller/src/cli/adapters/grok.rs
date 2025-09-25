//! Grok Adapter Implementation (Stub)

use crate::cli::base_adapter::{BaseAdapter, AdapterConfig};
use crate::cli::trait_adapter::*;
use crate::cli::types::CLIType;
use async_trait::async_trait;
use tracing::warn;

#[derive(Debug)]
pub struct GrokAdapter { base: BaseAdapter }

impl GrokAdapter {
    pub async fn new() -> Result<Self, AdapterError> {
        let base = BaseAdapter::new(CLIType::Grok);
        warn!("GrokAdapter is a stub implementation");
        Ok(Self { base })
    }
}

#[async_trait]
impl CliAdapter for GrokAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool, AdapterError> {
        Ok(model.starts_with("grok-") || model == "grok")
    }
    async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String, AdapterError> {
        Ok(r#"{"model": "grok-1"}"#.to_string())
    }
    fn format_prompt(&self, prompt: &str) -> String { prompt.to_string() }
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError> {
        Ok(ParsedResponse {
            content: response.to_string(), tool_calls: vec![],
            metadata: ResponseMetadata { id: None, usage: None, model: None, timing: None },
            finish_reason: FinishReason::Stop, streaming_delta: None,
        })
    }
    fn get_memory_filename(&self) -> &str { ".grok/GROK.md" }
    fn get_executable_name(&self) -> &str { "grok" }
    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true, supports_multimodal: false, supports_function_calling: true,
            supports_system_prompts: true, max_context_tokens: 100_000,
            memory_strategy: MemoryStrategy::Subdirectory(".grok".to_string()),
            config_format: ConfigFormat::Json, authentication_methods: vec![AuthMethod::ApiKey],
        }
    }
    async fn initialize(&self, _container: &dyn Container) -> Result<(), AdapterError> { Ok(()) }
    async fn cleanup(&self, _container: &dyn Container) -> Result<(), AdapterError> { Ok(()) }
    async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
        Ok(HealthStatus::Degraded("Stub implementation".to_string()))
    }
}