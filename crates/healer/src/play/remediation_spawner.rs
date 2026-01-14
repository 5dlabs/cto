//! Remediation Agent spawner for Play monitoring.
//!
//! Spawns Claude CLI `CodeRuns` that act as the Remediation Agent, responsible for:
//! - Fixing issues detected by the Evaluation Agent
//! - Creating PRs with fixes
//! - Retrying failed operations
//!
//! This implements the "Model 2: Remediation Agent" from the dual-model Healer architecture.

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use tracing::{debug, info, warn};

use super::session::{IssueSeverity, IssueType, PlaySession, SessionIssue};

/// Configuration for the Remediation Agent spawner.
#[derive(Debug, Clone)]
pub struct RemediationSpawnerConfig {
    /// Kubernetes namespace for `CodeRuns`
    pub namespace: String,
    /// GitHub App to use for the Remediation Agent
    pub github_app: String,
    /// Model to use for remediation
    pub model: String,
    /// CLI type (claude, opencode, etc.)
    pub cli: String,
    /// Repository being fixed
    pub repository: String,
    /// Maximum retries before escalation
    pub max_retries: u32,
}

impl Default for RemediationSpawnerConfig {
    fn default() -> Self {
        Self {
            namespace: "cto".to_string(),
            github_app: "5DLabs-Healer".to_string(),
            model: "claude-sonnet-4-5-20250514".to_string(),
            cli: "claude".to_string(),
            repository: "5dlabs/cto".to_string(),
            max_retries: 3,
        }
    }
}

/// Result of spawning a Remediation Agent `CodeRun`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSpawnResult {
    /// Name of the created `CodeRun`
    pub coderun_name: String,
    /// Play ID being remediated
    pub play_id: String,
    /// Issue being fixed
    pub issue_type: String,
    /// Whether spawn was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Attempt number (for retries)
    pub attempt: u32,
}

/// Remediation Agent spawner.
///
/// Creates `CodeRuns` that fix issues detected during Play sessions.
pub struct RemediationSpawner {
    config: RemediationSpawnerConfig,
}

impl RemediationSpawner {
    /// Create a new Remediation Agent spawner.
    #[must_use]
    pub fn new(config: RemediationSpawnerConfig) -> Self {
        Self { config }
    }

    /// Create spawner with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(RemediationSpawnerConfig::default())
    }

    /// Spawn a Remediation Agent `CodeRun` for an issue.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - kubectl cannot be executed
    /// - The `CodeRun` creation fails
    pub fn spawn_remediation(
        &self,
        session: &PlaySession,
        issue: &SessionIssue,
        attempt: u32,
    ) -> Result<RemediationSpawnResult> {
        let play_id = &session.play_id;
        let issue_type = format!("{:?}", issue.issue_type).to_lowercase();
        let timestamp = chrono::Utc::now().timestamp();
        let coderun_name = format!(
            "fix-{}-{}-{}",
            sanitize_name(play_id),
            sanitize_name(&issue_type),
            timestamp
        );

        info!(
            play_id = %play_id,
            issue_type = %issue_type,
            attempt = %attempt,
            coderun_name = %coderun_name,
            "Spawning Remediation Agent CodeRun"
        );

        // Check if we've exceeded max retries
        if attempt > self.config.max_retries {
            warn!(
                play_id = %play_id,
                issue_type = %issue_type,
                max_retries = %self.config.max_retries,
                "Max retries exceeded, escalating to human"
            );
            return Ok(RemediationSpawnResult {
                coderun_name,
                play_id: play_id.clone(),
                issue_type,
                success: false,
                error: Some(format!(
                    "Max retries ({}) exceeded - escalating to human",
                    self.config.max_retries
                )),
                attempt,
            });
        }

        // Build the remediation prompt
        let prompt = build_remediation_prompt(session, issue, attempt, self.config.max_retries);

        // Build `CodeRun` spec
        let coderun_spec =
            build_remediation_coderun_spec(&coderun_name, &self.config, &prompt, play_id, issue);

        // Apply the `CodeRun` using kubectl (use JSON format)
        let coderun_json = serde_json::to_string_pretty(&coderun_spec)
            .context("Failed to serialize CodeRun spec")?;

        debug!(
            coderun_name = %coderun_name,
            json_length = %coderun_json.len(),
            "Applying Remediation CodeRun"
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
                issue_type = %issue_type,
                "Remediation Agent CodeRun created successfully"
            );
            Ok(RemediationSpawnResult {
                coderun_name,
                play_id: play_id.clone(),
                issue_type,
                success: true,
                error: None,
                attempt,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                play_id = %play_id,
                error = %stderr,
                "Failed to create Remediation Agent CodeRun"
            );
            Ok(RemediationSpawnResult {
                coderun_name,
                play_id: play_id.clone(),
                issue_type,
                success: false,
                error: Some(stderr.to_string()),
                attempt,
            })
        }
    }

    /// Determine the appropriate remediation strategy for an issue.
    #[must_use]
    pub fn get_remediation_strategy(issue: &SessionIssue) -> RemediationStrategy {
        match (&issue.issue_type, &issue.severity) {
            // Config issues need config fixes
            (
                IssueType::PreFlightFailure | IssueType::ToolMismatch | IssueType::ConfigError,
                IssueSeverity::Critical,
            )
            | (IssueType::LanguageMismatch, _) => RemediationStrategy::FixConfig,

            // Build/test failures can be auto-fixed
            (IssueType::BuildFailure | IssueType::TestFailure, _) => RemediationStrategy::FixCode,

            // Stuck agents might need restart
            (IssueType::Stuck, _) => RemediationStrategy::Restart,

            // Default to retry for everything else
            _ => RemediationStrategy::Retry,
        }
    }
}

