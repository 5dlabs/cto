//! CLI-based AI provider implementation.
//!
//! Uses CLI adapters from the shared `cli` crate to interact with various
//! AI coding assistants (Claude, Codex, Cursor, Factory, OpenCode, Gemini).

use async_trait::async_trait;
use cli::{AdapterFactory, CLIType, CliAdapter};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::errors::{TasksError, TasksResult};

use super::provider::{
    AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage, DEFAULT_THINKING_BUDGET,
};

/// Default model for CLI-based generation
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Default model for extended thinking (Opus 4)
pub const DEFAULT_THINKING_MODEL: &str = "claude-opus-4-1-20250805";

/// CLI-based AI provider that uses CLI adapters for text generation.
///
/// This provider executes AI CLI tools (claude, codex, cursor, etc.) as
/// subprocesses and parses their output, rather than calling APIs directly.
pub struct CLITextGenerator {
    /// The CLI type to use
    cli_type: CLIType,
    /// The underlying CLI adapter
    adapter: Arc<dyn CliAdapter>,
    /// Whether the CLI is available
    is_available: bool,
    /// Optional MCP config file or JSON for tool access
    mcp_config: Option<String>,
    /// Enable extended thinking by default
    extended_thinking: bool,
    /// Default thinking budget
    thinking_budget: u32,
}

impl CLITextGenerator {
    /// Create a new CLI text generator with the specified CLI type.
    pub fn new(cli_type: CLIType) -> TasksResult<Self> {
        let adapter = AdapterFactory::create(cli_type)
            .map_err(|e| TasksError::Ai(format!("Failed to create CLI adapter: {e}")))?;

        // Check if the CLI is available
        let is_available = Self::check_cli_available(cli_type);

        // Try to find default MCP config
        let mcp_config = Self::find_default_mcp_config();

        Ok(Self {
            cli_type,
            adapter,
            is_available,
            mcp_config,
            extended_thinking: false,
            thinking_budget: DEFAULT_THINKING_BUDGET,
        })
    }

    /// Create with extended thinking enabled by default.
    pub fn with_extended_thinking(
        cli_type: CLIType,
        thinking_budget: Option<u32>,
    ) -> TasksResult<Self> {
        let mut provider = Self::new(cli_type)?;
        provider.extended_thinking = true;
        provider.thinking_budget = thinking_budget.unwrap_or(DEFAULT_THINKING_BUDGET);
        Ok(provider)
    }

    /// Create for benchmarking/testing without MCP config.
    ///
    /// This avoids MCP tools that could cause the model to explain rather than
    /// return structured JSON output.
    pub fn for_benchmark(cli_type: CLIType, extended_thinking: bool) -> TasksResult<Self> {
        let adapter = AdapterFactory::create(cli_type)
            .map_err(|e| TasksError::Ai(format!("Failed to create CLI adapter: {e}")))?;

        let is_available = Self::check_cli_available(cli_type);

        Ok(Self {
            cli_type,
            adapter,
            is_available,
            mcp_config: None, // Explicitly no MCP config
            extended_thinking,
            thinking_budget: DEFAULT_THINKING_BUDGET,
        })
    }

    /// Create with MCP config for tool access.
    pub fn with_mcp_config(cli_type: CLIType, mcp_config: String) -> TasksResult<Self> {
        let mut provider = Self::new(cli_type)?;
        provider.mcp_config = Some(mcp_config);
        Ok(provider)
    }

    /// Create with both extended thinking and MCP config.
    pub fn with_full_config(
        cli_type: CLIType,
        extended_thinking: bool,
        thinking_budget: Option<u32>,
        mcp_config: Option<String>,
    ) -> TasksResult<Self> {
        let mut provider = Self::new(cli_type)?;
        provider.extended_thinking = extended_thinking;
        provider.thinking_budget = thinking_budget.unwrap_or(DEFAULT_THINKING_BUDGET);
        if let Some(config) = mcp_config {
            provider.mcp_config = Some(config);
        }
        Ok(provider)
    }

    /// Create from environment variable or default to Claude.
    pub fn from_env() -> TasksResult<Self> {
        let cli_str = std::env::var("TASKS_CLI").unwrap_or_else(|_| "claude".to_string());
        let cli_type = CLIType::from_str_ci(&cli_str)
            .ok_or_else(|| TasksError::Ai(format!("Unknown CLI type: {cli_str}")))?;

        // Check for extended thinking env var
        let extended_thinking = std::env::var("TASKS_EXTENDED_THINKING")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let thinking_budget = std::env::var("TASKS_THINKING_BUDGET")
            .ok()
            .and_then(|v| v.parse().ok());

        // Check for MCP config env var
        let mcp_config = std::env::var("TASKS_MCP_CONFIG").ok();

        Self::with_full_config(cli_type, extended_thinking, thinking_budget, mcp_config)
    }

