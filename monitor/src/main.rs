//! Play Monitor CLI
//!
//! A comprehensive CLI tool for monitoring play workflows and all platform resources.
//! Uses kubectl --watch for real-time streaming of workflows, CRDs, pods, and sensors.
//! Emits unified JSON events for Cursor agent E2E feedback loop automation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Play workflow monitoring CLI for E2E feedback loop
#[derive(Parser)]
#[command(name = "play-monitor")]
#[command(about = "Monitor play workflows and retrieve failure logs")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format (json or text)
    #[arg(long, default_value = "json", global = true)]
    format: OutputFormat,

    /// Argo namespace for workflows
    #[arg(long, default_value = "cto", global = true)]
    namespace: String,

    /// Agent platform namespace for CRDs and pods
    #[arg(long, default_value = "cto", global = true)]
    agent_namespace: String,

    /// Namespace for Argo Events sensors
    #[arg(long, default_value = "automation", global = true)]
    sensor_namespace: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output file for JSONL events (in addition to stdout)
    #[arg(long, global = true)]
    output_file: Option<PathBuf>,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Json,
    Text,
}

#[derive(Subcommand)]
enum Commands {
    /// [PRIMARY] Full E2E loop: start play, monitor all resources until completion
    Full {
        /// Task ID for the play
        #[arg(long)]
        task_id: String,

        /// Path to cto-config.json (reads play configuration)
        #[arg(long, default_value = "cto-config.json")]
        config: String,

        /// Poll interval in seconds (for GitHub state polling)
        #[arg(long, default_value = "30")]
        interval: u64,

        /// Max consecutive failures before stopping (0 = unlimited)
        #[arg(long, default_value = "5")]
        max_failures: u32,

        /// Workflow template name
        #[arg(long, default_value = "play-workflow-template")]
        template: String,

        /// GitHub repository for PR state polling (e.g., 5dlabs/cto-parallel-test)
        #[arg(long)]
        repository: Option<String>,

        /// Enable self-healing mode with automatic remediation on failure
        /// Requires remediation section in cto-config.json
        #[arg(long)]
        self_healing: bool,
    },
    /// Monitor an existing play workflow using kubectl --watch streams
    Watch {
        /// Task ID to filter resources by (matches task-id label)
        #[arg(long)]
        task_id: String,

        /// GitHub repository for PR state polling (e.g., 5dlabs/cto-parallel-test)
        #[arg(long)]
        repository: Option<String>,

        /// Poll interval in seconds (for GitHub state polling)
        #[arg(long, default_value = "30")]
        github_interval: u64,

        /// Fetch logs automatically on failure
        #[arg(long, default_value = "true")]
        fetch_logs: bool,

        /// Max consecutive failures before stopping (0 = unlimited)
        #[arg(long, default_value = "5")]
        max_failures: u32,

        /// Tail lines for logs on failure
        #[arg(long, default_value = "500")]
        log_tail: u32,
    },
    /// [LEGACY] Monitor using polling instead of watch streams
    Loop {
        /// Play workflow name (e.g., play-42, play-task-123)
        #[arg(long)]
        play_id: String,

        /// Poll interval in seconds (for status checks between events)
        #[arg(long, default_value = "10")]
        interval: u64,

        /// Fetch logs automatically on failure
        #[arg(long, default_value = "true")]
        fetch_logs: bool,

        /// Max consecutive failures before stopping (0 = unlimited)
        #[arg(long, default_value = "5")]
        max_failures: u32,

        /// Tail lines for logs on failure
        #[arg(long, default_value = "500")]
        log_tail: u32,
    },
    /// Get current status of a workflow (single check)
    Status {
        /// Workflow name to check
        #[arg(long)]
        play_id: String,
    },
    /// Get logs for a specific workflow step or pod
    Logs {
        /// Workflow name
        #[arg(long)]
        play_id: String,

        /// Specific step/pod name (optional - gets failed step if not specified)
        #[arg(long)]
        step: Option<String>,

        /// Number of log lines to retrieve
        #[arg(long, default_value = "500")]
        tail: u32,

        /// Filter for error patterns only
        #[arg(long)]
        errors_only: bool,
    },
    /// Reset environment: clean cluster resources and reset test repo
    Reset {
        /// Test repository name (e.g., cto-parallel-test)
        #[arg(long, default_value = "cto-parallel-test")]
        repo: String,

        /// GitHub organization
        #[arg(long, default_value = "5dlabs")]
        org: String,

        /// Skip Kubernetes cleanup
        #[arg(long)]
        skip_k8s: bool,

        /// Skip GitHub repo reset
        #[arg(long)]
        skip_github: bool,

        /// Force without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Run/submit a play workflow via Argo CLI
    Run {
        /// Task ID for the play
        #[arg(long)]
        task_id: String,

        /// Repository to work on
        #[arg(long, default_value = "5dlabs/cto-parallel-test")]
        repository: String,

        /// Service name (for PVC naming and labels)
        #[arg(long, default_value = "cto-parallel-test")]
        service: String,

        /// Run type for workflow naming (e.g., monitor-test, diagnostic)
        #[arg(long, default_value = "monitor-test")]
        run_type: String,

        /// Implementation agent (Rex or Blaze)
        #[arg(long, default_value = "5DLabs-Rex")]
        agent: String,

        /// Agent CLI type (claude, codex, cursor, opencode, etc.)
        #[arg(long, default_value = "codex")]
        cli: String,

        /// Model to use
        #[arg(long, default_value = "gpt-5-codex")]
        model: String,

        /// Workflow template name
        #[arg(long, default_value = "play-workflow-template")]
        template: String,
    },
    /// Query and analyze `OpenMemory` for agent insights
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// List recent memories for a task or agent
    List {
        /// Filter by task ID
        #[arg(long)]
        task_id: Option<String>,

        /// Filter by agent name (rex, blaze, cleo, tess, atlas, etc.)
        #[arg(long)]
        agent: Option<String>,

        /// Maximum number of memories to return
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Semantic query for memories
    Query {
        /// Search query text
        #[arg(long)]
        text: String,

        /// Filter by agent name
        #[arg(long)]
        agent: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: u32,

        /// Include waypoint connections
        #[arg(long)]
        include_waypoints: bool,
    },
    /// Show memory statistics and health
    Stats {
        /// Filter stats by agent name
        #[arg(long)]
        agent: Option<String>,
    },
    /// Get a specific memory by ID
    Get {
        /// Memory ID
        #[arg(long)]
        id: String,
    },
}

// =============================================================================
// CTO Config Types - parsed from cto-config.json
// =============================================================================

/// CTO configuration file structure (cto-config.json)
#[derive(Debug, Deserialize)]
struct CtoConfig {
    defaults: CtoDefaults,
}

/// Default configurations
#[derive(Debug, Deserialize)]
struct CtoDefaults {
    play: PlayConfig,
    /// Remediation configuration for self-healing loop
    #[serde(default)]
    remediation: Option<RemediationConfig>,
}

/// Play workflow configuration
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Fields parsed from config, not all used yet
struct PlayConfig {
    /// Model to use (e.g., "claude-opus-4-5-20251101")
    #[serde(default)]
    model: Option<String>,
    /// CLI tool (e.g., "factory", "cursor", "codex")
    #[serde(default)]
    cli: Option<String>,
    /// Implementation agent (e.g., "5DLabs-Rex")
    implementation_agent: String,
    /// Frontend agent (e.g., "5DLabs-Blaze")
    #[serde(default)]
    frontend_agent: Option<String>,
    /// Quality agent (e.g., "5DLabs-Cleo")
    quality_agent: String,
    /// Security agent (e.g., "5DLabs-Cipher")
    #[serde(default)]
    security_agent: Option<String>,
    /// Testing agent (e.g., "5DLabs-Tess")
    testing_agent: String,
    /// Repository (e.g., "5dlabs/cto-parallel-test")
    repository: String,
    /// Service name
    #[serde(default)]
    service: Option<String>,
    /// Docs repository
    #[serde(default)]
    docs_repository: Option<String>,
    /// Working directory
    #[serde(default)]
    working_directory: Option<String>,
    /// Max retries for implementation
    #[serde(default)]
    implementation_max_retries: Option<u32>,
    /// Max retries for quality
    #[serde(default)]
    quality_max_retries: Option<u32>,
    /// Max retries for testing
    #[serde(default)]
    testing_max_retries: Option<u32>,
    /// Auto merge
    #[serde(default)]
    auto_merge: Option<bool>,
    /// Parallel execution
    #[serde(default)]
    parallel_execution: Option<bool>,
}

/// Remediation configuration for self-healing loop
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemediationConfig {
    /// Repository to fix (platform repo, e.g., "5dlabs/cto")
    repository: String,
    /// Docs repository for context
    #[serde(default)]
    docs_repository: Option<String>,
    /// Docs project directory
    #[serde(default)]
    docs_project_directory: Option<String>,
    /// GitHub App for remediation agent
    agent: String,
    /// CLI tool (e.g., "claude", "codex")
    cli: String,
    /// Model to use (e.g., "claude-opus-4-5-20251101")
    model: String,
    /// Maximum remediation iterations before giving up
    #[serde(default = "default_max_iterations")]
    max_iterations: u32,
    /// Template for remediation `CodeRun`
    #[serde(default = "default_remediation_template")]
    template: String,
    /// Timeout for `ArgoCD` sync in seconds
    #[serde(default = "default_sync_timeout")]
    sync_timeout_secs: u64,
}

fn default_max_iterations() -> u32 {
    3
}

fn default_remediation_template() -> String {
    "rex-remediation".to_string()
}

fn default_sync_timeout() -> u64 {
    300
}

/// Failure context captured when a failure is detected
#[derive(Debug, Clone, Serialize)]
struct FailureContext {
    /// Name of the failed workflow
    workflow_name: String,
    /// Name of the failed resource (pod, coderun, etc.)
    failed_resource: String,
    /// Type of the failed resource
    resource_type: String,
    /// Phase/status when failure occurred
    phase: String,
    /// Error message if available
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    /// Logs from the failed resource
    #[serde(skip_serializing_if = "Option::is_none")]
    logs: Option<String>,
    /// Container that failed (for pods)
    #[serde(skip_serializing_if = "Option::is_none")]
    container: Option<String>,
    /// Exit code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    /// Timestamp of the failure
    timestamp: DateTime<Utc>,
}

// =============================================================================
// Remediation Functions - Self-healing loop support
// =============================================================================

/// Trigger remediation by creating a `CodeRun` for the remediation agent
///
/// Returns the name of the created `CodeRun`
fn trigger_remediation(
    config: &RemediationConfig,
    failure: &FailureContext,
    task_id: &str,
    iteration: u32,
    namespace: &str,
) -> Result<String> {
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let coderun_name = format!("remediation-t{task_id}-i{iteration}-{uid}");

    // Serialize failure context to JSON for the agent
    let failure_json = serde_json::to_string(failure)
        .context("Failed to serialize failure context")?;

    // Create CodeRun YAML manifest
    let coderun_yaml = format!(
        r#"apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: {coderun_name}
  namespace: {namespace}
  labels:
    task-id: "{task_id}"
    remediation: "true"
    iteration: "{iteration}"
    agents.platform/type: remediation
spec:
  taskId: {task_id}
  githubApp: "{agent}"
  cli: "{cli}"
  model: "{model}"
  repository: "{repository}"
  docsRepository: "{docs_repo}"
  docsProjectDirectory: "{docs_dir}"
  template: "{template}"
  env:
    - name: REMEDIATION_MODE
      value: "true"
    - name: FAILURE_CONTEXT
      value: {failure_json_escaped}
    - name: ORIGINAL_WORKFLOW
      value: "{workflow_name}"
    - name: FAILURE_TYPE
      value: "{failure_type}"
    - name: ITERATION
      value: "{iteration}"
    - name: MAX_ITERATIONS
      value: "{max_iterations}"
"#,
        coderun_name = coderun_name,
        namespace = namespace,
        task_id = task_id,
        iteration = iteration,
        agent = config.agent,
        cli = config.cli,
        model = config.model,
        repository = config.repository,
        docs_repo = config.docs_repository.as_deref().unwrap_or(&config.repository),
        docs_dir = config.docs_project_directory.as_deref().unwrap_or("docs"),
        template = config.template,
        failure_json_escaped = serde_json::to_string(&failure_json)?,
        workflow_name = failure.workflow_name,
        failure_type = failure.resource_type,
        max_iterations = config.max_iterations,
    );

    // Apply via kubectl
    let mut child = Command::new("kubectl")
        .args(["apply", "-f", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn kubectl apply")?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(coderun_yaml.as_bytes())
            .context("Failed to write YAML to kubectl stdin")?;
    }

    let output = child.wait_with_output().context("Failed to wait for kubectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to create remediation CodeRun: {stderr}"
        ));
    }

    println!(
        "{}",
        format!("Created remediation CodeRun: {coderun_name}").green()
    );

    Ok(coderun_name)
}

/// Wait for a `CodeRun` to complete (Succeeded or Failed)
async fn wait_for_coderun(
    coderun_name: &str,
    namespace: &str,
    timeout_secs: u64,
) -> Result<(bool, Option<String>)> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            return Ok((false, Some("Timeout waiting for CodeRun".to_string())));
        }

        let output = Command::new("kubectl")
            .args([
                "get",
                "coderun",
                coderun_name,
                "-n",
                namespace,
                "-o",
                "jsonpath={.status.phase}",
            ])
            .output()
            .context("Failed to get CodeRun status")?;

        let phase = String::from_utf8_lossy(&output.stdout).trim().to_string();

        match phase.as_str() {
            "Succeeded" => return Ok((true, None)),
            "Failed" => {
                // Get failure message
                let msg_output = Command::new("kubectl")
                    .args([
                        "get",
                        "coderun",
                        coderun_name,
                        "-n",
                        namespace,
                        "-o",
                        "jsonpath={.status.message}",
                    ])
                    .output()
                    .ok();
                let message = msg_output
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .filter(|s| !s.is_empty());
                return Ok((false, message));
            }
            _ => {
                // Still running, wait and retry
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    }
}

