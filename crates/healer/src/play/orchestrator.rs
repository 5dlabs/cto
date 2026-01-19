//! Play Orchestrator for the dual-model Healer architecture.
//!
//! The orchestrator coordinates the feedback loop between:
//! - Evaluation Agent: Detects issues in Play sessions
//! - Remediation Agent: Fixes detected issues
//! - Re-evaluation: Verifies fixes worked
//!
//! This implements the "Feedback Loop" from the Healer architecture.

use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::loki::{LogEntry, LokiClient, LokiConfig};
use crate::scanner::{is_actual_error, is_false_positive};

use super::evaluation_spawner::{
    EvaluationSpawnResult, EvaluationSpawner, EvaluationSpawnerConfig,
};
use super::remediation_spawner::{
    RemediationSpawner, RemediationSpawnerConfig, RemediationStrategy,
};
use super::session::{IssueSeverity, IssueType, PlaySession, SessionIssue, SessionStoreHandle};

/// Truncate a log line for display purposes.
/// Uses character-aware truncation to avoid panicking on multi-byte UTF-8 sequences.
fn truncate_log_line(line: &str, max_len: usize) -> String {
    if line.len() <= max_len {
        line.to_string()
    } else {
        // Find the last valid char boundary at or before max_len bytes
        let truncate_at = line
            .char_indices()
            .take_while(|(idx, _)| *idx < max_len)
            .last()
            .map_or(0, |(idx, ch)| idx + ch.len_utf8());
        format!("{}...", &line[..truncate_at])
    }
}

/// Configuration for the Healer orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Namespace for `CodeRuns`
    pub namespace: String,
    /// Evaluation interval (how often to run evaluation)
    pub evaluation_interval: Duration,
    /// Maximum remediation attempts before escalation
    pub max_remediation_attempts: u32,
    /// Time to wait before re-evaluating after remediation
    pub re_evaluation_delay: Duration,
    /// Whether to automatically escalate on repeated failures
    pub auto_escalate: bool,
    /// Log window to scan for issues
    pub log_scan_window: Duration,
    /// Loki configuration for log queries
    pub loki_config: LokiConfig,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            namespace: "cto".to_string(),
            evaluation_interval: Duration::minutes(5),
            max_remediation_attempts: 3,
            re_evaluation_delay: Duration::minutes(2),
            auto_escalate: true,
            log_scan_window: Duration::minutes(30),
            loki_config: LokiConfig::default(),
        }
    }
}

/// Result of running the feedback loop for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackLoopResult {
    /// Play ID that was evaluated
    pub play_id: String,
    /// Number of issues detected
    pub issues_detected: usize,
    /// Number of remediations spawned
    pub remediations_spawned: usize,
    /// Number of issues fixed
    pub issues_fixed: usize,
    /// Number of issues escalated
    pub issues_escalated: usize,
    /// Whether the session is now healthy
    pub session_healthy: bool,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Healer orchestrator.
///
/// Coordinates the evaluation → remediation → re-evaluation feedback loop.
pub struct HealerOrchestrator {
    config: OrchestratorConfig,
    session_store: SessionStoreHandle,
    evaluation_spawner: EvaluationSpawner,
    remediation_spawner: RemediationSpawner,
    loki: LokiClient,
}

impl HealerOrchestrator {
    /// Create a new orchestrator.
    #[must_use]
    pub fn new(config: OrchestratorConfig, session_store: SessionStoreHandle) -> Self {
        let eval_config = EvaluationSpawnerConfig {
            namespace: config.namespace.clone(),
            ..EvaluationSpawnerConfig::default()
        };
        let evaluation_spawner = EvaluationSpawner::new(eval_config);

        let remediation_config = RemediationSpawnerConfig {
            namespace: config.namespace.clone(),
            max_retries: config.max_remediation_attempts,
            ..RemediationSpawnerConfig::default()
        };
        let remediation_spawner = RemediationSpawner::new(remediation_config);

        let loki = LokiClient::new(config.loki_config.clone());

        Self {
            config,
            session_store,
            evaluation_spawner,
            remediation_spawner,
            loki,
        }
    }