    /// Find default MCP config file.
    ///
    /// Only returns configs that follow the Claude CLI MCP schema.
    fn find_default_mcp_config() -> Option<String> {
        // Check for project-level Claude MCP config
        let project_config = PathBuf::from(".cursor/mcp.json");
        if project_config.exists() {
            return project_config.to_str().map(String::from);
        }

        // Check for tasks-specific MCP config (follows Claude schema)
        let tasks_config = PathBuf::from("tasks-mcp-config.json");
        if tasks_config.exists() {
            return tasks_config.to_str().map(String::from);
        }

        // Check in crates/tasks directory
        let crate_config = PathBuf::from("crates/tasks/tasks-mcp-config.json");
        if crate_config.exists() {
            return crate_config.to_str().map(String::from);
        }

        // NOTE: Do NOT use cto-config.json - it has a different schema
        // NOTE: Do NOT use tools-config.json - it has a different schema

        None
    }

    /// Check if a CLI tool is available in the system.
    fn check_cli_available(cli_type: CLIType) -> bool {
        let executable = match cli_type {
            CLIType::Claude => "claude",
            CLIType::Codex => "codex",
            CLIType::Cursor => "cursor",
            CLIType::Factory => "droid",
            CLIType::OpenCode => "opencode",
            CLIType::Gemini => "gemini",
            CLIType::Grok => "grok",
            CLIType::OpenHands => "openhands",
            CLIType::Qwen => "qwen",
        };

        // Try to find the executable in PATH
        if which::which(executable).is_ok() {
            return true;
        }

        // Check common installation locations for Claude CLI
        if matches!(cli_type, CLIType::Claude) {
            if let Ok(home) = std::env::var("HOME") {
                let claude_path = format!("{}/.claude/local/claude", home);
                if std::path::Path::new(&claude_path).exists() {
                    return true;
                }
            }
        }

        false
    }

    /// Get the full path to the executable for the CLI type.
    fn get_executable_path(cli_type: CLIType) -> String {
        let executable = match cli_type {
            CLIType::Claude => "claude",
            CLIType::Codex => "codex",
            CLIType::Cursor => "cursor",
            CLIType::Factory => "droid",
            CLIType::OpenCode => "opencode",
            CLIType::Gemini => "gemini",
            CLIType::Grok => "grok",
            CLIType::OpenHands => "openhands",
            CLIType::Qwen => "qwen",
        };

        // Try to find in PATH first
        if let Ok(path) = which::which(executable) {
            return path.to_string_lossy().to_string();
        }

        // Check common installation locations for Claude CLI
        if matches!(cli_type, CLIType::Claude) {
            if let Ok(home) = std::env::var("HOME") {
                let claude_path = format!("{}/.claude/local/claude", home);
                if std::path::Path::new(&claude_path).exists() {
                    return claude_path;
                }
            }
        }

        // Fall back to just the executable name
        executable.to_string()
    }

    /// Get the executable name for the CLI type.
    fn get_executable(&self) -> &str {
        self.adapter.get_executable_name()
    }

    /// Convert messages to a single prompt string.
    fn messages_to_prompt(&self, messages: &[AIMessage]) -> String {
        let mut prompt = String::new();

        for msg in messages {
            match msg.role {
                AIRole::System => {
                    prompt.push_str("<system>\n");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n</system>\n\n");
                }
                AIRole::User => {
                    prompt.push_str(&msg.content);
                    prompt.push('\n');
                }
                AIRole::Assistant => {
                    prompt.push_str("<assistant>\n");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n</assistant>\n\n");
                }
            }
        }

        prompt.trim().to_string()
    }

