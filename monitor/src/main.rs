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
    /// Get status of pods for a task
    Status {
        /// Task ID to monitor
        #[arg(long)]
        task_id: String,
    },
    /// Get logs for a task or specific pod
    Logs {
        /// Task ID to get logs for
        #[arg(long, group = "target")]
        task_id: Option<String>,

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
        /// Task ID to watch
        #[arg(long)]
        task_id: String,

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
        /// Task ID to run
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

/// Pod status information
#[derive(Debug, Serialize, Deserialize)]
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
    task_id: String,
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
    task_id: Option<String>,
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
        Commands::Status { task_id } => {
            let result = get_status(&task_id, &cli.namespace)?;
            output_result(&result, cli.format)?;
        }
        Commands::Logs {
            task_id,
            pod,
            tail,
            errors_only,
        } => {
            let result = get_logs(task_id, pod, &cli.namespace, tail, errors_only).await?;
            output_result(&result, cli.format)?;
        }
        Commands::Watch { task_id, interval } => {
            watch_status(&task_id, &cli.namespace, interval).await?;
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
            let result = run_workflow(&task_id, &repository, &agent, &template)?;
            output_result(&result, cli.format)?;
        }
    }

    Ok(())
}

/// Get status of pods for a task
fn get_status(task_id: &str, namespace: &str) -> Result<PlayStatusResponse> {
    debug!(
        "Getting status for task {} in namespace {}",
        task_id, namespace
    );

    // Query kubectl for pods matching the task ID
    let output = Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-n",
            namespace,
            "-l",
            &format!("task-id={task_id}"),
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
                task_id: task_id.to_string(),
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
        return parse_pods_by_pattern(&output.stdout, task_id, namespace);
    }

    parse_pods(&output.stdout, task_id, namespace)
}

/// Parse kubectl JSON output for pods
fn parse_pods(json_bytes: &[u8], task_id: &str, namespace: &str) -> Result<PlayStatusResponse> {
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
        task_id: task_id.to_string(),
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
    task_id: &str,
    namespace: &str,
) -> Result<PlayStatusResponse> {
    let pod_list: serde_json::Value =
        serde_json::from_slice(json_bytes).context("Failed to parse kubectl JSON output")?;

    let empty_vec = vec![];
    let items = pod_list["items"].as_array().unwrap_or(&empty_vec);

    // Filter pods that contain task ID in their name
    let task_pattern = format!("task-{task_id}");
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
        task_id: task_id.to_string(),
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

/// Get logs for a task or specific pod
async fn get_logs(
    task_id: Option<String>,
    pod: Option<String>,
    namespace: &str,
    tail: u32,
    errors_only: bool,
) -> Result<LogsResponse> {
    let logs = if let Some(pod_name) = &pod {
        // Get logs for specific pod
        get_pod_logs(pod_name, namespace, tail)?
    } else if let Some(task) = &task_id {
        // Get logs for all pods matching task ID
        get_task_logs(task, namespace, tail).await?
    } else {
        return Ok(LogsResponse {
            task_id: None,
            pod: None,
            namespace: namespace.to_string(),
            logs: String::new(),
            line_count: 0,
            timestamp: Utc::now(),
            error: Some("Must specify either --task-id or --pod".to_string()),
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
        task_id,
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

/// Get logs for all pods matching a task ID
async fn get_task_logs(task_id: &str, namespace: &str, tail: u32) -> Result<String> {
    // First get pod names
    let status = get_status(task_id, namespace)?;

    let mut all_logs = String::new();

    for pod in &status.pods {
        let pod_logs = get_pod_logs(&pod.name, namespace, tail)?;
        if !pod_logs.is_empty() {
            let _ = writeln!(all_logs, "\n=== Logs from {} ===", pod.name);
            all_logs.push_str(&pod_logs);
        }
    }

    // Also try to get logs from Victoria Logs if available
    if let Ok(victoria_logs) = get_victoria_logs(task_id, namespace, tail).await {
        if !victoria_logs.is_empty() {
            all_logs.push_str("\n=== Victoria Logs ===\n");
            all_logs.push_str(&victoria_logs);
        }
    }

    Ok(all_logs)
}

/// Query Victoria Logs API for historical logs
async fn get_victoria_logs(task_id: &str, namespace: &str, limit: u32) -> Result<String> {
    // Internal Kubernetes cluster service - HTTP is standard for in-cluster traffic
    // Set VICTORIA_LOGS_URL env var to override (e.g., for external/TLS endpoints)
    let victoria_logs_url = std::env::var("VICTORIA_LOGS_URL").unwrap_or_else(|_| {
        // codeql[rust/cleartext-transmission]: Internal K8s service, TLS not required
        String::from("http://victoria-logs-victoria-logs-single-server.telemetry.svc.cluster.local:9428")
    });

    let query = format!(
        r#"{{kubernetes_namespace="{namespace}", kubernetes_pod_name=~".*task-{task_id}.*"}}"#
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
async fn watch_status(task_id: &str, namespace: &str, interval: u64) -> Result<()> {
    let header = format!("Watching task {task_id} (interval: {interval}s)");
    println!("{}", header.cyan());
    println!("{}", "Press Ctrl+C to stop".dimmed());

    loop {
        let status = get_status(task_id, namespace)?;

        // Clear screen and print status
        print!("\x1B[2J\x1B[1;1H");
        let status_line = format!(
            "Task: {task_id} | Status: {} | Stage: {}",
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
    task_id: &str,
    repository: &str,
    agent: &str,
    template: &str,
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
            "argo",
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