    /// Run the feedback loop for all active sessions.
    pub async fn run_feedback_loop(&self) -> Vec<FeedbackLoopResult> {
        let sessions = self.session_store.get_active_sessions().await;
        info!(
            session_count = sessions.len(),
            "Running feedback loop for active sessions"
        );

        let mut results = Vec::new();
        for session in sessions {
            let result = self.process_session(&session).await;
            results.push(result);
        }

        results
    }

    /// Process a single session through the feedback loop.
    async fn process_session(&self, session: &PlaySession) -> FeedbackLoopResult {
        let play_id = &session.play_id;
        info!(play_id = %play_id, "Processing session");

        let mut result = FeedbackLoopResult {
            play_id: play_id.clone(),
            issues_detected: 0,
            remediations_spawned: 0,
            issues_fixed: 0,
            issues_escalated: 0,
            session_healthy: true,
            errors: Vec::new(),
        };

        // Step 1: Query Loki logs for this session's workflow
        let detected_issues = self.scan_session_logs(session).await;
        if !detected_issues.is_empty() {
            info!(
                play_id = %play_id,
                new_issues = detected_issues.len(),
                "Detected issues from logs"
            );
            // Add newly detected issues to the session store
            for issue in &detected_issues {
                if let Err(e) = self.session_store.add_issue(play_id, issue.clone()).await {
                    warn!(play_id = %play_id, error = %e, "Failed to add issue to session");
                }
            }
        }

        // Step 2: Get all pending issues (including newly detected ones)
        let all_session_issues = if let Some(s) = self.session_store.get_session(play_id).await {
            s.issues
        } else {
            session.issues.clone()
        };

        let pending_issues: Vec<&SessionIssue> = all_session_issues
            .iter()
            .filter(|i| !i.remediation_spawned && i.github_issue.is_none())
            .collect();

        result.issues_detected = pending_issues.len();

        if pending_issues.is_empty() {
            debug!(play_id = %play_id, "No pending issues, session healthy");
            return result;
        }

        info!(
            play_id = %play_id,
            issue_count = pending_issues.len(),
            "Found pending issues"
        );

        // Step 3: Process each issue
        for issue in pending_issues {
            let strategy = RemediationSpawner::get_remediation_strategy(issue);
            info!(
                play_id = %play_id,
                issue_type = ?issue.issue_type,
                strategy = ?strategy,
                "Determined remediation strategy"
            );

            if strategy == RemediationStrategy::Escalate {
                // Don't spawn a CodeRun, escalate to humans
                result.issues_escalated += 1;
                Self::escalate_issue(session, issue);
            } else {
                // Spawn a remediation CodeRun
                let attempt = Self::get_remediation_attempt_count(session, issue);
                match self
                    .remediation_spawner
                    .spawn_remediation(session, issue, attempt)
                {
                    Ok(spawn_result) => {
                        if spawn_result.success {
                            result.remediations_spawned += 1;
                            info!(
                                play_id = %play_id,
                                coderun = %spawn_result.coderun_name,
                                "Remediation CodeRun spawned"
                            );
                        } else if spawn_result
                            .error
                            .as_ref()
                            .is_some_and(|e| e.contains("Max retries") || e.contains("exceeded"))
                        {
                            result.issues_escalated += 1;
                            Self::escalate_issue(session, issue);
                        } else {
                            result.errors.push(
                                spawn_result
                                    .error
                                    .unwrap_or_else(|| "Unknown error".to_string()),
                            );
                        }
                    }
                    Err(e) => {
                        result.errors.push(e.to_string());
                        error!(
                            play_id = %play_id,
                            error = %e,
                            "Failed to spawn remediation"
                        );
                    }
                }
            }
        }

        result.session_healthy = result.errors.is_empty() && result.issues_escalated == 0;
        result
    }

