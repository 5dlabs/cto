//! Play Monitor CLI
//!
//! A comprehensive CLI tool for monitoring play workflows and all platform resources.
//! Uses kubectl --watch for real-time streaming of workflows, CRDs, pods, and sensors.
//! Emits unified JSON events for Cursor agent E2E feedback loop automation.

mod alerts;
pub mod ci;
mod dedup;
mod github;
mod k8s;
pub mod loki;
pub mod play;
mod reconcile;
pub mod scanner;
pub mod sensors;
mod templates;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Self-healing platform monitor - detects issues and spawns remediation agents
#[derive(Parser)]
#[command(name = "heal")]
#[command(about = "Self-healing platform monitor - detects issues and spawns remediation agents")]
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
    /// Run/submit a play workflow via Argo CLI (reads parameters from cto-config.json)
    Run {
        /// Path to cto-config.json file (required for agent configurations)
        #[arg(long, default_value = "cto-config.json")]
        config: String,

        /// Task ID for the play
        #[arg(long)]
        task_id: String,

        /// Repository to work on (overrides config if specified)
        #[arg(long)]
        repository: Option<String>,

        /// Service name (overrides config if specified)
        #[arg(long)]
        service: Option<String>,

        /// Docs repository (overrides config if specified)
        #[arg(long)]
        docs_repository: Option<String>,

        /// Docs project directory (overrides config if specified)
        #[arg(long)]
        docs_project_directory: Option<String>,

        /// Run type for workflow naming (e.g., monitor-test, diagnostic)
        #[arg(long, default_value = "monitor-test")]
        run_type: String,
    },
    /// [E2E] Start the self-healing E2E loop - creates Monitor `CodeRun` and exits
    Start {
        /// Path to cto-config.json
        #[arg(long, default_value = "monitor/cto-config.json")]
        config: String,
    },
    /// [E2E] Run monitor logic (called inside Monitor pod)
    Monitor {
        /// Path to cto-config.json (contains workflow parameters)
        #[arg(long, default_value = "cto-config.json")]
        config: String,

        /// Current iteration number (1 = first run)
        #[arg(long, default_value = "1")]
        iteration: u32,

        /// Maximum iterations before giving up
        #[arg(long, default_value = "3")]
        max_iterations: u32,

        /// Target repository (e.g., "5dlabs/cto-parallel-test") - overrides config
        #[arg(long)]
        repository: Option<String>,

        /// Service name - overrides config
        #[arg(long)]
        service: Option<String>,

        /// Task ID to run
        #[arg(long, default_value = "1")]
        task_id: String,

        /// Docs repository URL - overrides config
        #[arg(long)]
        docs_repository: Option<String>,

        /// Docs project directory - overrides config
        #[arg(long)]
        docs_project_directory: Option<String>,

        /// Path to acceptance criteria file
        #[arg(long, default_value = "/workspace/watch/acceptance-criteria.md")]
        criteria: String,
    },
    /// [E2E] Run remediation logic (called inside Remediation pod)
    Remediate {
        /// Current iteration number
        #[arg(long)]
        iteration: u32,

        /// Path to issue file
        #[arg(long, default_value = "/workspace/watch/current-issue.md")]
        issue_file: String,

        /// Path to config file
        #[arg(long, default_value = "/workspace/config/cto-config.json")]
        config: String,
    },
    /// Query and analyze `OpenMemory` for agent insights
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },
    /// [ALERTS] Watch for platform alerts and spawn Factory on detection
    AlertWatch {
        /// Namespace to watch for pods
        #[arg(long, default_value = "agent-platform")]
        namespace: String,
        /// Path to templates directory (defaults to `HEALER_TEMPLATES_DIR` env var or `/app/templates/healer`)
        #[arg(
            long,
            env = "HEALER_TEMPLATES_DIR",
            default_value = "/app/templates/healer"
        )]
        prompts_dir: String,
        /// Dry run - detect but don't spawn Factory
        #[arg(long)]
        dry_run: bool,
    },
    /// [ALERTS] Test an alert flow manually
    TestAlert {
        /// Alert ID to test (a1, a2, a3, a4, a5, a7, a8, completion)
        #[arg(long)]
        alert: String,
        /// Pod name for context
        #[arg(long, default_value = "test-pod-123")]
        pod_name: String,
        /// Task ID for context
        #[arg(long, default_value = "test-task")]
        task_id: String,
        /// Agent name for context
        #[arg(long, default_value = "rex")]
        agent: String,
        /// Path to templates directory (defaults to `HEALER_TEMPLATES_DIR` env var or `/app/templates/healer`)
        #[arg(
            long,
            env = "HEALER_TEMPLATES_DIR",
            default_value = "/app/templates/healer"
        )]
        prompts_dir: String,
        /// Dry run - show prompt but don't spawn Factory
        #[arg(long)]
        dry_run: bool,
    },
    /// [ALERTS] Spawn a remediation agent for a detected issue
    SpawnRemediation {
        /// Alert type that triggered this (a1, a2, a7, completion, etc.)
        #[arg(long)]
        alert: String,
        /// Task ID for the remediation
        #[arg(long)]
        task_id: String,
        /// Target pod name (for deduplication and labeling)
        #[arg(long)]
        target_pod: Option<String>,
        /// GitHub issue number (preferred - derives paths from /workspace/watch/issues/issue-{number}/)
        #[arg(long)]
        issue_number: Option<u64>,
        /// Path to issue file (legacy - use --issue-number instead)
        #[arg(long)]
        issue_file: Option<String>,
        /// Path to heal-config.json
        #[arg(long, default_value = "/app/heal-config.json")]
        config: String,
    },
    /// [ALERTS] Fetch all logs for a pod (current, previous, events, describe)
    FetchLogs {
        /// Pod name to fetch logs for
        #[arg(long)]
        pod_name: String,
        /// Namespace of the pod
        #[arg(long, default_value = "cto")]
        namespace: String,
        /// Output directory for log files
        #[arg(long, default_value = "/workspace/watch/logs")]
        output_dir: String,
        /// Tail lines per chunk (0 = all logs)
        #[arg(long, default_value = "10000")]
        tail: u32,
    },
    /// [PLAY] Track parallel task batches and remediate issues
    Play {
        #[command(subcommand)]
        action: PlayCommands,
    },
    /// [INSIGHTS] Agent optimization intelligence
    Insights {
        #[command(subcommand)]
        action: InsightsCommands,
    },
    /// [SERVER] Run CI remediation HTTP server
    Server {
        /// Address to bind to
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,

        /// Repository to monitor (e.g., 5dlabs/cto)
        #[arg(long, default_value = "5dlabs/cto")]
        repository: String,

        /// Kubernetes namespace
        #[arg(long, default_value = "cto")]
        namespace: String,

        /// Path to remediation config file
        #[arg(long)]
        config: Option<String>,
    },
    /// [SCANNER] Scan logs for errors and warnings across platform namespaces
    ScanLogs {
        /// Time window to scan (e.g., "1h", "30m", "2h")
        #[arg(long, default_value = "1h")]
        window: String,

        /// Namespaces to scan (comma-separated, empty = default platform namespaces)
        #[arg(long, default_value = "cto,automation,argocd,infra,observability")]
        namespaces: String,

        /// Minimum error count to report a service
        #[arg(long, default_value = "5")]
        error_threshold: u32,

        /// Minimum warning count to report a service
        #[arg(long, default_value = "20")]
        warn_threshold: u32,

        /// Output format (json, text, summary)
        #[arg(long, default_value = "summary")]
        output: String,

        /// Show remediation candidates
        #[arg(long)]
        show_candidates: bool,
    },
    /// [SCANNER] Trigger remediation from scan results (reads JSON from stdin)
    RemediateFromScan {
        /// Path to healer config file
        #[arg(long, default_value = "/app/heal-config.json")]
        config: String,

        /// Dry run - show what would be created but don't spawn `CodeRuns`
        #[arg(long)]
        dry_run: bool,

        /// Maximum number of `CodeRuns` to spawn (0 = unlimited)
        #[arg(long, default_value = "3")]
        max_coderuns: usize,
    },
    /// [RECONCILER] Check open issues and close ones where condition is resolved
    ReconcileIssues {
        /// Repository to check issues for
        #[arg(long, default_value = "5dlabs/cto")]
        repository: String,

        /// Kubernetes namespace for checking pod/workflow status
        #[arg(long, default_value = "cto")]
        namespace: String,

        /// Labels that identify healer-created issues (comma-separated)
        #[arg(long, default_value = "heal")]
        labels: String,

        /// Dry run - check but don't close issues
        #[arg(long)]
        dry_run: bool,

        /// Maximum issues to process per run
        #[arg(long, default_value = "50")]
        max_issues: usize,

        /// Output format (json or text)
        #[arg(long, default_value = "text")]
        output: String,
    },
    /// [SENSOR] Monitor GitHub Actions for workflow failures and spawn remediation
    Sensor {
        #[command(subcommand)]
        action: SensorCommands,
    },
    /// [PLAY-MONITOR] Monitor running plays with real-time log analysis and anomaly detection
    PlayMonitor {
        /// Kubernetes namespace for `CodeRuns`
        #[arg(long, default_value = "cto")]
        namespace: String,

        /// Poll interval in seconds
        #[arg(long, default_value = "30")]
        poll_interval: u64,

        /// GitHub repository for issue creation (e.g., 5dlabs/cto)
        #[arg(long)]
        repository: Option<String>,

        /// Minimum severity to report (critical, high, medium, low)
        #[arg(long, default_value = "medium")]
        min_severity: String,

        /// Disable automatic issue creation
        #[arg(long)]
        no_issues: bool,

        /// Maximum issues to create per play
        #[arg(long, default_value = "5")]
        max_issues: usize,

        /// Cooldown between issues for same pattern (minutes)
        #[arg(long, default_value = "10")]
        issue_cooldown: i64,

        /// Time window for log queries (minutes)
        #[arg(long, default_value = "5")]
        log_window: i64,

        /// Run once and exit (instead of continuous loop)
        #[arg(long)]
        once: bool,
    },
}

#[derive(Subcommand)]
enum SensorCommands {
    /// Run the GitHub Actions sensor (polls for failures)
    GithubActions {
        /// Repositories to monitor (comma-separated, e.g., "5dlabs/cto,5dlabs/docs")
        #[arg(long, default_value = "5dlabs/cto")]
        repositories: String,

        /// Poll interval in seconds
        #[arg(long, default_value = "60")]
        poll_interval: u64,

        /// How far back to look for failures on first poll (minutes)
        #[arg(long, default_value = "60")]
        lookback_mins: u64,

        /// Create GitHub issues for detected failures
        #[arg(long, default_value = "true")]
        create_issues: bool,

        /// Labels to add to created issues (comma-separated)
        #[arg(long, default_value = "healer,ci-failure")]
        issue_labels: String,

        /// Branches to monitor (comma-separated, empty = all)
        #[arg(long, default_value = "")]
        branches: String,

        /// Workflows to exclude (comma-separated)
        #[arg(long, default_value = "")]
        excluded_workflows: String,

        /// Kubernetes namespace for `CodeRuns`
        #[arg(long, default_value = "cto")]
        namespace: String,

        /// Run once and exit (instead of continuous loop)
        #[arg(long)]
        once: bool,

        /// Path to remediation config file (optional)
        #[arg(long)]
        config: Option<String>,
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

#[derive(Subcommand)]
enum PlayCommands {
    /// Show batch and task status
    Status {
        /// Show specific task details
        #[arg(long)]
        task_id: Option<String>,
        /// Show only stuck/unhealthy tasks
        #[arg(long)]
        stuck: bool,
    },
    /// Trigger remediation for a task
    Remediate {
        /// Task ID to remediate
        #[arg(long)]
        task_id: String,
    },
    /// Show active remediations
    Remediations,
    /// Cancel an active remediation
    CancelRemediation {
        /// Task ID to cancel remediation for
        #[arg(long)]
        task_id: String,
    },
    /// Cleanup play state after completion
    Cleanup {
        /// Force cleanup even if tasks still running
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum InsightsCommands {
    /// Show agent performance insights
    Show {
        /// Agent name to show insights for
        #[arg(long)]
        agent: String,
    },
    /// Show optimization suggestions
    Suggest,
    /// Show common failure patterns
    Failures,
    /// Export insights data
    Export {
        /// Output format (json, csv)
        #[arg(long, default_value = "json")]
        format: String,
    },
}

// =============================================================================
// CTO Config Types - parsed from cto-config.json
// =============================================================================

/// CTO configuration file structure (cto-config.json)
#[derive(Debug, Deserialize)]
struct CtoConfig {
    defaults: CtoDefaults,
    /// Agent configurations (rex, cleo, tess, cipher, blaze, etc.)
    #[serde(default)]
    agents: std::collections::HashMap<String, AgentConfig>,
}

/// Agent configuration (from agents section of config)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentConfig {
    /// GitHub App name (e.g., "5DLabs-Rex")
    github_app: String,
    /// CLI tool (e.g., "factory", "claude", "cursor")
    #[serde(default)]
    cli: String,
    /// Model to use
    #[serde(default)]
    model: String,
    /// Tools configuration
    #[serde(default)]
    tools: Option<AgentTools>,
    /// Model rotation configuration
    #[serde(default)]
    model_rotation: Option<ModelRotationConfig>,
    /// Max retries for this agent
    #[serde(default)]
    max_retries: Option<u32>,
}

/// Agent tools configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AgentTools {
    /// Remote MCP tools
    #[serde(default)]
    remote: Vec<String>,
    /// Local MCP servers
    #[serde(default, rename = "localServers")]
    local_servers: serde_json::Value,
}

/// Model rotation configuration
#[derive(Debug, Clone, Deserialize)]
struct ModelRotationConfig {
    /// Whether rotation is enabled
    #[serde(default)]
    enabled: bool,
    /// List of models to rotate through
    #[serde(default)]
    models: Vec<String>,
}

/// Default configurations
#[derive(Debug, Deserialize)]
struct CtoDefaults {
    play: PlayConfig,
    /// Remediation configuration for self-healing loop
    #[serde(default)]
    remediation: Option<RemediationConfig>,
    /// Monitor configuration for E2E watch loop
    #[serde(default)]
    monitor: Option<MonitorConfig>,
}

/// Monitor agent configuration for E2E watch loop
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MonitorConfig {
    /// GitHub App for monitor agent (e.g., "5DLabs-Rex")
    agent: String,
    /// CLI tool (e.g., "factory", "claude")
    cli: String,
    /// Model to use (e.g., "glm-4-plus")
    model: String,
    /// Template path (e.g., "watch/factory")
    #[serde(default = "default_monitor_template")]
    template: String,
    /// Maximum iterations before giving up
    #[serde(default = "default_max_iterations")]
    max_iterations: u32,
    /// Required kubectl context for creating monitor `CodeRuns` (ensures we deploy to Kind)
    #[serde(default = "default_monitor_context")]
    cluster_context: String,
}

fn default_monitor_template() -> String {
    "watch/factory".to_string()
}

fn default_monitor_context() -> String {
    "kind-cto-dev".to_string()
}

/// Parameters for monitor loop (passed from CLI args)
#[derive(Debug)]
#[allow(dead_code)] // criteria_path reserved for future use
struct MonitorParams {
    iteration: u32,
    max_iterations: u32,
    repository: String,
    service: String,
    task_id: String,
    docs_repository: Option<String>,
    docs_project_directory: String,
    criteria_path: String,
    namespace: String,
    /// Full CTO config for workflow submission
    cto_config: CtoConfig,
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
    /// Docs project directory
    #[serde(default)]
    docs_project_directory: Option<String>,
    /// Working directory
    #[serde(default)]
    working_directory: Option<String>,
    /// Max retries for implementation
    #[serde(default)]
    implementation_max_retries: Option<u32>,
    /// Max retries for frontend
    #[serde(default)]
    frontend_max_retries: Option<u32>,
    /// Max retries for quality
    #[serde(default)]
    quality_max_retries: Option<u32>,
    /// Max retries for security
    #[serde(default)]
    security_max_retries: Option<u32>,
    /// Max retries for testing
    #[serde(default)]
    testing_max_retries: Option<u32>,
    /// Max retries (general fallback)
    #[serde(default)]
    max_retries: Option<u32>,
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

/// Healer configuration for both server (inline Factory) and remediation `CodeRuns`.
/// Loaded from `heal-config.json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HealerConfig {
    /// Server config for the healer server's inline Factory usage (alert analysis/triage)
    #[serde(default)]
    server: ServerConfig,
    /// `CodeRun` config for spawning remediation agents
    coderun: CodeRunConfig,
}

/// Server configuration for the healer server's inline Factory usage.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerConfig {
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_auto_level")]
    auto_level: String,
    #[serde(default = "default_output_format")]
    output_format: String,
    /// Remote tools for the server (future: pass to droid exec via MCP config)
    #[serde(default = "default_server_remote_tools")]
    #[allow(dead_code)]
    remote_tools: Vec<String>,
    #[serde(default)]
    memory: MemoryConfig,
}

fn default_server_remote_tools() -> Vec<String> {
    vec![
        "mcp_tools_kubernetes_*".to_string(),
        "mcp_tools_argocd_*".to_string(),
        "mcp_tools_grafana_*".to_string(),
        "mcp_tools_openmemory_*".to_string(),
    ]
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            auto_level: default_auto_level(),
            output_format: default_output_format(),
            remote_tools: default_server_remote_tools(),
            memory: MemoryConfig::default(),
        }
    }
}

fn default_model() -> String {
    "claude-opus-4-5-20251101".to_string()
}

fn default_auto_level() -> String {
    "high".to_string()
}

fn default_output_format() -> String {
    "text".to_string()
}

/// Memory configuration for `OpenMemory` integration.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemoryConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_agent_name")]
    agent_name: String,
    #[serde(default = "default_session_prefix")]
    session_prefix: String,
    #[serde(default)]
    retrieve_on_start: bool,
    #[serde(default)]
    persist_on_complete: bool,
}

fn default_agent_name() -> String {
    "healer".to_string()
}

fn default_session_prefix() -> String {
    "heal-session".to_string()
}

