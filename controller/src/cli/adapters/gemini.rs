//! Gemini CLI Adapter Implementation
//!
//! Provides a concrete implementation of the `CliAdapter` trait for the Gemini CLI. The adapter
//! renders Gemini-specific configuration, emits `GEMINI.md` instructions, and parses streaming
//! JSON output produced by `gemini --output-format stream-json`.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, CliAdapter, CliCapabilities, ContainerContext,
    FinishReason, HealthStatus, ParsedResponse, ResponseMetadata, ToolCall,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::capabilities::cli_capabilities;
use crate::cli::types::CLIType;
use crate::tasks::template_paths::{
    CODE_GEMINI_MEMORY_TEMPLATE, CODE_GEMINI_USER_SETTINGS_TEMPLATE,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

const DEFAULT_TOOLMAN_URL: &str = "http://toolman.agent-platform.svc.cluster.local:3000/mcp";

#[derive(Debug)]
pub struct GeminiAdapter {
    base: Arc<BaseAdapter>,
    config_template: &'static str,
    memory_template: &'static str,
}

impl GeminiAdapter {
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Gemini)).await
    }

    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Gemini adapter");
        let base = Arc::new(BaseAdapter::new(config).await?);

        Ok(Self {
            base,
            config_template: CODE_GEMINI_USER_SETTINGS_TEMPLATE,
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

        let context = json!({
            "cli_config": cli_config,
            "github_app": agent_config.github_app,
            "model": agent_config.model,
            "toolman": {
                "tools": remote_tools,
            },
        });

        self.base
            .render_template_file(self.memory_template, &context)
            .map_err(|e| {
                AdapterError::TemplateError(format!("Failed to render Gemini memory template: {e}"))
            })
    }

    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        self.base
            .render_template_file(self.config_template, context)
            .map_err(|e| {
                AdapterError::TemplateError(format!("Failed to render Gemini config template: {e}"))
            })
    }

    fn resolve_toolman_url(cli_config: &Value) -> String {
        cli_config
            .get("settings")
            .and_then(|settings| settings.get("toolmanUrl"))
            .or_else(|| cli_config.get("toolmanUrl"))
            .and_then(Value::as_str)
            .map(|s| s.trim_end_matches('/').to_string())
            .or_else(|| {
                std::env::var("TOOLMAN_SERVER_URL")
                    .ok()
                    .map(|s| s.trim_end_matches('/').to_string())
            })
            .unwrap_or_else(|| DEFAULT_TOOLMAN_URL.to_string())
    }

    fn build_config_context(agent_config: &AgentConfig) -> Value {
        let cli_config = agent_config
            .cli_config
            .clone()
            .unwrap_or_else(|| json!({ "settings": {} }));
        let settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&agent_config.model)
            .to_string();

        let max_output_tokens = cli_config
            .get("maxTokens")
            .or_else(|| settings.get("maxTokens"))
            .and_then(Value::as_u64)
            .map(|value| value as u32)
            .or(agent_config.max_tokens);

        let temperature = cli_config
            .get("temperature")
            .or_else(|| settings.get("temperature"))
            .and_then(Value::as_f64)
            .or_else(|| agent_config.temperature.map(f64::from));

        let approval_policy = settings
            .get("approvalPolicy")
            .and_then(Value::as_str)
            .unwrap_or("auto_edit")
            .to_string();

        let sandbox_mode = settings
            .get("sandbox")
            .or_else(|| settings.get("sandboxMode"))
            .and_then(Value::as_str)
            .unwrap_or("workspace-write")
            .to_string();

        let editor_vim_mode = settings
            .get("general")
            .and_then(|general| general.get("vimMode"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let remote_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        json!({
            "render": {
                "model": model,
                "temperature": temperature,
                "max_output_tokens": max_output_tokens,
                "sandbox_mode": sandbox_mode,
                "approval_policy": approval_policy,
                "editor_vim_mode": editor_vim_mode,
            },
            "toolman": {
                "url": Self::resolve_toolman_url(&cli_config),
                "tools": remote_tools,
            },
        })
    }
}

