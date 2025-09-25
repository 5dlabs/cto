//! Codex CLI Adapter Implementation
//!
//! Implements the `CliAdapter` trait for the Codex CLI, generating the
//! appropriate TOML configuration, handling prompt formatting, response parsing,
//! and health checks.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

const CODEX_CONFIG_TEMPLATE: &str = "code/codex-config.toml.hbs";
const CODEX_MEMORY_TEMPLATE: &str = "code/codex-agents.md.hbs";

/// Codex CLI adapter implementation
#[derive(Debug)]
pub struct CodexAdapter {
    base: Arc<BaseAdapter>,
    config_template_name: &'static str,
    memory_template_name: &'static str,
}

impl CodexAdapter {
    /// Create a new Codex adapter using default configuration.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Codex)).await
    }

    /// Create a new Codex adapter with custom configuration.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Codex adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);

        let adapter = Self {
            base,
            config_template_name: CODEX_CONFIG_TEMPLATE,
            memory_template_name: CODEX_MEMORY_TEMPLATE,
        };

        info!("Codex adapter initialized successfully");
        Ok(adapter)
    }

    fn render_memory_file(&self, agent_config: &AgentConfig) -> AdapterResult<String> {
        let context = serde_json::json!({
            "instructions": agent_config.cli_config,
        });

        self.base
            .render_template(self.memory_template_name, &context)
            .map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to render Codex memory template: {e}",
                ))
            })
    }

    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        self.base
            .render_template(self.config_template_name, context)
            .map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to render Codex config template: {e}",
                ))
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
                    .map(|v| v as u32);
                metadata.output_tokens = tokens
                    .get("output_tokens")
                    .and_then(Value::as_i64)
                    .map(|v| v as u32);
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

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        debug!(
            github_app = %agent_config.github_app,
            model = %agent_config.model,
            "Generating Codex configuration"
        );

        self.base.validate_base_config(agent_config)?;

        let mcp_servers = serde_json::json!({});

        let context = serde_json::json!({
            "model": agent_config.model,
            "github_app": agent_config.github_app,
            "cli": agent_config.cli,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "correlation_id": self.base.config.correlation_id,
            "mcp_servers": mcp_servers,
            "cli_config": agent_config.cli_config,
        });

        let config = self.render_config(&context)?;

        info!(config_length = config.len(), "Codex configuration generated successfully");
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

    fn get_memory_filename(&self) -> &str {
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &str {
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
            model: "gpt-4.1-mini".to_string(),
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

        let parse_result = self.parse_response("{}").await;
        health
            .details
            .insert("response_parsing".to_string(), serde_json::json!(parse_result.is_ok()));

        Ok(health)
    }
}


