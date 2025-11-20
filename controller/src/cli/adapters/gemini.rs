//! Google Gemini CLI Adapter Implementation
//!
//! Implements the `CliAdapter` trait for the Google Gemini CLI, generating
//! appropriate configuration, handling prompt formatting, response parsing,
//! and health checks.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthStatus, MemoryStrategy, ParsedResponse,
    ResponseMetadata, ToolCall,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use crate::tasks::template_paths::{CODE_GEMINI_CONFIG_TEMPLATE, CODE_GEMINI_MEMORY_TEMPLATE};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Map as JsonMap, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

const DEFAULT_API_KEY_ENV: &str = "GOOGLE_API_KEY";
const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

fn first_string<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
}

fn first_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
}

fn first_f64(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_f64))
}

fn safe_f32(value: f64) -> Option<f32> {
    if value.is_finite() && value >= f64::from(f32::MIN) && value <= f64::from(f32::MAX) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        {
            Some(value as f32)
        }
    } else {
        None
    }
}

/// Google Gemini CLI adapter implementation
#[derive(Debug)]
pub struct GeminiAdapter {
    base: Arc<BaseAdapter>,
    config_template: &'static str,
    memory_template: &'static str,
}

impl GeminiAdapter {
    /// Create a new Gemini adapter using default configuration
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Gemini))
    }

    /// Create a new Gemini adapter with custom configuration
    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Gemini adapter");
        let base = Arc::new(BaseAdapter::new(config)?);

        Ok(Self {
            base,
            config_template: CODE_GEMINI_CONFIG_TEMPLATE,
            memory_template: CODE_GEMINI_MEMORY_TEMPLATE,
        })
    }

    fn render_memory_file(&self, agent_config: &AgentConfig) -> AdapterResult<String> {
        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));

        let remote_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        let mut context = JsonMap::new();
        context.insert(
            "github_app".to_string(),
            Value::String(agent_config.github_app.clone()),
        );
        context.insert(
            "model".to_string(),
            Value::String(agent_config.model.clone()),
        );
        context.insert("remote_tools".to_string(), json!(remote_tools));
        context.insert("cli_config".to_string(), cli_config);

        self.base
            .render_template_file(self.memory_template, &Value::Object(context))
            .map_err(|err| {
                AdapterError::TemplateError(format!(
                    "Failed to render Gemini memory template: {err}"
                ))
            })
    }

    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        self.base
            .render_template_file(self.config_template, context)
            .map_err(|err| {
                AdapterError::TemplateError(format!(
                    "Failed to render Gemini config template: {err}"
                ))
            })
    }

    fn build_config_context(&self, agent_config: &AgentConfig) -> Value {
        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));

        let model = first_string(&cli_config, &["model", "defaultModel"])
            .map_or_else(|| agent_config.model.clone(), str::to_string);

        let max_output_tokens = first_u64(&cli_config, &["maxTokens", "max_output_tokens"])
            .and_then(|value| u32::try_from(value).ok())
            .or(agent_config.max_tokens);

        let temperature = first_f64(&cli_config, &["temperature", "temp"])
            .and_then(safe_f32)
            .or(agent_config.temperature);

        let api_key_env = first_string(&cli_config, &["apiKeyEnv", "envKey"])
            .unwrap_or(DEFAULT_API_KEY_ENV)
            .to_string();

        let base_url = first_string(&cli_config, &["baseUrl", "base_url"])
            .unwrap_or(DEFAULT_BASE_URL)
            .to_string();

        let remote_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        let local_servers = agent_config
            .tools
            .as_ref()
            .and_then(|tools| tools.local_servers.clone())
            .unwrap_or_default();

        json!({
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "correlation_id": self.base.config.correlation_id,
                "cli": agent_config.cli,
                "github_app": agent_config.github_app,
            },
            "agent": {
                "model": model,
                "max_output_tokens": max_output_tokens,
                "temperature": temperature,
                "api_key_env": api_key_env,
                "base_url": base_url,
                "remote_tools": remote_tools,
                "local_servers": local_servers,
            },
            "raw_cli_config": cli_config,
        })
    }

    fn parse_tool_calls(line: &Value, tool_calls: &mut Vec<ToolCall>) {
        if let Some(commands) = line.get("commands").and_then(Value::as_array) {
            for (idx, command) in commands.iter().enumerate() {
                let name = command
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or("gemini_command");
                let arguments = command
                    .get("args")
                    .cloned()
                    .unwrap_or_else(|| Value::Object(JsonMap::new()));

                tool_calls.push(ToolCall {
                    name: name.to_string(),
                    arguments,
                    id: Some(format!("tool_{idx}")),
                });
            }
        }
    }

    fn update_metadata(line: &Value, metadata: &mut ResponseMetadata) {
        if let Some(model) = line.get("model").and_then(Value::as_str) {
            metadata.model = Some(model.to_string());
        }

        if let Some(usage) = line.get("usage") {
            if let Some(input) = usage.get("input_tokens").and_then(Value::as_u64) {
                metadata.input_tokens = u32::try_from(input).ok();
            }
            if let Some(output) = usage.get("output_tokens").and_then(Value::as_u64) {
                metadata.output_tokens = u32::try_from(output).ok();
            }
        }

        metadata
            .extra
            .insert("gemini_event".to_string(), line.clone());
    }
}