/// Build a memory-augmented prompt that includes session retrieval and persistence instructions.
fn build_memory_augmented_prompt(
    original_prompt: &str,
    alert_id: &str,
    pod_name: &str,
    memory_config: &MemoryConfig,
) -> String {
    use std::fmt::Write;

    let mut augmented = String::new();

    // Add memory retrieval instructions at the start
    if memory_config.retrieve_on_start {
        let _ = write!(
            augmented,
            r#"## 🧠 Memory Context (OpenMemory Integration)

Before starting, query OpenMemory for relevant context:

1. **Retrieve similar past alerts:**
   Use `openmemory_query` with query: "healer alert {alert_id} {pod_name} remediation"

2. **Check for known solutions:**
   Use `openmemory_query` with query: "solution fix {alert_id} platform issue"

3. **Review session history:**
   Use `openmemory_list` filtered by agent="{agent_name}" to see recent sessions

If relevant memories are found, use them to inform your analysis. Reinforce helpful memories with `openmemory_reinforce`.

---

"#,
            alert_id = alert_id,
            pod_name = pod_name,
            agent_name = memory_config.agent_name,
        );
    }

    // Add the original prompt
    augmented.push_str(original_prompt);

    // Add memory persistence instructions at the end
    if memory_config.persist_on_complete {
        let _ = write!(
            augmented,
            r#"

---

## 🧠 Memory Persistence

After completing your analysis, store key insights for future sessions:

1. **Store the alert pattern:**
   Use `openmemory_store` with content describing:
   - Alert type: {alert_id}
   - Root cause identified
   - Solution applied or recommended
   - Metadata: {{"agent": "{agent_name}", "alert_type": "{alert_id}", "pod": "{pod_name}", "session": "{session_prefix}-{alert_id}"}}

2. **Store any new patterns discovered:**
   If you identified a recurring issue or new failure mode, store it for future reference.

3. **Reinforce successful solutions:**
   If a memory-based solution worked, use `openmemory_reinforce` to increase its salience.
"#,
            alert_id = alert_id,
            pod_name = pod_name,
            agent_name = memory_config.agent_name,
            session_prefix = memory_config.session_prefix,
        );
    }

    augmented
}

/// `CodeRun` configuration matching the CRD spec fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeRunConfig {
    namespace: String,
    github_app: String,
    model: String,
    repository_url: String,
    docs_repository_url: String,
    #[serde(default)]
    docs_project_directory: String,
    #[serde(default = "default_docs_branch")]
    docs_branch: String,
    working_directory: String,
    service: String,
    #[serde(default = "default_run_type")]
    run_type: String,
    #[serde(default)]
    enable_docker: bool,
    #[serde(default = "default_coderun_remote_tools")]
    remote_tools: Vec<String>,
    #[serde(default)]
    local_tools: Vec<String>,
    cli_config: CliConfig,
    /// Memory configuration for remediation agent - passed to `CodeRun` cliConfig.settings
    #[serde(default)]
    memory: MemoryConfig,
}

fn default_coderun_remote_tools() -> Vec<String> {
    vec![
        "mcp_tools_github_*".to_string(),
        "mcp_tools_kubernetes_*".to_string(),
        "mcp_tools_argocd_*".to_string(),
        "mcp_tools_cto_*".to_string(),
        "mcp_tools_context7_*".to_string(),
        "mcp_tools_firecrawl_*".to_string(),
        "mcp_tools_grafana_*".to_string(),
        "mcp_tools_openmemory_*".to_string(),
    ]
}

fn default_docs_branch() -> String {
    "main".to_string()
}

fn default_run_type() -> String {
    "implementation".to_string()
}

/// CLI configuration for the `CodeRun`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliConfig {
    cli_type: String,
    model: String,
    #[serde(default)]
    settings: CliSettings,
}

/// CLI settings including template.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliSettings {
    #[serde(default)]
    template: String,
}

impl Default for HealerConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            coderun: CodeRunConfig {
                namespace: "cto".to_string(),
                github_app: "rex".to_string(),
                model: "claude-opus-4-5-20251101".to_string(),
                repository_url: "https://github.com/5dlabs/cto".to_string(),
                docs_repository_url: "https://github.com/5dlabs/cto".to_string(),
                docs_project_directory: "docs".to_string(),
                docs_branch: "main".to_string(),
                working_directory: ".".to_string(),
                service: "healer".to_string(),
                run_type: "implementation".to_string(),
                enable_docker: true,
                remote_tools: default_coderun_remote_tools(),
                local_tools: Vec::new(),
                cli_config: CliConfig {
                    cli_type: "factory".to_string(),
                    model: "claude-opus-4-5-20251101".to_string(),
                    settings: CliSettings {
                        template: "heal/factory".to_string(),
                    },
                },
                memory: MemoryConfig {
                    enabled: true,
                    agent_name: "healer-remediation".to_string(),
                    session_prefix: "heal-remediation".to_string(),
                    retrieve_on_start: true,
                    persist_on_complete: true,
                },
            },
        }
    }
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
///
/// The `CodeRun` is created with:
/// - Name prefix: `healer-remediation-` (for controller naming detection)
/// - Label: `agents.platform/type: healer-remediation` (for controller detection)
/// - Service: `healer` (for PVC sharing with healer deployment)
///
/// This ensures the remediation pod shares the `healer-workspace` PVC with the
/// healer monitor deployment, allowing access to prompts and logs.
fn trigger_remediation(
    config: &RemediationConfig,
    failure: &FailureContext,
    task_id: &str,
    iteration: u32,
    namespace: &str,
) -> Result<String> {
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    // Use healer-remediation- prefix for controller detection and naming
    let coderun_name = format!("healer-remediation-t{task_id}-i{iteration}-{uid}");

    // Serialize failure context to JSON for the agent
    let failure_json =
        serde_json::to_string(failure).context("Failed to serialize failure context")?;

    // Convert repository to URL format (CRD expects repositoryUrl)
    let repository_url = format!("https://github.com/{}", config.repository);
    let docs_repo = config
        .docs_repository
        .as_deref()
        .unwrap_or(&config.repository);
    let docs_repository_url = format!("https://github.com/{docs_repo}");
    let docs_dir = config.docs_project_directory.as_deref().unwrap_or("docs");

    // Ensure template starts with "healer/" for controller PVC detection
    // If config template doesn't start with healer/, prepend it
    let template = if config.template.starts_with("healer/") {
        config.template.clone()
    } else {
        format!("healer/{}", config.template)
    };

    // Create CodeRun YAML manifest
    // Uses correct CRD schema: repositoryUrl, cliConfig, env as map
    // Key fields for healer PVC sharing:
    // - Label: agents.platform/type: healer-remediation
    // - Service: healer (triggers controller's is_healer detection)
    // - Template: healer/... (also triggers is_healer detection)
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
    agents.platform/type: healer-remediation
spec:
  taskId: {task_id}
  githubApp: "{agent}"
  model: "{model}"
  repositoryUrl: "{repository_url}"
  docsRepositoryUrl: "{docs_repository_url}"
  docsProjectDirectory: "{docs_dir}"
  workingDirectory: "."
  service: "healer"
  cliConfig:
    cliType: "{cli}"
    model: "{model}"
    settings:
      template: "{template}"
      watchRole: "remediation"
  env:
    REMEDIATION_MODE: "true"
    FAILURE_CONTEXT: {failure_json_escaped}
    ORIGINAL_WORKFLOW: "{workflow_name}"
    FAILURE_TYPE: "{failure_type}"
    ITERATION: "{iteration}"
    MAX_ITERATIONS: "{max_iterations}"
"#,
        coderun_name = coderun_name,
        namespace = namespace,
        task_id = task_id,
        iteration = iteration,
        agent = config.agent,
        cli = config.cli,
        model = config.model,
        repository_url = repository_url,
        docs_repository_url = docs_repository_url,
        docs_dir = docs_dir,
        template = template,
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

    let output = child
        .wait_with_output()
        .context("Failed to wait for kubectl")?;

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

/// Create a Monitor `CodeRun` to start/continue the E2E watch loop
///
/// Returns the name of the created `CodeRun`
fn create_monitor_coderun(
    config: &MonitorConfig,
    play_config: &PlayConfig,
    iteration: u32,
    namespace: &str,
) -> Result<String> {
    ensure_kube_context(&config.cluster_context)?;

    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let coderun_name = format!("e2e-monitor-i{iteration}-{uid}");

    let repository = &play_config.repository;
    let repository_url = format!("https://github.com/{repository}");
    let service = play_config
        .service
        .as_deref()
        .unwrap_or("cto-parallel-test");

    // Get docs repository from play config
    let docs_repository = play_config.docs_repository.as_deref().unwrap_or(repository);
    let docs_repository_url = format!("https://github.com/{docs_repository}");
    let docs_project_directory = play_config
        .docs_project_directory
        .as_deref()
        .unwrap_or("docs");
    let working_directory = play_config.working_directory.as_deref().unwrap_or(".");

    // Create CodeRun YAML manifest for the monitor agent
    // Uses correct CRD schema: repositoryUrl, cliConfig, env as map
    let coderun_yaml = format!(
        r#"apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: {coderun_name}
  namespace: {namespace}
  labels:
    watch-role: monitor
    iteration: "{iteration}"
    agents.platform/type: e2e-monitor
spec:
  taskId: 1
  githubApp: "{agent}"
  model: "{model}"
  repositoryUrl: "{repository_url}"
  docsRepositoryUrl: "{docs_repository_url}"
  docsProjectDirectory: "{docs_project_directory}"
  workingDirectory: "{working_directory}"
  service: "{service}"
  cliConfig:
    cliType: "{cli}"
    model: "{model}"
    settings:
      template: "{template}"
      watchRole: "monitor"
  env:
    WATCH_MODE: "monitor"
    ITERATION: "{iteration}"
    MAX_ITERATIONS: "{max_iterations}"
    TARGET_REPOSITORY: "{repository}"
"#,
        coderun_name = coderun_name,
        namespace = namespace,
        iteration = iteration,
        agent = config.agent,
        cli = config.cli,
        model = config.model,
        repository = repository,
        repository_url = repository_url,
        docs_repository_url = docs_repository_url,
        docs_project_directory = docs_project_directory,
        working_directory = working_directory,
        service = service,
        template = config.template,
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

    let output = child
        .wait_with_output()
        .context("Failed to wait for kubectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to create monitor CodeRun: {stderr}"
        ));
    }

    println!(
        "{}",
        format!("Created monitor CodeRun: {coderun_name}").green()
    );

    Ok(coderun_name)
}

fn ensure_kube_context(expected: &str) -> Result<()> {
    let output = Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .context("Failed to read current kubectl context. Is kubectl installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Unable to determine kubectl context: {stderr}"
        ));
    }

    let current = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if current != expected {
        return Err(anyhow::anyhow!(
            "Monitor must be created from context '{expected}', but current context is '{current}'.\n\
             Run `kubectl config use-context {expected}` before starting the monitor."
        ));
    }

    Ok(())
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
            .args(["get", "application", app_name, "-n", "argocd", "-o", "json"])
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

#[derive(Debug, Clone)]
struct ContainerExitInfo {
    container_name: String,
    exit_code: Option<i32>,
    reason: Option<String>,
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
    Sensor,
    Pod,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workflow => write!(f, "workflow"),
            Self::CodeRun => write!(f, "coderun"),
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
    namespace: &'a str,
    docs_repository: &'a str,
    docs_project_directory: &'a str,
    /// Full CTO config for resolving agent configurations
    cto_config: &'a CtoConfig,
}

/// Resolved agent parameters for workflow submission
struct ResolvedAgent {
    github_app: String,
    cli: String,
    model: String,
    tools: String,
    model_rotation: String,
    max_retries: Option<u32>,
}

/// Helper to resolve agent configuration from config
fn resolve_agent_config(
    agent_name: &str,
    agents: &std::collections::HashMap<String, AgentConfig>,
    default_cli: &str,
    default_model: &str,
) -> ResolvedAgent {
    // Find agent by github_app name (e.g., "5DLabs-Rex" -> looks for agent with that githubApp)
    let agent_cfg = agents.values().find(|a| a.github_app == agent_name);

    if let Some(cfg) = agent_cfg {
        let cli = if cfg.cli.is_empty() {
            default_cli.to_string()
        } else {
            cfg.cli.clone()
        };
        let model = if cfg.model.is_empty() {
            default_model.to_string()
        } else {
            cfg.model.clone()
        };
        let tools = cfg
            .tools
            .as_ref()
            .and_then(|t| serde_json::to_string(t).ok())
            .unwrap_or_else(|| r#"{"remote":[],"localServers":{}}"#.to_string());
        let model_rotation = cfg
            .model_rotation
            .as_ref()
            .and_then(|mr| {
                if mr.enabled && !mr.models.is_empty() {
                    serde_json::to_string(&mr.models).ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "[]".to_string());

        ResolvedAgent {
            github_app: cfg.github_app.clone(),
            cli,
            model,
            tools,
            model_rotation,
            max_retries: cfg.max_retries,
        }
    } else {
        // Agent not found in config, use defaults
        ResolvedAgent {
            github_app: agent_name.to_string(),
            cli: default_cli.to_string(),
            model: default_model.to_string(),
            tools: r#"{"remote":[],"localServers":{}}"#.to_string(),
            model_rotation: "[]".to_string(),
            max_retries: None,
        }
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing if verbose
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("healer=debug")
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
            config,
            task_id,
            repository,
            service,
            docs_repository,
            docs_project_directory,
            run_type,
        } => {
            // Load config file for workflow parameters
            let config_content = std::fs::read_to_string(&config)
                .with_context(|| format!("Failed to read config file: {config}"))?;
            let cto_config: CtoConfig = serde_json::from_str(&config_content)
                .with_context(|| format!("Failed to parse config file: {config}"))?;

            // Use CLI overrides or fall back to config values
            let repo = repository
                .as_deref()
                .unwrap_or(&cto_config.defaults.play.repository);
            let svc = service
                .as_deref()
                .or(cto_config.defaults.play.service.as_deref())
                .unwrap_or("cto-parallel-test");
            let docs_repo = docs_repository
                .as_deref()
                .or(cto_config.defaults.play.docs_repository.as_deref())
                .unwrap_or(repo);
            let docs_dir = docs_project_directory
                .as_deref()
                .or(cto_config.defaults.play.docs_project_directory.as_deref())
                .unwrap_or("docs");

            let workflow_config = RunWorkflowConfig {
                task_id: &task_id,
                repository: repo,
                service: svc,
                run_type: &run_type,
                namespace: &cli.namespace,
                docs_repository: docs_repo,
                docs_project_directory: docs_dir,
                cto_config: &cto_config,
            };
            let result = run_workflow(&workflow_config)?;
            output_result(&result, cli.format)?;
        }
        Commands::Start { config } => {
            // Start the E2E self-healing loop by creating a Monitor CodeRun
            println!("{}", "Starting E2E self-healing loop...".cyan().bold());

            // Load config
            let config_content = std::fs::read_to_string(&config)
                .with_context(|| format!("Failed to read config file: {config}"))?;
            let cto_config: CtoConfig = serde_json::from_str(&config_content)
                .with_context(|| format!("Failed to parse config file: {config}"))?;

            // Get monitor config (required for start)
            let monitor_config = cto_config.defaults.monitor.ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing 'monitor' section in config. Add monitor config to cto-config.json"
                )
            })?;

            // Create Monitor CodeRun and exit
            let coderun_name = create_monitor_coderun(
                &monitor_config,
                &cto_config.defaults.play,
                1, // Always start at iteration 1
                &cli.namespace,
            )?;

            println!(
                "{}",
                format!("Monitor CodeRun created: {coderun_name}").green()
            );
            println!(
                "{}",
                "E2E loop is now running autonomously in the cluster.".cyan()
            );
            println!(
                "{}",
                "Use 'kubectl get coderuns -n cto -w' to watch progress.".dimmed()
            );
        }
        Commands::Monitor {
            config,
            iteration,
            max_iterations,
            repository,
            service,
            task_id,
            docs_repository,
            docs_project_directory,
            criteria,
        } => {
            // Load config file for workflow parameters
            let config_content = std::fs::read_to_string(&config)
                .with_context(|| format!("Failed to read config file: {config}"))?;
            let cto_config: CtoConfig = serde_json::from_str(&config_content)
                .with_context(|| format!("Failed to parse config file: {config}"))?;

            // Use CLI overrides or fall back to config values
            let repo = repository
                .as_deref()
                .unwrap_or(&cto_config.defaults.play.repository)
                .to_string();
            let svc = service
                .as_deref()
                .or(cto_config.defaults.play.service.as_deref())
                .unwrap_or("cto-parallel-test")
                .to_string();
            let docs_dir = docs_project_directory
                .as_deref()
                .or(cto_config.defaults.play.docs_project_directory.as_deref())
                .unwrap_or("docs")
                .to_string();

            // This runs inside the Monitor pod
            let monitor_params = MonitorParams {
                iteration,
                max_iterations,
                repository: repo,
                service: svc,
                task_id,
                docs_repository,
                docs_project_directory: docs_dir,
                criteria_path: criteria,
                namespace: cli.namespace.clone(),
                cto_config,
            };
            run_monitor_loop(&monitor_params).await?;
        }
        Commands::Remediate {
            iteration,
            issue_file,
            config,
        } => {
            // This runs inside the Remediation pod
            run_remediation_loop(iteration, &issue_file, &config, &cli.namespace)?;
        }
        Commands::Memory { action } => {
            handle_memory_command(action).await?;
        }
        Commands::AlertWatch {
            namespace,
            prompts_dir,
            dry_run,
        } => {
            // Default enable_docker to true (matches CRD default)
            run_alert_watch(&namespace, &prompts_dir, dry_run, true).await?;
        }
        Commands::TestAlert {
            alert,
            pod_name,
            task_id,
            agent,
            prompts_dir,
            dry_run,
        } => {
            // Default enable_docker to true for testing (matches CRD default)
            test_alert_flow(
                &alert,
                &pod_name,
                &task_id,
                &agent,
                &prompts_dir,
                dry_run,
                true,
            )
            .await?;
        }
        Commands::SpawnRemediation {
            alert,
            task_id,
            target_pod,
            issue_number,
            issue_file,
            config,
        } => {
            spawn_remediation_agent(
                &alert,
                &task_id,
                target_pod.as_deref(),
                issue_number,
                issue_file.as_deref(),
                &config,
            )?;
        }
        Commands::FetchLogs {
            pod_name,
            namespace,
            output_dir,
            tail,
        } => {
            fetch_pod_logs(&pod_name, &namespace, &output_dir, tail)?;
        }
        Commands::Play { action } => {
            handle_play_command(action, &cli.namespace)?;
        }
        Commands::Insights { action } => {
            handle_insights_command(action)?;
        }
        Commands::Server {
            addr,
            repository,
            namespace,
            config: config_path,
        } => {
            run_server_command(&addr, &repository, &namespace, config_path.as_deref()).await?;
        }
        Commands::ScanLogs {
            window,
            namespaces,
            error_threshold,
            warn_threshold,
            output,
            show_candidates,
        } => {
            run_scan_logs_command(
                &window,
                &namespaces,
                error_threshold,
                warn_threshold,
                &output,
                show_candidates,
            )
            .await?;
        }
        Commands::RemediateFromScan {
            config,
            dry_run,
            max_coderuns,
        } => {
            run_remediate_from_scan(&config, dry_run, max_coderuns)?;
        }
        Commands::ReconcileIssues {
            repository,
            namespace,
            labels,
            dry_run,
            max_issues,
            output,
        } => {
            run_reconcile_issues(
                &repository,
                &namespace,
                &labels,
                dry_run,
                max_issues,
                &output,
            )?;
        }
        Commands::Sensor { action } => match action {
            SensorCommands::GithubActions {
                repositories,
                poll_interval,
                lookback_mins,
                create_issues,
                issue_labels,
                branches,
                excluded_workflows,
                namespace,
                once,
                config,
            } => {
                run_github_actions_sensor(
                    &repositories,
                    poll_interval,
                    lookback_mins,
                    create_issues,
                    &issue_labels,
                    &branches,
                    &excluded_workflows,
                    &namespace,
                    once,
                    config.as_deref(),
                )
                .await?;
            }
        },
        Commands::PlayMonitor {
            namespace,
            poll_interval,
            repository,
            min_severity,
            no_issues,
            max_issues,
            issue_cooldown,
            log_window,
            once,
        } => {
            run_play_monitor(
                &namespace,
                poll_interval,
                repository.as_deref(),
                &min_severity,
                !no_issues,
                max_issues,
                issue_cooldown,
                log_window,
                once,
            )
            .await?;
        }
    }

    Ok(())
}

