//! Every Code CLI Adapter Implementation
//!
//! Implements the `CliAdapter` trait for Every Code (`code`), a fork of the
//! OpenAI Codex CLI with multi-agent support, browser integration, theming,
//! and enhanced reasoning controls.
//!
//! <https://github.com/just-every/code>
//!
//! Key differences from Codex:
//! - Binary: `code` instead of `codex`
//! - Multi-agent commands: `/plan`, `/solve`, `/code`, `/auto`
//! - Browser integration: `/chrome`, `/browser`
//! - Non-interactive flag: `--no-approval` instead of `--quiet`
//! - Multi-provider: OpenAI, Claude, Gemini, Qwen
//!
//! ## Multi-Agent Modes
//!
//! Every Code's killer feature is multi-agent consensus/racing:
//!
//! - `/plan` - Claude + Gemini + GPT-5 create a consolidated plan (consensus)
//! - `/solve` - Fastest-first race between models (see arxiv.org/abs/2505.17813)
//! - `/code` - Multi-worktree implementation with consensus review
//! - `/auto` - Auto Drive: multi-step task orchestration with agent coordination
//!
//! ## Config Profiles
//!
//! Every Code supports named profiles in `~/.code/config.toml`:
//! ```toml
//! [profiles.high-reasoning]
//! model = "gpt-5.1"
//! model_reasoning_effort = "high"
//! model_reasoning_summary = "detailed"
//! ```
//!
//! ## CTO Integration
//!
//! For Play workflow, we recommend:
//! - **Planning tasks**: Use `/plan` for consensus-based task breakdown
//! - **Implementation**: Use default mode or `/code` for complex features
//! - **Debugging/fixing**: Use `/solve` for fastest resolution
//! - **Multi-step orchestration**: Use `/auto` for autonomous task sequences

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

const CODE_MEMORY_TEMPLATE: &str = "code/codex/agents-default.md.hbs";

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

/// Every Code CLI adapter implementation
#[derive(Debug)]
pub struct CodeAdapter {
    base: Arc<BaseAdapter>,
    memory_template_name: &'static str,
}

