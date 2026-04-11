//! `CodeRun` Custom Resource Definition for code implementation tasks

use crate::cli::types::{CLIType, Provider};
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
    "develop".to_string()
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

/// Helper for serde defaults returning `true`.
fn default_true() -> bool {
    true
}

/// Provider entry inside an [`ACPEntry`].
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ACPProvider {
    /// Provider name (e.g. "Anthropic", "OpenAI")
    pub name: String,
    /// Available credits budget for this provider
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credits: Option<u64>,
}

/// Model entry inside an [`ACPEntry`].
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ACPModel {
    /// Model identifier (e.g. "claude-opus-4-20250514")
    pub name: String,
    /// Thinking level hint: "high", "medium", or "low"
    #[serde(default, rename = "thinkingLevel", skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<String>,
    /// Performance score 0-100
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
}

/// AI-CLI-Provider entry — one candidate runtime environment.
///
/// The OpenClaw harness agent picks from the ACP array based on
/// task difficulty, credits, and model scores.
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ACPEntry {
    /// CLI name (e.g. "Claude Code", "Codex")
    pub cli: String,
    /// Provider metadata
    pub provider: ACPProvider,
    /// Available models for this CLI/provider combination
    pub models: Vec<ACPModel>,
    /// Optional API base URL override
    #[serde(default, rename = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Environment variable name for the API key (e.g. "ANTHROPIC_API_KEY").
    /// The controller ensures this env var is set in the pod — this is NOT a raw secret.
    #[serde(default, rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Provider entry inside [`OpenClawConfig`].
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct OpenClawProvider {
    /// Provider name (e.g. "Fireworks")
    pub name: String,
}

/// Model entry inside [`OpenClawConfig`].
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct OpenClawModel {
    /// Model identifier
    pub name: String,
    /// Thinking level hint: "high", "medium", or "low"
    #[serde(default, rename = "thinkingLevel", skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<String>,
}

/// OpenClaw runtime configuration — maps to OpenClaw gateway provider settings.
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct OpenClawConfig {
    /// Provider metadata
    pub provider: OpenClawProvider,
    /// Available models
    pub models: Vec<OpenClawModel>,
    /// Optional API base URL
    #[serde(default, rename = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Environment variable name for the API key
    #[serde(default, rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Linear integration configuration for status sync
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct LinearIntegration {
    /// Whether Linear status sync is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Linear agent session ID for activity updates
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,

    /// OAuth access token for Linear agent API calls (from webhook)
    #[serde(rename = "accessToken", default)]
    pub access_token: Option<String>,

    /// Linear issue ID for status updates
    #[serde(rename = "issueId", default)]
    pub issue_id: Option<String>,

    /// Linear team ID for workflow state mapping
    #[serde(rename = "teamId", default)]
    pub team_id: Option<String>,
}

/// Subtask specification for breaking down work into smaller units
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SubtaskSpec {
    /// Unique identifier for the subtask within the parent task
    pub id: u32,

    /// Human-readable title of the subtask
    pub title: String,

    /// Optional detailed description of the subtask
    #[serde(default)]
    pub description: Option<String>,

    /// Optional subagent type to handle this subtask (e.g., "rex", "bolt", "tess")
    #[serde(default, rename = "subagentType")]
    pub subagent_type: Option<String>,

    /// Optional execution level for ordering (lower levels execute first)
    #[serde(default, rename = "executionLevel")]
    pub execution_level: Option<u32>,

    /// Whether this subtask can run in parallel with others at the same execution level
    #[serde(default)]
    pub parallelizable: bool,

    /// List of subtask IDs that must complete before this subtask can start
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Default watcher check interval in seconds (2 minutes).
fn default_watcher_check_interval() -> u64 {
    120
}

/// Default circuit breaker threshold.
fn default_watcher_circuit_breaker() -> u32 {
    3
}

/// Watcher configuration for dual-model execution pattern.
///
/// When enabled, a second "watcher" CodeRun is spawned alongside the executor
/// that monitors progress, detects issues, and writes them to a coordination
/// file for the executor to self-correct.
///
/// CLI-agnostic: supports any CLI (claude, codex, factory, droid, gemini, opencode, cursor).
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct WatcherConfig {
    /// Enable watcher mode for this CodeRun.
    /// When true, a paired watcher CodeRun is created alongside the executor.
    #[serde(default)]
    pub enabled: bool,

    /// CLI to use for the watcher (e.g., "factory", "droid", "claude").
    /// Any supported CLI works.
    #[serde(default)]
    pub cli: Option<String>,

    /// Model to use for the watcher.
    /// Typically a cheaper model since watcher does monitoring, not code generation.
    #[serde(default)]
    pub model: Option<String>,

    /// Interval between watcher checks in seconds.
    /// Default: 120 (2 minutes).
    #[serde(
        default = "default_watcher_check_interval",
        rename = "checkIntervalSecs"
    )]
    pub check_interval_secs: u64,

    /// Prompt template for the watcher.
    /// Default: "watcher/base".
    #[serde(default)]
    pub template: Option<String>,

    /// Circuit breaker threshold - after this many failures on the same step,
    /// escalate to human intervention.
    /// Default: 3.
    #[serde(
        default = "default_watcher_circuit_breaker",
        rename = "circuitBreakerThreshold"
    )]
    pub circuit_breaker_threshold: u32,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cli: None,
            model: None,
            check_interval_secs: default_watcher_check_interval(),
            template: None,
            circuit_breaker_threshold: default_watcher_circuit_breaker(),
        }
    }
}