    /// Build the CLI command arguments.
    fn build_cli_args(&self, model: &str, prompt: &str, options: &GenerateOptions) -> Vec<String> {
        let mut args = Vec::new();

        match self.cli_type {
            CLIType::Claude => {
                // Claude CLI: claude -p --model <model> --output-format json "prompt"
                args.push("-p".to_string()); // Print mode (non-interactive)
                args.push("--model".to_string());
                args.push(model.to_string());

                // Add JSON output format for structured output
                if options.json_mode {
                    args.push("--output-format".to_string());
                    args.push("json".to_string());
                } else {
                    args.push("--output-format".to_string());
                    args.push("text".to_string());
                }

                // Add extended thinking settings via --settings JSON
                let use_thinking = options.extended_thinking || self.extended_thinking;
                if use_thinking {
                    let thinking_budget = options.thinking_budget.unwrap_or(self.thinking_budget);
                    let settings_json = serde_json::json!({
                        "alwaysThinkingEnabled": true,
                        "thinkingBudget": thinking_budget
                    });
                    args.push("--settings".to_string());
                    args.push(settings_json.to_string());
                }

                // Add MCP config if available
                let mcp_config = options.mcp_config.as_ref().or(self.mcp_config.as_ref());
                if let Some(config) = mcp_config {
                    args.push("--mcp-config".to_string());
                    args.push(config.clone());
                }

                // Use -- to separate options from the prompt (prevents prompt being
                // interpreted as a file path if it starts with < or other special chars)
                args.push("--".to_string());

                // The prompt is the final positional argument
                args.push(prompt.to_string());
            }
            CLIType::Codex => {
                // Codex CLI: codex exec --json -m <model> -c max_output_tokens=16000 -- "prompt"
                // Must use 'exec' subcommand for non-interactive mode
                args.push("exec".to_string());
                args.push("--json".to_string());
                args.push("-m".to_string());
                args.push(model.to_string());

                // Set max output tokens (default to 16000 for detailed task breakdowns)
                let tokens = options.max_tokens.unwrap_or(16000);
                args.push("-c".to_string());
                args.push(format!("max_output_tokens={tokens}"));

                // Skip sandbox for task generation
                args.push("--skip-git-repo-check".to_string());

                // Use -- to separate options from prompt
                args.push("--".to_string());
                args.push(prompt.to_string());
            }
            CLIType::Cursor => {
                // Cursor: cursor agent --print --output-format json --model <model> "prompt"
                args.push("agent".to_string());
                args.push("--print".to_string());
                args.push("--output-format".to_string());
                args.push("json".to_string());
                args.push("--model".to_string());
                args.push(model.to_string());
                args.push("--".to_string());
                args.push(prompt.to_string());
            }
            CLIType::Factory => {
                // Factory: droid exec --output-format json -m <model> "prompt"
                // Must use 'exec' subcommand for non-interactive mode
                args.push("exec".to_string());
                args.push("--output-format".to_string());
                args.push("json".to_string());
                args.push("-m".to_string());
                args.push(model.to_string());

                // Add reasoning effort for models that support it
                if model.contains("claude") || model.contains("gpt-5") || model.contains("gemini") {
                    args.push("--reasoning-effort".to_string());
                    args.push("high".to_string());
                }

                args.push("--".to_string());
                args.push(prompt.to_string());
            }
            CLIType::OpenCode => {
                // OpenCode: opencode run --format json -m provider/model "prompt"
                // Must use 'run' subcommand for non-interactive mode
                // Model format is provider/model (e.g., anthropic/claude-opus-4-5)
                args.push("run".to_string());
                args.push("--format".to_string());
                args.push("json".to_string());
                args.push("-m".to_string());
                args.push(model.to_string());
                args.push("--".to_string());
                args.push(prompt.to_string());
            }
            CLIType::Gemini => {
                // Gemini: gemini --prompt "prompt"
                args.push("--prompt".to_string());
                args.push(prompt.to_string());

                args.push("--model".to_string());
                args.push(model.to_string());
            }
            CLIType::Grok | CLIType::OpenHands | CLIType::Qwen => {
                // Generic fallback
                args.push(prompt.to_string());
            }
        }

        args
    }

