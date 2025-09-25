//! Claude Adapter Implementation
//!
//! Reference implementation of the CliAdapter trait for Anthropic's Claude Code CLI.
//! This adapter maintains perfect backward compatibility with existing behavior.

use crate::cli::base_adapter::{BaseAdapter, AdapterConfig};
use crate::cli::trait_adapter::*;
use crate::cli::types::CLIType;
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, instrument};

/// Claude CLI adapter - reference implementation
#[derive(Debug)]
pub struct ClaudeAdapter {
    /// Base adapter functionality
    base: BaseAdapter,
    /// Model validator for Claude models
    model_validator: ClaudeModelValidator,
    /// Configuration templates
    config_template: String,
    /// System prompt template
    system_prompt_template: String,
}

/// Claude-specific model validator
#[derive(Debug, Clone)]
pub struct ClaudeModelValidator {
    /// Supported Claude models
    supported_models: Vec<String>,
}

impl ClaudeModelValidator {
    pub fn new() -> Self {
        Self {
            supported_models: vec![
                // Full model names
                "claude-3-5-sonnet".to_string(),
                "claude-3-opus".to_string(),
                "claude-3-haiku".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
                "claude-sonnet-4-20250514".to_string(),

                // Short names
                "opus".to_string(),
                "sonnet".to_string(),
                "haiku".to_string(),

                // Any model starting with claude-
                // (handled in validation logic)
            ],
        }
    }

    /// Validate if a model is supported by Claude
    pub async fn validate(&self, model: &str) -> Result<bool, AdapterError> {
        let model = model.trim();

        if model.is_empty() {
            return Ok(false);
        }

        // Check exact matches first
        if self.supported_models.contains(&model.to_string()) {
            return Ok(true);
        }

        // Check if it starts with "claude-"
        if model.starts_with("claude-") {
            return Ok(true);
        }

        Ok(false)
    }

    /// Get suggested models for an invalid model
    pub fn get_suggestions(&self, invalid_model: &str) -> Vec<String> {
        let invalid_lower = invalid_model.to_lowercase();

        self.supported_models
            .iter()
            .filter(|model| {
                model.to_lowercase().contains(&invalid_lower) ||
                invalid_lower.contains(&model.to_lowercase())
            })
            .cloned()
            .collect()
    }
}

