//! CLI Adapter Trait System
//!
//! Core abstraction layer providing unified CLI interactions across 8 different CLI tools.
//! This trait system enables consistent behavior while preserving each CLI's unique capabilities.

use crate::cli::types::{CLIExecutionContext, CLIExecutionResult, CLIType, ConfigFile};
use anyhow::Result;
use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// Core CLI abstraction trait providing unified interface for all CLI providers
#[async_trait]
pub trait CliAdapter: Send + Sync + Debug {
    /// Validate model name for this CLI type
    async fn validate_model(&self, model: &str) -> Result<bool>;

    /// Generate CLI-specific configuration from agent config
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String>;

    /// Format prompt for this CLI's requirements
    fn format_prompt(&self, prompt: &str) -> String;

    /// Parse CLI response into structured format
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse>;

    /// Get CLI-specific memory filename (e.g., "CLAUDE.md", "AGENTS.md")
    fn get_memory_filename(&self) -> &str;

    /// Get CLI executable name
    fn get_executable_name(&self) -> &str;

    /// Get CLI capabilities and limitations
    fn get_capabilities(&self) -> CliCapabilities;

    /// Initialize adapter for container execution
    async fn initialize(&self, container: &ContainerContext) -> Result<()>;

    /// Cleanup adapter resources after execution
    async fn cleanup(&self, container: &ContainerContext) -> Result<()>;

    /// Check adapter and CLI health status
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// Container context for adapter operations
#[derive(Debug, Clone)]
pub struct ContainerContext {
    /// Kubernetes pod reference
    pub pod: Option<Pod>,
    /// Container name within pod
    pub container_name: String,
    /// Working directory path
    pub working_dir: String,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Container namespace
    pub namespace: String,
}

/// Parsed response from CLI execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedResponse {
    /// Main response content
    pub content: String,
    /// Tool/function calls made during response
    pub tool_calls: Vec<ToolCall>,
    /// Response metadata (tokens, timing, etc.)
    pub metadata: ResponseMetadata,
    /// Reason for response completion
    pub finish_reason: FinishReason,
    /// Streaming delta if applicable
    pub streaming_delta: Option<StreamingDelta>,
}

/// Tool/function call representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
    /// Tool call ID for tracking
    pub id: Option<String>,
}

/// Response metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResponseMetadata {
    /// Input tokens consumed
    pub input_tokens: Option<u32>,
    /// Output tokens generated
    pub output_tokens: Option<u32>,
    /// Response generation time
    pub duration_ms: Option<u64>,
    /// Model used for generation
    pub model: Option<String>,
    /// Additional CLI-specific metadata
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response completion reasons
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Response completed normally
    Stop,
    /// Hit token/length limit
    Length,
    /// Function/tool call completed
    ToolCall,
    /// Content filtered
    ContentFilter,
    /// Error occurred
    Error,
    /// Response incomplete/interrupted
    Incomplete,
}

/// Streaming response delta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingDelta {
    /// Content delta
    pub content: Option<String>,
    /// Tool call delta
    pub tool_call_delta: Option<ToolCall>,
    /// Whether this is the final delta
    pub is_final: bool,
}

/// CLI-specific capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Appropriate for capabilities struct
pub struct CliCapabilities {
    /// Supports streaming responses
    pub supports_streaming: bool,
    /// Supports multimodal inputs (images, audio)
    pub supports_multimodal: bool,
    /// Supports function/tool calling
    pub supports_function_calling: bool,
    /// Supports system prompts
    pub supports_system_prompts: bool,
    /// Maximum context tokens
    pub max_context_tokens: u32,
    /// Memory strategy for persistence
    pub memory_strategy: MemoryStrategy,
    /// Configuration file format
    pub config_format: ConfigFormat,
    /// Supported authentication methods
    pub authentication_methods: Vec<AuthMethod>,
}

/// Memory persistence strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryStrategy {
    /// Single markdown file (Claude: CLAUDE.md, Codex: AGENTS.md)
    MarkdownFile(String),
    /// Subdirectory with multiple files (Grok: .grok/GROK.md)
    Subdirectory(String),
    /// Session-based memory (Cursor, `OpenHands`)
    SessionBased,
    /// Configuration-based persistence
    ConfigurationBased,
}

