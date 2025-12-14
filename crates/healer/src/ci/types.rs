//! Core types for CI remediation.
//!
//! This module defines the primary data structures for:
//! - Representing CI failures from webhooks
//! - Classifying failure types for routing
//! - Tracking remediation attempts and outcomes
//! - Agent specializations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Specialist agent identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    /// Rust specialist (Clippy, tests, builds, Cargo)
    Rex,
    /// Frontend specialist (JS, TS, npm, React)
    Blaze,
    /// Infrastructure specialist (Docker, Helm, K8s, `ArgoCD`, `GitOps`)
    Bolt,
    /// Security specialist (Dependabot, code scanning, secrets)
    Cipher,
    /// GitHub/Git specialist and fallback (merge conflicts, workflows)
    Atlas,
}

impl Agent {
    /// Get the agent's name as a string.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Rex => "rex",
            Self::Blaze => "blaze",
            Self::Bolt => "bolt",
            Self::Cipher => "cipher",
            Self::Atlas => "atlas",
        }
    }

    /// Get the GitHub App name for this agent.
    #[must_use]
    pub fn github_app(self) -> &'static str {
        match self {
            Self::Rex => "5DLabs-Rex",
            Self::Blaze => "5DLabs-Blaze",
            Self::Bolt => "5DLabs-Bolt",
            Self::Cipher => "5DLabs-Cipher",
            Self::Atlas => "5DLabs-Atlas",
        }
    }

    /// Get the default model for this agent.
    #[must_use]
    pub fn model(self) -> &'static str {
        // All agents use Opus 4.5 for initial implementation
        "claude-opus-4-5-20250929"
    }

    /// Get the prompt template name for this agent.
    #[must_use]
    pub fn template_name(self) -> &'static str {
        match self {
            Self::Rex => "rust-fix",
            Self::Blaze => "frontend-fix",
            Self::Bolt => "infra-fix",
            Self::Cipher => "security-fix",
            Self::Atlas => "github-fix",
        }
    }
}

/// Classification of CI failure types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiFailureType {
    // Rust failures
    /// Clippy lint errors
    RustClippy,
    /// Rust test failures
    RustTest,
    /// Cargo build errors
    RustBuild,
    /// Cargo dependency issues
    RustDeps,

    // Frontend failures
    /// npm/pnpm install failures
    FrontendDeps,
    /// TypeScript compilation errors
    FrontendTypeScript,
    /// `ESLint` errors
    FrontendLint,
    /// Frontend test failures
    FrontendTest,
    /// Frontend build failures
    FrontendBuild,

    // Infrastructure failures
    /// Docker build errors
    DockerBuild,
    /// Helm template errors
    HelmTemplate,
    /// Kubernetes manifest issues
    K8sManifest,
    /// `ArgoCD` sync failures
    ArgoCdSync,
    /// YAML syntax errors
    YamlSyntax,

    // Security failures
    /// Dependabot security alert
    SecurityDependabot,
    /// Code scanning alert
    SecurityCodeScan,
    /// Secret scanning alert
    SecuritySecret,

    // Git/GitHub failures
    /// Merge conflicts
    GitMergeConflict,
    /// GitHub Actions workflow syntax
    GithubWorkflow,
    /// Git permission issues
    GitPermission,

    // General
    /// Unclassified failure (routes to Atlas)
    General,
}

