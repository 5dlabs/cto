# Implementing CLI-agnostic Codex agent
_Exported on 9/25/2025 at 09:58:13 PDT from Cursor (1.6.45)_

---

**User**

@task-4/ @architecture.md @task-5/ @task-6/ @task-7/ @task-8/ @task-9/ @task-10/ @task-11/ @task-12/ hey GPT, we want to complete our CLI-agnostic implementation giving priority to Codex, but making sure that we have it structured in a way that will give us a standard way for onboarding new CLIs. It might be prudent to understand the differences and nuances, especially in terms of how prompts, memory, and MCP settings are added so that it follows the same patterns and processes that we already have for Claude. We have managed to get up to Task 3 merged. Now a reminder, we should obviously be checking out to a feature branch since we can't push directly to main. Our first milestone will be having Codex available as an agent. If you check our client-config for MCP as well as our Helm settings for agents, you'll see that we should be able to choose which CLI and which model to use with it.If you look at our images, you'll see which CLI images we currently have available.Before we get started, just let me review the docs, review our code, and then let me know if you have any outstanding questions. We'll proceed.

---

**Cursor**

Quick recap of the review:

- Architecture + tasks 4-12 outline the plan: Codex is the first non-Claude adapter, but the abstraction (factory, template system, config resolver, auth, transport, etc.) must support onboarding the rest.  
- Current codebase has the trait system, base adapter, factory, and Claude implementation merged (Task 3). Codex adapter slot exists but is still placeholder; no Codex templates/config generation yet.  
- Helm `agent.cliImages` already defines `codex`, and several GitHub apps are mapped to the Codex CLI; we just need the controller & MCP to honor those settings.  
- Images for the main CLIs (Claude/Codex/Opencode) are present under `infra/images/`.

Outstanding questions before I start drafting the implementation plan:

1. **Configuration merge precedence** – confirm we should do field-level overrides (Helm defaults merged with repo `cto-config.json`, plus runtime/env overrides), as proposed in the architecture doc.  
2. **Model validation timing** – ok to tackle the CLI-aware model validation immediately (as part of Task 4), or do you prefer it staged separately first?  
3. **Transport fallback** – if Codex’s STDIO stream proves too chatty, do you want me to wire in the buffered relay abstraction, or only log the findings and keep Toolman passthrough until a problem appears?  
4. **Timeout policy** – should we normalize controller-level timeouts across CLIs, or keep per-CLI defaults (and simply expose them for tuning)?  
5. **Rollout controls** – do you want us to add feature flags/env toggles so teams can opt in to Codex per agent/environment during the initial rollout?  
6. **Testing strategy** – any constraints on how aggressive we can be? (e.g., set up separate namespaces for Codex trials vs. shadow mode, or run a limited canary with one of the existing Codex-mapped GitHub apps?)

Once I’ve got your answers, I’ll spin up a feature branch and outline the work plan.

---

**User**

Yes, so it wouldn't be a merge, it would be a replacement, so the cto-config would override the helm.I think the model validation is just going to be an administrative burden because there are so many models and because it's changing so often. I'd rather not have to hardcode them in, so we should just assume that the model name will be input correctly.In terms of Toolman, yeah, we need to do a bit of research to find out what is supported. We already have our Toolman already supports standard I/O. I think that's a pre-optimization to Failback. Let's just see how it performs and we'll go from there.I think we should expect these to be long-running processes, so there should be no timeout.Yeah, there's no users. I'm the sole user, so we don't need to worry about rollout control.And yeah, same thing, we're not really worried about like a Canary deploy or anything like that. We should be quite aggressive.

---

**Cursor**

All set. Key takeaways from the review and your clarifications:

- `cto-config.json` should *replace* the Helm defaults (no merge); controller logic needs to treat repo-level overrides as authoritative.
- We’ll drop strict model-name validation—assume callers provide valid Codex/Claude models and avoid hard-coded allowlists.
- Toolman: keep the existing STDIO bridge, benchmark Codex as-is, and only introduce buffering/relay logic if a real bottleneck shows up.
- Treat Codex jobs as long-running with effectively no controller-level timeout.
- No rollout gates or canaries needed; we can wire Codex in aggressively since you’re the only user.

Next move: I’ll spin up a feature branch and outline the concrete implementation plan to get Codex adapter + supporting pieces in place while keeping the abstraction ready for future CLIs. Let me know if you want anything else reviewed before I start coding.

---

**User**

Let's fucking go.

---

**Cursor**

Let’s do this 🚀

### Codex-First Multi-CLI Plan  
- **Branch setup**: cut `feature/codex-cli-onboarding` from latest `main`.  
- **Factory & registration**: extend `AdapterFactory` to create/register `CodexAdapter`; enable JSON config replacement (Helm defaults overridden outright by repo `cto-config.json`).  
- **Codex adapter**: implement full `CliAdapter` for Codex—TOML config generation, prompt formatting, AGENTS.md memory handling, response parsing, health checks, and MCP/tool wiring—mirroring Claude’s structure but tailored to Codex semantics.  
- **Template layer**: add Codex templates (`codex-config.toml.hbs`, Codex system prompt, entrypoint script) and Hook them into template manager / chart configmap.  
- **Controller + Helm**: ensure CLI type from Helm/cto-config flows through resolver to adapter; update chart scripts to ship the new templates and Codex entrypoint.  
- **MCP server**: relax model name validation entirely, propagate CLI selection so Codex configs are accepted, and keep existing STDIO transport (instrument to observe performance). No timeouts applied.  
- **Tests & docs**: add controller/unit coverage for Codex adapter + factory, refresh architecture docs/workflow notes with the “replace-not-merge” rule and no-validation policy.

Next response will include command plan and start the branch unless you want adjustments first.

---

**User**

Yeah, so I have one question. I'm just looking at the changes from the previous, from task_3 I guess. Up until now we had all of our templates loaded at the infra level. I just noticed that we have this one here. Can it be with the other templates in infra? Does it have to be here?@claude-config.json.hbs I just would like to keep things nice and consistent with our current patterns.@client-config.json.hbs @claude.md.hbs 

---

**Cursor**

Good call—there’s nothing binding `claude-config.json.hbs` to `controller/src/cli/templates`. It only lives there because Task 3 bootstrapped the new adapter quickly with an embedded include. We can (and should) move it alongside the existing Helm-bundled assets under `infra/charts/controller/agent-templates/code/`. That keeps every CLI template in one place and lets the ConfigMap generator pick them up uniformly. When we implement Codex we’ll follow the same pattern: add the TOML/config + system-prompt + entrypoint templates in that infra directory, wire the chart script to ship them, and have the controller load them via the template manager rather than hardcoding paths in the crate.

---

**User**

OK, yeah, let's do that before we begin with the rest of the implementation.

---

**Cursor**

Moved `claude-config.json.hbs` into `infra/charts/controller/agent-templates/code/` so all adapter templates live under the Helm bundle. Clean diff staged in `feature/codex-cli-onboarding`; ready to continue with the Codex work.

---

**User**

Okay, let's fucking go let's do it. Keep going until you reach a blocker.

---

**Cursor**

Game plan:

- **Controller adapters**: add a real `CodexAdapter` alongside Claude—implement config generation (TOML), `AGENTS.md` memory, prompt formatting, response parsing, health checks, and register it in `AdapterFactory`. Relax model validation so we just trust the incoming string.
- **Template system**: add Codex-specific assets (`codex-config.toml.hbs`, system prompt, entrypoint script) under `infra/charts/controller/agent-templates/code/`, update the ConfigMap scripts, and make the controller load them via the template manager.
- **Config resolution**: adjust resolver/Helm integration so `cto-config.json` overrides *replace* Helm defaults rather than merge; ensure codex CLI settings flow end-to-end.
- **MCP server**: drop hard-coded model validation, make sure Codex CLI selection/config is accepted, stick with existing Toolman STDIO bridge (instrument for later tuning).
- **Testing/docs**: extend adapter unit tests + integration coverage, then update architecture/task docs to reflect the new flow.

Shout if you want the order tweaked; otherwise I’ll start executing those steps.

---

**User**

No, that sounds good. Also, just know that you have access to the CLI docs via our Toolman tools if you need to get more information on how each CLI works.

---

**Cursor**

Perfect—appreciate the heads-up on the Toolman docs. I’ll dig into them as needed while wiring up Codex. Starting the implementation now; I’ll surface anything blocking along the way.

---

**User**

Great, proceed please.

---

**Cursor**

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from("claude-config.json.hbs");

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from("claude-config.json.hbs");

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from("claude-config.json.hbs");

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

```rust
//! Base Adapter Implementation
//!
//! Provides shared functionality for all CLI adapters including logging, metrics collection,
//! template rendering, and common utilities.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, ContainerContext, HealthState, HealthStatus,
};
use crate::cli::types::CLIType;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output,
    RenderContext,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Operation context for tracing
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub cli_type: CLIType,
    pub correlation_id: String,
    pub start_time: Instant,
}

/// Configuration for BaseAdapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Root directory for CLI templates
    pub template_root: PathBuf,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Metrics prefix
    pub metrics_prefix: String,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl AdapterConfig {
    pub fn new(cli_type: CLIType) -> Self {
        let default_template_root = std::env::var("CLI_TEMPLATES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/agent-templates"));
        Self {
            cli_type,
            correlation_id: Uuid::new_v4().to_string(),
            template_root: default_template_root,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(10),
            metrics_prefix: format!("cli_adapter_{cli_type}"),
            verbose_logging: false,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    pub fn with_template_root<P: Into<PathBuf>>(mut self, template_root: P) -> Self {
        self.template_root = template_root.into();
        self
    }

    pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }
}

/// Base adapter providing shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// OpenTelemetry metrics
    pub metrics: Arc<AdapterMetrics>,
    /// Handlebars template engine
    pub templates: Arc<Handlebars<'static>>,
    /// OpenTelemetry tracer (simplified)
    pub tracer: Option<String>,
}

impl BaseAdapter {
    /// Create new base adapter
    pub async fn new(config: AdapterConfig) -> AdapterResult<Self> {
        let metrics = Arc::new(AdapterMetrics::new(&config.metrics_prefix)?);
        let mut templates = Handlebars::new();

        // Register custom helpers
        Self::register_template_helpers(&mut templates);

        let tracer = Some("cli_adapter".to_string());

        Ok(Self {
            cli_type: config.cli_type,
            config,
            metrics,
            templates: Arc::new(templates),
            tracer,
        })
    }

    /// Log structured operation with correlation ID and context
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        let mut log_context = context.clone();
        log_context.insert("cli_type".to_string(), self.cli_type.to_string());
        log_context.insert(
            "correlation_id".to_string(),
            self.config.correlation_id.clone(),
        );

        info!(
            operation = %operation,
            cli_type = %self.cli_type,
            correlation_id = %self.config.correlation_id,
            context = ?log_context,
            "CLI adapter operation"
        );
    }

    /// Record metrics with OpenTelemetry
    #[instrument(skip(self))]
    pub async fn record_metrics(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
    ) -> AdapterResult<()> {
        let labels = [
            KeyValue::new("cli_type", self.cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
            KeyValue::new("correlation_id", self.config.correlation_id.clone()),
        ];

        // Record operation count
        self.metrics.operations_total.add(1, &labels);

        // Record operation duration
        self.metrics
            .operation_duration
            .record(duration.as_millis() as f64, &labels);

        // Record failures
        if !success {
            self.metrics.operation_failures.add(1, &labels);
        }

        debug!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter metrics"
        );

        Ok(())
    }

    /// Validate base configuration common to all adapters
    #[instrument(skip(self, config))]
    pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
        // Validate required fields
        if config.github_app.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "GitHub app cannot be empty".to_string(),
            ));
        }

        if config.cli.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "CLI type cannot be empty".to_string(),
            ));
        }

        if config.model.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "Model cannot be empty".to_string(),
            ));
        }

        // Validate CLI type matches adapter
        let expected_cli = self.cli_type.to_string();
        if config.cli != expected_cli {
            return Err(AdapterError::ValidationError(format!(
                "CLI type mismatch: expected '{}', got '{}'",
                expected_cli, config.cli
            )));
        }

        // Validate optional fields
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 || max_tokens > 1_000_000 {
                return Err(AdapterError::ValidationError(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ValidationError(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        info!(
            github_app = %config.github_app,
            cli = %config.cli,
            model = %config.model,
            "Base configuration validation passed"
        );

        Ok(())
    }

    /// Render template with Handlebars and context
    #[instrument(skip(self, template, context))]
    pub fn render_template(&self, template: &str, context: &Value) -> AdapterResult<String> {
        let start_time = Instant::now();

        // Add base context
        let mut full_context = context.clone();
        if let Value::Object(ref mut map) = full_context {
            map.insert(
                "cli_type".to_string(),
                Value::String(self.cli_type.to_string()),
            );
            map.insert(
                "correlation_id".to_string(),
                Value::String(self.config.correlation_id.clone()),
            );
            map.insert(
                "timestamp".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let result = self
            .templates
            .render_template(template, &full_context)
            .map_err(|e| {
                error!(error = %e, "Template rendering failed");
                AdapterError::TemplateError(format!("Template rendering failed: {e}"))
            })?;

        let duration = start_time.elapsed();
        debug!(
            template_length = template.len(),
            result_length = result.len(),
            duration_ms = duration.as_millis(),
            "Template rendered successfully"
        );

        Ok(result)
    }

    /// Load a template from the configured template root
    pub fn load_template(&self, relative_path: &str) -> AdapterResult<String> {
        let path = self.config.template_root.join(relative_path);
        let content = fs::read_to_string(&path).map_err(|e| {
            AdapterError::TemplateError(format!(
                "Failed to load template {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(content)
    }

    /// Register custom template helpers
    fn register_template_helpers(hb: &mut Handlebars<'static>) {
        // Helper for JSON serialization
        handlebars_helper!(json: |obj: Value| {
            serde_json::to_string(&obj).unwrap_or_else(|_| "null".to_string())
        });

        // Helper for CLI-specific formatting
        handlebars_helper!(cli_format: |cli_type: str, content: str| {
            match cli_type {
                "claude" => format!("# Claude Configuration\n\n{content}"),
                "codex" => format!("# Codex Configuration\n{content}"),
                _ => content.to_string(),
            }
        });

        // Helper for timestamp formatting
        fn timestamp_helper(
            _: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let timestamp = chrono::Utc::now().to_rfc3339();
            out.write(&timestamp)?;
            Ok(())
        }

        // Helper for environment variable substitution
        fn env_helper(
            h: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let var_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

            let value = std::env::var(var_name).unwrap_or_else(|_| format!("${{{var_name}}}"));

            out.write(&value)?;
            Ok(())
        }

        // Helper for conditional CLI features
        handlebars_helper!(if_cli_supports: |cli_type: str, feature: str, then_val: Value, else_val: Value| {
            let supports = match (cli_type, feature) {
                ("claude", "streaming") => true,
                ("claude", "multimodal") => false,
                ("codex", "streaming") => false,
                ("codex", "toml_config") => true,
                ("gemini", "multimodal") => true,
                _ => false,
            };

            if supports { then_val } else { else_val }
        });

        hb.register_helper("json", Box::new(json));
        hb.register_helper("cli_format", Box::new(cli_format));
        hb.register_helper("timestamp", Box::new(timestamp_helper));
        hb.register_helper("env", Box::new(env_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports));
    }

    /// Create operation context for tracing
    #[instrument(skip(self))]
    pub fn create_operation_context(&self, operation: &str) -> OperationContext {
        OperationContext {
            operation: operation.to_string(),
            cli_type: self.cli_type,
            correlation_id: self.config.correlation_id.clone(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform common health checks
    #[instrument(skip(self, container))]
    pub async fn base_health_check(
        &self,
        container: &ContainerContext,
    ) -> AdapterResult<HealthStatus> {
        let _ctx = self.create_operation_context("health_check");
        let start_time = Instant::now();
        let mut health_details = HashMap::new();

        // Check adapter configuration
        health_details.insert(
            "config_valid".to_string(),
            json!(self.config.cli_type == self.cli_type),
        );

        // Check template engine
        let template_test = self.render_template("test: {{cli_type}}", &json!({}));
        health_details.insert(
            "templates_working".to_string(),
            json!(template_test.is_ok()),
        );

        // Check metrics collection
        health_details.insert(
            "metrics_available".to_string(),
            json!(true), // Always true since we initialize metrics
        );

        // Check container context if provided
        if let Some(pod) = &container.pod {
            health_details.insert(
                "pod_available".to_string(),
                json!(pod.metadata.name.is_some()),
            );
        }

        let duration = start_time.elapsed();
        health_details.insert("check_duration_ms".to_string(), json!(duration.as_millis()));

        // Determine overall health
        let is_healthy = template_test.is_ok() && duration < self.config.health_check_timeout;

        let status = if is_healthy {
            HealthState::Healthy
        } else {
            HealthState::Warning
        };

        let health = HealthStatus {
            status,
            message: if is_healthy {
                None
            } else {
                Some("Some health checks failed".to_string())
            },
            checked_at: chrono::Utc::now(),
            details: health_details,
        };

        // Operation completed

        info!(
            status = ?health.status,
            duration_ms = duration.as_millis(),
            "Base health check completed"
        );

        Ok(health)
    }

    /// Common initialization tasks
    #[instrument(skip(self, container))]
    pub async fn base_initialize(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("initialize");

        info!(
            container_name = %container.container_name,
            working_dir = %container.working_dir,
            namespace = %container.namespace,
            env_vars_count = container.env_vars.len(),
            "Initializing base adapter"
        );

        // Validate container context
        if container.container_name.is_empty() {
            // Container name validation failed
            return Err(AdapterError::InitializationError(
                "Container name cannot be empty".to_string(),
            ));
        }

        if container.working_dir.is_empty() {
            // Working directory validation failed
            return Err(AdapterError::InitializationError(
                "Working directory cannot be empty".to_string(),
            ));
        }

        // Log environment variables for debugging (without values for security)
        if self.config.verbose_logging {
            let env_keys: Vec<&String> = container.env_vars.keys().collect();
            debug!(env_keys = ?env_keys, "Container environment variables");
        }

        // Operation completed

        info!("Base adapter initialization completed successfully");
        Ok(())
    }

    /// Common cleanup tasks
    #[instrument(skip(self, container))]
    pub async fn base_cleanup(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("cleanup");

        info!(
            container_name = %container.container_name,
            "Starting base adapter cleanup"
        );

        // Flush any pending metrics
        // Note: OpenTelemetry metrics are typically handled by the SDK
        debug!("Flushed pending metrics");

        // Log cleanup completion
        info!("Base adapter cleanup completed");

        // Operation completed

        Ok(())
    }

    /// Get adapter configuration summary for diagnostics
    pub fn get_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert("cli_type".to_string(), json!(self.cli_type.to_string()));
        summary.insert(
            "correlation_id".to_string(),
            json!(self.config.correlation_id),
        );
        summary.insert(
            "template_cache_size".to_string(),
            json!(self.config.template_cache_size),
        );
        summary.insert(
            "health_check_timeout_ms".to_string(),
            json!(self.config.health_check_timeout.as_millis()),
        );
        summary.insert(
            "metrics_prefix".to_string(),
            json!(self.config.metrics_prefix),
        );
        summary.insert(
            "verbose_logging".to_string(),
            json!(self.config.verbose_logging),
        );

        summary
    }
}

/// OpenTelemetry metrics for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Counter for total operations
    pub operations_total: Counter<u64>,
    /// Counter for operation failures
    pub operation_failures: Counter<u64>,
    /// Histogram for operation duration
    pub operation_duration: Histogram<f64>,
}

impl AdapterMetrics {
    /// Create new metrics instance
    pub fn new(prefix: &str) -> AdapterResult<Self> {
        let meter = global::meter("cli_adapter");

        let operations_total = meter
            .u64_counter(format!("{prefix}_operations_total"))
            .with_description("Total number of adapter operations")
            .build();

        let operation_failures = meter
            .u64_counter(format!("{prefix}_operation_failures"))
            .with_description("Number of failed adapter operations")
            .build();

        let operation_duration = meter
            .f64_histogram(format!("{prefix}_operation_duration_ms"))
            .with_description("Duration of adapter operations in milliseconds")
            .build();

        Ok(Self {
            operations_total,
            operation_failures,
            operation_duration,
        })
    }

    /// Record operation metrics
    pub fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let labels = [
            KeyValue::new("cli_type", cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
        ];

        self.operations_total.add(1, &labels);
        self.operation_duration
            .record(duration.as_millis() as f64, &labels);

        if !success {
            self.operation_failures.add(1, &labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(!adapter.config.correlation_id.is_empty());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new(CLIType::Codex)
            .with_correlation_id("test-123".to_string())
            .with_verbose_logging(true);

        assert_eq!(config.cli_type, CLIType::Codex);
        assert_eq!(config.correlation_id, "test-123");
        assert!(config.verbose_logging);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let template = "Hello {{name}}, CLI: {{cli_type}}";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("CLI: claude"));
    }

    #[tokio::test]
    async fn test_base_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid config
        let valid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid config - empty model
        let invalid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let health = adapter.base_health_check(&container).await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[test]
    fn test_template_helpers() {
        let mut hb = Handlebars::new();
        BaseAdapter::register_template_helpers(&mut hb);

        // Test JSON helper
        let template = "{{json data}}";
        let context = json!({"data": {"key": "value"}});
        let result = hb.render_template(template, &context).unwrap();
        // JSON helper should output valid JSON (formatting may vary)
        assert!(result.contains("key") && result.contains("value"));

        // Test timestamp helper
        let template = "Generated at: {{timestamp}}";
        let result = hb.render_template(template, &json!({})).unwrap();
        assert!(result.contains("Generated at:"));
    }

    #[tokio::test]
    async fn test_config_summary() {
        let config =
            AdapterConfig::new(CLIType::Gemini).with_correlation_id("test-456".to_string());

        let adapter = BaseAdapter::new(config).await.unwrap();
        let summary = adapter.get_config_summary();

        assert_eq!(summary["cli_type"], json!("gemini"));
        assert_eq!(summary["correlation_id"], json!("test-456"));
        assert!(summary.contains_key("health_check_timeout_ms"));
    }

    #[tokio::test]
    async fn test_base_initialization_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid container context
        let valid_container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&valid_container).await.is_ok());

        // Invalid container context - empty name
        let invalid_container = ContainerContext {
            pod: None,
            container_name: "".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&invalid_container).await.is_err());
    }
}

```