// =============================================================================
// GitHub Actions Sensor
// =============================================================================

/// Run the GitHub Actions sensor to monitor for workflow failures.
async fn run_github_actions_sensor(
    repositories: &str,
    poll_interval: u64,
    lookback_mins: u64,
    create_issues: bool,
    issue_labels: &str,
    branches: &str,
    excluded_workflows: &str,
    namespace: &str,
    once: bool,
    config_path: Option<&str>,
) -> Result<()> {
    use sensors::{GitHubActionsSensor, SensorConfig};

    info!("Starting GitHub Actions sensor");

    // Parse repositories
    let repos: Vec<String> = repositories
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if repos.is_empty() {
        anyhow::bail!("No repositories specified");
    }

    // Parse labels
    let labels: Vec<String> = issue_labels
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Parse branches filter
    let branch_filter: Vec<String> = branches
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Parse excluded workflows
    let excluded: Vec<String> = excluded_workflows
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Build sensor config
    let sensor_config = SensorConfig {
        repositories: repos.clone(),
        poll_interval_secs: poll_interval,
        lookback_mins,
        create_issues,
        issue_labels: labels,
        branches: branch_filter,
        excluded_workflows: excluded,
        max_per_poll: 10,
        namespace: namespace.to_string(),
    };

    // Load remediation config if provided
    let remediation_config = if let Some(path) = config_path {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {path}"))?;
        serde_json::from_str(&content).with_context(|| "Failed to parse remediation config")?
    } else {
        ci::types::RemediationConfig::default()
    };

    info!("Monitoring {} repositories: {:?}", repos.len(), repos);
    info!(
        "Poll interval: {}s, lookback: {}m, create issues: {}",
        poll_interval, lookback_mins, create_issues
    );

    let mut sensor = GitHubActionsSensor::new(sensor_config, remediation_config);

    if once {
        // Run once and exit
        info!("Running single poll cycle...");
        let failures = sensor.poll_once()?;
        info!("Processed {} failure(s)", failures.len());
        for failure in &failures {
            println!(
                "  - {} ({}) @ {} [{}]",
                failure.workflow_name, failure.run_id, failure.branch, failure.html_url
            );
        }
    } else {
        // Run continuous loop
        info!("Starting continuous monitoring loop (Ctrl+C to stop)");
        sensor.run().await?;
    }

    Ok(())
}

// =============================================================================
// Play Monitor - Real-time log analysis for running plays
// =============================================================================