    /// Scan Loki logs for a session and detect issues.
    async fn scan_session_logs(&self, session: &PlaySession) -> Vec<SessionIssue> {
        let play_id = &session.play_id;
        let namespace = &session.namespace;
        let end = Utc::now();
        let start = end - self.config.log_scan_window;

        // Query logs for the play workflow (pods matching play-{play_id}*)
        let workflow_pattern = format!("play-{play_id}");
        let query_result = self
            .loki
            .query_workflow_logs(namespace, &workflow_pattern, start, end, 5000)
            .await;

        let logs = match query_result {
            Ok(entries) => entries,
            Err(e) => {
                warn!(
                    play_id = %play_id,
                    error = %e,
                    "Failed to query Loki logs"
                );
                return Vec::new();
            }
        };

        if logs.is_empty() {
            debug!(play_id = %play_id, "No logs found for session");
            return Vec::new();
        }

        info!(
            play_id = %play_id,
            log_count = logs.len(),
            "Retrieved logs from Loki"
        );

        // Filter for actual errors
        let error_logs = Self::filter_error_logs(&logs);

        // Convert error logs to issues
        self.logs_to_issues(session, &error_logs)
    }

    /// Filter log entries for actual errors (not false positives).
    fn filter_error_logs(logs: &[LogEntry]) -> Vec<&LogEntry> {
        logs.iter()
            .filter(|entry| {
                // Skip false positives
                if is_false_positive(&entry.line) {
                    return false;
                }
                // Keep actual errors
                is_actual_error(&entry.line)
            })
            .collect()
    }

