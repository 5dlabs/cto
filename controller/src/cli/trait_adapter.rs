//! CLI Adapter Trait System
//!
//! This module provides the core abstraction layer that unifies interactions with
//! different CLI providers while preserving their unique capabilities.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

/// Core CLI adapter trait that abstracts CLI operations
#[async_trait]
pub trait CliAdapter: Send + Sync + Debug {
    /// Validate if a model name is supported by this CLI
    async fn validate_model(&self, model: &str) -> Result<bool, AdapterError>;

    /// Generate CLI-specific configuration from agent configuration
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String, AdapterError>;

    /// Format a prompt according to CLI-specific requirements
    fn format_prompt(&self, prompt: &str) -> String;

    /// Parse CLI response into standardized format
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse, AdapterError>;

    /// Get the memory filename pattern for this CLI
    fn get_memory_filename(&self) -> &str;

    /// Get the executable name for this CLI
    fn get_executable_name(&self) -> &str;

    /// Get CLI capabilities and limitations
    fn get_capabilities(&self) -> CliCapabilities;

    /// Initialize the CLI adapter in a container environment
    async fn initialize(&self, container: &dyn Container) -> Result<(), AdapterError>;

    /// Cleanup resources when done
    async fn cleanup(&self, container: &dyn Container) -> Result<(), AdapterError>;

    /// Check health status of the CLI
    async fn health_check(&self) -> Result<HealthStatus, AdapterError>;
}

/// Agent configuration for CLI adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Model identifier
    pub model: String,
    /// Maximum tokens for response
    pub max_tokens: Option<u32>,
    /// Temperature setting (0.0 to 2.0)
    pub temperature: Option<f64>,
    /// Tool configuration
    pub tools: ToolConfiguration,
    /// System prompt or instructions
    pub system_prompt: Option<String>,
    /// CLI-specific configuration
    pub cli_config: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-3-5-sonnet".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: ToolConfiguration::default(),
            system_prompt: None,
            cli_config: HashMap::new(),
        }
    }
}

/// Tool configuration for the agent
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfiguration {
    /// Remote tools available
    pub remote: Vec<String>,
    /// Local server configurations
    pub local_servers: HashMap<String, LocalServerConfig>,
    /// MCP server configurations
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// Local server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerConfig {
    /// Whether the server is enabled
    pub enabled: bool,
    /// Tools provided by this server
    pub tools: Vec<String>,
    /// Server-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Command to run the MCP server
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory
    pub working_dir: Option<String>,
}

/// Parsed response from CLI
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedResponse {
    /// Main response content
    pub content: String,
    /// Tool calls made during response
    pub tool_calls: Vec<ToolCall>,
    /// Response metadata
    pub metadata: ResponseMetadata,
    /// Reason why response finished
    pub finish_reason: FinishReason,
    /// Streaming deltas if applicable
    pub streaming_delta: Option<StreamingDelta>,
}

/// Tool call information
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    /// Tool identifier
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool parameters
    pub parameters: serde_json::Value,
    /// Tool result if available
    pub result: Option<ToolResult>,
}

/// Tool execution result
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    /// Result content
    pub content: String,
    /// Whether the tool succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Response metadata
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseMetadata {
    /// Response ID or correlation ID
    pub id: Option<String>,
    /// Token usage information
    pub usage: Option<TokenUsage>,
    /// Model used for generation
    pub model: Option<String>,
    /// Response timing
    pub timing: Option<ResponseTiming>,
}

/// Token usage information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenUsage {
    /// Input tokens consumed
    pub input_tokens: u32,
    /// Output tokens generated
    pub output_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Response timing information
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseTiming {
    /// Time to first token
    pub time_to_first_token: Option<Duration>,
    /// Total response time
    pub total_time: Duration,
}

/// Reason why response finished
#[derive(Debug, Clone, PartialEq)]
pub enum FinishReason {
    /// Natural completion
    Stop,
    /// Hit token limit
    Length,
    /// Tool calls required
    ToolCall,
    /// Content filtered
    ContentFilter,
    /// Error occurred
    Error,
}

/// Streaming delta for partial responses
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingDelta {
    /// Incremental content
    pub content: String,
    /// Whether this is the final delta
    pub is_final: bool,
    /// Delta index
    pub index: u32,
}

/// CLI capabilities and limitations
#[derive(Debug, Clone, PartialEq)]
pub struct CliCapabilities {
    /// Supports streaming responses
    pub supports_streaming: bool,
    /// Supports multimodal inputs (images, audio)
    pub supports_multimodal: bool,
    /// Supports function/tool calling
    pub supports_function_calling: bool,
    /// Supports system prompts
    pub supports_system_prompts: bool,
    /// Maximum context window size
    pub max_context_tokens: u32,
    /// Memory persistence strategy
    pub memory_strategy: MemoryStrategy,
    /// Configuration file format
    pub config_format: ConfigFormat,
    /// Authentication methods supported
    pub authentication_methods: Vec<AuthMethod>,
}