/// Run the play monitor to watch for anomalies in running plays.
#[allow(clippy::too_many_arguments)]
async fn run_play_monitor(
    namespace: &str,
    poll_interval: u64,
    repository: Option<&str>,
    min_severity: &str,
    auto_create_issues: bool,
    max_issues: usize,
    issue_cooldown: i64,
    log_window: i64,
    once: bool,
) -> Result<()> {
    use play::{MonitorConfig, PlayMonitor};

    info!("Starting play monitor");
    info!(
        "Namespace: {}, poll interval: {}s, min severity: {}",
        namespace, poll_interval, min_severity
    );
    if let Some(repo) = repository {
        info!(
            "GitHub repository: {}, auto-create issues: {}",
            repo, auto_create_issues
        );
    }

    let config = MonitorConfig {
        namespace: namespace.to_string(),
        poll_interval_secs: poll_interval,
        min_severity: min_severity.to_string(),
        repository: repository.map(String::from),
        auto_create_issues,
        max_issues_per_play: max_issues,
        issue_cooldown_mins: issue_cooldown,
        log_window_mins: log_window,
    };

    let mut monitor = PlayMonitor::new(config);

    // Set up event channel for logging
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    monitor.set_event_sender(tx);

    // Spawn event logger
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event.event_type {
                play::monitor::MonitorEventType::PlayDetected => {
                    info!(
                        play_id = ?event.play_id,
                        "🎬 New play detected"
                    );
                }
                play::monitor::MonitorEventType::PlayCompleted => {
                    info!(
                        play_id = ?event.play_id,
                        details = %event.details,
                        "✅ Play completed"
                    );
                }
                play::monitor::MonitorEventType::AnomalyDetected => {
                    warn!(
                        play_id = ?event.play_id,
                        details = %event.details,
                        "⚠️ Anomaly detected"
                    );
                }
                play::monitor::MonitorEventType::IssueCreated => {
                    info!(
                        play_id = ?event.play_id,
                        details = %event.details,
                        "📝 GitHub issue created"
                    );
                }
                play::monitor::MonitorEventType::SuccessDetected => {
                    debug!(
                        play_id = ?event.play_id,
                        details = %event.details,
                        "✓ Success pattern detected"
                    );
                }
                play::monitor::MonitorEventType::Error => {
                    error!(
                        play_id = ?event.play_id,
                        details = %event.details,
                        "❌ Monitor error"
                    );
                }
                _ => {
                    debug!(
                        event_type = ?event.event_type,
                        play_id = ?event.play_id,
                        "Monitor event"
                    );
                }
            }
        }
    });

    if once {
        // Run once and exit
        info!("Running single poll cycle...");
        monitor.poll_once().await?;
        let status = monitor.get_status();
        info!(
            "Status: {} active plays, {} CodeRuns, {} anomalies, {} issues",
            status.active_plays, status.total_coderuns, status.total_anomalies, status.total_issues
        );
    } else {
        // Run continuous loop
        info!("Starting continuous monitoring loop (Ctrl+C to stop)");
        monitor.run().await?;
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
            let pod_name = node["podName"]
                .as_str()
                .or_else(|| node["id"].as_str())
                .or_else(|| node["name"].as_str())
                .map(ToString::to_string);

            let step = WorkflowStep {
                id: id.clone(),
                name: node["displayName"]
                    .as_str()
                    .or_else(|| node["name"].as_str())
                    .unwrap_or(id)
                    .to_string(),
                step_type: node["type"].as_str().unwrap_or("Unknown").to_string(),
                phase: node["phase"].as_str().unwrap_or("Unknown").to_string(),
                pod_name,
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

        println!("{}", format!("Workflow submitted: {workflow_name}").green());

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
                        format!(
                            "Workflow failed, triggering remediation (iteration {iteration})..."
                        )
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
                        println!("{}", "Waiting for PR merge and ArgoCD sync...".cyan());

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

                        // Still need to wait for ArgoCD to sync the inline fix
                        println!("{}", "Waiting for ArgoCD to sync inline fix...".cyan());

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

fn capture_terminated_agent_logs(
    status: &WorkflowStatus,
    namespace: &str,
    logs_dir: &str,
    archived_pods: &mut HashSet<String>,
) -> Result<Vec<(String, ContainerExitInfo)>> {
    let mut findings = Vec::new();
    for step in &status.steps {
        if step.step_type != "Pod" || step.phase != "Running" {
            continue;
        }

        let Some(pod_name) = step.pod_name.as_ref() else {
            continue;
        };

        if archived_pods.contains(pod_name) {
            continue;
        }

        if let Some(exit) = check_agent_container_exit(pod_name, namespace) {
            println!(
                "{}",
                format!(
                    "Detected terminated agent container '{}' (exit {:?}, reason {:?}) in pod {}. Archiving logs...",
                    exit.container_name, exit.exit_code, exit.reason, pod_name
                )
                .yellow()
            );

            let logs =
                get_step_logs(pod_name, namespace, 10_000).context("Failed to read pod logs")?;
            let safe_name = step.name.replace(' ', "_");
            let file_path = format!("{logs_dir}/{}_{}.log", safe_name, exit.container_name);
            std::fs::write(&file_path, logs)
                .with_context(|| format!("Failed to write logs to {file_path}"))?;
            archived_pods.insert(pod_name.clone());
            findings.push((step.name.clone(), exit));
        }
    }

    Ok(findings)
}

fn check_agent_container_exit(pod_name: &str, namespace: &str) -> Option<ContainerExitInfo> {
    let output = Command::new("kubectl")
        .args(["get", "pod", pod_name, "-n", namespace, "-o", "json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let pod: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let statuses = pod["status"]["containerStatuses"].as_array()?;

    let mut agent_exit: Option<ContainerExitInfo> = None;
    let mut sidecar_running = false;

    for status in statuses {
        let name = status["name"].as_str().unwrap_or("");
        let state = &status["state"];

        if let Some(term) = state["terminated"].as_object() {
            #[allow(clippy::cast_possible_truncation)] // exit codes are always small i32 values
            let exit_code = term
                .get("exitCode")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32);
            let reason = term
                .get("reason")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            if !name.contains("docker") {
                agent_exit = Some(ContainerExitInfo {
                    container_name: name.to_string(),
                    exit_code,
                    reason,
                });
            }
        } else if state["running"].is_object()
            && (name.contains("docker") || name == "docker-daemon")
        {
            sidecar_running = true;
        }
    }

    if let Some(exit) = agent_exit {
        if exit.exit_code.unwrap_or(0) != 0 || sidecar_running {
            return Some(exit);
        }
    }

    None
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

        // Delete test ConfigMaps (play-*, test-*, coderun-*, remediation-*)
        for pattern in &["play-", "test-", "coderun-", "remediation-"] {
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
    let temp_dir = std::env::temp_dir().join(format!("heal-{repo}"));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    // Create README
    let readme_content = format!(
        "# {repo}\n\nE2E test repository for CTO platform.\n\nThis repo is managed by heal."
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

/// Run/submit a play workflow via Argo CLI (reads all parameters from config)
#[allow(clippy::too_many_lines)]
fn run_workflow(config: &RunWorkflowConfig<'_>) -> Result<RunResponse> {
    println!(
        "{}",
        format!("Submitting play workflow for task {}...", config.task_id).cyan()
    );

    let play_config = &config.cto_config.defaults.play;
    let agents = &config.cto_config.agents;

    // Default CLI and model from play config
    let default_cli = play_config.cli.as_deref().unwrap_or("factory");
    let default_model = play_config
        .model
        .as_deref()
        .unwrap_or("claude-sonnet-4-20250514");

    // Resolve all 5 agent stages from config
    let impl_agent = resolve_agent_config(
        &play_config.implementation_agent,
        agents,
        default_cli,
        default_model,
    );

    let frontend_agent = if let Some(ref agent_name) = play_config.frontend_agent {
        resolve_agent_config(agent_name, agents, default_cli, default_model)
    } else {
        // Default frontend agent if not configured
        resolve_agent_config("5DLabs-Blaze", agents, default_cli, default_model)
    };

    let quality_agent =
        resolve_agent_config(&play_config.quality_agent, agents, "claude", default_model);

    let security_agent = if let Some(ref agent_name) = play_config.security_agent {
        resolve_agent_config(agent_name, agents, "cursor", default_model)
    } else {
        // Default security agent if not configured
        resolve_agent_config("5DLabs-Cipher", agents, "cursor", default_model)
    };

    let testing_agent =
        resolve_agent_config(&play_config.testing_agent, agents, "claude", default_model);

    // Get max retries from config with fallbacks
    let default_retries = play_config.max_retries.unwrap_or(10);
    let impl_max_retries = impl_agent
        .max_retries
        .or(play_config.implementation_max_retries)
        .unwrap_or(default_retries);
    let frontend_max_retries = frontend_agent
        .max_retries
        .or(play_config.frontend_max_retries)
        .unwrap_or(default_retries);
    let quality_max_retries = quality_agent
        .max_retries
        .or(play_config.quality_max_retries)
        .unwrap_or(5);
    let security_max_retries = security_agent
        .max_retries
        .or(play_config.security_max_retries)
        .unwrap_or(2);
    let testing_max_retries = testing_agent
        .max_retries
        .or(play_config.testing_max_retries)
        .unwrap_or(5);

    // Get other settings from config
    let auto_merge = play_config.auto_merge.unwrap_or(false);
    let parallel_execution = play_config.parallel_execution.unwrap_or(false);

    // Select workflow template based on parallel_execution
    let workflow_template = if parallel_execution {
        println!("{}", "🚀 Using parallel execution mode".dimmed());
        "workflowtemplate/play-project-workflow-template"
    } else {
        println!("{}", "🔄 Using sequential execution mode".dimmed());
        "workflowtemplate/play-workflow-template"
    };

    // Generate workflow name
    let agent_short = impl_agent
        .github_app
        .strip_prefix("5DLabs-")
        .unwrap_or(&impl_agent.github_app)
        .to_lowercase();
    let uid: String = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let workflow_name = format!(
        "play-{}-t{}-{}-{}-{}",
        config.run_type, config.task_id, agent_short, impl_agent.cli, uid
    );

    // Build all parameters (matching MCP server format)
    let params: Vec<String> = vec![
        format!("task-id={}", config.task_id),
        format!("repository={}", config.repository),
        format!("service={}", config.service),
        format!("docs-repository={}", config.docs_repository),
        format!("docs-project-directory={}", config.docs_project_directory),
        // Implementation agent
        format!("implementation-agent={}", impl_agent.github_app),
        format!("implementation-cli={}", impl_agent.cli),
        format!("implementation-model={}", impl_agent.model),
        format!("implementation-tools={}", impl_agent.tools),
        format!(
            "implementation-model-rotation={}",
            impl_agent.model_rotation
        ),
        // Frontend agent
        format!("frontend-agent={}", frontend_agent.github_app),
        format!("frontend-cli={}", frontend_agent.cli),
        format!("frontend-model={}", frontend_agent.model),
        format!("frontend-tools={}", frontend_agent.tools),
        format!("frontend-model-rotation={}", frontend_agent.model_rotation),
        // Quality agent
        format!("quality-agent={}", quality_agent.github_app),
        format!("quality-cli={}", quality_agent.cli),
        format!("quality-model={}", quality_agent.model),
        format!("quality-tools={}", quality_agent.tools),
        format!("quality-model-rotation={}", quality_agent.model_rotation),
        // Security agent
        format!("security-agent={}", security_agent.github_app),
        format!("security-cli={}", security_agent.cli),
        format!("security-model={}", security_agent.model),
        format!("security-tools={}", security_agent.tools),
        format!("security-model-rotation={}", security_agent.model_rotation),
        // Testing agent
        format!("testing-agent={}", testing_agent.github_app),
        format!("testing-cli={}", testing_agent.cli),
        format!("testing-model={}", testing_agent.model),
        format!("testing-tools={}", testing_agent.tools),
        format!("testing-model-rotation={}", testing_agent.model_rotation),
        // Max retries
        format!("implementation-max-retries={impl_max_retries}"),
        format!("frontend-max-retries={frontend_max_retries}"),
        format!("quality-max-retries={quality_max_retries}"),
        format!("security-max-retries={security_max_retries}"),
        format!("testing-max-retries={testing_max_retries}"),
        format!("opencode-max-retries={default_retries}"),
        // Other settings
        format!("auto-merge={auto_merge}"),
        format!("parallel-execution={parallel_execution}"),
        "final-task=false".to_string(),
        "task-requirements=".to_string(),
    ];

    // Build argo command
    let mut cmd = Command::new("argo");

    // Support ARGO_KUBECONFIG for hybrid cluster setups (Kind local + Talos remote)
    if let Ok(kubeconfig) = std::env::var("ARGO_KUBECONFIG") {
        cmd.arg("--kubeconfig").arg(&kubeconfig);
    }

    // Base args
    cmd.args([
        "submit",
        "--from",
        workflow_template,
        "-n",
        config.namespace,
    ]);

    // Add labels for workflow tracking (matches MCP server behavior)
    let repo_label = format!("repository={}", config.repository.replace('/', "-"));
    cmd.args(["-l", &repo_label]);
    cmd.args(["-l", "workflow-type=play"]);
    cmd.args(["-l", &format!("task-id={}", config.task_id)]);

    // Add all parameters
    for param in &params {
        cmd.args(["-p", param]);
    }

    // Add workflow name and output format
    cmd.args(["--name", &workflow_name, "-o", "json"]);

    let output = cmd
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

// =============================================================================
// E2E Self-Healing Loop Functions
// =============================================================================

/// Run the monitor loop (executed inside Monitor pod)
///
/// This function:
/// 1. Resets environment if iteration > 1
/// 2. Submits Play workflow for the specified task
/// 3. Waits for workflow completion
/// 4. Downloads and analyzes all logs
/// 5. Evaluates against acceptance criteria
/// 6. On success: exits 0 (ends loop)
/// 7. On failure: writes issue to PVC, creates Remediation `CodeRun`, exits 1
#[allow(clippy::too_many_lines)]
async fn run_monitor_loop(params: &MonitorParams) -> Result<()> {
    println!(
        "{}",
        format!(
            "=== Monitor Loop Iteration {} / {} ===",
            params.iteration, params.max_iterations
        )
        .cyan()
        .bold()
    );

    println!(
        "{}",
        format!(
            "Repository: {} | Service: {} | Task: {}",
            params.repository, params.service, params.task_id
        )
        .dimmed()
    );

    let logs_dir = "/workspace/watch/logs";
    std::fs::create_dir_all(logs_dir).ok();
    let mut archived_pods: HashSet<String> = HashSet::new();
    let mut detected_failures: Vec<String> = Vec::new();

    // Step 1: Reset environment if re-running (iteration > 1)
    if params.iteration > 1 {
        println!(
            "{}",
            "Resetting environment from previous iteration...".yellow()
        );
        // Extract org/repo from repository string
        let parts: Vec<&str> = params.repository.split('/').collect();
        if parts.len() == 2 {
            reset_environment(&params.namespace, parts[0], parts[1], false, false, true)?;
        }
    }

    // Step 2: Submit Play workflow
    println!(
        "{}",
        format!("Submitting Play workflow for task {}...", params.task_id).cyan()
    );
    // Use docs_repository if provided, otherwise fall back to repository
    let docs_repo = params
        .docs_repository
        .as_deref()
        .unwrap_or(&params.repository);

    let workflow_config = RunWorkflowConfig {
        task_id: &params.task_id,
        repository: &params.repository,
        service: &params.service,
        run_type: "e2e-monitor",
        namespace: &params.namespace,
        docs_repository: docs_repo,
        docs_project_directory: &params.docs_project_directory,
        cto_config: &params.cto_config,
    };

    let run_result = run_workflow(&workflow_config)?;
    let workflow_name = run_result.workflow_name.ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to submit workflow: {}",
            run_result.error.unwrap_or_default()
        )
    })?;

    println!("{}", format!("Workflow submitted: {workflow_name}").green());

    // Step 3: Wait for workflow completion
    println!("{}", "Waiting for workflow completion...".cyan());
    let mut last_phase = String::new();
    loop {
        let status = get_workflow_status(&workflow_name, &params.namespace)?;
        let new_failures = capture_terminated_agent_logs(
            &status,
            &params.namespace,
            logs_dir,
            &mut archived_pods,
        )?;
        if !new_failures.is_empty() {
            for (step_name, exit_info) in new_failures {
                detected_failures.push(format!(
                    "Stage {} failed early: container '{}' exited with code {:?} (reason: {:?})",
                    step_name, exit_info.container_name, exit_info.exit_code, exit_info.reason
                ));
            }
            println!(
                "{}",
                "Detected failed agent container while workflow still running. Triggering remediation."
                    .red()
                    .bold()
            );
            break;
        }

        if status.phase != last_phase {
            println!("{}", format!("Workflow phase: {}", status.phase).dimmed());
            last_phase.clone_from(&status.phase);
        }

        match status.phase.as_str() {
            "Succeeded" => {
                println!("{}", "Workflow succeeded!".green().bold());
                break;
            }
            "Failed" | "Error" => {
                println!(
                    "{}",
                    format!("Workflow failed: {}", status.message.unwrap_or_default()).red()
                );
                // Continue to evaluation - we'll record the failure
                break;
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        }
    }

    // Step 4: Download logs from all stages
    println!("{}", "Downloading logs from all stages...".cyan());

    // Get logs using argo CLI
    let logs_output = Command::new("argo")
        .args(["logs", &workflow_name, "-n", &params.namespace])
        .output()
        .context("Failed to get workflow logs")?;

    let all_logs = String::from_utf8_lossy(&logs_output.stdout);
    std::fs::write(format!("{logs_dir}/workflow-logs.txt"), all_logs.as_ref()).ok();

    // Step 5: Evaluate against acceptance criteria
    println!(
        "{}",
        "Evaluating results against acceptance criteria...".cyan()
    );
    let final_status = get_workflow_status(&workflow_name, &params.namespace)?;

    let mut issues: Vec<String> = detected_failures;

    // Check workflow phase
    if final_status.phase != "Succeeded" {
        issues.push(format!(
            "Workflow did not succeed (phase: {}, message: {})",
            final_status.phase,
            final_status.message.unwrap_or_default()
        ));
    }

    // Check for failed steps
    for step in &final_status.failed_steps {
        issues.push(format!(
            "Stage failed: {} - {}",
            step.name,
            step.message.as_deref().unwrap_or("no message")
        ));
    }

    // Check logs for critical errors
    let error_patterns = [
        "error[E",
        "FAILED",
        "panicked at",
        "fatal:",
        "OOMKilled",
        "CrashLoopBackOff",
    ];
    for pattern in &error_patterns {
        if all_logs.contains(pattern) {
            issues.push(format!("Error pattern found in logs: {pattern}"));
        }
    }

    // Step 6/7: Success or failure handling
    if issues.is_empty() {
        println!(
            "{}",
            "✅ All acceptance criteria met - E2E loop complete!"
                .green()
                .bold()
        );
        std::process::exit(0);
    }

    // Write issue report for remediation
    println!(
        "{}",
        format!("❌ Found {} issues - triggering remediation", issues.len()).red()
    );

    let issue_report = format!(
        r"# Issue Report - Iteration {}

## Summary
E2E workflow did not meet acceptance criteria.

## Issues Found
{}

## Workflow Details
- Name: {}
- Phase: {}
- Repository: {}
- Service: {}
- Task ID: {}

## Relevant Logs
See /workspace/watch/logs/workflow-logs.txt for full logs.

## Suggested Fix
Analyze the errors above and fix the underlying issues in the CTO platform.
",
        params.iteration,
        issues
            .iter()
            .map(|i| format!("- {i}"))
            .collect::<Vec<_>>()
            .join("\n"),
        workflow_name,
        final_status.phase,
        params.repository,
        params.service,
        params.task_id,
    );

    std::fs::write("/workspace/watch/current-issue.md", &issue_report)?;
    println!(
        "{}",
        "Issue report written to /workspace/watch/current-issue.md".dimmed()
    );

    // Create Remediation CodeRun using default config
    // In the future, this could be passed as params
    let remediation_config = RemediationConfig {
        repository: params.repository.clone(),
        docs_repository: params.docs_repository.clone(),
        docs_project_directory: Some(params.docs_project_directory.clone()),
        agent: "5DLabs-Rex".to_string(),
        cli: "factory".to_string(),
        model: "claude-opus-4-5-20251101".to_string(),
        max_iterations: params.max_iterations,
        template: "watch/factory".to_string(),
        sync_timeout_secs: 300,
    };

    let failure = FailureContext {
        workflow_name: workflow_name.clone(),
        failed_resource: workflow_name,
        resource_type: "workflow".to_string(),
        phase: final_status.phase,
        error_message: Some(issues.join("; ")),
        logs: Some(all_logs.chars().take(5000).collect()),
        container: None,
        exit_code: None,
        timestamp: Utc::now(),
    };

    let coderun_name = trigger_remediation(
        &remediation_config,
        &failure,
        &params.task_id,
        params.iteration,
        &params.namespace,
    )?;

    println!(
        "{}",
        format!("Remediation CodeRun created: {coderun_name}").green()
    );

    std::process::exit(1);
}

/// Run the remediation loop (executed inside Remediation pod)
///
/// This function:
/// 1. Reads issue from PVC
/// 2. Clones repo, creates branch
/// 3. (Agent fixes the issue via prompt)
/// 4. Runs validation
/// 5. Creates PR
/// 6. Waits for CI
/// 7. Checks Bugbot
/// 8. Merges PR
/// 9. Waits for `ArgoCD` sync
/// 10. Creates new Monitor `CodeRun`
fn run_remediation_loop(
    iteration: u32,
    issue_file: &str,
    config_path: &str,
    _namespace: &str,
) -> Result<()> {
    println!(
        "{}",
        format!("=== Remediation Loop Iteration {iteration} ===")
            .cyan()
            .bold()
    );

    // Load config
    let config_content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config: {config_path}"))?;
    let _config: CtoConfig = serde_json::from_str(&config_content)?;

    // Read issue file
    let issue_content = std::fs::read_to_string(issue_file)
        .with_context(|| format!("Failed to read issue file: {issue_file}"))?;

    println!("{}", "Issue to remediate:".cyan());
    println!("{}", issue_content.dimmed());

    // The actual fixing is done by the agent via its prompt.
    // This function provides the tooling and orchestration.
    // The agent will:
    // 1. Analyze the issue
    // 2. Make code changes
    // 3. Run validation via heal's helpers or shell scripts
    // 4. Create PR
    // 5. Wait for merge
    // 6. Trigger next iteration

    println!(
        "{}",
        "Remediation agent is analyzing and fixing the issue...".cyan()
    );
    println!(
        "{}",
        "The agent will use available tools to complete the fix.".dimmed()
    );

    // After agent completes its work and PR is merged, create next Monitor CodeRun
    // This is called by the agent at the end of remediation

    // For now, just print guidance - the agent handles the actual flow
    println!("{}", "Available remediation commands:".cyan().bold());
    println!("  - Run validation: ./scripts/run-validation.sh");
    println!("  - Create PR: gh pr create ...");
    println!("  - Check CI: gh pr checks ...");
    println!("  - Merge: gh pr merge --auto ...");
    println!("  - Wait for sync: argocd app wait controller --sync");

    // The agent will eventually call create_next_monitor_iteration
    // when remediation is complete and verified

    Ok(())
}

/// Create the next Monitor `CodeRun` after successful remediation
/// Called by the remediation agent after PR is merged and synced
#[allow(dead_code)]
fn create_next_monitor_iteration(config_path: &str, iteration: u32, namespace: &str) -> Result<()> {
    let config_content = std::fs::read_to_string(config_path)?;
    let config: CtoConfig = serde_json::from_str(&config_content)?;

    let monitor_config = config
        .defaults
        .monitor
        .ok_or_else(|| anyhow::anyhow!("Missing monitor config"))?;

    let coderun_name = create_monitor_coderun(
        &monitor_config,
        &config.defaults.play,
        iteration + 1,
        namespace,
    )?;

    println!(
        "{}",
        format!("Next Monitor CodeRun created: {coderun_name}").green()
    );

    Ok(())
}

// =============================================================================
// Alert System Functions
// =============================================================================

/// Message types for the alert watch event loop
#[derive(Debug)]
enum AlertWatchEvent {
    PodEvent(serde_json::Value),
    CodeRunEvent(serde_json::Value),
}

/// Verify that the healer templates directory exists and contains expected structure.
///
/// Returns an error if the templates directory is missing or incomplete.
fn verify_templates_directory(templates_dir: &str) -> Result<()> {
    use std::path::Path;

    let templates_path = Path::new(templates_dir);

    if !templates_path.exists() {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("❌ CRITICAL: Templates directory not found");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("Expected path: {templates_dir}");
        eprintln!();
        eprintln!("Templates are embedded in the Docker image at build time.");
        eprintln!("For local development, ensure HEALER_TEMPLATES_DIR points to");
        eprintln!("the templates/healer directory in the repo.");
        eprintln!();
        eprintln!("To fix:");
        eprintln!("  1. Set HEALER_TEMPLATES_DIR=templates/healer for local dev");
        eprintln!("  2. Or rebuild the healer image for production");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        anyhow::bail!("Templates directory not found: {templates_dir}");
    }

    // Check for expected subdirectories
    let expected_dirs = ["alerts", "partials", "expected", "ci"];
    let mut missing_dirs = Vec::new();

    for dir in &expected_dirs {
        let dir_path = templates_path.join(dir);
        if !dir_path.exists() {
            missing_dirs.push(*dir);
        }
    }

    if !missing_dirs.is_empty() {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("❌ CRITICAL: Templates directory structure incomplete");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("Templates path: {templates_dir}");
        eprintln!("Missing directories:");
        for dir in &missing_dirs {
            eprintln!("  - {dir}");
        }
        eprintln!();
        eprintln!("Expected structure:");
        eprintln!("  {templates_dir}/");
        eprintln!("  ├── alerts/      (alert-specific templates)");
        eprintln!("  ├── partials/    (shared template partials)");
        eprintln!("  ├── expected/    (agent expected behavior docs)");
        eprintln!("  └── ci/          (CI remediation templates)");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        anyhow::bail!(
            "Templates directory missing subdirectories: {}",
            missing_dirs.join(", ")
        );
    }

    println!(
        "{}",
        format!("  ✓ Templates directory: {templates_dir}").green()
    );
    for dir in &expected_dirs {
        println!("    ✓ {dir}/");
    }

    Ok(())
}

/// Watch for alerts and spawn Factory when detected.
#[allow(clippy::too_many_lines)]
async fn run_alert_watch(
    namespace: &str,
    prompts_dir: &str,
    dry_run: bool,
    enable_docker: bool,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command as AsyncCommand;
    use tokio::sync::mpsc;

    // Verify templates directory exists before starting
    println!("{}", "Verifying healer templates are available...".cyan());
    verify_templates_directory(prompts_dir)?;
    println!("{}", "✅ Healer templates verified".green());

    println!(
        "{}",
        format!("Starting alert watch in namespace: {namespace}").cyan()
    );
    println!("{}", format!("Templates directory: {prompts_dir}").dimmed());
    if dry_run {
        println!(
            "{}",
            "DRY RUN MODE - will detect but not spawn Factory".yellow()
        );
    }

    // Initialize notification system
    let notifier = notify::Notifier::from_env();
    if notifier.has_channels() {
        println!(
            "{}",
            format!(
                "Notifications enabled with {} channel(s)",
                notifier.channel_count()
            )
            .green()
        );
    }

    // Initialize alert registry and default context
    let registry = alerts::AlertRegistry::new();
    let github_state = github::GitHubState::default();

    // Track CodeRun timestamps for A9 alerts
    let mut coderun_tracker = alerts::CodeRunTracker::new();
    // Track which CodeRuns we've already alerted on (to avoid spam)
    let mut alerted_coderuns: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Track which pods we've already alerted on (key: "alert_id:pod_name")
    let mut alerted_pods: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Create a channel for events from both watches
    // Increased buffer to reduce chance of dropped events
    let (tx, mut rx) = mpsc::channel::<AlertWatchEvent>(500);

    // Start periodic pod poller as fallback for missed watch events
    // This catches silent failures that the watch stream might miss
    let tx_poller = tx.clone();
    let namespace_poller = namespace.to_string();
    let poller_handle = tokio::spawn(async move {
        // Wait 30s before first poll to let watch stabilize
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        loop {
            // Poll every 60 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            // Get all pods with kubectl and look for silent failures
            let output = match AsyncCommand::new("kubectl")
                .args(["get", "pods", "-n", &namespace_poller, "-o", "json"])
                .output()
                .await
            {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("⚠️  Pod poller kubectl error: {e}");
                    continue;
                }
            };

            if !output.status.success() {
                continue;
            }

            let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
                Ok(j) => j,
                Err(_) => continue,
            };

            let Some(items) = json["items"].as_array() else {
                continue;
            };

            for item in items {
                // Check for silent failure: phase=Running, container terminated with non-zero exit
                let phase = item["status"]["phase"].as_str().unwrap_or("");
                if phase != "Running" {
                    continue;
                }

                let Some(container_statuses) = item["status"]["containerStatuses"].as_array()
                else {
                    continue;
                };

                let has_failed_container = container_statuses.iter().any(|cs| {
                    if let Some(terminated) = cs["state"]["terminated"].as_object() {
                        let exit_code = terminated
                            .get("exitCode")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        exit_code != 0
                    } else {
                        false
                    }
                });

                if has_failed_container {
                    // Wrap in watch event format and send
                    let event = serde_json::json!({
                        "type": "MODIFIED",
                        "object": item
                    });
                    if tx_poller
                        .send(AlertWatchEvent::PodEvent(event))
                        .await
                        .is_err()
                    {
                        return; // Channel closed
                    }
                }
            }
        }
    });

    // Start kubectl watch for pods
    let tx_pods = tx.clone();
    let namespace_pods = namespace.to_string();
    let pod_watch_handle = tokio::spawn(async move {
        let mut child = match AsyncCommand::new("kubectl")
            .args([
                "get",
                "pods",
                "-n",
                &namespace_pods,
                "-w",
                "-o",
                "json",
                "--output-watch-events",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to start pod watch: {e}");
                return;
            }
        };

        let Some(stdout) = child.stdout.take() else {
            return;
        };
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            match serde_json::from_str::<serde_json::Value>(&line) {
                Ok(json) => {
                    if tx_pods.send(AlertWatchEvent::PodEvent(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "⚠️  Failed to parse pod watch JSON (len={}): {}",
                        line.len(),
                        e
                    );
                }
            }
        }
    });

    // Start kubectl watch for CodeRuns
    let tx_coderuns = tx;
    let namespace_coderuns = namespace.to_string();
    let coderun_watch_handle = tokio::spawn(async move {
        let mut child = match AsyncCommand::new("kubectl")
            .args([
                "get",
                "coderuns.agents.platform",
                "-n",
                &namespace_coderuns,
                "-w",
                "-o",
                "json",
                "--output-watch-events",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to start coderun watch: {e}");
                return;
            }
        };

        let Some(stdout) = child.stdout.take() else {
            return;
        };
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            match serde_json::from_str::<serde_json::Value>(&line) {
                Ok(json) => {
                    if tx_coderuns
                        .send(AlertWatchEvent::CodeRunEvent(json))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "⚠️  Failed to parse coderun watch JSON (len={}): {}",
                        line.len(),
                        e
                    );
                }
            }
        }
    });

    println!("{}", "Watching for pod and CodeRun events...".green());

    // Process events from both watches
    while let Some(event) = rx.recv().await {
        match event {
            AlertWatchEvent::PodEvent(event_json) => {
                // Convert JSON to our Pod type
                let pod = parse_pod_from_json(&event_json["object"], namespace);
                let event_type = event_json["type"].as_str().unwrap_or("");

                // Debug: Log all pod events with container status info
                let terminated_containers: Vec<_> = pod
                    .container_statuses
                    .iter()
                    .filter_map(|cs| {
                        if let k8s::ContainerState::Terminated { exit_code, .. } = &cs.state {
                            Some(format!("{}(exit={})", cs.name, exit_code))
                        } else {
                            None
                        }
                    })
                    .collect();
                if !terminated_containers.is_empty() || pod.phase == "Running" {
                    println!(
                        "{}",
                        format!(
                            "🔍 Pod event: {} type={} phase={} containers={} terminated=[{}]",
                            pod.name,
                            event_type,
                            pod.phase,
                            pod.container_statuses.len(),
                            terminated_containers.join(", ")
                        )
                        .dimmed()
                    );
                }

                // Clean up alerted_pods when pod is deleted (truly gone)
                // Must run BEFORE exclusion check to prevent unbounded memory growth
                // for pods that were alerted on before getting the exclusion label
                if event_type == "DELETED" {
                    let suffix = format!(":{}", pod.name);
                    alerted_pods.retain(|key| !key.ends_with(&suffix));
                    continue; // Nothing more to do for deleted pods
                }

                // Check for exclusion label - skip monitoring this pod if excluded
                if dedup::should_exclude_pod(&pod.labels) {
                    debug!(pod = %pod.name, "Skipping excluded pod (heal.platform/exclude=true)");
                    continue;
                }

                // Determine the K8sEvent type based on phase and event type
                let k8s_event = match (pod.phase.as_str(), event_type) {
                    ("Failed" | "Error", _) => k8s::K8sEvent::PodFailed(pod.clone()),
                    ("Succeeded", _) => k8s::K8sEvent::PodSucceeded(pod.clone()),
                    ("Running", "ADDED") => k8s::K8sEvent::PodRunning(pod.clone()),
                    ("Running" | _, "MODIFIED") => k8s::K8sEvent::PodModified(pod.clone()),
                    _ => continue, // Skip other events
                };

                // Debug: Log any terminated containers in Running pods (potential A2 alerts)
                if pod.phase == "Running" {
                    for cs in &pod.container_statuses {
                        if let k8s::ContainerState::Terminated { exit_code, .. } = &cs.state {
                            if *exit_code != 0 {
                                println!(
                                    "{}",
                                    format!(
                                        "👀 DEBUG: Pod {} has terminated container '{}' (exit={}) while still Running",
                                        pod.name, cs.name, exit_code
                                    ).yellow()
                                );
                            }
                        }
                    }
                }

                // Build alert context from pod labels
                let task_id = pod.labels.get("task-id").cloned().unwrap_or_default();
                let alert_ctx = alerts::AlertContext {
                    task_id: task_id.clone(),
                    repository: String::new(),
                    namespace: namespace.to_string(),
                    pr_number: None,
                    workflow_name: None,
                    config: alerts::types::AlertConfig::default(),
                };

                // Evaluate all alert handlers
                let detected_alerts = registry.evaluate(&k8s_event, &github_state, &alert_ctx);

                // Process each detected alert (with deduplication)
                for alert in detected_alerts {
                    // Build dedup key: "alert_id:pod_name"
                    let dedup_key = format!("{}:{}", alert.id.as_str(), pod.name);

                    // Skip if we've already alerted on this combination
                    if alerted_pods.contains(&dedup_key) {
                        println!(
                            "{}",
                            format!(
                                "⏭️  Skipping duplicate alert {}: {} (already alerted)",
                                alert.id.as_str(),
                                pod.name
                            )
                            .dimmed()
                        );
                        continue;
                    }

                    println!(
                        "{}",
                        format!(
                            "🚨 ALERT {}: {} [severity: {:?}]",
                            alert.id.as_str(),
                            alert.message,
                            alert.severity
                        )
                        .red()
                    );

                    // Emit notification for this alert
                    notifier.notify(notify::NotifyEvent::HealAlert {
                        alert_id: alert.id.as_str().to_string(),
                        severity: match alert.severity {
                            alerts::types::Severity::Info => notify::Severity::Info,
                            alerts::types::Severity::Warning => notify::Severity::Warning,
                            alerts::types::Severity::Critical => notify::Severity::Critical,
                        },
                        message: alert.message.clone(),
                        context: alert.context.clone(),
                        timestamp: chrono::Utc::now(),
                    });

                    // Mark as alerted BEFORE handling to prevent races
                    alerted_pods.insert(dedup_key);

                    // Handle the alert (load prompt, fetch logs, spawn Factory)
                    handle_detected_alert(
                        &alert,
                        &pod,
                        namespace,
                        prompts_dir,
                        dry_run,
                        enable_docker,
                    )
                    .await?;
                }

                // Also check for completion (pod succeeded) - this is a proactive check, not an alert
                // Skip infrastructure pods (cronjobs, platform services)
                if matches!(k8s_event, k8s::K8sEvent::PodSucceeded(_))
                    && !k8s::is_excluded_pod(&pod.name)
                {
                    println!(
                        "{}",
                        format!("✅ Pod {} succeeded - running completion check", pod.name).green()
                    );

                    handle_completion_check(&pod, namespace, prompts_dir, dry_run, enable_docker)
                        .await?;
                }
            }
            AlertWatchEvent::CodeRunEvent(event_json) => {
                // Parse CodeRun from JSON
                let coderun = parse_coderun_from_json(&event_json["object"], namespace);
                let event_type = event_json["type"].as_str().unwrap_or("");

                // Log CodeRun events for visibility
                println!(
                    "{}",
                    format!(
                        "📦 CodeRun {}: phase={} (event={})",
                        coderun.name, coderun.phase, event_type
                    )
                    .dimmed()
                );

                // Track CodeRun for A9 stuck detection
                let phase = coderun.phase.as_str();
                if phase == "Succeeded" || phase == "Failed" || event_type == "DELETED" {
                    // Terminal state or deleted - remove from tracking
                    coderun_tracker.remove(&coderun.name);
                    alerted_coderuns.remove(&coderun.name);
                } else {
                    // Non-terminal state - track first seen time
                    coderun_tracker.record_first_seen(&coderun.name);

                    // Check if we should alert (exceeded threshold and not yet alerted)
                    let config = alerts::types::AlertConfig::default();
                    if coderun_tracker
                        .exceeds_threshold(&coderun.name, config.stuck_coderun_threshold_mins)
                        && !alerted_coderuns.contains(&coderun.name)
                    {
                        // Create K8sEvent and evaluate
                        let k8s_event = k8s::K8sEvent::CodeRunChanged(coderun.clone());

                        let task_id = coderun.task_id.clone();
                        let alert_ctx = alerts::AlertContext {
                            task_id,
                            repository: String::new(),
                            namespace: namespace.to_string(),
                            pr_number: None,
                            workflow_name: None,
                            config,
                        };

                        let detected_alerts =
                            registry.evaluate(&k8s_event, &github_state, &alert_ctx);

                        for alert in detected_alerts {
                            println!(
                                "{}",
                                format!(
                                    "🚨 ALERT {}: {} [severity: {:?}]",
                                    alert.id.as_str(),
                                    alert.message,
                                    alert.severity
                                )
                                .red()
                            );

                            // Emit notification for this CodeRun alert
                            notifier.notify(notify::NotifyEvent::HealAlert {
                                alert_id: alert.id.as_str().to_string(),
                                severity: match alert.severity {
                                    alerts::types::Severity::Info => notify::Severity::Info,
                                    alerts::types::Severity::Warning => notify::Severity::Warning,
                                    alerts::types::Severity::Critical => notify::Severity::Critical,
                                },
                                message: alert.message.clone(),
                                context: alert.context.clone(),
                                timestamp: chrono::Utc::now(),
                            });

                            // Mark as alerted to avoid spam
                            alerted_coderuns.insert(coderun.name.clone());

                            // Handle the alert for CodeRun
                            handle_coderun_alert(
                                &alert,
                                &coderun,
                                namespace,
                                prompts_dir,
                                dry_run,
                                enable_docker,
                            )
                            .await?;
                        }
                    }
                }
            }
        }
    }

    // Wait for watch tasks (they'll run until the channel closes)
    let _ = pod_watch_handle.await;
    let _ = coderun_watch_handle.await;
    let _ = poller_handle.await;

    Ok(())
}

/// Parse kubectl JSON output into our `CodeRun` type.
fn parse_coderun_from_json(json: &serde_json::Value, namespace: &str) -> k8s::CodeRun {
    let mut labels = std::collections::HashMap::new();
    if let Some(obj) = json["metadata"]["labels"].as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                labels.insert(k.clone(), s.to_string());
            }
        }
    }

    k8s::CodeRun {
        name: json["metadata"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        namespace: namespace.to_string(),
        phase: json["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
        agent: json["spec"]["githubApp"].as_str().unwrap_or("").to_string(),
        task_id: json["spec"]["taskId"]
            .as_str()
            .map(String::from)
            .or_else(|| json["spec"]["taskId"].as_i64().map(|n| n.to_string()))
            .unwrap_or_default(),
        labels,
    }
}

/// Handle a detected alert for a `CodeRun`.
async fn handle_coderun_alert(
    alert: &alerts::Alert,
    coderun: &k8s::CodeRun,
    namespace: &str,
    prompts_dir: &str,
    dry_run: bool,
    enable_docker: bool,
) -> Result<()> {
    let alert_id = alert.id.as_str().to_lowercase();
    let task_id = &coderun.task_id;
    let agent = &coderun.agent;

    // For CodeRun alerts, we use the coderun name as the "pod" name for consistency
    handle_alert(
        &alert_id,
        &coderun.name,
        if task_id.is_empty() {
            "unknown"
        } else {
            task_id
        },
        if agent.is_empty() { "unknown" } else { agent },
        namespace,
        &coderun.phase,
        prompts_dir,
        dry_run,
        Some(&alert.context), // Pass full alert context for template rendering
        enable_docker,
    )
    .await
}

/// Parse kubectl JSON output into our Pod type
fn parse_pod_from_json(json: &serde_json::Value, namespace: &str) -> k8s::Pod {
    let mut labels = std::collections::HashMap::new();
    if let Some(obj) = json["metadata"]["labels"].as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                labels.insert(k.clone(), s.to_string());
            }
        }
    }

    let mut container_statuses = Vec::new();
    if let Some(statuses) = json["status"]["containerStatuses"].as_array() {
        for status in statuses {
            #[allow(clippy::cast_possible_truncation)] // exit codes are small i32 values
            let state = if status["state"]["terminated"].is_object() {
                let terminated = &status["state"]["terminated"];
                k8s::ContainerState::Terminated {
                    exit_code: terminated["exitCode"].as_i64().unwrap_or(0) as i32,
                    reason: terminated["reason"]
                        .as_str()
                        .map(std::string::ToString::to_string),
                    finished_at: terminated["finishedAt"]
                        .as_str()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                }
            } else if status["state"]["running"].is_object() {
                let running = &status["state"]["running"];
                k8s::ContainerState::Running {
                    started_at: running["startedAt"]
                        .as_str()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                }
            } else {
                let waiting = &status["state"]["waiting"];
                k8s::ContainerState::Waiting {
                    reason: waiting["reason"]
                        .as_str()
                        .map(std::string::ToString::to_string),
                }
            };

            #[allow(clippy::cast_possible_truncation)] // restart counts are small i32 values
            container_statuses.push(k8s::ContainerStatus {
                name: status["name"].as_str().unwrap_or("").to_string(),
                ready: status["ready"].as_bool().unwrap_or(false),
                state,
                restart_count: status["restartCount"].as_i64().unwrap_or(0) as i32,
            });
        }
    }

    // Parse pod conditions
    let mut conditions = Vec::new();
    if let Some(conds) = json["status"]["conditions"].as_array() {
        for cond in conds {
            conditions.push(k8s::PodCondition {
                condition_type: cond["type"].as_str().unwrap_or("").to_string(),
                status: cond["status"].as_str().unwrap_or("Unknown").to_string(),
                reason: cond["reason"]
                    .as_str()
                    .map(std::string::ToString::to_string),
                message: cond["message"]
                    .as_str()
                    .map(std::string::ToString::to_string),
            });
        }
    }

    // Parse pod start time
    let started_at = json["status"]["startTime"]
        .as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    k8s::Pod {
        name: json["metadata"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        namespace: namespace.to_string(),
        phase: json["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
        labels,
        conditions,
        container_statuses,
        started_at,
    }
}

/// Handle a detected alert from the registry
async fn handle_detected_alert(
    alert: &alerts::Alert,
    pod: &k8s::Pod,
    namespace: &str,
    prompts_dir: &str,
    dry_run: bool,
    enable_docker: bool,
) -> Result<()> {
    let alert_id = alert.id.as_str().to_lowercase();
    let task_id = alert
        .context
        .get("task_id")
        .map_or("unknown", String::as_str);
    let agent = alert.context.get("agent").map_or("unknown", String::as_str);

    // Check for recent GitHub issue with same alert type AND workflow family (prevents issue spam)
    // Different workflows (e.g., atlas-* vs play-*) get separate issues
    if !dry_run {
        match dedup::check_recent_alert_type_issue(&alert_id, &pod.name, "5dlabs/cto") {
            Ok(Some((issue_num, title))) => {
                println!(
                    "{}",
                    format!(
                        "⏭️  Skipping {} for {} - recent issue #{} exists: {}",
                        alert_id.to_uppercase(),
                        pod.name,
                        issue_num,
                        title
                    )
                    .yellow()
                );
                // TODO: Consider adding a comment to the existing issue instead
                return Ok(());
            }
            Ok(None) => {
                println!(
                    "{}",
                    format!(
                        "✅ No recent {} issues found, proceeding",
                        alert_id.to_uppercase()
                    )
                    .dimmed()
                );
            }
            Err(e) => {
                println!(
                    "{}",
                    format!("⚠️  Alert-type dedup check failed: {e}").yellow()
                );
                // Continue anyway - don't block on dedup failures
            }
        }
    }

    handle_alert(
        &alert_id,
        &pod.name,
        task_id,
        agent,
        namespace,
        &pod.phase,
        prompts_dir,
        dry_run,
        Some(&alert.context), // Pass full alert context for template rendering
        enable_docker,
    )
    .await
}

/// Handle completion check for a succeeded pod
async fn handle_completion_check(
    pod: &k8s::Pod,
    namespace: &str,
    prompts_dir: &str,
    dry_run: bool,
    enable_docker: bool,
) -> Result<()> {
    let task_id = pod.labels.get("task-id").map_or("unknown", String::as_str);
    let agent = pod.labels.get("agent").map_or("unknown", String::as_str);

    handle_alert(
        "completion",
        &pod.name,
        task_id,
        agent,
        namespace,
        &pod.phase,
        prompts_dir,
        dry_run,
        None, // No additional context for completion checks
        enable_docker,
    )
    .await
}

/// Handle a detected alert by loading prompt and spawning Factory
#[allow(clippy::too_many_arguments)]
async fn handle_alert(
    alert_id: &str,
    pod_name: &str,
    task_id: &str,
    agent: &str,
    namespace: &str,
    phase: &str,
    prompts_dir: &str,
    dry_run: bool,
    alert_context: Option<&std::collections::HashMap<String, String>>,
    enable_docker: bool,
) -> Result<()> {
    // Fetch pod logs (with Loki fallback for GC'd pods)
    let logs = get_pod_logs_with_loki_fallback(pod_name, namespace, 500).await;

    // For completion checks, load agent-specific expected behaviors
    let expected_behaviors = if alert_id == "completion" {
        let expected_file = format!("{}/expected/{}.md", prompts_dir, agent.to_lowercase());
        std::fs::read_to_string(&expected_file)
            .unwrap_or_else(|_| format!("# Expected behaviors for {agent} not found"))
    } else {
        String::new()
    };

    // Build template context
    let template_filename = templates::TemplateEngine::alert_to_filename(alert_id);
    let context = templates::AlertContext {
        alert_id: alert_id.to_string(),
        pod_name: pod_name.to_string(),
        namespace: namespace.to_string(),
        phase: phase.to_string(),
        task_id: task_id.to_string(),
        agent: agent.to_string(),
        logs,
        expected_behaviors,
        duration: "N/A".to_string(),
        enable_docker,
        extra: alert_context.cloned().unwrap_or_default(),
    };

    // Try Handlebars rendering first, fall back to legacy
    let rendered = match templates::TemplateEngine::new(prompts_dir) {
        Ok(engine) => match engine.render_alert(&template_filename, &context) {
            Ok(r) => r,
            Err(e) => {
                println!(
                    "{}",
                    format!("Template rendering failed, using legacy: {e}").yellow()
                );
                render_legacy_template(prompts_dir, alert_id, &context)?
            }
        },
        Err(e) => {
            println!(
                "{}",
                format!("Template engine init failed, using legacy: {e}").yellow()
            );
            render_legacy_template(prompts_dir, alert_id, &context)?
        }
    };

    // Warn about any unreplaced template variables (helps catch missing context)
    let unreplaced: Vec<&str> = rendered
        .match_indices("{{")
        .filter_map(|(start, _)| {
            let end = rendered[start..].find("}}")?;
            Some(&rendered[start..start + end + 2])
        })
        .collect();
    if !unreplaced.is_empty() && !dry_run {
        println!(
            "{}",
            format!(
                "⚠️  Template has unreplaced variables: {}",
                unreplaced.join(", ")
            )
            .yellow()
        );
    }

    if dry_run {
        println!("{}", "=".repeat(80).dimmed());
        println!(
            "{}",
            format!("RENDERED PROMPT FOR {}:", alert_id.to_uppercase()).cyan()
        );
        println!("{}", "=".repeat(80).dimmed());
        println!("{rendered}");
        println!("{}", "=".repeat(80).dimmed());
        if !unreplaced.is_empty() {
            println!(
                "{}",
                format!("⚠️  Unreplaced variables: {}", unreplaced.join(", ")).yellow()
            );
        }
        return Ok(());
    }

    // Write prompt to temp file and spawn Factory
    let prompt_path = format!("/tmp/alert-{alert_id}-{pod_name}.md");
    std::fs::write(&prompt_path, &rendered)?;

    spawn_factory_with_prompt(&prompt_path, pod_name, alert_id).await?;

    Ok(())
}

/// Legacy template rendering for backward compatibility with .md files
fn render_legacy_template(
    prompts_dir: &str,
    alert_id: &str,
    context: &templates::AlertContext,
) -> Result<String> {
    let prompt_file = match alert_id {
        "a1" => format!("{prompts_dir}/a1-comment-order.md"),
        "a2" => format!("{prompts_dir}/a2-silent-failure.md"),
        "a3" => format!("{prompts_dir}/a3-stale-progress.md"),
        "a4" => format!("{prompts_dir}/a4-approval-loop.md"),
        "a5" => format!("{prompts_dir}/a5-post-tess-ci.md"),
        "a7" => format!("{prompts_dir}/a7-pod-failure.md"),
        "a8" => format!("{prompts_dir}/a8-step-timeout.md"),
        "a9" => format!("{prompts_dir}/a9-stuck-coderun.md"),
        "completion" => format!("{prompts_dir}/success-completion.md"),
        _ => anyhow::bail!("Unknown alert ID: {alert_id}"),
    };

    let template = std::fs::read_to_string(&prompt_file)
        .with_context(|| format!("Failed to load prompt {prompt_file}"))?;

    let mut rendered = template
        .replace("{{pod_name}}", &context.pod_name)
        .replace("{{namespace}}", &context.namespace)
        .replace("{{phase}}", &context.phase)
        .replace("{{task_id}}", &context.task_id)
        .replace("{{agent}}", &context.agent)
        .replace("{{logs}}", &context.logs)
        .replace("{{expected_behaviors}}", &context.expected_behaviors)
        .replace("{{duration}}", &context.duration);

    for (key, value) in &context.extra {
        let pattern = format!("{{{{{key}}}}}");
        rendered = rendered.replace(&pattern, value);
    }

    Ok(rendered)
}

/// Fetch recent logs for a pod (sync version, kubectl only)
fn get_pod_logs_for_alert(pod_name: &str, namespace: &str, tail: u32) -> String {
    let output = std::process::Command::new("kubectl")
        .args([
            "logs",
            pod_name,
            "-n",
            namespace,
            "--tail",
            &tail.to_string(),
            "--all-containers",
        ])
        .output();

    let logs = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        Ok(out) => {
            format!(
                "[Failed to fetch logs: {}]",
                String::from_utf8_lossy(&out.stderr)
            )
        }
        Err(e) => format!("[Error fetching logs: {e}]"),
    };

    // Redact secrets before returning
    redact_secrets(&logs)
}