impl CiFailureType {
    /// Get a short name for logging/labels.
    #[must_use]
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::RustClippy => "clippy",
            Self::RustTest => "rust-test",
            Self::RustBuild => "rust-build",
            Self::RustDeps => "rust-deps",
            Self::FrontendDeps => "fe-deps",
            Self::FrontendTypeScript => "typescript",
            Self::FrontendLint => "eslint",
            Self::FrontendTest => "fe-test",
            Self::FrontendBuild => "fe-build",
            Self::DockerBuild => "docker",
            Self::HelmTemplate => "helm",
            Self::K8sManifest => "k8s",
            Self::ArgoCdSync => "argocd",
            Self::YamlSyntax => "yaml",
            Self::SecurityDependabot => "dependabot",
            Self::SecurityCodeScan => "codescan",
            Self::SecuritySecret => "secret",
            Self::GitMergeConflict => "merge",
            Self::GithubWorkflow => "workflow",
            Self::GitPermission => "git-perm",
            Self::General => "general",
        }
    }

    /// Get the category for grouping.
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::RustClippy | Self::RustTest | Self::RustBuild | Self::RustDeps => "rust",
            Self::FrontendDeps
            | Self::FrontendTypeScript
            | Self::FrontendLint
            | Self::FrontendTest
            | Self::FrontendBuild => "frontend",
            Self::DockerBuild
            | Self::HelmTemplate
            | Self::K8sManifest
            | Self::ArgoCdSync
            | Self::YamlSyntax => "infrastructure",
            Self::SecurityDependabot | Self::SecurityCodeScan | Self::SecuritySecret => "security",
            Self::GitMergeConflict | Self::GithubWorkflow | Self::GitPermission => "github",
            Self::General => "general",
        }
    }

    /// Check if this is a Rust-related failure.
    #[must_use]
    pub fn is_rust(&self) -> bool {
        self.category() == "rust"
    }

    /// Check if this is a frontend-related failure.
    #[must_use]
    pub fn is_frontend(&self) -> bool {
        self.category() == "frontend"
    }

    /// Check if this is an infrastructure-related failure.
    #[must_use]
    pub fn is_infra(&self) -> bool {
        self.category() == "infrastructure"
    }

    /// Check if this is a security-related failure.
    #[must_use]
    pub fn is_security(&self) -> bool {
        self.category() == "security"
    }

    /// Check if this is a merge conflict.
    #[must_use]
    pub fn is_merge_conflict(&self) -> bool {
        matches!(self, Self::GitMergeConflict)
    }

    /// Suggested fix approach for prompts.
    #[must_use]
    pub fn fix_approach(&self) -> &'static str {
        match self {
            Self::RustClippy => "Fix Clippy lints, run `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`",
            Self::RustTest => "Fix failing tests, ensure `cargo test` passes",
            Self::RustBuild => "Fix compilation errors, ensure `cargo build` succeeds",
            Self::RustDeps => "Resolve dependency conflicts in Cargo.toml/Cargo.lock",
            Self::FrontendDeps => "Fix npm/pnpm dependency issues, update package.json or lockfile",
            Self::FrontendTypeScript => "Fix TypeScript errors, ensure `tsc` passes",
            Self::FrontendLint => "Fix ESLint errors, run `npm run lint`",
            Self::FrontendTest => "Fix failing frontend tests",
            Self::FrontendBuild => "Fix frontend build errors",
            Self::DockerBuild => "Fix Dockerfile issues, ensure image builds",
            Self::HelmTemplate => "Fix Helm chart template errors, run `helm template`",
            Self::K8sManifest => "Fix Kubernetes manifest syntax/schema",
            Self::ArgoCdSync => "Resolve ArgoCD sync issues, check app health",
            Self::YamlSyntax => "Fix YAML syntax errors, validate with yamllint",
            Self::SecurityDependabot => "Update vulnerable dependency to patched version",
            Self::SecurityCodeScan => "Fix code scanning alert, address security issue",
            Self::SecuritySecret => "Rotate leaked secret, update references",
            Self::GitMergeConflict => "Resolve merge conflicts, reconcile changes",
            Self::GithubWorkflow => "Fix GitHub Actions workflow syntax",
            Self::GitPermission => "Resolve Git permission issues",
            Self::General => "Analyze CI logs and fix the root cause",
        }
    }
}

/// CI failure event received from webhook/sensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiFailure {
    /// Unique workflow run ID
    pub workflow_run_id: u64,
    /// Workflow name (e.g., "Controller CI")
    pub workflow_name: String,
    /// Job name within the workflow (e.g., "lint-rust")
    pub job_name: Option<String>,
    /// Conclusion of the workflow/job
    pub conclusion: String,
    /// Branch where the failure occurred
    pub branch: String,
    /// Commit SHA
    pub head_sha: String,
    /// Commit message
    pub commit_message: String,
    /// URL to the workflow run
    pub html_url: String,
    /// Repository full name (e.g., "5dlabs/cto")
    pub repository: String,
    /// Sender/author of the commit
    pub sender: String,
    /// When the failure was detected
    pub detected_at: DateTime<Utc>,
    /// Raw event payload for additional context
    #[serde(default)]
    pub raw_event: Option<serde_json::Value>,
}