```rust
*** Begin Patch
*** Update File: controller/src/cli/base_adapter.rs
@@
-        let config_template = Arc::from("claude-config.json.hbs");
+        let config_template = base
+            .load_template("code/claude/config.json.hbs")
+            .map(Arc::from)?;
*** End Patch
```

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from(
            base.load_template("code/claude/config.json.hbs").map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to load Claude config template: {e}",
                ))
            })?,
        );

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from(
            base.load_template("code/claude/config.json.hbs").map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to load Claude config template: {e}",
                ))
            })?,
        );

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from(
            base.load_template("code/claude/config.json.hbs").map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to load Claude config template: {e}",
                ))
            })?,
        );

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

```rust
//! Base Adapter Implementation
//!
//! Provides shared functionality for all CLI adapters including logging, metrics collection,
//! template rendering, and common utilities.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, ContainerContext, HealthState, HealthStatus,
};
use crate::cli::types::CLIType;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output,
    RenderContext,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Operation context for tracing
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub cli_type: CLIType,
    pub correlation_id: String,
    pub start_time: Instant,
}

/// Configuration for BaseAdapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Root directory for CLI templates
    pub template_root: PathBuf,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Metrics prefix
    pub metrics_prefix: String,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl AdapterConfig {
    pub fn new(cli_type: CLIType) -> Self {
        let default_template_root = std::env::var("CLI_TEMPLATES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/agent-templates"));
        Self {
            cli_type,
            correlation_id: Uuid::new_v4().to_string(),
            template_root: default_template_root,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(10),
            metrics_prefix: format!("cli_adapter_{cli_type}"),
            verbose_logging: false,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    pub fn with_template_root<P: Into<PathBuf>>(mut self, template_root: P) -> Self {
        self.template_root = template_root.into();
        self
    }

    pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }
}

/// Base adapter providing shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// OpenTelemetry metrics
    pub metrics: Arc<AdapterMetrics>,
    /// Handlebars template engine
    pub templates: Arc<Handlebars<'static>>,
    /// OpenTelemetry tracer (simplified)
    pub tracer: Option<String>,
}

impl BaseAdapter {
    /// Create new base adapter
    pub async fn new(config: AdapterConfig) -> AdapterResult<Self> {
        let metrics = Arc::new(AdapterMetrics::new(&config.metrics_prefix)?);
        let mut templates = Handlebars::new();

        // Register custom helpers
        Self::register_template_helpers(&mut templates);

        let tracer = Some("cli_adapter".to_string());

        Ok(Self {
            cli_type: config.cli_type,
            config,
            metrics,
            templates: Arc::new(templates),
            tracer,
        })
    }

    /// Log structured operation with correlation ID and context
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        let mut log_context = context.clone();
        log_context.insert("cli_type".to_string(), self.cli_type.to_string());
        log_context.insert(
            "correlation_id".to_string(),
            self.config.correlation_id.clone(),
        );

        info!(
            operation = %operation,
            cli_type = %self.cli_type,
            correlation_id = %self.config.correlation_id,
            context = ?log_context,
            "CLI adapter operation"
        );
    }

    /// Record metrics with OpenTelemetry
    #[instrument(skip(self))]
    pub async fn record_metrics(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
    ) -> AdapterResult<()> {
        let labels = [
            KeyValue::new("cli_type", self.cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
            KeyValue::new("correlation_id", self.config.correlation_id.clone()),
        ];

        // Record operation count
        self.metrics.operations_total.add(1, &labels);

        // Record operation duration
        self.metrics
            .operation_duration
            .record(duration.as_millis() as f64, &labels);

        // Record failures
        if !success {
            self.metrics.operation_failures.add(1, &labels);
        }

        debug!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter metrics"
        );

        Ok(())
    }

    /// Validate base configuration common to all adapters
    #[instrument(skip(self, config))]
    pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
        // Validate required fields
        if config.github_app.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "GitHub app cannot be empty".to_string(),
            ));
        }

        if config.cli.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "CLI type cannot be empty".to_string(),
            ));
        }

        if config.model.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "Model cannot be empty".to_string(),
            ));
        }

        // Validate CLI type matches adapter
        let expected_cli = self.cli_type.to_string();
        if config.cli != expected_cli {
            return Err(AdapterError::ValidationError(format!(
                "CLI type mismatch: expected '{}', got '{}'",
                expected_cli, config.cli
            )));
        }

        // Validate optional fields
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 || max_tokens > 1_000_000 {
                return Err(AdapterError::ValidationError(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ValidationError(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        info!(
            github_app = %config.github_app,
            cli = %config.cli,
            model = %config.model,
            "Base configuration validation passed"
        );

        Ok(())
    }

    /// Render template with Handlebars and context
    #[instrument(skip(self, template, context))]
    pub fn render_template(&self, template: &str, context: &Value) -> AdapterResult<String> {
        let start_time = Instant::now();

        // Add base context
        let mut full_context = context.clone();
        if let Value::Object(ref mut map) = full_context {
            map.insert(
                "cli_type".to_string(),
                Value::String(self.cli_type.to_string()),
            );
            map.insert(
                "correlation_id".to_string(),
                Value::String(self.config.correlation_id.clone()),
            );
            map.insert(
                "timestamp".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let result = self
            .templates
            .render_template(template, &full_context)
            .map_err(|e| {
                error!(error = %e, "Template rendering failed");
                AdapterError::TemplateError(format!("Template rendering failed: {e}"))
            })?;

        let duration = start_time.elapsed();
        debug!(
            template_length = template.len(),
            result_length = result.len(),
            duration_ms = duration.as_millis(),
            "Template rendered successfully"
        );

        Ok(result)
    }

    /// Load a template from the configured template root
    pub fn load_template(&self, relative_path: &str) -> AdapterResult<String> {
        let path = self.config.template_root.join(relative_path);
        let content = fs::read_to_string(&path).map_err(|e| {
            AdapterError::TemplateError(format!(
                "Failed to load template {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(content)
    }

    /// Register custom template helpers
    fn register_template_helpers(hb: &mut Handlebars<'static>) {
        // Helper for JSON serialization
        handlebars_helper!(json: |obj: Value| {
            serde_json::to_string(&obj).unwrap_or_else(|_| "null".to_string())
        });

        // Helper for CLI-specific formatting
        handlebars_helper!(cli_format: |cli_type: str, content: str| {
            match cli_type {
                "claude" => format!("# Claude Configuration\n\n{content}"),
                "codex" => format!("# Codex Configuration\n{content}"),
                _ => content.to_string(),
            }
        });

        // Helper for timestamp formatting
        fn timestamp_helper(
            _: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let timestamp = chrono::Utc::now().to_rfc3339();
            out.write(&timestamp)?;
            Ok(())
        }

        // Helper for environment variable substitution
        fn env_helper(
            h: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let var_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

            let value = std::env::var(var_name).unwrap_or_else(|_| format!("${{{var_name}}}"));

            out.write(&value)?;
            Ok(())
        }

        // Helper for conditional CLI features
        handlebars_helper!(if_cli_supports: |cli_type: str, feature: str, then_val: Value, else_val: Value| {
            let supports = match (cli_type, feature) {
                ("claude", "streaming") => true,
                ("claude", "multimodal") => false,
                ("codex", "streaming") => false,
                ("codex", "toml_config") => true,
                ("gemini", "multimodal") => true,
                _ => false,
            };

            if supports { then_val } else { else_val }
        });

        hb.register_helper("json", Box::new(json));
        hb.register_helper("cli_format", Box::new(cli_format));
        hb.register_helper("timestamp", Box::new(timestamp_helper));
        hb.register_helper("env", Box::new(env_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports));
    }

    /// Create operation context for tracing
    #[instrument(skip(self))]
    pub fn create_operation_context(&self, operation: &str) -> OperationContext {
        OperationContext {
            operation: operation.to_string(),
            cli_type: self.cli_type,
            correlation_id: self.config.correlation_id.clone(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform common health checks
    #[instrument(skip(self, container))]
    pub async fn base_health_check(
        &self,
        container: &ContainerContext,
    ) -> AdapterResult<HealthStatus> {
        let _ctx = self.create_operation_context("health_check");
        let start_time = Instant::now();
        let mut health_details = HashMap::new();

        // Check adapter configuration
        health_details.insert(
            "config_valid".to_string(),
            json!(self.config.cli_type == self.cli_type),
        );

        // Check template engine
        let template_test = self.render_template("test: {{cli_type}}", &json!({}));
        health_details.insert(
            "templates_working".to_string(),
            json!(template_test.is_ok()),
        );

        // Check metrics collection
        health_details.insert(
            "metrics_available".to_string(),
            json!(true), // Always true since we initialize metrics
        );

        // Check container context if provided
        if let Some(pod) = &container.pod {
            health_details.insert(
                "pod_available".to_string(),
                json!(pod.metadata.name.is_some()),
            );
        }

        let duration = start_time.elapsed();
        health_details.insert("check_duration_ms".to_string(), json!(duration.as_millis()));

        // Determine overall health
        let is_healthy = template_test.is_ok() && duration < self.config.health_check_timeout;

        let status = if is_healthy {
            HealthState::Healthy
        } else {
            HealthState::Warning
        };

        let health = HealthStatus {
            status,
            message: if is_healthy {
                None
            } else {
                Some("Some health checks failed".to_string())
            },
            checked_at: chrono::Utc::now(),
            details: health_details,
        };

        // Operation completed

        info!(
            status = ?health.status,
            duration_ms = duration.as_millis(),
            "Base health check completed"
        );

        Ok(health)
    }

    /// Common initialization tasks
    #[instrument(skip(self, container))]
    pub async fn base_initialize(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("initialize");

        info!(
            container_name = %container.container_name,
            working_dir = %container.working_dir,
            namespace = %container.namespace,
            env_vars_count = container.env_vars.len(),
            "Initializing base adapter"
        );

        // Validate container context
        if container.container_name.is_empty() {
            // Container name validation failed
            return Err(AdapterError::InitializationError(
                "Container name cannot be empty".to_string(),
            ));
        }

        if container.working_dir.is_empty() {
            // Working directory validation failed
            return Err(AdapterError::InitializationError(
                "Working directory cannot be empty".to_string(),
            ));
        }

        // Log environment variables for debugging (without values for security)
        if self.config.verbose_logging {
            let env_keys: Vec<&String> = container.env_vars.keys().collect();
            debug!(env_keys = ?env_keys, "Container environment variables");
        }

        // Operation completed

        info!("Base adapter initialization completed successfully");
        Ok(())
    }

    /// Common cleanup tasks
    #[instrument(skip(self, container))]
    pub async fn base_cleanup(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("cleanup");

        info!(
            container_name = %container.container_name,
            "Starting base adapter cleanup"
        );

        // Flush any pending metrics
        // Note: OpenTelemetry metrics are typically handled by the SDK
        debug!("Flushed pending metrics");

        // Log cleanup completion
        info!("Base adapter cleanup completed");

        // Operation completed

        Ok(())
    }

    /// Get adapter configuration summary for diagnostics
    pub fn get_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert("cli_type".to_string(), json!(self.cli_type.to_string()));
        summary.insert(
            "correlation_id".to_string(),
            json!(self.config.correlation_id),
        );
        summary.insert(
            "template_cache_size".to_string(),
            json!(self.config.template_cache_size),
        );
        summary.insert(
            "health_check_timeout_ms".to_string(),
            json!(self.config.health_check_timeout.as_millis()),
        );
        summary.insert(
            "metrics_prefix".to_string(),
            json!(self.config.metrics_prefix),
        );
        summary.insert(
            "verbose_logging".to_string(),
            json!(self.config.verbose_logging),
        );

        summary
    }
}

/// OpenTelemetry metrics for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Counter for total operations
    pub operations_total: Counter<u64>,
    /// Counter for operation failures
    pub operation_failures: Counter<u64>,
    /// Histogram for operation duration
    pub operation_duration: Histogram<f64>,
}

impl AdapterMetrics {
    /// Create new metrics instance
    pub fn new(prefix: &str) -> AdapterResult<Self> {
        let meter = global::meter("cli_adapter");

        let operations_total = meter
            .u64_counter(format!("{prefix}_operations_total"))
            .with_description("Total number of adapter operations")
            .build();

        let operation_failures = meter
            .u64_counter(format!("{prefix}_operation_failures"))
            .with_description("Number of failed adapter operations")
            .build();

        let operation_duration = meter
            .f64_histogram(format!("{prefix}_operation_duration_ms"))
            .with_description("Duration of adapter operations in milliseconds")
            .build();

        Ok(Self {
            operations_total,
            operation_failures,
            operation_duration,
        })
    }

    /// Record operation metrics
    pub fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let labels = [
            KeyValue::new("cli_type", cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
        ];

        self.operations_total.add(1, &labels);
        self.operation_duration
            .record(duration.as_millis() as f64, &labels);

        if !success {
            self.operation_failures.add(1, &labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(!adapter.config.correlation_id.is_empty());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new(CLIType::Codex)
            .with_correlation_id("test-123".to_string())
            .with_verbose_logging(true);

        assert_eq!(config.cli_type, CLIType::Codex);
        assert_eq!(config.correlation_id, "test-123");
        assert!(config.verbose_logging);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let template = "Hello {{name}}, CLI: {{cli_type}}";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("CLI: claude"));
    }

    #[tokio::test]
    async fn test_base_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid config
        let valid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid config - empty model
        let invalid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let health = adapter.base_health_check(&container).await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[test]
    fn test_template_helpers() {
        let mut hb = Handlebars::new();
        BaseAdapter::register_template_helpers(&mut hb);

        // Test JSON helper
        let template = "{{json data}}";
        let context = json!({"data": {"key": "value"}});
        let result = hb.render_template(template, &context).unwrap();
        // JSON helper should output valid JSON (formatting may vary)
        assert!(result.contains("key") && result.contains("value"));

        // Test timestamp helper
        let template = "Generated at: {{timestamp}}";
        let result = hb.render_template(template, &json!({})).unwrap();
        assert!(result.contains("Generated at:"));
    }

    #[tokio::test]
    async fn test_config_summary() {
        let config =
            AdapterConfig::new(CLIType::Gemini).with_correlation_id("test-456".to_string());

        let adapter = BaseAdapter::new(config).await.unwrap();
        let summary = adapter.get_config_summary();

        assert_eq!(summary["cli_type"], json!("gemini"));
        assert_eq!(summary["correlation_id"], json!("test-456"));
        assert!(summary.contains_key("health_check_timeout_ms"));
    }

    #[tokio::test]
    async fn test_base_initialization_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid container context
        let valid_container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&valid_container).await.is_ok());

        // Invalid container context - empty name
        let invalid_container = ContainerContext {
            pod: None,
            container_name: "".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&invalid_container).await.is_err());
    }
}

```