impl CodeAdapter {
    /// Create a new Code adapter using default configuration.
    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Code))
    }

    /// Create a new Code adapter with custom configuration.
    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Every Code adapter");

        let base = Arc::new(BaseAdapter::new(config)?);

        let adapter = Self {
            base,
            memory_template_name: CODE_MEMORY_TEMPLATE,
        };

        info!("Every Code adapter initialized successfully");
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
                AdapterError::TemplateError(format!(
                    "Failed to render Every Code memory template: {e}",
                ))
            })
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    fn render_config(&self, context: &Value) -> AdapterResult<String> {
        use std::fmt::Write;

        // Generate TOML configuration for Every Code (~/.code/config.toml)
        let model = context["model"].as_str().unwrap_or("gpt-5.1");
        let model_provider = context["model_provider_name"].as_str().unwrap_or("openai");
        let approval_policy = context["approval_policy"].as_str().unwrap_or("never");
        let sandbox_mode = context["sandbox_mode"]
            .as_str()
            .unwrap_or("workspace-write");
        let reasoning_effort = context["model_reasoning_effort"]
            .as_str()
            .unwrap_or("medium");

        let mut toml = format!(
            r#"# Every Code CLI Configuration
# Generated by CTO controller
# https://github.com/just-every/code

model = "{model}"
model_provider = "{model_provider}"
approval_policy = "{approval_policy}"
sandbox_mode = "{sandbox_mode}"
model_reasoning_effort = "{reasoning_effort}"
"#
        );

        // Add optional fields
        if let Some(temp) = context["temperature"].as_f64() {
            let _ = writeln!(toml, "temperature = {temp}");
        }
        if let Some(max_tokens) = context["max_output_tokens"].as_u64() {
            let _ = writeln!(toml, "model_max_output_tokens = {max_tokens}");
        }
        if let Some(reasoning_summary) = context["model_reasoning_summary"].as_str() {
            let _ = writeln!(toml, "model_reasoning_summary = \"{reasoning_summary}\"");
        }

        // Add MCP servers section if tools are present
        if let Some(tools) = context["tools"].as_object() {
            if let Some(url) = tools.get("url").and_then(|v| v.as_str()) {
                let _ = write!(
                    toml,
                    r#"
[mcp_servers.tools]
command = "tools"
args = ["--url", "{url}", "--working-dir", "/workspace"]
"#
                );
            }
        }

        // Add theme section
        let _ = write!(
            toml,
            r#"
[tui.theme]
name = "dark-nebula"
"#
        );

        // Add task-specific profiles for CTO workflow stages
        // These can be activated via --config profile=<name>
        let _ = write!(
            toml,
            r#"
# Profile for planning tasks (consensus mode)
[profiles.planning]
model_reasoning_effort = "high"
model_reasoning_summary = "detailed"

# Profile for implementation tasks
[profiles.implementation]
model_reasoning_effort = "medium"
model_reasoning_summary = "auto"

# Profile for quick fixes and debugging (race mode)
[profiles.quickfix]
model_reasoning_effort = "low"
model_reasoning_summary = "none"

# Profile for quality review
[profiles.review]
model_reasoning_effort = "high"
model_reasoning_summary = "detailed"
"#
        );

        Ok(toml)
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
impl CliAdapter for CodeAdapter {
    async fn validate_model(&self, _model: &str) -> Result<bool> {
        // Every Code supports multiple providers, validation is deferred to runtime
        Ok(true)
    }

    #[allow(clippy::too_many_lines)]
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        debug!(
            github_app = %agent_config.github_app,
            model = %agent_config.model,
            "Generating Every Code configuration"
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

        // Every Code uses 'never' for CI/automation
        let approval_policy = first_string(&cli_config, &["approvalPolicy"])
            .unwrap_or("never")
            .to_string();

        let sandbox_mode = first_string(&cli_config, &["sandboxPreset", "sandboxMode", "sandbox"])
            .unwrap_or("workspace-write")
            .to_string();

        let tools_url = env::var("TOOLS_SERVER_URL")
            .unwrap_or_else(|_| "http://tools.cto.svc.cluster.local:3000/mcp".to_string());
        let tools_url = tools_url.trim_end_matches('/').to_string();

        let tools_tools = agent_config
            .tools
            .as_ref()
            .map(|tools| tools.remote.clone())
            .unwrap_or_default();

        // Determine model provider from model name
        let model_provider_name = if model.starts_with("claude") {
            "anthropic"
        } else if model.starts_with("gemini") {
            "google"
        } else if model.starts_with("qwen") {
            "qwen"
        } else {
            "openai"
        };

        let reasoning_effort = cli_config
            .get("settings")
            .and_then(Value::as_object)
            .and_then(|settings| settings.get("reasoningEffort"))
            .and_then(Value::as_str)
            .unwrap_or("medium")
            .to_string();

        let reasoning_summary = cli_config
            .get("settings")
            .and_then(Value::as_object)
            .and_then(|settings| settings.get("reasoningSummary"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let context = json!({
            "model": model,
            "model_provider_name": model_provider_name,
            "github_app": agent_config.github_app,
            "cli": agent_config.cli,
            "max_output_tokens": max_output_tokens,
            "temperature": temperature,
            "approval_policy": approval_policy,
            "sandbox_mode": sandbox_mode,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "correlation_id": self.base.config.correlation_id,
            "tools": {
                "url": tools_url,
                "tools": tools_tools,
            },
            "cli_config": cli_config,
            "model_reasoning_effort": reasoning_effort,
            "model_reasoning_summary": reasoning_summary,
        });

        let config = self.render_config(&context)?;

        info!(
            config_length = config.len(),
            "Every Code configuration generated successfully"
        );
        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        prompt.to_string()
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(
            response_length = response.len(),
            "Parsing Every Code response"
        );

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
        // Every Code uses AGENTS.md (same as Codex)
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "code"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true, // Every Code supports streaming
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".to_string()),
            config_format: ConfigFormat::Toml,
            authentication_methods: vec![
                AuthMethod::ApiKey,
                AuthMethod::OAuth, // ChatGPT Plus sign-in
            ],
        }
    }

    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing Every Code adapter for container"
        );

        self.base.base_initialize(container).await?;

        let agents_path = format!("{}/AGENTS.md", container.working_dir);
        debug!(agents_path = %agents_path, "Every Code memory file path");

        info!("Every Code adapter initialization completed");
        Ok(())
    }

    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up Every Code adapter"
        );

        self.base.base_cleanup(container).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Every Code adapter health check");

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
            cli: "code".to_string(),
            model: "gpt-5.1".to_string(),
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

        let memory_result = self.render_memory_file(&mock_config);
        health
            .details
            .insert("memory_render".to_string(), json!(memory_result.is_ok()));

        let parse_result = self.parse_response("{}").await;
        health
            .details
            .insert("response_parsing".to_string(), json!(parse_result.is_ok()));

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::ToolConfiguration;
    use crate::cli::test_utils::templates_root;
    use serial_test::serial;

    fn sample_agent_config() -> AgentConfig {
        AgentConfig {
            github_app: "test-app".to_string(),
            cli: "code".to_string(),
            model: "fallback-model".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.5),
            tools: Some(ToolConfiguration {
                remote: vec![
                    "memory_create_entities".to_string(),
                    "brave_search_brave_web_search".to_string(),
                ],
                local_servers: None,
            }),
            cli_config: Some(json!({
                "model": "gpt-5.1",
                "maxTokens": 64000,
                "temperature": 0.72,
                "approvalPolicy": "never",
                "sandboxPreset": "workspace-write",
                "settings": {
                    "reasoningEffort": "high",
                    "reasoningSummary": "detailed"
                }
            })),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_config_applies_overrides() {
        // SAFETY: This test runs serially via #[serial] to avoid env var races
        unsafe {
            std::env::set_var("CLI_TEMPLATES_ROOT", templates_root());
            std::env::set_var("TOOLS_SERVER_URL", "http://localhost:9000/mcp");
        }

        let adapter = CodeAdapter::new().unwrap();
        let agent_config = sample_agent_config();

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("model = \"gpt-5.1\""));
        assert!(config.contains("model_provider = \"openai\""));
        assert!(config.contains("approval_policy = \"never\""));
        assert!(config.contains("sandbox_mode = \"workspace-write\""));
        assert!(config.contains("model_reasoning_effort = \"high\""));
        assert!(config.contains("[mcp_servers.tools]"));
    }

    #[tokio::test]
    async fn test_get_executable_name() {
        let adapter = CodeAdapter::new().unwrap();
        assert_eq!(adapter.get_executable_name(), "code");
    }

    #[tokio::test]
    async fn test_get_memory_filename() {
        let adapter = CodeAdapter::new().unwrap();
        assert_eq!(adapter.get_memory_filename(), "AGENTS.md");
    }

    #[tokio::test]
    async fn test_capabilities() {
        let adapter = CodeAdapter::new().unwrap();
        let caps = adapter.get_capabilities();

        assert!(caps.supports_streaming);
        assert!(caps.supports_function_calling);
        assert_eq!(caps.max_context_tokens, 128_000);
        assert!(matches!(caps.config_format, ConfigFormat::Toml));
    }

    #[tokio::test]
    async fn test_model_provider_detection() {
        let adapter = CodeAdapter::new().unwrap();

        // Test Claude model
        let claude_config = AgentConfig {
            github_app: "test".to_string(),
            cli: "code".to_string(),
            model: "claude-opus-4-5-20250929".to_string(),
            max_tokens: None,
            temperature: None,
            tools: None,
            cli_config: None,
        };
        let config = adapter.generate_config(&claude_config).await.unwrap();
        assert!(config.contains("model_provider = \"anthropic\""));

        // Test Gemini model
        let gemini_config = AgentConfig {
            github_app: "test".to_string(),
            cli: "code".to_string(),
            model: "gemini-2.5-pro".to_string(),
            max_tokens: None,
            temperature: None,
            tools: None,
            cli_config: None,
        };
        let config = adapter.generate_config(&gemini_config).await.unwrap();
        assert!(config.contains("model_provider = \"google\""));
    }
}
