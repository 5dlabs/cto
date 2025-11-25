//! Play Monitor CLI
//!
//! A simple CLI tool for monitoring play workflows and retrieving failure logs.
//! Used by Cursor agent for E2E feedback loop automation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
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

    /// Kubernetes namespace to query
    #[arg(long, default_value = "agent-platform", global = true)]
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
    /// Get status of pods for a play
    Status {
        /// Play ID to monitor
        #[arg(long)]
        play_id: String,
    },
    /// Get logs for a play or specific pod
    Logs {
        /// Play ID to get logs for
        #[arg(long, group = "target")]
        play_id: Option<String>,

        /// Specific pod name to get logs for
        #[arg(long, group = "target")]
        pod: Option<String>,

        /// Number of log lines to retrieve
        #[arg(long, default_value = "500")]
        tail: u32,

        /// Filter for error patterns only
        #[arg(long)]
        errors_only: bool,
    },
    /// Watch play workflow status continuously
    Watch {
        /// Play ID to watch
        #[arg(long)]
        play_id: String,

        /// Poll interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
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
        /// Play ID to run
        #[arg(long)]
        play_id: String,

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
    /// Start the full monitoring loop - watches workflow and emits events
    Loop {
        /// Play ID to monitor
        #[arg(long)]
        play_id: String,

        /// Poll interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,

        /// Query `OpenMemory` for solutions on failure
        #[arg(long, default_value = "true")]
        query_memory: bool,

        /// Automatically fetch logs on failure
        #[arg(long, default_value = "true")]
        fetch_logs: bool,

        /// Stop after this many consecutive failures (0 = never stop)
        #[arg(long, default_value = "5")]
        max_failures: u32,
    },
}

/// Pod status information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PodStatus {
    name: String,
    phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    restarts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_status: Option<String>,
    age: String,
}