    /// Execute the CLI and capture output.
    async fn execute_cli(
        &self,
        model: &str,
        prompt: &str,
        options: &GenerateOptions,
    ) -> TasksResult<String> {
        let executable = Self::get_executable_path(self.cli_type);
        let args = self.build_cli_args(model, prompt, options);

        debug!(
            cli = %executable,
            args_count = args.len(),
            args = ?args,
            "Executing CLI command"
        );

        // Log first 200 chars of each arg for debugging
        for (i, arg) in args.iter().enumerate() {
            let truncated = if arg.len() > 200 {
                format!("{}...[truncated {} chars]", &arg[..200], arg.len() - 200)
            } else {
                arg.clone()
            };
            debug!(arg_index = i, arg = %truncated, "CLI argument");
        }

        let mut cmd = Command::new(&executable);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            TasksError::Ai(format!("Failed to spawn CLI process '{executable}': {e}"))
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TasksError::Ai("Failed to capture stdout".to_string()))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TasksError::Ai("Failed to capture stderr".to_string()))?;

        // Read stdout
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut output = String::new();

        while let Some(line) = stdout_reader
            .next_line()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to read stdout: {e}")))?
        {
            output.push_str(&line);
            output.push('\n');
        }

        // Check stderr for errors
        let mut stderr_reader = BufReader::new(stderr).lines();
        let mut stderr_output = String::new();

        while let Some(line) = stderr_reader
            .next_line()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to read stderr: {e}")))?
        {
            stderr_output.push_str(&line);
            stderr_output.push('\n');
        }

        // Wait for process to complete
        let status = child
            .wait()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to wait for CLI process: {e}")))?;

        if !status.success() {
            warn!(
                exit_code = ?status.code(),
                stderr = %stderr_output.trim(),
                "CLI process failed"
            );
            return Err(TasksError::Ai(format!(
                "CLI process failed with exit code {}: {}",
                status.code().unwrap_or(-1),
                stderr_output.trim()
            )));
        }

        Ok(output.trim().to_string())
    }

    /// Parse CLI output and extract the actual content.
    #[allow(clippy::unnecessary_wraps)]
    fn parse_cli_output(&self, output: &str, _model: &str) -> TasksResult<(String, TokenUsage)> {
        // Log output for debugging
        let output_preview = if output.len() > 500 {
            format!(
                "{}...[truncated {} chars]",
                &output[..500],
                output.len() - 500
            )
        } else {
            output.to_string()
        };
        debug!(
            cli = %self.cli_type,
            output_len = output.len(),
            output_preview = %output_preview,
            "Parsing CLI output"
        );

        // Try to parse as JSONL (Codex/OpenCode CLI format: multiple JSON objects, one per line)
        if matches!(self.cli_type, CLIType::Codex | CLIType::OpenCode) {
            let mut agent_message = None;
            let mut usage = TokenUsage::default();

            for line in output.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    // Codex format: item.completed with agent_message
                    if json.get("type").and_then(serde_json::Value::as_str)
                        == Some("item.completed")
                    {
                        if let Some(item) = json.get("item") {
                            if item.get("type").and_then(serde_json::Value::as_str)
                                == Some("agent_message")
                            {
                                agent_message = item
                                    .get("text")
                                    .and_then(serde_json::Value::as_str)
                                    .map(String::from);
                            }
                        }
                    }
                    // Codex format: turn.completed with usage
                    if json.get("type").and_then(serde_json::Value::as_str)
                        == Some("turn.completed")
                    {
                        if let Some(usage_obj) = json.get("usage") {
                            #[allow(clippy::cast_possible_truncation)]
                            {
                                usage.input_tokens = usage_obj
                                    .get("input_tokens")
                                    .and_then(serde_json::Value::as_u64)
                                    .unwrap_or(0)
                                    as u32;
                                usage.output_tokens = usage_obj
                                    .get("output_tokens")
                                    .and_then(serde_json::Value::as_u64)
                                    .unwrap_or(0)
                                    as u32;
                            }
                        }
                    }
                    // OpenCode format: type "text" with part.text
                    if json.get("type").and_then(serde_json::Value::as_str) == Some("text") {
                        if let Some(part) = json.get("part") {
                            if let Some(text) = part.get("text").and_then(serde_json::Value::as_str)
                            {
                                // Concatenate text parts (OpenCode streams in chunks)
                                if let Some(ref mut msg) = agent_message {
                                    msg.push_str(text);
                                } else {
                                    agent_message = Some(text.to_string());
                                }
                            }
                        }
                    }
                    // OpenCode format: step_finish with tokens
                    if json.get("type").and_then(serde_json::Value::as_str) == Some("step_finish") {
                        if let Some(part) = json.get("part") {
                            if let Some(tokens) = part.get("tokens") {
                                #[allow(clippy::cast_possible_truncation)]
                                {
                                    usage.input_tokens = tokens
                                        .get("input")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or(0)
                                        as u32;
                                    usage.output_tokens = tokens
                                        .get("output")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or(0)
                                        as u32;
                                }
                            }
                        }
                    }
                }
            }

            if let Some(msg) = agent_message {
                return Ok((msg, usage));
            }
        }

        // Try to parse as JSON (Claude/Cursor/Factory JSON output format)
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
            // Claude/Cursor/Factory JSON format: {"type":"result","result":"...","duration_ms":...}
            if let Some(result) = json.get("result").and_then(serde_json::Value::as_str) {
                #[allow(clippy::cast_possible_truncation)]
                let usage = TokenUsage {
                    input_tokens: json
                        .get("input_tokens")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as u32,
                    output_tokens: json
                        .get("output_tokens")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as u32,
                    total_tokens: 0,
                };
                return Ok((result.to_string(), usage));
            }

            // If it's JSON but not the expected format, return the whole thing
            return Ok((output.to_string(), TokenUsage::default()));
        }

        // Not JSON, return as-is (plain text output)
        Ok((output.to_string(), TokenUsage::default()))
    }

    /// Get supported models for this CLI type.
    fn get_supported_models(&self) -> Vec<&str> {
        match self.cli_type {
            CLIType::Claude => vec![
                // Opus 4.5 (latest with extended thinking)
                "claude-opus-4-5-20251101",
                "opus",
                // Sonnet 4.5 (latest sonnet with thinking)
                "claude-sonnet-4-5-20250929",
                "sonnet",
                // Opus 4.1
                "claude-opus-4-1-20250805",
                // Sonnet 4.0
                "claude-sonnet-4-20250514",
                // Haiku
                "claude-3-5-haiku-20241022",
                "haiku",
                // Legacy models
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-3-haiku-20240307",
            ],
            CLIType::Codex => vec![
                // GPT-5.1 Codex models (latest ChatGPT models)
                "gpt-5.1-codex-max",
                "gpt-5.1-codex",
                "gpt-5.1",
                // o-series reasoning models
                "o3",
                "o3-mini",
                "o1",
                "o1-preview",
                "o1-mini",
                // Legacy GPT models
                "gpt-4-turbo",
                "gpt-4",
            ],
            CLIType::Cursor => vec![
                // Claude models via Cursor (with thinking support)
                "opus-4.5-thinking",
                "opus-4.5",
                "sonnet-4.5-thinking",
                "sonnet-4.5",
                "opus-4.1",
                // GPT models via Cursor
                "gpt-5.1-codex-max-high",
                "gpt-5.1-codex-max",
                "gpt-5.1-codex-high",
                "gpt-5.1-codex",
                "gpt-5.1-high",
                "gpt-5.1",
                // Gemini and other models
                "gemini-3-pro",
                "grok",
                // Auto/Composer
                "composer-1",
                "auto",
            ],
            CLIType::Factory => vec![
                // Claude models via Factory
                "claude-opus-4-5-20251101",
                "claude-sonnet-4-5-20250929",
                "claude-opus-4-1-20250805",
                "claude-haiku-4-5-20251001",
                // OpenAI GPT-5.1 Codex via Factory
                "gpt-5.1-codex",
                "gpt-5.1-codex-max",
                "gpt-5.1",
                // Gemini via Factory
                "gemini-3-pro-preview",
                // GLM/Droid Core models
                "glm-4.6",
                "glm-4-plus",
            ],
            CLIType::OpenCode => vec![
                // Claude models via Anthropic (provider/model format)
                "anthropic/claude-opus-4-5-20251101",
                "anthropic/claude-opus-4-5",
                "anthropic/claude-sonnet-4-5-20250929",
                "anthropic/claude-sonnet-4-5",
                "anthropic/claude-opus-4-1",
                // OpenAI models
                "openai/gpt-4.1",
                "openai/gpt-4o",
                "openai/gpt-4-turbo",
                "openai/codex-mini-latest",
                // Google models
                "google/gemini-2.5-pro",
                "google/gemini-2.5-flash",
                "google/gemini-3-pro-preview",
            ],
            CLIType::Gemini => vec![
                // Gemini 2.5 (latest)
                "gemini-2.5-flash",
                "gemini-2.5-pro",
                // Gemini 2.0
                "gemini-2.0-flash",
                "gemini-2.0-pro",
                // Legacy
                "gemini-1.5-pro",
                "gemini-1.5-flash",
            ],
            CLIType::Grok => vec!["grok-3", "grok-2", "grok-1"],
            CLIType::OpenHands => vec![
                "claude-opus-4-5-20251101",
                "claude-sonnet-4-5-20250929",
                "openhands-default",
            ],
            CLIType::Qwen => vec!["qwen-max", "qwen-plus", "qwen-turbo"],
        }
    }
}

