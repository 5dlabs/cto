//! `ReviewRun` Custom Resource Definition for PR review tasks
//!
//! Stitch - The PR Review Bot
//! Triggered by GitHub events (PR open, check failures, comments) to perform
//! automated code review, bug detection, and security analysis.

use crate::cli::types::CLIType;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Review mode determines the focus and depth of the review
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReviewMode {
    /// Quick bug detection - critical issues only
    Hunt,
    /// Deep technical analysis - architecture, patterns, design
    Analyze,
    /// Security-focused review - auth, input validation, secrets
    Security,
    /// Performance optimization - bottlenecks, memory, scalability
    Performance,
    /// Comprehensive review (default) - all categories, balanced
    #[default]
    Review,
}

impl std::fmt::Display for ReviewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewMode::Hunt => write!(f, "hunt"),
            ReviewMode::Analyze => write!(f, "analyze"),
            ReviewMode::Security => write!(f, "security"),
            ReviewMode::Performance => write!(f, "performance"),
            ReviewMode::Review => write!(f, "review"),
        }
    }
}

/// Trigger source for the review
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTrigger {
    /// Triggered by PR open/sync/reopen
    #[default]
    PullRequest,
    /// Triggered by a comment command (e.g., `stitch review`)
    Comment,
    /// Triggered by CI check failure
    CheckRun,
    /// Triggered by code scanning alert
    CodeScan,
    /// Triggered by secret scanning alert
    SecretScan,
    /// Manual trigger via CRD creation
    Manual,
}

impl std::fmt::Display for ReviewTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewTrigger::PullRequest => write!(f, "pull_request"),
            ReviewTrigger::Comment => write!(f, "comment"),
            ReviewTrigger::CheckRun => write!(f, "check_run"),
            ReviewTrigger::CodeScan => write!(f, "code_scan"),
            ReviewTrigger::SecretScan => write!(f, "secret_scan"),
            ReviewTrigger::Manual => write!(f, "manual"),
        }
    }
}

// Re-use SecretEnvVar from coderun module to avoid duplication
pub use super::coderun::SecretEnvVar;

/// CLI-specific configuration
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ReviewCLIConfig {
    /// CLI type to use (claude, codex, factory, etc.)
    #[serde(rename = "cliType")]
    pub cli_type: CLIType,

    /// Model identifier (e.g., "claude-opus-4-5-20251101")
    pub model: String,

    /// CLI-specific settings (key-value pairs)
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,

    /// Maximum output tokens
    #[serde(default, rename = "maxTokens")]
    pub max_tokens: Option<u32>,

    /// Temperature setting
    #[serde(default)]
    pub temperature: Option<f32>,
}

/// Check run context when triggered by CI failure
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CheckRunContext {
    /// Check run ID from GitHub
    #[serde(rename = "checkRunId")]
    pub check_run_id: u64,

    /// Check run name (e.g., "Lint", "Test", "Build")
    pub name: String,

    /// Check suite ID
    #[serde(rename = "checkSuiteId", default)]
    pub check_suite_id: Option<u64>,

    /// Conclusion (failure, success, etc.)
    #[serde(default)]
    pub conclusion: Option<String>,
}

/// Code scanning alert context
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CodeScanContext {
    /// Alert number
    #[serde(rename = "alertNumber")]
    pub alert_number: u64,

    /// Alert rule ID
    #[serde(rename = "ruleId", default)]
    pub rule_id: Option<String>,

    /// Severity (critical, high, medium, low)
    #[serde(default)]
    pub severity: Option<String>,

    /// File path where alert was found
    #[serde(rename = "filePath", default)]
    pub file_path: Option<String>,
}

/// Comment context when triggered by user command
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CommentContext {
    /// Comment ID from GitHub
    #[serde(rename = "commentId")]
    pub comment_id: u64,

    /// Comment author
    pub author: String,

    /// Comment body (the command)
    pub body: String,

    /// Whether this is a review comment (inline) vs issue comment
    #[serde(rename = "isReviewComment", default)]
    pub is_review_comment: bool,

    /// File path if inline review comment
    #[serde(rename = "filePath", default)]
    pub file_path: Option<String>,

    /// Line number if inline review comment
    #[serde(rename = "lineNumber", default)]
    pub line_number: Option<u32>,
}

fn default_github_app() -> String {
    "5DLabs-Stitch".to_string()
}

