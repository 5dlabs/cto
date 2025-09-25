//! Gemini Adapter Implementation (Stub)

use crate::cli::base_adapter::{BaseAdapter, AdapterConfig};
use crate::cli::trait_adapter::*;
use crate::cli::types::CLIType;
use async_trait::async_trait;
use tracing::warn;

#[derive(Debug)]
pub struct GeminiAdapter { base: BaseAdapter }

impl GeminiAdapter {
    pub async fn new() -> Result<Self, AdapterError> {
        let base = BaseAdapter::new(CLIType::Gemini);
        warn!("GeminiAdapter is a stub implementation");
        Ok(Self { base })
    }
}

#[async_trait]
impl CliAdapter for GeminiAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool, AdapterError> {
        Ok(model.starts_with("gemini-") || model == "gemini")
    }
    async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String, AdapterError> {
        Ok(r#"{"model": "gemini-pro"}"#.to_string())
    }
    fn format_prompt(&self, prompt: &str) -> String { prompt.to_string() }
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError> {
        Ok(ParsedResponse {
            content: response.to_string(), tool_calls: vec![],
            metadata: ResponseMetadata { id: None, usage: None, model: None, timing: None },
            finish_reason: FinishReason::Stop, streaming_delta: None,
        })
    }
    fn get_memory_filename(&self) -> &str { "GEMINI.md" }
    fn get_executable_name(&self) -> &str { "gemini" }
    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true, supports_multimodal: true, supports_function_calling: true,
            supports_system_prompts: true, max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("GEMINI.md".to_string()),
            config_format: ConfigFormat::Json, authentication_methods: vec![AuthMethod::OAuth],
        }
    }
    async fn initialize(&self, _container: &dyn Container) -> Result<(), AdapterError> { Ok(()) }
    async fn cleanup(&self, _container: &dyn Container) -> Result<(), AdapterError> { Ok(()) }
    async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
        Ok(HealthStatus::Degraded("Stub implementation".to_string()))
    }
}