#[async_trait]
impl AIProvider for CLITextGenerator {
    fn name(&self) -> &'static str {
        match self.cli_type {
            CLIType::Claude => "cli-claude",
            CLIType::Codex => "cli-codex",
            CLIType::Cursor => "cli-cursor",
            CLIType::Factory => "cli-factory",
            CLIType::OpenCode => "cli-opencode",
            CLIType::Gemini => "cli-gemini",
            CLIType::Grok => "cli-grok",
            CLIType::OpenHands => "cli-openhands",
            CLIType::Qwen => "cli-qwen",
        }
    }

    fn api_key_env_var(&self) -> &'static str {
        // CLI tools manage their own authentication
        match self.cli_type {
            CLIType::Claude => "ANTHROPIC_API_KEY",
            CLIType::Codex | CLIType::Factory | CLIType::OpenCode => "OPENAI_API_KEY",
            CLIType::Cursor => "CURSOR_API_KEY",
            CLIType::Gemini => "GOOGLE_API_KEY",
            CLIType::Grok => "XAI_API_KEY",
            CLIType::OpenHands => "OPENHANDS_API_KEY",
            CLIType::Qwen => "DASHSCOPE_API_KEY",
        }
    }

    fn is_configured(&self) -> bool {
        self.is_available
    }

    fn supported_models(&self) -> Vec<&str> {
        self.get_supported_models()
    }

    async fn generate_text(
        &self,
        model: &str,
        messages: &[AIMessage],
        options: &GenerateOptions,
    ) -> TasksResult<AIResponse> {
        if !self.is_available {
            return Err(TasksError::Ai(format!(
                "CLI '{}' is not available in the system",
                self.get_executable()
            )));
        }

        let prompt = self.messages_to_prompt(messages);
        let use_thinking = options.extended_thinking || self.extended_thinking;
        let thinking_budget = options.thinking_budget.unwrap_or(self.thinking_budget);

        info!(
            cli = %self.cli_type,
            model = %model,
            prompt_len = prompt.len(),
            extended_thinking = use_thinking,
            thinking_budget = if use_thinking { Some(thinking_budget) } else { None },
            mcp_config = ?options.mcp_config.as_ref().or(self.mcp_config.as_ref()),
            "Generating text via CLI"
        );

        let output = self.execute_cli(model, &prompt, options).await?;

        // Parse Claude CLI JSON output format
        let (text, usage) = self.parse_cli_output(&output, model)?;

        Ok(AIResponse {
            text,
            usage,
            model: model.to_string(),
            provider: self.name().to_string(),
        })
    }
}