/// Fetch logs for a pod with Loki fallback (async version)
///
/// Tries kubectl logs first (for live pods), then falls back to Loki
/// for historical logs if the pod has been garbage collected.
async fn get_pod_logs_with_loki_fallback(pod_name: &str, namespace: &str, tail: u32) -> String {
    // First, try kubectl logs (works for live pods)
    let kubectl_logs = get_pod_logs_for_alert(pod_name, namespace, tail);

    // If kubectl succeeded (doesn't contain error markers), return those logs
    if !kubectl_logs.contains("[Failed to fetch logs:")
        && !kubectl_logs.contains("[Error fetching logs:")
        && !kubectl_logs.is_empty()
    {
        return kubectl_logs;
    }

    // kubectl failed - try Loki for historical logs
    println!(
        "{}",
        format!(
            "📚 Pod {pod_name} logs unavailable via kubectl, querying Loki for historical logs..."
        )
        .cyan()
    );

    let loki_client = loki::LokiClient::with_defaults();

    // Check if Loki is reachable
    if !matches!(loki_client.health_check().await, Ok(true)) {
        println!(
            "{}",
            "⚠️  Loki not reachable, returning kubectl error".yellow()
        );
        return kubectl_logs;
    }

    // Query Loki for logs from the last 30 minutes
    let end = Utc::now();
    let start = end - chrono::Duration::minutes(30);

    match loki_client
        .query_pod_logs(namespace, pod_name, start, end, tail)
        .await
    {
        Ok(entries) if !entries.is_empty() => {
            println!(
                "{}",
                format!(
                    "✅ Retrieved {} log entries from Loki for {}",
                    entries.len(),
                    pod_name
                )
                .green()
            );

            // Format logs with timestamps
            let logs = format_loki_entries_as_logs(&entries, tail as usize);

            // Redact secrets before returning
            redact_secrets(&logs)
        }
        Ok(_) => {
            println!(
                "{}",
                format!("⚠️  No logs found in Loki for {pod_name}").yellow()
            );
            format!("[No logs available - pod {pod_name} not found in kubectl or Loki]")
        }
        Err(e) => {
            println!(
                "{}",
                format!("⚠️  Loki query failed for {pod_name}: {e}").yellow()
            );
            // Return the original kubectl error
            kubectl_logs
        }
    }
}

