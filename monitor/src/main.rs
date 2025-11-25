//! Play Monitor CLI
//!
//! A simple CLI tool for monitoring play workflows and retrieving failure logs.
//! Uses Argo Workflows CLI for efficient event-driven monitoring.
//! Used by Cursor agent for E2E feedback loop automation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::process::Command;
use tracing::debug;

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
    #[arg(long, default_value = "argo", global = true)]
    namespace: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Json,
    Text,
}

#[derive(Subcommand)]
enum Commands {
    /// [PRIMARY] Full E2E loop: start play, monitor, remediate until completion
    Full {
        /// Task ID for the play
        #[arg(long)]
        task_id: String,

        /// Path to cto-config.json (reads play configuration)
        #[arg(long, default_value = "cto-config.json")]
        config: String,

        /// Poll interval in seconds
        #[arg(long, default_value = "10")]
        interval: u64,

        /// Max consecutive failures before stopping (0 = unlimited)
        #[arg(long, default_value = "5")]
        max_failures: u32,

        /// Workflow template name
        #[arg(long, default_value = "play-workflow-template")]
        template: String,
    },
    /// Monitor an existing play workflow - emits JSON events
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

        /// Query `OpenMemory` for solutions on failure
        #[arg(long, default_value = "true")]
        query_memory: bool,

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