    /// Convert error log entries into session issues.
    #[allow(clippy::too_many_lines, clippy::unused_self)]
    fn logs_to_issues(
        &self,
        _session: &PlaySession,
        error_logs: &[&LogEntry],
    ) -> Vec<SessionIssue> {
        if error_logs.is_empty() {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Group errors by type
        let tool_mismatch_errors: Vec<_> = error_logs
            .iter()
            .filter(|e| {
                let line = e.line.to_lowercase();
                line.contains("tool inventory mismatch")
                    || line.contains("missing from cli")
                    || line.contains("tool not found")
            })
            .collect();

        let config_errors: Vec<_> = error_logs
            .iter()
            .filter(|e| {
                let line = e.line.to_lowercase();
                line.contains("cto-config")
                    || line.contains("config missing")
                    || line.contains("config invalid")
            })
            .collect();

        let mcp_errors: Vec<_> = error_logs
            .iter()
            .filter(|e| {
                let line = e.line.to_lowercase();
                line.contains("mcp") && (line.contains("failed") || line.contains("unreachable"))
            })
            .collect();

        let general_errors: Vec<_> = error_logs
            .iter()
            .filter(|e| {
                let line = e.line.to_lowercase();
                // Exclude all patterns that match specific error types to prevent duplicate issues
                !line.contains("tool inventory")
                    && !line.contains("missing from cli")
                    && !line.contains("tool not found")
                    && !line.contains("cto-config")
                    && !line.contains("config missing")
                    && !line.contains("config invalid")
                    && !(line.contains("mcp")
                        && (line.contains("failed") || line.contains("unreachable")))
            })
            .collect();

        // Create issues for each error type
        if !tool_mismatch_errors.is_empty() {
            let sample = tool_mismatch_errors
                .first()
                .map_or("Tool inventory mismatch detected", |e| e.line.as_str());
            issues.push(SessionIssue {
                issue_type: IssueType::ToolMismatch,
                severity: IssueSeverity::Critical,
                description: format!(
                    "Tool inventory mismatch: {} occurrences. Sample: {}",
                    tool_mismatch_errors.len(),
                    truncate_log_line(sample, 200)
                ),
                detected_at: Utc::now(),
                agent: None,
                task_id: None,
                remediation_spawned: false,
                github_issue: None,
            });
        }

        if !config_errors.is_empty() {
            let sample = config_errors
                .first()
                .map_or("CTO config error detected", |e| e.line.as_str());
            issues.push(SessionIssue {
                issue_type: IssueType::ConfigError,
                severity: IssueSeverity::Critical,
                description: format!(
                    "CTO config error: {} occurrences. Sample: {}",
                    config_errors.len(),
                    truncate_log_line(sample, 200)
                ),
                detected_at: Utc::now(),
                agent: None,
                task_id: None,
                remediation_spawned: false,
                github_issue: None,
            });
        }

        if !mcp_errors.is_empty() {
            let sample = mcp_errors
                .first()
                .map_or("MCP initialization error", |e| e.line.as_str());
            issues.push(SessionIssue {
                issue_type: IssueType::McpInitFailed,
                severity: IssueSeverity::High,
                description: format!(
                    "MCP initialization failed: {} occurrences. Sample: {}",
                    mcp_errors.len(),
                    truncate_log_line(sample, 200)
                ),
                detected_at: Utc::now(),
                agent: None,
                task_id: None,
                remediation_spawned: false,
                github_issue: None,
            });
        }

        if !general_errors.is_empty() {
            // Group general errors - create at most one issue for general errors
            let sample = general_errors
                .first()
                .map_or("Error detected in logs", |e| e.line.as_str());
            issues.push(SessionIssue {
                issue_type: IssueType::AgentError,
                severity: if general_errors.len() > 5 {
                    IssueSeverity::High
                } else {
                    IssueSeverity::Medium
                },
                description: format!(
                    "Agent errors: {} occurrences. Sample: {}",
                    general_errors.len(),
                    truncate_log_line(sample, 200)
                ),
                detected_at: Utc::now(),
                agent: None,
                task_id: None,
                remediation_spawned: false,
                github_issue: None,
            });
        }

        issues
    }

    /// Escalate an issue to humans.
    ///
    /// This will:
    /// 1. Create a GitHub issue for tracking
    /// 2. Send Discord notification via the notify crate
    fn escalate_issue(session: &PlaySession, issue: &SessionIssue) {
        warn!(
            play_id = %session.play_id,
            issue_type = ?issue.issue_type,
            severity = ?issue.severity,
            "Escalating issue to humans"
        );

        // Create GitHub issue for tracking
        if let Err(e) = Self::create_github_issue_for_play(session, issue) {
            warn!(
                play_id = %session.play_id,
                error = %e,
                "Failed to create GitHub issue for escalation"
            );
        }

        // Send notification via the notify crate
        let notifier = notify::Notifier::from_env();
        let mut context = std::collections::HashMap::new();
        context.insert("play_id".to_string(), session.play_id.clone());
        context.insert("repository".to_string(), session.repository.clone());
        context.insert(
            "service".to_string(),
            session
                .service
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
        );
        context.insert("issue_type".to_string(), format!("{:?}", issue.issue_type));
        context.insert("severity".to_string(), format!("{:?}", issue.severity));
        context.insert("description".to_string(), issue.description.clone());

        // Map IssueSeverity to notify::Severity (only has Info, Warning, Critical)
        let notify_severity = match issue.severity {
            IssueSeverity::Critical | IssueSeverity::High => notify::Severity::Critical,
            IssueSeverity::Medium => notify::Severity::Warning,
            IssueSeverity::Low => notify::Severity::Info,
        };

        let event = notify::NotifyEvent::HealAlert {
            alert_id: format!("PLAY-{}-{:?}", session.play_id, issue.issue_type),
            severity: notify_severity,
            message: format!(
                "Play {} needs attention: {:?} - {}",
                session.play_id, issue.issue_type, issue.description
            ),
            context,
            timestamp: Utc::now(),
        };

        notifier.notify(event);
        info!(
            play_id = %session.play_id,
            "Escalation notification sent (Discord + GitHub issue)"
        );
    }

    /// Create a GitHub issue for a Play escalation.
    fn create_github_issue_for_play(
        session: &PlaySession,
        issue: &SessionIssue,
    ) -> std::result::Result<(), String> {
        use std::fmt::Write as _;
        use std::process::Command;

        let title = format!(
            "[Healer] Play {} - {:?} Escalation",
            session.play_id, issue.issue_type
        );

        // Build the issue body
        let mut body = String::new();
        body.push_str("## 🚨 Play Workflow Escalation\n\n");
        let _ = writeln!(
            body,
            "Automated remediation could not resolve this issue.\n"
        );

        body.push_str("### Play Details\n\n");
        let _ = writeln!(body, "- **Play ID**: {}", session.play_id);
        let _ = writeln!(body, "- **Repository**: `{}`", session.repository);
        if let Some(service) = &session.service {
            let _ = writeln!(body, "- **Service**: `{service}`");
        }
        let _ = writeln!(body, "- **Namespace**: `{}`", session.namespace);
        let _ = writeln!(
            body,
            "- **Started**: {}",
            session.started_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        body.push_str("\n### Issue Details\n\n");
        let _ = writeln!(body, "- **Type**: `{:?}`", issue.issue_type);
        let _ = writeln!(body, "- **Severity**: `{:?}`", issue.severity);
        let _ = writeln!(
            body,
            "- **Detected**: {}",
            issue.detected_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        if let Some(agent) = &issue.agent {
            let _ = writeln!(body, "- **Agent**: {agent}");
        }
        if let Some(task_id) = &issue.task_id {
            let _ = writeln!(body, "- **Task**: {task_id}");
        }

        body.push_str("\n### Description\n\n");
        body.push_str(&issue.description);
        body.push_str("\n\n");

        // Add tasks summary if available
        if !session.tasks.is_empty() {
            body.push_str("### Tasks in This Play\n\n");
            for task in &session.tasks {
                let agent_hint = task.agent_hint.as_deref().unwrap_or("unassigned");
                let _ = writeln!(
                    body,
                    "- **Task {}**: {} (agent: {})",
                    task.id, task.title, agent_hint
                );
            }
            body.push('\n');
        }

        body.push_str("---\n");
        body.push_str("*This issue requires manual intervention. ");
        body.push_str("Please investigate and fix the root cause.*\n");
        body.push_str("\n_Generated by Healer Play Monitor_\n");

        // Determine which labels to use based on issue type
        let labels = match issue.issue_type {
            IssueType::ToolMismatch | IssueType::ConfigError | IssueType::McpInitFailed => {
                "healer,play-workflow,config-issue,needs-attention"
            }
            IssueType::AgentError | IssueType::AgentFailure => {
                "healer,play-workflow,agent-error,needs-attention"
            }
            IssueType::BuildFailure | IssueType::TestFailure => {
                "healer,play-workflow,ci-failure,needs-attention"
            }
            _ => "healer,play-workflow,needs-attention",
        };

        // Create the issue using gh CLI
        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                &session.repository,
                "--title",
                &title,
                "--body",
                &body,
                "--label",
                labels,
            ])
            .output()
            .map_err(|e| format!("Failed to execute gh issue create: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("gh issue create failed: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        info!(
            "Created GitHub issue for Play escalation: {}",
            stdout.trim()
        );

        Ok(())
    }

    /// Get the number of remediation attempts for an issue.
    // TODO: Will use session state when attempt tracking is implemented
    fn get_remediation_attempt_count(_session: &PlaySession, _issue: &SessionIssue) -> u32 {
        // TODO: Track attempts in session state
        // For now, return 1 (first attempt)
        1
    }

    /// Run a single evaluation cycle (spawn Evaluation Agent).
    ///
    /// # Errors
    ///
    /// Returns an error if the evaluation cannot be spawned.
    pub fn run_evaluation(&self, session: &PlaySession) -> Result<EvaluationSpawnResult> {
        info!(
            play_id = %session.play_id,
            "Running evaluation for session"
        );

        self.evaluation_spawner.spawn_evaluation(session)
    }

    /// Check if a session should be re-evaluated after remediation.
    #[must_use]
    pub fn should_re_evaluate(&self, session: &PlaySession) -> bool {
        // Check if any issues have been remediated recently
        let re_eval_window = Utc::now() - self.config.re_evaluation_delay;

        session.issues.iter().any(|issue| {
            issue.remediation_spawned
                && issue.github_issue.is_none()
                && issue.detected_at < re_eval_window
        })
    }

    /// Get the orchestrator configuration.
    #[must_use]
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }
}

/// Language matching verification for quality/security/testing agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationLanguage {
    Rust,
    Go,
    TypeScript,
    Python,
    CSharp,
    Unknown,
}

impl ImplementationLanguage {
    /// Get the language for an agent.
    #[must_use]
    pub fn for_agent(agent: &str) -> Self {
        match agent.to_lowercase().as_str() {
            "rex" => Self::Rust,
            "grizz" => Self::Go,
            "nova" | "blaze" | "tap" | "spark" => Self::TypeScript,
            "vex" => Self::CSharp,
            _ => Self::Unknown,
        }
    }

