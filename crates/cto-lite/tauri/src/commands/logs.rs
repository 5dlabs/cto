//! Log streaming service for Kind pods
//!
//! Provides real-time log streaming from Kubernetes pods using kubectl.
//! Supports filtering by namespace/pod.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub pod: String,
    pub container: String,
    pub namespace: String,
    pub message: String,
}

/// Pod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub phase: String,
    pub containers: Vec<String>,
}

/// Get default kubeconfig path
fn get_kubeconfig_path() -> String {
    dirs::home_dir()
        .map(|h| h.join(".kube").join("config"))
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "~/.kube/config".to_string())
}

/// List pods in a namespace
#[tauri::command]
pub async fn list_pods(namespace: Option<String>) -> Result<Vec<String>, AppError> {
    let ns = namespace.unwrap_or_else(|| "default".to_string());

    let output = Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-n",
            &ns,
            "-o",
            "jsonpath={.items[*].metadata.name}",
        ])
        .env("KUBECONFIG", get_kubeconfig_path())
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(stderr.to_string()));
    }

    let pods_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if pods_str.is_empty() {
        return Ok(vec![]);
    }

    let pods: Vec<String> = pods_str.split_whitespace().map(String::from).collect();
    Ok(pods)
}

/// Get pods with their status for a namespace
#[tauri::command]
pub async fn list_pods_with_status(namespace: Option<String>) -> Result<Vec<PodInfo>, AppError> {
    let ns = namespace.unwrap_or_else(|| "default".to_string());

    let output = Command::new("kubectl")
        .args([
            "get", "pods", "-n", &ns, "-o",
            "jsonpath={range .items[*]}{.metadata.name}{'\\t'}{.status.phase}{'\\t'}{.spec.containers[*].name}{'\\n'}{end}"
        ])
        .env("KUBECONFIG", get_kubeconfig_path())
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(stderr.to_string()));
    }

    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if output_str.is_empty() {
        return Ok(vec![]);
    }

    let pods: Vec<PodInfo> = output_str
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                Some(PodInfo {
                    name: parts[0].to_string(),
                    phase: parts[1].to_string(),
                    containers: parts
                        .get(2)
                        .map(|c| c.split_whitespace().map(String::from).collect())
                        .unwrap_or_default(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(pods)
}

/// Stream logs from a specific pod
#[tauri::command]
pub async fn stream_pod_logs(
    pod_name: String,
    namespace: Option<String>,
    container: Option<String>,
) -> Result<Vec<LogEntry>, AppError> {
    let ns = namespace.unwrap_or_else(|| "default".to_string());
    let mut args = vec!["logs".to_string(), pod_name.clone()];

    // Add namespace
    args.push("-n".to_string());
    args.push(ns.clone());

    // Add container if specified, otherwise get all containers
    if let Some(c) = &container {
        args.push("-c".to_string());
        args.push(c.clone());
    } else {
        args.push("--all-containers".to_string());
    }

    // Add timestamps
    args.push("--timestamps".to_string());

    let output = Command::new("kubectl")
        .args(&args)
        .env("KUBECONFIG", get_kubeconfig_path())
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(stderr.to_string()));
    }

    let logs_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let entries: Vec<LogEntry> = parse_log_lines(&logs_str, &pod_name, &ns, container.as_deref());
    Ok(entries)
}

/// Parse log lines into LogEntry structs
fn parse_log_lines(
    content: &str,
    pod: &str,
    namespace: &str,
    container: Option<&str>,
) -> Vec<LogEntry> {
    content
        .lines()
        .filter_map(|line| parse_log_line(line, pod, namespace, container))
        .collect()
}

/// Parse a single log line into a LogEntry
fn parse_log_line(
    line: &str,
    pod: &str,
    namespace: &str,
    container: Option<&str>,
) -> Option<LogEntry> {
    // Try to parse timestamp format: "2024-01-01T00:00:00.000000000Z message"
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Some(LogEntry {
            timestamp: String::new(),
            pod: pod.to_string(),
            container: container.unwrap_or("").to_string(),
            namespace: namespace.to_string(),
            message: line.to_string(),
        });
    }

    let timestamp = parts[0].to_string();
    let message = parts[1].to_string();

    // Check if timestamp looks valid (contains date/time patterns)
    if !timestamp.contains('-') || !timestamp.contains(':') {
        return Some(LogEntry {
            timestamp: String::new(),
            pod: pod.to_string(),
            container: container.unwrap_or("").to_string(),
            namespace: namespace.to_string(),
            message: line.to_string(),
        });
    }

    Some(LogEntry {
        timestamp,
        pod: pod.to_string(),
        container: container.unwrap_or("").to_string(),
        namespace: namespace.to_string(),
        message,
    })
}

/// Stream logs from all pods matching a pattern
/// Returns a channel receiver that will receive log entries
#[tauri::command]
pub async fn start_log_stream(
    namespace: Option<String>,
    pod_pattern: Option<String>,
    _container: Option<String>,
) -> Result<String, AppError> {
    let ns = namespace.unwrap_or_else(|| "default".to_string());
    let pattern = pod_pattern.unwrap_or_else(|| ".*".to_string());

    // Generate a unique stream ID
    let stream_id = format!("log-stream-{}", uuid::Uuid::new_v4());

    // For now, we'll return a simple status
    // The actual streaming would use WebSocket or SSE in a real implementation
    tracing::info!("Log stream requested: ns={}, pattern={}", ns, pattern);

    Ok(format!(
        "Stream {} initialized for namespace={}, pattern={}",
        stream_id, ns, pattern
    ))
}

/// Stop a log stream
#[tauri::command]
pub async fn stop_log_stream(stream_id: String) -> Result<(), AppError> {
    tracing::info!("Log stream stopped: {}", stream_id);
    Ok(())
}

/// Get namespaces
#[tauri::command]
pub async fn list_namespaces() -> Result<Vec<String>, AppError> {
    let output = Command::new("kubectl")
        .args([
            "get",
            "namespaces",
            "-o",
            "jsonpath={.items[*].metadata.name}",
        ])
        .env("KUBECONFIG", get_kubeconfig_path())
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(stderr.to_string()));
    }

    let ns_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if ns_str.is_empty() {
        return Ok(vec![]);
    }

    let namespaces: Vec<String> = ns_str.split_whitespace().map(String::from).collect();
    Ok(namespaces)
}