/// Configuration file formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format (Codex)
    Toml,
    /// YAML format
    Yaml,
    /// Markdown format (Claude)
    Markdown,
    /// Custom/proprietary format
    Custom(String),
}

/// Authentication methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Session token (Claude)
    SessionToken,
    /// API key (`OpenAI`, Anthropic)
    ApiKey,
    /// OAuth flow (Google)
    OAuth,
    /// No authentication needed
    None,
    /// Custom authentication
    Custom(String),
}

/// Health status for adapters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: HealthState,
    /// Status message
    pub message: Option<String>,
    /// Last check timestamp
    pub checked_at: chrono::DateTime<chrono::Utc>,
    /// Additional health details
    pub details: HashMap<String, serde_json::Value>,
}

/// Health state enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthState {
    /// Adapter is healthy
    Healthy,
    /// Adapter has warnings but is functional
    Warning,
    /// Adapter is unhealthy
    Unhealthy,
    /// Adapter health is unknown
    Unknown,
}

/// Agent configuration for CLI adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// GitHub app identifier
    pub github_app: String,
    /// CLI type to use
    pub cli: String,
    /// Model name/identifier
    pub model: String,
    /// Maximum tokens for generation
    pub max_tokens: Option<u32>,
    /// Temperature for generation
    pub temperature: Option<f32>,
    /// Tool configuration
    pub tools: Option<ToolConfiguration>,
    /// Additional CLI-specific config
    pub cli_config: Option<serde_json::Value>,
}

/// Tool configuration for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfiguration {
    /// Remote tools available
    pub remote: Vec<String>,
    /// Local server configurations
    pub local_servers: Option<HashMap<String, LocalServerConfig>>,
}

/// Local server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerConfig {
    /// Whether server is enabled
    pub enabled: bool,
    /// Tools provided by this server
    pub tools: Vec<String>,
}

/// CLI execution adapter (legacy compatibility)
pub struct CLIExecutionAdapter {
    /// CLI type this adapter handles
    cli_type: CLIType,
}

impl CLIExecutionAdapter {
    /// Create a new legacy adapter for a specific CLI type
    #[must_use]
    pub fn new(cli_type: CLIType) -> Self {
        Self { cli_type }
    }

    /// Execute a CLI command with the given context (legacy method)
    ///
    /// # Errors
    /// Returns an error if command execution fails or if I/O operations fail
    pub async fn execute(&self, context: &CLIExecutionContext) -> Result<CLIExecutionResult> {
        use std::process::Stdio;
        use tokio::process::Command;

        let start_time = std::time::Instant::now();

        // Create the command
        let mut command = Command::new(&context.command[0]);
        command
            .args(&context.command[1..])
            .current_dir(&context.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &context.env_vars {
            command.env(key, value);
        }

        // Execute the command
        let output = command.output().await.map_err(|e| {
            AdapterError::ExecutionFailed(format!("Failed to execute command: {e}"))
        })?;

        let duration = start_time.elapsed();

        let result = CLIExecutionResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms: u64::try_from(duration.as_millis().min(u128::from(u64::MAX)))
                .unwrap_or(u64::MAX),
            cli_type: self.cli_type,
        };

        Ok(result)
    }