impl CiFailure {
    /// Create a new CI failure from a workflow job event.
    #[must_use]
    pub fn from_workflow_job(event: &serde_json::Value) -> Option<Self> {
        let job = event.get("workflow_job")?;
        let repo = event.get("repository")?;

        Some(Self {
            workflow_run_id: job.get("run_id")?.as_u64()?,
            workflow_name: job
                .get("workflow_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            job_name: job.get("name").and_then(|v| v.as_str()).map(String::from),
            conclusion: job
                .get("conclusion")
                .and_then(|v| v.as_str())
                .unwrap_or("failure")
                .to_string(),
            branch: job
                .get("head_branch")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            head_sha: job
                .get("head_sha")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            commit_message: event
                .get("workflow_job")
                .and_then(|j| j.get("head_commit"))
                .and_then(|c| c.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            html_url: job
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repository: repo
                .get("full_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            sender: event
                .get("sender")
                .and_then(|s| s.get("login"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            detected_at: Utc::now(),
            raw_event: Some(event.clone()),
        })
    }

    /// Create a new CI failure from a check run event.
    #[must_use]
    pub fn from_check_run(event: &serde_json::Value) -> Option<Self> {
        let check_run = event.get("check_run")?;
        let repo = event.get("repository")?;
        let check_suite = check_run.get("check_suite")?;

        Some(Self {
            workflow_run_id: check_run.get("id")?.as_u64()?,
            workflow_name: check_run
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            job_name: None,
            conclusion: check_run
                .get("conclusion")
                .and_then(|v| v.as_str())
                .unwrap_or("failure")
                .to_string(),
            branch: check_suite
                .get("head_branch")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            head_sha: check_suite
                .get("head_sha")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            commit_message: String::new(),
            html_url: check_run
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repository: repo
                .get("full_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            sender: event
                .get("sender")
                .and_then(|s| s.get("login"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            detected_at: Utc::now(),
            raw_event: Some(event.clone()),
        })
    }

    /// Get an error summary for memory queries.
    #[must_use]
    pub fn error_summary(&self) -> String {
        format!(
            "{} failed in {} on branch {}",
            self.job_name.as_deref().unwrap_or(&self.workflow_name),
            self.workflow_name,
            self.branch
        )
    }
}

/// Security alert event from GitHub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    /// Alert type (dependabot, `code_scanning`, `secret_scanning`)
    pub alert_type: String,
    /// Alert severity
    pub severity: String,
    /// Affected package (for dependency alerts)
    pub package_name: Option<String>,
    /// CVE identifier (if applicable)
    pub cve_id: Option<String>,
    /// Alert description
    pub description: String,
    /// Repository
    pub repository: String,
    /// Branch (if applicable)
    pub branch: Option<String>,
    /// Alert URL
    pub html_url: String,
    /// When detected
    pub detected_at: DateTime<Utc>,
}

/// Full context gathered for routing and prompting.
#[derive(Debug, Clone, Default)]
pub struct RemediationContext {
    /// The original CI failure event
    pub failure: Option<CiFailure>,
    /// Security alert (if applicable)
    pub security_alert: Option<SecurityAlert>,
    /// Classified failure type
    pub failure_type: Option<CiFailureType>,
    /// Workflow logs
    pub workflow_logs: String,
    /// PR associated with this failure (if any)
    pub pr: Option<PullRequest>,
    /// Changed files in the failing commit
    pub changed_files: Vec<ChangedFile>,
    /// `ArgoCD` application status (if relevant)
    pub argocd_status: Option<ArgoCdStatus>,
    /// Recent error logs from Loki
    pub recent_logs: String,
    /// Pod state from Kubernetes
    pub pod_state: Option<PodState>,
    /// Error rate metrics from Prometheus
    pub error_rate: Option<f64>,
    /// Historical context from `OpenMemory`
    pub historical: Option<HistoricalContext>,
    /// Previous remediation attempts (for retries)
    pub previous_attempts: Vec<RemediationAttempt>,
    /// Agent's failure output (for retries)
    pub agent_failure_output: Option<String>,
    /// Changes made in previous attempts
    pub changes_made_so_far: Vec<CommitInfo>,
}

impl RemediationContext {
    /// Check if this context indicates a security event.
    #[must_use]
    pub fn is_security_event(&self) -> bool {
        self.security_alert.is_some()
            || self
                .failure_type
                .as_ref()
                .is_some_and(CiFailureType::is_security)
    }

    /// Get a task ID for tracking this remediation.
    #[must_use]
    pub fn task_id(&self) -> String {
        if let Some(failure) = &self.failure {
            format!("ci-{}", failure.workflow_run_id)
        } else if let Some(alert) = &self.security_alert {
            format!("sec-{}-{}", alert.alert_type, alert.detected_at.timestamp())
        } else {
            format!("unknown-{}", Utc::now().timestamp())
        }
    }

    /// Summarize the diff for prompts.
    #[must_use]
    pub fn summarize_diff(&self) -> String {
        use std::fmt::Write as _;

        if self.changed_files.is_empty() {
            return "No changed files information available".to_string();
        }

        let mut summary = String::new();
        for file in &self.changed_files {
            let _ = writeln!(
                summary,
                "- {} ({}, +{} -{} lines)",
                file.filename, file.status, file.additions, file.deletions
            );
        }
        summary
    }
}

/// Pull request information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR number
    pub number: u32,
    /// PR title
    pub title: String,
    /// PR state (open, closed, merged)
    pub state: String,
    /// Head branch
    pub head_ref: String,
    /// Base branch
    pub base_ref: String,
    /// Whether PR is mergeable
    pub mergeable: Option<bool>,
    /// Check status summary
    pub checks_status: String,
    /// PR URL
    pub html_url: String,
}

/// Changed file information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    /// File path
    pub filename: String,
    /// Change status (added, modified, deleted, renamed)
    pub status: String,
    /// Lines added
    pub additions: u32,
    /// Lines deleted
    pub deletions: u32,
}

impl ChangedFile {
    /// Check if this is a Rust file.
    #[must_use]
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    pub fn is_rust(&self) -> bool {
        self.filename.ends_with(".rs")
            || self.filename == "Cargo.toml"
            || self.filename == "Cargo.lock"
    }

    /// Check if this is a frontend file.
    #[must_use]
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    pub fn is_frontend(&self) -> bool {
        self.filename.ends_with(".ts")
            || self.filename.ends_with(".tsx")
            || self.filename.ends_with(".js")
            || self.filename.ends_with(".jsx")
            || self.filename.ends_with(".css")
            || self.filename.ends_with(".scss")
            || self.filename == "package.json"
            || self.filename == "pnpm-lock.yaml"
    }

    /// Check if this is an infrastructure file.
    #[must_use]
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    pub fn is_infra(&self) -> bool {
        self.filename.starts_with("infra/")
            || self.filename.starts_with(".github/")
            || self.filename.ends_with(".yaml")
            || self.filename.ends_with(".yml")
            || self.is_dockerfile()
            || self.filename == "Chart.yaml"
    }

    /// Check if this is a Dockerfile.
    #[must_use]
    fn is_dockerfile(&self) -> bool {
        // Match: "Dockerfile", "Dockerfile.prod", "path/to/Dockerfile", "path/to/Dockerfile.dev"
        let name = self.filename.rsplit('/').next().unwrap_or(&self.filename);
        name == "Dockerfile" || name.starts_with("Dockerfile.")
    }
}

/// `ArgoCD` application status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgoCdStatus {
    /// Health status (Healthy, Progressing, Degraded, etc.)
    pub health: String,
    /// Sync status (Synced, `OutOfSync`)
    pub sync: String,
    /// List of unhealthy resources
    pub unhealthy_resources: Vec<String>,
}

impl ArgoCdStatus {
    /// Check if the app is out of sync.
    #[must_use]
    pub fn is_out_of_sync(&self) -> bool {
        self.sync != "Synced"
    }

    /// Check if the app has health issues.
    #[must_use]
    pub fn has_health_issues(&self) -> bool {
        self.health != "Healthy"
    }
}

/// Kubernetes pod state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodState {
    /// Pod names
    pub names: Vec<String>,
    /// Recent events
    pub events: Vec<String>,
}

/// Commit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit SHA
    pub sha: String,
    /// Commit message
    pub message: String,
    /// Author
    pub author: String,
}

/// Historical context from `OpenMemory`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoricalContext {
    /// Similar failures found in memory
    pub similar_failures: Vec<MemoryEntry>,
    /// Agent success patterns
    pub agent_success_patterns: Vec<MemoryEntry>,
    /// Known solutions
    pub known_solutions: Vec<MemoryEntry>,
}

/// A memory entry from `OpenMemory`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Memory ID
    pub id: String,
    /// Memory content
    pub content: String,
    /// Relevance score
    pub score: f64,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// State of an ongoing remediation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationState {
    /// Unique task ID
    pub task_id: String,
    /// Original CI failure
    pub failure: CiFailure,
    /// Classified failure type
    pub failure_type: CiFailureType,
    /// All attempts made
    pub attempts: Vec<RemediationAttempt>,
    /// PR number (if associated)
    pub pr_number: Option<u32>,
    /// Original commit SHA
    pub original_sha: String,
    /// When remediation started
    pub started_at: DateTime<Utc>,
    /// Current status
    pub status: RemediationStatus,
}