/// Wait for `ArgoCD` to sync after a PR is merged
///
/// This polls the `ArgoCD` application status to detect when changes are deployed
async fn wait_for_argocd_sync(
    app_name: &str,
    expected_commit: Option<&str>,
    timeout_secs: u64,
) -> Result<bool> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    println!(
        "{}",
        format!("Waiting for ArgoCD sync (app: {app_name}, timeout: {timeout_secs}s)...").cyan()
    );

    loop {
        if start.elapsed() > timeout {
            warn!("Timeout waiting for ArgoCD sync");
            return Ok(false);
        }

        // Get ArgoCD app status
        let output = Command::new("kubectl")
            .args([
                "get",
                "application",
                app_name,
                "-n",
                "argocd",
                "-o",
                "json",
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                if let Ok(app) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    let sync_status = app["status"]["sync"]["status"]
                        .as_str()
                        .unwrap_or("Unknown");
                    let health_status = app["status"]["health"]["status"]
                        .as_str()
                        .unwrap_or("Unknown");
                    let revision = app["status"]["sync"]["revision"]
                        .as_str()
                        .unwrap_or("unknown");

                    debug!(
                        "ArgoCD app status: sync={}, health={}, revision={}",
                        sync_status, health_status, revision
                    );

                    // Check if synced and healthy
                    if sync_status == "Synced" && health_status == "Healthy" {
                        // If we have an expected commit, verify it matches
                        if let Some(expected) = expected_commit {
                            if revision.starts_with(expected) || expected.starts_with(revision) {
                                println!(
                                    "{}",
                                    format!("ArgoCD synced to commit: {revision}").green()
                                );
                                return Ok(true);
                            }
                        } else {
                            // No specific commit expected, just need synced + healthy
                            println!(
                                "{}",
                                format!("ArgoCD synced and healthy (revision: {revision})").green()
                            );
                            return Ok(true);
                        }
                    }
                }
            }
        }

        // Wait before next poll
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}

/// Get the PR URL from a `CodeRun` after remediation completes
fn get_coderun_pr_url(coderun_name: &str, namespace: &str) -> Option<String> {
    let output = Command::new("kubectl")
        .args([
            "get",
            "coderun",
            coderun_name,
            "-n",
            namespace,
            "-o",
            "jsonpath={.status.outputs.pr-url}",
        ])
        .output()
        .ok()?;

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        None
    } else {
        Some(url)
    }
}

// =============================================================================
// Argo Workflow Types - parsed from `argo get -o json`
// =============================================================================

/// Argo workflow step/node information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowStep {
    /// Step/node ID
    id: String,
    /// Step display name
    name: String,
    /// Step type (Pod, Steps, DAG, etc.)
    #[serde(rename = "type")]
    step_type: String,
    /// Phase: Pending, Running, Succeeded, Failed, Error, Skipped
    phase: String,
    /// Pod name for this step (if type=Pod)
    #[serde(skip_serializing_if = "Option::is_none")]
    pod_name: Option<String>,
    /// Exit code if completed
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    /// Message (often error message)
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    /// Start time
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at: Option<String>,
    /// Finish time
    #[serde(skip_serializing_if = "Option::is_none")]
    finished_at: Option<String>,
}

/// Workflow status from `argo get -o json`
#[derive(Debug, Serialize, Deserialize)]
struct WorkflowStatus {
    /// Workflow name
    name: String,
    /// Workflow namespace
    namespace: String,
    /// Overall phase: Pending, Running, Succeeded, Failed, Error
    phase: String,
    /// Status message
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    /// Current stage (derived from running/failed steps)
    #[serde(skip_serializing_if = "Option::is_none")]
    stage: Option<String>,
    /// All workflow steps/nodes
    steps: Vec<WorkflowStep>,
    /// Failed steps (for quick access)
    failed_steps: Vec<WorkflowStep>,
    /// Start time
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at: Option<String>,
    /// Finish time
    #[serde(skip_serializing_if = "Option::is_none")]
    finished_at: Option<String>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<i64>,
    /// Timestamp of this status check
    timestamp: DateTime<Utc>,
    /// Error if status check failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// =============================================================================
// Loop Events - JSON events emitted by the monitor
// =============================================================================

/// Resource type for watch events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Workflow,
    CodeRun,
    DocsRun,
    Sensor,
    Pod,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workflow => write!(f, "workflow"),
            Self::CodeRun => write!(f, "coderun"),
            Self::DocsRun => write!(f, "docsrun"),
            Self::Sensor => write!(f, "sensor"),
            Self::Pod => write!(f, "pod"),
        }
    }
}

/// Resource change action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAction {
    Added,
    Modified,
    Deleted,
}

/// GitHub PR state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestState {
    pub number: u64,
    pub state: String,
    pub title: String,
    pub mergeable: Option<bool>,
    pub draft: bool,
    pub labels: Vec<String>,
    pub reviews: Vec<ReviewState>,
    pub checks: Vec<CheckState>,
}

/// GitHub review state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    pub author: String,
    pub state: String,
}

/// GitHub check state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckState {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