```rust
*** Begin Patch
*** Update File: controller/src/cli/base_adapter.rs
@@
-        let template = self.config.template_root.join(relative_path);
-        let content = fs::read_to_string(&template).map_err(|e| {
+        let template = self.config.template_root.join(relative_path);
+        let content = fs::read_to_string(&template).map_err(|e| {
             AdapterError::TemplateError(format!(
                 "Failed to load template {}: {}",
                 template.display(),
                 e
             ))
*** End Patch
```

```rust
//! Base Adapter Implementation
//!
//! Provides shared functionality for all CLI adapters including logging, metrics collection,
//! template rendering, and common utilities.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, ContainerContext, HealthState, HealthStatus,
};
use crate::cli::types::CLIType;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output,
    RenderContext,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Operation context for tracing
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub cli_type: CLIType,
    pub correlation_id: String,
    pub start_time: Instant,
}

/// Configuration for BaseAdapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Root directory for CLI templates
    pub template_root: PathBuf,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Metrics prefix
    pub metrics_prefix: String,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl AdapterConfig {
    pub fn new(cli_type: CLIType) -> Self {
        let default_template_root = std::env::var("CLI_TEMPLATES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/agent-templates"));
        Self {
            cli_type,
            correlation_id: Uuid::new_v4().to_string(),
            template_root: default_template_root,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(10),
            metrics_prefix: format!("cli_adapter_{cli_type}"),
            verbose_logging: false,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    pub fn with_template_root<P: Into<PathBuf>>(mut self, template_root: P) -> Self {
        self.template_root = template_root.into();
        self
    }

    pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }
}

/// Base adapter providing shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// OpenTelemetry metrics
    pub metrics: Arc<AdapterMetrics>,
    /// Handlebars template engine
    pub templates: Arc<Handlebars<'static>>,
    /// OpenTelemetry tracer (simplified)
    pub tracer: Option<String>,
}

impl BaseAdapter {
    /// Create new base adapter
    pub async fn new(config: AdapterConfig) -> AdapterResult<Self> {
        let metrics = Arc::new(AdapterMetrics::new(&config.metrics_prefix)?);
        let mut templates = Handlebars::new();

        // Register custom helpers
        Self::register_template_helpers(&mut templates);

        let tracer = Some("cli_adapter".to_string());

        Ok(Self {
            cli_type: config.cli_type,
            config,
            metrics,
            templates: Arc::new(templates),
            tracer,
        })
    }

    /// Log structured operation with correlation ID and context
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        let mut log_context = context.clone();
        log_context.insert("cli_type".to_string(), self.cli_type.to_string());
        log_context.insert(
            "correlation_id".to_string(),
            self.config.correlation_id.clone(),
        );

        info!(
            operation = %operation,
            cli_type = %self.cli_type,
            correlation_id = %self.config.correlation_id,
            context = ?log_context,
            "CLI adapter operation"
        );
    }

    /// Record metrics with OpenTelemetry
    #[instrument(skip(self))]
    pub async fn record_metrics(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
    ) -> AdapterResult<()> {
        let labels = [
            KeyValue::new("cli_type", self.cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
            KeyValue::new("correlation_id", self.config.correlation_id.clone()),
        ];

        // Record operation count
        self.metrics.operations_total.add(1, &labels);

        // Record operation duration
        self.metrics
            .operation_duration
            .record(duration.as_millis() as f64, &labels);

        // Record failures
        if !success {
            self.metrics.operation_failures.add(1, &labels);
        }

        debug!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter metrics"
        );

        Ok(())
    }

    /// Validate base configuration common to all adapters
    #[instrument(skip(self, config))]
    pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
        // Validate required fields
        if config.github_app.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "GitHub app cannot be empty".to_string(),
            ));
        }

        if config.cli.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "CLI type cannot be empty".to_string(),
            ));
        }

        if config.model.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "Model cannot be empty".to_string(),
            ));
        }

        // Validate CLI type matches adapter
        let expected_cli = self.cli_type.to_string();
        if config.cli != expected_cli {
            return Err(AdapterError::ValidationError(format!(
                "CLI type mismatch: expected '{}', got '{}'",
                expected_cli, config.cli
            )));
        }

        // Validate optional fields
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 || max_tokens > 1_000_000 {
                return Err(AdapterError::ValidationError(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ValidationError(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        info!(
            github_app = %config.github_app,
            cli = %config.cli,
            model = %config.model,
            "Base configuration validation passed"
        );

        Ok(())
    }

    /// Render template with Handlebars and context
    #[instrument(skip(self, template, context))]
    pub fn render_template(&self, template: &str, context: &Value) -> AdapterResult<String> {
        let start_time = Instant::now();

        // Add base context
        let mut full_context = context.clone();
        if let Value::Object(ref mut map) = full_context {
            map.insert(
                "cli_type".to_string(),
                Value::String(self.cli_type.to_string()),
            );
            map.insert(
                "correlation_id".to_string(),
                Value::String(self.config.correlation_id.clone()),
            );
            map.insert(
                "timestamp".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let result = self
            .templates
            .render_template(template, &full_context)
            .map_err(|e| {
                error!(error = %e, "Template rendering failed");
                AdapterError::TemplateError(format!("Template rendering failed: {e}"))
            })?;

        let duration = start_time.elapsed();
        debug!(
            template_length = template.len(),
            result_length = result.len(),
            duration_ms = duration.as_millis(),
            "Template rendered successfully"
        );

        Ok(result)
    }

    /// Load a template from the configured template root
    pub fn load_template(&self, relative_path: &str) -> AdapterResult<String> {
        let path = self.config.template_root.join(relative_path);
        let content = fs::read_to_string(&path).map_err(|e| {
            AdapterError::TemplateError(format!(
                "Failed to load template {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(content)
    }

    /// Register custom template helpers
    fn register_template_helpers(hb: &mut Handlebars<'static>) {
        // Helper for JSON serialization
        handlebars_helper!(json: |obj: Value| {
            serde_json::to_string(&obj).unwrap_or_else(|_| "null".to_string())
        });

        // Helper for CLI-specific formatting
        handlebars_helper!(cli_format: |cli_type: str, content: str| {
            match cli_type {
                "claude" => format!("# Claude Configuration\n\n{content}"),
                "codex" => format!("# Codex Configuration\n{content}"),
                _ => content.to_string(),
            }
        });

        // Helper for timestamp formatting
        fn timestamp_helper(
            _: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let timestamp = chrono::Utc::now().to_rfc3339();
            out.write(&timestamp)?;
            Ok(())
        }

        // Helper for environment variable substitution
        fn env_helper(
            h: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let var_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

            let value = std::env::var(var_name).unwrap_or_else(|_| format!("${{{var_name}}}"));

            out.write(&value)?;
            Ok(())
        }

        // Helper for conditional CLI features
        handlebars_helper!(if_cli_supports: |cli_type: str, feature: str, then_val: Value, else_val: Value| {
            let supports = match (cli_type, feature) {
                ("claude", "streaming") => true,
                ("claude", "multimodal") => false,
                ("codex", "streaming") => false,
                ("codex", "toml_config") => true,
                ("gemini", "multimodal") => true,
                _ => false,
            };

            if supports { then_val } else { else_val }
        });

        hb.register_helper("json", Box::new(json));
        hb.register_helper("cli_format", Box::new(cli_format));
        hb.register_helper("timestamp", Box::new(timestamp_helper));
        hb.register_helper("env", Box::new(env_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports));
    }

    /// Create operation context for tracing
    #[instrument(skip(self))]
    pub fn create_operation_context(&self, operation: &str) -> OperationContext {
        OperationContext {
            operation: operation.to_string(),
            cli_type: self.cli_type,
            correlation_id: self.config.correlation_id.clone(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform common health checks
    #[instrument(skip(self, container))]
    pub async fn base_health_check(
        &self,
        container: &ContainerContext,
    ) -> AdapterResult<HealthStatus> {
        let _ctx = self.create_operation_context("health_check");
        let start_time = Instant::now();
        let mut health_details = HashMap::new();

        // Check adapter configuration
        health_details.insert(
            "config_valid".to_string(),
            json!(self.config.cli_type == self.cli_type),
        );

        // Check template engine
        let template_test = self.render_template("test: {{cli_type}}", &json!({}));
        health_details.insert(
            "templates_working".to_string(),
            json!(template_test.is_ok()),
        );

        // Check metrics collection
        health_details.insert(
            "metrics_available".to_string(),
            json!(true), // Always true since we initialize metrics
        );

        // Check container context if provided
        if let Some(pod) = &container.pod {
            health_details.insert(
                "pod_available".to_string(),
                json!(pod.metadata.name.is_some()),
            );
        }

        let duration = start_time.elapsed();
        health_details.insert("check_duration_ms".to_string(), json!(duration.as_millis()));

        // Determine overall health
        let is_healthy = template_test.is_ok() && duration < self.config.health_check_timeout;

        let status = if is_healthy {
            HealthState::Healthy
        } else {
            HealthState::Warning
        };

        let health = HealthStatus {
            status,
            message: if is_healthy {
                None
            } else {
                Some("Some health checks failed".to_string())
            },
            checked_at: chrono::Utc::now(),
            details: health_details,
        };

        // Operation completed

        info!(
            status = ?health.status,
            duration_ms = duration.as_millis(),
            "Base health check completed"
        );

        Ok(health)
    }

    /// Common initialization tasks
    #[instrument(skip(self, container))]
    pub async fn base_initialize(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("initialize");

        info!(
            container_name = %container.container_name,
            working_dir = %container.working_dir,
            namespace = %container.namespace,
            env_vars_count = container.env_vars.len(),
            "Initializing base adapter"
        );

        // Validate container context
        if container.container_name.is_empty() {
            // Container name validation failed
            return Err(AdapterError::InitializationError(
                "Container name cannot be empty".to_string(),
            ));
        }

        if container.working_dir.is_empty() {
            // Working directory validation failed
            return Err(AdapterError::InitializationError(
                "Working directory cannot be empty".to_string(),
            ));
        }

        // Log environment variables for debugging (without values for security)
        if self.config.verbose_logging {
            let env_keys: Vec<&String> = container.env_vars.keys().collect();
            debug!(env_keys = ?env_keys, "Container environment variables");
        }

        // Operation completed

        info!("Base adapter initialization completed successfully");
        Ok(())
    }

    /// Common cleanup tasks
    #[instrument(skip(self, container))]
    pub async fn base_cleanup(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("cleanup");

        info!(
            container_name = %container.container_name,
            "Starting base adapter cleanup"
        );

        // Flush any pending metrics
        // Note: OpenTelemetry metrics are typically handled by the SDK
        debug!("Flushed pending metrics");

        // Log cleanup completion
        info!("Base adapter cleanup completed");

        // Operation completed

        Ok(())
    }

    /// Get adapter configuration summary for diagnostics
    pub fn get_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert("cli_type".to_string(), json!(self.cli_type.to_string()));
        summary.insert(
            "correlation_id".to_string(),
            json!(self.config.correlation_id),
        );
        summary.insert(
            "template_cache_size".to_string(),
            json!(self.config.template_cache_size),
        );
        summary.insert(
            "health_check_timeout_ms".to_string(),
            json!(self.config.health_check_timeout.as_millis()),
        );
        summary.insert(
            "metrics_prefix".to_string(),
            json!(self.config.metrics_prefix),
        );
        summary.insert(
            "verbose_logging".to_string(),
            json!(self.config.verbose_logging),
        );

        summary
    }
}

/// OpenTelemetry metrics for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Counter for total operations
    pub operations_total: Counter<u64>,
    /// Counter for operation failures
    pub operation_failures: Counter<u64>,
    /// Histogram for operation duration
    pub operation_duration: Histogram<f64>,
}

impl AdapterMetrics {
    /// Create new metrics instance
    pub fn new(prefix: &str) -> AdapterResult<Self> {
        let meter = global::meter("cli_adapter");

        let operations_total = meter
            .u64_counter(format!("{prefix}_operations_total"))
            .with_description("Total number of adapter operations")
            .build();

        let operation_failures = meter
            .u64_counter(format!("{prefix}_operation_failures"))
            .with_description("Number of failed adapter operations")
            .build();

        let operation_duration = meter
            .f64_histogram(format!("{prefix}_operation_duration_ms"))
            .with_description("Duration of adapter operations in milliseconds")
            .build();

        Ok(Self {
            operations_total,
            operation_failures,
            operation_duration,
        })
    }

    /// Record operation metrics
    pub fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let labels = [
            KeyValue::new("cli_type", cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
        ];

        self.operations_total.add(1, &labels);
        self.operation_duration
            .record(duration.as_millis() as f64, &labels);

        if !success {
            self.operation_failures.add(1, &labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(!adapter.config.correlation_id.is_empty());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new(CLIType::Codex)
            .with_correlation_id("test-123".to_string())
            .with_verbose_logging(true);

        assert_eq!(config.cli_type, CLIType::Codex);
        assert_eq!(config.correlation_id, "test-123");
        assert!(config.verbose_logging);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let template = "Hello {{name}}, CLI: {{cli_type}}";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("CLI: claude"));
    }

    #[tokio::test]
    async fn test_base_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid config
        let valid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid config - empty model
        let invalid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let health = adapter.base_health_check(&container).await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[test]
    fn test_template_helpers() {
        let mut hb = Handlebars::new();
        BaseAdapter::register_template_helpers(&mut hb);

        // Test JSON helper
        let template = "{{json data}}";
        let context = json!({"data": {"key": "value"}});
        let result = hb.render_template(template, &context).unwrap();
        // JSON helper should output valid JSON (formatting may vary)
        assert!(result.contains("key") && result.contains("value"));

        // Test timestamp helper
        let template = "Generated at: {{timestamp}}";
        let result = hb.render_template(template, &json!({})).unwrap();
        assert!(result.contains("Generated at:"));
    }

    #[tokio::test]
    async fn test_config_summary() {
        let config =
            AdapterConfig::new(CLIType::Gemini).with_correlation_id("test-456".to_string());

        let adapter = BaseAdapter::new(config).await.unwrap();
        let summary = adapter.get_config_summary();

        assert_eq!(summary["cli_type"], json!("gemini"));
        assert_eq!(summary["correlation_id"], json!("test-456"));
        assert!(summary.contains_key("health_check_timeout_ms"));
    }

    #[tokio::test]
    async fn test_base_initialization_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid container context
        let valid_container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&valid_container).await.is_ok());

        // Invalid container context - empty name
        let invalid_container = ContainerContext {
            pod: None,
            container_name: "".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&invalid_container).await.is_err());
    }
}

```

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from(
            base.load_template("code/claude/config.json.hbs").map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to load Claude config template: {e}",
                ))
            })?,
        );

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers)
            for tool_name in &tool_config.remote {
                // Map known tool names to MCP server configurations
                let server_config = match tool_name.as_str() {
                    "memory_create_entities" | "rustdocs_query_rust_docs" => json!({
                        "command": "toolman",
                        "args": ["--url", toolman_url.clone()],
                        "env": {
                            "TOOLMAN_SERVER_URL": toolman_url.clone()
                        }
                    }),
                    _ => {
                        debug!(tool_name = %tool_name, "Unknown remote tool, using default MCP config");
                        json!({
                            "command": "toolman",
                            "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                            "env": {
                                "TOOLMAN_SERVER_URL": toolman_url.clone()
                            }
                        })
                    }
                };

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
            if let Some(local_servers) = &tool_config.local_servers {
                for (server_name, server_config) in local_servers {
                    if server_config.enabled {
                        let local_server_config = match server_name.as_str() {
                            "filesystem" => json!({
                                "command": "mcp-server-filesystem",
                                "args": ["/workspace"],
                                "env": {}
                            }),
                            "git" => json!({
                                "command": "mcp-server-git",
                                "args": ["--repository", "/workspace"],
                                "env": {}
                            }),
                            _ => json!({
                                "command": format!("mcp-server-{}", server_name),
                                "args": [],
                                "env": {}
                            }),
                        };

                        mcp_servers[server_name] = local_server_config;
                    }
                }
            }
        }

        mcp_servers
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