        /// Implementation agent (Rex or Blaze)
        #[arg(long, default_value = "5DLabs-Rex")]
        agent: String,

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
// Loop Events - JSON events emitted by the loop command
// =============================================================================

/// Events emitted by the loop command
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
enum LoopEvent {
    /// Loop has started monitoring
    Started {
        play_id: String,
        interval_seconds: u64,
        timestamp: DateTime<Utc>,
    },
    /// Current workflow status update
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
        memory_suggestions: Vec<MemorySuggestion>,
        consecutive_failures: u32,
        timestamp: DateTime<Utc>,
    },
    /// Workflow completed successfully
    Completed {
        play_id: String,
        duration_seconds: i64,
        timestamp: DateTime<Utc>,
    },
    /// Loop stopped (max failures reached or user interrupt)
    Stopped {
        play_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

/// Memory suggestion from `OpenMemory` query
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemorySuggestion {
    content: String,
    relevance_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing if verbose
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("play_monitor=debug")
            .init();
    }

    match cli.command {
        Commands::Full {
            task_id,
            config,
            interval,
            max_failures,
            template,
        } => {
            run_full_loop(&task_id, &config, &cli.namespace, interval, max_failures, &template)
                .await?;
        }
        Commands::Loop {
            play_id,
            interval,
            fetch_logs,
            query_memory,
            max_failures,
            log_tail,
        } => {
            run_loop(
                &play_id,
                &cli.namespace,
                interval,
                fetch_logs,
                query_memory,
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
            agent,
            template,
        } => {
            let result = run_workflow(&task_id, &repository, &agent, &template, &cli.namespace)?;
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
    debug!("Getting workflow status for {} in {}", workflow_name, namespace);

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

/// Run the full E2E loop: load config, start workflow, monitor until completion
#[allow(clippy::too_many_arguments)]
async fn run_full_loop(
    task_id: &str,
    config_path: &str,
    namespace: &str,
    interval: u64,
    max_failures: u32,
    template: &str,
) -> Result<()> {
    // Step 1: Read and parse cto-config.json
    let config_content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    let config: CtoConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {config_path}"))?;

    let play = &config.defaults.play;

    // Emit config loaded event
    emit_event(&LoopEvent::Started {
        play_id: format!("play-{task_id}"),
        interval_seconds: interval,
        timestamp: Utc::now(),
    })?;

    println!(
        "{}",
        format!(
            "Loaded config: repo={}, impl={}, quality={}, testing={}",
            play.repository, play.implementation_agent, play.quality_agent, play.testing_agent
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
            namespace,
            "-p",
            &format!("task-id={task_id}"),
            "-p",
            &format!("repository={}", play.repository),
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

    // Step 3: Start monitoring loop
    println!(
        "{}",
        format!("Starting monitoring loop for {workflow_name}...").cyan()
    );

    run_loop(
        &workflow_name,
        namespace,
        interval,
        true,  // fetch_logs
        true,  // query_memory
        max_failures,
        500,   // log_tail
    )
    .await
}

/// Run the monitoring loop - emits JSON events
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn run_loop(
    play_id: &str,
    namespace: &str,
    interval: u64,
    fetch_logs: bool,
    query_memory: bool,
    max_failures: u32,
    log_tail: u32,
) -> Result<()> {
    // Emit started event
    emit_event(&LoopEvent::Started {
        play_id: play_id.to_string(),
        interval_seconds: interval,
        timestamp: Utc::now(),
    })?;

    let mut consecutive_failures: u32 = 0;
    let mut last_stage: Option<String> = None;
    let mut last_phase = String::new();

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
                    memory_suggestions: vec![],
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
                memory_suggestions: vec![],
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

        // Handle workflow failure
        if status.phase == "Failed" || status.phase == "Error" || !status.failed_steps.is_empty() {
            consecutive_failures += 1;

            // Get the first failed step
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

            // Query memory if enabled
            let memory_suggestions = if query_memory {
                query_openmemory_for_error(logs.as_deref(), failed_step.as_ref()).await
            } else {
                vec![]
            };

            emit_event(&LoopEvent::Failure {
                play_id: play_id.to_string(),
                stage: status.stage.clone(),
                failed_step,
                logs,
                memory_suggestions,
                consecutive_failures,
                timestamp: Utc::now(),
            })?;

            // Check max failures
            if max_failures > 0 && consecutive_failures >= max_failures {
                emit_event(&LoopEvent::Stopped {
                    play_id: play_id.to_string(),
                    reason: format!("Max consecutive failures reached ({max_failures})"),
                    timestamp: Utc::now(),
                })?;
                return Ok(());
            }
        } else {
            // Reset consecutive failures on successful status
            consecutive_failures = 0;
        }

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
// OpenMemory Integration
// =============================================================================

/// Query `OpenMemory` for solutions to an error
async fn query_openmemory_for_error(
    logs: Option<&str>,
    failed_step: Option<&WorkflowStep>,
) -> Vec<MemorySuggestion> {
    // Extract error message for query
    let query = build_memory_query(logs, failed_step);
    if query.is_empty() {
        return vec![];
    }

    // Get OpenMemory URL from env
    let openmemory_url = std::env::var("OPENMEMORY_URL")
        .unwrap_or_else(|_| "http://openmemory.openmemory.svc.cluster.local:8080".to_string());

    debug!("Querying OpenMemory: {}", query);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{openmemory_url}/api/v1/search"))
        .json(&serde_json::json!({
            "query": query,
            "limit": 5
        }))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                parse_memory_response(&json)
            } else {
                vec![]
            }
        }
        Ok(resp) => {
            debug!("OpenMemory returned status: {}", resp.status());
            vec![]
        }
        Err(e) => {
            debug!("Failed to query OpenMemory: {}", e);
            vec![]
        }
    }
}

/// Build a query from logs and failed step info
fn build_memory_query(logs: Option<&str>, failed_step: Option<&WorkflowStep>) -> String {
    let mut query_parts = Vec::new();

    // Add step name/stage context
    if let Some(step) = failed_step {
        query_parts.push(format!("workflow step {} failed", step.name));
        if let Some(ref msg) = step.message {
            query_parts.push(msg.clone());
        }
    }

    // Extract key error from logs
    if let Some(log_text) = logs {
        let errors = filter_error_logs(log_text);
        // Take first few error lines
        let key_errors: Vec<&str> = errors.lines().take(3).collect();
        if !key_errors.is_empty() {
            query_parts.push(key_errors.join(" "));
        }
    }

    query_parts.join(" ").chars().take(500).collect()
}

/// Parse `OpenMemory` response into suggestions
fn parse_memory_response(json: &serde_json::Value) -> Vec<MemorySuggestion> {
    let mut suggestions = Vec::new();

    if let Some(results) = json["results"].as_array() {
        for result in results {
            if let Some(content) = result["content"].as_str() {
                suggestions.push(MemorySuggestion {
                    content: content.to_string(),
                    relevance_score: result["score"].as_f64().unwrap_or(0.0),
                    source: result["source"].as_str().map(ToString::to_string),
                });
            }
        }
    }

    suggestions
}

/// Handle memory subcommands
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
                .map_or_else(std::string::ToString::to_string, |o| String::from_utf8_lossy(&o.stderr).to_string());
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
fn run_workflow(
    task_id: &str,
    repository: &str,
    agent: &str,
    template: &str,
    namespace: &str,
) -> Result<RunResponse> {
    println!(
        "{}",
        format!("Submitting play workflow for task {task_id}...").cyan()
    );

    // Submit workflow using argo CLI
    let output = Command::new("argo")
        .args([
            "submit",
            "--from",
            &format!("workflowtemplate/{template}"),
            "-n",
            namespace,
            "-p",
            &format!("task-id={task_id}"),
            "-p",
            &format!("repository={repository}"),
            "-p",
            &format!("implementation-agent={agent}"),
            "-p",
            "quality-agent=5DLabs-Cleo",
            "-p",
            "testing-agent=5DLabs-Tess",
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
            task_id: task_id.to_string(),
            repository: repository.to_string(),
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
        task_id: task_id.to_string(),
        repository: repository.to_string(),
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
        let duration = calculate_duration(
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-01T00:05:00Z"),
        );
        assert_eq!(duration, Some(300)); // 5 minutes = 300 seconds

        // Missing start time
        let duration = calculate_duration(None, Some("2024-01-01T00:05:00Z"));
        assert_eq!(duration, None);

        // Invalid format
        let duration = calculate_duration(Some("invalid"), Some("2024-01-01T00:05:00Z"));
        assert_eq!(duration, None);
    }

    #[test]
    fn test_build_memory_query() {
        let step = WorkflowStep {
            id: "test".to_string(),
            name: "cleo-quality".to_string(),
            step_type: "Pod".to_string(),
            phase: "Failed".to_string(),
            pod_name: None,
            exit_code: Some(1),
            message: Some("clippy::uninlined_format_args".to_string()),
            started_at: None,
            finished_at: None,
        };

        let query = build_memory_query(None, Some(&step));
        assert!(query.contains("cleo-quality"));
        assert!(query.contains("clippy::uninlined_format_args"));
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