    /// Get expected quality tools for this language.
    #[must_use]
    pub fn quality_tools(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["cargo clippy", "cargo fmt", "rust-analyzer"],
            Self::Go => vec!["golint", "gofmt", "go vet", "staticcheck"],
            Self::TypeScript => vec!["eslint", "prettier", "biome", "tsc"],
            Self::Python => vec!["ruff", "black", "mypy", "pylint"],
            Self::CSharp => vec!["dotnet format", "StyleCop", "Roslynator"],
            Self::Unknown => vec![],
        }
    }

    /// Get expected security tools for this language.
    #[must_use]
    pub fn security_tools(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["cargo audit", "cargo deny", "rustsec"],
            Self::Go => vec!["gosec", "govulncheck", "nancy"],
            Self::TypeScript => vec!["npm audit", "snyk", "audit-ci"],
            Self::Python => vec!["bandit", "safety", "pip-audit"],
            Self::CSharp => vec!["dotnet list package --vulnerable", "security-code-scan"],
            Self::Unknown => vec![],
        }
    }

    /// Get expected testing tools for this language.
    #[must_use]
    pub fn testing_tools(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["cargo test", "cargo nextest", "cargo tarpaulin"],
            Self::Go => vec!["go test", "gotest", "gotestsum"],
            Self::TypeScript => vec!["vitest", "jest", "playwright", "cypress"],
            Self::Python => vec!["pytest", "unittest", "coverage"],
            Self::CSharp => vec!["dotnet test", "xunit", "NUnit"],
            Self::Unknown => vec![],
        }
    }
}