impl Default for CLITextGenerator {
    fn default() -> Self {
        Self::from_env().unwrap_or_else(|_| Self {
            cli_type: CLIType::Claude,
            adapter: AdapterFactory::create(CLIType::Claude)
                .expect("Claude adapter should always be available"),
            is_available: false,
            mcp_config: Self::find_default_mcp_config(),
            extended_thinking: false,
            thinking_budget: DEFAULT_THINKING_BUDGET,
        })
    }
}

/// Get the default model for CLI-based generation.
pub fn default_model() -> &'static str {
    DEFAULT_MODEL
}

/// Get all available CLI types.
pub fn available_cli_types() -> Vec<CLIType> {
    AdapterFactory::supported_types()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = CLITextGenerator::default();
        assert!(provider.name().starts_with("cli-"));
    }

    #[test]
    fn test_supported_models() {
        let provider = CLITextGenerator::new(CLIType::Claude).unwrap();
        assert!(provider.supports_model("claude-sonnet-4-20250514"));
        assert!(!provider.supports_model("gpt-4"));
    }

    #[test]
    fn test_messages_to_prompt() {
        let provider = CLITextGenerator::default();
        let messages = vec![
            AIMessage::system("You are a helpful assistant"),
            AIMessage::user("Hello"),
        ];

        let prompt = provider.messages_to_prompt(&messages);
        assert!(prompt.contains("<system>"));
        assert!(prompt.contains("You are a helpful assistant"));
        assert!(prompt.contains("Hello"));
    }

    #[test]
    fn test_available_cli_types() {
        let types = available_cli_types();
        assert!(!types.is_empty());
        assert!(types.contains(&CLIType::Claude));
    }
}
