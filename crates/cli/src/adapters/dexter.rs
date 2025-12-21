//! Dexter CLI Adapter Implementation
//!
//! Implements the `CliAdapter` trait for the Dexter CLI, an autonomous financial
//! research agent. Dexter uses task planning, self-reflection, and real-time
//! market data via LangChain.
//!
//! <https://github.com/virattt/dexter>

use crate::adapter::{
    AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities, ConfigFormat,
    ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy, ParsedResponse,
    ResponseMetadata, ToolCall,
};
use crate::base_adapter::{AdapterConfig, BaseAdapter};
use crate::types::CLIType;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Dexter CLI adapter implementation
#[derive(Debug)]
pub struct DexterAdapter {
    base: Arc<BaseAdapter>,
}

impl DexterAdapter {
    /// Create a new Dexter adapter using default configuration.
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Dexter))
    }

    /// Create a new Dexter adapter with custom configuration.
    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Dexter adapter");

        let base = Arc::new(BaseAdapter::new(config)?);

        let adapter = Self { base };

        info!("Dexter adapter initialized successfully");
        Ok(adapter)
    }

    #[instrument(skip(self, response))]
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        let mut tool_calls = Vec::new();

        // Dexter outputs structured research results, parse if JSON
        if let Ok(json_val) = serde_json::from_str::<Value>(response) {
            // Look for tool invocations in the response
            if let Some(actions) = json_val.get("actions").and_then(Value::as_array) {
                for (idx, action) in actions.iter().enumerate() {
                    let name = action
                        .get("tool")
                        .and_then(Value::as_str)
                        .unwrap_or("research");
                    let args = action.get("input").cloned().unwrap_or(Value::Null);
                    let arguments = match args {
                        Value::Null => Value::Object(serde_json::Map::new()),
                        other => other,
                    };

                    tool_calls.push(ToolCall {
                        name: name.to_string(),
                        arguments,
                        id: Some(format!("dexter_action_{idx}")),
                    });
                }
            }
        }

        Ok(tool_calls)
    }

    #[instrument(skip(self, response))]
    async fn extract_response_metadata(&self, response: &str) -> ResponseMetadata {
        let mut metadata = ResponseMetadata::default();

        if let Ok(json_val) = serde_json::from_str::<Value>(response) {
            if let Some(model) = json_val.get("model").and_then(Value::as_str) {
                metadata.model = Some(model.to_string());
            }

            // Extract any token usage if present
            if let Some(usage) = json_val.get("usage") {
                metadata.input_tokens = usage
                    .get("input_tokens")
                    .and_then(Value::as_i64)
                    .and_then(|v| u32::try_from(v).ok());
                metadata.output_tokens = usage
                    .get("output_tokens")
                    .and_then(Value::as_i64)
                    .and_then(|v| u32::try_from(v).ok());
            }
        }

        metadata
    }
}

#[async_trait]
impl CliAdapter for DexterAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool> {
        // Dexter supports Claude, GPT, and Gemini models
        let valid_prefixes = ["claude", "gpt", "gemini", "o1", "o3", "o4"];
        Ok(valid_prefixes
            .iter()
            .any(|prefix| model.starts_with(prefix)))
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        debug!(
            github_app = %agent_config.github_app,
            model = %agent_config.model,
            "Generating Dexter configuration"
        );