/// Verify that an agent is using the correct language tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageMatchResult {
    /// The implementation agent
    pub implementation_agent: String,
    /// Expected language
    pub expected_language: String,
    /// The agent being verified (cleo, cipher, tess)
    pub verification_agent: String,
    /// Expected tools for this language
    pub expected_tools: Vec<String>,
    /// Whether the match is correct
    pub matches: bool,
    /// Description of any mismatch
    pub mismatch_description: Option<String>,
}

/// Check language matching for a quality/security/testing agent.
#[must_use]
pub fn verify_language_match(
    implementation_agent: &str,
    verification_agent: &str,
    tools_in_use: &[String],
) -> LanguageMatchResult {
    let lang = ImplementationLanguage::for_agent(implementation_agent);
    let expected_tools = match verification_agent.to_lowercase().as_str() {
        "cleo" => lang.quality_tools(),
        "cipher" => lang.security_tools(),
        "tess" => lang.testing_tools(),
        _ => vec![],
    };

    // Check if any expected tools are being used
    let matches = expected_tools.is_empty()
        || tools_in_use
            .iter()
            .any(|t| expected_tools.iter().any(|e| t.contains(e)));

    let mismatch_description = if matches {
        None
    } else {
        Some(format!(
            "Expected {} tools ({:?}) but found {:?}",
            format!("{lang:?}").to_lowercase(),
            expected_tools,
            tools_in_use
        ))
    };

    LanguageMatchResult {
        implementation_agent: implementation_agent.to_string(),
        expected_language: format!("{lang:?}"),
        verification_agent: verification_agent.to_string(),
        expected_tools: expected_tools.iter().map(|s| (*s).to_string()).collect(),
        matches,
        mismatch_description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_for_agent() {
        assert_eq!(
            ImplementationLanguage::for_agent("rex"),
            ImplementationLanguage::Rust
        );
        assert_eq!(
            ImplementationLanguage::for_agent("grizz"),
            ImplementationLanguage::Go
        );
        assert_eq!(
            ImplementationLanguage::for_agent("blaze"),
            ImplementationLanguage::TypeScript
        );
        assert_eq!(
            ImplementationLanguage::for_agent("nova"),
            ImplementationLanguage::TypeScript
        );
        assert_eq!(
            ImplementationLanguage::for_agent("vex"),
            ImplementationLanguage::CSharp
        );
        assert_eq!(
            ImplementationLanguage::for_agent("unknown"),
            ImplementationLanguage::Unknown
        );
    }

    #[test]
    fn test_rust_quality_tools() {
        let tools = ImplementationLanguage::Rust.quality_tools();
        assert!(tools.contains(&"cargo clippy"));
        assert!(tools.contains(&"cargo fmt"));
    }

    #[test]
    fn test_go_security_tools() {
        let tools = ImplementationLanguage::Go.security_tools();
        assert!(tools.contains(&"gosec"));
        assert!(tools.contains(&"govulncheck"));
    }

    #[test]
    fn test_truncate_log_line_ascii() {
        // ASCII text - should truncate at exact byte boundary
        let line = "This is a test log line that exceeds the maximum length";
        let truncated = truncate_log_line(line, 20);
        assert!(truncated.starts_with("This is a test"));
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 23); // max_len + "..."
    }

    #[test]
    fn test_truncate_log_line_emoji() {
        // Emoji are multi-byte UTF-8 characters
        // 🚀 is 4 bytes, 😀 is 4 bytes, 👍 is 4 bytes
        let line = "Starting process 🚀 with status 😀 and feedback 👍 complete";

        // Truncate at 25 bytes - should not panic even if it falls mid-emoji
        let truncated = truncate_log_line(line, 25);

        // Should not panic and should produce valid UTF-8
        assert!(truncated.ends_with("..."));

        // Verify it's valid UTF-8 by converting back from string
        let _chars: Vec<char> = truncated.chars().collect();
    }

    #[test]
    fn test_truncate_log_line_chinese() {
        // Chinese characters are typically 3 bytes each in UTF-8
        let line = "日志条目：处理完成，状态正常";

        // Truncate at 10 bytes - may fall in middle of multi-byte character
        let truncated = truncate_log_line(line, 10);

        // Should not panic and should produce valid UTF-8
        assert!(truncated.ends_with("..."));

        // Verify all characters are valid
        for ch in truncated.chars() {
            assert!(ch.is_alphabetic() || ch == '.' || ch == '：');
        }
    }

    #[test]
    fn test_truncate_log_line_mixed_unicode() {
        // Mix of ASCII, emoji, and non-ASCII characters
        let line = "Error in module 模块 🔥 failed with code 错误代码 123";

        // Try various truncation lengths
        for max_len in [10, 20, 30, 40] {
            let truncated = truncate_log_line(line, max_len);

            // Should never panic regardless of where we truncate
            // The key is that it produces valid UTF-8, not exact length
            // (may be slightly shorter than max_len to preserve full characters)
            if line.len() > max_len {
                assert!(
                    truncated.ends_with("..."),
                    "Should have ellipsis for truncated line"
                );
                // Truncated string should be close to max_len but may vary by a few bytes
                // due to character boundaries
                assert!(
                    truncated.len() < line.len(),
                    "Should be shorter than original"
                );
            } else {
                assert_eq!(truncated, line, "Short lines should be unchanged");
            }

            // Verify it's valid UTF-8
            let _chars: Vec<char> = truncated.chars().collect();
        }
    }

    #[test]
    fn test_truncate_log_line_short() {
        // Line shorter than max_len should be returned unchanged
        let line = "Short line";
        let truncated = truncate_log_line(line, 100);
        assert_eq!(truncated, line);
    }

    #[test]
    fn test_verify_language_match_success() {
        let result =
            verify_language_match("rex", "cleo", &["cargo clippy --all-targets".to_string()]);
        assert!(result.matches);
        assert!(result.mismatch_description.is_none());
        assert_eq!(result.expected_language, "Rust");
    }

    #[test]
    fn test_verify_language_match_failure() {
        let result = verify_language_match(
            "rex",
            "cleo",
            &["eslint src/".to_string(), "prettier".to_string()],
        );
        // eslint/prettier are not Rust tools, but we don't fail just because they're present
        // The check is if ANY expected tools are found, so this should fail
        assert!(!result.matches);
        assert!(result.mismatch_description.is_some());
    }

    #[test]
    fn test_verify_language_match_mixed_tools() {
        let result = verify_language_match(
            "blaze",
            "tess",
            &[
                "vitest run".to_string(),
                "cargo test".to_string(), // This shouldn't matter
            ],
        );
        assert!(result.matches); // vitest is a TypeScript testing tool
    }

    #[test]
    fn test_unknown_agent_language() {
        let result = verify_language_match("unknown-agent", "cleo", &["some-tool".to_string()]);
        assert!(result.matches); // Unknown language has no expected tools
        assert_eq!(result.expected_language, "Unknown");
    }
}
