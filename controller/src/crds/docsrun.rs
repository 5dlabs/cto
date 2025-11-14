//! `DocsRun` Custom Resource Definition for documentation generation

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "agents.platform", version = "v1", kind = "DocsRun")]
#[kube(namespaced)]
#[kube(status = "DocsRunStatus")]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct DocsRunSpec {
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,
    #[serde(rename = "workingDirectory")]
    pub working_directory: String,
    #[serde(rename = "sourceBranch")]
    pub source_branch: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(rename = "githubUser", default)]
    pub github_user: Option<String>,
    #[serde(rename = "githubApp", default)]
    pub github_app: Option<String>,
    /// Optional: CLI type to use (e.g., "Claude", "Cursor", "Codex", "OpenCode")
    #[serde(default)]
    pub cli: Option<String>,
    #[serde(rename = "includeCodebase", default)]
    pub include_codebase: Option<bool>,
    /// Optional: additional remote MCP tools to merge into the baseline
    #[serde(rename = "remoteTools", default)]
    pub remote_tools: Option<Vec<String>>,
    /// Optional: local MCP servers to enable if available in agent profile
    #[serde(rename = "localTools", default)]
    pub local_tools: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocsRunStatus {
    pub phase: String,
    pub message: Option<String>,
    pub last_update: Option<String>,
    pub job_name: Option<String>,
    pub pull_request_url: Option<String>,
    pub conditions: Option<Vec<DocsRunCondition>>,
    pub configmap_name: Option<String>,
    #[serde(rename = "finishedAt", skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(rename = "expireAt", skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<String>,
    #[serde(rename = "cleanupCompletedAt", skip_serializing_if = "Option::is_none")]
    pub cleanup_completed_at: Option<String>,
    /// Tracks whether the documentation work has been completed successfully
    /// This field is used for idempotent reconciliation and TTL safety
    pub work_completed: Option<bool>,
}

/// Condition for the `DocsRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocsRunCondition {
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

/// Phase of `DocsRun` execution
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum DocsRunPhase {
    /// `DocsRun` has been created but not yet processed
    Pending,
    /// Documentation generation is in progress
    Running,
    /// Documentation generation completed successfully
    Succeeded,
    /// Documentation generation failed
    Failed,
    /// `DocsRun` was manually cancelled
    Cancelled,
}