/// Format Loki log entries as a string for inclusion in prompts
fn format_loki_entries_as_logs(entries: &[loki::LogEntry], max_entries: usize) -> String {
    use std::fmt::Write;

    let mut logs =
        String::from("# Historical logs from Loki (pod may have been garbage collected)\n\n");

    // Take the LAST max_entries entries (most recent) to match kubectl --tail behavior
    // Entries are sorted oldest-first, so slice from the end
    let total = entries.len();
    let entries_to_show = if max_entries > 0 && total > max_entries {
        &entries[total - max_entries..]
    } else {
        entries
    };

    for entry in entries_to_show {
        let time = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(logs, "[{}] {}", time, entry.line);
    }
    logs
}

/// Redact sensitive information from logs to prevent secret leakage
fn redact_secrets(text: &str) -> String {
    use std::borrow::Cow;

    let mut result = Cow::Borrowed(text);

    // Patterns for common secret formats
    let secret_patterns = [
        // API keys with known prefixes
        (r"sk-ant-[a-zA-Z0-9_-]+", "[REDACTED_ANTHROPIC_KEY]"),
        (r"sk-proj-[a-zA-Z0-9_-]+", "[REDACTED_OPENAI_KEY]"),
        (r"ctx7sk-[a-zA-Z0-9-]+", "[REDACTED_CONTEXT7_KEY]"),
        (r"fk-[a-zA-Z0-9_-]+", "[REDACTED_FACTORY_KEY]"),
        (r"pplx-[a-zA-Z0-9]+", "[REDACTED_PERPLEXITY_KEY]"),
        (r"xai-[a-zA-Z0-9]+", "[REDACTED_XAI_KEY]"),
        (r"key_[a-f0-9]{64}", "[REDACTED_CURSOR_KEY]"),
        (r"AIzaSy[a-zA-Z0-9_-]+", "[REDACTED_GOOGLE_KEY]"),
        // Generic patterns for JSON secret blocks
        (
            r#""ANTHROPIC_API_KEY":"[^"]+""#,
            r#""ANTHROPIC_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""OPENAI_API_KEY":"[^"]+""#,
            r#""OPENAI_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""GEMINI_API_KEY":"[^"]+""#,
            r#""GEMINI_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""GOOGLE_API_KEY":"[^"]+""#,
            r#""GOOGLE_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""CONTEXT7_API_KEY":"[^"]+""#,
            r#""CONTEXT7_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""CURSOR_API_KEY":"[^"]+""#,
            r#""CURSOR_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""FACTORY_API_KEY":"[^"]+""#,
            r#""FACTORY_API_KEY":"[REDACTED]""#,
        ),
        (
            r#""PERPLEXITY_API_KEY":"[^"]+""#,
            r#""PERPLEXITY_API_KEY":"[REDACTED]""#,
        ),
        (r#""XAI_API_KEY":"[^"]+""#, r#""XAI_API_KEY":"[REDACTED]""#),
        // OpenBao/secrets raw output blocks (entire _raw JSON)
        (r"_raw=\{[^}]+\}", "_raw={[REDACTED_SECRETS_DATA]}"),
    ];

    for (pattern, replacement) in secret_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = Cow::Owned(re.replace_all(&result, replacement).to_string());
        }
    }

    result.into_owned()
}

/// Spawn Factory (droid exec) with the rendered prompt
/// Output is written to /workspace/watch/logs/ for sidecar to tail
#[allow(clippy::too_many_lines)] // Complex log handling requires all steps together
async fn spawn_factory_with_prompt(
    prompt_path: &str,
    pod_name: &str,
    alert_id: &str,
) -> Result<()> {
    use std::io::Write;
    use tokio::process::Command as AsyncCommand;

    // Load healer config for server settings (use absolute path to match CLI default)
    let config = load_healer_config("/app/heal-config.json");
    let server = &config.server;

    let prompt_content =
        std::fs::read_to_string(prompt_path).context("Failed to read prompt file")?;

    // Build memory-augmented prompt if memory is enabled
    let augmented_prompt = if server.memory.enabled {
        build_memory_augmented_prompt(&prompt_content, alert_id, pod_name, &server.memory)
    } else {
        prompt_content.clone()
    };

    // Create log directory and file
    let log_dir = "/workspace/watch/logs";
    std::fs::create_dir_all(log_dir).ok();

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let safe_pod_name = pod_name.chars().take(50).collect::<String>();
    let log_file = format!(
        "{}/{}-{}-{}.log",
        log_dir,
        alert_id.to_uppercase(),
        safe_pod_name,
        timestamp
    );

    println!(
        "{}",
        format!("🚀 Spawning Factory for alert {alert_id} on pod {pod_name} → {log_file}").cyan()
    );
    if server.memory.enabled {
        println!(
            "{}",
            format!(
                "🧠 Memory enabled: agent={}, session={}-{alert_id}",
                server.memory.agent_name, server.memory.session_prefix
            )
            .dimmed()
        );
    }

    // Write header to log file
    let mut file = std::fs::File::create(&log_file).context("Failed to create log file")?;
    writeln!(
        file,
        "═══════════════════════════════════════════════════════════════"
    )?;
    writeln!(
        file,
        "ALERT: {} | POD: {}",
        alert_id.to_uppercase(),
        pod_name
    )?;
    writeln!(file, "TIME: {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(
        file,
        "MODEL: {} | AUTO: {}",
        server.model, server.auto_level
    )?;
    writeln!(
        file,
        "═══════════════════════════════════════════════════════════════"
    )?;
    writeln!(file)?;
    writeln!(file, "=== PROMPT ===")?;
    writeln!(file, "{augmented_prompt}")?;
    writeln!(file)?;
    writeln!(file, "=== FACTORY OUTPUT ===")?;
    drop(file); // Close before spawning

    // Build droid exec args from config
    let auto_arg = format!("--auto={}", server.auto_level);
    let format_arg = format!("--output-format={}", server.output_format);
    let model_arg = format!("--model={}", server.model);

    // Use stdin to pass prompt content to avoid "Argument list too long" errors
    // when prompts include large pod logs (OS arg limit is typically 128KB-2MB)
    // droid exec reads from stdin when no prompt argument is provided
    let output = AsyncCommand::new("droid")
        .args(["exec", &format_arg, &auto_arg, &model_arg])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let output = match output {
        Ok(mut child) => {
            // Write prompt to stdin (use augmented_prompt which includes memory instructions if enabled)
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                if let Err(e) = stdin.write_all(augmented_prompt.as_bytes()).await {
                    anyhow::bail!("Failed to write prompt to stdin: {e}");
                }
                drop(stdin); // Close stdin to signal EOF
            }
            child.wait_with_output().await
        }
        Err(e) => Err(e),
    };

    // Append output to log file
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&log_file)
        .context("Failed to open log file for append")?;

    match output {
        Ok(out) => {
            // Write stdout
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.is_empty() {
                writeln!(file, "{stdout}")?;
            }

            // Write stderr
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.is_empty() {
                writeln!(file, "=== STDERR ===")?;
                writeln!(file, "{stderr}")?;
            }

            // Write exit status
            writeln!(file)?;
            writeln!(
                file,
                "═══════════════════════════════════════════════════════════════"
            )?;
            writeln!(file, "EXIT CODE: {:?}", out.status.code())?;
            writeln!(
                file,
                "═══════════════════════════════════════════════════════════════"
            )?;

            if out.status.success() {
                println!("{}", format!("✅ Factory completed → {log_file}").green());
            } else {
                println!(
                    "{}",
                    format!("⚠️ Factory exited {:?} → {}", out.status.code(), log_file).yellow()
                );
            }

            // Echo Factory analysis to console for visibility in pod logs
            if !stdout.is_empty() {
                println!("{}", "─── Factory Analysis ───".cyan());
                let lines: Vec<&str> = stdout.lines().collect();
                for line in lines.iter().take(30) {
                    println!("{line}");
                }
                if lines.len() > 30 {
                    println!(
                        "{}",
                        format!("... ({} more lines in {log_file})", lines.len() - 30).dimmed()
                    );
                }
                println!("{}", "────────────────────────".cyan());
            }
        }
        Err(e) => {
            writeln!(file, "ERROR: Failed to spawn: {e}")?;
            println!(
                "{}",
                format!("❌ Failed to spawn Factory: {e}. Is 'droid' in PATH?").red()
            );
        }
    }

    Ok(())
}

/// Test an alert flow manually
async fn test_alert_flow(
    alert_id: &str,
    pod_name: &str,
    task_id: &str,
    agent: &str,
    prompts_dir: &str,
    dry_run: bool,
    enable_docker: bool,
) -> Result<()> {
    println!(
        "{}",
        format!("Testing alert flow for: {}", alert_id.to_uppercase()).cyan()
    );

    handle_alert(
        alert_id,
        pod_name,
        task_id,
        agent,
        "test-namespace",
        "Failed", // Simulated phase
        prompts_dir,
        dry_run,
        None, // No context for test alerts
        enable_docker,
    )
    .await?;

    Ok(())
}

/// Spawn a remediation agent for a detected issue by creating a `CodeRun` CRD.
#[allow(clippy::too_many_lines)] // Dedup check + file validation + YAML generation in one flow
fn spawn_remediation_agent(
    alert: &str,
    task_id: &str,
    target_pod: Option<&str>,
    issue_number: Option<u64>,
    issue_file: Option<&str>,
    config_path: &str,
) -> Result<()> {
    println!("{}", "═".repeat(60).cyan());
    println!(
        "{}",
        format!("🔧 SPAWN REMEDIATION: alert={alert} task={task_id}")
            .cyan()
            .bold()
    );
    println!("{}", "═".repeat(60).cyan());

    // Load heal config (fall back to defaults if not found)
    let config = load_healer_config(config_path);

    // Check for existing remediation (deduplication)
    if let Some(pod_name) = target_pod {
        println!(
            "{}",
            format!("🔍 Checking for existing remediation: alert={alert} pod={pod_name}").dimmed()
        );
        match dedup::check_existing_remediation(alert, pod_name, &config.coderun.namespace) {
            Ok(Some(existing)) => {
                println!(
                    "{}",
                    format!("⏭️  Skipping - active remediation exists: {existing}").yellow()
                );
                return Ok(());
            }
            Ok(None) => {
                println!("{}", "✅ No existing remediation found".dimmed());
            }
            Err(e) => {
                println!(
                    "{}",
                    format!("⚠️  Dedup check failed, proceeding anyway: {e}").yellow()
                );
            }
        }
    }
    println!(
        "{}",
        format!(
            "📋 Config: {} ({})",
            if std::path::Path::new(config_path).exists() {
                config_path
            } else {
                "defaults"
            },
            config.coderun.github_app
        )
        .dimmed()
    );

    // Determine issue directory and files based on issue_number or issue_file
    let (issue_dir, prompt_file, acceptance_file) = if let Some(num) = issue_number {
        let dir = format!("/workspace/watch/issues/issue-{num}");
        // Validate directory exists before deriving file paths
        if !std::path::Path::new(&dir).exists() {
            println!(
                "{}",
                format!("❌ Issue directory not found: {dir} (issue #{num})").red()
            );
            return Err(anyhow::anyhow!(
                "Issue directory not found: {dir} (issue #{num})"
            ));
        }
        let prompt = format!("{dir}/prompt.md");
        let acceptance = format!("{dir}/acceptance-criteria.md");
        println!(
            "{}",
            format!("📁 Issue directory: {dir} (from issue #{num})").dimmed()
        );
        (Some(dir), prompt, Some(acceptance))
    } else if let Some(file) = issue_file {
        println!("{}", format!("📄 Using legacy issue file: {file}").yellow());
        (None, file.to_string(), None)
    } else {
        anyhow::bail!("Either --issue-number or --issue-file must be provided");
    };

    // Verify the prompt file exists
    println!(
        "{}",
        format!("📄 Checking prompt file: {prompt_file}").dimmed()
    );
    if !std::path::Path::new(&prompt_file).exists() {
        println!(
            "{}",
            format!("❌ Prompt file not found: {prompt_file}").red()
        );
        return Err(anyhow::anyhow!("Prompt file not found: {prompt_file}"));
    }
    println!("{}", "   ✓ Prompt file exists".green());

    // Check acceptance criteria if using issue_number
    if let Some(ref acc_file) = acceptance_file {
        if std::path::Path::new(acc_file).exists() {
            println!("{}", "   ✓ Acceptance criteria exists".green());
        } else {
            println!(
                "{}",
                format!("   ⚠️  Acceptance criteria not found: {acc_file}").yellow()
            );
        }
    }

    // Generate unique CodeRun name with new naming pattern:
    // healer-remediation-task{task_id}-{alert_type}-{alert_id}
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let coderun_name = format!("healer-remediation-task{task_id}-{alert}-{uid}");

    println!("{}", format!("🏷️  CodeRun name: {coderun_name}").dimmed());
    println!("{}", format!("⏰ Timestamp: {timestamp}").dimmed());
    if let Some(num) = issue_number {
        println!(
            "{}",
            format!("🔗 GitHub Issue: #{num} (PR will link with 'Fixes #{num}')").green()
        );
    }

    // Derive log file path from alert pattern (matches spawn_factory_with_prompt output)
    let log_dir = "/workspace/watch/logs";
    let log_file = format!("{log_dir}/{}-*.log", alert.to_uppercase());

    let coderun_yaml = build_coderun_yaml(
        alert,
        task_id,
        target_pod,
        &prompt_file,
        &log_file,
        &coderun_name,
        &timestamp,
        issue_number,
        issue_dir.as_deref(),
        acceptance_file.as_deref(),
        &config,
    );
    apply_coderun(&coderun_yaml, &coderun_name)
}

/// Load heal configuration from file, falling back to defaults if not found.
fn load_healer_config(config_path: &str) -> HealerConfig {
    match std::fs::read_to_string(config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                println!(
                    "{}",
                    format!("⚠️  Config parse error, using defaults: {e}").yellow()
                );
                HealerConfig::default()
            }
        },
        Err(_) => {
            // Config file not found - use defaults silently
            HealerConfig::default()
        }
    }
}

/// Build the `CodeRun` YAML manifest using values from config.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn build_coderun_yaml(
    alert: &str,
    task_id: &str,
    target_pod: Option<&str>,
    prompt_file: &str,
    log_file: &str,
    coderun_name: &str,
    timestamp: &impl std::fmt::Display,
    issue_number: Option<u64>,
    issue_dir: Option<&str>,
    acceptance_file: Option<&str>,
    config: &HealerConfig,
) -> String {
    // Hash task_id to numeric (CodeRun requires integer taskId)
    let task_id_numeric: u32 = task_id.bytes().fold(0u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u32::from(b))
    });

    let c = &config.coderun;

    // Build target-pod label (sanitized for K8s label requirements)
    let target_pod_label = target_pod
        .map(|p| format!("    target-pod: \"{}\"\n", dedup::sanitize_label_value(p)))
        .unwrap_or_default();

    // Build issue-number label
    let issue_number_label = issue_number
        .map(|n| format!("    issue-number: \"{n}\"\n"))
        .unwrap_or_default();

    // Build optional fields only if non-empty
    // Join arrays into comma-separated strings for CRD spec
    let remote_tools_line = if c.remote_tools.is_empty() {
        String::new()
    } else {
        format!("  remoteTools: \"{}\"\n", c.remote_tools.join(","))
    };
    let local_tools_line = if c.local_tools.is_empty() {
        String::new()
    } else {
        format!("  localTools: \"{}\"\n", c.local_tools.join(","))
    };

    // Build issue-related env vars
    // Transform paths from heal server view (/workspace/watch/...) to remediation pod view (/workspace/...)
    // The heal server mounts PVC at /workspace/watch, but remediation pods mount at /workspace
    let transform_path = |p: &str| p.replace("/workspace/watch/", "/workspace/");

    let issue_number_line = issue_number
        .map(|n| format!("    HEAL_ISSUE_NUMBER: \"{n}\"\n"))
        .unwrap_or_default();
    let issue_dir_line = issue_dir
        .map(|d| format!("    HEAL_ISSUE_DIR: \"{}\"\n", transform_path(d)))
        .unwrap_or_default();
    let acceptance_line = acceptance_file
        .map(|f| format!("    HEAL_ACCEPTANCE_FILE: \"{}\"\n", transform_path(f)))
        .unwrap_or_default();

    format!(
        r#"apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: {coderun_name}
  namespace: {namespace}
  labels:
    alert-type: "{alert}"
    task-id: "{task_id}"
    remediation: "true"
    created-at: "{timestamp}"
    agents.platform/type: healer-remediation
{target_pod_label}{issue_number_label}spec:
  taskId: {task_id_numeric}
  runType: "{run_type}"
  githubApp: "{github_app}"
  model: "{model}"
  repositoryUrl: "{repository_url}"
  docsRepositoryUrl: "{docs_repository_url}"
  docsProjectDirectory: "{docs_project_directory}"
  docsBranch: "{docs_branch}"
  workingDirectory: "{working_directory}"
  service: "{service}"
  enableDocker: {enable_docker}
{remote_tools}{local_tools}  cliConfig:
    cliType: "{cli_type}"
    model: "{cli_model}"
    settings:
      template: "{template}"
      memoryEnabled: {memory_enabled}
      memoryAgentName: "{memory_agent_name}"
      memorySessionPrefix: "{memory_session_prefix}"
      memoryRetrieveOnStart: {memory_retrieve_on_start}
      memoryPersistOnComplete: {memory_persist_on_complete}
  env:
    ALERT_TYPE: "{alert}"
    TASK_ID: "{task_id}"
    HEAL_PROMPT_FILE: "{prompt_file_transformed}"
    HEAL_LOG_FILE: "{log_file_transformed}"
    CODERUN_NAME: "{coderun_name}"
    REMEDIATION_MODE: "true"
{issue_number}{issue_dir}{acceptance}
"#,
        namespace = c.namespace,
        run_type = c.run_type,
        github_app = c.github_app,
        model = c.model,
        repository_url = c.repository_url,
        docs_repository_url = c.docs_repository_url,
        docs_project_directory = c.docs_project_directory,
        docs_branch = c.docs_branch,
        working_directory = c.working_directory,
        service = c.service,
        enable_docker = c.enable_docker,
        remote_tools = remote_tools_line,
        local_tools = local_tools_line,
        cli_type = c.cli_config.cli_type,
        cli_model = c.cli_config.model,
        template = c.cli_config.settings.template,
        memory_enabled = c.memory.enabled,
        memory_agent_name = c.memory.agent_name,
        memory_session_prefix = c.memory.session_prefix,
        memory_retrieve_on_start = c.memory.retrieve_on_start,
        memory_persist_on_complete = c.memory.persist_on_complete,
        prompt_file_transformed = transform_path(prompt_file),
        log_file_transformed = transform_path(log_file),
        coderun_name = coderun_name,
        target_pod_label = target_pod_label,
        issue_number_label = issue_number_label,
        issue_number = issue_number_line,
        issue_dir = issue_dir_line,
        acceptance = acceptance_line,
    )
}