/// Remediation strategy types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStrategy {
    /// Retry the failed operation
    Retry,
    /// Fix configuration issues
    FixConfig,
    /// Fix code issues (build/test failures)
    FixCode,
    /// Restart the agent
    Restart,
    /// Escalate to human
    Escalate,
}

/// Build the remediation prompt for the Remediation Agent.
#[allow(clippy::format_push_string, clippy::single_char_add_str)]
fn build_remediation_prompt(
    session: &PlaySession,
    issue: &SessionIssue,
    attempt: u32,
    max_retries: u32,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Healer Remediation Agent\n\n");
    prompt.push_str("You are the Remediation Agent for the CTO Healer system. ");
    prompt.push_str("Your job is to fix issues detected by the Evaluation Agent.\n\n");

    // Issue context
    prompt.push_str("## Issue to Fix\n\n");
    prompt.push_str(&format!("- **Type:** {:?}\n", issue.issue_type));
    prompt.push_str(&format!("- **Severity:** {:?}\n", issue.severity));
    prompt.push_str(&format!("- **Description:** {}\n", issue.description));
    if let Some(agent) = &issue.agent {
        prompt.push_str(&format!("- **Agent:** {agent}\n"));
    }
    if let Some(task_id) = &issue.task_id {
        prompt.push_str(&format!("- **Task ID:** {task_id}\n"));
    }
    prompt.push_str(&format!("- **Detected at:** {}\n", issue.detected_at));
    prompt.push_str("\n");

    // Retry context
    prompt.push_str("## Retry Status\n\n");
    prompt.push_str(&format!("- **Attempt:** {attempt} of {max_retries}\n"));
    if attempt > 1 {
        prompt.push_str("- **Note:** Previous attempts failed. Try a different approach.\n");
    }
    prompt.push_str("\n");

    // Play context
    prompt.push_str("## Play Session Context\n\n");
    prompt.push_str(&format!("- **Play ID:** {}\n", session.play_id));
    prompt.push_str(&format!("- **Repository:** {}\n", session.repository));
    if let Some(service) = &session.service {
        prompt.push_str(&format!("- **Service:** {service}\n"));
    }
    prompt.push_str("\n");

    // Remediation strategy
    let strategy = RemediationSpawner::get_remediation_strategy(issue);
    prompt.push_str("## Remediation Strategy\n\n");
    match strategy {
        RemediationStrategy::Retry => {
            prompt.push_str("**Strategy: RETRY**\n\n");
            prompt.push_str("1. Analyze the failure logs\n");
            prompt.push_str("2. Identify transient vs persistent issues\n");
            prompt.push_str("3. If transient, trigger a retry of the failed operation\n");
            prompt.push_str("4. If persistent, escalate with detailed analysis\n");
        }
        RemediationStrategy::FixConfig => {
            prompt.push_str("**Strategy: FIX CONFIGURATION**\n\n");
            prompt.push_str("1. Check the CTO config for the affected agent\n");
            prompt.push_str("2. Verify tool declarations match available tools\n");
            prompt.push_str("3. Create a PR to fix the configuration\n");
            prompt.push_str("4. Ensure the fix doesn't break other agents\n");
        }
        RemediationStrategy::FixCode => {
            prompt.push_str("**Strategy: FIX CODE**\n\n");
            prompt.push_str("1. Analyze build/test failure output\n");
            prompt.push_str("2. Identify the root cause\n");
            prompt.push_str("3. Create a PR with the fix\n");
            prompt.push_str("4. Run tests to verify the fix works\n");
        }
        RemediationStrategy::Restart => {
            prompt.push_str("**Strategy: RESTART AGENT**\n\n");
            prompt.push_str("1. Check if the agent is truly stuck (no progress >30 min)\n");
            prompt.push_str("2. Terminate the stuck CodeRun if needed\n");
            prompt.push_str("3. Spawn a fresh CodeRun for the task\n");
            prompt.push_str("4. Monitor for the same issue recurring\n");
        }
        RemediationStrategy::Escalate => {
            prompt.push_str("**Strategy: ESCALATE TO HUMAN**\n\n");
            prompt.push_str("1. This issue requires human intervention\n");
            prompt.push_str("2. Create a detailed GitHub issue\n");
            prompt.push_str("3. Include all relevant context and logs\n");
            prompt.push_str("4. Tag the appropriate team members\n");
        }
    }
    prompt.push_str("\n");

    // Instructions
    prompt.push_str("## Your Task\n\n");
    prompt.push_str("1. Understand the issue from the description and context\n");
    prompt.push_str("2. Follow the remediation strategy above\n");
    prompt.push_str("3. Create a PR if code/config changes are needed\n");
    prompt.push_str("4. Output a JSON summary of your remediation\n\n");

    // Output format
    prompt.push_str("## Output Format\n\n");
    prompt.push_str("Output a JSON summary of your remediation:\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"issue_type\": \"...\",\n");
    prompt.push_str("  \"remediation_time\": \"...\",\n");
    prompt.push_str("  \"status\": \"fixed\" | \"retrying\" | \"escalated\" | \"failed\",\n");
    prompt.push_str("  \"actions_taken\": [\n");
    prompt.push_str("    \"Analyzed logs\",\n");
    prompt.push_str("    \"Created PR #123\",\n");
    prompt.push_str("    \"...\"\n");
    prompt.push_str("  ],\n");
    prompt.push_str("  \"pr_url\": \"https://github.com/...\",\n");
    prompt.push_str("  \"notes\": \"...\"\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n");

    prompt
}