impl RemediationState {
    /// Create a new remediation state.
    #[must_use]
    pub fn new(failure: CiFailure, failure_type: CiFailureType) -> Self {
        let task_id = format!("ci-{}", failure.workflow_run_id);
        let original_sha = failure.head_sha.clone();
        Self {
            task_id,
            failure,
            failure_type,
            attempts: Vec::new(),
            pr_number: None,
            original_sha,
            started_at: Utc::now(),
            status: RemediationStatus::Pending,
        }
    }

    /// Record a new attempt.
    #[allow(clippy::cast_possible_truncation)]
    pub fn record_attempt(&mut self, outcome: AttemptOutcome, coderun_name: &str, agent: Agent) {
        let attempt = RemediationAttempt {
            attempt_number: u32::try_from(self.attempts.len()).unwrap_or_default() + 1,
            agent,
            coderun_name: coderun_name.to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            outcome: Some(outcome),
            failure_reason: None,
            agent_output: None,
        };
        self.attempts.push(attempt);

        // Update status based on outcome
        match outcome {
            AttemptOutcome::Success => self.status = RemediationStatus::Succeeded,
            AttemptOutcome::Escalated => self.status = RemediationStatus::Escalated,
            _ => self.status = RemediationStatus::InProgress,
        }
    }

    /// Get the current agent.
    #[must_use]
    pub fn current_agent(&self) -> Option<Agent> {
        self.attempts.last().map(|a| a.agent)
    }

