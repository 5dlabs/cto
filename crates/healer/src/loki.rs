//! Loki client for querying historical pod logs
//!
//! Provides functions to query Grafana Loki for pod logs, enabling heal to retrieve
//! historical logs even after pods have been garbage collected.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use heal::loki::{LokiClient, LokiConfig};
//! use chrono::{Utc, Duration};
//!
//! let client = LokiClient::new(LokiConfig::default());
//!
//! // Query logs for a specific pod
//! let logs = client.query_pod_logs(
//!     "cto",
//!     "play-task-4-abc-step-123",
//!     Utc::now() - Duration::minutes(10),
//!     Utc::now(),
//!     1000,
//! ).await?;
//!
//! for entry in logs {
//!     println!("[{}] {}", entry.timestamp, entry.line);
//! }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{debug, warn};

/// Default Loki service URL (internal Kubernetes DNS)
const DEFAULT_LOKI_URL: &str = "http://loki.logging.svc.cluster.local:3100";

/// Configuration for the Loki client
#[derive(Debug, Clone)]
pub struct LokiConfig {
    /// Base URL for the Loki API
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of log entries to return per query
    pub default_limit: u32,
}

impl Default for LokiConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("LOKI_URL").unwrap_or_else(|_| DEFAULT_LOKI_URL.to_string()),
            timeout_secs: 30,
            default_limit: 5000,
        }
    }
}

/// A single log entry from Loki
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// The log line content
    pub line: String,
    /// Labels associated with this log entry
    pub labels: std::collections::HashMap<String, String>,
}

/// Loki query response structure
#[derive(Debug, Deserialize)]
struct LokiResponse {
    status: String,
    data: LokiData,
}

#[derive(Debug, Deserialize)]
struct LokiData {
    #[serde(rename = "resultType")]
    #[allow(dead_code)]
    result_type: String,
    result: Vec<LokiStream>,
}

#[derive(Debug, Deserialize)]
struct LokiStream {
    stream: std::collections::HashMap<String, String>,
    values: Vec<(String, String)>, // (timestamp_ns, line)
}

/// Client for querying Grafana Loki
#[derive(Debug, Clone)]
pub struct LokiClient {
    config: LokiConfig,
    client: reqwest::Client,
}

impl LokiClient {
    /// Create a new Loki client with the given configuration
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be created (should never happen in practice).
    #[must_use]
    pub fn new(config: LokiConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Create a new Loki client with default configuration
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be created (should never happen in practice).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(LokiConfig::default())
    }