#[async_trait]
impl CliAdapter for GeminiAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool> {
        // Gemini CLI validates models during execution; accept all names here.
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
        let mut tool_calls = Vec::new();
        let mut metadata = ResponseMetadata::default();
        let mut finish_reason = FinishReason::Stop;

        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(trimmed) {
                Ok(event) => {
                    if let Some(event_type) = event.get("type").and_then(Value::as_str) {
                        match event_type {
                            "init" => {
                                if let Some(model) = event.get("model").and_then(Value::as_str) {
                                    metadata.model = Some(model.to_string());
                                }
                                if let Some(session_id) =
                                    event.get("session_id").and_then(Value::as_str)
                                {
                                    metadata
                                        .extra
                                        .insert("session_id".to_string(), json!(session_id));
                                }
                            }
                            "message" => {
                                let role = event.get("role").and_then(Value::as_str).unwrap_or("");
                                if role.eq_ignore_ascii_case("assistant") {
                                    if let Some(content) =
                                        event.get("content").and_then(Value::as_str)
                                    {
                                        aggregated.push(content.to_string());
                                    }
                                }
                            }
                            "tool_use" => {
                                let name = event
                                    .get("tool_name")
                                    .or_else(|| event.get("toolName"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("gemini_tool")
                                    .to_string();
                                let arguments = event
                                    .get("parameters")
                                    .or_else(|| event.get("input"))
                                    .cloned()
                                    .unwrap_or(Value::Null);
                                let id = event
                                    .get("tool_id")
                                    .or_else(|| event.get("toolId"))
                                    .and_then(Value::as_str)
                                    .map(|s| s.to_string());
                                tool_calls.push(ToolCall {
                                    name,
                                    arguments,
                                    id,
                                });
                                finish_reason = FinishReason::ToolCall;
                            }
                            "tool_result" => {
                                metadata
                                    .extra
                                    .insert("last_tool_result".into(), event.clone());
                            }
                            "result" => {
                                if let Some(stats) = event.get("stats") {
                                    if let Some(input) =
                                        stats.get("input_tokens").and_then(Value::as_u64)
                                    {
                                        metadata.input_tokens = Some(input as u32);
                                    }
                                    if let Some(output) =
                                        stats.get("output_tokens").and_then(Value::as_u64)
                                    {
                                        metadata.output_tokens = Some(output as u32);
                                    }
                                    if let Some(duration) =
                                        stats.get("duration_ms").and_then(Value::as_u64)
                                    {
                                        metadata.duration_ms = Some(duration);
                                    }
                                }

                                if let Some(status) = event.get("status").and_then(Value::as_str) {
                                    if status.eq_ignore_ascii_case("error") {
                                        finish_reason = FinishReason::Error;
                                    } else if finish_reason != FinishReason::ToolCall {
                                        finish_reason = FinishReason::Stop;
                                    }
                                }

                                metadata
                                    .extra
                                    .insert("result_event".to_string(), event.clone());
                            }
                            "error" => {
                                finish_reason = FinishReason::Error;
                                metadata.extra.insert("error_event".into(), event.clone());
                            }
                            _ => {}
                        }
                    }
                }
                Err(_) => aggregated.push(trimmed.to_string()),
            }
        }

        if aggregated.is_empty() {
            aggregated.push(response.trim().to_string());
        }

        if finish_reason == FinishReason::Stop && !tool_calls.is_empty() {
            finish_reason = FinishReason::ToolCall;
        }

        Ok(ParsedResponse {
            content: aggregated.join("\n"),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None,
        })
    }

    fn get_memory_filename(&self) -> &str {
        "GEMINI.md"
    }

    fn get_executable_name(&self) -> &str {
        "gemini"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        cli_capabilities(CLIType::Gemini)
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing Gemini adapter for container"
        );
        self.base
            .base_initialize(container)
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up Gemini adapter"
        );
        self.base
            .base_cleanup(container)
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn health_check(&self) -> Result<HealthStatus> {
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
            model: "gemini-2.5-pro".to_string(),
            max_tokens: Some(128_000),
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

        let sample_response = r#"{"type":"init","model":"gemini-2.5-pro","session_id":"abc123"}
{"type":"message","role":"assistant","content":"Working on the task now"}
{"type":"tool_use","tool_name":"bash","tool_id":"bash-1","parameters":{"command":"ls"}}
{"type":"result","status":"success","stats":{"input_tokens":512,"output_tokens":128,"duration_ms":900}}"#;

        health.details.insert(
            "response_parsing".to_string(),
            json!(self.parse_response(sample_response).await.is_ok()),
        );

        if health.details.values().any(|value| value == &json!(false)) {
            health.status = crate::cli::adapter::HealthState::Warning;
            health.message = Some("One or more Gemini adapter checks failed".to_string());
        }

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::ToolConfiguration;
    use serde_json::json;
    use std::path::PathBuf;

    fn templates_root() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("infra/charts/controller/agent-templates")
            .to_string_lossy()
            .into_owned()
    }

    fn sample_agent() -> AgentConfig {
        AgentConfig {
            github_app: "5DLabs-Rex".to_string(),
            cli: "gemini".to_string(),
            model: "gemini-2.5-pro".to_string(),
            max_tokens: Some(32_000),
            temperature: Some(0.4),
            tools: Some(ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gemini-2.5-pro-exp",
                "maxTokens": 64000,
                "temperature": 0.6,
                "settings": {
                    "toolmanUrl": "http://toolman.test/mcp",
                    "approvalPolicy": "auto_edit",
                    "sandbox": "workspace-write",
                    "general": {
                        "vimMode": true
                    }
                }
            })),
        }
    }

    #[tokio::test]
    async fn test_generate_config_includes_overrides() {
        std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        let adapter = GeminiAdapter::new().await.unwrap();
        let config = adapter.generate_config(&sample_agent()).await.unwrap();
        let parsed: Value = serde_json::from_str(&config).unwrap();
        assert_eq!(
            parsed
                .get("model")
                .and_then(|model| model.get("name"))
                .and_then(Value::as_str)
                .unwrap(),
            "gemini-2.5-pro-exp"
        );
        assert!(parsed
            .get("general")
            .and_then(|general| general.get("vimMode"))
            .and_then(Value::as_bool)
            .unwrap());
        assert_eq!(
            parsed
                .get("automation")
                .and_then(|automation| automation.get("approvalMode"))
                .and_then(Value::as_str)
                .unwrap(),
            "auto_edit"
        );
    }

    #[tokio::test]
    async fn test_parse_response_handles_stream_json() {
        let adapter = GeminiAdapter::new().await.unwrap();
        let sample = r#"{"type":"message","role":"assistant","content":"Working..."}
{"type":"tool_use","tool_name":"bash","tool_id":"bash-42","parameters":{"command":"ls"}}
{"type":"result","status":"success","stats":{"input_tokens":256,"output_tokens":64,"duration_ms":1200}}"#;

        let parsed = adapter.parse_response(sample).await.unwrap();
        assert!(parsed.content.contains("Working"));
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.metadata.input_tokens, Some(256));
        assert_eq!(parsed.metadata.output_tokens, Some(64));
        assert_eq!(parsed.metadata.duration_ms, Some(1200));
        assert_eq!(parsed.finish_reason, FinishReason::ToolCall);
    }
}
