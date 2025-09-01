//! Core types for CLI-agnostic platform

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported CLI types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum CLIType {
    /// Anthropic Claude Code CLI
    Claude,
    /// OpenAI Codex CLI
    Codex,
    /// OpenCode AI CLI
    OpenCode,
    /// Cursor Agent
    Cursor,
    /// OpenHands
    OpenHands,
    /// Grok CLI
    Grok,
    /// Google Gemini CLI
    Gemini,
    /// Alibaba Qwen CLI
    Qwen,
}

impl std::fmt::Display for CLIType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CLIType::Claude => write!(f, "claude"),
            CLIType::Codex => write!(f, "codex"),
            CLIType::OpenCode => write!(f, "opencode"),
            CLIType::Cursor => write!(f, "cursor"),
            CLIType::OpenHands => write!(f, "openhands"),
            CLIType::Grok => write!(f, "grok"),
            CLIType::Gemini => write!(f, "gemini"),
            CLIType::Qwen => write!(f, "qwen"),
        }
    }
}

/// Configuration format types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// Markdown format (Claude)
    Markdown,
    /// JSON format
    JSON,
    /// YAML format
    YAML,
    /// TOML format
    TOML,
    /// Custom/proprietary format
    Custom(String),
}

/// Session persistence model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    /// Persistent sessions across runs
    Persistent,
    /// Stateless operation
    Stateless,
    /// Session files with automatic cleanup
    Ephemeral,
}

/// CLI capability profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLICapabilities {
    /// Maximum context window size
    pub max_context_window: usize,
    /// Supports tool/function calling
    pub supports_tools: bool,
    /// Supports vision/image processing
    pub supports_vision: bool,
    /// Supports web search
    pub supports_web_search: bool,
    /// Supports code execution
    pub supports_code_execution: bool,
    /// Supports file operations
    pub supports_file_operations: bool,
    /// Session persistence type
    pub session_persistence: SessionType,
}

/// CLI configuration requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIConfiguration {
    /// Configuration format
    pub config_format: ConfigFormat,
    /// Configuration file location
    pub config_location: String,
    /// Required environment variables
    pub required_env_vars: Vec<String>,
    /// Initialization commands
    pub init_commands: Vec<String>,
    /// Cleanup commands
    pub cleanup_commands: Vec<String>,
}

/// Cost model for CLI usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    /// Cost per input token
    pub input_token_cost: f64,
    /// Cost per output token
    pub output_token_cost: f64,
    /// Free tier limits
    pub free_tier_tokens: Option<usize>,
}

/// Complete CLI profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIProfile {
    /// CLI type identifier
    pub cli_type: CLIType,
    /// Human-readable name
    pub name: String,
    /// CLI capabilities
    pub capabilities: CLICapabilities,
    /// Configuration requirements
    pub configuration: CLIConfiguration,
    /// Cost model
    pub cost_model: CostModel,
    /// When this profile was discovered/updated
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Universal configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalConfig {
    /// Project context information
    pub context: ContextConfig,
    /// Available tools and functions
    pub tools: Vec<ToolDefinition>,
    /// CLI settings and preferences
    pub settings: SettingsConfig,
    /// Agent-specific configuration
    pub agent: AgentConfig,
    /// MCP server configuration
    pub mcp_config: Option<UniversalMCPConfig>,
}

/// Project context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Project name
    pub project_name: String,
    /// Project description
    pub project_description: String,
    /// Architecture overview
    pub architecture_notes: String,
    /// Project constraints and requirements
    pub constraints: Vec<String>,
}

/// Tool/function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters schema
    pub parameters: serde_json::Value,
    /// Implementation details per CLI
    pub implementations: HashMap<String, ToolImplementation>,
}

/// Tool implementation for specific CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolImplementation {
    /// CLI-specific tool name
    pub cli_tool_name: String,
    /// Parameter mapping
    pub parameter_mapping: HashMap<String, String>,
    /// Additional configuration
    pub config: serde_json::Value,
}

/// CLI settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// Model identifier
    pub model: String,
    /// Temperature setting
    pub temperature: f64,
    /// Maximum tokens
    pub max_tokens: usize,
    /// Timeout in seconds
    pub timeout: u64,
    /// Sandbox mode
    pub sandbox_mode: String,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent role/name
    pub role: String,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// Agent instructions/prompt
    pub instructions: String,
}

/// Configuration file definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// File permissions (octal)
    pub permissions: Option<String>,
}

/// CLI execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIExecutionResult {
    /// Success status
    pub success: bool,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration_ms: u64,
    /// CLI type used
    pub cli_type: CLIType,
}

/// Discovery test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// CLI type
    pub cli_type: CLIType,
    /// Test timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Test results
    pub test_results: HashMap<String, TestResult>,
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test passed
    pub passed: bool,
    /// Test output
    pub output: String,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Test duration
    pub duration_ms: u64,
}

/// CLI availability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIAvailability {
    /// Whether the CLI is available
    pub available: bool,
    /// Version string if available
    pub version: String,
    /// Error message if not available
    pub error: Option<String>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    /// Server name
    pub name: String,
    /// Command to run the server
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
}

/// Universal MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalMCPConfig {
    /// List of MCP servers
    pub servers: Vec<MCPServer>,
}

/// Bridge translation result
#[derive(Debug, Clone)]
pub struct TranslationResult {
    /// Translated configuration content
    pub content: String,
    /// Required config files
    pub config_files: Vec<ConfigFile>,
    /// Environment variables needed
    pub env_vars: Vec<String>,
}

/// CLI execution context
#[derive(Debug, Clone)]
pub struct CLIExecutionContext {
    /// CLI type to use
    pub cli_type: CLIType,
    /// Working directory
    pub working_dir: String,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Configuration files to create
    pub config_files: Vec<ConfigFile>,
    /// Command to execute
    pub command: Vec<String>,
}