/// Apply the `CodeRun` YAML via kubectl.
fn apply_coderun(coderun_yaml: &str, coderun_name: &str) -> Result<()> {
    use std::io::Write as _;

    println!("{}", "📝 Generated CodeRun YAML:".dimmed());
    for line in coderun_yaml.lines().take(10) {
        println!("   {}", line.dimmed());
    }
    println!("   {}", "...".dimmed());
    println!("{}", "🚀 Applying CodeRun via kubectl...".yellow());

    let mut child = Command::new("kubectl")
        .args(["apply", "-f", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn kubectl apply")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(coderun_yaml.as_bytes())
            .context("Failed to write YAML")?;
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for kubectl")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{}", format!("❌ kubectl apply failed: {stderr}").red());
        return Err(anyhow::anyhow!(
            "Failed to create remediation CodeRun: {stderr}"
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", format!("   kubectl: {}", stdout.trim()).green());
    println!("{}", "═".repeat(60).green());
    println!(
        "{}",
        format!("✅ CREATED: {coderun_name} in namespace cto")
            .green()
            .bold()
    );
    println!("{}", "═".repeat(60).green());
    println!(
        "{}",
        format!("👀 Monitor: kubectl get coderun {coderun_name} -n cto -w").dimmed()
    );
    Ok(())
}

/// Fetch all logs for a pod (current, previous, events, describe).
fn fetch_pod_logs(pod_name: &str, namespace: &str, output_dir: &str, tail: u32) -> Result<()> {
    println!("{}", "═".repeat(60).cyan());
    println!(
        "{}",
        format!("📥 FETCH LOGS: pod={pod_name} ns={namespace}")
            .cyan()
            .bold()
    );
    println!("{}", "═".repeat(60).cyan());

    println!(
        "{}",
        format!("📁 Creating output dir: {output_dir}").dimmed()
    );
    std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    println!("{}", "   ✓ Directory ready".green());

    let tail_arg = if tail == 0 {
        String::new()
    } else {
        format!("--tail={tail}")
    };

    // Track successes to report accurately
    let mut successes = 0u8;
    let mut files_created = Vec::new();

    // 1. Current logs
    let current = fetch_log_type(pod_name, namespace, output_dir, &tail_arg, "current", false);
    if current.is_ok() {
        successes += 1;
        files_created.push(format!("{pod_name}-current.log"));
    }

    // 2. Previous logs (optional - pod may not have restarted)
    let previous = fetch_log_type(pod_name, namespace, output_dir, &tail_arg, "previous", true);
    if previous.is_ok() {
        files_created.push(format!("{pod_name}-previous.log"));
    }

    // 3. Events
    let events = fetch_pod_events(pod_name, namespace, output_dir);
    if events.is_ok() {
        successes += 1;
        files_created.push(format!("{pod_name}-events.yaml"));
    }

    // 4. Describe
    let describe = fetch_pod_describe(pod_name, namespace, output_dir);
    if describe.is_ok() {
        successes += 1;
        files_created.push(format!("{pod_name}-describe.txt"));
    }

    // Summary - report actual results
    if successes > 0 {
        println!("{}", "═".repeat(60).green());
        println!(
            "{}",
            format!("✅ LOG FETCH COMPLETE ({successes}/3 required succeeded)")
                .green()
                .bold()
        );
        println!("{}", "═".repeat(60).green());
        println!("{}", format!("📂 Output directory: {output_dir}").dimmed());
        println!("{}", "   Files created:".dimmed());
        for f in &files_created {
            println!("{}", format!("   - {f}").dimmed());
        }
    } else {
        println!("{}", "═".repeat(60).red());
        println!(
            "{}",
            "❌ LOG FETCH FAILED - no files could be retrieved"
                .red()
                .bold()
        );
        println!("{}", "═".repeat(60).red());
        return Err(anyhow::anyhow!(
            "Failed to fetch any logs for pod {pod_name}"
        ));
    }
    Ok(())
}

/// Fetch current or previous logs for a pod.
fn fetch_log_type(
    pod_name: &str,
    namespace: &str,
    output_dir: &str,
    tail_arg: &str,
    log_type: &str,
    optional: bool,
) -> Result<usize> {
    println!("{}", "─".repeat(40));
    println!("{}", format!("📜 Fetching {log_type} logs...").yellow());

    let output_file = format!("{output_dir}/{pod_name}-{log_type}.log");
    let mut args = vec!["logs", pod_name, "-n", namespace, "--all-containers"];
    if log_type == "previous" {
        args.push("--previous");
    }
    if !tail_arg.is_empty() {
        args.push(tail_arg);
    }

    match fetch_kubectl_output(&args, &output_file) {
        Ok(size) => {
            println!(
                "{}",
                format!("   ✓ {log_type} logs: {size} bytes → {output_file}").green()
            );
            Ok(size)
        }
        Err(e) => {
            if optional {
                println!(
                    "{}",
                    format!("   ⚠ No {log_type} logs (pod may not have restarted)").yellow()
                );
            } else {
                println!("{}", format!("   ⚠ {log_type} logs failed: {e}").yellow());
            }
            Err(e)
        }
    }
}

/// Fetch pod events.
fn fetch_pod_events(pod_name: &str, namespace: &str, output_dir: &str) -> Result<usize> {
    println!("{}", "─".repeat(40));
    println!("{}", "📋 Fetching pod events...".yellow());

    let output_file = format!("{output_dir}/{pod_name}-events.yaml");
    let field_selector = format!("involvedObject.name={pod_name}");
    let args = [
        "get",
        "events",
        "-n",
        namespace,
        "--field-selector",
        &field_selector,
        "-o",
        "yaml",
    ];

    match fetch_kubectl_output(&args, &output_file) {
        Ok(size) => {
            println!(
                "{}",
                format!("   ✓ Events: {size} bytes → {output_file}").green()
            );
            Ok(size)
        }
        Err(e) => {
            println!("{}", format!("   ⚠ Events failed: {e}").yellow());
            Err(e)
        }
    }
}

/// Fetch pod describe output.
fn fetch_pod_describe(pod_name: &str, namespace: &str, output_dir: &str) -> Result<usize> {
    println!("{}", "─".repeat(40));
    println!("{}", "📝 Fetching pod description...".yellow());

    let output_file = format!("{output_dir}/{pod_name}-describe.txt");
    let args = ["describe", "pod", pod_name, "-n", namespace];

    match fetch_kubectl_output(&args, &output_file) {
        Ok(size) => {
            println!(
                "{}",
                format!("   ✓ Describe: {size} bytes → {output_file}").green()
            );
            Ok(size)
        }
        Err(e) => {
            println!("{}", format!("   ⚠ Describe failed: {e}").yellow());
            Err(e)
        }
    }
}

/// Helper to run kubectl and save output to a file.
fn fetch_kubectl_output(args: &[&str], output_file: &str) -> Result<usize> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .context("Failed to run kubectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("kubectl failed: {stderr}"));
    }

    let content = &output.stdout;
    std::fs::write(output_file, content).context(format!("Failed to write to {output_file}"))?;
    Ok(content.len())
}

// =============================================================================
// Play Command Handlers
// =============================================================================

/// Handle play orchestration commands.
#[allow(clippy::too_many_lines)]
fn handle_play_command(action: PlayCommands, namespace: &str) -> Result<()> {
    use play::cleanup::PlayCleanup;
    use play::{PlayBatch, PlayTracker};

    match action {
        PlayCommands::Status { task_id, stuck } => {
            let batch = PlayBatch::load_from_k8s(namespace)?;
            let tracker = PlayTracker::new(batch);

            if let Some(tid) = task_id {
                // Show specific task
                if let Some(task) = tracker.get_task(&tid) {
                    print_task_detail(task);
                } else {
                    println!("{}", format!("Task {tid} not found").red());
                }
            } else if stuck {
                // Show only stuck tasks
                let summary = tracker.health_summary();
                let stuck_tasks = tracker.batch.stuck_tasks();

                if stuck_tasks.is_empty() {
                    println!("{}", "No stuck tasks found".green());
                } else {
                    println!(
                        "{}",
                        format!("Found {} stuck tasks:", stuck_tasks.len())
                            .red()
                            .bold()
                    );
                    for task in stuck_tasks {
                        print_task_row(task);
                    }
                }

                // Also show issues
                for issue in &summary.issues {
                    println!("{}", format!("  Issue: {}", issue.description()).yellow());
                }
            } else {
                // Show full batch status
                print_batch_status(&tracker);
            }
        }
        PlayCommands::Remediate { task_id } => {
            let batch = PlayBatch::load_from_k8s(namespace)?;
            let tracker = PlayTracker::new(batch);

            // Find the task
            let task = tracker.get_task(&task_id);
            if task.is_none() {
                anyhow::bail!("Task {task_id} not found");
            }

            // Check health to get issues
            let issues = tracker.check_health();
            let task_issue = issues.iter().find(|i| i.task_id() == task_id);

            if let Some(issue) = task_issue {
                println!(
                    "{}",
                    format!("Spawning remediation for task {task_id}...").cyan()
                );

                // Run remediation
                let result = tracker.remediate(issue)?;

                println!("{}", "Remediation spawned:".green().bold());
                println!("  CodeRun: {}", result.coderun_name.cyan());
                println!("  Diagnosis: {}", result.diagnosis);
            } else {
                println!(
                    "{}",
                    format!("Task {task_id} has no active issues to remediate").yellow()
                );
            }
        }
        PlayCommands::Remediations => {
            // Query CodeRuns directly since batch state doesn't track remediations
            println!("{}", "Active Remediations:".cyan().bold());

            let output = Command::new("kubectl")
                .args([
                    "get", "coderuns",
                    "-n", namespace,
                    "-l", "app.kubernetes.io/name=healer,app.kubernetes.io/component=remediation",
                    "-o", "jsonpath={range .items[*]}{.metadata.name}|{.metadata.labels.task-id}|{.metadata.creationTimestamp}\\n{end}",
                ])
                .output()
                .context("Failed to query CodeRuns")?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

                if lines.is_empty() {
                    println!("{}", "  No active remediations".dimmed());
                } else {
                    for line in lines {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() >= 3 {
                            let name = parts[0];
                            let task_id = parts[1];
                            let created = parts[2];
                            println!(
                                "  Task {}: {} (created {})",
                                task_id.cyan(),
                                name.yellow(),
                                created
                            );
                        }
                    }
                }
            } else {
                // CodeRun CRD might not exist
                println!(
                    "{}",
                    "  No active remediations (or CodeRun CRD not installed)".dimmed()
                );
            }
        }
        PlayCommands::CancelRemediation { task_id } => {
            // Cancel by deleting the CodeRun
            println!(
                "{}",
                format!("Cancelling remediation for task {task_id}...").yellow()
            );

            let output = Command::new("kubectl")
                .args([
                    "delete",
                    "coderun",
                    "-n",
                    namespace,
                    "-l",
                    &format!("task-id={task_id},app.kubernetes.io/name=healer"),
                ])
                .output()
                .context("Failed to delete CodeRun")?;

            if output.status.success() {
                println!("{}", "Remediation cancelled".green());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("{}", format!("Failed to cancel: {stderr}").red());
            }
        }
        PlayCommands::Cleanup { force } => {
            let batch = PlayBatch::load_from_k8s(namespace)?;
            let cleanup = if force {
                PlayCleanup::new(namespace).force()
            } else {
                PlayCleanup::new(namespace)
            };

            println!("{}", "Cleaning up play state...".cyan());
            let report = cleanup.cleanup(&batch)?;

            println!("{}", format!("Cleanup complete: {report}").green());
        }
    }

    Ok(())
}

/// Print batch status overview.
fn print_batch_status(tracker: &play::PlayTracker) {
    let summary = tracker.health_summary();
    let batch = &tracker.batch;

    // Header
    println!("{}", "═".repeat(70).cyan());
    println!(
        "{}",
        format!("PLAY BATCH: {}", batch.project_name).cyan().bold()
    );
    println!("{}", "═".repeat(70).cyan());
    println!();

    // Batch info
    println!("  Repository: {}", batch.repository.yellow());
    println!(
        "  Started: {}",
        batch.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Elapsed: {}m", summary.elapsed_mins);
    println!();

    // Progress bar
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let progress_filled = (summary.progress / 100.0 * 30.0) as usize;
    let progress_empty = 30 - progress_filled;
    println!(
        "  Progress: [{}{}] {:.0}% ({}/{})",
        "█".repeat(progress_filled).green(),
        "░".repeat(progress_empty),
        summary.progress,
        summary.completed,
        summary.total
    );
    println!();

    // Task table header
    println!("  {:─<66}", "");
    println!(
        "  {:5} │ {:15} │ {:10} │ {:8} │ {:6} │ {:8}",
        "Task", "Stage", "Status", "Duration", "PR", "Health"
    );
    println!("  {:─<66}", "");

    // Task rows
    for task in &batch.tasks {
        print_task_row(task);
    }

    println!("  {:─<66}", "");

    // Alerts
    if !summary.issues.is_empty() {
        println!();
        for issue in &summary.issues {
            println!("{}", format!("  ⚠ {}", issue.description()).red());
        }
    }

    // Overall status
    println!();
    let status_color = match summary.status_str() {
        "Completed" => "green",
        "Critical" => "red",
        "Warning" => "yellow",
        _ => "white",
    };
    println!(
        "  Status: {}",
        match status_color {
            "green" => summary.status_str().green().bold(),
            "red" => summary.status_str().red().bold(),
            "yellow" => summary.status_str().yellow().bold(),
            _ => summary.status_str().white().bold(),
        }
    );
}

/// Print a single task row.
fn print_task_row(task: &play::TaskState) {
    let stage = task
        .current_stage()
        .map_or_else(|| "-".to_string(), |s| s.display_name().to_string());

    let status = match &task.status {
        play::types::TaskStatus::Pending => "Pending".dimmed(),
        play::types::TaskStatus::InProgress { .. } => "Active".cyan(),
        play::types::TaskStatus::Completed => "Done".green(),
        play::types::TaskStatus::Failed { .. } => "Failed".red(),
    };

    let duration = task
        .stage_duration()
        .map_or_else(|| "-".to_string(), |d| format!("{}m", d.num_minutes()));

    let pr = task
        .pr_number
        .map_or_else(|| "-".to_string(), |n| format!("#{n}"));

    let health = match task.health_indicator() {
        "healthy" => "OK".green(),
        "critical" => "CRIT".red(),
        "warning" => "WARN".yellow(),
        "pending" => "...".dimmed(),
        _ => "?".white(),
    };

    println!(
        "  {:5} │ {:15} │ {:10} │ {:8} │ {:6} │ {:8}",
        task.task_id, stage, status, duration, pr, health
    );
}

/// Print detailed task information.
fn print_task_detail(task: &play::TaskState) {
    println!("{}", "═".repeat(70).cyan());
    println!("{}", format!("TASK {} DETAIL", task.task_id).cyan().bold());
    println!("{}", "═".repeat(70).cyan());
    println!();

    // Stage info
    let stage = task
        .current_stage()
        .map_or_else(|| "-".to_string(), |s| s.display_name().to_string());
    let agent = task.current_stage().and_then(|s| s.agent()).unwrap_or("-");

    println!("  Stage: {} ({})", stage.yellow(), agent);

    // Duration
    if let Some(duration) = task.stage_duration() {
        let warn = if task.is_stuck() {
            " ⚠ OVER THRESHOLD"
        } else {
            ""
        };
        println!("  Duration: {}m{}", duration.num_minutes(), warn.red());
    }

    // CodeRun
    if let Some(ref cr) = task.active_coderun {
        println!("  CodeRun: {}", cr.cyan());
    }

    // Workflow
    if let Some(ref wf) = task.workflow_name {
        println!("  Workflow: {wf}");
    }

    // PR
    if let Some(pr) = task.pr_number {
        println!("  PR: #{pr}");
    }

    // Status details
    println!();
    match &task.status {
        play::types::TaskStatus::Failed {
            stage,
            reason,
            remediation,
        } => {
            println!("  {}", "Issue Detected:".red().bold());
            println!("    Type: Failed at {stage:?}");
            println!("    Reason: {reason}");

            if let Some(r) = remediation {
                println!();
                println!("  {}", "Remediation:".yellow().bold());
                println!("    Status: In Progress");
                println!("    CodeRun: {}", r.coderun_name.cyan());
                println!("    Diagnosis: {}", r.diagnosis);
                println!("    Started: {}", r.started_at.format("%H:%M:%S"));
            } else {
                println!();
                println!("  {}", "No active remediation".dimmed());
            }
        }
        play::types::TaskStatus::InProgress { .. } if task.is_stuck() => {
            println!("  {}", "Issue Detected:".red().bold());
            println!("    Type: StageTimeout");
            println!("    Stage: {stage}");
            println!("    Threshold: 30 minutes");
        }
        _ => {}
    }
}

// =============================================================================
// Insights Command Handlers
// =============================================================================

/// Handle insights commands.
fn handle_insights_command(action: InsightsCommands) -> Result<()> {
    use play::insights::InsightCollector;

    // For now, we use an in-memory collector
    // Future: load from persistent storage
    let collector = InsightCollector::new();

    match action {
        InsightsCommands::Show { agent } => {
            let stats = collector.agent_stats(&agent);

            println!("{}", "═".repeat(60).cyan());
            println!("{}", format!("AGENT INSIGHTS: {agent}").cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("  Runs Analyzed: {}", stats.runs_analyzed);
            println!("  Success Rate: {:.1}%", stats.success_rate);
            println!("  Avg Duration: {}m", stats.avg_duration_mins);
            println!();

            if stats.top_issues.is_empty() {
                println!("  {}", "No issues recorded yet".dimmed());
            } else {
                println!("  {}", "Top Issues:".yellow());
                for issue in &stats.top_issues {
                    println!(
                        "    • {} ({} occurrences)",
                        issue.description, issue.occurrences
                    );
                }
            }
        }
        InsightsCommands::Suggest => {
            let suggestions = collector.suggest_optimizations();

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "OPTIMIZATION SUGGESTIONS".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            if suggestions.is_empty() {
                println!(
                    "  {}",
                    "No suggestions yet - need more observations".dimmed()
                );
            } else {
                for (i, s) in suggestions.iter().enumerate() {
                    println!("  {}. {} ({})", i + 1, s.agent.yellow(), s.confidence);
                    println!("     Observation: {}", s.observation);
                    println!("     Suggestion: {}", s.suggested_change.green());
                    println!();
                }
            }
        }
        InsightsCommands::Failures => {
            let patterns = collector.failure_patterns();

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "COMMON FAILURE PATTERNS".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            if patterns.is_empty() {
                println!("  {}", "No failure patterns recorded yet".dimmed());
            } else {
                for p in &patterns {
                    println!(
                        "  {} │ {} │ {} occurrences",
                        p.agent.yellow(),
                        p.description,
                        p.occurrences
                    );
                }
            }
        }
        InsightsCommands::Export { format } => {
            let patterns = collector.failure_patterns();
            let suggestions = collector.suggest_optimizations();

            if format == "json" {
                let export = serde_json::json!({
                    "failure_patterns": patterns.iter().map(|p| {
                        serde_json::json!({
                            "agent": p.agent,
                            "description": p.description,
                            "occurrences": p.occurrences,
                        })
                    }).collect::<Vec<_>>(),
                    "suggestions": suggestions.iter().map(|s| {
                        serde_json::json!({
                            "agent": s.agent,
                            "observation": s.observation,
                            "suggested_change": s.suggested_change,
                            "confidence": s.confidence.to_string(),
                        })
                    }).collect::<Vec<_>>(),
                });
                println!("{}", serde_json::to_string_pretty(&export)?);
            } else {
                // CSV format
                println!("agent,type,description,count");
                for p in &patterns {
                    println!(
                        "{},failure,\"{}\",{}",
                        p.agent, p.description, p.occurrences
                    );
                }
            }
        }
    }

    Ok(())
}

// =============================================================================
// Server Command Handler
// =============================================================================

/// Run the CI remediation HTTP server.
async fn run_server_command(
    addr: &str,
    repository: &str,
    namespace: &str,
    config_path: Option<&str>,
) -> Result<()> {
    use std::sync::Arc;

    println!("{}", "═".repeat(60).cyan());
    println!("{}", "HEALER CI REMEDIATION SERVER".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    println!("  Address:    {}", addr.green());
    println!("  Repository: {}", repository.green());
    println!("  Namespace:  {}", namespace.green());
    println!();

    // Load configuration
    let config = if let Some(path) = config_path {
        let content = std::fs::read_to_string(path).context("Failed to read config file")?;
        serde_json::from_str(&content).context("Failed to parse config file")?
    } else {
        ci::RemediationConfig::default()
    };

    println!("  CLI:        {}", config.cli.cyan());
    println!("  Model:      {}", config.model.cyan());
    println!("  Max Attempts: {}", config.max_attempts);
    println!("  Time Window: {}m", config.time_window_mins);
    println!(
        "  Memory:     {}",
        if config.memory_enabled {
            "enabled".green()
        } else {
            "disabled".red()
        }
    );
    println!();

    // Create server state
    let state = ci::ServerState::new(config, repository, namespace)
        .context("Failed to initialize server state")?;
    let state = Arc::new(state);

    // Run the server
    println!("{}", format!("Starting server on {addr}...").cyan());
    ci::run_server(state, addr).await?;

    Ok(())
}

/// Run the log scanning command.
#[allow(clippy::too_many_lines)]
async fn run_scan_logs_command(
    window: &str,
    namespaces: &str,
    error_threshold: u32,
    warn_threshold: u32,
    output_format: &str,
    show_candidates: bool,
) -> Result<()> {
    use scanner::{format_report_text, LogScanner, ScannerConfig};

    // Parse window duration
    let window_duration = parse_duration(window)?;

    // Parse namespaces
    let namespace_list: Vec<String> = namespaces
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Only print header for non-JSON output (JSON must be clean for parsing)
    let is_json_output = output_format == "json";

    if !is_json_output {
        println!("{}", "═".repeat(60).cyan());
        println!("{}", "HEALER LOG SCANNER".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
        println!("  Window:          {}", window.green());
        println!("  Namespaces:      {}", namespace_list.join(", ").green());
        println!("  Error Threshold: {error_threshold}");
        println!("  Warn Threshold:  {warn_threshold}");
        println!();
    }

    // Create scanner
    let loki = loki::LokiClient::with_defaults();

    // Check Loki connectivity (only warn for non-JSON output)
    if !loki.health_check().await.unwrap_or(false) && !is_json_output {
        println!(
            "{}",
            "⚠️  Warning: Loki health check failed - results may be incomplete".yellow()
        );
    }

    let config = ScannerConfig {
        namespaces: namespace_list,
        error_threshold,
        warn_threshold,
        include_info: false,
    };

    let scanner = LogScanner::with_config(loki, config);

    if !is_json_output {
        println!("{}", "Scanning logs...".cyan());
    }
    let report = scanner.scan(window_duration).await?;

    // Output based on format
    match output_format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{json}");
        }
        "text" => {
            println!("{}", format_report_text(&report));
        }
        _ => {
            println!();
            println!(
                "  Total Errors:   {}",
                if report.total_errors > 0 {
                    report.total_errors.to_string().red()
                } else {
                    report.total_errors.to_string().green()
                }
            );
            println!(
                "  Total Warnings: {}",
                if report.total_warnings > 50 {
                    report.total_warnings.to_string().yellow()
                } else {
                    report.total_warnings.to_string().green()
                }
            );
            println!(
                "  Services with Issues: {}",
                report.services_with_issues.len()
            );
            println!();

            if !report.services_with_issues.is_empty() {
                println!("{}", "Services with Issues:".yellow());
                for issue in &report.services_with_issues {
                    let severity_icon = if issue.error_count >= 20 {
                        "🔴"
                    } else if issue.error_count >= 10 {
                        "🟠"
                    } else if issue.error_count >= 5 {
                        "🟡"
                    } else {
                        "⚪"
                    };
                    println!(
                        "  {} {}/{}: {} errors, {} warnings",
                        severity_icon,
                        issue.namespace,
                        issue.service,
                        issue.error_count,
                        issue.warn_count
                    );
                }
                println!();
            }

            if report.remediation_recommended {
                println!("{}", "⚠️  REMEDIATION RECOMMENDED".red().bold());
                if let Some(reason) = &report.recommendation_reason {
                    println!("    Reason: {reason}");
                }
            } else {
                println!("{}", "✓ No immediate remediation needed".green());
            }
        }
    }

    // Show remediation candidates if requested (skip for JSON output to keep it clean)
    if show_candidates && !report.services_with_issues.is_empty() && !is_json_output {
        println!();
        println!("{}", "Remediation Candidates:".yellow());
        let candidates = scanner.determine_candidates(&report);
        for candidate in candidates {
            println!(
                "  - {} {}/{} ({})",
                match candidate.severity.as_str() {
                    "critical" => "🔴",
                    "high" => "🟠",
                    "medium" => "🟡",
                    _ => "⚪",
                },
                candidate.namespace,
                candidate.service,
                candidate.severity
            );
            println!("    Agent: {}", candidate.suggested_agent.cyan());
            println!("    Reason: {}", candidate.reason);
        }
    }

    Ok(())
}

/// Run remediation from scan results.
///
/// Reads scan JSON from stdin, parses it, and spawns `CodeRuns` for services
/// that need remediation.
#[allow(clippy::too_many_lines)]
fn run_remediate_from_scan(config_path: &str, dry_run: bool, max_coderuns: usize) -> Result<()> {
    use scanner::{determine_agent_for_service, ScanReport};
    use std::io::{self, BufRead};

    println!("{}", "═".repeat(60).cyan());
    println!("{}", "🔧 REMEDIATE FROM SCAN".cyan().bold());
    println!("{}", "═".repeat(60).cyan());

    // Read JSON from stdin
    println!("{}", "📥 Reading scan results from stdin...".dimmed());
    let stdin = io::stdin();
    let mut json_input = String::new();
    for line in stdin.lock().lines() {
        json_input.push_str(&line?);
        json_input.push('\n');
    }

    if json_input.trim().is_empty() {
        println!("{}", "❌ No input received from stdin".red());
        return Err(anyhow::anyhow!("No scan JSON provided on stdin"));
    }

    // Parse scan report
    let report: ScanReport =
        serde_json::from_str(&json_input).context("Failed to parse scan JSON")?;

    println!("{}", format!("📊 Scan Time: {}", report.scan_time).dimmed());
    println!(
        "{}",
        format!(
            "   Window: {} minutes | Errors: {} | Warnings: {}",
            report.window_minutes, report.total_errors, report.total_warnings
        )
        .dimmed()
    );
    println!(
        "{}",
        format!(
            "   Services with issues: {}",
            report.services_with_issues.len()
        )
        .dimmed()
    );
    println!(
        "{}",
        format!(
            "   Remediation recommended: {}",
            report.remediation_recommended
        )
        .dimmed()
    );

    if !report.remediation_recommended {
        println!();
        println!("{}", "✓ No remediation needed".green());
        return Ok(());
    }

    if report.services_with_issues.is_empty() {
        println!();
        println!("{}", "✓ No services with issues to remediate".green());
        return Ok(());
    }

    // Load healer config
    let config = load_healer_config(config_path);
    println!(
        "{}",
        format!(
            "📋 Config: {} ({})",
            if std::path::Path::new(config_path).exists() {
                config_path
            } else {
                "defaults"
            },
            config.coderun.github_app
        )
        .dimmed()
    );

    // Determine which services need CodeRuns
    let mut coderuns_created = 0usize;
    let max = if max_coderuns == 0 {
        usize::MAX
    } else {
        max_coderuns
    };

    println!();
    println!(
        "{}",
        format!(
            "🎯 Processing {} services (max CodeRuns: {})",
            report.services_with_issues.len(),
            if max_coderuns == 0 {
                "unlimited".to_string()
            } else {
                max_coderuns.to_string()
            }
        )
        .yellow()
    );

    for issue in &report.services_with_issues {
        if coderuns_created >= max {
            println!(
                "{}",
                format!("⏭️  Max CodeRuns ({max_coderuns}) reached, stopping").yellow()
            );
            break;
        }

        let suggested_agent = determine_agent_for_service(&issue.service, &issue.sample_errors);
        let severity = if issue.error_count >= 50 {
            "critical"
        } else if issue.error_count >= 20 {
            "high"
        } else if issue.error_count >= 10 {
            "medium"
        } else {
            "low"
        };

        println!();
        println!(
            "{}",
            format!(
                "📦 {}/{}: {} errors, {} warnings [{}]",
                issue.namespace, issue.service, issue.error_count, issue.warn_count, severity
            )
            .cyan()
        );
        println!(
            "{}",
            format!("   Suggested agent: {suggested_agent}").dimmed()
        );

        if !issue.sample_errors.is_empty() {
            println!("{}", "   Sample errors:".dimmed());
            for (i, err) in issue.sample_errors.iter().take(3).enumerate() {
                let truncated = if err.len() > 80 {
                    format!("{}...", &err[..77])
                } else {
                    err.clone()
                };
                println!("   {}. {}", i + 1, truncated.dimmed());
            }
        }

        // Generate CodeRun name
        let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let sanitized_service = issue
            .service
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect::<String>()
            .to_lowercase();
        let coderun_name = format!(
            "scan-remediation-{}-{}-{uid}",
            issue.namespace, sanitized_service
        );

        // Build prompt with context
        let prompt = format!(
            r"# Log Scan Remediation

## Issue Detected
- **Service**: {service}
- **Namespace**: {namespace}
- **Error Count**: {error_count}
- **Warning Count**: {warn_count}
- **Severity**: {severity}
- **Scan Time**: {scan_time}

## Sample Errors
{sample_errors}

## Task
Investigate and fix the errors detected in the {service} service.

1. Analyze the error patterns above
2. Identify the root cause
3. Implement a fix
4. Create a PR with the changes

## Affected Pods
{affected_pods}
",
            service = issue.service,
            namespace = issue.namespace,
            error_count = issue.error_count,
            warn_count = issue.warn_count,
            severity = severity,
            scan_time = report.scan_time,
            sample_errors = issue
                .sample_errors
                .iter()
                .map(|e| format!("- {e}"))
                .collect::<Vec<_>>()
                .join("\n"),
            affected_pods = issue
                .affected_pods
                .iter()
                .map(|p| format!("- {p}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        if dry_run {
            println!(
                "{}",
                format!("   [DRY RUN] Would create CodeRun: {coderun_name}").yellow()
            );
            println!(
                "{}",
                format!("   [DRY RUN] Agent: {suggested_agent}").yellow()
            );
            coderuns_created += 1;
            continue;
        }

        // Build and apply CodeRun
        let coderun_yaml = build_scan_remediation_coderun(
            &coderun_name,
            &issue.namespace,
            &issue.service,
            &suggested_agent,
            &prompt,
            &timestamp,
            &config,
        );

        match apply_coderun(&coderun_yaml, &coderun_name) {
            Ok(()) => {
                coderuns_created += 1;
                println!(
                    "{}",
                    format!("   ✅ Created CodeRun: {coderun_name}").green()
                );
            }
            Err(e) => {
                println!("{}", format!("   ❌ Failed to create CodeRun: {e}").red());
            }
        }
    }

    println!();
    println!("{}", "═".repeat(60).green());
    if dry_run {
        println!(
            "{}",
            format!("🔍 DRY RUN: Would have created {coderuns_created} CodeRun(s)")
                .yellow()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!("✅ Created {coderuns_created} CodeRun(s)")
                .green()
                .bold()
        );
    }
    println!("{}", "═".repeat(60).green());

    Ok(())
}

/// Run issue reconciliation - check and close resolved issues.
fn run_reconcile_issues(
    repository: &str,
    namespace: &str,
    labels: &str,
    dry_run: bool,
    max_issues: usize,
    output_format: &str,
) -> Result<()> {
    use reconcile::{format_report_text, IssueReconciler, ReconcileConfig};

    let is_json_output = output_format == "json";

    if !is_json_output {
        println!("{}", "═".repeat(60).cyan());
        println!("{}", "🔄 ISSUE RECONCILIATION".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
    }

    // Parse labels
    let healer_labels: Vec<String> = labels
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let config = ReconcileConfig {
        repository: repository.to_string(),
        namespace: namespace.to_string(),
        healer_labels,
        dry_run,
        max_issues,
    };

    if !is_json_output {
        println!("{}", format!("Repository: {}", config.repository).dimmed());
        println!("{}", format!("Namespace: {}", config.namespace).dimmed());
        println!(
            "{}",
            format!("Dry run: {}", if dry_run { "yes" } else { "no" }).dimmed()
        );
        println!();
    }

    let reconciler = IssueReconciler::new(config);
    let report = reconciler.reconcile()?;

    // Output based on format
    if output_format == "json" {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{json}");
    } else {
        println!("{}", format_report_text(&report));

        // Summary
        println!();
        if report.issues_closed > 0 {
            println!(
                "{}",
                format!("✅ Closed {} resolved issues", report.issues_closed).green()
            );
        } else {
            println!("{}", "ℹ️  No issues to close".dimmed());
        }
    }

    Ok(())
}

/// Build `CodeRun` YAML for scan-based remediation.
#[allow(clippy::too_many_arguments)]
fn build_scan_remediation_coderun(
    coderun_name: &str,
    namespace: &str,
    service: &str,
    agent: &str,
    prompt: &str,
    timestamp: &impl std::fmt::Display,
    config: &HealerConfig,
) -> String {
    let c = &config.coderun;

    // Build optional fields only if non-empty
    let remote_tools_line = if c.remote_tools.is_empty() {
        String::new()
    } else {
        format!("  remoteTools: \"{}\"\n", c.remote_tools.join(","))
    };
    let local_tools_line = if c.local_tools.is_empty() {
        String::new()
    } else {
        format!("  localTools: \"{}\"\n", c.local_tools.join(","))
    };

    // Escape prompt for YAML (use multiline literal block)
    let prompt_escaped = prompt
        .lines()
        .map(|line| format!("      {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    // Hash service+namespace to numeric task ID
    let task_id_numeric: u32 = format!("{namespace}-{service}")
        .bytes()
        .fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(u32::from(b))
        });

    format!(
        r#"apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: {coderun_name}
  namespace: {cto_namespace}
  labels:
    agents.platform/type: scan-remediation
    scan-service: "{service}"
    scan-namespace: "{namespace}"
    suggested-agent: "{agent}"
    created-at: "{timestamp}"
spec:
  taskId: {task_id_numeric}
  runType: "{run_type}"
  githubApp: "{github_app}"
  model: "{model}"
  repositoryUrl: "{repository_url}"
  docsRepositoryUrl: "{docs_repository_url}"
  docsProjectDirectory: "{docs_project_directory}"
  docsBranch: "{docs_branch}"
  workingDirectory: "{working_directory}"
  service: "{cto_service}"
  enableDocker: {enable_docker}
{remote_tools}{local_tools}  cliConfig:
    cliType: "{cli_type}"
    model: "{cli_model}"
    settings:
      template: "{template}"
      memoryEnabled: {memory_enabled}
      memoryAgentName: "{memory_agent_name}"
      memorySessionPrefix: "{memory_session_prefix}"
      memoryRetrieveOnStart: {memory_retrieve_on_start}
      memoryPersistOnComplete: {memory_persist_on_complete}
  env:
    SCAN_SERVICE: "{service}"
    SCAN_NAMESPACE: "{namespace}"
    SUGGESTED_AGENT: "{agent}"
    REMEDIATION_MODE: "scan"
    SCAN_PROMPT: |
{prompt_escaped}
"#,
        cto_namespace = c.namespace,
        run_type = c.run_type,
        github_app = c.github_app,
        model = c.model,
        repository_url = c.repository_url,
        docs_repository_url = c.docs_repository_url,
        docs_project_directory = c.docs_project_directory,
        docs_branch = c.docs_branch,
        working_directory = c.working_directory,
        cto_service = c.service,
        enable_docker = c.enable_docker,
        remote_tools = remote_tools_line,
        local_tools = local_tools_line,
        cli_type = c.cli_config.cli_type,
        cli_model = c.cli_config.model,
        template = c.cli_config.settings.template,
        memory_enabled = c.memory.enabled,
        memory_agent_name = c.memory.agent_name,
        memory_session_prefix = c.memory.session_prefix,
        memory_retrieve_on_start = c.memory.retrieve_on_start,
        memory_persist_on_complete = c.memory.persist_on_complete,
        prompt_escaped = prompt_escaped,
    )
}

/// Parse a duration string like "1h", "30m", "2h30m".
fn parse_duration(s: &str) -> Result<chrono::Duration> {
    let s = s.trim().to_lowercase();

    if let Some(hours) = s.strip_suffix('h') {
        let h: i64 = hours.parse().context("Invalid hours")?;
        return Ok(chrono::Duration::hours(h));
    }

    if let Some(mins) = s.strip_suffix('m') {
        let m: i64 = mins.parse().context("Invalid minutes")?;
        return Ok(chrono::Duration::minutes(m));
    }

    // Try parsing as minutes by default
    let m: i64 = s.parse().context("Invalid duration format")?;
    Ok(chrono::Duration::minutes(m))
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