/// Memory persistence strategies
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryStrategy {
    /// Single markdown file (e.g., CLAUDE.md)
    MarkdownFile(String),
    /// Subdirectory with files (e.g., .grok/GROK.md)
    Subdirectory(String),
    /// Session-based in-memory only
    SessionBased,
    /// Configuration file based
    ConfigurationBased,
}

/// Configuration file formats
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Markdown format
    Markdown,
    /// Custom format
    Custom(String),
}

/// Authentication methods
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    /// Session token based
    SessionToken,
    /// API key based
    ApiKey,
    /// OAuth flow
    OAuth,
    /// Username/password
    UserPassword,
    /// Custom authentication
    Custom(String),
}

/// Container environment interface
#[async_trait]
pub trait Container: Send + Sync + Debug {
    /// Get container ID
    fn id(&self) -> &str;
    /// Get working directory
    fn working_dir(&self) -> &str;
    /// Get environment variables
    fn env_vars(&self) -> &HashMap<String, String>;
    /// Execute command in container
    async fn execute(&self, command: &[String]) -> Result<ExecutionResult, String>;
    /// Create file in container
    async fn create_file(&self, path: &str, content: &str) -> Result<(), String>;
    /// Read file from container
    async fn read_file(&self, path: &str) -> Result<String, String>;
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration: Duration,
}

/// Health check status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Adapter is healthy
    Healthy,
    /// Adapter is degraded but functional
    Degraded(String),
    /// Adapter is unhealthy
    Unhealthy(String),
}

/// Adapter-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Model validation failed: {0}")]
    ModelValidation(String),

    #[error("Configuration generation failed: {0}")]
    ConfigGeneration(String),

    #[error("Response parsing failed: {0}")]
    ResponseParsing(String),

    #[error("Container operation failed: {0}")]
    ContainerOperation(String),

    #[error("Health check failed: {0}")]
    HealthCheck(String),

    #[error("Initialization failed: {0}")]
    Initialization(String),

    #[error("Cleanup failed: {0}")]
    Cleanup(String),

    #[error("Template rendering failed: {0}")]
    TemplateRendering(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Generic adapter error: {0}")]
    Generic(String),
}

impl AdapterError {
    /// Create a new generic adapter error
    pub fn new(message: impl Into<String>) -> Self {
        Self::Generic(message.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AdapterError::ContainerOperation(_)
                | AdapterError::HealthCheck(_)
                | AdapterError::TemplateRendering(_)
                | AdapterError::Io(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.model, "claude-3-5-sonnet");
        assert_eq!(config.max_tokens, Some(4096));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_parsed_response_creation() {
        let response = ParsedResponse {
            content: "Test response".to_string(),
            tool_calls: vec![],
            metadata: ResponseMetadata {
                id: Some("test-id".to_string()),
                usage: None,
                model: Some("test-model".to_string()),
                timing: None,
            },
            finish_reason: FinishReason::Stop,
            streaming_delta: None,
        };

        assert_eq!(response.content, "Test response");
        assert_eq!(response.finish_reason, FinishReason::Stop);
    }

    #[test]
    fn test_cli_capabilities() {
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

        assert!(caps.supports_streaming);
        assert!(!caps.supports_multimodal);
        assert_eq!(caps.max_context_tokens, 200_000);
    }

    #[test]
    fn test_adapter_error_recoverable() {
        let recoverable_error = AdapterError::ContainerOperation("Test error".to_string());
        assert!(recoverable_error.is_recoverable());

        let non_recoverable_error = AdapterError::ModelValidation("Invalid model".to_string());
        assert!(!non_recoverable_error.is_recoverable());
    }

    #[test]
    fn test_memory_strategy() {
        let claude_strategy = MemoryStrategy::MarkdownFile("CLAUDE.md".to_string());
        let grok_strategy = MemoryStrategy::Subdirectory(".grok".to_string());
        let session_strategy = MemoryStrategy::SessionBased;

        match claude_strategy {
            MemoryStrategy::MarkdownFile(filename) => {
                assert_eq!(filename, "CLAUDE.md");
            }
            _ => panic!("Expected MarkdownFile strategy"),
        }

        match grok_strategy {
            MemoryStrategy::Subdirectory(dir) => {
                assert_eq!(dir, ".grok");
            }
            _ => panic!("Expected Subdirectory strategy"),
        }

        match session_strategy {
            MemoryStrategy::SessionBased => {}
            _ => panic!("Expected SessionBased strategy"),
        }
    }
}