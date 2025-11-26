//! Codex CLI Adapter Implementation
//!
//! Implements the `CliAdapter` trait for the Codex CLI, generating the
//! appropriate TOML configuration, handling prompt formatting, response parsing,
//! and health checks.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthStatus, MemoryStrategy, ParsedResponse,
    ResponseMetadata, ToolCall,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::{debug, info, instrument};

const CODEX_CONFIG_TEMPLATE: &str = "code/codex/config.toml.hbs";
const CODEX_MEMORY_TEMPLATE: &str = "code/codex/agents.md.hbs";

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

/// Codex CLI adapter implementation
#[derive(Debug)]
pub struct CodexAdapter {
    base: Arc<BaseAdapter>,
    config_template_name: &'static str,
    memory_template_name: &'static str,
}

impl CodexAdapter {
    /// Create a new Codex adapter using default configuration.
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Codex))
    }

    /// Create a new Codex adapter with custom configuration.
    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Codex adapter");

        let base = Arc::new(BaseAdapter::new(config)?);

        let adapter = Self {
            base,
            config_template_name: CODEX_CONFIG_TEMPLATE,
            memory_template_name: CODEX_MEMORY_TEMPLATE,
        };

        info!("Codex adapter initialized successfully");
        Ok(adapter)
    }

    fn render_memory_file(&self, agent_config: &AgentConfig) -> AdapterResult<String> {
        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));

        let tools_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        let context = json!({
            "cli_config": cli_config,
            "github_app": agent_config.github_app,
            "model": agent_config.model,
            "tools": {
                "tools": tools_tools,
            },
        });

        self.base
            .render_template_file(self.memory_template_name, &context)
            .map_err(|e| {
                AdapterError::TemplateError(format!("Failed to render Codex memory template: {e}",))
            })
    }

    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        self.base
            .render_template_file(self.config_template_name, context)
            .map_err(|e| {
                AdapterError::TemplateError(format!("Failed to render Codex config template: {e}",))
            })
    }

    #[instrument(skip(self, response))]
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        let mut tool_calls = Vec::new();

        if let Ok(json_val) = serde_json::from_str::<Value>(response) {
            if let Some(commands) = json_val.get("commands").and_then(Value::as_array) {
                for (idx, command) in commands.iter().enumerate() {
                    let name = command
                        .get("command")
                        .and_then(Value::as_str)
                        .unwrap_or("local_shell");
                    let args = command.get("args").cloned().unwrap_or(Value::Null);
                    let arguments = match args {
                        Value::Null => Value::Object(serde_json::Map::new()),
                        other => other,
                    };

                    tool_calls.push(ToolCall {
                        name: name.to_string(),
                        arguments,
                        id: Some(format!("tool_{idx}")),
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

            if let Some(tokens) = json_val.get("usage") {
                metadata.input_tokens = tokens
                    .get("input_tokens")
                    .and_then(Value::as_i64)
                    .and_then(|v| u32::try_from(v).ok());
                metadata.output_tokens = tokens
                    .get("output_tokens")
                    .and_then(Value::as_i64)
                    .and_then(|v| u32::try_from(v).ok());
            }
        }

        metadata
    }
}

#[async_trait]
impl CliAdapter for CodexAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool> {
        // Codex accepts arbitrary model names; validation is deferred to runtime
        Ok(true)
    }

    #[allow(clippy::too_many_lines)]
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        debug!(
            github_app = %agent_config.github_app,
            model = %agent_config.model,
            "Generating Codex configuration"
        );

        self.base.validate_base_config(agent_config)?;

        let cli_config = agent_config.cli_config.clone().unwrap_or_else(|| json!({}));

        let model = first_string(&cli_config, &["model"])
            .map_or_else(|| agent_config.model.clone(), str::to_string);

        let max_output_tokens = first_u64(&cli_config, &["maxTokens", "modelMaxOutputTokens"])
            .and_then(|value| u32::try_from(value).ok())
            .or(agent_config.max_tokens);

        let temperature = first_f64(&cli_config, &["temperature"])
            .and_then(safe_f32)
            .or(agent_config.temperature);

        let approval_policy = first_string(&cli_config, &["approvalPolicy"])
            .unwrap_or("never")
            .to_string();

        let sandbox_mode = first_string(&cli_config, &["sandboxPreset", "sandboxMode", "sandbox"])
            .unwrap_or("danger-full-access")
            .to_string();

        let project_doc_max_bytes =
            first_u64(&cli_config, &["projectDocMaxBytes"]).unwrap_or(32_768);

        let tools_url = env::var("TOOLS_SERVER_URL")
            .unwrap_or_else(|_| "http://tools.cto.svc.cluster.local:3000/mcp".to_string());
        let tools_url = tools_url.trim_end_matches('/').to_string();

        let tools_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        let model_provider = if let Some(provider_map) =
            cli_config.get("modelProvider").and_then(Value::as_object)
        {
            let name = provider_map
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("OpenAI");
            let base_url = provider_map
                .get("base_url")
                .and_then(Value::as_str)
                .or_else(|| provider_map.get("baseUrl").and_then(Value::as_str))
                .unwrap_or("https://api.openai.com/v1");
            let env_key = provider_map
                .get("env_key")
                .and_then(Value::as_str)
                .or_else(|| provider_map.get("envKey").and_then(Value::as_str))
                .unwrap_or("OPENAI_API_KEY");
            let wire_api = provider_map
                .get("wire_api")
                .and_then(Value::as_str)
                .or_else(|| provider_map.get("wireApi").and_then(Value::as_str))
                .unwrap_or("chat");

            json!({
                "name": name,
                "base_url": base_url,
                "env_key": env_key,
                "wire_api": wire_api,
                "request_max_retries": provider_map
                    .get("request_max_retries")
                    .and_then(Value::as_u64),
                "stream_max_retries": provider_map
                    .get("stream_max_retries")
                    .and_then(Value::as_u64),
            })
        } else {
            json!({
                "name": "OpenAI",
                "base_url": "https://api.openai.com/v1",
                "env_key": "OPENAI_API_KEY",
                "wire_api": "chat",
            })
        };

        let raw_additional_toml = cli_config
            .get("rawToml")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                cli_config
                    .get("raw_config")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });

        let reasoning_effort = cli_config
            .get("settings")
            .and_then(Value::as_object)
            .and_then(|settings| settings.get("reasoningEffort"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let context = json!({
            "model": model,
            "github_app": agent_config.github_app,
            "cli": agent_config.cli,
            "max_output_tokens": max_output_tokens,
            "temperature": temperature,
            "approval_policy": approval_policy,
            "sandbox_mode": sandbox_mode,
            "project_doc_max_bytes": project_doc_max_bytes,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "correlation_id": self.base.config.correlation_id,
            "tools": {
                "url": tools_url,
                "tools": tools_tools,
            },
            "model_provider": model_provider,
            "cli_config": cli_config,
            "model_reasoning_effort": reasoning_effort,
            "raw_additional_toml": raw_additional_toml,
        });

        let config = self.render_config(&context)?;

        info!(
            config_length = config.len(),
            "Codex configuration generated successfully"
        );
        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        prompt.to_string()
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Codex response");

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
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "codex"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: false,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".to_string()),
            config_format: ConfigFormat::Toml,
            authentication_methods: vec![AuthMethod::ApiKey],
        }
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing Codex adapter for container"
        );

        self.base.base_initialize(container).await?;

        let agents_path = format!("{}/AGENTS.md", container.working_dir);
        debug!(agents_path = %agents_path, "Codex memory file path");

        info!("Codex adapter initialization completed");
        Ok(())
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up Codex adapter"
        );

        self.base.base_cleanup(container).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Codex adapter health check");

        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let mut health = self.base.base_health_check(&container).await?;

        let mock_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "codex".to_string(),
            model: "gpt-5-codex".to_string(),
            max_tokens: Some(8192),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        let config_result = self.generate_config(&mock_config).await;
        health.details.insert(
            "config_generation".to_string(),
            serde_json::json!(config_result.is_ok()),
        );

        let memory_result = self.render_memory_file(&mock_config);
        health.details.insert(
            "memory_render".to_string(),
            serde_json::json!(memory_result.is_ok()),
        );

        let parse_result = self.parse_response("{}").await;
        health.details.insert(
            "response_parsing".to_string(),
            serde_json::json!(parse_result.is_ok()),
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
            cli: "codex".to_string(),
            model: "fallback-model".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.5),
            tools: Some(ToolConfiguration {
                remote: vec![
                    "memory_create_entities".to_string(),
                    "rustdocs_query_rust_docs".to_string(),
                ],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gpt-5-codex",
                "maxTokens": 64000,
                "temperature": 0.72,
                "approvalPolicy": "on-request",
                "sandboxPreset": "workspace-write",
                "instructions": "Follow team standards",
                "settings": {
                    "reasoningEffort": "medium"
                }
            })),
        }
    }

    #[tokio::test]
    async fn test_generate_config_applies_overrides() {
        std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        std::env::set_var("TOOLS_SERVER_URL", "http://localhost:9000/mcp");

        let adapter = CodexAdapter::new().unwrap();
        let agent_config = sample_agent_config();
        let expected_model = agent_config
            .cli_config
            .as_ref()
            .and_then(|cfg| cfg.get("model").and_then(|m| m.as_str()))
            .expect("sample agent config should include model override");

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains(&format!("model = \"{expected_model}\"")));
        assert!(config.contains("model_max_output_tokens = 64000"));
        assert!(config.contains("temperature = 0.72"));
        assert!(config.contains("approval_policy = \"on-request\""));
        assert!(config.contains("sandbox_mode = \"workspace-write\""));
        assert!(config.contains("project_doc_max_bytes = 32768"));
        assert!(config.contains("model_reasoning_effort = \"medium\""));
        assert!(config.contains("[mcp_servers.tools]"));
        assert!(config.contains("--url"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("[model_providers.openai]"));
    }

    #[tokio::test]
    async fn test_memory_template_includes_tools_and_instructions() {
        std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
        let adapter = CodexAdapter::new().unwrap();
        let agent_config = sample_agent_config();

        let memory = adapter.render_memory_file(&agent_config).unwrap();

        assert!(memory.contains("Follow team standards"));
        assert!(memory.contains("memory_create_entities"));
    }
}