/// Events emitted by the monitor
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
enum LoopEvent {
    /// Monitor has started
    Started {
        task_id: String,
        watching: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    /// Current workflow status update (legacy compatibility)
    Status {
        play_id: String,
        workflow_phase: String,
        stage: Option<String>,
        steps: Vec<WorkflowStep>,
        timestamp: DateTime<Utc>,
    },
    /// A stage has completed successfully
    StageComplete {
        play_id: String,
        stage: String,
        next_stage: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Workflow or step failure detected
    Failure {
        play_id: String,
        stage: Option<String>,
        failed_step: Option<WorkflowStep>,
        logs: Option<String>,
        consecutive_failures: u32,
        timestamp: DateTime<Utc>,
    },
    /// Workflow completed successfully
    Completed {
        play_id: String,
        duration_seconds: i64,
        timestamp: DateTime<Utc>,
    },
    /// Monitor stopped (max failures reached or user interrupt)
    Stopped {
        play_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// Resource change from kubectl --watch
    Resource {
        task_id: String,
        resource_type: ResourceType,
        action: ResourceAction,
        name: String,
        namespace: String,
        phase: Option<String>,
        labels: Option<std::collections::HashMap<String, String>>,
        message: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// GitHub PR state update
    Github {
        task_id: String,
        repository: String,
        pull_request: Option<PullRequestState>,
        timestamp: DateTime<Utc>,
    },
    /// Sensor health/activity event
    SensorHealth {
        task_id: String,
        sensors: Vec<SensorState>,
        timestamp: DateTime<Utc>,
    },
}

/// Sensor state from watch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorState {
    pub name: String,
    pub status: String,
    pub last_triggered: Option<String>,
}

// =============================================================================
// Response Types for non-loop commands
// =============================================================================

/// Logs response
#[derive(Debug, Serialize, Deserialize)]
struct LogsResponse {
    play_id: String,
    step: Option<String>,
    namespace: String,
    logs: String,
    line_count: usize,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Reset response
#[derive(Debug, Serialize, Deserialize)]
struct ResetResponse {
    success: bool,
    k8s_cleanup: CleanupResult,
    github_reset: Option<GithubResetResult>,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// K8s cleanup result
#[derive(Debug, Serialize, Deserialize)]
struct CleanupResult {
    workflows_deleted: i32,
    pods_deleted: i32,
    configmaps_deleted: i32,
    pvcs_deleted: i32,
    skipped: bool,
}

/// GitHub reset result
#[derive(Debug, Serialize, Deserialize)]
struct GithubResetResult {
    repo: String,
    deleted: bool,
    created: bool,
    pushed: bool,
}

/// Run workflow response
#[derive(Debug, Serialize, Deserialize)]
struct RunResponse {
    success: bool,
    workflow_name: Option<String>,
    task_id: String,
    repository: String,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Configuration for running a play workflow
struct RunWorkflowConfig<'a> {
    task_id: &'a str,
    repository: &'a str,
    service: &'a str,
    run_type: &'a str,
    agent: &'a str,
    agent_cli: &'a str,
    model: &'a str,
    template: &'a str,
    namespace: &'a str,
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing if verbose
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("play_monitor=debug")
            .init();
    }

    // Create event emitter with optional file output
    let emitter = EventEmitter::new(cli.output_file.clone());

    match cli.command {
        Commands::Full {
            task_id,
            config,
            interval,
            max_failures,
            template,
            repository,
            self_healing,
        } => {
            if self_healing {
                // Run self-healing loop with automatic remediation
                run_self_healing_loop(
                    &task_id,
                    &config,
                    &cli.namespace,
                    &cli.agent_namespace,
                    interval,
                    &template,
                    &emitter,
                )
                .await?;
            } else {
                // Run regular full watch without remediation
                run_full_watch(
                    &task_id,
                    &config,
                    &cli.namespace,
                    &cli.agent_namespace,
                    interval,
                    max_failures,
                    &template,
                    repository.as_deref(),
                    &emitter,
                )
                .await?;
            }
        }
        Commands::Watch {
            task_id,
            repository,
            github_interval,
            fetch_logs,
            max_failures,
            log_tail,
        } => {
            run_multi_watch(
                &task_id,
                &cli.namespace,
                &cli.agent_namespace,
                repository.as_deref(),
                github_interval,
                fetch_logs,
                max_failures,
                log_tail,
                &emitter,
                None, // No remediation for Watch command
            )
            .await?;
        }
        Commands::Loop {
            play_id,
            interval,
            fetch_logs,
            max_failures,
            log_tail,
        } => {
            run_loop(
                &play_id,
                &cli.namespace,
                interval,
                fetch_logs,
                max_failures,
                log_tail,
            )
            .await?;
        }
        Commands::Status { play_id } => {
            let result = get_workflow_status(&play_id, &cli.namespace)?;
            output_result(&result, cli.format)?;
        }
        Commands::Logs {
            play_id,
            step,
            tail,
            errors_only,
        } => {
            let result = get_logs(&play_id, step.as_deref(), &cli.namespace, tail, errors_only)?;
            output_result(&result, cli.format)?;
        }
        Commands::Reset {
            repo,
            org,
            skip_k8s,
            skip_github,
            force,
        } => {
            let result =
                reset_environment(&cli.namespace, &org, &repo, skip_k8s, skip_github, force)?;
            output_result(&result, cli.format)?;
        }
        Commands::Run {
            task_id,
            repository,
            service,
            run_type,
            agent,
            cli: agent_cli,
            model,
            template,
        } => {
            let config = RunWorkflowConfig {
                task_id: &task_id,
                repository: &repository,
                service: &service,
                run_type: &run_type,
                agent: &agent,
                agent_cli: &agent_cli,
                model: &model,
                template: &template,
                namespace: &cli.namespace,
            };
            let result = run_workflow(&config)?;
            output_result(&result, cli.format)?;
        }
        Commands::Memory { action } => {
            handle_memory_command(action).await?;
        }
    }

    Ok(())
}

// =============================================================================
// Core Functions: Argo Workflow Status & Monitoring
// =============================================================================

/// Get workflow status using `argo get -o json`
fn get_workflow_status(workflow_name: &str, namespace: &str) -> Result<WorkflowStatus> {
    debug!(
        "Getting workflow status for {} in {}",
        workflow_name, namespace
    );

    let output = Command::new("argo")
        .args(["get", workflow_name, "-n", namespace, "-o", "json"])
        .output()
        .context("Failed to execute argo get")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(WorkflowStatus {
            name: workflow_name.to_string(),
            namespace: namespace.to_string(),
            phase: "Unknown".to_string(),
            message: None,
            stage: None,
            steps: vec![],
            failed_steps: vec![],
            started_at: None,
            finished_at: None,
            duration_seconds: None,
            timestamp: Utc::now(),
            error: Some(stderr.to_string()),
        });
    }

    parse_workflow_status(&output.stdout, workflow_name, namespace)
}

/// Parse argo workflow JSON output
fn parse_workflow_status(
    json_bytes: &[u8],
    workflow_name: &str,
    namespace: &str,
) -> Result<WorkflowStatus> {
    let workflow: serde_json::Value =
        serde_json::from_slice(json_bytes).context("Failed to parse argo JSON output")?;

    let status = &workflow["status"];
    let phase = status["phase"].as_str().unwrap_or("Unknown").to_string();
    let message = status["message"].as_str().map(ToString::to_string);
    let started_at = status["startedAt"].as_str().map(ToString::to_string);
    let finished_at = status["finishedAt"].as_str().map(ToString::to_string);

    // Calculate duration if both times present
    let duration_seconds = calculate_duration(started_at.as_deref(), finished_at.as_deref());

    // Parse nodes/steps
    let (steps, failed_steps) = parse_workflow_nodes(&status["nodes"]);

    // Determine current stage from running or most recent step
    let stage = determine_stage_from_steps(&steps);

    Ok(WorkflowStatus {
        name: workflow_name.to_string(),
        namespace: namespace.to_string(),
        phase,
        message,
        stage,
        steps,
        failed_steps,
        started_at,
        finished_at,
        duration_seconds,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Parse workflow nodes into steps
fn parse_workflow_nodes(nodes: &serde_json::Value) -> (Vec<WorkflowStep>, Vec<WorkflowStep>) {
    let mut steps = Vec::new();
    let mut failed_steps = Vec::new();

    if let Some(nodes_obj) = nodes.as_object() {
        for (id, node) in nodes_obj {
            let step = WorkflowStep {
                id: id.clone(),
                name: node["displayName"]
                    .as_str()
                    .or_else(|| node["name"].as_str())
                    .unwrap_or(id)
                    .to_string(),
                step_type: node["type"].as_str().unwrap_or("Unknown").to_string(),
                phase: node["phase"].as_str().unwrap_or("Unknown").to_string(),
                pod_name: node["id"].as_str().map(ToString::to_string),
                exit_code: node["outputs"]["exitCode"]
                    .as_str()
                    .and_then(|s| s.parse().ok()),
                message: node["message"].as_str().map(ToString::to_string),
                started_at: node["startedAt"].as_str().map(ToString::to_string),
                finished_at: node["finishedAt"].as_str().map(ToString::to_string),
            };

            // Only include Pod-type steps (actual work with retrievable logs)
            if step.step_type == "Pod" {
                // Track failed Pod steps (only Pods have logs we can retrieve)
                if step.phase == "Failed" || step.phase == "Error" {
                    failed_steps.push(step.clone());
                }
                steps.push(step);
            }
        }
    }

    // Sort steps by start time
    steps.sort_by(|a, b| a.started_at.cmp(&b.started_at));
    failed_steps.sort_by(|a, b| a.started_at.cmp(&b.started_at));

    (steps, failed_steps)
}

/// Determine current stage from workflow steps
fn determine_stage_from_steps(steps: &[WorkflowStep]) -> Option<String> {
    // Find the currently running step, or the most recently failed/completed
    for step in steps.iter().rev() {
        let name = step.name.to_lowercase();

        if step.phase == "Running" || step.phase == "Failed" || step.phase == "Error" {
            if name.contains("rex") || name.contains("blaze") || name.contains("implementation") {
                return Some("implementation".to_string());
            }
            if name.contains("cleo") || name.contains("quality") {
                return Some("code-quality".to_string());
            }
            if name.contains("cypher") || name.contains("security") {
                return Some("security".to_string());
            }
            if name.contains("tess") || name.contains("testing") || name.contains("qa") {
                return Some("qa".to_string());
            }
            if name.contains("atlas") || name.contains("integration") || name.contains("merge") {
                return Some("integration".to_string());
            }
            if name.contains("bolt") || name.contains("deploy") {
                return Some("deployment".to_string());
            }
        }
    }

    None
}

/// Calculate duration between two timestamps
fn calculate_duration(started: Option<&str>, finished: Option<&str>) -> Option<i64> {
    let start = started.and_then(|s| DateTime::parse_from_rfc3339(s).ok());
    let end = finished
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .or_else(|| Some(Utc::now().into()));

    match (start, end) {
        (Some(s), Some(e)) => Some(e.signed_duration_since(s).num_seconds()),
        _ => None,
    }
}

// =============================================================================
// Main Loop: Event-driven workflow monitoring
// =============================================================================

// Legacy run_full_loop removed - use run_full_watch instead

/// Run the monitoring loop - emits JSON events (legacy polling mode)
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn run_loop(
    play_id: &str,
    namespace: &str,
    interval: u64,
    fetch_logs: bool,
    max_failures: u32,
    log_tail: u32,
) -> Result<()> {
    // Emit started event
    emit_event(&LoopEvent::Started {
        task_id: play_id.to_string(),
        watching: vec![format!("workflow/{play_id} (polling mode)")],
        timestamp: Utc::now(),
    })?;

    let mut consecutive_failures: u32 = 0;
    let mut last_stage: Option<String> = None;
    let mut last_phase = String::new();
    let mut last_failed_count: usize = 0;

    loop {
        // Get current workflow status
        let status = match get_workflow_status(play_id, namespace) {
            Ok(s) => s,
            Err(e) => {
                consecutive_failures += 1;
                emit_event(&LoopEvent::Failure {
                    play_id: play_id.to_string(),
                    stage: last_stage.clone(),
                    failed_step: None,
                    logs: Some(format!("Error getting workflow status: {e}")),
                    consecutive_failures,
                    timestamp: Utc::now(),
                })?;

                if max_failures > 0 && consecutive_failures >= max_failures {
                    emit_event(&LoopEvent::Stopped {
                        play_id: play_id.to_string(),
                        reason: format!("Max consecutive failures reached ({max_failures})"),
                        timestamp: Utc::now(),
                    })?;
                    return Ok(());
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                continue;
            }
        };

        // Check for error in status
        if let Some(ref err) = status.error {
            consecutive_failures += 1;
            emit_event(&LoopEvent::Failure {
                play_id: play_id.to_string(),
                stage: last_stage.clone(),
                failed_step: None,
                logs: Some(format!("Workflow error: {err}")),
                consecutive_failures,
                timestamp: Utc::now(),
            })?;

            if max_failures > 0 && consecutive_failures >= max_failures {
                emit_event(&LoopEvent::Stopped {
                    play_id: play_id.to_string(),
                    reason: format!("Max consecutive failures reached ({max_failures})"),
                    timestamp: Utc::now(),
                })?;
                return Ok(());
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            continue;
        }

        // Emit status event (only when phase changes to reduce noise)
        if status.phase != last_phase {
            emit_event(&LoopEvent::Status {
                play_id: play_id.to_string(),
                workflow_phase: status.phase.clone(),
                stage: status.stage.clone(),
                steps: status.steps.clone(),
                timestamp: Utc::now(),
            })?;
            last_phase.clone_from(&status.phase);
        }

        // Check for stage change
        if status.stage.is_some() && status.stage != last_stage {
            if let Some(ref prev_stage) = last_stage {
                emit_event(&LoopEvent::StageComplete {
                    play_id: play_id.to_string(),
                    stage: prev_stage.clone(),
                    next_stage: status.stage.clone(),
                    timestamp: Utc::now(),
                })?;
            }
            last_stage.clone_from(&status.stage);
        }

        // Handle workflow completion
        if status.phase == "Succeeded" {
            emit_event(&LoopEvent::Completed {
                play_id: play_id.to_string(),
                duration_seconds: status.duration_seconds.unwrap_or(0),
                timestamp: Utc::now(),
            })?;
            return Ok(());
        }

        // Handle workflow failure - only check phase, not historical failed steps
        // Workflows may have failed steps but still be running (retries, self-healing)
        if status.phase == "Failed" || status.phase == "Error" {
            consecutive_failures += 1;

            // Get the first failed step for context
            let failed_step = status.failed_steps.first().cloned();

            // Fetch logs if enabled and we have a failed step
            let logs = if fetch_logs {
                if let Some(ref step) = failed_step {
                    if let Some(ref pod_name) = step.pod_name {
                        get_step_logs(pod_name, namespace, log_tail).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            emit_event(&LoopEvent::Failure {
                play_id: play_id.to_string(),
                stage: status.stage.clone(),
                failed_step,
                logs,
                consecutive_failures,
                timestamp: Utc::now(),
            })?;

            // Workflow failure is terminal - exit immediately after emitting the failure event
            emit_event(&LoopEvent::Stopped {
                play_id: play_id.to_string(),
                reason: "Workflow entered terminal failure state".to_string(),
                timestamp: Utc::now(),
            })?;
            return Ok(());
        }

        // Track new failures (e.g., crash-looping pods) even when workflow is Running
        // This prevents infinite loops when pods repeatedly fail but workflow stays active
        let current_failed_count = status.failed_steps.len();
        if current_failed_count > last_failed_count {
            // New failures detected - increment counter
            consecutive_failures += 1;

            // Get the most recent failed step for context
            let failed_step = status.failed_steps.last().cloned();

            // Fetch logs if enabled
            let logs = if fetch_logs {
                if let Some(ref step) = failed_step {
                    if let Some(ref pod_name) = step.pod_name {
                        get_step_logs(pod_name, namespace, log_tail).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            emit_event(&LoopEvent::Failure {
                play_id: play_id.to_string(),
                stage: status.stage.clone(),
                failed_step,
                logs,
                consecutive_failures,
                timestamp: Utc::now(),
            })?;

            if max_failures > 0 && consecutive_failures >= max_failures {
                emit_event(&LoopEvent::Stopped {
                    play_id: play_id.to_string(),
                    reason: format!("Max consecutive failures reached ({max_failures})"),
                    timestamp: Utc::now(),
                })?;
                return Ok(());
            }
        } else {
            // Workflow is stable or improving - reset counter
            // This handles:
            // - current_failed_count == 0 (fully healthy)
            // - current_failed_count == last_failed_count (stable, no new failures)
            // - current_failed_count < last_failed_count (improving, some resolved)
            // Critically, this resets after transient kubectl errors once the next
            // successful poll shows the workflow isn't getting worse.
            consecutive_failures = 0;
        }
        last_failed_count = current_failed_count;

        // Wait before next check
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

/// Emit a JSON event to stdout
fn emit_event(event: &LoopEvent) -> Result<()> {
    let json = serde_json::to_string(event)?;
    println!("{json}");
    std::io::stdout().flush()?;
    Ok(())
}

// =============================================================================
// Event Emitter - handles stdout and optional file output
// =============================================================================

/// Event emitter that writes to stdout and optionally to a file
#[derive(Clone)]
struct EventEmitter {
    output_file: Option<PathBuf>,
}

impl EventEmitter {
    fn new(output_file: Option<PathBuf>) -> Self {
        Self { output_file }
    }

    fn emit(&self, event: &LoopEvent) -> Result<()> {
        let json = serde_json::to_string(event)?;

        // Always write to stdout
        println!("{json}");
        std::io::stdout().flush()?;

        // Optionally write to file (JSONL format)
        if let Some(ref path) = self.output_file {
            use std::fs::OpenOptions;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .with_context(|| format!("Failed to open output file: {}", path.display()))?;
            writeln!(file, "{json}")?;
        }

        Ok(())
    }
}

// =============================================================================
// Multi-Watch Infrastructure
// =============================================================================

/// Message from a watch stream
#[derive(Debug)]
#[allow(dead_code)] // Error variant reserved for future use
enum WatchMessage {
    Resource {
        resource_type: ResourceType,
        action: ResourceAction,
        name: String,
        namespace: String,
        phase: Option<String>,
        labels: Option<std::collections::HashMap<String, String>>,
        message: Option<String>,
    },
    GitHub {
        task_id: String,
        repository: String,
        pull_request: Option<PullRequestState>,
    },
    Error {
        resource_type: ResourceType,
        error: String,
    },
    Closed {
        resource_type: ResourceType,
    },
}

/// Spawn a kubectl watch process and send events to the channel
fn spawn_watch(
    resource_type: ResourceType,
    namespace: &str,
    label_selector: Option<&str>,
    tx: &mpsc::Sender<WatchMessage>,
) -> Result<tokio::process::Child> {
    let resource_name = match resource_type {
        ResourceType::Workflow => "workflows.argoproj.io",
        ResourceType::CodeRun => "coderuns.agents.platform",
        ResourceType::DocsRun => "docsruns.agents.platform",
        ResourceType::Sensor => "sensors.argoproj.io",
        ResourceType::Pod => "pods",
    };

    let mut args = vec![
        "get".to_string(),
        resource_name.to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "--watch".to_string(),
        "--request-timeout=0".to_string(), // Disable timeout to keep watch open indefinitely
        "-o".to_string(),
        "json".to_string(),
    ];

    if let Some(selector) = label_selector {
        args.push("-l".to_string());
        args.push(selector.to_string());
    }

    debug!("Spawning watch: kubectl {}", args.join(" "));

    let mut child = AsyncCommand::new("kubectl")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn kubectl watch for {resource_type}"))?;

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture stdout from kubectl")?;

    let rt = resource_type;
    let tx_clone = tx.clone();

    // Spawn a task to read stdout and parse JSON
    // kubectl --watch -o json outputs pretty-printed multi-line JSON objects
    // We need to accumulate lines until we have a complete JSON object
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut json_buffer = String::new();
        let mut brace_depth: i32 = 0;
        let mut in_object = false;

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();

            // Track brace depth to find complete JSON objects
            for ch in trimmed.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        in_object = true;
                    }
                    '}' => {
                        brace_depth -= 1;
                    }
                    _ => {}
                }
            }

            if in_object {
                json_buffer.push_str(&line);
                json_buffer.push('\n');
            }

            // When brace depth returns to 0, we have a complete object
            if in_object && brace_depth == 0 {
                match parse_watch_line(&json_buffer, rt) {
                    Ok(msg) => {
                        if tx_clone.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse watch JSON for {}: {}", rt, e);
                    }
                }
                json_buffer.clear();
                in_object = false;
            }
        }

        // Watch closed
        let _ = tx_clone
            .send(WatchMessage::Closed { resource_type: rt })
            .await;
    });

    Ok(child)
}

/// Parse a JSON line from kubectl --watch output
fn parse_watch_line(line: &str, resource_type: ResourceType) -> Result<WatchMessage> {
    let json: serde_json::Value =
        serde_json::from_str(line).context("Invalid JSON from kubectl watch")?;

    let metadata = &json["metadata"];
    let name = metadata["name"].as_str().unwrap_or("unknown").to_string();
    let namespace = metadata["namespace"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    // Parse labels
    let labels = metadata["labels"].as_object().map(|obj| {
        obj.iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect()
    });

    // Determine action from resource version changes (simplified)
    // kubectl --watch outputs the full object, we track if it's new or modified
    let action = if metadata["deletionTimestamp"].is_string() {
        ResourceAction::Deleted
    } else {
        // We can't easily distinguish Added vs Modified without tracking state
        // For now, treat all as Modified (the consumer can track first-seen)
        ResourceAction::Modified
    };

    // Get phase from status
    let phase = json["status"]["phase"].as_str().map(ToString::to_string);

    // Get message from status
    let message = json["status"]["message"].as_str().map(ToString::to_string);

    Ok(WatchMessage::Resource {
        resource_type,
        action,
        name,
        namespace,
        phase,
        labels,
        message,
    })
}

/// Run the full E2E monitor with multi-watch streams
#[allow(clippy::too_many_arguments)]
async fn run_full_watch(
    task_id: &str,
    config_path: &str,
    argo_namespace: &str,
    agent_namespace: &str,
    github_interval: u64,
    max_failures: u32,
    template: &str,
    repository_override: Option<&str>,
    emitter: &EventEmitter,
) -> Result<()> {
    // Step 1: Read and parse cto-config.json
    let config_content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    let config: CtoConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {config_path}"))?;

    let play = &config.defaults.play;
    let repository = repository_override.unwrap_or(&play.repository);

    println!(
        "{}",
        format!(
            "Loaded config: repo={}, impl={}, quality={}, testing={}",
            repository, play.implementation_agent, play.quality_agent, play.testing_agent
        )
        .cyan()
    );

    // Step 2: Submit the workflow with config values
    println!(
        "{}",
        format!("Submitting play workflow for task {task_id}...").cyan()
    );

    let output = Command::new("argo")
        .args([
            "submit",
            "--from",
            &format!("workflowtemplate/{template}"),
            "-n",
            argo_namespace,
            "-p",
            &format!("task-id={task_id}"),
            "-p",
            &format!("repository={repository}"),
            "-p",
            &format!("implementation-agent={}", play.implementation_agent),
            "-p",
            &format!("quality-agent={}", play.quality_agent),
            "-p",
            &format!("testing-agent={}", play.testing_agent),
            "-o",
            "json",
        ])
        .output()
        .context("Failed to submit workflow via argo CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to submit workflow: {stderr}"));
    }

    // Parse workflow name from output
    let workflow_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|_| serde_json::json!({}));

    let workflow_name = workflow_json["metadata"]["name"]
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("Failed to get workflow name from submission response"))?;

    println!(
        "{}",
        format!("✓ Workflow submitted: {workflow_name}").green()
    );

    // Step 3: Start multi-watch monitoring
    println!(
        "{}",
        format!("Starting multi-watch monitoring for task {task_id}...").cyan()
    );

    run_multi_watch(
        task_id,
        argo_namespace,
        agent_namespace,
        Some(repository),
        github_interval,
        true, // fetch_logs
        max_failures,
        500, // log_tail
        emitter,
        None, // No remediation config - use run_self_healing_loop for that
    )
    .await
}

/// Run self-healing E2E loop with automatic remediation
///
/// This is the main entry point for E2E testing with self-healing capabilities.
/// On failure, it triggers a remediation agent to fix the issue, waits for
/// `ArgoCD` sync, and retries the workflow.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn run_self_healing_loop(
    task_id: &str,
    config_path: &str,
    argo_namespace: &str,
    agent_namespace: &str,
    github_interval: u64,
    template: &str,
    emitter: &EventEmitter,
) -> Result<()> {
    // Read config
    let config_content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    let config: CtoConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {config_path}"))?;

    let play = &config.defaults.play;
    let remediation_config = config.defaults.remediation.clone();

    // Check if remediation is enabled
    let Some(remediation) = remediation_config else {
        println!(
            "{}",
            "Warning: No remediation config found - self-healing disabled".yellow()
        );
        // Fall back to regular full watch without remediation
        return run_full_watch(
            task_id,
            config_path,
            argo_namespace,
            agent_namespace,
            github_interval,
            5, // max_failures
            template,
            None,
            emitter,
        )
        .await;
    };

    println!(
        "{}",
        format!(
            "Self-healing enabled: remediation repo={}, agent={}, max_iterations={}",
            remediation.repository, remediation.agent, remediation.max_iterations
        )
        .cyan()
    );

    let repository = &play.repository;
    let mut iteration: u32 = 0;
    #[allow(unused_assignments)] // Set inside loop before first use
    let mut current_workflow_name: Option<String> = None;

    // Main self-healing loop
    loop {
        iteration += 1;

        if iteration > remediation.max_iterations + 1 {
            println!(
                "{}",
                format!(
                    "Max remediation iterations ({}) exceeded - giving up",
                    remediation.max_iterations
                )
                .red()
            );
            return Err(anyhow::anyhow!(
                "Max remediation iterations exceeded without success"
            ));
        }

        println!(
            "{}",
            format!(
                "=== Self-Healing Loop Iteration {} (max: {}) ===",
                iteration,
                remediation.max_iterations + 1
            )
            .cyan()
            .bold()
        );

        // Submit workflow
        println!(
            "{}",
            format!("Submitting play workflow for task {task_id}...").cyan()
        );

        let output = Command::new("argo")
            .args([
                "submit",
                "--from",
                &format!("workflowtemplate/{template}"),
                "-n",
                argo_namespace,
                "-p",
                &format!("task-id={task_id}"),
                "-p",
                &format!("repository={repository}"),
                "-p",
                &format!("implementation-agent={}", play.implementation_agent),
                "-p",
                &format!("quality-agent={}", play.quality_agent),
                "-p",
                &format!("testing-agent={}", play.testing_agent),
                "-o",
                "json",
            ])
            .output()
            .context("Failed to submit workflow via argo CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to submit workflow: {stderr}"));
        }

        let workflow_json: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|_| serde_json::json!({}));

        let workflow_name = workflow_json["metadata"]["name"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| anyhow::anyhow!("Failed to get workflow name"))?;

        current_workflow_name = Some(workflow_name.clone());

        println!(
            "{}",
            format!("Workflow submitted: {workflow_name}").green()
        );

        // Monitor workflow with remediation support
        let result = run_multi_watch(
            task_id,
            argo_namespace,
            agent_namespace,
            Some(repository),
            github_interval,
            true,
            1, // Stop on first failure for remediation
            500,
            emitter,
            Some(remediation.clone()),
        )
        .await;

        match result {
            Ok(()) => {
                // Success!
                println!(
                    "{}",
                    format!("Workflow completed successfully on iteration {iteration}").green()
                );
                return Ok(());
            }
            Err(e) => {
                let error_msg = e.to_string();

                // Check if this is a remediation-triggerable failure
                if error_msg.contains("REMEDIATION_NEEDED:") {
                    println!(
                        "{}",
                        format!("Workflow failed, triggering remediation (iteration {iteration})...")
                            .yellow()
                    );

                    // Extract failure context from error
                    let failure_context = FailureContext {
                        workflow_name: workflow_name.clone(),
                        failed_resource: current_workflow_name.clone().unwrap_or_default(),
                        resource_type: "Workflow".to_string(),
                        phase: "Failed".to_string(),
                        error_message: Some(error_msg.clone()),
                        logs: None, // TODO: extract from error
                        container: None,
                        exit_code: None,
                        timestamp: Utc::now(),
                    };

                    // Trigger remediation
                    let coderun_name = trigger_remediation(
                        &remediation,
                        &failure_context,
                        task_id,
                        iteration,
                        agent_namespace,
                    )?;

                    // Wait for remediation CodeRun to complete
                    println!(
                        "{}",
                        format!("Waiting for remediation CodeRun: {coderun_name}...").cyan()
                    );

                    let (success, message) =
                        wait_for_coderun(&coderun_name, agent_namespace, 3600).await?;

                    if !success {
                        println!(
                            "{}",
                            format!(
                                "Remediation CodeRun failed: {}",
                                message.unwrap_or_default()
                            )
                            .red()
                        );
                        continue; // Try again
                    }

                    // Get PR URL from CodeRun
                    if let Some(pr_url) = get_coderun_pr_url(&coderun_name, agent_namespace) {
                        println!("{}", format!("Remediation PR created: {pr_url}").green());

                        // Wait for PR to be merged and ArgoCD to sync
                        println!(
                            "{}",
                            "Waiting for PR merge and ArgoCD sync...".cyan()
                        );

                        // Note: In production, we'd monitor the PR status and wait for merge
                        // For now, we wait for ArgoCD to sync (which happens after merge)
                        let synced = wait_for_argocd_sync(
                            "cto-controller", // ArgoCD app name
                            None,             // No specific commit
                            remediation.sync_timeout_secs,
                        )
                        .await?;

                        if !synced {
                            println!(
                                "{}",
                                "Warning: ArgoCD sync timeout - retrying anyway".yellow()
                            );
                        }

                        // Clean up old workflow before retry
                        let _ = Command::new("argo")
                            .args(["delete", &workflow_name, "-n", argo_namespace])
                            .output();
                    } else {
                        println!(
                            "{}",
                            "Warning: No PR URL from remediation - agent may have fixed inline"
                                .yellow()
                        );
                    }

                    // Continue to next iteration (retry workflow)
                    continue;
                }

                // Non-remediation error - propagate
                return Err(e);
            }
        }
    }
}

/// Run multi-watch monitoring for all resources
///
/// If `remediation_config` is provided and a failure occurs, returns an error
/// with prefix `REMEDIATION_NEEDED:` to signal the caller to trigger remediation.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn run_multi_watch(
    task_id: &str,
    argo_namespace: &str,
    agent_namespace: &str,
    repository: Option<&str>,
    github_interval: u64,
    fetch_logs: bool,
    max_failures: u32,
    log_tail: u32,
    emitter: &EventEmitter,
    remediation_config: Option<RemediationConfig>,
) -> Result<()> {
    let remediation_enabled = remediation_config.is_some();
    let label_selector = format!("task-id={task_id}");

    // Create channel for watch events
    let (tx, mut rx) = mpsc::channel::<WatchMessage>(100);

    // Spawn all watch processes
    let children = vec![
        // Watch workflows in argo namespace
        spawn_watch(
            ResourceType::Workflow,
            argo_namespace,
            Some(&label_selector),
            &tx,
        )?,
        // Watch CodeRuns in agent-platform namespace
        spawn_watch(
            ResourceType::CodeRun,
            agent_namespace,
            Some(&label_selector),
            &tx,
        )?,
        // Watch DocsRuns in agent-platform namespace
        spawn_watch(
            ResourceType::DocsRun,
            agent_namespace,
            Some(&label_selector),
            &tx,
        )?,
        // Watch Sensors in argo namespace (no label filter - watch all)
        spawn_watch(ResourceType::Sensor, argo_namespace, None, &tx)?,
        // Watch Pods in agent-platform namespace
        spawn_watch(
            ResourceType::Pod,
            agent_namespace,
            Some(&label_selector),
            &tx,
        )?,
    ];

    // Emit started event
    emitter.emit(&LoopEvent::Started {
        task_id: task_id.to_string(),
        watching: vec![
            format!("workflows.argoproj.io (ns: {argo_namespace})"),
            format!("coderuns.agents.platform (ns: {agent_namespace})"),
            format!("docsruns.agents.platform (ns: {agent_namespace})"),
            format!("sensors.argoproj.io (ns: {argo_namespace})"),
            format!("pods (ns: {agent_namespace})"),
        ],
        timestamp: Utc::now(),
    })?;

    // Spawn GitHub polling task if repository is specified
    let github_tx = tx.clone();
    let github_repo = repository.map(ToString::to_string);
    let github_task_id = task_id.to_string();

    let github_handle = if github_repo.is_some() {
        Some(tokio::spawn(async move {
            poll_github_state(
                &github_task_id,
                github_repo.as_deref(),
                github_interval,
                github_tx,
            )
            .await;
        }))
    } else {
        None
    };

    // Track state for failure detection
    let mut consecutive_failures: u32 = 0;
    let mut workflow_completed = false;
    let mut last_workflow_phase = String::new();

    // Process events from all watches
    while let Some(msg) = rx.recv().await {
        match msg {
            WatchMessage::Resource {
                resource_type,
                action,
                name,
                namespace,
                phase,
                labels,
                message,
            } => {
                // Emit resource event
                emitter.emit(&LoopEvent::Resource {
                    task_id: task_id.to_string(),
                    resource_type,
                    action: action.clone(),
                    name: name.clone(),
                    namespace: namespace.clone(),
                    phase: phase.clone(),
                    labels: labels.clone(),
                    message: message.clone(),
                    timestamp: Utc::now(),
                })?;

                // Check for workflow completion/failure
                // NOTE: Only workflow state changes affect the consecutive_failures counter
                if resource_type == ResourceType::Workflow {
                    if let Some(ref p) = phase {
                        if p != &last_workflow_phase {
                            last_workflow_phase.clone_from(p);

                            if p == "Succeeded" {
                                // Reset failure counter on workflow success
                                consecutive_failures = 0;
                                emitter.emit(&LoopEvent::Completed {
                                    play_id: name.clone(),
                                    duration_seconds: 0, // TODO: calculate from timestamps
                                    timestamp: Utc::now(),
                                })?;
                                workflow_completed = true;
                            } else if p == "Running" {
                                // Reset failure counter when workflow is running again
                                // (e.g., after a retry)
                                consecutive_failures = 0;
                            } else if p == "Failed" || p == "Error" {
                                consecutive_failures += 1;

                                // Fetch logs on failure
                                let logs = if fetch_logs {
                                    get_step_logs(&name, &namespace, log_tail).ok()
                                } else {
                                    None
                                };

                                emitter.emit(&LoopEvent::Failure {
                                    play_id: name.clone(),
                                    stage: None,
                                    failed_step: None,
                                    logs: logs.clone(),
                                    consecutive_failures,
                                    timestamp: Utc::now(),
                                })?;

                                if max_failures > 0 && consecutive_failures >= max_failures {
                                    // If remediation is enabled, return error to trigger remediation
                                    if remediation_enabled {
                                        // Cleanup watch processes before returning
                                        for mut child in children {
                                            let _ = child.kill().await;
                                        }
                                        if let Some(handle) = github_handle {
                                            handle.abort();
                                        }

                                        return Err(anyhow::anyhow!(
                                            "REMEDIATION_NEEDED: Workflow {} failed. Logs: {}",
                                            name,
                                            logs.unwrap_or_else(|| "No logs available".to_string())
                                        ));
                                    }

                                    emitter.emit(&LoopEvent::Stopped {
                                        play_id: name.clone(),
                                        reason: format!(
                                            "Max consecutive failures reached ({max_failures})"
                                        ),
                                        timestamp: Utc::now(),
                                    })?;
                                    workflow_completed = true;
                                }
                            }
                        }
                    }
                }

                // Emit informational failure events for pod failures
                // NOTE: Pod/CRD failures are informational but don't affect the
                // consecutive_failures counter - only workflow state changes do.
                // The workflow will eventually transition to Failed if pods fail,
                // which is when the counter should increment.
                if resource_type == ResourceType::Pod {
                    if let Some(ref p) = phase {
                        if p == "Failed" {
                            let logs = if fetch_logs {
                                get_step_logs(&name, &namespace, log_tail).ok()
                            } else {
                                None
                            };

                            // Emit failure event for visibility, but don't increment
                            // consecutive_failures - that's tracked by workflow state only
                            emitter.emit(&LoopEvent::Failure {
                                play_id: task_id.to_string(),
                                stage: None,
                                failed_step: Some(WorkflowStep {
                                    id: name.clone(),
                                    name: name.clone(),
                                    step_type: "Pod".to_string(),
                                    phase: p.clone(),
                                    pod_name: Some(name.clone()),
                                    exit_code: None,
                                    message: message.clone(),
                                    started_at: None,
                                    finished_at: None,
                                }),
                                logs,
                                // Report current count but don't increment for pods
                                consecutive_failures,
                                timestamp: Utc::now(),
                            })?;
                        }
                    }
                }
            }
            WatchMessage::GitHub {
                task_id: gh_task_id,
                repository,
                pull_request,
            } => {
                // Emit GitHub event
                emitter.emit(&LoopEvent::Github {
                    task_id: gh_task_id,
                    repository,
                    pull_request,
                    timestamp: Utc::now(),
                })?;
            }
            WatchMessage::Error {
                resource_type,
                error,
            } => {
                warn!("Watch error for {}: {}", resource_type, error);
            }
            WatchMessage::Closed { resource_type } => {
                warn!("Watch closed for {}", resource_type);
                // Could restart the watch here if needed
            }
        }

        if workflow_completed {
            break;
        }
    }

    // Cleanup
    for mut child in children {
        let _ = child.kill().await;
    }

    if let Some(handle) = github_handle {
        handle.abort();
    }

    Ok(())
}

/// Poll GitHub for PR state and send events through the channel
async fn poll_github_state(
    task_id: &str,
    repository: Option<&str>,
    interval: u64,
    tx: mpsc::Sender<WatchMessage>,
) {
    let Some(repo) = repository else {
        return;
    };

    loop {
        // Use gh CLI to get PR state (poll immediately, then sleep)
        let pr_state = get_github_pr_state(repo, task_id).await;

        match pr_state {
            Ok(state) => {
                // Send GitHub event through the channel
                let msg = WatchMessage::GitHub {
                    task_id: task_id.to_string(),
                    repository: repo.to_string(),
                    pull_request: state.clone(),
                };

                if tx.send(msg).await.is_err() {
                    // Channel closed, exit the polling loop
                    debug!("GitHub polling channel closed, stopping");
                    break;
                }

                if let Some(ref pr) = state {
                    debug!(
                        "GitHub PR #{} state: {} (mergeable: {:?})",
                        pr.number, pr.state, pr.mergeable
                    );
                }
            }
            Err(e) => {
                warn!("Failed to get GitHub PR state: {}", e);
            }
        }

        // Sleep after polling, so first event is immediate
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

/// Get GitHub PR state using gh CLI
async fn get_github_pr_state(repository: &str, task_id: &str) -> Result<Option<PullRequestState>> {
    // List PRs with the task label
    let output = AsyncCommand::new("gh")
        .args([
            "pr",
            "list",
            "-R",
            repository,
            "-l",
            &format!("task-{task_id}"),
            "--json",
            "number,state,title,mergeable,isDraft,labels,reviews,statusCheckRollup",
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(None);
    }

    let prs: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)?;

    if let Some(pr) = prs.first() {
        let labels: Vec<String> = pr["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let reviews: Vec<ReviewState> = pr["reviews"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        Some(ReviewState {
                            author: r["author"]["login"].as_str()?.to_string(),
                            state: r["state"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let checks: Vec<CheckState> = pr["statusCheckRollup"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        Some(CheckState {
                            name: c["name"].as_str()?.to_string(),
                            status: c["status"].as_str()?.to_string(),
                            conclusion: c["conclusion"].as_str().map(ToString::to_string),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        return Ok(Some(PullRequestState {
            number: pr["number"].as_u64().unwrap_or(0),
            state: pr["state"].as_str().unwrap_or("unknown").to_string(),
            title: pr["title"].as_str().unwrap_or("").to_string(),
            mergeable: pr["mergeable"].as_str().map(|s| s == "MERGEABLE"),
            draft: pr["isDraft"].as_bool().unwrap_or(false),
            labels,
            reviews,
            checks,
        }));
    }

    Ok(None)
}

// =============================================================================
// Logs: Get logs from workflow steps
// =============================================================================

/// Get logs for a workflow, optionally for a specific step
fn get_logs(
    play_id: &str,
    step: Option<&str>,
    namespace: &str,
    tail: u32,
    errors_only: bool,
) -> Result<LogsResponse> {
    let logs = if let Some(step_name) = step {
        // Get logs for specific step/pod
        get_step_logs(step_name, namespace, tail)?
    } else {
        // Get logs from failed step(s) in the workflow
        let status = get_workflow_status(play_id, namespace)?;
        let mut all_logs = String::new();

        if status.failed_steps.is_empty() {
            // No failures, get logs from most recent step
            if let Some(recent) = status.steps.last() {
                if let Some(ref pod_name) = recent.pod_name {
                    let _ = writeln!(all_logs, "=== {} ({}) ===", recent.name, recent.phase);
                    all_logs.push_str(&get_step_logs(pod_name, namespace, tail)?);
                }
            }
        } else {
            // Get logs from each failed step
            for failed in &status.failed_steps {
                if let Some(ref pod_name) = failed.pod_name {
                    let _ = writeln!(all_logs, "=== {} (FAILED) ===", failed.name);
                    if let Some(ref msg) = failed.message {
                        let _ = writeln!(all_logs, "Message: {msg}");
                    }
                    all_logs.push_str(&get_step_logs(pod_name, namespace, tail)?);
                    all_logs.push('\n');
                }
            }
        }

        all_logs
    };

    let filtered = if errors_only {
        filter_error_logs(&logs)
    } else {
        logs
    };

    let line_count = filtered.lines().count();

    Ok(LogsResponse {
        play_id: play_id.to_string(),
        step: step.map(ToString::to_string),
        namespace: namespace.to_string(),
        logs: filtered,
        line_count,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Get logs for a specific step/pod
fn get_step_logs(pod_name: &str, namespace: &str, tail: u32) -> Result<String> {
    debug!("Getting logs for pod {} in {}", pod_name, namespace);

    // First try argo logs (works even for completed pods)
    let output = Command::new("argo")
        .args([
            "logs",
            pod_name,
            "-n",
            namespace,
            "--tail",
            &tail.to_string(),
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let logs = String::from_utf8_lossy(&out.stdout).to_string();
            if !logs.is_empty() {
                return Ok(logs);
            }
        }
    }

    // Fallback to kubectl logs
    let output = Command::new("kubectl")
        .args([
            "logs",
            pod_name,
            "-n",
            namespace,
            "--tail",
            &tail.to_string(),
            "--all-containers=true",
        ])
        .output()
        .context("Failed to execute kubectl logs")?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    // Try previous logs
    let output = Command::new("kubectl")
        .args([
            "logs",
            pod_name,
            "-n",
            namespace,
            "--tail",
            &tail.to_string(),
            "--all-containers=true",
            "--previous",
        ])
        .output()
        .context("Failed to execute kubectl logs --previous")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Filter logs to only include error-related lines
fn filter_error_logs(logs: &str) -> String {
    let error_patterns = [
        "error",
        "Error",
        "ERROR",
        "failed",
        "Failed",
        "FAILED",
        "panic",
        "PANIC",
        "fatal",
        "FATAL",
        "exception",
        "Exception",
        "EXCEPTION",
        "OOMKilled",
        "CrashLoopBackOff",
        "clippy::",
        "warning:",
    ];

    logs.lines()
        .filter(|line| error_patterns.iter().any(|p| line.contains(p)))
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// OpenMemory Verification Commands
// =============================================================================

/// Handle memory subcommands for verification (not remediation)
async fn handle_memory_command(action: MemoryCommands) -> Result<()> {
    let openmemory_url = std::env::var("OPENMEMORY_URL")
        .unwrap_or_else(|_| "http://openmemory.openmemory.svc.cluster.local:8080".to_string());

    match action {
        MemoryCommands::List {
            task_id,
            agent,
            limit,
        } => {
            let mut url = format!("{openmemory_url}/api/v1/memories?limit={limit}");
            if let Some(t) = task_id {
                let _ = write!(url, "&task_id={t}");
            }
            if let Some(a) = agent {
                let _ = write!(url, "&agent={a}");
            }
            let resp = reqwest::get(&url).await?;
            let json: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        MemoryCommands::Query {
            text,
            agent,
            limit,
            include_waypoints,
        } => {
            let client = reqwest::Client::new();
            let mut body = serde_json::json!({
                "query": text,
                "limit": limit,
                "include_waypoints": include_waypoints
            });
            if let Some(a) = agent {
                body["agent"] = serde_json::json!(a);
            }
            let resp = client
                .post(format!("{openmemory_url}/api/v1/search"))
                .json(&body)
                .send()
                .await?;
            let json: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        MemoryCommands::Stats { agent } => {
            let mut url = format!("{openmemory_url}/api/v1/stats");
            if let Some(a) = agent {
                let _ = write!(url, "?agent={a}");
            }
            let resp = reqwest::get(&url).await?;
            let json: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        MemoryCommands::Get { id } => {
            let resp = reqwest::get(format!("{openmemory_url}/api/v1/memories/{id}")).await?;
            let json: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}

/// Output result in requested format
fn output_result<T: Serialize>(result: &T, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(result)?;
            println!("{json}");
        }
        OutputFormat::Text => {
            // For text format, we just print the JSON for now
            // Could be enhanced with prettier formatting
            let json = serde_json::to_string_pretty(result)?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Reset the E2E environment - clean cluster and reset test repo
#[allow(clippy::too_many_lines)]
fn reset_environment(
    namespace: &str,
    org: &str,
    repo: &str,
    skip_k8s: bool,
    skip_github: bool,
    force: bool,
) -> Result<ResetResponse> {
    // Prompt for confirmation unless force flag is set
    if !force {
        use std::io::{self, Write};
        print!(
            "{}",
            "WARNING: This will delete all workflows, pods, ConfigMaps, PVCs, and reset the GitHub repo.\nContinue? [y/N]: "
                .yellow()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Err(anyhow::anyhow!("Reset cancelled by user"));
        }
    }

    let mut k8s_cleanup = CleanupResult {
        workflows_deleted: 0,
        pods_deleted: 0,
        configmaps_deleted: 0,
        pvcs_deleted: 0,
        skipped: skip_k8s,
    };

    // Kubernetes cleanup
    if !skip_k8s {
        println!("{}", "Cleaning up Kubernetes resources...".cyan());

        // Delete workflows
        let output = Command::new("kubectl")
            .args([
                "delete",
                "workflows",
                "--all",
                "-n",
                namespace,
                "--force",
                "--grace-period=0",
            ])
            .output()
            .context("Failed to delete workflows")?;
        if output.status.success() {
            k8s_cleanup.workflows_deleted = count_deleted(&output.stdout);
            println!("  {} Deleted workflows", "✓".green());
        }

        // Delete pods
        let output = Command::new("kubectl")
            .args([
                "delete",
                "pods",
                "--all",
                "-n",
                namespace,
                "--force",
                "--grace-period=0",
            ])
            .output()
            .context("Failed to delete pods")?;
        if output.status.success() {
            k8s_cleanup.pods_deleted = count_deleted(&output.stdout);
            println!("  {} Deleted pods", "✓".green());
        }

        // Delete test ConfigMaps (play-*, test-*, coderun-*, docsrun-*)
        for pattern in &["play-", "test-", "coderun-", "docsrun-", "remediation-"] {
            let list_output = Command::new("kubectl")
                .args(["get", "configmaps", "-n", namespace, "-o", "name"])
                .output()?;

            if list_output.status.success() {
                let output_str = String::from_utf8_lossy(&list_output.stdout);
                let cms: Vec<&str> = output_str.lines().filter(|l| l.contains(pattern)).collect();

                for cm in &cms {
                    let delete_result = Command::new("kubectl")
                        .args(["delete", cm, "-n", namespace, "--force", "--grace-period=0"])
                        .output();
                    if delete_result.is_ok_and(|o| o.status.success()) {
                        k8s_cleanup.configmaps_deleted += 1;
                    }
                }
            }
        }
        println!(
            "  {} Deleted {} ConfigMaps",
            "✓".green(),
            k8s_cleanup.configmaps_deleted
        );

        // Delete test PVCs (workspace-play-*, workspace-test-*)
        for pattern in &["workspace-play-", "workspace-test-"] {
            let list_output = Command::new("kubectl")
                .args(["get", "pvc", "-n", namespace, "-o", "name"])
                .output()?;

            if list_output.status.success() {
                let output_str = String::from_utf8_lossy(&list_output.stdout);
                let pvcs: Vec<&str> = output_str.lines().filter(|l| l.contains(pattern)).collect();

                for pvc in &pvcs {
                    let delete_result = Command::new("kubectl")
                        .args([
                            "delete",
                            pvc,
                            "-n",
                            namespace,
                            "--force",
                            "--grace-period=0",
                        ])
                        .output();
                    if delete_result.is_ok_and(|o| o.status.success()) {
                        k8s_cleanup.pvcs_deleted += 1;
                    }
                }
            }
        }
        println!(
            "  {} Deleted {} PVCs",
            "✓".green(),
            k8s_cleanup.pvcs_deleted
        );
    }

    // GitHub repository reset
    let github_reset = if skip_github {
        None
    } else {
        println!("{}", "Resetting GitHub repository...".cyan());
        Some(reset_github_repo(org, repo, force)?)
    };

    Ok(ResetResponse {
        success: true,
        k8s_cleanup,
        github_reset,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Reset GitHub repository - delete and recreate with minimal structure
#[allow(clippy::too_many_lines)]
fn reset_github_repo(org: &str, repo: &str, _force: bool) -> Result<GithubResetResult> {
    let full_repo = format!("{org}/{repo}");
    let mut result = GithubResetResult {
        repo: full_repo.clone(),
        deleted: false,
        created: false,
        pushed: false,
    };

    // Check if repo exists and delete it
    let check = Command::new("gh")
        .args(["repo", "view", &full_repo])
        .output()?;

    if check.status.success() {
        println!("  Deleting existing repository...");
        let delete = Command::new("gh")
            .args(["repo", "delete", &full_repo, "--yes"])
            .output()?;

        if delete.status.success() {
            result.deleted = true;
            println!("  {} Deleted {full_repo}", "✓".green());
        } else {
            let err = String::from_utf8_lossy(&delete.stderr);
            return Err(anyhow::anyhow!("Failed to delete repo: {err}"));
        }
    }

    // Create new repository
    println!("  Creating new repository...");
    let create = Command::new("gh")
        .args([
            "repo",
            "create",
            &full_repo,
            "--private",
            "--description",
            "E2E test repository for CTO platform",
        ])
        .output()?;

    if create.status.success() {
        result.created = true;
        println!("  {} Created {full_repo}", "✓".green());
    } else {
        let err = String::from_utf8_lossy(&create.stderr);
        return Err(anyhow::anyhow!("Failed to create repo: {err}"));
    }

    // Initialize with minimal structure using a temp dir and git
    println!("  Initializing repository structure...");

    // Create temp directory and initialize repo
    let temp_dir = std::env::temp_dir().join(format!("play-monitor-{repo}"));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    // Create README
    let readme_content = format!(
        "# {repo}\n\nE2E test repository for CTO platform.\n\nThis repo is managed by play-monitor."
    );
    std::fs::write(temp_dir.join("README.md"), readme_content)?;

    // Create .gitignore
    std::fs::write(temp_dir.join(".gitignore"), "target/\n*.log\n.env\n")?;

    // Initialize git and push
    let git_init = Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()?;

    if git_init.status.success() {
        // Configure git user locally for this repo (required for commit)
        let _ = Command::new("git")
            .args(["config", "user.email", "automation@5dlabs.io"])
            .current_dir(&temp_dir)
            .output();
        let _ = Command::new("git")
            .args(["config", "user.name", "5DLabs Automation"])
            .current_dir(&temp_dir)
            .output();

        let _ = Command::new("git")
            .args(["add", "."])
            .current_dir(&temp_dir)
            .output();

        let commit_result = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&temp_dir)
            .output();

        if commit_result.is_err() || !commit_result.as_ref().is_ok_and(|o| o.status.success()) {
            let err_msg = commit_result
                .as_ref()
                .map_or_else(std::string::ToString::to_string, |o| {
                    String::from_utf8_lossy(&o.stderr).to_string()
                });
            println!("  {} Git commit failed: {err_msg}", "⚠".yellow());
        }

        let _ = Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(&temp_dir)
            .output();

        // Use HTTPS with token if GITHUB_TOKEN is set (for automation)
        // Otherwise fall back to SSH (for local dev)
        let remote_url = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            format!("https://x-access-token:{token}@github.com/{full_repo}.git")
        } else {
            format!("git@github.com:{full_repo}.git")
        };

        let _ = Command::new("git")
            .args(["remote", "add", "origin", &remote_url])
            .current_dir(&temp_dir)
            .output();

        let push = Command::new("git")
            .args(["push", "-u", "origin", "main", "--force"])
            .current_dir(&temp_dir)
            .output()?;

        if push.status.success() {
            result.pushed = true;
            println!("  {} Initialized repository", "✓".green());
        } else {
            let err = String::from_utf8_lossy(&push.stderr);
            println!("  {} Git push failed: {err}", "⚠".yellow());
        }
    }

    // Cleanup temp dir
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(result)
}

/// Count deleted resources from kubectl output
fn count_deleted(output: &[u8]) -> i32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let count = String::from_utf8_lossy(output)
        .lines()
        .filter(|l| l.contains("deleted"))
        .count() as i32;
    count
}

/// Run/submit a play workflow via Argo CLI
fn run_workflow(config: &RunWorkflowConfig<'_>) -> Result<RunResponse> {
    println!(
        "{}",
        format!("Submitting play workflow for task {}...", config.task_id).cyan()
    );

    // Generate descriptive workflow name
    // Format: play-{run_type}-t{task_id}-{agent_short}-{cli}-{uid}
    let agent_short = config
        .agent
        .strip_prefix("5DLabs-")
        .unwrap_or(config.agent)
        .to_lowercase();
    let uid: String = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let workflow_name = format!(
        "play-{}-t{}-{}-{}-{}",
        config.run_type, config.task_id, agent_short, config.agent_cli, uid
    );

    // Submit workflow using argo CLI
    let output = Command::new("argo")
        .args([
            "submit",
            "--from",
            &format!("workflowtemplate/{}", config.template),
            "-n",
            config.namespace,
            "-p",
            &format!("task-id={}", config.task_id),
            "-p",
            &format!("repository={}", config.repository),
            "-p",
            &format!("service={}", config.service),
            "-p",
            &format!("implementation-agent={}", config.agent),
            "-p",
            &format!("implementation-cli={}", config.agent_cli),
            "-p",
            &format!("implementation-model={}", config.model),
            "-p",
            "quality-agent=5DLabs-Cleo",
            "-p",
            "quality-cli=claude",
            "-p",
            "quality-model=claude-sonnet-4-20250514",
            "-p",
            "testing-agent=5DLabs-Tess",
            "-p",
            "testing-cli=claude",
            "-p",
            "testing-model=claude-sonnet-4-20250514",
            "--name",
            &workflow_name,
            "-o",
            "json",
        ])
        .output()
        .context("Failed to submit workflow via argo CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(RunResponse {
            success: false,
            workflow_name: None,
            task_id: config.task_id.to_string(),
            repository: config.repository.to_string(),
            timestamp: Utc::now(),
            error: Some(stderr.to_string()),
        });
    }

    // Parse workflow name from output
    let workflow_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|_| serde_json::json!({}));

    let workflow_name = workflow_json["metadata"]["name"]
        .as_str()
        .map(ToString::to_string);

    println!(
        "{}",
        format!(
            "✓ Workflow submitted: {}",
            workflow_name.as_deref().unwrap_or("unknown")
        )
        .green()
    );

    Ok(RunResponse {
        success: true,
        workflow_name,
        task_id: config.task_id.to_string(),
        repository: config.repository.to_string(),
        timestamp: Utc::now(),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_error_logs() {
        let logs = r"INFO: Starting application
ERROR: Failed to connect
DEBUG: Checking status
error: something went wrong
INFO: Continuing";

        let filtered = filter_error_logs(logs);
        assert!(filtered.contains("ERROR: Failed to connect"));
        assert!(filtered.contains("error: something went wrong"));
        assert!(!filtered.contains("INFO: Starting"));
        assert!(!filtered.contains("DEBUG: Checking"));
    }

    #[test]
    fn test_determine_stage_from_steps() {
        let steps = vec![WorkflowStep {
            id: "abc123".to_string(),
            name: "rex-implementation".to_string(),
            step_type: "Pod".to_string(),
            phase: "Running".to_string(),
            pod_name: Some("play-42-rex-abc".to_string()),
            exit_code: None,
            message: None,
            started_at: Some("2024-01-01T00:00:00Z".to_string()),
            finished_at: None,
        }];

        assert_eq!(
            determine_stage_from_steps(&steps),
            Some("implementation".to_string())
        );

        let steps = vec![WorkflowStep {
            id: "xyz789".to_string(),
            name: "cleo-quality".to_string(),
            step_type: "Pod".to_string(),
            phase: "Failed".to_string(),
            pod_name: Some("play-42-cleo-xyz".to_string()),
            exit_code: Some(1),
            message: Some("clippy failed".to_string()),
            started_at: Some("2024-01-01T00:05:00Z".to_string()),
            finished_at: Some("2024-01-01T00:06:00Z".to_string()),
        }];

        assert_eq!(
            determine_stage_from_steps(&steps),
            Some("code-quality".to_string())
        );
    }

    #[test]
    fn test_calculate_duration() {
        // Both times present
        let duration =
            calculate_duration(Some("2024-01-01T00:00:00Z"), Some("2024-01-01T00:05:00Z"));
        assert_eq!(duration, Some(300)); // 5 minutes = 300 seconds

        // Missing start time
        let duration = calculate_duration(None, Some("2024-01-01T00:05:00Z"));
        assert_eq!(duration, None);

        // Invalid format
        let duration = calculate_duration(Some("invalid"), Some("2024-01-01T00:05:00Z"));
        assert_eq!(duration, None);
    }

    #[test]
    fn test_parse_workflow_nodes() {
        let nodes_json = serde_json::json!({
            "node-1": {
                "displayName": "rex-impl",
                "type": "Pod",
                "phase": "Succeeded",
                "id": "play-42-rex-abc",
                "startedAt": "2024-01-01T00:00:00Z",
                "finishedAt": "2024-01-01T00:05:00Z"
            },
            "node-2": {
                "displayName": "cleo-quality",
                "type": "Pod",
                "phase": "Failed",
                "id": "play-42-cleo-xyz",
                "message": "exit code 1",
                "startedAt": "2024-01-01T00:05:00Z",
                "finishedAt": "2024-01-01T00:06:00Z"
            },
            "node-3": {
                "displayName": "workflow-root",
                "type": "Steps",
                "phase": "Running"
            }
        });

        let (steps, failed) = parse_workflow_nodes(&nodes_json);

        // Should only include Pod types
        assert_eq!(steps.len(), 2);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].name, "cleo-quality");
    }
}
