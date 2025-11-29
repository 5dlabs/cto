//! `CodeRun` Custom Resource Definition for code implementation tasks

use crate::cli::types::CLIType;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reference to a secret for environment variable
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SecretEnvVar {
    /// Name of the environment variable
    pub name: String,
    /// Name of the secret
    #[serde(rename = "secretName")]
    pub secret_name: String,
    /// Key within the secret
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

/// Default function for `run_type` field
fn default_run_type() -> String {
    "implementation".to_string()
}

/// Default function for `context_version` field
fn default_context_version() -> u32 {
    1
}

/// Default function for `docs_branch` field
fn default_docs_branch() -> String {
    "main".to_string()
}

/// Default function for `continue_session` field
fn default_continue_session() -> bool {
    false
}

/// Default function for `overwrite_memory` field
fn default_overwrite_memory() -> bool {
    false
}

fn default_enable_docker() -> bool {
    true
}

/// CLI-specific configuration
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CLIConfig {
    /// CLI type to use (claude, codex, opencode, cursor, etc.)
    #[serde(rename = "cliType")]
    pub cli_type: CLIType,

    /// Model identifier (CLI-specific, e.g., "sonnet", "gpt-4", "claude-3-5-sonnet-20241022")
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

    /// Model rotation array for retry attempts (JSON array as string or Vec<String>)
    #[serde(
        default,
        rename = "modelRotation",
        skip_serializing_if = "Option::is_none"
    )]
    pub model_rotation: Option<serde_json::Value>,
}

/// `CodeRun` CRD for code implementation tasks
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "agents.platform", version = "v1", kind = "CodeRun")]
#[kube(namespaced)]
#[kube(status = "CodeRunStatus")]
#[kube(printcolumn = r#"{"name":"Type","type":"string","jsonPath":".spec.runType"}"#)]
#[kube(printcolumn = r#"{"name":"Task","type":"integer","jsonPath":".spec.taskId"}"#)]
#[kube(printcolumn = r#"{"name":"Service","type":"string","jsonPath":".spec.service"}"#)]
#[kube(printcolumn = r#"{"name":"Model","type":"string","jsonPath":".spec.model"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct CodeRunSpec {
    /// Type of run: "implementation" (default), "documentation", "intake"
    #[serde(default = "default_run_type", rename = "runType")]
    pub run_type: String,

    /// Task ID to implement (required for implementation, optional for docs/intake)
    #[serde(rename = "taskId", default)]
    pub task_id: Option<u32>,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,

    /// Documentation repository URL (where Task Master definitions come from)
    #[serde(rename = "docsRepositoryUrl")]
    pub docs_repository_url: String,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    #[serde(default, rename = "docsProjectDirectory")]
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    #[serde(default, rename = "workingDirectory")]
    pub working_directory: Option<String>,

    /// Model identifier to use with the selected CLI (e.g., gpt-5-codex, claude-sonnet-4-20250514)
    pub model: String,

    /// GitHub username for authentication and commits (deprecated - use githubApp)
    #[serde(rename = "githubUser", default)]
    pub github_user: Option<String>,

    /// GitHub App name for authentication (e.g., "5DLabs-Rex")
    #[serde(rename = "githubApp", default)]
    pub github_app: Option<String>,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version", rename = "contextVersion")]
    pub context_version: u32,

    /// Docs branch to use (e.g., "main", "feature/branch")
    #[serde(default = "default_docs_branch", rename = "docsBranch")]
    pub docs_branch: String,

    /// Whether to continue a previous session (auto-continue on retries or user-requested)
    #[serde(default = "default_continue_session", rename = "continueSession")]
    pub continue_session: bool,

    /// Whether to overwrite memory before starting
    #[serde(default = "default_overwrite_memory", rename = "overwriteMemory")]
    pub overwrite_memory: bool,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default, rename = "envFromSecrets")]
    pub env_from_secrets: Vec<SecretEnvVar>,

    /// Whether to enable Docker-in-Docker support for this `CodeRun` (defaults to true)
    #[serde(default = "default_enable_docker", rename = "enableDocker")]
    pub enable_docker: bool,

    /// Base64-encoded YAML containing task requirements (secrets and environment variables)
    #[serde(default, rename = "taskRequirements")]
    pub task_requirements: Option<String>,

    /// Kubernetes `ServiceAccount` name for the Job pods created to execute this `CodeRun`
    #[serde(default, rename = "serviceAccountName")]
    pub service_account_name: Option<String>,

    /// CLI configuration for CLI-agnostic operation (optional)
    #[serde(default, rename = "cliConfig")]
    pub cli_config: Option<CLIConfig>,
}

/// Status of the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CodeRunStatus {
    /// Current phase of the code implementation
    pub phase: String,

    /// Human-readable message about the current state
    pub message: Option<String>,

    /// Timestamp when this phase was reached
    pub last_update: Option<String>,

    /// Associated Kubernetes Job name
    pub job_name: Option<String>,

    /// Pull request URL if created
    pub pull_request_url: Option<String>,

    /// Latest remediation status label applied to the PR (e.g., needs-fixes, needs-tess, approved)
    #[serde(rename = "remediationStatus", skip_serializing_if = "Option::is_none")]
    pub remediation_status: Option<String>,

    /// QA decision captured from Tess (approved, `changes_requested`, pending)
    #[serde(rename = "qaStatus", skip_serializing_if = "Option::is_none")]
    pub qa_status: Option<String>,

    /// Current retry attempt (if applicable)
    pub retry_count: Option<u32>,

    /// Conditions for the `CodeRun`
    pub conditions: Option<Vec<CodeRunCondition>>,

    /// Name of the `ConfigMap` containing the prompt and context
    pub configmap_name: Option<String>,

    /// Version of the context and prompt used
    pub context_version: Option<u32>,

    /// Modification to the prompt if any
    pub prompt_modification: Option<String>,

    /// Mode of prompt (e.g., "direct", "indirect")
    pub prompt_mode: Option<String>,

    /// Session ID for tracking
    pub session_id: Option<String>,

    /// Timestamp when the run finished (Succeeded/Failed)
    #[serde(rename = "finishedAt", skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,

    /// Time when controller should attempt TTL cleanup
    #[serde(rename = "expireAt", skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<String>,

    /// Timestamp when cleanup completed
    #[serde(rename = "cleanupCompletedAt", skip_serializing_if = "Option::is_none")]
    pub cleanup_completed_at: Option<String>,

    /// Tracks whether the code implementation work has been completed successfully
    /// This field is used for idempotent reconciliation and TTL safety
    pub work_completed: Option<bool>,
}

/// Condition for the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeRunCondition {
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
    fn test_cli_config_creation() {
        let cli_config = CLIConfig {
            cli_type: CLIType::Codex,
            model: "gpt-4".to_string(),
            settings: {
                let mut settings = HashMap::new();
                settings.insert(
                    "approval_policy".to_string(),
                    serde_json::json!("on-failure"),
                );
                settings
            },
            max_tokens: Some(4096),
            temperature: Some(0.7),
            model_rotation: None,
        };

        assert_eq!(cli_config.cli_type, CLIType::Codex);
        assert_eq!(cli_config.model, "gpt-4");
        assert_eq!(cli_config.max_tokens, Some(4096));
        assert_eq!(cli_config.temperature, Some(0.7));
    }
}