    /// Check if the same agent failed twice consecutively.
    #[must_use]
    pub fn same_agent_failed_twice(&self) -> bool {
        if self.attempts.len() < 2 {
            return false;
        }
        let last_two: Vec<_> = self.attempts.iter().rev().take(2).collect();
        last_two[0].agent == last_two[1].agent
            && !matches!(last_two[0].outcome, Some(AttemptOutcome::Success))
            && !matches!(last_two[1].outcome, Some(AttemptOutcome::Success))
    }

    /// Get PR URL if available.
    #[must_use]
    pub fn pr_url(&self) -> Option<String> {
        self.pr_number
            .map(|n| format!("https://github.com/{}/pull/{}", self.failure.repository, n))
    }

    /// Summarize all attempts for notifications.
    #[must_use]
    pub fn summarize_attempts(&self) -> String {
        self.attempts
            .iter()
            .map(|a| {
                format!(
                    "{}. {}: {}",
                    a.attempt_number,
                    a.agent.name(),
                    a.failure_reason.as_deref().unwrap_or("no details")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Status of a remediation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemediationStatus {
    /// Waiting to start
    Pending,
    /// Currently being worked on
    InProgress,
    /// Successfully fixed
    Succeeded,
    /// Escalated to human
    Escalated,
    /// Cancelled
    Cancelled,
}

/// Individual remediation attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAttempt {
    /// Attempt number (1-indexed)
    pub attempt_number: u32,
    /// Agent that made this attempt
    pub agent: Agent,
    /// Name of the `CodeRun` resource
    pub coderun_name: String,
    /// When the attempt started
    pub started_at: DateTime<Utc>,
    /// When the attempt completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Outcome of the attempt
    pub outcome: Option<AttemptOutcome>,
    /// Why it failed (if applicable)
    pub failure_reason: Option<String>,
    /// Agent's output/logs
    pub agent_output: Option<String>,
}

impl RemediationAttempt {
    /// Get the duration of this attempt.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.completed_at
            .map(|end| (end - self.started_at).to_std().unwrap_or_default())
    }
}

/// Outcome of a remediation attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttemptOutcome {
    /// CI passed after fix
    Success,
    /// Agent crashed or errored
    AgentFailed,
    /// Agent pushed but CI still fails
    CiStillFailing,
    /// Agent exceeded time limit
    Timeout,
    /// Escalated to human
    Escalated,
}

/// Configuration for CI remediation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationConfig {
    /// CLI to use (Factory, Claude, etc.)
    pub cli: String,
    /// Model to use
    pub model: String,
    /// Maximum remediation attempts before escalation
    pub max_attempts: u32,
    /// Time window (minutes) for deduplication
    pub time_window_mins: u32,
    /// `OpenMemory` URL
    pub memory_url: Option<String>,
    /// Whether to enable memory queries
    pub memory_enabled: bool,
}

impl Default for RemediationConfig {
    fn default() -> Self {
        Self {
            cli: "Factory".into(),
            model: "claude-opus-4-5-20250929".into(),
            max_attempts: 3,
            time_window_mins: 10,
            memory_url: Some("http://openmemory.cto-system.svc:3000".into()),
            memory_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_names() {
        assert_eq!(Agent::Rex.name(), "rex");
        assert_eq!(Agent::Rex.github_app(), "5DLabs-Rex");
        assert_eq!(Agent::Rex.template_name(), "rust-fix");
    }

    #[test]
    fn test_failure_type_categories() {
        assert!(CiFailureType::RustClippy.is_rust());
        assert!(CiFailureType::FrontendTypeScript.is_frontend());
        assert!(CiFailureType::DockerBuild.is_infra());
        assert!(CiFailureType::SecurityDependabot.is_security());
        assert!(CiFailureType::GitMergeConflict.is_merge_conflict());
    }

    #[test]
    fn test_changed_file_detection() {
        let rust_file = ChangedFile {
            filename: "src/main.rs".to_string(),
            status: "modified".to_string(),
            additions: 10,
            deletions: 5,
        };
        assert!(rust_file.is_rust());
        assert!(!rust_file.is_frontend());

        let ts_file = ChangedFile {
            filename: "src/App.tsx".to_string(),
            status: "modified".to_string(),
            additions: 20,
            deletions: 10,
        };
        assert!(ts_file.is_frontend());
        assert!(!ts_file.is_rust());

        let infra_file = ChangedFile {
            filename: "infra/gitops/apps.yaml".to_string(),
            status: "added".to_string(),
            additions: 50,
            deletions: 0,
        };
        assert!(infra_file.is_infra());
    }

    #[test]
    fn test_remediation_state_attempts() {
        let failure = CiFailure {
            workflow_run_id: 12345,
            workflow_name: "Test CI".into(),
            job_name: Some("clippy".into()),
            conclusion: "failure".into(),
            branch: "main".into(),
            head_sha: "abc123".into(),
            commit_message: "test".into(),
            html_url: "https://github.com/5dlabs/cto/actions/runs/12345".into(),
            repository: "5dlabs/cto".into(),
            sender: "user".into(),
            detected_at: Utc::now(),
            raw_event: None,
        };

        let mut state = RemediationState::new(failure, CiFailureType::RustClippy);
        assert!(state.attempts.is_empty());

        state.record_attempt(
            AttemptOutcome::CiStillFailing,
            "healer-ci-rex-123",
            Agent::Rex,
        );
        assert_eq!(state.attempts.len(), 1);
        assert_eq!(state.current_agent(), Some(Agent::Rex));
        assert!(!state.same_agent_failed_twice());

        state.record_attempt(
            AttemptOutcome::CiStillFailing,
            "healer-ci-rex-456",
            Agent::Rex,
        );
        assert!(state.same_agent_failed_twice());
    }
}
