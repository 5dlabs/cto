//! AI-based prompt generation for individual tasks (Session 2).
//!
//! Uses Claude CLI with MCP tools to generate rich, contextual prompts
//! for each task with bounded context.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};

/// Configuration for prompt generation.
#[derive(Debug, Clone)]
pub struct GeneratePromptsConfig {
    /// Include PRD content in context.
    pub include_prd: bool,
    /// Include architecture document in context.
    pub include_arch: bool,
    /// Generate code-examples.md for each task.
    pub with_examples: bool,
    /// Enable task-specific research via MCP tools.
    pub research: bool,
    /// Path to PRD file.
    pub prd_path: Option<PathBuf>,
    /// Path to architecture document.
    pub arch_path: Option<PathBuf>,
    /// Output directory for generated prompts.
    pub output_dir: PathBuf,
    /// CLI to use (claude, cursor, codex).
    pub cli: String,
    /// Model to use.
    pub model: Option<String>,
    /// MCP config path.
    pub mcp_config: Option<PathBuf>,
}

impl Default for GeneratePromptsConfig {
    fn default() -> Self {
        Self {
            include_prd: true,
            include_arch: true,
            with_examples: false,
            research: true,
            prd_path: None,
            arch_path: None,
            output_dir: PathBuf::from(".tasks/docs"),
            cli: "claude".to_string(),
            model: None,
            mcp_config: None,
        }
    }
}

/// Generated prompt files for a task.
#[derive(Debug, Clone, Default)]
pub struct PromptFiles {
    /// Path to generated prompt.md.
    pub prompt_md: Option<PathBuf>,
    /// Path to generated prompt.xml.
    pub prompt_xml: Option<PathBuf>,
    /// Path to generated acceptance.md.
    pub acceptance_md: Option<PathBuf>,
    /// Path to generated code-examples.md (optional).
    pub code_examples_md: Option<PathBuf>,
}

/// Result of generating prompts for multiple tasks.
#[derive(Debug, Clone, Default)]
pub struct PromptGeneratorResult {
    /// Number of tasks processed.
    pub tasks_processed: usize,
    /// Number of tasks that succeeded.
    pub tasks_succeeded: usize,
    /// Number of tasks that failed.
    pub tasks_failed: usize,
    /// Total input tokens used.
    pub total_input_tokens: u64,
    /// Total output tokens used.
    pub total_output_tokens: u64,
    /// Generated files per task.
    pub task_files: Vec<(String, PromptFiles)>,
}

/// AI-based prompt generator using Claude CLI.
pub struct PromptGenerator {
    config: GeneratePromptsConfig,
}

impl PromptGenerator {
    /// Create a new prompt generator with the given configuration.
    #[must_use]
    pub fn new(config: GeneratePromptsConfig) -> Self {
        Self { config }
    }