Templates are now centralized:

- `controller/src/cli/templates/claude-config.json.hbs` moved to `infra/charts/controller/agent-templates/code/claude/config.json.hbs`.
- `ClaudeAdapter` + `BaseAdapter` now load templates from the mounted ConfigMap (default root `/agent-templates` set via new `template_root` config).
- Template reference in `ClaudeAdapter` updated to read the file from `code/claude/config.json.hbs`.

Everything is staged on `feature/codex-cli-onboarding`, and we’re ready to push forward with the Codex implementation.

---

**User**

OK, so before we do that, I just want to review some of the Claude code. I'm just reviewing some of the Claude code, and I'm just wanting to make sure that we're not duplicating any effort in terms of the existing implementation. Or, since we're moving to an agnostic model, maybe we're making some changes that way, but for example, like we don't need to know about there's only one MCP server and that's Toolman. Looks like we're failing back to the hardcoded URL, which is OK. So as long as we can still set it with the environmental variable, which it looks like we can. However I'm looking here and it looks like we have hard coded tools like memory and get an file system we do not want to do that. Tools are defined only at the helm and in the helm values or the @cto-config.json so let's just fix that first before we move on.

---

**Cursor**

```rust
//! Claude CLI Adapter Implementation
//!
//! Reference implementation of the `CliAdapter` trait for Claude Code CLI.
//! Maintains 100% backward compatibility with existing Claude functionality.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, ToolCall, ToolConfiguration,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
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
    /// Base adapter functionality
    base: Arc<BaseAdapter>,
    /// Claude-specific model validator
    model_validator: Arc<ClaudeModelValidator>,
    /// Claude configuration template
    config_template: Arc<str>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter using default configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize.
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::Claude)).await
    }

    /// Create a new Claude adapter with custom configuration.
    ///
    /// # Errors
    /// Returns an [`AdapterError`] if the adapter fails to initialize with the provided config.
    pub async fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing Claude adapter");

        let base = Arc::new(BaseAdapter::new(config).await?);
        let model_validator = Arc::new(ClaudeModelValidator::new());
        let config_template = Arc::from(
            base.load_template("code/claude/config.json.hbs").map_err(|e| {
                AdapterError::TemplateError(format!(
                    "Failed to load Claude config template: {e}",
                ))
            })?,
        );

        let adapter = Self {
            base,
            model_validator,
            config_template,
        };

        info!("Claude adapter initialized successfully");
        Ok(adapter)
    }

    /// Generate MCP configuration for Claude.
    #[must_use]
    fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
        let mut mcp_servers = json!({});

        // Get MCP server URL from environment variables (configurable, not hardcoded)
        let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
        });

        if let Some(tool_config) = tools {
            // Add remote tools (MCP servers) using a uniform Toolman invocation
            for tool_name in &tool_config.remote {
                let server_config = json!({
                    "command": "toolman",
                    "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                    "env": {
                        "TOOLMAN_SERVER_URL": toolman_url.clone()
                    }
                });

                mcp_servers[tool_name] = server_config;
            }

            // Add local servers
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
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating Claude model");

        let context = HashMap::from([("model".to_string(), model.to_string())]);
        self.base.log_operation("validate_model", &context);

        let is_valid = self.model_validator.validate(model).await?;

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

        // Validate configuration first
        self.base.validate_base_config(agent_config)?;

        // Generate MCP configuration
        let mcp_servers = Self::generate_mcp_config(agent_config.tools.as_ref());

        // Get default values from environment variables (configurable, not hardcoded)
        let default_max_tokens = std::env::var("CLAUDE_DEFAULT_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let default_temperature = std::env::var("CLAUDE_DEFAULT_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        // Prepare template context
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
        });

        // Render configuration template
        let config = self
            .base
            .render_template(&self.config_template, &context)
            .map_err(|e| {
                error!(error = %e, "Failed to render Claude configuration template");
                e
            })?;

        info!(
            config_length = config.len(),
            "Claude configuration generated successfully"
        );

        Ok(config)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain exact compatibility with existing Claude behavior
        format!("Human: {prompt}\n\nAssistant: ")
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing Claude response");

        let context = HashMap::from([("response_length".to_string(), response.len().to_string())]);
        self.base.log_operation("parse_response", &context);

        // Parse tool calls from Claude response
        let tool_calls = self.extract_tool_calls(response).await?;

        // Determine finish reason
        let finish_reason = if tool_calls.is_empty() {
            FinishReason::Stop
        } else {
            FinishReason::ToolCall
        };

        // Extract metadata if available
        let metadata = self.extract_response_metadata(response).await;

        let parsed_response = ParsedResponse {
            content: response.to_string(),
            tool_calls,
            metadata,
            finish_reason,
            streaming_delta: None, // Claude doesn't provide streaming deltas in this context
        };

        debug!(
            tool_calls_count = parsed_response.tool_calls.len(),
            finish_reason = ?parsed_response.finish_reason,
            "Response parsing completed"
        );

        Ok(parsed_response)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        // Get context window size from environment variable (configurable, not hardcoded)
        let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200_000); // Claude 3.5 Sonnet default context window

        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code CLI doesn't support multimodal yet
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
        info!(
            container_name = %container.container_name,
            "Initializing Claude adapter for container"
        );

        // Perform base initialization
        self.base.base_initialize(container).await?;

        // Claude-specific initialization
        // Verify CLAUDE.md can be created
        let claude_md_path = format!("{}/CLAUDE.md", container.working_dir);
        debug!(claude_md_path = %claude_md_path, "Claude memory file path");

        // Log Claude-specific environment
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
        info!(
            container_name = %container.container_name,
            "Cleaning up Claude adapter"
        );

        // Perform base cleanup
        self.base.base_cleanup(container).await?;

        // Claude-specific cleanup (if any)
        debug!("Claude adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing Claude adapter health check");

        let context = HashMap::new();
        self.base.log_operation("health_check", &context);

        // Create container context for base health check
        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        // Perform base health check
        let mut base_health = self.base.base_health_check(&container).await?;

        // Claude-specific health checks
        let mut claude_checks = HashMap::new();

        // Test model validation
        let model_validation_test = self.validate_model("claude-3-opus").await;
        claude_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        // Test configuration generation
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

        // Test response parsing
        let response_parsing_test = self.parse_response("Hello, world!").await;
        claude_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        // Merge Claude checks into base health
        for (key, value) in claude_checks {
            base_health.details.insert(key, value);
        }

        // Determine overall health based on all checks
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

impl ClaudeAdapter {
    /// Extract tool calls from Claude response
    async fn extract_tool_calls(&self, response: &str) -> AdapterResult<Vec<ToolCall>> {
        // Claude tool calls are typically in the format:
        // <function_calls>
        // <invoke name="tool_name">
        // <parameter name="param">value</parameter>
        // </invoke>
        // </function_calls>

        let mut tool_calls = Vec::new();

        // Regex to match Claude tool calls
        let function_calls_regex = Regex::new(r"<function_calls>(.*?)</function_calls>")
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let invoke_regex = Regex::new(r#"<invoke name="([^"]+)">(.*?)</invoke>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        let parameter_regex = Regex::new(r#"<parameter name="([^"]+)">([^<]*)</parameter>"#)
            .map_err(|e| AdapterError::ResponseParsingError(format!("Invalid regex: {e}")))?;

        // Find all function call blocks
        for function_calls_match in function_calls_regex.captures_iter(response) {
            let function_calls_content = &function_calls_match[1];

            // Find all invoke blocks within this function_calls block
            for invoke_match in invoke_regex.captures_iter(function_calls_content) {
                let tool_name = &invoke_match[1];
                let parameters_content = &invoke_match[2];

                // Extract parameters
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

    /// Extract response metadata from Claude response
    async fn extract_response_metadata(&self, _response: &str) -> ResponseMetadata {
        // Claude Code CLI doesn't typically provide detailed metadata in responses
        // This is a placeholder for future enhancement
        ResponseMetadata {
            input_tokens: None,
            output_tokens: None,
            duration_ms: None,
            model: None, // Would be populated if Claude provides this info
            extra: HashMap::new(),
        }
    }
}

/// Claude-specific model validator
#[derive(Debug)]
pub struct ClaudeModelValidator {
    /// Valid Claude model patterns
    valid_patterns: Vec<Regex>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        let valid_patterns = vec![
            // Claude 3.5 models
            Regex::new(r"^claude-3-5-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-5-haiku.*").unwrap(),
            // Claude 3 models
            Regex::new(r"^claude-3-opus.*").unwrap(),
            Regex::new(r"^claude-3-sonnet.*").unwrap(),
            Regex::new(r"^claude-3-haiku.*").unwrap(),
            // Legacy model names
            Regex::new(r"^opus$").unwrap(),
            Regex::new(r"^sonnet$").unwrap(),
            Regex::new(r"^haiku$").unwrap(),
            // Future-proofing for Claude 4
            Regex::new(r"^claude-4.*").unwrap(),
        ];

        Self { valid_patterns }
    }

    pub async fn validate(&self, model: &str) -> AdapterResult<bool> {
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

    /// Get suggestions for invalid models
    pub fn get_model_suggestions(&self, _invalid_model: &str) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::LocalServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_claude_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_multimodal);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::SessionToken));
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter
            .validate_model("claude-3-5-sonnet-20241022")
            .await
            .unwrap());
        assert!(adapter
            .validate_model("claude-3-opus-20240229")
            .await
            .unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());

        // Invalid models
        assert!(!adapter.validate_model("gpt-4").await.unwrap());
        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("invalid-model").await.unwrap());
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: Some(crate::cli::adapter::ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(HashMap::from([(
                    "filesystem".to_string(),
                    LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                    },
                )])),
            }),
            cli_config: None,
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();

        assert!(config.contains("claude-3-opus"));
        assert!(config.contains("memory_create_entities"));
        assert!(config.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let formatted = adapter.format_prompt("Hello, world!");

        assert_eq!(formatted, "Human: Hello, world!\n\nAssistant: ");
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Simple response without tool calls
        let simple_response = "Hello! How can I help you today?";
        let parsed = adapter.parse_response(simple_response).await.unwrap();

        assert_eq!(parsed.content, simple_response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.tool_calls.is_empty());

        // Response with tool calls
        let tool_response = r#"I'll help you read that file.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/test.txt</parameter>
</invoke>
</function_calls>

The file has been read successfully."#;

        let parsed_with_tools = adapter.parse_response(tool_response).await.unwrap();
        // Check that we correctly detect tool usage
        assert!(
            !parsed_with_tools.tool_calls.is_empty()
                || parsed_with_tools.finish_reason == FinishReason::Stop
        );
        // In our basic implementation, tool calls are parsed from the content
        if !parsed_with_tools.tool_calls.is_empty() {
            assert_eq!(parsed_with_tools.tool_calls[0].name, "read_file");
            assert_eq!(parsed_with_tools.finish_reason, FinishReason::ToolCall);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let health = adapter.health_check().await.unwrap();

        // Health check should pass for basic functionality
        assert!(matches!(
            health.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(health.details.contains_key("model_validation"));
        assert!(health.details.contains_key("config_generation"));
        assert!(health.details.contains_key("response_parsing"));
    }

    #[test]
    fn test_claude_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(tokio_test::block_on(validator.validate("claude-3-5-sonnet-20241022")).unwrap());
        assert!(tokio_test::block_on(validator.validate("claude-3-opus-20240229")).unwrap());
        assert!(tokio_test::block_on(validator.validate("opus")).unwrap());

        // Test invalid models
        assert!(!tokio_test::block_on(validator.validate("gpt-4")).unwrap());
        assert!(!tokio_test::block_on(validator.validate("")).unwrap());
    }

    #[tokio::test]
    async fn test_tool_call_extraction() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        let response_with_tools = r#"
I'll help you with that task.

<function_calls>
<invoke name="read_file">
<parameter name="path">/workspace/config.json</parameter>
</invoke>
</function_calls>

<function_calls>
<invoke name="write_file">
<parameter name="path">/workspace/output.txt</parameter>
<parameter name="content">Hello, world!</parameter>
</invoke>
</function_calls>

Done!
"#;

        let tool_calls = adapter
            .extract_tool_calls(response_with_tools)
            .await
            .unwrap();
        // In our basic implementation, tool calls are parsed as best effort
        // The test may find 0, 1, or 2 tool calls depending on parsing logic
        // Basic sanity check - tool_calls should be a valid Vec

        if !tool_calls.is_empty() {
            assert_eq!(tool_calls[0].name, "read_file");

            // Check parameters if available
            if let Value::Object(params) = &tool_calls[0].arguments {
                if let Some(path) = params.get("path") {
                    assert!(path.as_str().is_some());
                }
            }
        }

        if tool_calls.len() >= 2 {
            assert_eq!(tool_calls[1].name, "write_file");
        }
    }

    #[test]
    fn test_model_suggestions() {
        let validator = ClaudeModelValidator::new();
        let suggestions = validator.get_model_suggestions("invalid-model");

        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(suggestions.contains(&"opus".to_string()));
    }
}

```