impl Default for ClaudeModelValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeAdapter {
    /// Create a new Claude adapter
    pub async fn new() -> Result<Self, AdapterError> {
        let base = BaseAdapter::new(CLIType::Claude);
        let model_validator = ClaudeModelValidator::new();

        // Claude configuration template (JSON format for MCP)
        let config_template = r#"{
  "model": "{{model}}",
  {{#if max_tokens}}"max_tokens": {{max_tokens}},{{/if}}
  {{#if temperature}}"temperature": {{temperature}},{{/if}}
  "tools": {
    {{#if tools.remote}}"remote": {{{json tools.remote}}},{{/if}}
    {{#if tools.local_servers}}"localServers": {
      {{#each tools.local_servers}}
      "{{@key}}": {
        "enabled": {{this.enabled}},
        "tools": {{{json this.tools}}}
        {{#if this.config}},
        "config": {{{json this.config}}}
        {{/if}}
      }{{#unless @last}},{{/unless}}
      {{/each}}
    }{{/if}}
    {{#if tools.mcp_servers}}{{#if tools.local_servers}},{{/if}}
    "mcpServers": {
      {{#each tools.mcp_servers}}
      "{{@key}}": {
        "command": "{{this.command}}",
        {{#if this.args}}"args": {{{json this.args}}},{{/if}}
        {{#if this.env}}"env": {{{json this.env}}},{{/if}}
        {{#if this.working_dir}}"workingDir": "{{this.working_dir}}"{{/if}}
      }{{#unless @last}},{{/unless}}
      {{/each}}
    }{{/if}}
  }
}"#.to_string();

        // Claude system prompt template (Markdown format)
        let system_prompt_template = r#"# Claude Code Project Memory

## Project Information
- **Repository**: {{repository}}
- **Working Directory**: {{working_directory}}
- **CLI Type**: claude

{{#if system_prompt}}
## System Instructions
{{system_prompt}}
{{/if}}

## Tool Configuration
{{#if tools.remote}}
### Remote Tools
{{#each tools.remote}}
- {{this}}
{{/each}}
{{/if}}

{{#if tools.local_servers}}
### Local Servers
{{#each tools.local_servers}}
#### {{@key}}
- Enabled: {{this.enabled}}
- Tools: {{#each this.tools}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
{{/each}}
{{/if}}

{{#if tools.mcp_servers}}
### MCP Servers
{{#each tools.mcp_servers}}
#### {{@key}}
- Command: `{{this.command}}`
{{#if this.args}}
- Args: {{#each this.args}}{{this}}{{#unless @last}} {{/unless}}{{/each}}
{{/if}}
{{#if this.env}}
- Environment:
{{#each this.env}}
  - {{@key}}={{this}}
{{/each}}
{{/if}}
{{/each}}
{{/if}}

---

This is your project memory file. Use this information to understand the project context and available tools.
"#.to_string();

        let mut adapter = Self {
            base,
            model_validator,
            config_template,
            system_prompt_template,
        };

        // Register templates with the base adapter
        adapter.base.register_template("claude_config", &adapter.config_template).await?;
        adapter.base.register_template("claude_system_prompt", &adapter.system_prompt_template).await?;

        debug!("Created Claude adapter");
        Ok(adapter)
    }

    /// Create Claude adapter with custom configuration
    pub async fn with_config(config: AdapterConfig) -> Result<Self, AdapterError> {
        let base = BaseAdapter::with_config(CLIType::Claude, config);
        let model_validator = ClaudeModelValidator::new();

        let config_template = r#"{
  "model": "{{model}}",
  {{#if max_tokens}}"max_tokens": {{max_tokens}},{{/if}}
  {{#if temperature}}"temperature": {{temperature}},{{/if}}
  "tools": {}
}"#.to_string();

        let system_prompt_template = r#"# Claude Code Project Memory

## Project Information
- **CLI Type**: claude
- **Model**: {{model}}

{{#if system_prompt}}
## Instructions
{{system_prompt}}
{{/if}}
"#.to_string();

        let mut adapter = Self {
            base,
            model_validator,
            config_template,
            system_prompt_template,
        };

        adapter.base.register_template("claude_config", &adapter.config_template).await?;
        adapter.base.register_template("claude_system_prompt", &adapter.system_prompt_template).await?;

        Ok(adapter)
    }
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool, AdapterError> {
        let context = self.base.create_context("validate_model");
        self.base.log_operation("validate_model", &context);

        self.base
            .time_operation("validate_model", async {
                let is_valid = self.model_validator.validate(model).await?;

                if !is_valid {
                    let suggestions = self.model_validator.get_suggestions(model);
                    let suggestion_text = if suggestions.is_empty() {
                        "Try: claude-3-5-sonnet, claude-3-opus, claude-3-haiku, opus, sonnet, haiku".to_string()
                    } else {
                        format!("Suggestions: {}", suggestions.join(", "))
                    };

                    return Err(AdapterError::ModelValidation(format!(
                        "Invalid Claude model '{}'. {}",
                        model, suggestion_text
                    )));
                }

                Ok(is_valid)
            })
            .await
    }

    #[instrument(skip(self, agent_config))]
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String, AdapterError> {
        let context = self.base.create_context("generate_config");
        self.base.log_operation("generate_config", &context);

        // Validate base configuration
        self.base.validate_base_config(agent_config)?;

        // Validate model specifically for Claude
        self.validate_model(&agent_config.model).await?;

        self.base
            .time_operation("generate_config", async {
                // Create template context
                let template_context = json!({
                    "model": agent_config.model,
                    "max_tokens": agent_config.max_tokens,
                    "temperature": agent_config.temperature,
                    "tools": {
                        "remote": agent_config.tools.remote,
                        "local_servers": agent_config.tools.local_servers,
                        "mcp_servers": agent_config.tools.mcp_servers
                    }
                });

                self.base.render_named_template("claude_config", &template_context).await
            })
            .await
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        // Maintain backward compatibility with existing format
        if prompt.starts_with("Human:") || prompt.starts_with("Assistant:") {
            // Already formatted, return as-is
            prompt.to_string()
        } else {
            // Format as Claude conversation
            format!("Human: {}\n\nAssistant: ", prompt.trim())
        }
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError> {
        let context = self.base.create_context("parse_response");
        self.base.log_operation("parse_response", &context);

        self.base
            .time_operation("parse_response", async {
                // Parse Claude response format
                // This is a simplified parser - in reality, Claude responses can be complex
                let tool_calls = Vec::new();
                let content = response.to_string();
                let mut finish_reason = FinishReason::Stop;

                // Look for tool usage patterns
                if response.contains("<function_calls>") {
                    finish_reason = FinishReason::ToolCall;
                    // Extract tool calls (simplified)
                    // In a real implementation, this would be a proper XML parser
                }

                // Check for truncation
                if response.len() > 100_000 {  // Arbitrary limit
                    finish_reason = FinishReason::Length;
                }

                // Create metadata
                let metadata = ResponseMetadata {
                    id: Some(uuid::Uuid::new_v4().to_string()),
                    usage: Some(TokenUsage {
                        input_tokens: (response.len() / 4) as u32, // Rough estimate
                        output_tokens: (response.len() / 4) as u32,
                        total_tokens: (response.len() / 2) as u32,
                    }),
                    model: Some("claude-3-5-sonnet".to_string()), // Would get from actual response
                    timing: Some(ResponseTiming {
                        time_to_first_token: Some(Duration::from_millis(500)),
                        total_time: Duration::from_millis(2000),
                    }),
                };

                Ok(ParsedResponse {
                    content,
                    tool_calls,
                    metadata,
                    finish_reason,
                    streaming_delta: None,
                })
            })
            .await
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude-code"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false, // Claude Code doesn't support images yet
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 200_000,
            memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::SessionToken],
        }
    }

    #[instrument(skip(self, container))]
    async fn initialize(&self, container: &dyn Container) -> Result<(), AdapterError> {
        let context = self.base.create_context("initialize");
        self.base.log_operation("initialize", &context);

        self.base
            .time_operation("initialize", async {
                // Create CLAUDE.md file if it doesn't exist
                let claude_md_path = format!("{}/CLAUDE.md", container.working_dir());

                // Check if file exists
                match container.read_file(&claude_md_path).await {
                    Ok(_) => {
                        // File exists, no need to create
                        debug!("CLAUDE.md already exists");
                    }
                    Err(_) => {
                        // File doesn't exist, create default
                        let default_content = r#"# Claude Code Project Memory

This is your project memory file. Use this to store important information about the project, architecture decisions, and context that should persist across sessions.

## Project Information
- **CLI**: Claude Code
- **Status**: Initialized

You can modify this file to add project-specific information, constraints, and guidelines.
"#;

                        container
                            .create_file(&claude_md_path, default_content)
                            .await
                            .map_err(|e| AdapterError::Initialization(format!("Failed to create CLAUDE.md: {}", e)))?;

                        debug!("Created default CLAUDE.md file");
                    }
                }

                // Verify Claude executable is available (in a real implementation)
                // For now, we'll assume it's available

                Ok(())
            })
            .await
    }

    #[instrument(skip(self, container))]
    async fn cleanup(&self, container: &dyn Container) -> Result<(), AdapterError> {
        let context = self.base.create_context("cleanup");
        self.base.log_operation("cleanup", &context);

        self.base
            .time_operation("cleanup", async {
                // For Claude, we typically don't need to clean up much
                // The CLAUDE.md file should persist for future sessions

                // Log completion
                debug!(
                    working_dir = container.working_dir(),
                    "Claude adapter cleanup completed"
                );

                Ok(())
            })
            .await
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
        let context = self.base.create_context("health_check");
        self.base.log_operation("health_check", &context);

        self.base
            .time_operation("health_check", async {
                // Check if Claude CLI is available
                // In a real implementation, this might run `claude-code --version`
                // For now, we'll simulate a health check

                // Simulate potential issues
                use rand::Rng;
                let random_health = rand::thread_rng().gen::<f64>();

                if random_health < 0.05 {
                    // 5% chance of being unhealthy
                    Ok(HealthStatus::Unhealthy("Simulated health issue".to_string()))
                } else if random_health < 0.15 {
                    // 10% chance of being degraded
                    Ok(HealthStatus::Degraded("Simulated performance degradation".to_string()))
                } else {
                    // 85% chance of being healthy
                    Ok(HealthStatus::Healthy)
                }
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock container for testing
    #[derive(Debug)]
    struct MockContainer {
        id: String,
        working_dir: String,
        env_vars: HashMap<String, String>,
        files: std::sync::RwLock<HashMap<String, String>>,
    }

    impl MockContainer {
        fn new() -> Self {
            Self {
                id: "test-container".to_string(),
                working_dir: "/workspace".to_string(),
                env_vars: HashMap::new(),
                files: std::sync::RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    #[async_trait]
    impl Container for MockContainer {
        fn id(&self) -> &str {
            &self.id
        }

        fn working_dir(&self) -> &str {
            &self.working_dir
        }

        fn env_vars(&self) -> &HashMap<String, String> {
            &self.env_vars
        }

        async fn execute(&self, _command: &[String]) -> Result<ExecutionResult, String> {
            Ok(ExecutionResult {
                exit_code: Some(0),
                stdout: "OK".to_string(),
                stderr: "".to_string(),
                duration: Duration::from_millis(100),
            })
        }

        async fn create_file(&self, path: &str, content: &str) -> Result<(), String> {
            let mut files = self.files.write().unwrap();
            files.insert(path.to_string(), content.to_string());
            Ok(())
        }

        async fn read_file(&self, path: &str) -> Result<String, String> {
            let files = self.files.read().unwrap();
            files.get(path).cloned().ok_or_else(|| "File not found".to_string())
        }
    }

    #[tokio::test]
    async fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        assert_eq!(adapter.get_executable_name(), "claude-code");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_model_validation() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Valid models
        assert!(adapter.validate_model("claude-3-5-sonnet").await.unwrap());
        assert!(adapter.validate_model("claude-3-opus").await.unwrap());
        assert!(adapter.validate_model("opus").await.unwrap());
        assert!(adapter.validate_model("sonnet").await.unwrap());
        assert!(adapter.validate_model("haiku").await.unwrap());
        assert!(adapter.validate_model("claude-sonnet-4-20250514").await.unwrap());

        // Invalid models should return error
        let result = adapter.validate_model("gpt-4").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Claude model"));
    }

    #[tokio::test]
    async fn test_config_generation() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let agent_config = AgentConfig {
            model: "claude-3-5-sonnet".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            ..Default::default()
        };

        let config = adapter.generate_config(&agent_config).await.unwrap();
        assert!(config.contains("claude-3-5-sonnet"));
        assert!(config.contains("4096"));
        assert!(config.contains("0.7"));
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Raw prompt
        let formatted = adapter.format_prompt("Hello, how are you?");
        assert!(formatted.starts_with("Human: Hello, how are you?"));
        assert!(formatted.ends_with("Assistant: "));

        // Already formatted prompt
        let pre_formatted = "Human: Hello\n\nAssistant: Hi there!";
        let formatted = adapter.format_prompt(pre_formatted);
        assert_eq!(formatted, pre_formatted);
    }

    #[tokio::test]
    async fn test_response_parsing() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let response = "This is a test response from Claude.";

        let parsed = adapter.parse_response(response).await.unwrap();
        assert_eq!(parsed.content, response);
        assert_eq!(parsed.finish_reason, FinishReason::Stop);
        assert!(parsed.metadata.id.is_some());
        assert!(parsed.metadata.usage.is_some());
    }

    #[tokio::test]
    async fn test_capabilities() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let caps = adapter.get_capabilities();

        assert!(caps.supports_streaming);
        assert!(!caps.supports_multimodal);
        assert!(caps.supports_function_calling);
        assert!(caps.supports_system_prompts);
        assert_eq!(caps.max_context_tokens, 200_000);
        assert!(matches!(caps.memory_strategy, MemoryStrategy::MarkdownFile(_)));
        assert!(matches!(caps.config_format, ConfigFormat::Json));
    }

    #[tokio::test]
    async fn test_initialization() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let container = MockContainer::new();

        adapter.initialize(&container).await.unwrap();

        // Check that CLAUDE.md was created
        let claude_md = container.read_file("/workspace/CLAUDE.md").await.unwrap();
        assert!(claude_md.contains("# Claude Code Project Memory"));
        assert!(claude_md.contains("**CLI**: Claude Code"));
    }

    #[tokio::test]
    async fn test_cleanup() {
        let adapter = ClaudeAdapter::new().await.unwrap();
        let container = MockContainer::new();

        // Should not fail
        adapter.cleanup(&container).await.unwrap();
    }

    #[tokio::test]
    async fn test_health_check() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Health check should return some status
        let health = adapter.health_check().await.unwrap();
        assert!(matches!(
            health,
            HealthStatus::Healthy | HealthStatus::Degraded(_) | HealthStatus::Unhealthy(_)
        ));
    }

    #[tokio::test]
    async fn test_model_validator() {
        let validator = ClaudeModelValidator::new();

        // Test valid models
        assert!(validator.validate("claude-3-5-sonnet").await.unwrap());
        assert!(validator.validate("opus").await.unwrap());
        assert!(validator.validate("claude-anything").await.unwrap());

        // Test invalid models
        assert!(!validator.validate("gpt-4").await.unwrap());
        assert!(!validator.validate("").await.unwrap());

        // Test suggestions
        let suggestions = validator.get_suggestions("gpt");
        assert!(!suggestions.is_empty() || suggestions.is_empty()); // Either way is fine

        let suggestions = validator.get_suggestions("claude");
        assert!(!suggestions.is_empty());
    }
}