    /// Generate prompts for a single task.
    ///
    /// Runs Claude CLI with bounded context to generate rich prompts.
    pub async fn generate_for_task(&self, task: &Task) -> TasksResult<PromptFiles> {
        let task_id = if task.id.starts_with("task-") {
            task.id.clone()
        } else {
            format!("task-{}", task.id)
        };

        let task_dir = self.config.output_dir.join(&task_id);
        tokio::fs::create_dir_all(&task_dir)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: task_dir.display().to_string(),
                reason: e.to_string(),
            })?;

        // Build the prompt for Claude
        let prompt = self.build_prompt(task).await?;

        // Run Claude CLI
        let output = self.run_claude_cli(&prompt, &task_dir).await?;

        // Parse output and return file paths
        let mut files = PromptFiles::default();

        let prompt_md_path = task_dir.join("prompt.md");
        if prompt_md_path.exists() {
            files.prompt_md = Some(prompt_md_path);
        }

        let prompt_xml_path = task_dir.join("prompt.xml");
        if prompt_xml_path.exists() {
            files.prompt_xml = Some(prompt_xml_path);
        }

        let acceptance_path = task_dir.join("acceptance.md");
        if acceptance_path.exists() {
            files.acceptance_md = Some(acceptance_path);
        }

        if self.config.with_examples {
            let examples_path = task_dir.join("code-examples.md");
            if examples_path.exists() {
                files.code_examples_md = Some(examples_path);
            }
        }

        // If no files were created, write them using the output
        if files.prompt_md.is_none() {
            let prompt_md_path = task_dir.join("prompt.md");
            tokio::fs::write(&prompt_md_path, &output)
                .await
                .map_err(|e| TasksError::FileWriteError {
                    path: prompt_md_path.display().to_string(),
                    reason: e.to_string(),
                })?;
            files.prompt_md = Some(prompt_md_path);
        }

        Ok(files)
    }

    /// Generate prompts for all tasks in a directory.
    pub async fn generate_all(&self, tasks_dir: &Path) -> TasksResult<PromptGeneratorResult> {
        let mut result = PromptGeneratorResult::default();

        // Find all task-*.json files
        let mut entries =
            tokio::fs::read_dir(tasks_dir)
                .await
                .map_err(|e| TasksError::FileReadError {
                    path: tasks_dir.display().to_string(),
                    reason: e.to_string(),
                })?;

        let mut task_files = Vec::new();
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| TasksError::FileReadError {
                    path: tasks_dir.display().to_string(),
                    reason: e.to_string(),
                })?
        {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("task-")
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                {
                    task_files.push(path);
                }
            }
        }

        // Sort by task number
        task_files.sort();

        for task_file in task_files {
            result.tasks_processed += 1;

            // Read and parse task
            let content = tokio::fs::read_to_string(&task_file).await.map_err(|e| {
                TasksError::FileReadError {
                    path: task_file.display().to_string(),
                    reason: e.to_string(),
                }
            })?;

            let task: Task =
                serde_json::from_str(&content).map_err(|e| TasksError::JsonParseError {
                    reason: format!("Failed to parse task file {}: {e}", task_file.display()),
                })?;

            match self.generate_for_task(&task).await {
                Ok(files) => {
                    result.tasks_succeeded += 1;
                    result.task_files.push((task.id.clone(), files));
                }
                Err(e) => {
                    result.tasks_failed += 1;
                    tracing::error!("Failed to generate prompts for task {}: {e}", task.id);
                }
            }
        }

        Ok(result)
    }

    /// Build the prompt for Claude CLI.
    async fn build_prompt(&self, task: &Task) -> TasksResult<String> {
        let mut prompt = String::new();

        // Add role and context
        prompt.push_str("You are an expert technical writer generating implementation prompts for development tasks.\n\n");

        // Add PRD context if available
        if self.config.include_prd {
            if let Some(prd_path) = &self.config.prd_path {
                if prd_path.exists() {
                    let prd_content = tokio::fs::read_to_string(prd_path).await.map_err(|e| {
                        TasksError::FileReadError {
                            path: prd_path.display().to_string(),
                            reason: e.to_string(),
                        }
                    })?;
                    prompt.push_str("## Project Requirements (PRD)\n\n");
                    prompt.push_str(&prd_content);
                    prompt.push_str("\n\n");
                }
            }
        }

        // Add architecture context if available
        if self.config.include_arch {
            if let Some(arch_path) = &self.config.arch_path {
                if arch_path.exists() {
                    let arch_content = tokio::fs::read_to_string(arch_path).await.map_err(|e| {
                        TasksError::FileReadError {
                            path: arch_path.display().to_string(),
                            reason: e.to_string(),
                        }
                    })?;
                    prompt.push_str("## Architecture\n\n");
                    prompt.push_str(&arch_content);
                    prompt.push_str("\n\n");
                }
            }
        }

        // Add task details
        prompt.push_str("## Task to Document\n\n");
        writeln!(prompt, "**Task ID**: {}", task.id).ok();
        writeln!(prompt, "**Title**: {}", task.title).ok();
        writeln!(prompt, "**Description**: {}", task.description).ok();

        if let Some(agent) = &task.agent_hint {
            writeln!(prompt, "**Assigned Agent**: {}", agent).ok();
        }

        if !task.dependencies.is_empty() {
            writeln!(prompt, "**Dependencies**: {}", task.dependencies.join(", ")).ok();
        }

        writeln!(prompt, "**Priority**: {}\n", task.priority).ok();

        if !task.details.is_empty() {
            prompt.push_str("### Implementation Details\n\n");
            prompt.push_str(&task.details);
            prompt.push_str("\n\n");
        }

        if !task.test_strategy.is_empty() {
            prompt.push_str("### Test Strategy\n\n");
            prompt.push_str(&task.test_strategy);
            prompt.push_str("\n\n");
        }

        // Add generation instructions
        prompt.push_str("## Instructions\n\n");
        prompt.push_str("Generate the following files for this task:\n\n");
        prompt.push_str("1. **prompt.md** - A detailed markdown prompt that an AI agent can use to implement this task. Include:\n");
        prompt.push_str("   - Clear role definition for the implementing agent\n");
        prompt.push_str("   - Detailed requirements and acceptance criteria\n");
        prompt.push_str("   - Code signatures and interfaces to implement\n");
        prompt.push_str("   - Testing requirements and validation commands\n\n");
        prompt.push_str(
            "2. **prompt.xml** - An XML-formatted prompt with structured sections for:\n",
        );
        prompt.push_str("   - Task metadata (id, priority, agent, dependencies)\n");
        prompt.push_str("   - Context and requirements\n");
        prompt.push_str("   - Code signatures in CDATA blocks\n");
        prompt.push_str("   - Acceptance criteria and validation commands\n\n");
        prompt.push_str(
            "3. **acceptance.md** - A checklist of acceptance criteria that must be met:\n",
        );
        prompt.push_str("   - Functional requirements\n");
        prompt.push_str("   - Test coverage requirements\n");
        prompt.push_str("   - Code quality requirements\n");
        prompt.push_str("   - PR requirements\n\n");

        if self.config.with_examples {
            prompt.push_str("4. **code-examples.md** - Reference code examples and patterns:\n");
            prompt.push_str("   - Similar implementations from the codebase\n");
            prompt.push_str("   - Library/framework usage examples\n");
            prompt.push_str("   - Best practices for the tech stack\n\n");
        }

        if self.config.research {
            prompt.push_str("Use the available MCP tools to research:\n");
            prompt.push_str("- Library documentation (Context7)\n");
            prompt.push_str("- API references and examples (Firecrawl)\n\n");
        }

        prompt.push_str("Write the files to the current directory using the filesystem tools.\n");

        Ok(prompt)
    }

    /// Run Claude CLI with the given prompt.
    async fn run_claude_cli(&self, prompt: &str, working_dir: &Path) -> TasksResult<String> {
        let mut cmd = Command::new(&self.config.cli);

        cmd.current_dir(working_dir)
            .arg("-p")
            .arg(prompt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add model if specified
        if let Some(model) = &self.config.model {
            cmd.arg("--model").arg(model);
        }

        // Add MCP config if specified
        if let Some(mcp_config) = &self.config.mcp_config {
            cmd.arg("--mcp-config").arg(mcp_config);
        }

        // Add allowed tools for research
        if self.config.research {
            cmd.arg("--allowedTools")
                .arg("mcp__firecrawl_*,mcp__context7_*,mcp__filesystem_*,Write,Edit,MultiEdit");
        } else {
            cmd.arg("--allowedTools")
                .arg("mcp__filesystem_*,Write,Edit,MultiEdit");
        }

        // Run in non-interactive mode
        cmd.env("CLAUDE_CODE_ENTRYPOINT", "cli");

        let mut child = cmd
            .spawn()
            .map_err(|e| TasksError::Ai(format!("Failed to spawn {}: {e}", self.config.cli)))?;

        // Close stdin to signal we're done
        if let Some(mut stdin) = child.stdin.take() {
            stdin.shutdown().await.ok();
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to wait for {}: {e}", self.config.cli)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TasksError::Ai(format!(
                "{} exited with status {}: {}",
                self.config.cli, output.status, stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