/// `ReviewRun` CRD for PR review tasks
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "agents.platform", version = "v1", kind = "ReviewRun")]
#[kube(namespaced)]
#[kube(status = "ReviewRunStatus")]
#[kube(printcolumn = r#"{"name":"PR","type":"integer","jsonPath":".spec.prNumber"}"#)]
#[kube(printcolumn = r#"{"name":"Mode","type":"string","jsonPath":".spec.reviewMode"}"#)]
#[kube(printcolumn = r#"{"name":"Trigger","type":"string","jsonPath":".spec.trigger"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct ReviewRunSpec {
    /// Pull request number (required)
    #[serde(rename = "prNumber")]
    pub pr_number: u32,

    /// Repository URL (e.g., "https://github.com/5dlabs/cto")
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,

    /// Head SHA of the PR (commit to review)
    #[serde(rename = "headSha")]
    pub head_sha: String,

    /// Base branch (e.g., "main")
    #[serde(rename = "baseBranch", default)]
    pub base_branch: Option<String>,

    /// Head branch (e.g., "feature/my-feature")
    #[serde(rename = "headBranch", default)]
    pub head_branch: Option<String>,

    /// Review mode (hunt, analyze, security, performance, review)
    #[serde(rename = "reviewMode", default)]
    pub review_mode: ReviewMode,

    /// What triggered this review
    #[serde(default)]
    pub trigger: ReviewTrigger,

    /// GitHub App name for authentication (defaults to "5DLabs-Stitch")
    #[serde(rename = "githubApp", default = "default_github_app")]
    pub github_app: String,

    /// Model identifier to use
    pub model: String,

    /// Check run context (when triggered by CI failure)
    #[serde(rename = "checkRunContext", default)]
    pub check_run_context: Option<CheckRunContext>,

    /// Code scanning context (when triggered by security alert)
    #[serde(rename = "codeScanContext", default)]
    pub code_scan_context: Option<CodeScanContext>,

    /// Comment context (when triggered by user command)
    #[serde(rename = "commentContext", default)]
    pub comment_context: Option<CommentContext>,

    /// PR author username
    #[serde(rename = "prAuthor", default)]
    pub pr_author: Option<String>,

    /// PR title
    #[serde(rename = "prTitle", default)]
    pub pr_title: Option<String>,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default, rename = "envFromSecrets")]
    pub env_from_secrets: Vec<SecretEnvVar>,

    /// CLI configuration for CLI-agnostic operation
    #[serde(default, rename = "cliConfig")]
    pub cli_config: Option<ReviewCLIConfig>,

    /// Kubernetes `ServiceAccount` name for the Job pods
    #[serde(default, rename = "serviceAccountName")]
    pub service_account_name: Option<String>,
}

/// Status of the `ReviewRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
pub struct ReviewRunStatus {
    /// Current phase of the review
    pub phase: String,

    /// Human-readable message about the current state
    pub message: Option<String>,

    /// Timestamp when this phase was reached
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<String>,

    /// Associated Kubernetes Job name
    #[serde(rename = "jobName")]
    pub job_name: Option<String>,

    /// Review comment URL if posted
    #[serde(rename = "reviewCommentUrl")]
    pub review_comment_url: Option<String>,

    /// Number of issues found
    #[serde(rename = "issuesFound")]
    pub issues_found: Option<u32>,

    /// Number of suggestions made
    #[serde(rename = "suggestionsCount")]
    pub suggestions_count: Option<u32>,

    /// CI alerts analyzed (from utils alerts tool)
    #[serde(rename = "ciAlertsAnalyzed")]
    pub ci_alerts_analyzed: Option<u32>,

    /// Conditions for the `ReviewRun`
    pub conditions: Option<Vec<ReviewRunCondition>>,

    /// Name of the `ConfigMap` containing the prompt and context
    #[serde(rename = "configmapName")]
    pub configmap_name: Option<String>,

    /// Timestamp when the run finished
    #[serde(rename = "finishedAt")]
    pub finished_at: Option<String>,

    /// Time when controller should attempt TTL cleanup
    #[serde(rename = "expireAt")]
    pub expire_at: Option<String>,
}

/// Condition for the `ReviewRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, or Unknown)
    pub status: String,

    /// Last time the condition transitioned (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_mode_default() {
        let mode: ReviewMode = ReviewMode::default();
        assert_eq!(mode, ReviewMode::Review);
    }

    #[test]
    fn test_review_mode_display() {
        assert_eq!(ReviewMode::Hunt.to_string(), "hunt");
        assert_eq!(ReviewMode::Analyze.to_string(), "analyze");
        assert_eq!(ReviewMode::Security.to_string(), "security");
        assert_eq!(ReviewMode::Performance.to_string(), "performance");
        assert_eq!(ReviewMode::Review.to_string(), "review");
    }

    #[test]
    fn test_review_trigger_display() {
        assert_eq!(ReviewTrigger::PullRequest.to_string(), "pull_request");
        assert_eq!(ReviewTrigger::Comment.to_string(), "comment");
        assert_eq!(ReviewTrigger::CheckRun.to_string(), "check_run");
        assert_eq!(ReviewTrigger::CodeScan.to_string(), "code_scan");
    }

    #[test]
    fn test_review_spec_serialization() {
        let spec = ReviewRunSpec {
            pr_number: 123,
            repository_url: "https://github.com/5dlabs/cto".to_string(),
            head_sha: "abc123".to_string(),
            base_branch: Some("main".to_string()),
            head_branch: Some("feature/test".to_string()),
            review_mode: ReviewMode::Hunt,
            trigger: ReviewTrigger::CheckRun,
            github_app: "5DLabs-Stitch".to_string(),
            model: "claude-opus-4-5-20251101".to_string(),
            check_run_context: Some(CheckRunContext {
                check_run_id: 456,
                name: "Lint".to_string(),
                check_suite_id: Some(789),
                conclusion: Some("failure".to_string()),
            }),
            code_scan_context: None,
            comment_context: None,
            pr_author: Some("testuser".to_string()),
            pr_title: Some("Test PR".to_string()),
            env: HashMap::new(),
            env_from_secrets: vec![],
            cli_config: None,
            service_account_name: None,
        };

        let json = serde_json::to_string(&spec).expect("Failed to serialize");
        assert!(json.contains("\"prNumber\":123"));
        assert!(json.contains("\"reviewMode\":\"hunt\""));
        assert!(json.contains("\"trigger\":\"check_run\""));
    }
}