/// Overall play status response
#[derive(Debug, Serialize, Deserialize)]
struct PlayStatusResponse {
    play_id: String,
    namespace: String,
    status: String,
    stage: Option<String>,
    pods: Vec<PodStatus>,
    failed_pods: Vec<String>,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Logs response
#[derive(Debug, Serialize, Deserialize)]
struct LogsResponse {
    play_id: Option<String>,
    pod: Option<String>,
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
    play_id: String,
    repository: String,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// =============================================================================
// Loop Event Types
// =============================================================================

/// Event emitted by the monitoring loop
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type")]
enum LoopEvent {
    /// Loop has started monitoring
    #[serde(rename = "started")]
    Started {
        play_id: String,
        interval_seconds: u64,
        timestamp: DateTime<Utc>,
    },
    /// Status check completed
    #[serde(rename = "status")]
    Status {
        play_id: String,
        workflow_status: String,
        stage: Option<String>,
        pods: Vec<PodStatus>,
        timestamp: DateTime<Utc>,
    },
    /// Failure detected - includes logs and memory suggestions
    #[serde(rename = "failure")]
    Failure {
        play_id: String,
        stage: Option<String>,
        failed_pods: Vec<PodStatus>,
        logs: Option<String>,
        memory_suggestions: Vec<MemorySuggestion>,
        consecutive_failures: u32,
        timestamp: DateTime<Utc>,
    },
    /// Stage completed successfully
    #[serde(rename = "stage_complete")]
    StageComplete {
        play_id: String,
        stage: String,
        next_stage: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// All stages completed - workflow done
    #[serde(rename = "completed")]
    Completed {
        play_id: String,
        duration_seconds: i64,
        timestamp: DateTime<Utc>,
    },
    /// Loop stopped (max failures or manual stop)
    #[serde(rename = "stopped")]
    Stopped {
        play_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

/// Memory suggestion from `OpenMemory` query
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemorySuggestion {
    id: String,
    content: String,
    relevance_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_type: Option<String>,
}

// =============================================================================
// Memory Response Types
// =============================================================================

/// A single memory entry from `OpenMemory`
#[derive(Debug, Serialize, Deserialize)]
struct MemoryEntry {
    id: String,
    content: String,
    #[serde(default)]
    metadata: MemoryMetadata,
    #[serde(default)]
    salience: f64,
    #[serde(default)]
    reinforcements: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_accessed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(default)]
    score: f64,
}

/// Metadata associated with a memory
#[derive(Debug, Default, Serialize, Deserialize)]
struct MemoryMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    play_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_type: Option<String>,
    #[serde(default)]
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

/// Response for memory list command
#[derive(Debug, Serialize, Deserialize)]
struct MemoryListResponse {
    success: bool,
    memories: Vec<MemoryEntry>,
    total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<MemoryFilter>,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    play_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    limit: u32,
}

/// Response for memory query command
#[derive(Debug, Serialize, Deserialize)]
struct MemoryQueryResponse {
    success: bool,
    query: String,
    results: Vec<MemoryEntry>,
    total: usize,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Response for memory stats command
#[derive(Debug, Serialize, Deserialize)]
struct MemoryStatsResponse {
    success: bool,
    health: MemoryHealth,
    stats: MemoryStatistics,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryHealth {
    status: String,
    uptime_seconds: Option<i64>,
    database_size_bytes: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryStatistics {
    total_memories: i64,
    memories_by_agent: std::collections::HashMap<String, i64>,
    memories_by_pattern: std::collections::HashMap<String, i64>,
    average_salience: f64,
    recent_queries: i64,
    recent_additions: i64,
}

/// Response for memory get command
#[derive(Debug, Serialize, Deserialize)]
struct MemoryGetResponse {
    success: bool,
    memory: Option<MemoryEntry>,
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
        Commands::Status { play_id } => {
            let result = get_status(&play_id, &cli.namespace)?;
            output_result(&result, cli.format)?;
        }
        Commands::Logs {
            play_id,
            pod,
            tail,
            errors_only,
        } => {
            let result = get_logs(play_id, pod, &cli.namespace, tail, errors_only).await?;
            output_result(&result, cli.format)?;
        }
        Commands::Watch { play_id, interval } => {
            watch_status(&play_id, &cli.namespace, interval).await?;
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
            play_id,
            repository,
            agent,
            template,
        } => {
            let result = run_workflow(&play_id, &repository, &agent, &template)?;
            output_result(&result, cli.format)?;
        }
        Commands::Loop {
            play_id,
            interval,
            query_memory,
            fetch_logs,
            max_failures,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_monitoring_loop(
                &play_id,
                &cli.namespace,
                interval,
                query_memory,
                fetch_logs,
                max_failures,
            ))?;
        }
    }

    Ok(())
}

/// Get status of pods for a play
fn get_status(play_id: &str, namespace: &str) -> Result<PlayStatusResponse> {
    debug!(
        "Getting status for play {} in namespace {}",
        play_id, namespace
    );

    // Query kubectl for pods matching the task ID
    let output = Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-n",
            namespace,
            "-l",
            &format!("task-id={play_id}"),
            "-o",
            "json",
        ])
        .output()
        .context("Failed to execute kubectl")?;

    if !output.status.success() {
        // Try alternative label selector patterns
        let output = Command::new("kubectl")
            .args(["get", "pods", "-n", namespace, "-o", "json"])
            .output()
            .context("Failed to execute kubectl")?;

        if !output.status.success() {
            return Ok(PlayStatusResponse {
                play_id: play_id.to_string(),
                namespace: namespace.to_string(),
                status: "error".to_string(),
                stage: None,
                pods: vec![],
                failed_pods: vec![],
                timestamp: Utc::now(),
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }

        // Filter pods by name pattern containing task ID
        return parse_pods_by_pattern(&output.stdout, play_id, namespace);
    }

    parse_pods(&output.stdout, play_id, namespace)
}

/// Parse kubectl JSON output for pods
fn parse_pods(json_bytes: &[u8], play_id: &str, namespace: &str) -> Result<PlayStatusResponse> {
    let pod_list: serde_json::Value =
        serde_json::from_slice(json_bytes).context("Failed to parse kubectl JSON output")?;

    let empty_vec = vec![];
    let items = pod_list["items"].as_array().unwrap_or(&empty_vec);

    let mut pods = Vec::new();
    let mut failed_pods = Vec::new();
    let mut overall_status = "running";

    for item in items {
        let name = item["metadata"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let phase = item["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        // Get container status details
        let container_statuses = item["status"]["containerStatuses"].as_array();
        let (exit_code, reason, restarts, container_status) =
            extract_container_info(container_statuses);

        let creation_timestamp = item["metadata"]["creationTimestamp"].as_str().unwrap_or("");
        let age = calculate_age(creation_timestamp);

        // Check if pod is failed
        let is_failed = phase == "Failed"
            || reason.as_deref() == Some("Error")
            || reason.as_deref() == Some("OOMKilled")
            || reason.as_deref() == Some("CrashLoopBackOff")
            || exit_code.is_some_and(|c| c != 0);

        if is_failed {
            failed_pods.push(name.clone());
            overall_status = "failed";
        }

        pods.push(PodStatus {
            name,
            phase,
            exit_code,
            reason,
            restarts,
            container_status,
            age,
        });
    }

    // Determine stage from pod names
    let stage = determine_stage(&pods);

    // If no pods found, status is pending
    if pods.is_empty() {
        overall_status = "pending";
    } else if failed_pods.is_empty() && pods.iter().all(|p| p.phase == "Succeeded") {
        overall_status = "completed";
    }

    Ok(PlayStatusResponse {
        play_id: play_id.to_string(),
        namespace: namespace.to_string(),
        status: overall_status.to_string(),
        stage,
        pods,
        failed_pods,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Parse pods filtered by name pattern
fn parse_pods_by_pattern(
    json_bytes: &[u8],
    play_id: &str,
    namespace: &str,
) -> Result<PlayStatusResponse> {
    let pod_list: serde_json::Value =
        serde_json::from_slice(json_bytes).context("Failed to parse kubectl JSON output")?;

    let empty_vec = vec![];
    let items = pod_list["items"].as_array().unwrap_or(&empty_vec);

    // Filter pods that contain task ID in their name
    let task_pattern = format!("task-{play_id}");
    let filtered: Vec<&serde_json::Value> = items
        .iter()
        .filter(|item| {
            item["metadata"]["name"]
                .as_str()
                .is_some_and(|n| n.contains(&task_pattern))
        })
        .collect();

    let mut pods = Vec::new();
    let mut failed_pods = Vec::new();
    let mut overall_status = "running";

    for item in filtered {
        let name = item["metadata"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let phase = item["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        let container_statuses = item["status"]["containerStatuses"].as_array();
        let (exit_code, reason, restarts, container_status) =
            extract_container_info(container_statuses);

        let creation_timestamp = item["metadata"]["creationTimestamp"].as_str().unwrap_or("");
        let age = calculate_age(creation_timestamp);

        let is_failed = phase == "Failed"
            || reason.as_deref() == Some("Error")
            || reason.as_deref() == Some("OOMKilled")
            || reason.as_deref() == Some("CrashLoopBackOff")
            || exit_code.is_some_and(|c| c != 0);

        if is_failed {
            failed_pods.push(name.clone());
            overall_status = "failed";
        }

        pods.push(PodStatus {
            name,
            phase,
            exit_code,
            reason,
            restarts,
            container_status,
            age,
        });
    }

    let stage = determine_stage(&pods);

    if pods.is_empty() {
        overall_status = "pending";
    } else if failed_pods.is_empty() && pods.iter().all(|p| p.phase == "Succeeded") {
        overall_status = "completed";
    }

    Ok(PlayStatusResponse {
        play_id: play_id.to_string(),
        namespace: namespace.to_string(),
        status: overall_status.to_string(),
        stage,
        pods,
        failed_pods,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Extract container status information
fn extract_container_info(
    container_statuses: Option<&Vec<serde_json::Value>>,
) -> (Option<i32>, Option<String>, i32, Option<String>) {
    let mut exit_code = None;
    let mut reason = None;
    let mut restarts = 0;
    let mut container_status = None;

    if let Some(statuses) = container_statuses {
        for status in statuses {
            #[allow(clippy::cast_possible_truncation)]
            {
                restarts += status["restartCount"].as_i64().unwrap_or(0) as i32;
            }

            // Check terminated state
            if let Some(terminated) = status["state"]["terminated"].as_object() {
                #[allow(clippy::cast_possible_truncation)]
                {
                    exit_code = terminated["exitCode"].as_i64().map(|c| c as i32);
                }
                reason = terminated["reason"].as_str().map(ToString::to_string);
                container_status = Some("terminated".to_string());
            }
            // Check waiting state (CrashLoopBackOff, etc.)
            else if let Some(waiting) = status["state"]["waiting"].as_object() {
                reason = waiting["reason"].as_str().map(ToString::to_string);
                container_status = Some("waiting".to_string());
            }
            // Running state
            else if status["state"]["running"].is_object() {
                container_status = Some("running".to_string());
            }
        }
    }

    (exit_code, reason, restarts, container_status)
}

/// Determine current stage from pod names
fn determine_stage(pods: &[PodStatus]) -> Option<String> {
    for pod in pods {
        let name = pod.name.to_lowercase();
        if name.contains("rex") || name.contains("blaze") {
            return Some("implementation".to_string());
        }
        if name.contains("cleo") {
            return Some("code-quality".to_string());
        }
        if name.contains("cypher") {
            return Some("security".to_string());
        }
        if name.contains("tess") {
            return Some("qa".to_string());
        }
        if name.contains("atlas") {
            return Some("integration".to_string());
        }
    }
    None
}

/// Calculate age from creation timestamp
fn calculate_age(timestamp: &str) -> String {
    if timestamp.is_empty() {
        return "unknown".to_string();
    }

    if let Ok(created) = DateTime::parse_from_rfc3339(timestamp) {
        let duration = Utc::now().signed_duration_since(created.with_timezone(&Utc));
        let minutes = duration.num_minutes();
        let hours = duration.num_hours();
        let days = duration.num_days();

        if days > 0 {
            format!("{days}d")
        } else if hours > 0 {
            format!("{hours}h")
        } else {
            format!("{minutes}m")
        }
    } else {
        "unknown".to_string()
    }
}

/// Get logs for a play or specific pod
async fn get_logs(
    play_id: Option<String>,
    pod: Option<String>,
    namespace: &str,
    tail: u32,
    errors_only: bool,
) -> Result<LogsResponse> {
    let logs = if let Some(pod_name) = &pod {
        // Get logs for specific pod
        get_pod_logs(pod_name, namespace, tail)?
    } else if let Some(task) = &play_id {
        // Get logs for all pods matching task ID
        get_task_logs(task, namespace, tail).await?
    } else {
        return Ok(LogsResponse {
            play_id: None,
            pod: None,
            namespace: namespace.to_string(),
            logs: String::new(),
            line_count: 0,
            timestamp: Utc::now(),
            error: Some("Must specify either --play-id or --pod".to_string()),
        });
    };

    // Filter for errors if requested
    let filtered_logs = if errors_only {
        filter_error_logs(&logs)
    } else {
        logs
    };

    let line_count = filtered_logs.lines().count();

    Ok(LogsResponse {
        play_id,
        pod,
        namespace: namespace.to_string(),
        logs: filtered_logs,
        line_count,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Get logs for a specific pod via kubectl
fn get_pod_logs(pod_name: &str, namespace: &str, tail: u32) -> Result<String> {
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

    if !output.status.success() {
        // Try getting previous logs if current logs fail
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

        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get logs for all pods matching a play ID
async fn get_task_logs(play_id: &str, namespace: &str, tail: u32) -> Result<String> {
    // First get pod names
    let status = get_status(play_id, namespace)?;

    let mut all_logs = String::new();

    for pod in &status.pods {
        let pod_logs = get_pod_logs(&pod.name, namespace, tail)?;
        if !pod_logs.is_empty() {
            let _ = writeln!(all_logs, "\n=== Logs from {} ===", pod.name);
            all_logs.push_str(&pod_logs);
        }
    }

    // Also try to get logs from Victoria Logs if available
    if let Ok(victoria_logs) = get_victoria_logs(play_id, namespace, tail).await {
        if !victoria_logs.is_empty() {
            all_logs.push_str("\n=== Victoria Logs ===\n");
            all_logs.push_str(&victoria_logs);
        }
    }

    Ok(all_logs)
}

/// Query Victoria Logs API for historical logs
async fn get_victoria_logs(play_id: &str, namespace: &str, limit: u32) -> Result<String> {
    let victoria_logs_url = std::env::var("VICTORIA_LOGS_URL").unwrap_or_else(|_| {
        "http://victoria-logs-victoria-logs-single-server.telemetry.svc.cluster.local:9428"
            .to_string()
    });

    let query = format!(
        r#"{{kubernetes_namespace="{namespace}", kubernetes_pod_name=~".*task-{play_id}.*"}}"#
    );

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{victoria_logs_url}/select/logsql/query"))
        .form(&[("query", query.as_str()), ("limit", &limit.to_string())])
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let text = resp.text().await.unwrap_or_default();
            Ok(text)
        }
        Ok(resp) => {
            debug!("Victoria Logs returned status: {}", resp.status());
            Ok(String::new())
        }
        Err(e) => {
            debug!("Failed to query Victoria Logs: {}", e);
            Ok(String::new())
        }
    }
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
    ];

    logs.lines()
        .filter(|line| error_patterns.iter().any(|p| line.contains(p)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Watch status continuously
async fn watch_status(play_id: &str, namespace: &str, interval: u64) -> Result<()> {
    let header = format!("Watching play {play_id} (interval: {interval}s)");
    println!("{}", header.cyan());
    println!("{}", "Press Ctrl+C to stop".dimmed());

    loop {
        let status = get_status(play_id, namespace)?;

        // Clear screen and print status
        print!("\x1B[2J\x1B[1;1H");
        let status_line = format!(
            "Task: {play_id} | Status: {} | Stage: {}",
            colorize_status(&status.status),
            status.stage.as_deref().unwrap_or("unknown")
        );
        println!("{status_line}");
        let ns_line = format!("Namespace: {namespace} | Time: {}", status.timestamp);
        println!("{}", ns_line.dimmed());
        println!();

        for pod in &status.pods {
            let status_icon = match pod.phase.as_str() {
                "Running" => "●".green(),
                "Succeeded" => "✓".green(),
                "Failed" => "✗".red(),
                "Pending" => "○".yellow(),
                _ => "?".dimmed(),
            };
            println!(
                "  {status_icon} {} ({}) - {} restarts - {}",
                pod.name, pod.phase, pod.restarts, pod.age
            );
            if let Some(reason) = &pod.reason {
                println!("    └─ {}", reason.red());
            }
        }

        if !status.failed_pods.is_empty() {
            println!();
            println!("{}", "Failed pods:".red().bold());
            for pod in &status.failed_pods {
                println!("  - {pod}");
            }
        }

        // Check for completion or failure
        if status.status == "completed" {
            println!();
            println!("{}", "✓ Task completed successfully!".green().bold());
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }

    Ok(())
}

/// Colorize status string
fn colorize_status(status: &str) -> colored::ColoredString {
    match status {
        "running" => status.cyan(),
        "completed" => status.green(),
        "failed" => status.red(),
        "pending" => status.yellow(),
        _ => status.dimmed(),
    }
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
                    let _ = Command::new("kubectl")
                        .args(["delete", cm, "-n", namespace, "--force", "--grace-period=0"])
                        .output();
                    k8s_cleanup.configmaps_deleted += 1;
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
                    let _ = Command::new("kubectl")
                        .args([
                            "delete",
                            pvc,
                            "-n",
                            namespace,
                            "--force",
                            "--grace-period=0",
                        ])
                        .output();
                    k8s_cleanup.pvcs_deleted += 1;
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
        let _ = Command::new("git")
            .args(["add", "."])
            .current_dir(&temp_dir)
            .output();

        let _ = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&temp_dir)
            .output();

        let _ = Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(&temp_dir)
            .output();

        let _ = Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                &format!("git@github.com:{full_repo}.git"),
            ])
            .current_dir(&temp_dir)
            .output();

        let push = Command::new("git")
            .args(["push", "-u", "origin", "main", "--force"])
            .current_dir(&temp_dir)
            .output()?;

        if push.status.success() {
            result.pushed = true;
            println!("  {} Initialized repository", "✓".green());
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
    play_id: &str,
    repository: &str,
    agent: &str,
    template: &str,
) -> Result<RunResponse> {
    println!(
        "{}",
        format!("Submitting play workflow {play_id}...").cyan()
    );

    // Submit workflow using argo CLI
    let output = Command::new("argo")
        .args([
            "submit",
            "--from",
            &format!("workflowtemplate/{template}"),
            "-n",
            "argo",
            "-p",
            &format!("task-id={play_id}"),
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
            play_id: play_id.to_string(),
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
        play_id: play_id.to_string(),
        repository: repository.to_string(),
        timestamp: Utc::now(),
        error: None,
    })
}

// =============================================================================
// Monitoring Loop
// =============================================================================

/// Run the full monitoring loop - continuously watch workflow and emit events
#[allow(clippy::too_many_lines)]
async fn run_monitoring_loop(
    play_id: &str,
    namespace: &str,
    interval_seconds: u64,
    query_memory: bool,
    fetch_logs: bool,
    max_failures: u32,
) -> Result<()> {
    use std::io::Write;
    use tokio::time::{sleep, Duration};

    let start_time = Utc::now();
    let mut consecutive_failures: u32 = 0;
    let mut last_stage: Option<String> = None;
    let mut last_status = String::new();

    // Emit started event
    let started_event = LoopEvent::Started {
        play_id: play_id.to_string(),
        interval_seconds,
        timestamp: Utc::now(),
    };
    println!("{}", serde_json::to_string(&started_event)?);
    std::io::stdout().flush()?;

    loop {
        // Get current status
        let status_result = get_status(play_id, namespace);

        match status_result {
            Ok(status) => {
                let current_status = status.status.clone();
                let current_stage = status.stage.clone();

                // Check for stage change
                if current_stage != last_stage && current_stage.is_some() {
                    if let Some(ref prev_stage) = last_stage {
                        // Previous stage completed
                        let stage_complete = LoopEvent::StageComplete {
                            play_id: play_id.to_string(),
                            stage: prev_stage.clone(),
                            next_stage: current_stage.clone(),
                            timestamp: Utc::now(),
                        };
                        println!("{}", serde_json::to_string(&stage_complete)?);
                        std::io::stdout().flush()?;
                    }
                    last_stage.clone_from(&current_stage);
                }

                // Handle different statuses
                match current_status.as_str() {
                    "completed" => {
                        let duration = Utc::now().signed_duration_since(start_time).num_seconds();
                        let completed = LoopEvent::Completed {
                            play_id: play_id.to_string(),
                            duration_seconds: duration,
                            timestamp: Utc::now(),
                        };
                        println!("{}", serde_json::to_string(&completed)?);
                        std::io::stdout().flush()?;
                        return Ok(());
                    }
                    "failed" => {
                        consecutive_failures += 1;

                        // Get logs if enabled
                        let logs = if fetch_logs && !status.failed_pods.is_empty() {
                            let pod_name = &status.failed_pods[0];
                            get_logs_for_pod(pod_name, namespace, 200, true).ok()
                        } else {
                            None
                        };

                        // Query memory for suggestions if enabled
                        let memory_suggestions = if query_memory {
                            query_memory_for_error(logs.as_ref(), &status.failed_pods).await
                        } else {
                            vec![]
                        };

                        // Get failed pod details
                        let failed_pods: Vec<PodStatus> = status
                            .pods
                            .iter()
                            .filter(|p| status.failed_pods.contains(&p.name))
                            .cloned()
                            .collect();

                        let failure_event = LoopEvent::Failure {
                            play_id: play_id.to_string(),
                            stage: current_stage.clone(),
                            failed_pods,
                            logs,
                            memory_suggestions,
                            consecutive_failures,
                            timestamp: Utc::now(),
                        };
                        println!("{}", serde_json::to_string(&failure_event)?);
                        std::io::stdout().flush()?;

                        // Check if we should stop
                        if max_failures > 0 && consecutive_failures >= max_failures {
                            let stopped = LoopEvent::Stopped {
                                play_id: play_id.to_string(),
                                reason: format!(
                                    "Max consecutive failures reached ({max_failures})"
                                ),
                                timestamp: Utc::now(),
                            };
                            println!("{}", serde_json::to_string(&stopped)?);
                            std::io::stdout().flush()?;
                            return Ok(());
                        }
                    }
                    "running" | "pending" => {
                        // Reset failure count on successful status
                        if current_status != last_status {
                            consecutive_failures = 0;
                        }

                        // Emit status event periodically
                        let status_event = LoopEvent::Status {
                            play_id: play_id.to_string(),
                            workflow_status: current_status.clone(),
                            stage: current_stage.clone(),
                            pods: status.pods.clone(),
                            timestamp: Utc::now(),
                        };
                        println!("{}", serde_json::to_string(&status_event)?);
                        std::io::stdout().flush()?;
                    }
                    _ => {
                        // Unknown status - emit as status event
                        let status_event = LoopEvent::Status {
                            play_id: play_id.to_string(),
                            workflow_status: current_status.clone(),
                            stage: current_stage,
                            pods: status.pods.clone(),
                            timestamp: Utc::now(),
                        };
                        println!("{}", serde_json::to_string(&status_event)?);
                        std::io::stdout().flush()?;
                    }
                }

                last_status = current_status;
            }
            Err(e) => {
                // Error getting status - emit as failure
                let failure_event = LoopEvent::Failure {
                    play_id: play_id.to_string(),
                    stage: last_stage.clone(),
                    failed_pods: vec![],
                    logs: Some(format!("Error getting status: {e}")),
                    memory_suggestions: vec![],
                    consecutive_failures,
                    timestamp: Utc::now(),
                };
                println!("{}", serde_json::to_string(&failure_event)?);
                std::io::stdout().flush()?;
            }
        }

        // Wait for next poll
        sleep(Duration::from_secs(interval_seconds)).await;
    }
}

/// Query `OpenMemory` for suggestions based on error logs
async fn query_memory_for_error(
    logs: Option<&String>,
    failed_pods: &[String],
) -> Vec<MemorySuggestion> {
    let openmemory_url = get_openmemory_url();

    // Build query from logs and pod names
    let query_text = if let Some(log_content) = logs {
        // Extract error keywords from logs
        let error_lines: Vec<&str> = log_content
            .lines()
            .filter(|l| {
                l.contains("error")
                    || l.contains("Error")
                    || l.contains("ERROR")
                    || l.contains("failed")
                    || l.contains("Failed")
            })
            .take(3)
            .collect();
        error_lines.join(" ")
    } else {
        failed_pods.join(" ")
    };

    if query_text.is_empty() {
        return vec![];
    }

    let client = reqwest::Client::new();
    let query_json = serde_json::json!({
        "query": query_text,
        "k": 5,
        "include_waypoints": true
    });

    let response = client
        .post(format!("{openmemory_url}/memory/query"))
        .json(&query_json)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let results: Vec<serde_json::Value> = resp.json().await.unwrap_or_default();
            results
                .into_iter()
                .filter_map(|r| {
                    Some(MemorySuggestion {
                        id: r["id"].as_str()?.to_string(),
                        content: r["content"].as_str()?.to_string(),
                        relevance_score: r["score"].as_f64().unwrap_or(0.0),
                        agent: r["metadata"]["agent"].as_str().map(String::from),
                        pattern_type: r["metadata"]["pattern_type"].as_str().map(String::from),
                    })
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Get logs for a specific pod
fn get_logs_for_pod(
    pod_name: &str,
    namespace: &str,
    tail_lines: u32,
    errors_only: bool,
) -> Result<String> {
    let output = Command::new("kubectl")
        .args([
            "logs",
            pod_name,
            "-n",
            namespace,
            "--tail",
            &tail_lines.to_string(),
        ])
        .output()
        .context("Failed to get pod logs")?;

    let logs = String::from_utf8_lossy(&output.stdout).to_string();

    if errors_only {
        Ok(filter_error_logs(&logs))
    } else {
        Ok(logs)
    }
}

// =============================================================================
// OpenMemory Functions
// =============================================================================

/// Default `OpenMemory` URL - internal K8s service
const DEFAULT_OPENMEMORY_URL: &str = "http://openmemory:3000";

/// Get `OpenMemory` URL from environment or use default
fn get_openmemory_url() -> String {
    std::env::var("OPENMEMORY_URL").unwrap_or_else(|_| DEFAULT_OPENMEMORY_URL.to_string())
}

/// List recent memories from `OpenMemory`
#[allow(dead_code)]
async fn memory_list(
    play_id: Option<&str>,
    agent: Option<&str>,
    limit: u32,
) -> Result<MemoryListResponse> {
    let openmemory_url = get_openmemory_url();

    // Build filter JSON
    let mut filter_json = serde_json::json!({
        "limit": limit
    });

    if let Some(tid) = play_id {
        filter_json["metadata"] = serde_json::json!({ "play_id": tid });
    }

    if let Some(ag) = agent {
        if filter_json.get("metadata").is_some() {
            filter_json["metadata"]["agent"] = serde_json::json!(ag);
        } else {
            filter_json["metadata"] = serde_json::json!({ "agent": ag });
        }
    }

    let client = reqwest::Client::new();

    // Validate URL
    let _ = Url::parse(&openmemory_url).context("Invalid OPENMEMORY_URL")?;

    let response = client
        .post(format!("{openmemory_url}/memory/list"))
        .json(&filter_json)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let memories: Vec<MemoryEntry> = resp.json().await.unwrap_or_default();
            let total = memories.len();
            Ok(MemoryListResponse {
                success: true,
                memories,
                total,
                filter: Some(MemoryFilter {
                    play_id: play_id.map(String::from),
                    agent: agent.map(String::from),
                    limit,
                }),
                timestamp: Utc::now(),
                error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            Ok(MemoryListResponse {
                success: false,
                memories: vec![],
                total: 0,
                filter: None,
                timestamp: Utc::now(),
                error: Some(format!("OpenMemory returned {status}: {error_text}")),
            })
        }
        Err(e) => Ok(MemoryListResponse {
            success: false,
            memories: vec![],
            total: 0,
            filter: None,
            timestamp: Utc::now(),
            error: Some(format!("Failed to connect to OpenMemory: {e}")),
        }),
    }
}

/// Query memories semantically
#[allow(dead_code)]
async fn memory_query(
    text: &str,
    agent: Option<&str>,
    limit: u32,
    include_waypoints: bool,
) -> Result<MemoryQueryResponse> {
    let openmemory_url = get_openmemory_url();

    let mut query_json = serde_json::json!({
        "query": text,
        "k": limit,
        "include_waypoints": include_waypoints
    });

    if let Some(ag) = agent {
        query_json["agent"] = serde_json::json!(ag);
    }

    let client = reqwest::Client::new();

    // Validate URL
    let _ = Url::parse(&openmemory_url).context("Invalid OPENMEMORY_URL")?;

    let response = client
        .post(format!("{openmemory_url}/memory/query"))
        .json(&query_json)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let results: Vec<MemoryEntry> = resp.json().await.unwrap_or_default();
            let total = results.len();
            Ok(MemoryQueryResponse {
                success: true,
                query: text.to_string(),
                results,
                total,
                timestamp: Utc::now(),
                error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            Ok(MemoryQueryResponse {
                success: false,
                query: text.to_string(),
                results: vec![],
                total: 0,
                timestamp: Utc::now(),
                error: Some(format!("OpenMemory returned {status}: {error_text}")),
            })
        }
        Err(e) => Ok(MemoryQueryResponse {
            success: false,
            query: text.to_string(),
            results: vec![],
            total: 0,
            timestamp: Utc::now(),
            error: Some(format!("Failed to connect to OpenMemory: {e}")),
        }),
    }
}

/// Get memory statistics and health
#[allow(dead_code)]
async fn memory_stats(agent: Option<&str>) -> Result<MemoryStatsResponse> {
    let openmemory_url = get_openmemory_url();
    let client = reqwest::Client::new();

    // Validate URL
    let _ = Url::parse(&openmemory_url).context("Invalid OPENMEMORY_URL")?;

    // Get health status
    let health_response = client
        .get(format!("{openmemory_url}/health"))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    let health = match health_response {
        Ok(resp) if resp.status().is_success() => {
            let health_json: serde_json::Value = resp.json().await.unwrap_or_default();
            MemoryHealth {
                status: health_json["status"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                uptime_seconds: health_json["uptime_seconds"].as_i64(),
                database_size_bytes: health_json["database_size_bytes"].as_i64(),
            }
        }
        _ => MemoryHealth {
            status: "unreachable".to_string(),
            uptime_seconds: None,
            database_size_bytes: None,
        },
    };

    // Get statistics
    let mut stats_url = format!("{openmemory_url}/memory/stats");
    if let Some(ag) = agent {
        stats_url = format!("{stats_url}?agent={ag}");
    }

    let stats_response = client
        .get(&stats_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    let stats = match stats_response {
        Ok(resp) if resp.status().is_success() => {
            let stats_json: serde_json::Value = resp.json().await.unwrap_or_default();
            MemoryStatistics {
                total_memories: stats_json["total_memories"].as_i64().unwrap_or(0),
                memories_by_agent: stats_json["by_agent"]
                    .as_object()
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| (k.clone(), v.as_i64().unwrap_or(0)))
                            .collect()
                    })
                    .unwrap_or_default(),
                memories_by_pattern: stats_json["by_pattern"]
                    .as_object()
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| (k.clone(), v.as_i64().unwrap_or(0)))
                            .collect()
                    })
                    .unwrap_or_default(),
                average_salience: stats_json["average_salience"].as_f64().unwrap_or(0.0),
                recent_queries: stats_json["recent_queries"].as_i64().unwrap_or(0),
                recent_additions: stats_json["recent_additions"].as_i64().unwrap_or(0),
            }
        }
        _ => MemoryStatistics {
            total_memories: 0,
            memories_by_agent: std::collections::HashMap::new(),
            memories_by_pattern: std::collections::HashMap::new(),
            average_salience: 0.0,
            recent_queries: 0,
            recent_additions: 0,
        },
    };

    Ok(MemoryStatsResponse {
        success: health.status != "unreachable",
        health,
        stats,
        timestamp: Utc::now(),
        error: None,
    })
}

/// Get a specific memory by ID
#[allow(dead_code)]
async fn memory_get(id: &str) -> Result<MemoryGetResponse> {
    let openmemory_url = get_openmemory_url();
    let client = reqwest::Client::new();

    // Validate URL
    let _ = Url::parse(&openmemory_url).context("Invalid OPENMEMORY_URL")?;

    let response = client
        .get(format!("{openmemory_url}/memory/{id}"))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let memory: MemoryEntry = resp.json().await?;
            Ok(MemoryGetResponse {
                success: true,
                memory: Some(memory),
                timestamp: Utc::now(),
                error: None,
            })
        }
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => Ok(MemoryGetResponse {
            success: false,
            memory: None,
            timestamp: Utc::now(),
            error: Some(format!("Memory not found: {id}")),
        }),
        Ok(resp) => {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            Ok(MemoryGetResponse {
                success: false,
                memory: None,
                timestamp: Utc::now(),
                error: Some(format!("OpenMemory returned {status}: {error_text}")),
            })
        }
        Err(e) => Ok(MemoryGetResponse {
            success: false,
            memory: None,
            timestamp: Utc::now(),
            error: Some(format!("Failed to connect to OpenMemory: {e}")),
        }),
    }
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
    fn test_determine_stage() {
        let pods = vec![PodStatus {
            name: "rex-task-42-abc123".to_string(),
            phase: "Running".to_string(),
            exit_code: None,
            reason: None,
            restarts: 0,
            container_status: Some("running".to_string()),
            age: "5m".to_string(),
        }];

        assert_eq!(determine_stage(&pods), Some("implementation".to_string()));

        let pods = vec![PodStatus {
            name: "cleo-task-42-xyz789".to_string(),
            phase: "Running".to_string(),
            exit_code: None,
            reason: None,
            restarts: 0,
            container_status: Some("running".to_string()),
            age: "3m".to_string(),
        }];

        assert_eq!(determine_stage(&pods), Some("code-quality".to_string()));
    }

    #[test]
    fn test_calculate_age() {
        // Test with empty string
        assert_eq!(calculate_age(""), "unknown");

        // Test with invalid format
        assert_eq!(calculate_age("not-a-date"), "unknown");
    }
}