/// How the MCP tools server should handle `tools_request_capability` escalation
/// calls for agents running under this CodeRun.
///
/// Mirrors `tools::escalation::EscalationMode` — duplicated here because the
/// controller crate does not depend on the tools crate and CRD types need
/// `schemars::JsonSchema`.
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EscalationMode {
    /// Grant any tool present in the catalog. `deny` globs still apply.
    Auto,
    /// Grant only tools matching at least one `allow` glob. `deny` globs still apply.
    #[default]
    Allowlist,
    /// Deny every escalation and log for human review.
    Review,
}

/// Policy governing mid-session tool escalation requests.
///
/// Serialized as JSON and forwarded to the tools HTTP server via the
/// `X-Escalation-Policy` header so each agent session gets its own policy.
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct EscalationPolicy {
    /// Escalation mode. Defaults to `allowlist`.
    #[serde(default)]
    pub mode: EscalationMode,
    /// Glob patterns allowing tools (only consulted when `mode == allowlist`).
    #[serde(default)]
    pub allow: Vec<String>,
    /// Glob patterns blocking tools regardless of mode. Takes precedence over `allow`.
    #[serde(default)]
    pub deny: Vec<String>,
}

/// CLI-specific configuration
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CLIConfig {
    /// CLI type to use (claude, codex, opencode, cursor, etc.)
    #[serde(rename = "cliType")]
    pub cli_type: CLIType,

    /// Model identifier (CLI-specific, e.g., "sonnet", "gpt-4", "claude-sonnet-4-5-20250929")
    pub model: String,

    /// Inference provider (fireworks, anthropic, google, openai, cursor, factory, moonshot).
    /// When omitted, inferred from model ID or controller-level `cliProviders` config.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,

    /// Custom base URL for the provider API.
    /// Overrides the provider's default base URL when set.
    #[serde(
        default,
        rename = "providerBaseUrl",
        skip_serializing_if = "Option::is_none"
    )]
    pub provider_base_url: Option<String>,

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
#[allow(clippy::struct_excessive_bools)]
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

    /// Optional base URL of a skills-release repo. When set, the controller
    /// downloads per-skill tarballs from the repo's GitHub Releases into its
    /// local cache and resolves skill content from there. When unset, the
    /// controller falls back to the baked-in /app/templates/skills directory.
    ///
    /// Format: "https://github.com/{owner}/{repo}"
    #[serde(default, rename = "skillsUrl", skip_serializing_if = "Option::is_none")]
    pub skills_url: Option<String>,

    /// Optional project name for skills/persona overlays. When set, the controller
    /// downloads `{agent}-{project}.tar.gz` instead of `{agent}-default.tar.gz`,
    /// which contains the merged `_default` + project-specific overrides.
    ///
    /// Example: "test-sandbox" → downloads `rex-test-sandbox.tar.gz`
    #[serde(
        default,
        rename = "skillsProject",
        skip_serializing_if = "Option::is_none"
    )]
    pub skills_project: Option<String>,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    #[serde(default, rename = "docsProjectDirectory")]
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    #[serde(default, rename = "workingDirectory")]
    pub working_directory: Option<String>,

    /// Model identifier to use with the selected CLI (e.g., gpt-5-codex, claude-sonnet-4-20250514)
    pub model: String,

    /// Prompt style variant (e.g., "minimal" for Ralph-style prompts)
    #[serde(default, rename = "promptStyle")]
    pub prompt_style: Option<String>,

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

    /// Linear integration configuration for status sync sidecar
    #[serde(default, rename = "linearIntegration")]
    pub linear_integration: Option<LinearIntegration>,

    /// Direct prompt modification content (used by healer CI runs)
    /// When set, this content is written to prompt.md in the task ConfigMap
    #[serde(default, rename = "promptModification")]
    pub prompt_modification: Option<String>,

    /// Direct acceptance criteria content (used by healer CI runs)
    /// When set, this content is written to acceptance-criteria.md in the task ConfigMap
    /// The acceptance criteria probe will verify these checkboxes after task completion
    #[serde(default, rename = "acceptanceCriteria")]
    pub acceptance_criteria: Option<String>,

    /// Comma-separated list of remote MCP tools to make available
    /// These are resolved by the controller and written to client-config.json
    /// Example: "mcp_tools_github_*,mcp_tools_kubernetes_*"
    #[serde(default, rename = "remoteTools")]
    pub remote_tools: Option<String>,

    /// Comma-separated list of local MCP server tools to spawn
    /// Example: "postgres,filesystem"
    #[serde(default, rename = "localTools")]
    pub local_tools: Option<String>,

    /// Whether to delete existing PVC and start with a fresh workspace
    /// Defaults to true for intake runs, false otherwise
    #[serde(default, rename = "freshWorkspace")]
    pub fresh_workspace: Option<bool>,

    /// Optional list of subtasks that break down this CodeRun into smaller units of work
    #[serde(default)]
    pub subtasks: Option<Vec<SubtaskSpec>>,

    /// Watcher configuration for dual-model execution pattern.
    /// When enabled, a paired watcher CodeRun monitors this executor and provides
    /// real-time feedback via a coordination file.
    #[serde(default, rename = "watcherConfig")]
    pub watcher_config: Option<WatcherConfig>,

    /// If this CodeRun is a watcher, the name of the executor CodeRun it monitors.
    /// This field is set automatically by the controller when creating watcher CodeRuns.
    #[serde(default, rename = "watcherFor")]
    pub watcher_for: Option<String>,

    /// Escalation policy for mid-session tool requests.
    /// When set, serialized as JSON and forwarded to the tools HTTP server via
    /// the `X-Escalation-Policy` header. When absent the server's default
    /// policy applies (typically `allowlist` with no allow patterns → deny all).
    #[serde(default, rename = "escalationPolicy")]
    pub escalation_policy: Option<EscalationPolicy>,

    // ── New fields: multi-agent CodeRun overhaul ─────────────────────

    /// Explicit implementation agent name (e.g. "rex", "blaze").
    /// Takes precedence over `github_app` derivation for naming and labels.
    #[serde(default, rename = "implementationAgent")]
    pub implementation_agent: Option<String>,

    /// Run quality review phase (Cleo). Defaults to true.
    #[serde(default = "default_true")]
    pub quality: bool,

    /// Run security scan phase (Cipher). Defaults to true.
    #[serde(default = "default_true")]
    pub security: bool,

    /// Run testing phase (Tess). Defaults to true.
    #[serde(default = "default_true")]
    pub testing: bool,

    /// Run deployment phase (Bolt). Defaults to false (opt-in).
    #[serde(default)]
    pub deployment: bool,

    /// AI-CLI-Provider candidates. The OpenClaw harness agent picks from this
    /// array based on task difficulty, credits, and model scores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acp: Option<Vec<ACPEntry>>,

    /// OpenClaw runtime configuration — maps to OpenClaw gateway provider settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openclaw: Option<OpenClawConfig>,
}

