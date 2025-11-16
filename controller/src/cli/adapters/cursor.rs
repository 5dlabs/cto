//! Cursor CLI Adapter Implementation
//!
//! Provides a fully featured implementation of the Cursor CLI adapter so the
//! controller can treat Cursor the same way it treats the other CLIs. The
//! adapter renders the Cursor configuration template, formats prompts,
//! understands stream-json output, and surfaces detailed health diagnostics.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::info;

const CURSOR_CONFIG_TEMPLATE: &str = "code/cursor/cursor-cli-config.json.hbs";
const CURSOR_MEMORY_TEMPLATE: &str = "code/cursor/agents.md.hbs";
const DEFAULT_TOOLMAN_URL: &str = "http://toolman.agent-platform.svc.cluster.local:3000/mcp";

#[derive(Debug)]
pub struct CursorAdapter {
    base: Arc<BaseAdapter>,
    config_template: &'static str,
    memory_template: &'static str,
}

impl CursorAdapter {
    /// Create a new Cursor adapter using default configuration.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Cursor)).await
    }

    /// Create a new Cursor adapter with a custom base configuration.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Cursor adapter");
        let base = Arc::new(BaseAdapter::new(config).await?);

        Ok(Self {
            base,
            config_template: CURSOR_CONFIG_TEMPLATE,
            memory_template: CURSOR_MEMORY_TEMPLATE,
        })
    }

    fn render_memory_file(&self, agent_config: &AgentConfig) -> AdapterResult<String> {
        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));

        let toolman_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        let context = json!({
            "cli_config": cli_config,
            "github_app": agent_config.github_app,
            "model": agent_config.model,
            "toolman": {
                "tools": toolman_tools,
            },
        });

        self.base
            .render_template_file(self.memory_template, &context)
            .map_err(|e| {
                AdapterError::TemplateError(format!("Failed to render Cursor memory template: {e}"))
            })
    }

    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        self.base
            .render_template_file(self.config_template, context)
            .map_err(|e| {
                AdapterError::TemplateError(format!("Failed to render Cursor config template: {e}"))
            })
    }

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

    fn resolve_toolman_url(cli_config: &Value) -> String {
        let settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let from_config = settings
            .get("toolmanUrl")
            .or_else(|| cli_config.get("toolmanUrl"))
            .and_then(Value::as_str)
            .map(|s| s.to_string());

        let url = from_config.unwrap_or_else(|| {
            env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| DEFAULT_TOOLMAN_URL.to_string())
        });

        url.trim_end_matches('/').to_string()
    }

    fn build_config_context(agent_config: &AgentConfig) -> Value {
        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));
        let settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let model = Self::first_string(&cli_config, &["model"])
            .unwrap_or(&agent_config.model)
            .to_string();

        let max_output_tokens =
            Self::first_u64(&cli_config, &["maxTokens", "modelMaxOutputTokens"])
                .or_else(|| Self::first_u64(&settings, &["maxTokens", "modelMaxOutputTokens"]))
                .map(|value| value as u32)
                .or(agent_config.max_tokens);

        let temperature = Self::first_f64(&cli_config, &["temperature"])
            .or_else(|| Self::first_f64(&settings, &["temperature"]))
            .or_else(|| agent_config.temperature.map(f64::from));

        let approval_policy = Self::first_string(&settings, &["approvalPolicy"])
            .unwrap_or("never")
            .to_string();

        let sandbox_mode =
            Self::first_string(&settings, &["sandboxPreset", "sandboxMode", "sandbox"])
                .unwrap_or("danger-full-access")
                .to_string();

        let project_doc_max_bytes =
            Self::first_u64(&settings, &["projectDocMaxBytes"]).unwrap_or(32_768);

        let reasoning_effort =
            Self::first_string(&cli_config, &["reasoningEffort", "modelReasoningEffort"])
                .or_else(|| {
                    Self::first_string(&settings, &["reasoningEffort", "modelReasoningEffort"])
                })
                .map(str::to_string);

        let editor_vim_mode = settings
            .get("editor")
            .and_then(|editor| editor.get("vimMode"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let toolman_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        json!({
            "model": model,
            "model_reasoning_effort": reasoning_effort,
            "temperature": temperature,
            "max_output_tokens": max_output_tokens,
            "approval_policy": approval_policy,
            "sandbox_mode": sandbox_mode,
            "project_doc_max_bytes": project_doc_max_bytes,
            "editor_vim_mode": editor_vim_mode,
            "toolman": {
                "url": Self::resolve_toolman_url(&cli_config),
                "tools": toolman_tools,
            },
        })
    }
}