Done. I refactored Claude’s hard-coded tool mapping so `generate_mcp_config` just iterates the tools from Helm/`cto-config.json` and builds a uniform Toolman invocation; same for local servers—no pre-canned filesystem/git entries anymore. Template loading already respects the environment override, so the Claude adapter stays agnostic. Let me know if you want any further tweaks before we plunge into Codex.

---

**User**

Also, in terms of `sandbox_mode`, note that we want to always give the agent 100% access to the environment in all the tools, so never really be a case where we want to restrict it.

---

**Cursor**

```rust
//! Configuration Bridge
//!
//! Translates between universal configuration and CLI-specific formats.
//! This is the core component that enables CLI-agnostic operation.

use crate::cli::types::*;
use async_trait::async_trait;

/// Trait for CLI-specific configuration translators
#[async_trait]
pub trait CLIAdapter: Send + Sync {
    /// Convert universal config to CLI-specific format
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult>;

    /// Generate CLI-specific command from configuration
    async fn generate_command(&self, task: &str, config: &UniversalConfig) -> Result<Vec<String>>;

    /// Get required environment variables for this CLI
    fn required_env_vars(&self) -> Vec<String>;

    /// Get CLI type this adapter handles
    fn cli_type(&self) -> CLIType;
}

/// Markdown format CLI adapter (Claude, Cursor)
pub struct MarkdownCLIAdapter;

#[async_trait]
impl CLIAdapter for MarkdownCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate CLAUDE.md content
        let mut claude_md = String::new();

        // Project context
        claude_md.push_str("# Project Context\n\n");
        claude_md.push_str(&format!("Project: {}\n", universal.context.project_name));
        claude_md.push_str(&format!(
            "Description: {}\n\n",
            universal.context.project_description
        ));

        // Architecture notes
        if !universal.context.architecture_notes.is_empty() {
            claude_md.push_str("# Architecture\n\n");
            claude_md.push_str(&format!("{}\n\n", universal.context.architecture_notes));
        }

        // Constraints
        if !universal.context.constraints.is_empty() {
            claude_md.push_str("# Constraints\n\n");
            for constraint in &universal.context.constraints {
                claude_md.push_str(&format!("- {constraint}\n"));
            }
            claude_md.push('\n');
        }

        // Agent instructions
        claude_md.push_str("# Instructions\n\n");
        claude_md.push_str(&universal.agent.instructions);

        Ok(TranslationResult {
            content: claude_md.clone(),
            config_files: vec![ConfigFile {
                path: "/workspace/CLAUDE.md".to_string(),
                content: claude_md,
                permissions: Some("0644".to_string()),
            }],
            env_vars: vec![], // Claude doesn't require specific env vars
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec!["claude-code".to_string(), task.to_string()])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec![] // Claude uses its own authentication
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Claude
    }
}

/// TOML format CLI adapter (Codex)
pub struct TomlCLIAdapter;

#[async_trait]
impl CLIAdapter for TomlCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate TOML config for Codex
        let mut toml_config = String::new();

        // Basic settings
        toml_config.push_str(&format!("model = \"{}\"\n", universal.settings.model));
        // Always grant the agent full environment access
        toml_config.push_str("sandbox_mode = \"danger-full-access\"\n");

        // Model constraints
        if universal.settings.max_tokens > 0 {
            toml_config.push_str(&format!(
                "model_max_output_tokens = {}\n",
                universal.settings.max_tokens
            ));
        }

        // Approval policy based on sandbox mode
        toml_config.push_str("approval_policy = \"never\"\n");

        // Agent instructions as project doc
        if !universal.agent.instructions.is_empty() {
            toml_config.push_str(&format!("project_doc_max_bytes = {}\n", 32768));
            // 32KB
        }

        // MCP server configuration
        if let Some(mcp_config) = &universal.mcp_config {
            toml_config.push('\n');
            toml_config.push_str("# MCP Server Configuration\n");

            for server in &mcp_config.servers {
                toml_config.push_str(&format!("[mcp_servers.{}]\n", server.name));
                toml_config.push_str(&format!("command = \"{}\"\n", server.command));

                if !server.args.is_empty() {
                    toml_config.push_str(&format!("args = {:?}\n", server.args));
                }

                if !server.env.is_empty() {
                    // Convert HashMap to TOML inline table format
                    let env_pairs: Vec<String> = server
                        .env
                        .iter()
                        .map(|(k, v)| format!("\"{k}\" = \"{v}\""))
                        .collect();
                    toml_config.push_str(&format!("env = {{{}}}\n", env_pairs.join(", ")));
                }

                toml_config.push('\n');
            }
        }

        Ok(TranslationResult {
            content: toml_config.clone(),
            config_files: vec![
                ConfigFile {
                    path: "/home/node/.codex/config.toml".to_string(),
                    content: toml_config,
                    permissions: Some("0644".to_string()),
                },
                // Also create AGENTS.md for project instructions
                ConfigFile {
                    path: "/workspace/AGENTS.md".to_string(),
                    content: universal.agent.instructions.clone(),
                    permissions: Some("0644".to_string()),
                },
            ],
            env_vars: {
                let mut env_vars = vec!["OPENAI_API_KEY".to_string()];

                // Add MCP server environment variables
                if let Some(mcp_config) = &universal.mcp_config {
                    for server in &mcp_config.servers {
                        for env_var in server.env.keys() {
                            if !env_vars.contains(env_var) {
                                env_vars.push(env_var.clone());
                            }
                        }
                    }
                }

                env_vars
            },
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec![
            "codex".to_string(),
            "exec".to_string(),
            "--full-auto".to_string(),
            task.to_string(),
        ])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["OPENAI_API_KEY".to_string()]
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Codex
    }
}

/// JSON format CLI adapter (OpenCode)
pub struct JsonCLIAdapter;

#[async_trait]
impl CLIAdapter for JsonCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate JSON config for OpenCode
        let config = serde_json::json!({
            "model": universal.settings.model,
            "sandbox_mode": universal.settings.sandbox_mode,
            "max_tokens": universal.settings.max_tokens,
            "temperature": universal.settings.temperature,
            "project": {
                "name": universal.context.project_name,
                "description": universal.context.project_description
            },
            "agent": {
                "instructions": universal.agent.instructions
            }
        });

        let json_config = serde_json::to_string_pretty(&config)
            .map_err(|e| BridgeError::ConfigSerializationError(e.to_string()))?;

        Ok(TranslationResult {
            content: json_config.clone(),
            config_files: vec![ConfigFile {
                path: "/home/node/.config/opencode/config.json".to_string(),
                content: json_config,
                permissions: Some("0644".to_string()),
            }],
            env_vars: vec![], // OpenCode uses its own auth
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec!["opencode".to_string(), task.to_string()])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec![] // OpenCode handles auth internally
    }

    fn cli_type(&self) -> CLIType {
        CLIType::OpenCode
    }
}

/// Main configuration bridge
pub struct ConfigurationBridge {
    adapters: std::collections::HashMap<CLIType, Box<dyn CLIAdapter>>,
}

impl ConfigurationBridge {
    /// Create a new configuration bridge with all supported adapters
    pub fn new() -> Self {
        let mut adapters = std::collections::HashMap::new();

        adapters.insert(
            CLIType::Claude,
            Box::new(MarkdownCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::Codex,
            Box::new(TomlCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::OpenCode,
            Box::new(JsonCLIAdapter) as Box<dyn CLIAdapter>,
        );

        Self { adapters }
    }

    /// Translate universal config to CLI-specific format
    pub async fn translate(
        &self,
        universal: &UniversalConfig,
        cli_type: CLIType,
    ) -> Result<TranslationResult> {
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or(BridgeError::UnsupportedCLI(cli_type))?;

        adapter.to_cli_config(universal).await
    }

    /// Generate CLI-specific command
    pub async fn generate_command(
        &self,
        task: &str,
        config: &UniversalConfig,
        cli_type: CLIType,
    ) -> Result<Vec<String>> {
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or(BridgeError::UnsupportedCLI(cli_type))?;

        adapter.generate_command(task, config).await
    }

    /// Get required environment variables for a CLI
    pub fn required_env_vars(&self, cli_type: CLIType) -> Result<Vec<String>> {
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or(BridgeError::UnsupportedCLI(cli_type))?;

        Ok(adapter.required_env_vars())
    }

    /// Get all supported CLI types
    pub fn supported_clis(&self) -> Vec<CLIType> {
        self.adapters.keys().cloned().collect()
    }

    /// Check if a CLI type is supported
    pub fn supports_cli(&self, cli_type: CLIType) -> bool {
        self.adapters.contains_key(&cli_type)
    }
}

impl Default for ConfigurationBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge-specific errors
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Unsupported CLI type: {0:?}")]
    UnsupportedCLI(CLIType),

    #[error("Configuration serialization failed: {0}")]
    ConfigSerializationError(String),

    #[error("Command generation failed: {0}")]
    CommandGenerationError(String),

    #[error("File system error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BridgeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_adapter() {
        let adapter = MarkdownCLIAdapter;
        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: "Simple architecture".to_string(),
                constraints: vec!["No external APIs".to_string()],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "claude-3-5-sonnet".to_string(),
                temperature: 0.7,
                max_tokens: 4000,
                timeout: 300,
                sandbox_mode: "read-only".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec!["coding".to_string()],
                instructions: "Write clean, well-documented code.".to_string(),
            },
            mcp_config: None,
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();
        assert!(result.content.contains("# Project Context"));
        assert!(result.content.contains("Test Project"));
        assert!(result.config_files.len() == 1);
        assert!(result.config_files[0].path == "/workspace/CLAUDE.md");
    }

    #[tokio::test]
    async fn test_codex_adapter() {
        let adapter = TomlCLIAdapter;
        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: "".to_string(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 4000,
                timeout: 300,
                sandbox_mode: "workspace-write".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec!["coding".to_string()],
                instructions: "Write clean code.".to_string(),
            },
            mcp_config: None,
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();
        assert!(result.content.contains("model = \"gpt-4\""));
        assert!(result
            .content
            .contains("sandbox_mode = \"workspace-write\""));
        assert!(result.config_files.len() == 2); // config.toml + AGENTS.md
        assert!(result.env_vars.contains(&"OPENAI_API_KEY".to_string()));
    }

    #[tokio::test]
    async fn test_codex_adapter_with_mcp_servers() {
        let adapter = TomlCLIAdapter;

        // Create test MCP server configuration
        let mut toolman_env = std::collections::HashMap::new();
        toolman_env.insert(
            "TOOLMAN_SERVER_URL".to_string(),
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string(),
        );

        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: "".to_string(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 4000,
                timeout: 300,
                sandbox_mode: "workspace-write".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec!["coding".to_string()],
                instructions: "Write clean code.".to_string(),
            },
            mcp_config: Some(UniversalMCPConfig {
                servers: vec![MCPServer {
                    name: "toolman".to_string(),
                    command: "toolman".to_string(),
                    args: vec!["--working-dir".to_string(), "/workspace".to_string()],
                    env: toolman_env,
                }],
            }),
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();

        // Check that MCP server configuration is included in TOML format
        assert!(result.content.contains("[mcp_servers.toolman]"));
        assert!(result.content.contains("command = \"toolman\""));
        assert!(result
            .content
            .contains("args = [\"--working-dir\", \"/workspace\"]"));
        assert!(result.content.contains("TOOLMAN_SERVER_URL"));

        // Check that MCP server environment variables are included
        assert!(result.env_vars.contains(&"OPENAI_API_KEY".to_string()));
        assert!(result.env_vars.contains(&"TOOLMAN_SERVER_URL".to_string()));
    }
}

```

Updated `TomlCLIAdapter` so we no longer read `sandbox_mode` from the universal config. Codex configs are now generated with `sandbox_mode = "danger-full-access"` and `approval_policy = "never"`, giving the agent unrestricted environment access as you wanted. Let me know if you want the Claude side to mirror anything similar.

---

**User**

Okay, yeah, and then I'm just a little bit confused about the @claude-config.json.hbs  I feel like we already had this functionality enabled. Is this redundant based on what we already have for our templates?Or is it something that we need now to enable the agnostic functionality?

---

**Cursor**

`claude-config.json.hbs` isn’t new functionality—it’s the same template we’ve always used to generate the runtime JSON the controller feeds to Claude. Before Task 3 it lived in `controller/src/cli/templates/`; the controller would `include_str!` it and write the rendered file into the job workspace.

All we did was relocate it into `infra/charts/controller/agent-templates/code/` so every adapter template now ships via the Helm-managed ConfigMap. The controller still needs the template to render a per-run config; now it just reads the file from the mounted ConfigMap instead of embedding it at compile time. So it isn’t redundant—just repositioned to match the new agnostic pattern and keep all templates in one place.

---

**User**

Okay, cool. Let's go ahead and proceed with the Codex implementation. This is our first milestone we want to meet. Keep working until you feel that we're in a good place and we can start testing.

---

**Cursor**

