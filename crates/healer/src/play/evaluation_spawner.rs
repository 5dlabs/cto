//! Evaluation Agent spawner for Play monitoring.
//!
//! Spawns Claude CLI `CodeRuns` that act as the Evaluation Agent, responsible for:
//! - Universal pre-flight checks (prompts, MCP tools)
//! - Lifecycle progression monitoring
//! - Issue detection and structured output
//!
//! This implements the "Model 1: Evaluation Agent" from the dual-model Healer architecture.

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use tracing::{debug, info, warn};

use super::session::PlaySession;

/// Configuration for the Evaluation Agent spawner.
#[derive(Debug, Clone)]
pub struct EvaluationSpawnerConfig {
    /// Kubernetes namespace for `CodeRuns`
    pub namespace: String,
    /// GitHub App to use for the Evaluation Agent
    pub github_app: String,
    /// Model to use for evaluation
    pub model: String,
    /// CLI type (claude, opencode, etc.)
    pub cli: String,
    /// Repository being monitored
    pub repository: String,
}

impl Default for EvaluationSpawnerConfig {
    fn default() -> Self {
        Self {
            namespace: "cto".to_string(),
            github_app: "5DLabs-Healer".to_string(),
            model: "claude-sonnet-4-5-20250514".to_string(),
            cli: "claude".to_string(),
            repository: "5dlabs/cto".to_string(),
        }
    }
}

/// Result of spawning an Evaluation Agent `CodeRun`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSpawnResult {
    /// Name of the created `CodeRun`
    pub coderun_name: String,
    /// Play ID being evaluated
    pub play_id: String,
    /// Whether spawn was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Evaluation Agent spawner.
///
/// Creates `CodeRuns` that evaluate Play sessions for issues.
pub struct EvaluationSpawner {
    config: EvaluationSpawnerConfig,
}

impl EvaluationSpawner {
    /// Create a new Evaluation Agent spawner.
    #[must_use]
    pub fn new(config: EvaluationSpawnerConfig) -> Self {
        Self { config }
    }