#[async_trait]
impl CliAdapter for CursorAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool> {
        Ok(true)
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        self.base.validate_base_config(agent_config)?;
        let context = Self::build_config_context(agent_config);
        self.render_config(&context).map_err(|e| anyhow!(e))
    }

    fn format_prompt(&self, prompt: &str) -> String {
        if prompt.ends_with('\n') {
            prompt.to_string()
        } else {
            format!("{prompt}\n")
        }
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        let mut aggregated = Vec::new();
        let mut plain_segments = Vec::new();
        let mut tool_calls = Vec::new();
        let mut metadata = ResponseMetadata::default();
        let mut finish_reason = FinishReason::Stop;
        let mut last_event: Option<Value> = None;

        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(trimmed) {
                Ok(event) => {
                    if let Some(model) = event.get("model").and_then(Value::as_str) {
                        metadata.model = Some(model.to_string());
                    }

                    if let Some(duration) = event
                        .get("duration_ms")
                        .or_else(|| event.get("durationMs"))
                        .and_then(Value::as_u64)
                    {
                        metadata.duration_ms = Some(duration);
                    }

                    if let Some(usage) = event.get("usage") {
                        if let Some(tokens) = usage
                            .get("input_tokens")
                            .or_else(|| usage.get("inputTokens"))
                            .and_then(Value::as_u64)
                        {
                            metadata.input_tokens = Some(tokens as u32);
                        }
                        if let Some(tokens) = usage
                            .get("output_tokens")
                            .or_else(|| usage.get("outputTokens"))
                            .and_then(Value::as_u64)
                        {
                            metadata.output_tokens = Some(tokens as u32);
                        }
                    }

                    if let Some(message) = event
                        .get("message")
                        .and_then(Value::as_str)
                        .or_else(|| event.get("text").and_then(Value::as_str))
                    {
                        if !message.trim().is_empty() {
                            aggregated.push(message.to_string());
                        }
                    }

                    if let Some(event_type) = event.get("type").and_then(Value::as_str) {
                        match event_type {
                            "tool_call" | "toolCall" => {
                                let name = event
                                    .get("toolName")
                                    .or_else(|| event.get("name"))
                                    .or_else(|| event.get("command"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("cursor_tool")
                                    .to_string();
                                let arguments = event
                                    .get("parameters")
                                    .or_else(|| event.get("args"))
                                    .cloned()
                                    .unwrap_or(Value::Null);
                                let id = event
                                    .get("id")
                                    .or_else(|| event.get("callId"))
                                    .and_then(Value::as_str)
                                    .map(|s| s.to_string());
                                tool_calls.push(ToolCall {
                                    name,
                                    arguments,
                                    id,
                                });
                                finish_reason = FinishReason::ToolCall;
                            }
                            "error" => {
                                finish_reason = FinishReason::Error;
                            }
                            "result" | "completion" => {
                                let is_error = event
                                    .get("is_error")
                                    .or_else(|| event.get("isError"))
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false);

                                finish_reason = if is_error {
                                    FinishReason::Error
                                } else if finish_reason == FinishReason::ToolCall {
                                    FinishReason::ToolCall
                                } else {
                                    FinishReason::Stop
                                };
                            }
                            _ => {}
                        }
                    }

                    last_event = Some(event);
                }
                Err(_) => plain_segments.push(trimmed.to_string()),
            }
        }

        if !plain_segments.is_empty() {
            aggregated.push(plain_segments.join("\n"));
        }

        if finish_reason == FinishReason::Stop && !tool_calls.is_empty() {
            finish_reason = FinishReason::ToolCall;
        }

        if let Some(event) = last_event {
            metadata
                .extra
                .insert("cursor_last_event".to_string(), event);
        }

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

    fn get_memory_filename(&self) -> &str {
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &str {
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
            "Initializing Cursor adapter for container"
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

        let mock_agent = AgentConfig {
            github_app: "5DLabs-Rex".to_string(),
            cli: "cursor".to_string(),
            model: "gpt-5-cursor".to_string(),
            max_tokens: Some(32_000),
            temperature: Some(0.4),
            tools: None,
            cli_config: Some(json!({
                "model": "gpt-5-cursor",
                "maxTokens": 64000,
                "temperature": 0.5,
                "settings": {
                    "approvalPolicy": "never",
                    "sandboxMode": "workspace-write",
                    "projectDocMaxBytes": 65536,
                    "editor": {
                        "vimMode": true
                    }
                }
            })),
        };

        let config_result = self.generate_config(&mock_agent).await;
        health.details.insert(
            "config_generation".to_string(),
            json!(config_result.is_ok()),
        );

        let memory_result = self.render_memory_file(&mock_agent);
        health
            .details
            .insert("memory_render".to_string(), json!(memory_result.is_ok()));

        let sample_response = r#"{"type":"message","message":"Running task"}
{"type":"tool_call","toolName":"Shell","parameters":{"command":"ls"}}
{"type":"result","message":"Done","model":"gpt-5-cursor","usage":{"input_tokens":128,"output_tokens":256},"durationMs":900}"#;
        let parse_result = self.parse_response(sample_response).await;
        health
            .details
            .insert("response_parsing".to_string(), json!(parse_result.is_ok()));

        if config_result.is_err() || memory_result.is_err() || parse_result.is_err() {
            health.status = HealthState::Warning;
        }

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
            github_app: "5DLabs-Rex".to_string(),
            cli: "cursor".to_string(),
            model: "gpt-5-cursor".to_string(),
            max_tokens: Some(48_000),
            temperature: Some(0.65),
            tools: Some(ToolConfiguration {
                remote: vec![
                    "memory_create_entities".to_string(),
                    "rustdocs_query_rust_docs".to_string(),
                ],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gpt-5-cursor",
                "maxTokens": 80000,
                "temperature": 0.5,
                "reasoningEffort": "medium",
                "settings": {
                    "approvalPolicy": "never",
                    "sandboxMode": "workspace-write",
                    "projectDocMaxBytes": 65536,
                    "editor": {
                        "vimMode": true
                    },
                    "toolmanUrl": "http://toolman.test/mcp"
                }
            })),
        }
    }

    #[tokio::test]
    async fn test_generate_config_includes_overrides() {
        std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        let adapter = CursorAdapter::new().await.unwrap();
        let config = adapter
            .generate_config(&sample_agent_config())
            .await
            .unwrap();
        let parsed: Value = serde_json::from_str(&config).unwrap();
        assert_eq!(parsed["model"]["default"], "gpt-5-cursor");
        assert_eq!(parsed["automation"]["sandboxMode"], "workspace-write");
        assert_eq!(parsed["editor"]["vimMode"], true);
        assert_eq!(
            parsed["mcp"]["servers"]["toolman"]["env"]["TOOLMAN_SERVER_URL"],
            "http://toolman.test/mcp"
        );
    }

    #[tokio::test]
    async fn test_parse_response_handles_stream_json() {
        let adapter = CursorAdapter::new().await.unwrap();
        let response = r#"{"type":"message","message":"Working"}
{"type":"tool_call","toolName":"Shell","parameters":{"command":"ls"}}
{"type":"result","message":"Done","model":"gpt-5-cursor","usage":{"input_tokens":123,"output_tokens":456},"durationMs":789}"#;

        let parsed = adapter.parse_response(response).await.unwrap();
        assert!(parsed.content.contains("Working"));
        assert!(parsed.content.contains("Done"));
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.metadata.input_tokens, Some(123));
        assert_eq!(parsed.metadata.output_tokens, Some(456));
        assert_eq!(parsed.metadata.duration_ms, Some(789));
        assert_eq!(parsed.finish_reason, FinishReason::ToolCall);
    }
}