    /// Prepare files for CLI execution
    ///
    /// # Errors
    /// Returns an error if file operations fail or if paths are invalid
    pub async fn prepare_files(&self, config_files: &[ConfigFile]) -> Result<()> {
        for config_file in config_files {
            // Ensure parent directory exists
            if let Some(parent) = std::path::Path::new(&config_file.path).parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    AdapterError::FilePreparationError(format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Write the file
            tokio::fs::write(&config_file.path, &config_file.content)
                .await
                .map_err(|e| {
                    AdapterError::FilePreparationError(format!(
                        "Failed to write file {}: {}",
                        config_file.path, e
                    ))
                })?;

            // Set permissions if specified
            if let Some(perms) = &config_file.permissions {
                if let Ok(mode) = u32::from_str_radix(perms, 8) {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut permissions =
                            tokio::fs::metadata(&config_file.path).await?.permissions();
                        permissions.set_mode(mode);
                        tokio::fs::set_permissions(&config_file.path, permissions).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate CLI environment
    ///
    /// # Errors
    /// Returns an error if environment validation fails or if required variables are missing
    pub async fn validate_environment(&self, required_vars: &[String]) -> Result<Vec<String>> {
        let mut missing = Vec::new();

        for var in required_vars {
            if std::env::var(var).is_err() {
                missing.push(var.clone());
            }
        }

        if !missing.is_empty() {
            return Err(AdapterError::MissingEnvironmentVariables(missing).into());
        }

        Ok(required_vars.to_vec())
    }

    /// Get CLI-specific execution hints
    #[must_use]
    pub fn get_execution_hints(&self) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => vec![
                "CLAUDE.md will be created automatically".to_string(),
                "Claude supports MCP servers for tool integration".to_string(),
            ],
            CLIType::Codex => vec![
                "AGENTS.md will be created for project instructions".to_string(),
                "config.toml will be created for Codex settings".to_string(),
                "Requires OPENAI_API_KEY environment variable".to_string(),
                "Supports --full-auto for non-interactive execution".to_string(),
            ],
            CLIType::OpenCode => vec![
                "Creates cache directory at ~/.cache/opencode/".to_string(),
                "Supports multiple AI providers".to_string(),
                "Uses Bun runtime for execution".to_string(),
                "Requires OPENAI_API_KEY environment variable".to_string(),
            ],
            CLIType::Factory => vec![
                "AGENTS.md and factory-cli config files will be generated".to_string(),
                "Requires FACTORY_API_KEY environment variable".to_string(),
                "Supports droid exec auto-run levels for headless workflows".to_string(),
            ],
            _ => vec!["CLI execution will use default settings".to_string()],
        }
    }
}

/// CLI-specific command builder
pub struct CommandBuilder {
    cli_type: CLIType,
}

impl CommandBuilder {
    #[must_use]
    pub fn new(cli_type: CLIType) -> Self {
        Self { cli_type }
    }

    /// Build command for task execution
    #[must_use]
    pub fn build_task_command(&self, task: &str, auto_mode: bool) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => {
                vec!["claude-code".to_string(), task.to_string()]
            }
            CLIType::Codex => {
                if auto_mode {
                    vec![
                        "codex".to_string(),
                        "exec".to_string(),
                        "--full-auto".to_string(),
                        task.to_string(),
                    ]
                } else {
                    vec!["codex".to_string(), task.to_string()]
                }
            }
            CLIType::Cursor => {
                // TODO(cursor): Finalise print/force flags once Cursor adapter wiring is complete.
                vec![
                    "cursor-agent".to_string(),
                    "--print".to_string(),
                    "--force".to_string(),
                    task.to_string(),
                ]
            }
            CLIType::Factory => {
                let mut cmd = vec!["droid".to_string(), "exec".to_string()];
                if auto_mode {
                    cmd.push("--auto".to_string());
                    cmd.push("high".to_string());
                }
                cmd.push(task.to_string());
                cmd
            }
            CLIType::OpenCode => {
                vec!["opencode".to_string(), task.to_string()]
            }
            _ => vec!["echo".to_string(), format!("Task: {task}")],
        }
    }

    /// Build command for version check
    #[must_use]
    pub fn build_version_command(&self) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => vec!["claude-code".to_string(), "--version".to_string()],
            CLIType::Codex => vec!["codex".to_string(), "--version".to_string()],
            CLIType::Cursor => vec!["cursor-agent".to_string(), "--version".to_string()],
            CLIType::OpenCode => vec!["opencode".to_string(), "--version".to_string()],
            CLIType::Factory => vec!["droid".to_string(), "--version".to_string()],
            _ => vec!["echo".to_string(), "unknown".to_string()],
        }
    }

    /// Build command for help
    #[must_use]
    pub fn build_help_command(&self) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => vec!["claude-code".to_string(), "--help".to_string()],
            CLIType::Codex => vec!["codex".to_string(), "--help".to_string()],
            CLIType::Cursor => vec!["cursor-agent".to_string(), "--help".to_string()],
            CLIType::OpenCode => vec!["opencode".to_string(), "--help".to_string()],
            CLIType::Factory => vec!["droid".to_string(), "--help".to_string()],
            _ => vec!["echo".to_string(), "help not available".to_string()],
        }
    }
}

/// Result processor for CLI outputs
pub struct ResultProcessor {
    cli_type: CLIType,
}

impl ResultProcessor {
    #[must_use]
    pub fn new(cli_type: CLIType) -> Self {
        Self { cli_type }
    }

