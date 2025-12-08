//! Factory Droid CLI Adapter Implementation

use crate::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::base_adapter::{AdapterConfig, BaseAdapter};
use crate::types::CLIType;
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

/// Factory CLI adapter
#[derive(Debug)]
pub struct FactoryAdapter {
    base: Arc<BaseAdapter>,
}

impl FactoryAdapter {
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Factory))
    }

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

        let rendered = serde_json::to_string_pretty(&Value::Object(context)).map_err(|e| {
            AdapterError::ConfigGenerationError(format!("Failed to serialize Factory config: {e}"))
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
        info!(container_name = %container.container_name, "Factory adapter placeholder initialize");
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