    /// Query logs for a specific pod within a time range
    ///
    /// # Arguments
    /// * `namespace` - Kubernetes namespace
    /// * `pod_name` - Name of the pod
    /// * `start` - Start time for the query
    /// * `end` - End time for the query
    /// * `limit` - Maximum number of entries to return (0 = use default)
    ///
    /// # Returns
    /// A vector of log entries sorted by timestamp (oldest first)
    ///
    /// # Errors
    /// Returns an error if the Loki query fails or returns an invalid response.
    pub async fn query_pod_logs(
        &self,
        namespace: &str,
        pod_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<LogEntry>> {
        let query = format!(r#"{{namespace="{namespace}", pod="{pod_name}"}}"#);
        self.query_logs(&query, start, end, limit).await
    }

    /// Query logs for a container within a specific pod
    ///
    /// # Arguments
    /// * `namespace` - Kubernetes namespace
    /// * `pod_name` - Name of the pod
    /// * `container_name` - Name of the container
    /// * `start` - Start time for the query
    /// * `end` - End time for the query
    /// * `limit` - Maximum number of entries to return (0 = use default)
    ///
    /// # Errors
    /// Returns an error if the Loki query fails or returns an invalid response.
    pub async fn query_container_logs(
        &self,
        namespace: &str,
        pod_name: &str,
        container_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<LogEntry>> {
        let query = format!(
            r#"{{namespace="{namespace}", pod="{pod_name}", container="{container_name}"}}"#
        );
        self.query_logs(&query, start, end, limit).await
    }

    /// Query logs for all pods matching a label selector
    ///
    /// # Arguments
    /// * `namespace` - Kubernetes namespace
    /// * `label_key` - Label key to match
    /// * `label_value` - Label value to match
    /// * `start` - Start time for the query
    /// * `end` - End time for the query
    /// * `limit` - Maximum number of entries to return (0 = use default)
    ///
    /// # Errors
    /// Returns an error if the Loki query fails or returns an invalid response.
    pub async fn query_logs_by_label(
        &self,
        namespace: &str,
        label_key: &str,
        label_value: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<LogEntry>> {
        let query = format!(r#"{{namespace="{namespace}", {label_key}="{label_value}"}}"#);
        self.query_logs(&query, start, end, limit).await
    }

    /// Query logs for a workflow (all pods matching the workflow name prefix)
    ///
    /// # Arguments
    /// * `namespace` - Kubernetes namespace
    /// * `workflow_name` - Name of the Argo workflow
    /// * `start` - Start time for the query
    /// * `end` - End time for the query
    /// * `limit` - Maximum number of entries to return (0 = use default)
    ///
    /// # Note
    /// Argo workflows label pods with `workflows.argoproj.io/workflow`, but log
    /// collectors often relabel this to different keys. This function uses pod
    /// name regex matching (`pod =~ "workflow_name.*"`) which works regardless
    /// of how workflow labels are relabeled in the log pipeline.
    ///
    /// # Errors
    /// Returns an error if the Loki query fails or returns an invalid response.
    pub async fn query_workflow_logs(
        &self,
        namespace: &str,
        workflow_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<LogEntry>> {
        // Use pod name regex matching instead of workflow labels.
        // Argo workflow pod names follow the pattern: {workflow_name}-{step}-{random}
        // This is more reliable than depending on label relabeling configuration.
        let query = format!(r#"{{namespace="{namespace}", pod=~"{workflow_name}.*"}}"#);
        self.query_logs(&query, start, end, limit).await
    }

    /// Execute a raw `LogQL` query
    ///
    /// # Arguments
    /// * `query` - `LogQL` query string
    /// * `start` - Start time for the query
    /// * `end` - End time for the query
    /// * `limit` - Maximum number of entries to return (0 = use default)
    ///
    /// # Errors
    /// Returns an error if the Loki query fails, the response cannot be parsed,
    /// or the Loki API returns a non-success status.
    pub async fn query_logs(
        &self,
        query: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<LogEntry>> {
        let limit = if limit == 0 {
            self.config.default_limit
        } else {
            limit
        };

        // Convert timestamps to nanoseconds (Loki format)
        let start_ns = start.timestamp_nanos_opt().unwrap_or(0);
        let end_ns = end.timestamp_nanos_opt().unwrap_or(0);

        let url = format!(
            "{}/loki/api/v1/query_range",
            self.config.base_url.trim_end_matches('/')
        );

        debug!(
            query = %query,
            start = %start,
            end = %end,
            limit = %limit,
            "Querying Loki"
        );

        let response = self
            .client
            .get(&url)
            .query(&[
                ("query", query),
                ("start", &start_ns.to_string()),
                ("end", &end_ns.to_string()),
                ("limit", &limit.to_string()),
                ("direction", "backward"), // Most recent first (like kubectl logs --tail)
            ])
            .send()
            .await
            .context("Failed to send request to Loki")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Loki query failed with status {status}: {body}");
        }

        let loki_response: LokiResponse = response
            .json()
            .await
            .context("Failed to parse Loki response")?;

        if loki_response.status != "success" {
            anyhow::bail!("Loki query returned status: {}", loki_response.status);
        }

        // Parse the streams into log entries
        let mut entries = Vec::new();
        for stream in loki_response.data.result {
            let labels = stream.stream;
            for (timestamp_ns, line) in stream.values {
                // Parse nanosecond timestamp
                if let Ok(ns) = timestamp_ns.parse::<i64>() {
                    let secs = ns / 1_000_000_000;
                    #[allow(clippy::cast_sign_loss)]
                    let nsecs = (ns % 1_000_000_000) as u32;
                    if let Some(dt) = DateTime::from_timestamp(secs, nsecs) {
                        entries.push(LogEntry {
                            timestamp: dt,
                            line,
                            labels: labels.clone(),
                        });
                    }
                }
            }
        }

        // Sort by timestamp (should already be sorted, but ensure)
        entries.sort_by_key(|e| e.timestamp);

        debug!(entries = entries.len(), "Retrieved log entries from Loki");
        Ok(entries)
    }

    /// Check if Loki is reachable
    ///
    /// # Errors
    /// Returns an error if there's an issue building the request (but not
    /// if the server is unreachable - that returns `Ok(false)`).
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/ready", self.config.base_url.trim_end_matches('/'));

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!(error = %e, "Loki health check failed");
                Ok(false)
            }
        }
    }

    /// Get logs around a specific failure time
    ///
    /// Convenience method that queries logs from `minutes_before` before the failure
    /// to `minutes_after` after the failure.
    ///
    /// # Arguments
    /// * `namespace` - Kubernetes namespace
    /// * `pod_name` - Name of the pod
    /// * `failure_time` - The time when the failure occurred
    /// * `minutes_before` - Minutes before failure to include (must be non-negative)
    /// * `minutes_after` - Minutes after failure to include (must be non-negative)
    ///
    /// # Errors
    /// Returns an error if the Loki query fails or returns an invalid response.
    #[allow(clippy::cast_possible_wrap)]
    pub async fn query_logs_around_failure(
        &self,
        namespace: &str,
        pod_name: &str,
        failure_time: DateTime<Utc>,
        minutes_before: u64,
        minutes_after: u64,
    ) -> Result<Vec<LogEntry>> {
        // Use i64 for Duration but values are guaranteed non-negative via u64 params
        // and realistically won't exceed i64::MAX minutes
        let start = failure_time - chrono::Duration::minutes(minutes_before as i64);
        let end = failure_time + chrono::Duration::minutes(minutes_after as i64);
        self.query_pod_logs(namespace, pod_name, start, end, 0)
            .await
    }
}

/// Format log entries as a string suitable for GitHub issues or analysis
///
/// # Arguments
/// * `entries` - Log entries to format
/// * `max_lines` - Maximum number of lines to show (0 = show all)
#[must_use]
pub fn format_logs_for_issue(entries: &[LogEntry], max_lines: usize) -> String {
    use std::fmt::Write;

    if entries.is_empty() {
        return "No logs available.".to_string();
    }

    let mut output = String::new();
    let total = entries.len();

    // If max_lines is 0, show all entries; otherwise truncate to max_lines
    let entries_to_show: &[LogEntry] = if max_lines > 0 && total > max_lines {
        let _ = writeln!(
            output,
            "⚠️ Showing last {max_lines} of {total} log entries\n",
        );
        &entries[total - max_lines..]
    } else {
        entries
    };

    output.push_str("```\n");
    for entry in entries_to_show {
        let time = entry.timestamp.format("%H:%M:%S%.3f");
        let _ = writeln!(output, "[{}] {}", time, entry.line);
    }
    output.push_str("```\n");

    output
}

/// Extract error lines from log entries
#[must_use]
pub fn extract_error_lines(entries: &[LogEntry]) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|e| {
            let line_lower = e.line.to_lowercase();
            line_lower.contains("error")
                || line_lower.contains("fatal")
                || line_lower.contains("panic")
                || line_lower.contains("failed")
                || line_lower.contains("exception")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LokiConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.default_limit, 5000);
    }

    #[test]
    fn test_format_logs_empty() {
        let entries: Vec<LogEntry> = vec![];
        assert_eq!(format_logs_for_issue(&entries, 100), "No logs available.");
    }

    #[test]
    fn test_format_logs_truncation() {
        let entries: Vec<LogEntry> = (0..10)
            .map(|i| LogEntry {
                timestamp: Utc::now(),
                line: format!("Log line {i}"),
                labels: std::collections::HashMap::new(),
            })
            .collect();

        let output = format_logs_for_issue(&entries, 5);
        assert!(output.contains("Showing last 5 of 10"));
        assert!(output.contains("Log line 9")); // Last line should be included
        assert!(!output.contains("Log line 0")); // First line should be excluded
    }

    #[test]
    fn test_format_logs_zero_max_lines_shows_all() {
        let entries: Vec<LogEntry> = (0..10)
            .map(|i| LogEntry {
                timestamp: Utc::now(),
                line: format!("Log line {i}"),
                labels: std::collections::HashMap::new(),
            })
            .collect();

        // max_lines=0 should show all entries
        let output = format_logs_for_issue(&entries, 0);
        assert!(!output.contains("Showing last")); // No truncation warning
        assert!(output.contains("Log line 0")); // First line included
        assert!(output.contains("Log line 9")); // Last line included
    }

    #[test]
    fn test_extract_error_lines() {
        let entries = vec![
            LogEntry {
                timestamp: Utc::now(),
                line: "Starting server...".to_string(),
                labels: std::collections::HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: "ERROR: Connection failed".to_string(),
                labels: std::collections::HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: "Retrying...".to_string(),
                labels: std::collections::HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: "fatal: unable to connect".to_string(),
                labels: std::collections::HashMap::new(),
            },
        ];

        let errors = extract_error_lines(&entries);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].line.contains("ERROR"));
        assert!(errors[1].line.contains("fatal"));
    }
}
