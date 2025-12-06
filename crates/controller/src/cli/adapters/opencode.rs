//! `OpenCode` CLI Adapter Implementation
//!
//! Provides a concrete implementation of the `CliAdapter` trait for the `OpenCode` CLI.
//! The adapter mirrors the structure used by other adapters (Codex/Factory) so the
//! controller can generate CLI-specific configuration, render memory files, and
//! translate responses into the shared `ParsedResponse` format.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthStatus, MemoryStrategy, ParsedResponse,
    ResponseMetadata, ToolCall,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use crate::tasks::template_paths::CODE_OPENCODE_MEMORY_TEMPLATE;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Map as JsonMap, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

const DEFAULT_PROVIDER_NAME: &str = "openai";
const DEFAULT_PROVIDER_ENV_KEY: &str = "OPENAI_API_KEY";

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

#[derive(Debug)]
pub struct OpenCodeAdapter {
    base: Arc<BaseAdapter>,
    memory_template: &'static str,
}

impl OpenCodeAdapter {
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::OpenCode))
    }

    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing OpenCode adapter");
        let base = Arc::new(BaseAdapter::new(config)?);

        Ok(Self {
            base,
            memory_template: CODE_OPENCODE_MEMORY_TEMPLATE,
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
                    "Failed to render OpenCode memory template: {err}"
                ))
            })
    }

    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        // Serialize configuration directly (no template needed)
        serde_json::to_string_pretty(context).map_err(|err| {
            AdapterError::ConfigGenerationError(format!(
                "Failed to serialize OpenCode config: {err}"
            ))
        })
    }

    fn build_provider_context(cli_config: &Value) -> Value {
        let provider = cli_config
            .get("provider")
            .cloned()
            .unwrap_or_else(|| json!({"name": DEFAULT_PROVIDER_NAME}));

        let mut provider_map = provider.as_object().cloned().unwrap_or_else(JsonMap::new);

        provider_map
            .entry("name".to_string())
            .or_insert_with(|| Value::String(DEFAULT_PROVIDER_NAME.to_string()));
        provider_map
            .entry("envKey".to_string())
            .or_insert_with(|| Value::String(DEFAULT_PROVIDER_ENV_KEY.to_string()));

        Value::Object(provider_map)
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

        let instructions = cli_config
            .get("instructions")
            .and_then(Value::as_str)
            .map(str::to_string);

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

        let provider = Self::build_provider_context(&cli_config);

        // Get Tools MCP server URL from environment (same pattern as other adapters)
        let tools_url = std::env::var("TOOLS_SERVER_URL")
            .unwrap_or_else(|_| "http://tools.cto.svc.cluster.local:3000/mcp".to_string());
        let tools_url = tools_url.trim_end_matches('/').to_string();

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
                "instructions": instructions,
                "remote_tools": remote_tools,
                "local_servers": local_servers,
                "provider": provider,
                "tools_url": tools_url,
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
                    .unwrap_or("opencode_command");
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
            .insert("opencode_event".to_string(), line.clone());
    }
}

#[async_trait]
impl CliAdapter for OpenCodeAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool> {
        Ok(!model.trim().is_empty())
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        self.base.validate_base_config(agent_config)?;

        let context = self.build_config_context(agent_config);
        let rendered = self.render_config(&context)?;

        debug!(
            config_length = rendered.len(),
            "OpenCode configuration generated successfully"
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
        "OPENCODE.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "opencode"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: true,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("OPENCODE.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        }
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing OpenCode adapter"
        );
        self.base.base_initialize(container).await?;
        Ok(())
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up OpenCode adapter"
        );
        self.base.base_cleanup(container).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing OpenCode adapter health check");

        let container = ContainerContext {
            pod: None,
            container_name: "opencode-health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let mut health = self.base.base_health_check(&container).await?;

        let mock_agent = AgentConfig {
            github_app: "health-check".to_string(),
            cli: "opencode".to_string(),
            model: "gpt-4.1".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.3),
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
    use crate::cli::test_utils::templates_root;
    use serde_json::json;
    use serial_test::serial;

    fn sample_agent_config() -> AgentConfig {
        AgentConfig {
            github_app: "test-app".to_string(),
            cli: "opencode".to_string(),
            model: "fallback-model".to_string(),
            max_tokens: Some(2048),
            temperature: Some(0.4),
            tools: Some(ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "opencode-sonnet",
                "maxTokens": 16384,
                "temperature": 0.65,
                "instructions": "Follow OpenCode best practices",
                "provider": {
                    "name": "anthropic",
                    "envKey": "ANTHROPIC_API_KEY"
                }
            })),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_config_serializes_directly() {
        // SAFETY: This test runs serially via #[serial] to avoid env var races
        unsafe {
            std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        }
        let adapter = OpenCodeAdapter::new().unwrap();
        let agent = sample_agent_config();

        let rendered = adapter.generate_config(&agent).await.unwrap();
        let parsed: Value = serde_json::from_str(&rendered).unwrap();

        // Config is now serialized directly as JSON (no template rendering)
        // Verify the context structure from build_config_context

        // Metadata section
        assert!(parsed.get("metadata").is_some());
        assert_eq!(parsed["metadata"]["cli"].as_str().unwrap(), "opencode");
        assert_eq!(
            parsed["metadata"]["github_app"].as_str().unwrap(),
            "test-app"
        );

        // Agent section with model and tools
        assert!(parsed.get("agent").is_some());
        assert_eq!(
            parsed["agent"]["model"].as_str().unwrap(),
            "opencode-sonnet"
        );

        // Remote tools
        let remote_tools = parsed["agent"]["remote_tools"]
            .as_array()
            .expect("remote_tools should be an array");
        assert!(remote_tools.contains(&json!("memory_create_entities")));
    }

    #[tokio::test]
    async fn test_parse_response_extracts_tool_calls() {
        let adapter = OpenCodeAdapter::new().unwrap();
        let payload = r#"
        {"message":"Started run","model":"opencode-sonnet"}
        {"commands":[{"command":"shell","args":{"cmd":"ls"}}],"usage":{"input_tokens":120,"output_tokens":32}}
        "#;

        let parsed = adapter.parse_response(payload).await.unwrap();
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.finish_reason, FinishReason::ToolCall);
        assert_eq!(parsed.metadata.input_tokens, Some(120));
        assert!(parsed.content.contains("Started run"));
    }
}
