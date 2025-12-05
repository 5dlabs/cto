//! Factory Droid CLI Adapter Implementation
//!
//! Provides a placeholder implementation so the controller can schedule
//! Factory CLI jobs without panicking. The heavier lifting (prompt
//! formatting, response parsing, configuration rendering) will be layered
//! in subsequent tasks.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use crate::tasks::template_paths::CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::info;

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

/// Factory CLI adapter skeleton.
/// Mirrors the Cursor placeholder: we wire the variant into the
/// adapter factory while we build out end-to-end behaviour.
#[derive(Debug)]
pub struct FactoryAdapter {
    base: Arc<BaseAdapter>,
}

impl FactoryAdapter {
    /// Create a new Factory adapter using default configuration.
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Factory))
    }

    /// Create a new Factory adapter with a custom base configuration.
    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Factory adapter");
        let base = Arc::new(BaseAdapter::new(config)?);
        Ok(Self { base })
    }
}

#[async_trait]
impl CliAdapter for FactoryAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool> {
        Ok(!model.trim().is_empty())
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        self.base.validate_base_config(agent_config)?;

        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));
        let settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let model = first_string(&cli_config, &["model"])
            .or_else(|| first_string(&settings, &["model"]))
            .filter(|value| !value.trim().is_empty())
            .map_or_else(|| agent_config.model.clone(), str::to_string);

        let max_output_tokens = first_u64(&cli_config, &["maxTokens", "modelMaxOutputTokens"])
            .or_else(|| first_u64(&settings, &["maxOutputTokens", "modelMaxOutputTokens"]))
            .and_then(|value| u32::try_from(value).ok())
            .or(agent_config.max_tokens);

        let temperature = first_f64(&cli_config, &["temperature"])
            .or_else(|| first_f64(&settings, &["temperature"]))
            .or_else(|| agent_config.temperature.map(f64::from));

        let approval_policy = first_string(&settings, &["approvalPolicy"]).unwrap_or("never");

        let sandbox_mode = first_string(&settings, &["sandboxPreset", "sandboxMode", "sandbox"])
            .unwrap_or("danger-full-access");

        let project_doc_max_bytes = first_u64(&settings, &["projectDocMaxBytes"]).unwrap_or(32_768);

        let reasoning_effort =
            first_string(&settings, &["reasoningEffort", "modelReasoningEffort"])
                .or_else(|| first_string(&cli_config, &["reasoningEffort"]))
                .map(std::string::ToString::to_string);

        let auto_level = settings
            .get("autoLevel")
            .and_then(Value::as_str)
            .or_else(|| cli_config.get("autoLevel").and_then(Value::as_str))
            .map(std::string::ToString::to_string)
            .or_else(|| reasoning_effort.clone());

        let output_format = settings
            .get("outputFormat")
            .or_else(|| settings.get("output_format"))
            .or_else(|| cli_config.get("outputFormat"))
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string);

        let raw_additional_json = first_string(&settings, &["rawJson", "raw_json"]) // legacy & snake_case
            .map(std::string::ToString::to_string);

        let tools_url = env::var("TOOLS_SERVER_URL")
            .unwrap_or_else(|_| "http://tools.cto.svc.cluster.local:3000/mcp".to_string());
        let tools_url = tools_url.trim_end_matches('/').to_string();

        let remote_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        let mut context = serde_json::Map::new();
        context.insert("model".to_string(), Value::String(model));
        if let Some(value) = temperature {
            context.insert("temperature".to_string(), json!(value));
        }
        if let Some(value) = max_output_tokens {
            context.insert("max_output_tokens".to_string(), json!(value));
        }
        context.insert(
            "approval_policy".to_string(),
            Value::String(approval_policy.to_string()),
        );
        context.insert(
            "sandbox_mode".to_string(),
            Value::String(sandbox_mode.to_string()),
        );
        context.insert(
            "project_doc_max_bytes".to_string(),
            json!(project_doc_max_bytes),
        );
        if let Some(value) = reasoning_effort.clone() {
            context.insert("reasoning_effort".to_string(), Value::String(value));
        }
        if let Some(value) = auto_level {
            context.insert("auto_level".to_string(), Value::String(value));
        }
        context.insert(
            "tools".to_string(),
            json!({
                "url": tools_url,
                "tools": remote_tools,
            }),
        );
        if let Some(value) = output_format {
            context.insert("output_format".to_string(), Value::String(value));
        }
        if let Some(raw) = raw_additional_json {
            context.insert("raw_additional_json".to_string(), Value::String(raw));
        }

        let rendered = self
            .base
            .render_template_file(CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE, &Value::Object(context))
            .map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to render Factory CLI config template: {e}"
                ))
            })?;

        Ok(rendered)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        if prompt.ends_with('\n') {
            prompt.to_string()
        } else {
            format!("{prompt}\n")
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        let mut aggregated_messages: Vec<String> = Vec::new();
        let mut plain_segments: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut metadata = ResponseMetadata::default();
        let mut finish_reason = FinishReason::Stop;
        let mut last_structured_event: Option<Value> = None;

        for raw_line in response.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(trimmed) {
                Ok(event) => {
                    let event_type = event.get("type").and_then(Value::as_str).unwrap_or("");
                    match event_type {
                        "message" => {
                            if event
                                .get("role")
                                .and_then(Value::as_str)
                                .is_none_or(|role| role.eq_ignore_ascii_case("assistant"))
                            {
                                if let Some(text) = event.get("text").and_then(Value::as_str) {
                                    aggregated_messages.push(text.to_string());
                                }
                            }
                        }
                        "tool_call" => {
                            let name = event
                                .get("toolName")
                                .and_then(Value::as_str)
                                .unwrap_or("tool_call");
                            let arguments = event.get("parameters").cloned().unwrap_or(Value::Null);
                            let id = event
                                .get("id")
                                .and_then(Value::as_str)
                                .or_else(|| event.get("callId").and_then(Value::as_str))
                                .map(std::string::ToString::to_string);
                            tool_calls.push(ToolCall {
                                name: name.to_string(),
                                arguments,
                                id,
                            });
                        }
                        "tool_result" => {
                            // Capture tool output in metadata for debugging purposes
                            last_structured_event = Some(event.clone());
                        }
                        "result" => {
                            let is_error = event
                                .get("is_error")
                                .or_else(|| event.get("isError"))
                                .and_then(Value::as_bool)
                                .unwrap_or(false);
                            if is_error {
                                finish_reason = FinishReason::Error;
                            }

                            if let Some(result_text) = event.get("result").and_then(Value::as_str) {
                                aggregated_messages.push(result_text.to_string());
                            } else if let Some(message) =
                                event.get("message").and_then(Value::as_str)
                            {
                                aggregated_messages.push(message.to_string());
                            }

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
                                if let Some(input) = usage
                                    .get("input_tokens")
                                    .or_else(|| usage.get("inputTokens"))
                                    .and_then(Value::as_u64)
                                {
                                    metadata.input_tokens = u32::try_from(input).ok();
                                }
                                if let Some(output) = usage
                                    .get("output_tokens")
                                    .or_else(|| usage.get("outputTokens"))
                                    .and_then(Value::as_u64)
                                {
                                    metadata.output_tokens = u32::try_from(output).ok();
                                }
                            }

                            last_structured_event = Some(event.clone());
                        }
                        "error" => {
                            finish_reason = FinishReason::Error;
                            if let Some(message) = event.get("message").and_then(Value::as_str) {
                                aggregated_messages.push(message.to_string());
                            }
                            last_structured_event = Some(event.clone());
                        }
                        _ => {
                            if let Some(text) = event.get("text").and_then(Value::as_str) {
                                aggregated_messages.push(text.to_string());
                            }
                            last_structured_event = Some(event.clone());
                        }
                    }
                }
                Err(_) => {
                    plain_segments.push(trimmed.to_string());
                }
            }
        }

        if !plain_segments.is_empty() {
            aggregated_messages.push(plain_segments.join("\n"));
        }

        if finish_reason == FinishReason::Stop && !tool_calls.is_empty() {
            finish_reason = FinishReason::ToolCall;
        }

        if let Some(last_event) = last_structured_event {
            metadata
                .extra
                .insert("factory_last_event".to_string(), last_event);
        }

        let content = if aggregated_messages.is_empty() {
            response.trim().to_string()
        } else {
            aggregated_messages.join("\n")
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
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &'static str {
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

        let mock_agent = AgentConfig {
            github_app: "factory-health".to_string(),
            cli: "factory".to_string(),
            model: "gpt-5-factory".to_string(),
            max_tokens: Some(16_000),
            temperature: Some(0.4),
            tools: Some(ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gpt-5-factory-high",
                "maxTokens": 64000,
                "settings": {
                    "sandboxMode": "danger-full-access",
                    "approvalPolicy": "never",
                    "reasoningEffort": "high"
                }
            })),
        };

        let config_result = self.generate_config(&mock_agent).await;
        health.details.insert(
            "config_generation".to_string(),
            Value::Bool(config_result.is_ok()),
        );

        let sample_response = r#"{"type":"result","is_error":false,"result":"Factory execution complete","model":"gpt-5-factory","duration_ms":1200,"usage":{"input_tokens":256,"output_tokens":512}}"#;
        let parse_result = self.parse_response(sample_response).await;
        health.details.insert(
            "response_parsing".to_string(),
            Value::Bool(parse_result.is_ok()),
        );

        if config_result.is_err() || parse_result.is_err() {
            health.status = HealthState::Warning;
        }

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::test_utils::templates_root;
    use serde_json::json;
    use serial_test::serial;

    fn sample_agent_config() -> AgentConfig {
        AgentConfig {
            github_app: "5DLabs-Rex".to_string(),
            cli: "factory".to_string(),
            model: "gpt-5-factory".to_string(),
            max_tokens: Some(32_000),
            temperature: Some(0.5),
            tools: Some(ToolConfiguration {
                remote: vec![
                    "memory_create_entities".to_string(),
                    "brave_search_brave_web_search".to_string(),
                ],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gpt-5-factory-high",
                "maxTokens": 64000,
                "temperature": 0.42,
                "settings": {
                    "sandboxMode": "workspace-write",
                    "approvalPolicy": "never",
                    "projectDocMaxBytes": 65536,
                    "reasoningEffort": "high",
                    "rawJson": "\"custom\":{\"enabled\":true}"
                }
            })),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_config_renders_factory_template() {
        // SAFETY: This test runs serially via #[serial] to avoid env var races
        unsafe {
            std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
            std::env::set_var("TOOLS_SERVER_URL", "http://localhost:3000/mcp");
        }

        let adapter = FactoryAdapter::new().unwrap();
        let config = adapter
            .generate_config(&sample_agent_config())
            .await
            .unwrap();

        let parsed: Value = serde_json::from_str(&config).expect("config should be valid JSON");
        assert_eq!(
            parsed
                .get("model")
                .and_then(|model| model.get("default"))
                .and_then(Value::as_str)
                .unwrap(),
            "gpt-5-factory-high"
        );
        assert_eq!(
            parsed
                .get("execution")
                .and_then(|exec| exec.get("sandboxMode"))
                .and_then(Value::as_str)
                .unwrap(),
            "workspace-write"
        );
        assert!(parsed
            .get("autoRun")
            .and_then(|auto| auto.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap());
        assert_eq!(
            parsed
                .get("autoRun")
                .and_then(|auto| auto.get("level"))
                .and_then(Value::as_str)
                .unwrap(),
            "high"
        );
        let tools = parsed
            .get("tools")
            .and_then(|tools| tools.get("tools"))
            .and_then(Value::as_array)
            .expect("tool list");
        assert!(tools.contains(&Value::String("memory_create_entities".to_string())));
        assert!(tools.contains(&Value::String("brave_search_brave_web_search".to_string())));
    }

    #[tokio::test]
    async fn test_parse_response_extracts_metadata() {
        let adapter = FactoryAdapter::new().unwrap();
        let response = r#"{"type":"message","role":"assistant","text":"Running tasks"}
{"type":"tool_call","toolName":"Execute","parameters":{"command":"ls"}}
{"type":"result","is_error":false,"result":"Task completed","model":"gpt-5-factory","duration_ms":987,"usage":{"input_tokens":128,"output_tokens":256}}"#;

        let parsed = adapter.parse_response(response).await.unwrap();

        assert!(parsed.content.contains("Task completed"));
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].name, "Execute");
        assert_eq!(parsed.finish_reason, FinishReason::ToolCall);
        assert_eq!(parsed.metadata.model.as_deref(), Some("gpt-5-factory"));
        assert_eq!(parsed.metadata.duration_ms, Some(987));
        assert_eq!(parsed.metadata.input_tokens, Some(128));
        assert_eq!(parsed.metadata.output_tokens, Some(256));
        assert!(parsed.metadata.extra.contains_key("factory_last_event"));
    }

    #[tokio::test]
    #[serial]
    async fn test_health_check_reports_details() {
        // SAFETY: This test runs serially via #[serial] to avoid env var races
        unsafe {
            std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        }
        let adapter = FactoryAdapter::new().unwrap();
        let health = adapter.health_check().await.unwrap();

        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }
}