    /// Create spawner with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(EvaluationSpawnerConfig::default())
    }

    /// Spawn an Evaluation Agent `CodeRun` for a Play session.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - kubectl cannot be executed
    /// - The `CodeRun` creation fails
    pub fn spawn_evaluation(&self, session: &PlaySession) -> Result<EvaluationSpawnResult> {
        let play_id = &session.play_id;
        let timestamp = chrono::Utc::now().timestamp();
        let coderun_name = format!("eval-{}-{}", sanitize_name(play_id), timestamp);

        info!(
            play_id = %play_id,
            coderun_name = %coderun_name,
            "Spawning Evaluation Agent CodeRun"
        );

        // Build the evaluation prompt
        let prompt = build_evaluation_prompt(session);

        // Build `CodeRun` spec
        let coderun_spec = build_coderun_spec(&coderun_name, &self.config, &prompt, play_id);

        // Apply the `CodeRun` using kubectl (use JSON format)
        let coderun_json = serde_json::to_string_pretty(&coderun_spec)
            .context("Failed to serialize CodeRun spec")?;

        debug!(
            coderun_name = %coderun_name,
            json_length = %coderun_json.len(),
            "Applying Evaluation CodeRun"
        );

        // Pipe the JSON spec to kubectl apply
        let mut child = Command::new("kubectl")
            .args(["apply", "-f", "-", "-n", &self.config.namespace])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(coderun_json.as_bytes())
                .context("Failed to write to kubectl stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("kubectl execution failed")?;

        if output.status.success() {
            info!(
                play_id = %play_id,
                coderun_name = %coderun_name,
                "Evaluation Agent CodeRun created successfully"
            );
            Ok(EvaluationSpawnResult {
                coderun_name,
                play_id: play_id.clone(),
                success: true,
                error: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                play_id = %play_id,
                error = %stderr,
                "Failed to create Evaluation Agent CodeRun"
            );
            Ok(EvaluationSpawnResult {
                coderun_name,
                play_id: play_id.clone(),
                success: false,
                error: Some(stderr.to_string()),
            })
        }
    }
}

/// Build the evaluation prompt for the Evaluation Agent.
#[allow(clippy::format_push_string, clippy::single_char_add_str)]
fn build_evaluation_prompt(session: &PlaySession) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Healer Evaluation Agent\n\n");
    prompt.push_str("You are the Evaluation Agent for the CTO Healer system. ");
    prompt.push_str("Your job is to monitor a Play session and detect issues.\n\n");

    // Play context
    prompt.push_str("## Play Session Context\n\n");
    prompt.push_str(&format!("- **Play ID:** {}\n", session.play_id));
    prompt.push_str(&format!("- **Repository:** {}\n", session.repository));
    if let Some(service) = &session.service {
        prompt.push_str(&format!("- **Service:** {service}\n"));
    }
    prompt.push_str(&format!("- **Started:** {}\n", session.started_at));
    prompt.push_str("\n");

    // Expected tools per agent
    prompt.push_str("## Expected Agent Tools (from CTO Config)\n\n");
    for (agent_name, agent_config) in &session.cto_config.agents {
        prompt.push_str(&format!("### Agent: {agent_name}\n"));
        if let Some(cli) = &agent_config.cli {
            prompt.push_str(&format!("- CLI: {cli}\n"));
        }
        if let Some(model) = &agent_config.model {
            prompt.push_str(&format!("- Model: {model}\n"));
        }
        prompt.push_str(&format!(
            "- Remote tools: {:?}\n",
            agent_config.tools.remote
        ));
        prompt.push_str(&format!(
            "- Local servers: {:?}\n",
            agent_config.tools.local_servers.keys().collect::<Vec<_>>()
        ));
        prompt.push_str("\n");
    }

    // Tasks
    prompt.push_str("## Tasks\n\n");
    for task in &session.tasks {
        prompt.push_str(&format!(
            "- Task {}: {} (agent hint: {:?})\n",
            task.id, task.title, task.agent_hint
        ));
    }
    prompt.push_str("\n");

    // Universal pre-flight checks
    prompt.push_str("## Universal Pre-Flight Checks\n\n");
    prompt.push_str("For EVERY agent run, verify:\n\n");
    prompt.push_str("### 1. Prompt Verification\n");
    prompt.push_str("- [ ] Agent type matches task requirements\n");
    prompt.push_str("- [ ] Prompt template loaded successfully\n");
    prompt.push_str("- [ ] Role-specific instructions present\n");
    prompt.push_str("- [ ] Language/stack context included\n");
    prompt.push_str("\n");
    prompt.push_str("### 2. MCP Tool Verification\n");
    prompt.push_str("- [ ] CTO config loaded and valid\n");
    prompt.push_str("- [ ] ALL declared remote tools accessible\n");
    prompt.push_str("- [ ] tools-server reachable and authenticated\n");
    prompt.push_str("- [ ] ALL declared local servers initialized\n");
    prompt.push_str("- [ ] **declared tools == available tools** (no mismatch!)\n");
    prompt.push_str("\n");

    // Instructions
    prompt.push_str("## Your Task\n\n");
    prompt.push_str("1. Query Loki logs for this Play session using `kubectl logs` or Loki API\n");
    prompt.push_str("2. Check for any pre-flight failures (tool mismatch, config errors)\n");
    prompt.push_str("3. Check for stuck agents (no progress >30 min)\n");
    prompt.push_str("4. Check for error patterns in logs\n");
    prompt.push_str("5. If issues found, create a detailed GitHub issue\n\n");

    prompt.push_str("## Output Format\n\n");
    prompt.push_str("Output a JSON summary of your evaluation:\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"play_id\": \"...\",\n");
    prompt.push_str("  \"evaluation_time\": \"...\",\n");
    prompt.push_str("  \"status\": \"healthy\" | \"warning\" | \"critical\",\n");
    prompt.push_str("  \"issues\": [\n");
    prompt.push_str("    {\n");
    prompt.push_str(
        "      \"type\": \"tool_mismatch\" | \"stuck\" | \"error\" | \"pre_flight_failure\",\n",
    );
    prompt.push_str("      \"severity\": \"critical\" | \"high\" | \"medium\" | \"low\",\n");
    prompt.push_str("      \"agent\": \"...\",\n");
    prompt.push_str("      \"description\": \"...\",\n");
    prompt.push_str("      \"evidence\": \"...\"\n");
    prompt.push_str("    }\n");
    prompt.push_str("  ],\n");
    prompt.push_str("  \"recommendation\": \"...\"\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n");

    prompt
}