impl Default for CodeRunSpec {
    fn default() -> Self {
        Self {
            run_type: "implementation".to_string(),
            task_id: None,
            service: String::new(),
            repository_url: String::new(),
            docs_repository_url: String::new(),
            skills_url: None,
            skills_project: None,
            docs_project_directory: None,
            working_directory: None,
            model: String::new(),
            prompt_style: None,
            github_user: None,
            github_app: None,
            context_version: 1,
            docs_branch: "develop".to_string(),
            continue_session: false,
            overwrite_memory: false,
            env: std::collections::HashMap::new(),
            env_from_secrets: vec![],
            enable_docker: true,
            task_requirements: None,
            service_account_name: None,
            cli_config: None,
            linear_integration: None,
            prompt_modification: None,
            acceptance_criteria: None,
            remote_tools: None,
            local_tools: None,
            fresh_workspace: None,
            subtasks: None,
            watcher_config: None,
            watcher_for: None,
            escalation_policy: None,
            implementation_agent: None,
            quality: true,
            security: true,
            testing: true,
            deployment: false,
            acp: None,
            openclaw: None,
        }
    }
}

/// Status of the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
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

    /// Name of the associated watcher CodeRun (if watcher mode is enabled)
    #[serde(rename = "watcherCodeRun", skip_serializing_if = "Option::is_none")]
    pub watcher_coderun: Option<String>,

    /// Name of the coordination ConfigMap shared between executor and watcher
    #[serde(
        rename = "coordinationConfigMap",
        skip_serializing_if = "Option::is_none"
    )]
    pub coordination_configmap: Option<String>,
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
            provider: None,
            provider_base_url: None,
        };

        assert_eq!(cli_config.cli_type, CLIType::Codex);
        assert_eq!(cli_config.model, "gpt-4");
        assert_eq!(cli_config.max_tokens, Some(4096));
        assert_eq!(cli_config.temperature, Some(0.7));
    }

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert!(!config.enabled);
        assert!(config.cli.is_none());
        assert!(config.model.is_none());
        assert_eq!(config.check_interval_secs, 120);
        assert!(config.template.is_none());
        assert_eq!(config.circuit_breaker_threshold, 3);
    }

    #[test]
    fn test_watcher_config_from_json() {
        let json = r#"{
            "enabled": true,
            "cli": "factory",
            "model": "glm-4-plus",
            "checkIntervalSecs": 60,
            "template": "watcher/custom",
            "circuitBreakerThreshold": 5
        }"#;
        let config: WatcherConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.cli, Some("factory".to_string()));
        assert_eq!(config.model, Some("glm-4-plus".to_string()));
        assert_eq!(config.check_interval_secs, 60);
        assert_eq!(config.template, Some("watcher/custom".to_string()));
        assert_eq!(config.circuit_breaker_threshold, 5);
    }

    #[test]
    fn test_coderun_spec_with_watcher() {
        let json = r#"{
            "service": "test-service",
            "repositoryUrl": "https://github.com/test/repo",
            "docsRepositoryUrl": "https://github.com/test/docs",
            "model": "claude-opus",
            "watcherConfig": {
                "enabled": true,
                "cli": "droid",
                "model": "glm-4-plus"
            }
        }"#;
        let spec: CodeRunSpec = serde_json::from_str(json).unwrap();
        assert!(spec.watcher_config.is_some());
        let watcher = spec.watcher_config.unwrap();
        assert!(watcher.enabled);
        assert_eq!(watcher.cli, Some("droid".to_string()));
    }

    #[test]
    fn test_coderun_spec_with_skills_url() {
        let json = r#"{
            "service": "test-service",
            "repositoryUrl": "https://github.com/test/repo",
            "docsRepositoryUrl": "https://github.com/test/docs",
            "model": "claude-opus",
            "skillsUrl": "https://github.com/5dlabs/cto-agent-personas",
            "skillsProject": "test-sandbox"
        }"#;
        let spec: CodeRunSpec = serde_json::from_str(json).unwrap();
        assert_eq!(
            spec.skills_url,
            Some("https://github.com/5dlabs/cto-agent-personas".to_string())
        );
        assert_eq!(spec.skills_project, Some("test-sandbox".to_string()));

        // Round-trip: omitted on the wire when None
        let default_json = r#"{
            "service": "s",
            "repositoryUrl": "r",
            "docsRepositoryUrl": "d",
            "model": "m"
        }"#;
        let default_spec: CodeRunSpec = serde_json::from_str(default_json).unwrap();
        assert!(default_spec.skills_url.is_none());
        assert!(default_spec.skills_project.is_none());
        let serialized = serde_json::to_string(&default_spec).unwrap();
        assert!(
            !serialized.contains("skillsUrl"),
            "skillsUrl should be omitted when None, got: {serialized}"
        );
        assert!(
            !serialized.contains("skillsProject"),
            "skillsProject should be omitted when None, got: {serialized}"
        );
    }

    #[test]
    fn test_coderun_spec_watcher_for() {
        let json = r#"{
            "service": "test-service",
            "repositoryUrl": "https://github.com/test/repo",
            "docsRepositoryUrl": "https://github.com/test/docs",
            "model": "glm-4-plus",
            "runType": "watcher",
            "watcherFor": "my-executor-coderun"
        }"#;
        let spec: CodeRunSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.watcher_for, Some("my-executor-coderun".to_string()));
        assert_eq!(spec.run_type, "watcher");
    }

    #[test]
    fn test_acp_entry_serde_roundtrip() {
        let entry = ACPEntry {
            cli: "Claude Code".to_string(),
            provider: ACPProvider {
                name: "Anthropic".to_string(),
                credits: Some(250_000),
            },
            models: vec![
                ACPModel {
                    name: "claude-opus-4-20250514".to_string(),
                    thinking_level: Some("high".to_string()),
                    score: Some(96),
                },
                ACPModel {
                    name: "claude-sonnet-4-20250514".to_string(),
                    thinking_level: Some("medium".to_string()),
                    score: Some(91),
                },
            ],
            base_url: Some("https://api.anthropic.com".to_string()),
            api_key: Some("ANTHROPIC_API_KEY".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: ACPEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cli, "Claude Code");
        assert_eq!(deserialized.provider.name, "Anthropic");
        assert_eq!(deserialized.provider.credits, Some(250_000));
        assert_eq!(deserialized.models.len(), 2);
        assert_eq!(deserialized.models[0].score, Some(96));
    }

    #[test]
    fn test_openclaw_config_serde_roundtrip() {
        let config = OpenClawConfig {
            provider: OpenClawProvider {
                name: "Fireworks".to_string(),
            },
            models: vec![OpenClawModel {
                name: "kimi-k2p5-turbo".to_string(),
                thinking_level: Some("high".to_string()),
            }],
            base_url: Some("https://api.fireworks.ai/inference".to_string()),
            api_key: Some("FIREWORKS_API_KEY".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OpenClawConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.provider.name, "Fireworks");
        assert_eq!(deserialized.models.len(), 1);
        assert_eq!(deserialized.base_url.unwrap(), "https://api.fireworks.ai/inference");
    }

    #[test]
    fn test_coderun_spec_backward_compat() {
        // Minimal JSON without any new fields — must deserialize with defaults
        let json = r#"{
            "runType": "implementation",
            "service": "cto",
            "repositoryUrl": "https://github.com/5dlabs/cto.git",
            "docsRepositoryUrl": "https://github.com/5dlabs/cto.git",
            "model": "sonnet",
            "contextVersion": 1
        }"#;
        let spec: CodeRunSpec = serde_json::from_str(json).unwrap();
        assert!(spec.quality);
        assert!(spec.security);
        assert!(spec.testing);
        assert!(!spec.deployment);
        assert!(spec.implementation_agent.is_none());
        assert!(spec.acp.is_none());
        assert!(spec.openclaw.is_none());
    }

    #[test]
    fn test_coderun_spec_with_new_fields() {
        let json = r#"{
            "runType": "implementation",
            "service": "cto",
            "repositoryUrl": "https://github.com/5dlabs/cto.git",
            "docsRepositoryUrl": "https://github.com/5dlabs/cto.git",
            "model": "sonnet",
            "contextVersion": 1,
            "implementationAgent": "rex",
            "quality": false,
            "security": true,
            "testing": false,
            "deployment": true,
            "acp": [{
                "cli": "Claude Code",
                "provider": { "name": "Anthropic", "credits": 250000 },
                "models": [{ "name": "opus", "thinkingLevel": "high", "score": 96 }],
                "baseUrl": "https://api.anthropic.com",
                "apiKey": "ANTHROPIC_API_KEY"
            }],
            "openclaw": {
                "provider": { "name": "Fireworks" },
                "models": [{ "name": "kimi-k2p5-turbo", "thinkingLevel": "high" }],
                "baseUrl": "https://api.fireworks.ai/inference",
                "apiKey": "FIREWORKS_API_KEY"
            }
        }"#;
        let spec: CodeRunSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.implementation_agent, Some("rex".to_string()));
        assert!(!spec.quality);
        assert!(spec.security);
        assert!(!spec.testing);
        assert!(spec.deployment);
        assert!(spec.acp.is_some());
        let acp = spec.acp.unwrap();
        assert_eq!(acp.len(), 1);
        assert_eq!(acp[0].cli, "Claude Code");
        assert_eq!(acp[0].provider.credits, Some(250_000));
        assert!(spec.openclaw.is_some());
        assert_eq!(spec.openclaw.unwrap().provider.name, "Fireworks");
    }
}