/// Build a `CodeRun` spec for the Remediation Agent.
fn build_remediation_coderun_spec(
    name: &str,
    config: &RemediationSpawnerConfig,
    prompt: &str,
    play_id: &str,
    issue: &SessionIssue,
) -> serde_json::Value {
    let issue_type = format!("{:?}", issue.issue_type).to_lowercase();

    json!({
        "apiVersion": "agents.platform/v1alpha1",
        "kind": "CodeRun",
        "metadata": {
            "name": name,
            "labels": {
                "healer.agents.platform/type": "remediation",
                "healer.agents.platform/play-id": play_id,
                "healer.agents.platform/issue-type": issue_type,
                "agents.platform/service": "healer-remediation"
            }
        },
        "spec": {
            "service": "healer-remediation",
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
    use crate::play::session::{CtoConfig, SessionStatus};
    use chrono::Utc;
    use std::collections::HashMap;

    fn sample_session() -> PlaySession {
        PlaySession {
            play_id: "test-play-1".to_string(),
            repository: "5dlabs/test".to_string(),
            service: Some("test-service".to_string()),
            cto_config: CtoConfig {
                agents: HashMap::new(),
            },
            tasks: vec![],
            namespace: "cto".to_string(),
            started_at: Utc::now(),
            last_updated: Utc::now(),
            issues: vec![],
            status: SessionStatus::Active,
        }
    }

    fn sample_issue(issue_type: IssueType, severity: IssueSeverity) -> SessionIssue {
        SessionIssue {
            detected_at: Utc::now(),
            issue_type,
            severity,
            description: "Test issue description".to_string(),
            agent: Some("rex".to_string()),
            task_id: Some("1".to_string()),
            remediation_spawned: false,
            github_issue: None,
        }
    }

    #[test]
    fn test_remediation_strategy_preflight_failure() {
        let issue = sample_issue(IssueType::PreFlightFailure, IssueSeverity::Critical);
        assert_eq!(
            RemediationSpawner::get_remediation_strategy(&issue),
            RemediationStrategy::FixConfig
        );
    }

    #[test]
    fn test_remediation_strategy_build_failure() {
        let issue = sample_issue(IssueType::BuildFailure, IssueSeverity::High);
        assert_eq!(
            RemediationSpawner::get_remediation_strategy(&issue),
            RemediationStrategy::FixCode
        );
    }

    #[test]
    fn test_remediation_strategy_stuck() {
        let issue = sample_issue(IssueType::Stuck, IssueSeverity::High);
        assert_eq!(
            RemediationSpawner::get_remediation_strategy(&issue),
            RemediationStrategy::Restart
        );
    }

    #[test]
    fn test_build_remediation_prompt() {
        let session = sample_session();
        let issue = sample_issue(IssueType::BuildFailure, IssueSeverity::High);
        let prompt = build_remediation_prompt(&session, &issue, 1, 3);

        assert!(prompt.contains("Healer Remediation Agent"));
        assert!(prompt.contains("BuildFailure"));
        assert!(prompt.contains("test-play-1"));
        assert!(prompt.contains("FIX CODE"));
        assert!(prompt.contains("**Attempt:** 1 of 3"));
    }

    #[test]
    fn test_build_remediation_prompt_retry_note() {
        let session = sample_session();
        let issue = sample_issue(IssueType::AgentFailure, IssueSeverity::High);
        let prompt = build_remediation_prompt(&session, &issue, 2, 3);

        assert!(prompt.contains("**Attempt:** 2 of 3"));
        assert!(prompt.contains("Previous attempts failed"));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("play-123"), "play-123");
        assert_eq!(sanitize_name("BuildFailure"), "buildfailure");
        assert_eq!(sanitize_name("pre_flight_failure"), "pre-flight-failure");
    }
}