    /// Process execution result and extract key information
    #[must_use]
    pub fn process_result(&self, result: &CLIExecutionResult) -> ProcessedResult {
        let mut processed = ProcessedResult {
            success: result.success,
            exit_code: result.exit_code,
            key_outputs: vec![],
            errors: vec![],
            warnings: vec![],
            files_modified: vec![],
            cli_specific_info: std::collections::HashMap::new(),
        };

        // Extract key information based on CLI type
        match self.cli_type {
            CLIType::Claude => {
                self.process_claude_output(result, &mut processed);
            }
            CLIType::Codex => {
                self.process_codex_output(result, &mut processed);
            }
            CLIType::OpenCode => {
                self.process_opencode_output(result, &mut processed);
            }
            _ => {
                self.process_generic_output(result, &mut processed);
            }
        }

        processed
    }

    fn process_claude_output(&self, result: &CLIExecutionResult, processed: &mut ProcessedResult) {
        let _ = self;
        // Look for CLAUDE.md creation
        if result.stdout.contains("CLAUDE.md") {
            processed.files_modified.push("CLAUDE.md".to_string());
        }

        // Extract any error messages
        if result.stderr.contains("error") || result.stderr.contains("Error") {
            processed.errors.push(result.stderr.clone());
        }
    }

    fn process_codex_output(&self, result: &CLIExecutionResult, processed: &mut ProcessedResult) {
        let _ = self;
        // Look for config file creation
        if result.stdout.contains("config.toml") {
            processed
                .files_modified
                .push("~/.codex/config.toml".to_string());
        }
        if result.stdout.contains("AGENTS.md") {
            processed.files_modified.push("AGENTS.md".to_string());
        }

        // Extract execution information
        if result.stdout.contains("codex exec") {
            processed
                .cli_specific_info
                .insert("execution_mode".to_string(), "non-interactive".to_string());
        }
    }

    fn process_opencode_output(
        &self,
        result: &CLIExecutionResult,
        processed: &mut ProcessedResult,
    ) {
        let _ = self;
        // Look for cache directory usage
        if result.stdout.contains(".cache/opencode") {
            processed
                .cli_specific_info
                .insert("cache_used".to_string(), "~/.cache/opencode".to_string());
        }

        // Extract provider information if mentioned
        if result.stdout.contains("provider") {
            processed
                .cli_specific_info
                .insert("provider_info".to_string(), "multi-provider".to_string());
        }
    }

    fn process_generic_output(&self, result: &CLIExecutionResult, processed: &mut ProcessedResult) {
        let _ = self;
        // Basic error extraction
        if !result.stderr.is_empty() {
            processed.errors.push(result.stderr.clone());
        }

        // Basic success indicators
        if result.stdout.contains("success") || result.stdout.contains("complete") {
            processed
                .key_outputs
                .push("Task completed successfully".to_string());
        }
    }
}

/// Processed result with extracted information
#[derive(Debug, Clone)]
pub struct ProcessedResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub key_outputs: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub files_modified: Vec<String>,
    pub cli_specific_info: std::collections::HashMap<String, String>,
}

/// Adapter-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("File preparation failed: {0}")]
    FilePreparationError(String),

    #[error("Missing environment variables: {0:?}")]
    MissingEnvironmentVariables(Vec<String>),

    #[error("CLI validation failed: {0}")]
    ValidationError(String),

    #[error("Model validation failed for CLI {cli_type}: model '{model}' is not supported")]
    InvalidModel {
        cli_type: String,
        model: String,
        suggestions: Option<Vec<String>>,
    },

    #[error("Configuration generation failed: {0}")]
    ConfigGenerationError(String),

    #[error("Response parsing failed: {0}")]
    ResponseParsingError(String),

    #[error("Adapter initialization failed: {0}")]
    InitializationError(String),

    #[error("Health check failed: {0}")]
    HealthCheckError(String),

    #[error("Unsupported CLI type: {0}")]
    UnsupportedCliType(String),

    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlError(String),

    #[error("YAML error: {0}")]
    YamlError(String),
}