        self.base.validate_base_config(agent_config)?;

        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));

        // Dexter uses environment variables for configuration
        let model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .map_or_else(|| agent_config.model.clone(), str::to_string);

        let max_steps = cli_config
            .get("maxSteps")
            .and_then(Value::as_u64)
            .unwrap_or(20);

        let max_steps_per_task = cli_config
            .get("maxStepsPerTask")
            .and_then(Value::as_u64)
            .unwrap_or(5);

        // Generate shell environment configuration
        let config = format!(
            r#"# Dexter Agent Configuration
# Generated by CTO controller

export DEXTER_MODEL="{model}"
export DEXTER_MAX_STEPS="{max_steps}"
export DEXTER_MAX_STEPS_PER_TASK="{max_steps_per_task}"

# API keys are expected to be set externally:
# - OPENAI_API_KEY (for GPT models)
# - ANTHROPIC_API_KEY (for Claude models)
# - GOOGLE_API_KEY (for Gemini models)
# - FINANCIAL_DATASETS_API_KEY (for financial data)
# - TAVILY_API_KEY (optional, for web search)
"#
        );

        info!(
            config_length = config.len(),
            "Dexter configuration generated successfully"
        );
        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Dexter accepts prompts directly
        prompt.to_string()
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Dexter response");

        let tool_calls = self.extract_tool_calls(response).await?;
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };
        let metadata = self.extract_response_metadata(response).await;

        Ok(ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None,
        })
    }

    fn get_memory_filename(&self) -> &'static str {
        // Dexter doesn't use a standard memory file, but we provide one for consistency
        "DEXTER.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "dexter-agent"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: false,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("DEXTER.md".to_string()),
            config_format: ConfigFormat::Custom("env".to_string()),
            authentication_methods: vec![AuthMethod::ApiKey],
        }
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing Dexter adapter for container"
        );

        self.base.base_initialize(container).await?;

        info!("Dexter adapter initialization completed");
        Ok(())
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up Dexter adapter"
        );

        self.base.base_cleanup(container).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Dexter adapter health check");

        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let mut health = self.base.base_health_check(&container).await?;

        // Check if we can generate a valid config
        let mock_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "dexter".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(8192),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        let config_result = self.generate_config(&mock_config).await;
        health.details.insert(
            "config_generation".to_string(),
            json!(config_result.is_ok()),
        );

        // Verify response parsing works
        let parse_result = self.parse_response("{}").await;
        health
            .details
            .insert("response_parsing".to_string(), json!(parse_result.is_ok()));

        // Check for required API key environment variables
        let has_api_key = std::env::var("ANTHROPIC_API_KEY").is_ok()
            || std::env::var("OPENAI_API_KEY").is_ok()
            || std::env::var("GOOGLE_API_KEY").is_ok();

        health
            .details
            .insert("has_api_key".to_string(), json!(has_api_key));

        if !has_api_key {
            health.status = HealthState::Warning;
            health.message = Some("No LLM API key found in environment".to_string());
        }

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_agent_config() -> AgentConfig {
        AgentConfig {
            github_app: "test-app".to_string(),
            cli: "dexter".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.5),
            tools: None,
            cli_config: Some(json!({
                "model": "claude-sonnet-4-20250514",
                "maxSteps": 30,
                "maxStepsPerTask": 10
            })),
        }
    }

    #[tokio::test]
    async fn test_generate_config() {
        let adapter = DexterAdapter::new().unwrap();
        let agent_config = sample_agent_config();

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("DEXTER_MODEL"));
        assert!(config.contains("claude-sonnet-4-20250514"));
        assert!(config.contains("DEXTER_MAX_STEPS=\"30\""));
        assert!(config.contains("DEXTER_MAX_STEPS_PER_TASK=\"10\""));
    }

    #[tokio::test]
    async fn test_validate_model() {
        let adapter = DexterAdapter::new().unwrap();

        assert!(adapter
            .validate_model("claude-sonnet-4-20250514")
            .await
            .unwrap());
        assert!(adapter.validate_model("gpt-4").await.unwrap());
        assert!(adapter.validate_model("gemini-2.5-pro").await.unwrap());
        assert!(!adapter.validate_model("llama-3").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_executable_name() {
        let adapter = DexterAdapter::new().unwrap();
        assert_eq!(adapter.get_executable_name(), "dexter-agent");
    }

    #[tokio::test]
    async fn test_get_memory_filename() {
        let adapter = DexterAdapter::new().unwrap();
        assert_eq!(adapter.get_memory_filename(), "DEXTER.md");
    }

    #[tokio::test]
    async fn test_parse_response() {
        let adapter = DexterAdapter::new().unwrap();

        let response = r#"{"result": "Analysis complete", "actions": []}"#;
        let parsed = adapter.parse_response(response).await.unwrap();

        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());
    }

    #[tokio::test]
    async fn test_parse_response_with_actions() {
        let adapter = DexterAdapter::new().unwrap();

        let response = r#"{"result": "Analysis", "actions": [{"tool": "get_stock_price", "input": {"symbol": "AAPL"}}]}"#;
        let parsed = adapter.parse_response(response).await.unwrap();

        assert_eq!(parsed.finish_reason, FinishReason::ToolCall);
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].name, "get_stock_price");
    }
}
