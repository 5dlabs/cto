//! CLI-based AI provider implementation.
//!
//! Uses CLI adapters from the shared `cli` crate to interact with various
//! AI coding assistants (Claude, Codex, Cursor, Factory, OpenCode, Gemini).
//!
//! For Claude CLI, uses `--output-format stream-json` to enable real-time
//! activity streaming to Linear via the sidecar.

use async_trait::async_trait;
use cli::{AdapterFactory, CLIType, CliAdapter};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::errors::{TasksError, TasksResult};

use super::provider::{
    AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage, DEFAULT_THINKING_BUDGET,
};

/// Default model for CLI-based generation - Opus 4.5 (most intelligent)
const DEFAULT_MODEL: &str = "claude-opus-4-5-20251101";

/// Default model for extended thinking (same as default - Opus 4.5)
pub const DEFAULT_THINKING_MODEL: &str = "claude-opus-4-5-20251101";

/// Default stream file path for sidecar integration
const DEFAULT_STREAM_FILE: &str = "/workspace/claude-stream.jsonl";

/// CLI-based AI provider that uses CLI adapters for text generation.
///
/// This provider executes AI CLI tools (claude, codex, cursor, etc.) as
/// subprocesses and parses their output, rather than calling APIs directly.
///
/// For Claude CLI, uses `--output-format stream-json` and writes to a stream
/// file that the Linear sidecar monitors for real-time activity updates.
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
    /// Stream file path for sidecar integration (Claude only)
    stream_file: Option<PathBuf>,
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

        // Get stream file from environment (for sidecar integration)
        let stream_file = Self::get_stream_file_path();

        Ok(Self {
            cli_type,
            adapter,
            is_available,
            mcp_config,
            extended_thinking: false,
            thinking_budget: DEFAULT_THINKING_BUDGET,
            stream_file,
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

    /// Create for benchmarking/testing without MCP config or stream file.
    ///
    /// This avoids MCP tools that could cause the model to explain rather than
    /// return structured JSON output, and skips stream file writing.
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
            stream_file: None, // No stream file for benchmarks
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

        // Check for intake-specific MCP config (follows Claude schema)
        let intake_config = PathBuf::from("intake-mcp-config.json");
        if intake_config.exists() {
            return intake_config.to_str().map(String::from);
        }

        // Check in crates/intake directory
        let crate_config = PathBuf::from("crates/intake/intake-mcp-config.json");
        if crate_config.exists() {
            return crate_config.to_str().map(String::from);
        }

        // NOTE: Do NOT use cto-config.json - it has a different schema
        // NOTE: Do NOT use tools-config.json - it has a different schema

        None
    }

    /// Get stream file path for sidecar integration.
    ///
    /// When running in-cluster, the sidecar monitors this file to emit
    /// structured activities to Linear. Returns None if not in-cluster
    /// or if the environment variable is not set.
    fn get_stream_file_path() -> Option<PathBuf> {
        // Check for explicit environment variable
        if let Ok(path) = std::env::var("CLAUDE_STREAM_FILE") {
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }

        // Check if default stream file location exists (indicates in-cluster)
        let default_path = PathBuf::from(DEFAULT_STREAM_FILE);
        if default_path.parent().is_some_and(std::path::Path::exists) {
            return Some(default_path);
        }

        None
    }

    /// Check if a CLI tool is available in the system.
    fn check_cli_available(cli_type: CLIType) -> bool {
        let executable = match cli_type {
            CLIType::Claude => "claude",
            CLIType::Code => "code",
            CLIType::Codex => "codex",
            CLIType::Cursor => "cursor",
            CLIType::Factory => "droid",
            CLIType::OpenCode => "opencode",
            CLIType::Gemini => "gemini",
            CLIType::Dexter => "dexter-agent",
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
            CLIType::Code => "code",
            CLIType::Codex => "codex",
            CLIType::Cursor => "cursor",
            CLIType::Factory => "droid",
            CLIType::OpenCode => "opencode",
            CLIType::Gemini => "gemini",
            CLIType::Dexter => "dexter-agent",
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

    /// Get the CLI type.
    pub fn cli_type(&self) -> CLIType {
        self.cli_type
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
                // Claude CLI: claude -p --model <model> --output-format stream-json --verbose "prompt"
                args.push("-p".to_string()); // Print mode (non-interactive)
                args.push("--model".to_string());
                args.push(model.to_string());

                // Use stream-json format for sidecar integration
                // This outputs JSONL that the sidecar parses for Linear activities
                args.push("--output-format".to_string());
                args.push("stream-json".to_string());

                // --verbose is required when using --print with --output-format=stream-json
                args.push("--verbose".to_string());

                // Add extended thinking settings via --settings JSON
                // force_disable_thinking overrides provider defaults (useful for retry logic)
                let use_thinking = !options.force_disable_thinking
                    && (options.extended_thinking || self.extended_thinking);
                if use_thinking {
                    let thinking_budget = options.thinking_budget.unwrap_or(self.thinking_budget);
                    let settings_json = serde_json::json!({
                        "alwaysThinkingEnabled": true,
                        "thinkingBudget": thinking_budget
                    });
                    args.push("--settings".to_string());
                    args.push(settings_json.to_string());
                }

                // Add MCP config if available (unless explicitly disabled)
                if !options.disable_mcp {
                    let mcp_config = options.mcp_config.as_ref().or(self.mcp_config.as_ref());
                    if let Some(config) = mcp_config {
                        args.push("--mcp-config".to_string());
                        args.push(config.clone());
                    }
                }

                // Use -- to separate options from the prompt (prevents prompt being
                // interpreted as a file path if it starts with < or other special chars)
                args.push("--".to_string());

                // The prompt is the final positional argument
                args.push(prompt.to_string());
            }
            CLIType::Code => {
                // Every Code CLI (fork of Codex): code exec --json -m <model> -c max_output_tokens=16000 -- "prompt"
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
                // Factory: droid exec --auto high --output-format json -m <model> "prompt"
                // Must use 'exec' subcommand for non-interactive mode
                args.push("exec".to_string());

                // Enable full autonomy for CI/CD and development work
                // Levels: low (safe edits), medium (dev work), high (CI/CD - full access)
                args.push("--auto".to_string());
                args.push("high".to_string());

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
            CLIType::Dexter => {
                // Dexter: dexter-agent "prompt"
                // Dexter accepts prompts directly as the command argument
                args.push(prompt.to_string());
            }
        }

        args
    }

    /// Execute the CLI and capture output.
    ///
    /// For Claude CLI with stream-json format, also writes output to the stream
    /// file in real-time for sidecar integration.
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

        // Set max output tokens for Claude CLI via environment variable
        // This ensures the response isn't truncated for large task generation
        if matches!(self.cli_type, CLIType::Claude) {
            let max_tokens = options.max_tokens.unwrap_or(64_000);
            cmd.env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", max_tokens.to_string());
            debug!(
                max_tokens,
                "Setting CLAUDE_CODE_MAX_OUTPUT_TOKENS environment variable"
            );
        }

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

        // Open stream file for writing if available (for sidecar integration)
        let mut stream_file = if matches!(self.cli_type, CLIType::Claude) {
            if let Some(ref path) = self.stream_file {
                match OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(path)
                    .await
                {
                    Ok(file) => {
                        info!(path = %path.display(), "Streaming output to file for sidecar");
                        Some(file)
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Failed to open stream file, continuing without streaming");
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Read stdout line by line, writing to stream file in real-time
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut output = String::new();

        while let Some(line) = stdout_reader
            .next_line()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to read stdout: {e}")))?
        {
            // Write to stream file immediately for sidecar to parse
            if let Some(ref mut file) = stream_file {
                // Write line with newline, flush immediately for real-time streaming
                if let Err(e) = file.write_all(format!("{line}\n").as_bytes()).await {
                    warn!(error = %e, "Failed to write to stream file");
                }
                if let Err(e) = file.flush().await {
                    warn!(error = %e, "Failed to flush stream file");
                }
            }

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
    #[allow(clippy::unnecessary_wraps)] // Returns Result for consistency with trait methods
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

        // Parse JSONL format (Claude stream-json, Codex, OpenCode)
        // Each line is a separate JSON object
        if matches!(
            self.cli_type,
            CLIType::Claude | CLIType::Codex | CLIType::OpenCode
        ) {
            let mut result_text = None;
            let mut assistant_text = String::new();
            let mut usage = TokenUsage::default();

            for line in output.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    let event_type = json.get("type").and_then(serde_json::Value::as_str);

                    match event_type {
                        // Claude stream-json: result event contains the final result
                        Some("result") => {
                            if let Some(result) =
                                json.get("result").and_then(serde_json::Value::as_str)
                            {
                                result_text = Some(result.to_string());
                            }
                            // Extract usage from result event
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            // Token counts are small positive integers
                            if let Some(total_cost) = json
                                .get("total_cost_usd")
                                .and_then(serde_json::Value::as_f64)
                            {
                                // Estimate tokens from cost (rough approximation)
                                // Claude Opus: ~$15/1M input, ~$75/1M output
                                // Cost is always positive, so sign loss is safe
                                usage.total_tokens = (total_cost * 20000.0) as u32;
                            }
                        }

                        // Claude stream-json: assistant message contains text content
                        Some("assistant") => {
                            if let Some(message) = json.get("message") {
                                if let Some(content) = message.get("content") {
                                    if let Some(content_arr) = content.as_array() {
                                        for block in content_arr {
                                            if block.get("type").and_then(serde_json::Value::as_str)
                                                == Some("text")
                                            {
                                                if let Some(text) = block
                                                    .get("text")
                                                    .and_then(serde_json::Value::as_str)
                                                {
                                                    if !assistant_text.is_empty() {
                                                        assistant_text.push('\n');
                                                    }
                                                    assistant_text.push_str(text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Codex format: item.completed with agent_message
                        Some("item.completed") => {
                            if let Some(item) = json.get("item") {
                                if item.get("type").and_then(serde_json::Value::as_str)
                                    == Some("agent_message")
                                {
                                    result_text = item
                                        .get("text")
                                        .and_then(serde_json::Value::as_str)
                                        .map(String::from);
                                }
                            }
                        }

                        // Codex format: turn.completed with usage
                        Some("turn.completed") => {
                            if let Some(usage_obj) = json.get("usage") {
                                #[allow(clippy::cast_possible_truncation)]
                                // Token counts fit in u32
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
                        Some("text") => {
                            if let Some(part) = json.get("part") {
                                if let Some(text) =
                                    part.get("text").and_then(serde_json::Value::as_str)
                                {
                                    assistant_text.push_str(text);
                                }
                            }
                        }

                        // OpenCode format: step_finish with tokens
                        Some("step_finish") => {
                            if let Some(part) = json.get("part") {
                                if let Some(tokens) = part.get("tokens") {
                                    #[allow(clippy::cast_possible_truncation)]
                                    // Token counts fit in u32
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

                        _ => {}
                    }
                }
            }

            // Prefer result_text (from result event), fall back to accumulated assistant text
            if let Some(result) = result_text {
                return Ok((result, usage));
            }
            if !assistant_text.is_empty() {
                return Ok((assistant_text, usage));
            }
        }

        // Try to parse as single JSON object (legacy format)
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
            // Claude/Cursor/Factory JSON format: {"type":"result","result":"...","duration_ms":...}
            if let Some(result) = json.get("result").and_then(serde_json::Value::as_str) {
                #[allow(clippy::cast_possible_truncation)] // Token counts fit in u32
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
                // Claude 4.5 models (current - Jan 2025)
                "claude-opus-4-5-20251101",
                "claude-sonnet-4-5-20250929",
                "claude-haiku-4-5-20251001",
                // Short names
                "opus",
                "sonnet",
                "haiku",
                // Aliases
                "claude-opus-4-5",
                "claude-sonnet-4-5",
                "claude-haiku-4-5",
                // Claude 4.1 (legacy)
                "claude-opus-4-1-20250805",
                // Claude 3.5 (deprecated)
                "claude-3-5-haiku-20241022",
                // Claude 3 (deprecated)
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-3-haiku-20240307",
            ],
            CLIType::Code => vec![
                // Every Code supports multi-provider models (fork of Codex)
                // OpenAI GPT-5.1 Codex models
                "gpt-5.1-codex-max",
                "gpt-5.1-codex",
                "gpt-5.1",
                // o-series reasoning models
                "o3",
                "o3-mini",
                "o1",
                "o1-preview",
                "o1-mini",
                // Claude models via Anthropic
                "claude-opus-4-5-20251101",
                "claude-sonnet-4-5-20250929",
                // Legacy GPT models
                "gpt-4-turbo",
                "gpt-4",
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
            CLIType::Dexter => vec![
                // Claude models
                "claude-opus-4-5-20251101",
                "claude-sonnet-4-5-20250929",
                "claude-sonnet-4-20250514",
                // GPT models
                "gpt-5.1",
                "gpt-4-turbo",
                "gpt-4",
                // Gemini models
                "gemini-2.5-pro",
                "gemini-2.5-flash",
                // Reasoning models
                "o3",
                "o1",
                "o4-mini",
            ],
        }
    }
}

#[async_trait]
impl AIProvider for CLITextGenerator {
    fn name(&self) -> &'static str {
        match self.cli_type {
            CLIType::Claude => "cli-claude",
            CLIType::Code => "cli-code",
            CLIType::Codex => "cli-codex",
            CLIType::Cursor => "cli-cursor",
            CLIType::Factory => "cli-factory",
            CLIType::OpenCode => "cli-opencode",
            CLIType::Gemini => "cli-gemini",
            CLIType::Dexter => "cli-dexter",
        }
    }

    fn api_key_env_var(&self) -> &'static str {
        // CLI tools manage their own authentication
        // Note: Dexter supports multiple API keys (Anthropic, OpenAI, Google)
        // but we return Anthropic as the primary since Claude is commonly used
        // Code (Every Code) supports multiple providers but uses OpenAI as primary
        match self.cli_type {
            CLIType::Claude | CLIType::Dexter => "ANTHROPIC_API_KEY",
            CLIType::Code | CLIType::Codex | CLIType::Factory | CLIType::OpenCode => {
                "OPENAI_API_KEY"
            }
            CLIType::Cursor => "CURSOR_API_KEY",
            CLIType::Gemini => "GOOGLE_API_KEY",
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
        // force_disable_thinking overrides provider defaults (useful for retry logic)
        let use_thinking = !options.force_disable_thinking
            && (options.extended_thinking || self.extended_thinking);
        let thinking_budget = options.thinking_budget.unwrap_or(self.thinking_budget);

        info!(
            cli = %self.cli_type,
            model = %model,
            prompt_len = prompt.len(),
            extended_thinking = use_thinking,
            force_disable_thinking = options.force_disable_thinking,
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
            stream_file: Self::get_stream_file_path(),
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
        assert!(provider.supports_model("claude-opus-4-5-20251101"));
        assert!(provider.supports_model("opus")); // Short alias
        assert!(!provider.supports_model("gpt-4")); // Not a Claude model
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
