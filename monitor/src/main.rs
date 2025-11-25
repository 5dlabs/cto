//! Play Monitor CLI
//!
//! A simple CLI tool for monitoring play workflows and retrieving failure logs.
//! Used by Cursor agent for E2E feedback loop automation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
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
    let victoria_logs_url = std::env::var("VICTORIA_LOGS_URL").unwrap_or_else(|_| {
        "http://victoria-logs-victoria-logs-single-server.telemetry.svc.cluster.local:9428"
            .to_string()
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
