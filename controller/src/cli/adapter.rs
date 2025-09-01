//! CLI Adapter
//!
//! Handles CLI-specific execution and command adaptation.
//! Manages the actual execution of CLI commands and result processing.

use crate::cli::types::*;
use std::process::Stdio;
use tokio::process::Command;

/// CLI execution adapter
pub struct CLIExecutionAdapter {
    /// CLI type this adapter handles
    cli_type: CLIType,
}

impl CLIExecutionAdapter {
    /// Create a new adapter for a specific CLI type
    pub fn new(cli_type: CLIType) -> Self {
        Self { cli_type }
    }

    /// Execute a CLI command with the given context
    pub async fn execute(&self, context: &CLIExecutionContext) -> Result<CLIExecutionResult> {
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
            duration_ms: duration.as_millis() as u64,
            cli_type: self.cli_type,
        };

        Ok(result)
    }

    /// Prepare files for CLI execution
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
    pub async fn validate_environment(&self, required_vars: &[String]) -> Result<Vec<String>> {
        let mut missing = Vec::new();

        for var in required_vars {
            if std::env::var(var).is_err() {
                missing.push(var.clone());
            }
        }

        if !missing.is_empty() {
            return Err(AdapterError::MissingEnvironmentVariables(missing));
        }

        Ok(required_vars.to_vec())
    }

    /// Get CLI-specific execution hints
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
    pub fn new(cli_type: CLIType) -> Self {
        Self { cli_type }
    }

    /// Build command for task execution
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
            CLIType::OpenCode => {
                vec!["opencode".to_string(), task.to_string()]
            }
            _ => {
                vec!["echo".to_string(), format!("Task: {}", task)]
            }
        }
    }

    /// Build command for version check
    pub fn build_version_command(&self) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => vec!["claude-code".to_string(), "--version".to_string()],
            CLIType::Codex => vec!["codex".to_string(), "--version".to_string()],
            CLIType::OpenCode => vec!["opencode".to_string(), "--version".to_string()],
            _ => vec!["echo".to_string(), "unknown".to_string()],
        }
    }

    /// Build command for help
    pub fn build_help_command(&self) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => vec!["claude-code".to_string(), "--help".to_string()],
            CLIType::Codex => vec!["codex".to_string(), "--help".to_string()],
            CLIType::OpenCode => vec!["opencode".to_string(), "--help".to_string()],
            _ => vec!["echo".to_string(), "help not available".to_string()],
        }
    }
}

/// Result processor for CLI outputs
pub struct ResultProcessor {
    cli_type: CLIType,
}

impl ResultProcessor {
    pub fn new(cli_type: CLIType) -> Self {
        Self { cli_type }
    }

    /// Process execution result and extract key information
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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AdapterError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_builder_claude() {
        let builder = CommandBuilder::new(CLIType::Claude);
        let cmd = builder.build_task_command("implement auth", false);
        assert_eq!(cmd, vec!["claude-code", "implement auth"]);
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
    fn test_version_commands() {
        let claude_builder = CommandBuilder::new(CLIType::Claude);
        let codex_builder = CommandBuilder::new(CLIType::Codex);

        assert_eq!(
            claude_builder.build_version_command(),
            vec!["claude-code", "--version"]
        );
        assert_eq!(
            codex_builder.build_version_command(),
            vec!["codex", "--version"]
        );
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
}