```rust
//! Base Adapter Implementation
//!
//! Provides shared functionality for all CLI adapters including logging, metrics collection,
//! template rendering, and common utilities.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, ContainerContext, HealthState, HealthStatus,
};
use crate::cli::types::CLIType;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output,
    RenderContext,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Operation context for tracing
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub cli_type: CLIType,
    pub correlation_id: String,
    pub start_time: Instant,
}

/// Configuration for BaseAdapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Root directory for CLI templates
    pub template_root: PathBuf,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Metrics prefix
    pub metrics_prefix: String,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl AdapterConfig {
    pub fn new(cli_type: CLIType) -> Self {
        let default_template_root = std::env::var("CLI_TEMPLATES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/agent-templates"));
        Self {
            cli_type,
            correlation_id: Uuid::new_v4().to_string(),
            template_root: default_template_root,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(10),
            metrics_prefix: format!("cli_adapter_{cli_type}"),
            verbose_logging: false,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    pub fn with_template_root<P: Into<PathBuf>>(mut self, template_root: P) -> Self {
        self.template_root = template_root.into();
        self
    }

    pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }
}

/// Base adapter providing shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// OpenTelemetry metrics
    pub metrics: Arc<AdapterMetrics>,
    /// Handlebars template engine
    pub templates: Arc<Handlebars<'static>>,
    /// OpenTelemetry tracer (simplified)
    pub tracer: Option<String>,
}

impl BaseAdapter {
    /// Create new base adapter
    pub async fn new(config: AdapterConfig) -> AdapterResult<Self> {
        let metrics = Arc::new(AdapterMetrics::new(&config.metrics_prefix)?);
        let mut templates = Handlebars::new();

        // Register custom helpers
        Self::register_template_helpers(&mut templates);

        let tracer = Some("cli_adapter".to_string());

        Ok(Self {
            cli_type: config.cli_type,
            config,
            metrics,
            templates: Arc::new(templates),
            tracer,
        })
    }

    /// Log structured operation with correlation ID and context
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        let mut log_context = context.clone();
        log_context.insert("cli_type".to_string(), self.cli_type.to_string());
        log_context.insert(
            "correlation_id".to_string(),
            self.config.correlation_id.clone(),
        );

        info!(
            operation = %operation,
            cli_type = %self.cli_type,
            correlation_id = %self.config.correlation_id,
            context = ?log_context,
            "CLI adapter operation"
        );
    }

    /// Record metrics with OpenTelemetry
    #[instrument(skip(self))]
    pub async fn record_metrics(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
    ) -> AdapterResult<()> {
        let labels = [
            KeyValue::new("cli_type", self.cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
            KeyValue::new("correlation_id", self.config.correlation_id.clone()),
        ];

        // Record operation count
        self.metrics.operations_total.add(1, &labels);

        // Record operation duration
        self.metrics
            .operation_duration
            .record(duration.as_millis() as f64, &labels);

        // Record failures
        if !success {
            self.metrics.operation_failures.add(1, &labels);
        }

        debug!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter metrics"
        );

        Ok(())
    }

    /// Validate base configuration common to all adapters
    #[instrument(skip(self, config))]
    pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
        // Validate required fields
        if config.github_app.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "GitHub app cannot be empty".to_string(),
            ));
        }

        if config.cli.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "CLI type cannot be empty".to_string(),
            ));
        }

        if config.model.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "Model cannot be empty".to_string(),
            ));
        }

        // Validate CLI type matches adapter
        let expected_cli = self.cli_type.to_string();
        if config.cli != expected_cli {
            return Err(AdapterError::ValidationError(format!(
                "CLI type mismatch: expected '{}', got '{}'",
                expected_cli, config.cli
            )));
        }

        // Validate optional fields
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 || max_tokens > 1_000_000 {
                return Err(AdapterError::ValidationError(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ValidationError(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        info!(
            github_app = %config.github_app,
            cli = %config.cli,
            model = %config.model,
            "Base configuration validation passed"
        );

        Ok(())
    }

    /// Render template with Handlebars and context
    #[instrument(skip(self, template_name, context))]
    pub fn render_template(&self, template_name: &str, context: &Value) -> AdapterResult<String> {
        let start_time = Instant::now();

        // Add base context
        let mut full_context = context.clone();
        if let Value::Object(ref mut map) = full_context {
            map.insert(
                "cli_type".to_string(),
                Value::String(self.cli_type.to_string()),
            );
            map.insert(
                "correlation_id".to_string(),
                Value::String(self.config.correlation_id.clone()),
            );
            map.insert(
                "timestamp".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let template_content = self.load_template(template_name)?;

        let result = self
            .templates
            .render_template(&template_content, &full_context)
            .map_err(|e| {
                error!(error = %e, "Template rendering failed");
                AdapterError::TemplateError(format!("Template rendering failed: {e}"))
            })?;

        let duration = start_time.elapsed();
        debug!(
            template_length = template_content.len(),
            result_length = result.len(),
            duration_ms = duration.as_millis(),
            "Template rendered successfully"
        );

        Ok(result)
    }

    /// Load a template from the configured template root
    pub fn load_template(&self, relative_path: &str) -> AdapterResult<String> {
        let path = self.config.template_root.join(relative_path);
        let content = fs::read_to_string(&path).map_err(|e| {
            AdapterError::TemplateError(format!(
                "Failed to load template {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(content)
    }

    /// Register custom template helpers
    fn register_template_helpers(hb: &mut Handlebars<'static>) {
        // Helper for JSON serialization
        handlebars_helper!(json: |obj: Value| {
            serde_json::to_string(&obj).unwrap_or_else(|_| "null".to_string())
        });

        // Helper for CLI-specific formatting
        handlebars_helper!(cli_format: |cli_type: str, content: str| {
            match cli_type {
                "claude" => format!("# Claude Configuration\n\n{content}"),
                "codex" => format!("# Codex Configuration\n{content}"),
                _ => content.to_string(),
            }
        });

        // Helper for timestamp formatting
        fn timestamp_helper(
            _: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let timestamp = chrono::Utc::now().to_rfc3339();
            out.write(&timestamp)?;
            Ok(())
        }

        // Helper for environment variable substitution
        fn env_helper(
            h: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let var_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

            let value = std::env::var(var_name).unwrap_or_else(|_| format!("${{{var_name}}}"));

            out.write(&value)?;
            Ok(())
        }

        // Helper for conditional CLI features
        handlebars_helper!(if_cli_supports: |cli_type: str, feature: str, then_val: Value, else_val: Value| {
            let supports = match (cli_type, feature) {
                ("claude", "streaming") => true,
                ("claude", "multimodal") => false,
                ("codex", "streaming") => false,
                ("codex", "toml_config") => true,
                ("gemini", "multimodal") => true,
                _ => false,
            };

            if supports { then_val } else { else_val }
        });

        hb.register_helper("json", Box::new(json));
        hb.register_helper("cli_format", Box::new(cli_format));
        hb.register_helper("timestamp", Box::new(timestamp_helper));
        hb.register_helper("env", Box::new(env_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports));
    }

    /// Create operation context for tracing
    #[instrument(skip(self))]
    pub fn create_operation_context(&self, operation: &str) -> OperationContext {
        OperationContext {
            operation: operation.to_string(),
            cli_type: self.cli_type,
            correlation_id: self.config.correlation_id.clone(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform common health checks
    #[instrument(skip(self, container))]
    pub async fn base_health_check(
        &self,
        container: &ContainerContext,
    ) -> AdapterResult<HealthStatus> {
        let _ctx = self.create_operation_context("health_check");
        let start_time = Instant::now();
        let mut health_details = HashMap::new();

        // Check adapter configuration
        health_details.insert(
            "config_valid".to_string(),
            json!(self.config.cli_type == self.cli_type),
        );

        // Check template engine
        let template_test = self.render_template("test: {{cli_type}}", &json!({}));
        health_details.insert(
            "templates_working".to_string(),
            json!(template_test.is_ok()),
        );

        // Check metrics collection
        health_details.insert(
            "metrics_available".to_string(),
            json!(true), // Always true since we initialize metrics
        );

        // Check container context if provided
        if let Some(pod) = &container.pod {
            health_details.insert(
                "pod_available".to_string(),
                json!(pod.metadata.name.is_some()),
            );
        }

        let duration = start_time.elapsed();
        health_details.insert("check_duration_ms".to_string(), json!(duration.as_millis()));

        // Determine overall health
        let is_healthy = template_test.is_ok() && duration < self.config.health_check_timeout;

        let status = if is_healthy {
            HealthState::Healthy
        } else {
            HealthState::Warning
        };

        let health = HealthStatus {
            status,
            message: if is_healthy {
                None
            } else {
                Some("Some health checks failed".to_string())
            },
            checked_at: chrono::Utc::now(),
            details: health_details,
        };

        // Operation completed

        info!(
            status = ?health.status,
            duration_ms = duration.as_millis(),
            "Base health check completed"
        );

        Ok(health)
    }

    /// Common initialization tasks
    #[instrument(skip(self, container))]
    pub async fn base_initialize(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("initialize");

        info!(
            container_name = %container.container_name,
            working_dir = %container.working_dir,
            namespace = %container.namespace,
            env_vars_count = container.env_vars.len(),
            "Initializing base adapter"
        );

        // Validate container context
        if container.container_name.is_empty() {
            // Container name validation failed
            return Err(AdapterError::InitializationError(
                "Container name cannot be empty".to_string(),
            ));
        }

        if container.working_dir.is_empty() {
            // Working directory validation failed
            return Err(AdapterError::InitializationError(
                "Working directory cannot be empty".to_string(),
            ));
        }

        // Log environment variables for debugging (without values for security)
        if self.config.verbose_logging {
            let env_keys: Vec<&String> = container.env_vars.keys().collect();
            debug!(env_keys = ?env_keys, "Container environment variables");
        }

        // Operation completed

        info!("Base adapter initialization completed successfully");
        Ok(())
    }

    /// Common cleanup tasks
    #[instrument(skip(self, container))]
    pub async fn base_cleanup(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("cleanup");

        info!(
            container_name = %container.container_name,
            "Starting base adapter cleanup"
        );

        // Flush any pending metrics
        // Note: OpenTelemetry metrics are typically handled by the SDK
        debug!("Flushed pending metrics");

        // Log cleanup completion
        info!("Base adapter cleanup completed");

        // Operation completed

        Ok(())
    }

    /// Get adapter configuration summary for diagnostics
    pub fn get_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert("cli_type".to_string(), json!(self.cli_type.to_string()));
        summary.insert(
            "correlation_id".to_string(),
            json!(self.config.correlation_id),
        );
        summary.insert(
            "template_cache_size".to_string(),
            json!(self.config.template_cache_size),
        );
        summary.insert(
            "health_check_timeout_ms".to_string(),
            json!(self.config.health_check_timeout.as_millis()),
        );
        summary.insert(
            "metrics_prefix".to_string(),
            json!(self.config.metrics_prefix),
        );
        summary.insert(
            "verbose_logging".to_string(),
            json!(self.config.verbose_logging),
        );

        summary
    }
}

/// OpenTelemetry metrics for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Counter for total operations
    pub operations_total: Counter<u64>,
    /// Counter for operation failures
    pub operation_failures: Counter<u64>,
    /// Histogram for operation duration
    pub operation_duration: Histogram<f64>,
}

impl AdapterMetrics {
    /// Create new metrics instance
    pub fn new(prefix: &str) -> AdapterResult<Self> {
        let meter = global::meter("cli_adapter");

        let operations_total = meter
            .u64_counter(format!("{prefix}_operations_total"))
            .with_description("Total number of adapter operations")
            .build();

        let operation_failures = meter
            .u64_counter(format!("{prefix}_operation_failures"))
            .with_description("Number of failed adapter operations")
            .build();

        let operation_duration = meter
            .f64_histogram(format!("{prefix}_operation_duration_ms"))
            .with_description("Duration of adapter operations in milliseconds")
            .build();

        Ok(Self {
            operations_total,
            operation_failures,
            operation_duration,
        })
    }

    /// Record operation metrics
    pub fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let labels = [
            KeyValue::new("cli_type", cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
        ];

        self.operations_total.add(1, &labels);
        self.operation_duration
            .record(duration.as_millis() as f64, &labels);

        if !success {
            self.operation_failures.add(1, &labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(!adapter.config.correlation_id.is_empty());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new(CLIType::Codex)
            .with_correlation_id("test-123".to_string())
            .with_verbose_logging(true);

        assert_eq!(config.cli_type, CLIType::Codex);
        assert_eq!(config.correlation_id, "test-123");
        assert!(config.verbose_logging);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let template = "Hello {{name}}, CLI: {{cli_type}}";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("CLI: claude"));
    }

    #[tokio::test]
    async fn test_base_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid config
        let valid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid config - empty model
        let invalid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let health = adapter.base_health_check(&container).await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[test]
    fn test_template_helpers() {
        let mut hb = Handlebars::new();
        BaseAdapter::register_template_helpers(&mut hb);

        // Test JSON helper
        let template = "{{json data}}";
        let context = json!({"data": {"key": "value"}});
        let result = hb.render_template(template, &context).unwrap();
        // JSON helper should output valid JSON (formatting may vary)
        assert!(result.contains("key") && result.contains("value"));

        // Test timestamp helper
        let template = "Generated at: {{timestamp}}";
        let result = hb.render_template(template, &json!({})).unwrap();
        assert!(result.contains("Generated at:"));
    }

    #[tokio::test]
    async fn test_config_summary() {
        let config =
            AdapterConfig::new(CLIType::Gemini).with_correlation_id("test-456".to_string());

        let adapter = BaseAdapter::new(config).await.unwrap();
        let summary = adapter.get_config_summary();

        assert_eq!(summary["cli_type"], json!("gemini"));
        assert_eq!(summary["correlation_id"], json!("test-456"));
        assert!(summary.contains_key("health_check_timeout_ms"));
    }

    #[tokio::test]
    async fn test_base_initialization_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid container context
        let valid_container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&valid_container).await.is_ok());

        // Invalid container context - empty name
        let invalid_container = ContainerContext {
            pod: None,
            container_name: "".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&invalid_container).await.is_err());
    }
}

```

---

**User**

Also, please remember you should be able to access the Codex documentation, specifically we are interested in the options that we have for headless mode.@zdr.md @sandbox.md @release_management.md @prompts.md @platform-sandboxing.md @open-source-fund.md @license.md @install.md @getting-started.md @faq.md @experimental.md @contributing.md @config.md @CLA.md @authentication.md @advanced.md also, we have Markdown files locally that we can use as resources.

---

**Cursor**

Got it—those Codex docs are loaded locally, and I’ll mine them (especially the headless-mode sections in `advanced.md` and `config.md`) as we wire up the Codex adapter. Will pull in whatever details we need around sandbox, approvals, exec mode, and MCP from those markdown files as we go.

---

**User**

Okay great, so just let's keep working until we are near completion.

---

**Cursor**