pub type AdapterResult<T> = std::result::Result<T, AdapterError>;

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: HealthState::Unknown,
            message: None,
            checked_at: chrono::Utc::now(),
            details: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_capabilities_serialization() {
        let caps = CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 200_000,
            memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::SessionToken],
        };

        let serialized = serde_json::to_string(&caps).unwrap();
        let deserialized: CliCapabilities = serde_json::from_str(&serialized).unwrap();

        assert_eq!(caps, deserialized);
    }

    #[test]
    fn test_command_builder_claude() {
        let builder = CommandBuilder::new(CLIType::Claude);
        let cmd = builder.build_task_command("implement auth", false);
        assert_eq!(cmd, vec!["claude-code", "implement auth"]);
    }

    #[test]
    fn test_parsed_response_creation() {
        let response = ParsedResponse {
            content: "Hello, world!".to_string(),
            tool_calls: vec![],
            metadata: ResponseMetadata::default(),
            finish_reason: FinishReason::Stop,
            streaming_delta: None,
        };

        assert_eq!(response.content, "Hello, world!");
        assert_eq!(response.finish_reason, FinishReason::Stop);
    }

    #[test]
    fn test_command_builder_codex_auto() {
        let builder = CommandBuilder::new(CLIType::Codex);
        let cmd = builder.build_task_command("implement auth", true);
        assert_eq!(cmd, vec!["codex", "exec", "--full-auto", "implement auth"]);
    }

    #[test]
    fn test_command_builder_codex_interactive() {
        let builder = CommandBuilder::new(CLIType::Codex);
        let cmd = builder.build_task_command("implement auth", false);
        assert_eq!(cmd, vec!["codex", "implement auth"]);
    }

    #[test]
    fn test_command_builder_cursor_print_mode() {
        let builder = CommandBuilder::new(CLIType::Cursor);
        let cmd = builder.build_task_command("implement auth", true);
        assert_eq!(
            cmd,
            vec!["cursor-agent", "--print", "--force", "implement auth"]
        );
    }

    #[test]
    fn test_version_commands() {
        let claude_builder = CommandBuilder::new(CLIType::Claude);
        let codex_builder = CommandBuilder::new(CLIType::Codex);
        let cursor_builder = CommandBuilder::new(CLIType::Cursor);

        assert_eq!(
            claude_builder.build_version_command(),
            vec!["claude-code", "--version"]
        );
        assert_eq!(
            codex_builder.build_version_command(),
            vec!["codex", "--version"]
        );
        assert_eq!(
            cursor_builder.build_version_command(),
            vec!["cursor-agent", "--version"]
        );
    }

    #[test]
    fn test_health_status_default() {
        let health = HealthStatus::default();
        assert_eq!(health.status, HealthState::Unknown);
        assert!(health.message.is_none());
    }

    #[tokio::test]
    async fn test_environment_validation() {
        let adapter = CLIExecutionAdapter::new(CLIType::Codex);

        // This will fail because NON_EXISTENT_TEST_VAR is definitely not set
        let result = adapter
            .validate_environment(&["NON_EXISTENT_TEST_VAR".to_string()])
            .await;
        assert!(result.is_err());

        // This should pass
        let result = adapter.validate_environment(&[]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_result_processor() {
        let processor = ResultProcessor::new(CLIType::Codex);

        let execution_result = CLIExecutionResult {
            success: true,
            exit_code: Some(0),
            stdout: "Created config.toml and AGENTS.md".to_string(),
            stderr: "".to_string(),
            duration_ms: 1000,
            cli_type: CLIType::Codex,
        };

        let processed = processor.process_result(&execution_result);

        assert!(processed.success);
        assert_eq!(processed.files_modified.len(), 2);
        assert!(processed
            .files_modified
            .contains(&"~/.codex/config.toml".to_string()));
        assert!(processed.files_modified.contains(&"AGENTS.md".to_string()));
    }

    #[test]
    fn test_memory_strategy_variants() {
        let claude_memory = MemoryStrategy::MarkdownFile("CLAUDE.md".to_string());
        let grok_memory = MemoryStrategy::Subdirectory(".grok".to_string());
        let session_memory = MemoryStrategy::SessionBased;

        match claude_memory {
            MemoryStrategy::MarkdownFile(filename) => assert_eq!(filename, "CLAUDE.md"),
            _ => panic!("Expected MarkdownFile variant"),
        }

        match grok_memory {
            MemoryStrategy::Subdirectory(dir) => assert_eq!(dir, ".grok"),
            _ => panic!("Expected Subdirectory variant"),
        }

        match session_memory {
            MemoryStrategy::SessionBased => {}
            _ => panic!("Expected SessionBased variant"),
        }
    }
}
