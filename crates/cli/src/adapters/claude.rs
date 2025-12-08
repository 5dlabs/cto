//! Claude CLI Adapter Implementation

use crate::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::base_adapter::{AdapterConfig, BaseAdapter};
use crate::types::CLIType;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// Claude CLI adapter - reference implementation
#[derive(Debug)]
pub struct ClaudeAdapter {
    base: Arc<BaseAdapter>,
    model_validator: Arc<ClaudeModelValidator>,
}

impl ClaudeAdapter {
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude))
    }

    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");
        let base = Arc::new(BaseAdapter::new(config)?);
        let model_validator = Arc::new(ClaudeModelValidator::new());

        Ok(Self {
            base,
            model_validator,
        })
    }

    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        let tools_url = std::env::var("TOOLS_SERVER_URL")
            .unwrap_or_else(|_| "http://tools.cto.svc.cluster.local:3000/mcp".to_string());
        let tools_url = tools_url.trim_end_matches('/').to_string();

        if let Some(tool_config) = tools {
            for tool_name in &tool_config.remote {
                let server_config = json!({
                    "command": "tools",
                    "args": ["--url", tools_url.clone(), "--tool", tool_name],
                    "env": {
                        "TOOLS_SERVER_URL": tools_url.clone()
                    }
                });
                mcp_servers[tool_name] = server_config;
            }

            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = json!({
                            "command": format!("mcp-server-{}", server_name),
                            "args": [],
                            "env": {}
                        });
                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }

    fn extract_tool_calls(response: &str) -> AdapterResult<Vec<ToolCall>> {
        let mut tool_calls = Vec::new();

        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                let mut arguments = serde_json::Map::new();
                for param_match in parameter_regex.captures_iter(parameters_content) {
                    let param_name = &param_match[1];
                    let param_value = &param_match[2];
                    arguments.insert(
                        param_name.to_string(),
                        Value::String(param_value.to_string()),
                    );
                }

                tool_calls.push(ToolCall {
                    name: tool_name.to_string(),
                    arguments: Value::Object(arguments),
                    id: Some(format!("tool_{}", tool_calls.len())),
                });
            }
        }

        debug!(
            tool_calls_count = tool_calls.len(),
            "Extracted tool calls from response"
        );
        Ok(tool_calls)
    }

    fn extract_response_metadata(_response: &str) -> ResponseMetadata {
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");
        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);
        let is_valid = self.model_validator.validate(model)?;
        info!(model = %model, is_valid = is_valid, "Model validation completed");
        Ok(is_valid)
    }

    #[instrument(skip(self, agent_config))]
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        debug!(
            github_app = %agent_config.github_app,
            model = %agent_config.model,
            "Generating Claude configuration"
        );

        self.base.validate_base_config(agent_config)?;

        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        let context = json!({
            "model": agent_config.model,
            "max_tokens": agent_config.max_tokens.unwrap_or(default_max_tokens),
            "temperature": agent_config.temperature.unwrap_or(default_temperature),
            "github_app": agent_config.github_app,
            "cli": agent_config.cli,
            "mcp_servers": mcp_servers,
            "tools": agent_config.tools,
            "cli_config": agent_config.cli_config,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "correlation_id": self.base.config.correlation_id,
            "generated_by": "cli_adapter_claude",
            "version": "1.0",
        });

        let config = serde_json::to_string_pretty(&context).map_err(|e| {
            error!(error = %e, "Failed to serialize Claude configuration");
            AdapterError::ConfigGenerationError(format!("Failed to serialize config: {e}"))
        })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );
        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        let tool_calls = Self::extract_tool_calls(response)?;

        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        let metadata = Self::extract_response_metadata(response);

        Ok(ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None,
        })
    }

    fn get_memory_filename(&self) -> &'static str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000);

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens,
            memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::SessionToken],
        }
    }

    #[instrument(skip(self, container))]
    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(container_name = %container.container_name, "Initializing Claude adapter for container");
        self.base.base_initialize(container).await?;

        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        if let Some(claude_session) = container.env_vars.get("CLAUDE_SESSION_TOKEN") {
            debug!(
                has_session_token = !claude_session.is_empty(),
                "Claude session token status"
            );
        }

        info!("Claude adapter initialization completed");
        Ok(())
    }

    #[instrument(skip(self, container))]
    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(container_name = %container.container_name, "Cleaning up Claude adapter");
        self.base.base_cleanup(container).await?;
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let mut base_health = self.base.base_health_check(&container).await?;

        let mut claude_checks = HashMap::new();

        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        let test_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        let config_generation_test = self.generate_config(&test_config).await;
        claude_checks.insert(
            "config_generation".to_string(),
            json!(config_generation_test.is_ok()),
        );

        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        let all_checks_passed = base_health
            .details
            .values()
            .all(|v| v.as_bool().unwrap_or(false));

        if !all_checks_passed && base_health.status == HealthState::Healthy {
            base_health.status = HealthState::Warning;
            base_health.message = Some("Some Claude-specific checks failed".to_string());
        }

        info!(status = ?base_health.status, "Claude adapter health check completed");
        Ok(base_health)
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    #[must_use]
    pub fn new() -> Self {
        let valid_patterns = vec![
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            Regex::new(r"^claude-4.*").unwrap(),
            Regex::new(r"^claude-sonnet-4.*").unwrap(),
            Regex::new(r"^claude-opus-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub fn validate(&self, model: &str) -> AdapterResult<bool> {
        if model.trim().is_empty() {
            return Ok(false);
        }

        let is_valid = self
            .valid_patterns
            .iter()
            .any(|pattern| pattern.is_match(model));

        debug!(model = %model, is_valid = is_valid, "Claude model validation result");
        Ok(is_valid)
    }

    #[must_use]
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-sonnet-4-5-20250929".to_string(),
            "claude-opus-4-5-20251101".to_string(),
            "claude-sonnet-4-20250514".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "opus".to_string(),
            "sonnet".to_string(),
            "haiku".to_string(),
        ]
    }
}

impl Default for ClaudeModelValidator {
    fn default() -> Self {
        Self::new()
    }
}