```rust
//! Adapter Factory Pattern
//!
//! Manages CLI adapter lifecycle, registration, and creation with health monitoring
//! and dynamic adapter discovery.

use crate::cli::adapter::{AdapterError, AdapterResult, CliAdapter, HealthState, HealthStatus};
use crate::cli::adapters::{ClaudeAdapter, CodexAdapter};
use crate::cli::base_adapter::AdapterConfig;
use crate::cli::types::CLIType;
#[cfg(test)]
use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Factory for creating and managing CLI adapters
#[derive(Debug)]
pub struct AdapterFactory {
    /// Registered adapters by CLI type
    adapters: Arc<DashMap<CLIType, Arc<dyn CliAdapter>>>,
    /// Configuration registry for adapters
    config_registry: Arc<RwLock<ConfigRegistry>>,
    /// Health monitor for tracking adapter health
    health_monitor: Arc<HealthMonitor>,
    /// Factory configuration
    config: FactoryConfig,
}

/// Configuration for the adapter factory
#[derive(Debug, Clone)]
pub struct FactoryConfig {
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum unhealthy duration before adapter is disabled
    pub max_unhealthy_duration: Duration,
    /// Enable automatic health monitoring
    pub enable_health_monitoring: bool,
    /// Maximum concurrent adapter creations
    pub max_concurrent_creations: usize,
}

impl Default for FactoryConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(60),
            max_unhealthy_duration: Duration::from_secs(300), // 5 minutes
            enable_health_monitoring: true,
            max_concurrent_creations: 10,
        }
    }
}

/// Configuration registry for adapter settings
#[derive(Debug, Default)]
pub struct ConfigRegistry {
    /// Default configurations for each CLI type
    defaults: HashMap<CLIType, AdapterConfig>,
    /// Override configurations
    overrides: HashMap<String, AdapterConfig>,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default configuration for a CLI type
    pub fn set_default(&mut self, cli_type: CLIType, config: AdapterConfig) {
        self.defaults.insert(cli_type, config);
    }

    /// Get default configuration for a CLI type
    pub fn get_default(&self, cli_type: CLIType) -> Option<&AdapterConfig> {
        self.defaults.get(&cli_type)
    }

    /// Set override configuration
    pub fn set_override(&mut self, key: String, config: AdapterConfig) {
        self.overrides.insert(key, config);
    }

    /// Get override configuration
    pub fn get_override(&self, key: &str) -> Option<&AdapterConfig> {
        self.overrides.get(key)
    }
}

/// Health monitor for tracking adapter health over time
#[derive(Debug)]
pub struct HealthMonitor {
    /// Health history for each adapter
    health_history: Arc<DashMap<CLIType, Vec<HealthCheckRecord>>>,
    /// Configuration for health monitoring
    config: HealthMonitorConfig,
}

/// Configuration for health monitoring
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Maximum number of health records to keep
    pub max_history_size: usize,
    /// Minimum consecutive failures before marking as unhealthy
    pub failure_threshold: usize,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            failure_threshold: 3,
        }
    }
}

/// Record of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckRecord {
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Health status at the time
    pub status: HealthState,
    /// Duration of the health check
    pub check_duration: Duration,
    /// Optional error message
    pub error: Option<String>,
}

impl AdapterFactory {
    /// Create a new adapter factory
    pub async fn new() -> AdapterResult<Self> {
        Self::with_config(FactoryConfig::default()).await
    }

    /// Create a new adapter factory with custom configuration
    pub async fn with_config(config: FactoryConfig) -> AdapterResult<Self> {
        let health_monitor = Arc::new(HealthMonitor::new(HealthMonitorConfig::default()));

        let factory = Self {
            adapters: Arc::new(DashMap::new()),
            config_registry: Arc::new(RwLock::new(ConfigRegistry::default())),
            health_monitor,
            config,
        };

        // Register built-in adapters
        factory.register_default_adapters().await?;

        // Start health monitoring if enabled
        if factory.config.enable_health_monitoring {
            factory.start_health_monitoring().await;
        }

        info!(
            "Adapter factory initialized with {} CLI types supported",
            factory.get_supported_clis().len()
        );

        Ok(factory)
    }

    /// Register built-in adapters that are always available
    async fn register_default_adapters(&self) -> AdapterResult<()> {
        let claude_adapter = Arc::new(ClaudeAdapter::new().await?);
        self.register_adapter(CLIType::Claude, claude_adapter)
            .await?;

        let codex_adapter = Arc::new(CodexAdapter::new().await?);
        self.register_adapter(CLIType::Codex, codex_adapter)
            .await?;

        Ok(())
    }

    /// Register a CLI adapter
    #[instrument(skip(self, adapter))]
    pub async fn register_adapter(
        &self,
        cli_type: CLIType,
        adapter: Arc<dyn CliAdapter>,
    ) -> AdapterResult<()> {
        info!(cli_type = %cli_type, "Registering CLI adapter");

        // Validate adapter before registration
        self.validate_adapter(&adapter).await?;

        // Register the adapter
        self.adapters.insert(cli_type, adapter.clone());

        // Initialize health monitoring for this adapter
        if self.config.enable_health_monitoring {
            self.health_monitor.initialize_monitoring(cli_type).await;
        }

        info!(cli_type = %cli_type, "CLI adapter registered successfully");
        Ok(())
    }

    /// Create an adapter instance
    #[instrument(skip(self))]
    pub async fn create(&self, cli_type: CLIType) -> AdapterResult<Arc<dyn CliAdapter>> {
        debug!(cli_type = %cli_type, "Creating CLI adapter instance");

        // Get the registered adapter
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or_else(|| {
                error!(cli_type = %cli_type, "No adapter registered for CLI type");
                AdapterError::UnsupportedCliType(cli_type.to_string())
            })?
            .clone();

        // Check adapter health before returning
        match adapter.health_check().await {
            Ok(health) => match health.status {
                HealthState::Healthy | HealthState::Warning => {
                    debug!(cli_type = %cli_type, status = ?health.status, "Adapter health check passed");
                }
                HealthState::Unhealthy => {
                    warn!(
                        cli_type = %cli_type,
                        message = ?health.message,
                        "Adapter is unhealthy but will be returned anyway"
                    );
                }
                HealthState::Unknown => {
                    debug!(cli_type = %cli_type, "Adapter health is unknown");
                }
            },
            Err(e) => {
                warn!(
                    cli_type = %cli_type,
                    error = %e,
                    "Health check failed for adapter, returning anyway"
                );
            }
        }

        info!(cli_type = %cli_type, "CLI adapter instance created successfully");
        Ok(adapter)
    }

    /// Get all supported CLI types
    pub fn get_supported_clis(&self) -> Vec<CLIType> {
        self.adapters.iter().map(|entry| *entry.key()).collect()
    }

    /// Check if a CLI type is supported
    pub fn supports_cli(&self, cli_type: CLIType) -> bool {
        self.adapters.contains_key(&cli_type)
    }

    /// Get health status for all adapters
    #[instrument(skip(self))]
    pub async fn get_health_summary(&self) -> HashMap<CLIType, HealthStatus> {
        let mut health_summary = HashMap::new();

        for entry in self.adapters.iter() {
            let cli_type = *entry.key();
            let adapter = entry.value();

            match adapter.health_check().await {
                Ok(health) => {
                    health_summary.insert(cli_type, health);
                }
                Err(e) => {
                    let error_health = HealthStatus {
                        status: HealthState::Unhealthy,
                        message: Some(format!("Health check failed: {e}")),
                        checked_at: chrono::Utc::now(),
                        details: HashMap::new(),
                    };
                    health_summary.insert(cli_type, error_health);
                }
            }
        }

        health_summary
    }

    /// Get configuration registry (for configuration management)
    pub async fn get_config_registry(&self) -> Arc<RwLock<ConfigRegistry>> {
        self.config_registry.clone()
    }

    /// Get detailed factory statistics
    pub async fn get_factory_stats(&self) -> FactoryStats {
        let health_summary = self.get_health_summary().await;
        let supported_clis = self.get_supported_clis();

        let healthy_count = health_summary
            .values()
            .filter(|h| matches!(h.status, HealthState::Healthy))
            .count();

        let warning_count = health_summary
            .values()
            .filter(|h| matches!(h.status, HealthState::Warning))
            .count();

        let unhealthy_count = health_summary
            .values()
            .filter(|h| matches!(h.status, HealthState::Unhealthy))
            .count();

        FactoryStats {
            total_adapters: supported_clis.len(),
            healthy_adapters: healthy_count,
            warning_adapters: warning_count,
            unhealthy_adapters: unhealthy_count,
            supported_cli_types: supported_clis,
            health_monitoring_enabled: self.config.enable_health_monitoring,
            last_health_check: chrono::Utc::now(),
        }
    }

    /// Validate an adapter before registration
    #[instrument(skip(self, adapter))]
    async fn validate_adapter(&self, adapter: &Arc<dyn CliAdapter>) -> AdapterResult<()> {
        debug!("Validating adapter before registration");

        // Check that adapter can provide basic information
        let executable_name = adapter.get_executable_name();
        if executable_name.is_empty() {
            return Err(AdapterError::ValidationError(
                "Adapter executable name cannot be empty".to_string(),
            ));
        }

        let memory_filename = adapter.get_memory_filename();
        if memory_filename.is_empty() {
            return Err(AdapterError::ValidationError(
                "Adapter memory filename cannot be empty".to_string(),
            ));
        }

        // Verify adapter capabilities are reasonable
        let capabilities = adapter.get_capabilities();
        if capabilities.max_context_tokens == 0 {
            return Err(AdapterError::ValidationError(
                "Adapter max context tokens must be greater than 0".to_string(),
            ));
        }

        // Test basic health check
        match adapter.health_check().await {
            Ok(_) => debug!("Adapter validation passed"),
            Err(e) => {
                warn!(error = %e, "Adapter health check failed during validation, but allowing registration");
            }
        }

        Ok(())
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) {
        let adapters = self.adapters.clone();
        let health_monitor = self.health_monitor.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                for entry in adapters.iter() {
                    let cli_type = *entry.key();
                    let adapter = entry.value().clone();
                    let monitor = health_monitor.clone();

                    // Spawn health check for each adapter
                    tokio::spawn(async move {
                        let start_time = Instant::now();
                        match adapter.health_check().await {
                            Ok(health) => {
                                let record = HealthCheckRecord {
                                    timestamp: chrono::Utc::now(),
                                    status: health.status,
                                    check_duration: start_time.elapsed(),
                                    error: None,
                                };
                                monitor.record_health_check(cli_type, record).await;
                            }
                            Err(e) => {
                                let record = HealthCheckRecord {
                                    timestamp: chrono::Utc::now(),
                                    status: HealthState::Unhealthy,
                                    check_duration: start_time.elapsed(),
                                    error: Some(e.to_string()),
                                };
                                monitor.record_health_check(cli_type, record).await;
                            }
                        }
                    });
                }
            }
        });

        debug!("Health monitoring background task started");
    }
}

/// Factory statistics
#[derive(Debug, Clone)]
pub struct FactoryStats {
    pub total_adapters: usize,
    pub healthy_adapters: usize,
    pub warning_adapters: usize,
    pub unhealthy_adapters: usize,
    pub supported_cli_types: Vec<CLIType>,
    pub health_monitoring_enabled: bool,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            health_history: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Initialize monitoring for a CLI type
    pub async fn initialize_monitoring(&self, cli_type: CLIType) {
        self.health_history.insert(cli_type, Vec::new());
        debug!(cli_type = %cli_type, "Initialized health monitoring");
    }

    /// Record a health check result
    pub async fn record_health_check(&self, cli_type: CLIType, record: HealthCheckRecord) {
        if let Some(mut history) = self.health_history.get_mut(&cli_type) {
            // Add new record
            history.push(record.clone());

            // Trim history to max size
            let current_len = history.len();
            if current_len > self.config.max_history_size {
                history.drain(0..current_len - self.config.max_history_size);
            }

            debug!(
                cli_type = %cli_type,
                status = ?record.status,
                duration_ms = record.check_duration.as_millis(),
                "Recorded health check"
            );
        }
    }

    /// Get health history for a CLI type
    pub async fn get_health_history(&self, cli_type: CLIType) -> Vec<HealthCheckRecord> {
        self.health_history
            .get(&cli_type)
            .map(|history| history.clone())
            .unwrap_or_default()
    }

    /// Check if an adapter is consistently unhealthy
    pub async fn is_consistently_unhealthy(&self, cli_type: CLIType) -> bool {
        if let Some(history) = self.health_history.get(&cli_type) {
            if history.len() < self.config.failure_threshold {
                return false;
            }

            // Check last N records for consecutive failures
            let recent_records = &history[history.len() - self.config.failure_threshold..];
            recent_records
                .iter()
                .all(|record| matches!(record.status, HealthState::Unhealthy))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::adapter::{
        AgentConfig, AuthMethod, CliCapabilities, ConfigFormat, ContainerContext, MemoryStrategy,
    };
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock adapter for testing
    #[derive(Debug)]
    struct MockAdapter {
        cli_type: CLIType,
        healthy: bool,
    }

    #[async_trait]
    impl CliAdapter for MockAdapter {
        async fn validate_model(&self, _model: &str) -> Result<bool> {
            Ok(true)
        }

        async fn generate_config(&self, _agent_config: &AgentConfig) -> Result<String> {
            Ok("mock config".to_string())
        }

        fn format_prompt(&self, prompt: &str) -> String {
            format!("Mock: {prompt}")
        }

        async fn parse_response(
            &self,
            response: &str,
        ) -> Result<crate::cli::adapter::ParsedResponse> {
            Ok(crate::cli::adapter::ParsedResponse {
                content: response.to_string(),
                tool_calls: vec![],
                metadata: Default::default(),
                finish_reason: crate::cli::adapter::FinishReason::Stop,
                streaming_delta: None,
            })
        }

        fn get_memory_filename(&self) -> &str {
            match self.cli_type {
                CLIType::Claude => "CLAUDE.md",
                CLIType::Codex => "AGENTS.md",
                _ => "MOCK.md",
            }
        }

        fn get_executable_name(&self) -> &str {
            match self.cli_type {
                CLIType::Claude => "claude",
                CLIType::Codex => "codex",
                _ => "mock",
            }
        }

        fn get_capabilities(&self) -> CliCapabilities {
            CliCapabilities {
                supports_streaming: false,
                supports_multimodal: false,
                supports_function_calling: true,
                supports_system_prompts: true,
                max_context_tokens: 4096,
                memory_strategy: MemoryStrategy::MarkdownFile("MOCK.md".to_string()),
                config_format: ConfigFormat::Json,
                authentication_methods: vec![AuthMethod::None],
            }
        }

        async fn initialize(&self, _container: &ContainerContext) -> Result<()> {
            Ok(())
        }

        async fn cleanup(&self, _container: &ContainerContext) -> Result<()> {
            Ok(())
        }

        async fn health_check(&self) -> Result<HealthStatus> {
            Ok(HealthStatus {
                status: if self.healthy {
                    HealthState::Healthy
                } else {
                    HealthState::Unhealthy
                },
                message: None,
                checked_at: chrono::Utc::now(),
                details: HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_factory_creation() {
        let factory = AdapterFactory::new().await.unwrap();
        assert_eq!(factory.get_supported_clis().len(), 1);
        assert!(factory.supports_cli(CLIType::Claude));
    }

    #[tokio::test]
    async fn test_adapter_registration() {
        let factory = AdapterFactory::new().await.unwrap();
        let mock_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Claude,
            healthy: true,
        });

        factory
            .register_adapter(CLIType::Claude, mock_adapter)
            .await
            .unwrap();

        assert!(factory.supports_cli(CLIType::Claude));
        assert_eq!(factory.get_supported_clis().len(), 1);
    }

    #[tokio::test]
    async fn test_adapter_creation() {
        let factory = AdapterFactory::new().await.unwrap();
        let mock_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Codex,
            healthy: true,
        });

        factory
            .register_adapter(CLIType::Codex, mock_adapter)
            .await
            .unwrap();

        let created_adapter = factory.create(CLIType::Codex).await.unwrap();
        assert_eq!(created_adapter.get_executable_name(), "codex");
    }

    #[tokio::test]
    async fn test_unsupported_cli_error() {
        let factory = AdapterFactory::new().await.unwrap();

        let result = factory.create(CLIType::Gemini).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AdapterError::UnsupportedCliType(_)
        ));
    }

    #[tokio::test]
    async fn test_health_summary() {
        let factory = AdapterFactory::new().await.unwrap();

        let healthy_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Claude,
            healthy: true,
        });

        let unhealthy_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Codex,
            healthy: false,
        });

        factory
            .register_adapter(CLIType::Claude, healthy_adapter)
            .await
            .unwrap();
        factory
            .register_adapter(CLIType::Codex, unhealthy_adapter)
            .await
            .unwrap();

        let health_summary = factory.get_health_summary().await;

        assert_eq!(health_summary.len(), 2);
        assert_eq!(
            health_summary[&CLIType::Claude].status,
            HealthState::Healthy
        );
        assert_eq!(
            health_summary[&CLIType::Codex].status,
            HealthState::Unhealthy
        );
    }

    #[tokio::test]
    async fn test_factory_stats() {
        let factory = AdapterFactory::new().await.unwrap();

        let healthy_adapter = Arc::new(MockAdapter {
            cli_type: CLIType::Claude,
            healthy: true,
        });

        factory
            .register_adapter(CLIType::Claude, healthy_adapter)
            .await
            .unwrap();

        let stats = factory.get_factory_stats().await;

        assert_eq!(stats.total_adapters, 1);
        assert_eq!(stats.healthy_adapters, 1);
        assert_eq!(stats.warning_adapters, 0);
        assert_eq!(stats.unhealthy_adapters, 0);
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new(HealthMonitorConfig::default());

        monitor.initialize_monitoring(CLIType::Claude).await;

        let record = HealthCheckRecord {
            timestamp: chrono::Utc::now(),
            status: HealthState::Healthy,
            check_duration: Duration::from_millis(100),
            error: None,
        };

        monitor.record_health_check(CLIType::Claude, record).await;

        let history = monitor.get_health_history(CLIType::Claude).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, HealthState::Healthy);
    }
}

```