/// Build a `CodeRun` spec for the Evaluation Agent.
fn build_coderun_spec(
    name: &str,
    config: &EvaluationSpawnerConfig,
    prompt: &str,
    play_id: &str,
) -> serde_json::Value {
    json!({
        "apiVersion": "agents.platform/v1alpha1",
        "kind": "CodeRun",
        "metadata": {
            "name": name,
            "labels": {
                "healer.agents.platform/type": "evaluation",
                "healer.agents.platform/play-id": play_id,
                "agents.platform/service": "healer-evaluation"
            }
        },
        "spec": {
            "service": "healer-evaluation",
            "repository": config.repository,
            "branch": "main",
            "githubApp": config.github_app,
            "model": config.model,
            "cliConfig": {
                "cliType": config.cli
            },
            "prompt": prompt,
            "timeout": "30m"
        }
    })
}

/// Sanitize a name for use in Kubernetes resource names.
fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::session::{AgentConfig, AgentTools, CtoConfig, TaskInfo};
    use chrono::Utc;
    use std::collections::HashMap;

    fn sample_session() -> PlaySession {
        let mut agents = HashMap::new();
        agents.insert(
            "rex".to_string(),
            AgentConfig {
                github_app: Some("5DLabs-Rex".to_string()),
                cli: Some("claude".to_string()),
                model: Some("claude-sonnet-4-5-20250514".to_string()),
                tools: AgentTools {
                    remote: vec!["brave_search".to_string(), "github_create_pr".to_string()],
                    local_servers: HashMap::new(),
                },
            },
        );

        PlaySession {
            play_id: "test-play-1".to_string(),
            repository: "5dlabs/test".to_string(),
            service: Some("test-service".to_string()),
            cto_config: CtoConfig { agents },
            tasks: vec![TaskInfo {
                id: "1".to_string(),
                title: "Setup infrastructure".to_string(),
                agent_hint: Some("bolt".to_string()),
                dependencies: vec![],
                priority: 0,
            }],
            namespace: "cto".to_string(),
            started_at: Utc::now(),
            last_updated: Utc::now(),
            issues: vec![],
            status: crate::play::session::SessionStatus::Active,
        }
    }

    #[test]
    fn test_build_evaluation_prompt() {
        let session = sample_session();
        let prompt = build_evaluation_prompt(&session);

        assert!(prompt.contains("Healer Evaluation Agent"));
        assert!(prompt.contains("test-play-1"));
        assert!(prompt.contains("5dlabs/test"));
        assert!(prompt.contains("rex"));
        assert!(prompt.contains("brave_search"));
        assert!(prompt.contains("Universal Pre-Flight Checks"));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("play-123"), "play-123");
        assert_eq!(sanitize_name("Play_123"), "play-123");
        assert_eq!(sanitize_name("play.123.test"), "play-123-test");
        assert_eq!(sanitize_name("--test--"), "test");
    }

    #[test]
    fn test_build_coderun_spec() {
        let config = EvaluationSpawnerConfig::default();
        let spec = build_coderun_spec("eval-test-1", &config, "Test prompt", "test-play");

        assert_eq!(spec["kind"], "CodeRun");
        assert_eq!(spec["metadata"]["name"], "eval-test-1");
        assert_eq!(
            spec["metadata"]["labels"]["healer.agents.platform/type"],
            "evaluation"
        );
        assert_eq!(spec["spec"]["model"], "claude-sonnet-4-5-20250514");
    }
}