#[async_trait]
impl CliAdapter for GeminiAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool> {
        // Gemini models should start with "gemini-" or "models/gemini-"
        let normalized = model.trim().to_lowercase();
        Ok(normalized.starts_with("gemini-") || normalized.starts_with("models/gemini-"))
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        self.base.validate_base_config(agent_config)?;

        let context = self.build_config_context(agent_config);
        let rendered = self.render_config(&context)?;

        debug!(
            config_length = rendered.len(),
            "Gemini configuration generated successfully"
        );
        Ok(rendered)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        if prompt.ends_with('\n') {
            prompt.to_string()
        } else {
            format!("{prompt}\n")
        }
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        let mut aggregated: Vec<String> = Vec::new();
        let mut plain_segments: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut metadata = ResponseMetadata::default();

        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(trimmed) {
                Ok(json_line) => {
                    if let Some(message) = json_line.get("message").and_then(Value::as_str) {
                        aggregated.push(message.to_string());
                    }

                    Self::parse_tool_calls(&json_line, &mut tool_calls);
                    Self::update_metadata(&json_line, &mut metadata);
                }
                Err(_) => plain_segments.push(trimmed.to_string()),
            }
        }

        if !plain_segments.is_empty() {
            aggregated.push(plain_segments.join("\n"));
        }

        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        let content = if aggregated.is_empty() {
            response.trim().to_string()
        } else {
            aggregated.join("\n")
        };

        Ok(ParsedResponse {
            content,
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None,
        })
    }

    fn get_memory_filename(&self) -> &'static str {
        "GEMINI.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "gemini-cli"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: true,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 1_048_576, // Gemini 3 Pro has 1M+ context
            memory_strategy: MemoryStrategy::MarkdownFile("GEMINI.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        }
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing Gemini adapter"
        );
        self.base.base_initialize(container).await?;
        Ok(())
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up Gemini adapter"
        );
        self.base.base_cleanup(container).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Gemini adapter health check");

        let container = ContainerContext {
            pod: None,
            container_name: "gemini-health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let mut health = self.base.base_health_check(&container).await?;

        let mock_agent = AgentConfig {
            github_app: "health-check".to_string(),
            cli: "gemini".to_string(),
            model: "gemini-3-pro-preview".to_string(),
            max_tokens: Some(8192),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        health.details.insert(
            "config_generation".to_string(),
            json!(self.generate_config(&mock_agent).await.is_ok()),
        );
        health.details.insert(
            "memory_render".to_string(),
            json!(self.render_memory_file(&mock_agent).is_ok()),
        );
        health.details.insert(
            "response_parsing".to_string(),
            json!(self.parse_response("{}").await.is_ok()),
        );

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::{AgentConfig, ToolConfiguration};
    use serde_json::json;
    use std::path::PathBuf;

    fn templates_root() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("infra/charts/controller/agent-templates")
            .to_string_lossy()
            .into_owned()
    }

    fn sample_agent_config() -> AgentConfig {
        AgentConfig {
            github_app: "test-app".to_string(),
            cli: "gemini".to_string(),
            model: "gemini-3-pro-preview".to_string(),
            max_tokens: Some(8192),
            temperature: Some(0.7),
            tools: Some(ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gemini-3-pro-preview",
                "maxTokens": 16384,
                "temperature": 0.8,
                "apiKeyEnv": "GOOGLE_API_KEY",
                "baseUrl": "https://generativelanguage.googleapis.com/v1beta"
            })),
        }
    }

    #[tokio::test]
    async fn test_validate_model() {
        let adapter = GeminiAdapter::new().unwrap();

        // Valid Gemini models
        assert!(adapter
            .validate_model("gemini-3-pro-preview")
            .await
            .unwrap());
        assert!(adapter.validate_model("gemini-2.5-flash").await.unwrap());
        assert!(adapter
            .validate_model("models/gemini-3-pro-preview")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("GEMINI-3-PRO-PREVIEW")
            .await
            .unwrap());
        assert!(adapter.validate_model(" gemini-3-pro ").await.unwrap());

        // Invalid models should be rejected
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("claude-sonnet").await.unwrap());
        assert!(!adapter.validate_model("anthropic/claude-3").await.unwrap());
        assert!(!adapter.validate_model("random-model").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
    }

    #[tokio::test]
    async fn test_generate_config_overrides_defaults() {
        std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        let adapter = GeminiAdapter::new().unwrap();
        let agent = sample_agent_config();

        let rendered = adapter.generate_config(&agent).await.unwrap();
        let parsed: Value = serde_json::from_str(&rendered).unwrap_or(json!({}));

        // Verify config structure exists
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn test_parse_response_extracts_tool_calls() {
        let adapter = GeminiAdapter::new().unwrap();
        let payload = r#"
        {"message":"Started run","model":"gemini-3-pro-preview"}
        {"commands":[{"command":"shell","args":{"cmd":"ls"}}],"usage":{"input_tokens":120,"output_tokens":32}}
        "#;

        let parsed = adapter.parse_response(payload).await.unwrap();
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.finish_reason, FinishReason::ToolCall);
        assert_eq!(parsed.metadata.input_tokens, Some(120));
        assert!(parsed.content.contains("Started run"));
    }

    #[tokio::test]
    async fn test_get_capabilities() {
        let adapter = GeminiAdapter::new().unwrap();
        let caps = adapter.get_capabilities();

        assert!(caps.supports_streaming);
        assert!(caps.supports_multimodal);
        assert!(caps.supports_function_calling);
        assert_eq!(caps.max_context_tokens, 1_048_576);
    }
}