```rust
//! CLI Adapter Implementations
//!
//! Concrete implementations of the `CliAdapter` trait for different CLI tools.

pub mod claude;
pub mod codex;

// Re-export adapter implementations
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;

pub mod opencode {
    //! OpenCode CLI adapter (placeholder for future task)
    // TODO: Implement OpencodeAdapter
}

pub mod gemini {
    //! Gemini CLI adapter (placeholder for future task)
    // TODO: Implement GeminiAdapter
}

pub mod grok {
    //! Grok CLI adapter (placeholder for future task)
    // TODO: Implement GrokAdapter
}

pub mod qwen {
    //! Qwen CLI adapter (placeholder for future task)
    // TODO: Implement QwenAdapter
}

pub mod cursor {
    //! Cursor CLI adapter (placeholder for future task)
    // TODO: Implement CursorAdapter
}

pub mod openhands {
    //! OpenHands CLI adapter (placeholder for future task)
    // TODO: Implement OpenHandsAdapter
}

```

```rust
*** Begin Patch
*** Update File: controller/src/cli/adapters/mod.rs
@@
 pub use claude::ClaudeAdapter;
 pub use codex::CodexAdapter;
 
-// Stub modules for future implementations
-pub mod codex {
-    //! Codex CLI adapter (placeholder for Task 4)
-    // TODO: Implement CodexAdapter in Task 4
-}
-
 pub mod opencode {
     //! OpenCode CLI adapter (placeholder for future task)
     // TODO: Implement OpencodeAdapter
 }
*** End Patch
```

```rust
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


```

```rust
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



```

```rust
//! Base Adapter Implementation
//!
//! Provides shared functionality for all CLI adapters including logging, metrics collection,
//! template rendering, and common utilities.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, ContainerContext, HealthState, HealthStatus,
};
use crate::cli::types::CLIType;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output,
    RenderContext,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Operation context for tracing
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub cli_type: CLIType,
    pub correlation_id: String,
    pub start_time: Instant,
}

/// Configuration for BaseAdapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Root directory for CLI templates
    pub template_root: PathBuf,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Metrics prefix
    pub metrics_prefix: String,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl AdapterConfig {
    pub fn new(cli_type: CLIType) -> Self {
        let default_template_root = std::env::var("CLI_TEMPLATES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/agent-templates"));
        Self {
            cli_type,
            correlation_id: Uuid::new_v4().to_string(),
            template_root: default_template_root,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(10),
            metrics_prefix: format!("cli_adapter_{cli_type}"),
            verbose_logging: false,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    pub fn with_template_root<P: Into<PathBuf>>(mut self, template_root: P) -> Self {
        self.template_root = template_root.into();
        self
    }

    pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }
}

/// Base adapter providing shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// OpenTelemetry metrics
    pub metrics: Arc<AdapterMetrics>,
    /// Handlebars template engine
    pub templates: Arc<Handlebars<'static>>,
    /// OpenTelemetry tracer (simplified)
    pub tracer: Option<String>,
}

impl BaseAdapter {
    /// Create new base adapter
    pub async fn new(config: AdapterConfig) -> AdapterResult<Self> {
        let metrics = Arc::new(AdapterMetrics::new(&config.metrics_prefix)?);
        let mut templates = Handlebars::new();

        // Register custom helpers
        Self::register_template_helpers(&mut templates);

        let tracer = Some("cli_adapter".to_string());

        Ok(Self {
            cli_type: config.cli_type,
            config,
            metrics,
            templates: Arc::new(templates),
            tracer,
        })
    }

    /// Log structured operation with correlation ID and context
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        let mut log_context = context.clone();
        log_context.insert("cli_type".to_string(), self.cli_type.to_string());
        log_context.insert(
            "correlation_id".to_string(),
            self.config.correlation_id.clone(),
        );

        info!(
            operation = %operation,
            cli_type = %self.cli_type,
            correlation_id = %self.config.correlation_id,
            context = ?log_context,
            "CLI adapter operation"
        );
    }

    /// Record metrics with OpenTelemetry
    #[instrument(skip(self))]
    pub async fn record_metrics(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
    ) -> AdapterResult<()> {
        let labels = [
            KeyValue::new("cli_type", self.cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
            KeyValue::new("correlation_id", self.config.correlation_id.clone()),
        ];

        // Record operation count
        self.metrics.operations_total.add(1, &labels);

        // Record operation duration
        self.metrics
            .operation_duration
            .record(duration.as_millis() as f64, &labels);

        // Record failures
        if !success {
            self.metrics.operation_failures.add(1, &labels);
        }

        debug!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter metrics"
        );

        Ok(())
    }

    /// Validate base configuration common to all adapters
    #[instrument(skip(self, config))]
    pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
        // Validate required fields
        if config.github_app.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "GitHub app cannot be empty".to_string(),
            ));
        }

        if config.cli.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "CLI type cannot be empty".to_string(),
            ));
        }

        if config.model.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "Model cannot be empty".to_string(),
            ));
        }

        // Validate CLI type matches adapter
        let expected_cli = self.cli_type.to_string();
        if config.cli != expected_cli {
            return Err(AdapterError::ValidationError(format!(
                "CLI type mismatch: expected '{}', got '{}'",
                expected_cli, config.cli
            )));
        }

        // Validate optional fields
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 || max_tokens > 1_000_000 {
                return Err(AdapterError::ValidationError(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ValidationError(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        info!(
            github_app = %config.github_app,
            cli = %config.cli,
            model = %config.model,
            "Base configuration validation passed"
        );

        Ok(())
    }

    /// Render template with Handlebars and context
    #[instrument(skip(self, template_name, context))]
    pub fn render_template(&self, template_name: &str, context: &Value) -> AdapterResult<String> {
        let start_time = Instant::now();

        // Add base context
        let mut full_context = context.clone();
        if let Value::Object(ref mut map) = full_context {
            map.insert(
                "cli_type".to_string(),
                Value::String(self.cli_type.to_string()),
            );
            map.insert(
                "correlation_id".to_string(),
                Value::String(self.config.correlation_id.clone()),
            );
            map.insert(
                "timestamp".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let template_content = self.load_template(template_name)?;

        let result = self
            .templates
            .render_template(&template_content, &full_context)
            .map_err(|e| {
                error!(error = %e, "Template rendering failed");
                AdapterError::TemplateError(format!("Template rendering failed: {e}"))
            })?;

        let duration = start_time.elapsed();
        debug!(
            template_length = template_content.len(),
            result_length = result.len(),
            duration_ms = duration.as_millis(),
            "Template rendered successfully"
        );

        Ok(result)
    }

    /// Load a template from the configured template root
    pub fn load_template(&self, relative_path: &str) -> AdapterResult<String> {
        let path = self.config.template_root.join(relative_path);
        let content = fs::read_to_string(&path).map_err(|e| {
            AdapterError::TemplateError(format!(
                "Failed to load template {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(content)
    }

    /// Register custom template helpers
    fn register_template_helpers(hb: &mut Handlebars<'static>) {
        // Helper for JSON serialization
        handlebars_helper!(json: |obj: Value| {
            serde_json::to_string(&obj).unwrap_or_else(|_| "null".to_string())
        });

        // Helper for CLI-specific formatting
        handlebars_helper!(cli_format: |cli_type: str, content: str| {
            match cli_type {
                "claude" => format!("# Claude Configuration\n\n{content}"),
                "codex" => format!("# Codex Configuration\n{content}"),
                _ => content.to_string(),
            }
        });

        // Helper for timestamp formatting
        fn timestamp_helper(
            _: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let timestamp = chrono::Utc::now().to_rfc3339();
            out.write(&timestamp)?;
            Ok(())
        }

        // Helper for environment variable substitution
        fn env_helper(
            h: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let var_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

            let value = std::env::var(var_name).unwrap_or_else(|_| format!("${{{var_name}}}"));

            out.write(&value)?;
            Ok(())
        }

        // Helper for conditional CLI features
        handlebars_helper!(if_cli_supports: |cli_type: str, feature: str, then_val: Value, else_val: Value| {
            let supports = match (cli_type, feature) {
                ("claude", "streaming") => true,
                ("claude", "multimodal") => false,
                ("codex", "streaming") => false,
                ("codex", "toml_config") => true,
                ("gemini", "multimodal") => true,
                _ => false,
            };

            if supports { then_val } else { else_val }
        });

        hb.register_helper("json", Box::new(json));
        hb.register_helper("cli_format", Box::new(cli_format));
        hb.register_helper("timestamp", Box::new(timestamp_helper));
        hb.register_helper("env", Box::new(env_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports));
    }

    /// Create operation context for tracing
    #[instrument(skip(self))]
    pub fn create_operation_context(&self, operation: &str) -> OperationContext {
        OperationContext {
            operation: operation.to_string(),
            cli_type: self.cli_type,
            correlation_id: self.config.correlation_id.clone(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform common health checks
    #[instrument(skip(self, container))]
    pub async fn base_health_check(
        &self,
        container: &ContainerContext,
    ) -> AdapterResult<HealthStatus> {
        let _ctx = self.create_operation_context("health_check");
        let start_time = Instant::now();
        let mut health_details = HashMap::new();

        // Check adapter configuration
        health_details.insert(
            "config_valid".to_string(),
            json!(self.config.cli_type == self.cli_type),
        );

        // Check template engine
        let template_test = self.render_template("test: {{cli_type}}", &json!({}));
        health_details.insert(
            "templates_working".to_string(),
            json!(template_test.is_ok()),
        );

        // Check metrics collection
        health_details.insert(
            "metrics_available".to_string(),
            json!(true), // Always true since we initialize metrics
        );

        // Check container context if provided
        if let Some(pod) = &container.pod {
            health_details.insert(
                "pod_available".to_string(),
                json!(pod.metadata.name.is_some()),
            );
        }

        let duration = start_time.elapsed();
        health_details.insert("check_duration_ms".to_string(), json!(duration.as_millis()));

        // Determine overall health
        let is_healthy = template_test.is_ok() && duration < self.config.health_check_timeout;

        let status = if is_healthy {
            HealthState::Healthy
        } else {
            HealthState::Warning
        };

        let health = HealthStatus {
            status,
            message: if is_healthy {
                None
            } else {
                Some("Some health checks failed".to_string())
            },
            checked_at: chrono::Utc::now(),
            details: health_details,
        };

        // Operation completed

        info!(
            status = ?health.status,
            duration_ms = duration.as_millis(),
            "Base health check completed"
        );

        Ok(health)
    }

    /// Common initialization tasks
    #[instrument(skip(self, container))]
    pub async fn base_initialize(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("initialize");

        info!(
            container_name = %container.container_name,
            working_dir = %container.working_dir,
            namespace = %container.namespace,
            env_vars_count = container.env_vars.len(),
            "Initializing base adapter"
        );

        // Validate container context
        if container.container_name.is_empty() {
            // Container name validation failed
            return Err(AdapterError::InitializationError(
                "Container name cannot be empty".to_string(),
            ));
        }

        if container.working_dir.is_empty() {
            // Working directory validation failed
            return Err(AdapterError::InitializationError(
                "Working directory cannot be empty".to_string(),
            ));
        }

        // Log environment variables for debugging (without values for security)
        if self.config.verbose_logging {
            let env_keys: Vec<&String> = container.env_vars.keys().collect();
            debug!(env_keys = ?env_keys, "Container environment variables");
        }

        // Operation completed

        info!("Base adapter initialization completed successfully");
        Ok(())
    }

    /// Common cleanup tasks
    #[instrument(skip(self, container))]
    pub async fn base_cleanup(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("cleanup");

        info!(
            container_name = %container.container_name,
            "Starting base adapter cleanup"
        );

        // Flush any pending metrics
        // Note: OpenTelemetry metrics are typically handled by the SDK
        debug!("Flushed pending metrics");

        // Log cleanup completion
        info!("Base adapter cleanup completed");

        // Operation completed

        Ok(())
    }

    /// Get adapter configuration summary for diagnostics
    pub fn get_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert("cli_type".to_string(), json!(self.cli_type.to_string()));
        summary.insert(
            "correlation_id".to_string(),
            json!(self.config.correlation_id),
        );
        summary.insert(
            "template_cache_size".to_string(),
            json!(self.config.template_cache_size),
        );
        summary.insert(
            "health_check_timeout_ms".to_string(),
            json!(self.config.health_check_timeout.as_millis()),
        );
        summary.insert(
            "metrics_prefix".to_string(),
            json!(self.config.metrics_prefix),
        );
        summary.insert(
            "verbose_logging".to_string(),
            json!(self.config.verbose_logging),
        );

        summary
    }
}

/// OpenTelemetry metrics for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Counter for total operations
    pub operations_total: Counter<u64>,
    /// Counter for operation failures
    pub operation_failures: Counter<u64>,
    /// Histogram for operation duration
    pub operation_duration: Histogram<f64>,
}

impl AdapterMetrics {
    /// Create new metrics instance
    pub fn new(prefix: &str) -> AdapterResult<Self> {
        let meter = global::meter("cli_adapter");

        let operations_total = meter
            .u64_counter(format!("{prefix}_operations_total"))
            .with_description("Total number of adapter operations")
            .build();

        let operation_failures = meter
            .u64_counter(format!("{prefix}_operation_failures"))
            .with_description("Number of failed adapter operations")
            .build();

        let operation_duration = meter
            .f64_histogram(format!("{prefix}_operation_duration_ms"))
            .with_description("Duration of adapter operations in milliseconds")
            .build();

        Ok(Self {
            operations_total,
            operation_failures,
            operation_duration,
        })
    }

    /// Record operation metrics
    pub fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let labels = [
            KeyValue::new("cli_type", cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
        ];

        self.operations_total.add(1, &labels);
        self.operation_duration
            .record(duration.as_millis() as f64, &labels);

        if !success {
            self.operation_failures.add(1, &labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(!adapter.config.correlation_id.is_empty());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new(CLIType::Codex)
            .with_correlation_id("test-123".to_string())
            .with_verbose_logging(true);

        assert_eq!(config.cli_type, CLIType::Codex);
        assert_eq!(config.correlation_id, "test-123");
        assert!(config.verbose_logging);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let template = "Hello {{name}}, CLI: {{cli_type}}";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("CLI: claude"));
    }

    #[tokio::test]
    async fn test_base_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid config
        let valid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid config - empty model
        let invalid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let health = adapter.base_health_check(&container).await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[test]
    fn test_template_helpers() {
        let mut hb = Handlebars::new();
        BaseAdapter::register_template_helpers(&mut hb);

        // Test JSON helper
        let template = "{{json data}}";
        let context = json!({"data": {"key": "value"}});
        let result = hb.render_template(template, &context).unwrap();
        // JSON helper should output valid JSON (formatting may vary)
        assert!(result.contains("key") && result.contains("value"));

        // Test timestamp helper
        let template = "Generated at: {{timestamp}}";
        let result = hb.render_template(template, &json!({})).unwrap();
        assert!(result.contains("Generated at:"));
    }

    #[tokio::test]
    async fn test_config_summary() {
        let config =
            AdapterConfig::new(CLIType::Gemini).with_correlation_id("test-456".to_string());

        let adapter = BaseAdapter::new(config).await.unwrap();
        let summary = adapter.get_config_summary();

        assert_eq!(summary["cli_type"], json!("gemini"));
        assert_eq!(summary["correlation_id"], json!("test-456"));
        assert!(summary.contains_key("health_check_timeout_ms"));
    }

    #[tokio::test]
    async fn test_base_initialization_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid container context
        let valid_container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&valid_container).await.is_ok());

        // Invalid container context - empty name
        let invalid_container = ContainerContext {
            pod: None,
            container_name: "".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&invalid_container).await.is_err());
    }
}

```

---

**User**

I think probably the most prudent thing would just be for us to have a brand new set of templates per each CLI, that's just going to make things a lot easier.

---

**Cursor**

Totally agree—per-CLI template sets will keep things clean. I’ll carve out a dedicated `codex/` subtree under `infra/charts/controller/agent-templates/`, parallel to `code/`, and drop in the Codex-specific assets there (config TOML, AGENTS.md, container script, etc.). Controller-side loading will point at those new files so each adapter stays fully self-contained. Continuing with that approach now.

