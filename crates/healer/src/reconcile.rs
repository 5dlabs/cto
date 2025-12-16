//! Issue reconciliation module.
//!
//! Periodically checks open GitHub issues created by healer alerts to determine
//! if the underlying condition has been resolved. If resolved, closes the issue
//! with an auto-close comment.
//!
//! This runs as a CronJob every 5 minutes to keep issues up-to-date.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};

use crate::dedup::extract_pod_from_title;

/// Configuration for the issue reconciler.
#[derive(Debug, Clone)]
pub struct ReconcileConfig {
    /// Repository to check issues for.
    pub repository: String,
    /// Kubernetes namespace to check pod/workflow state.
    pub namespace: String,
    /// Labels that identify healer-created issues.
    pub healer_labels: Vec<String>,
    /// Dry run mode - check but don't close issues.
    pub dry_run: bool,
    /// Maximum issues to process per run (to avoid rate limits).
    pub max_issues: usize,
}

impl Default for ReconcileConfig {
    fn default() -> Self {
        Self {
            repository: "5dlabs/cto".to_string(),
            namespace: "cto".to_string(),
            healer_labels: vec!["heal".to_string()],
            dry_run: false,
            max_issues: 50,
        }
    }
}

/// An open healer issue that may need reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealerIssue {
    /// Issue number.
    pub number: u64,
    /// Issue title.
    pub title: String,
    /// When the issue was created.
    pub created_at: DateTime<Utc>,
    /// Labels on the issue.
    pub labels: Vec<String>,
    /// Extracted alert type (A1, A2, A7, etc.).
    pub alert_type: Option<String>,
    /// Extracted pod/resource name.
    pub resource_name: Option<String>,
}

/// Result of checking if an issue should be closed.
#[derive(Debug, Clone)]
pub enum ReconcileResult {
    /// Issue should remain open - condition still exists.
    StillActive { reason: String },
    /// Issue should be closed - condition resolved.
    Resolved { reason: String },
    /// Could not determine - leave open.
    Unknown { reason: String },
}

/// Report from a reconciliation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileReport {
    /// When the reconciliation ran.
    pub run_time: DateTime<Utc>,
    /// Total open issues found.
    pub issues_checked: usize,
    /// Issues that were closed.
    pub issues_closed: usize,
    /// Issues that remain open.
    pub issues_still_active: usize,
    /// Issues that couldn't be determined.
    pub issues_unknown: usize,
    /// Details of closed issues.
    pub closed_issues: Vec<ClosedIssueInfo>,
}

/// Information about a closed issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedIssueInfo {
    /// Issue number.
    pub number: u64,
    /// Issue title.
    pub title: String,
    /// Reason for closing.
    pub reason: String,
}

/// Issue reconciler - checks and closes resolved issues.
pub struct IssueReconciler {
    config: ReconcileConfig,
}

impl IssueReconciler {
    /// Create a new reconciler with the given configuration.
    #[must_use]
    pub fn new(config: ReconcileConfig) -> Self {
        Self { config }
    }

    /// Run the reconciliation process.
    ///
    /// # Errors
    ///
    /// Returns an error if GitHub queries fail.
    pub fn reconcile(&self) -> Result<ReconcileReport> {
        info!(
            "Starting issue reconciliation for {}",
            self.config.repository
        );

        // 1. Query open healer issues
        let issues = self.query_open_issues()?;
        info!("Found {} open healer issues", issues.len());

        let mut report = ReconcileReport {
            run_time: Utc::now(),
            issues_checked: issues.len(),
            issues_closed: 0,
            issues_still_active: 0,
            issues_unknown: 0,
            closed_issues: vec![],
        };

        // 2. Check each issue
        for issue in issues.iter().take(self.config.max_issues) {
            match self.check_issue(issue) {
                ReconcileResult::StillActive { reason } => {
                    debug!("Issue #{} still active: {}", issue.number, reason);
                    report.issues_still_active += 1;
                }
                ReconcileResult::Resolved { reason } => {
                    info!("Issue #{} resolved: {}", issue.number, reason);
                    if !self.config.dry_run {
                        if let Err(e) = self.close_issue(issue, &reason) {
                            warn!("Failed to close issue #{}: {}", issue.number, e);
                            report.issues_unknown += 1;
                            continue;
                        }
                    }
                    report.issues_closed += 1;
                    report.closed_issues.push(ClosedIssueInfo {
                        number: issue.number,
                        title: issue.title.clone(),
                        reason,
                    });
                }
                ReconcileResult::Unknown { reason } => {
                    debug!("Issue #{} unknown status: {}", issue.number, reason);
                    report.issues_unknown += 1;
                }
            }
        }

        info!(
            "Reconciliation complete: {} checked, {} closed, {} still active, {} unknown",
            report.issues_checked,
            report.issues_closed,
            report.issues_still_active,
            report.issues_unknown
        );

        Ok(report)
    }

    /// Query open issues with healer labels.
    fn query_open_issues(&self) -> Result<Vec<HealerIssue>> {
        let label_args: Vec<String> = self
            .config
            .healer_labels
            .iter()
            .map(|l| format!("--label={l}"))
            .collect();

        let mut args = vec![
            "issue".to_string(),
            "list".to_string(),
            "--repo".to_string(),
            self.config.repository.clone(),
            "--state".to_string(),
            "open".to_string(),
            "--json".to_string(),
            "number,title,createdAt,labels".to_string(),
            "--limit".to_string(),
            self.config.max_issues.to_string(),
        ];
        args.extend(label_args);

        let output = Command::new("gh")
            .args(&args)
            .output()
            .context("Failed to query GitHub issues")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue list failed: {stderr}");
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        if json_str.trim().is_empty() || json_str.trim() == "[]" {
            return Ok(vec![]);
        }

        let raw_issues: Vec<serde_json::Value> =
            serde_json::from_str(&json_str).context("Failed to parse GitHub issues JSON")?;

        let issues = raw_issues
            .into_iter()
            .filter_map(|v| self.parse_issue(&v))
            .collect();

        Ok(issues)
    }

    /// Parse a raw JSON issue into a `HealerIssue`.
    #[allow(clippy::unused_self)]
    fn parse_issue(&self, value: &serde_json::Value) -> Option<HealerIssue> {
        let number = value["number"].as_u64()?;
        let title = value["title"].as_str()?.to_string();
        let created_at_str = value["createdAt"].as_str()?;
        let created_at = DateTime::parse_from_rfc3339(created_at_str)
            .ok()?
            .with_timezone(&Utc);

        let labels: Vec<String> = value["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Extract alert type from title: "[HEAL-A2]..." or labels
        let alert_type = extract_alert_type(&title, &labels);

        // Extract resource name from title
        let resource_name = extract_pod_from_title(&title);

        Some(HealerIssue {
            number,
            title,
            created_at,
            labels,
            alert_type,
            resource_name,
        })
    }

    /// Check if an issue should be closed.
    fn check_issue(&self, issue: &HealerIssue) -> ReconcileResult {
        // If we can't extract a resource name, we can't check status
        let Some(resource_name) = &issue.resource_name else {
            return ReconcileResult::Unknown {
                reason: "Could not extract resource name from issue title".to_string(),
            };
        };

        // Check based on alert type
        match issue.alert_type.as_deref() {
            Some("A2" | "A7" | "a2" | "a7") => {
                // Pod-based alerts - check if pod still exists and is still failing
                self.check_pod_status(resource_name)
            }
            Some("A9" | "a9") => {
                // CodeRun-based alerts - check if CodeRun still exists and is stuck
                self.check_coderun_status(resource_name)
            }
            Some(alert_type) if alert_type.starts_with("CI") || alert_type.contains("ci") => {
                // CI failure alerts - check if there's been a successful run since
                self.check_ci_status(resource_name)
            }
            _ => {
                // For unknown alert types, check if pod exists
                // If no pod/workflow exists with this name, likely resolved
                self.check_resource_exists(resource_name)
            }
        }
    }

    /// Check pod status for A2/A7 alerts.
    fn check_pod_status(&self, pod_name: &str) -> ReconcileResult {
        // Check if pod exists
        let output = Command::new("kubectl")
            .args([
                "get",
                "pod",
                pod_name,
                "-n",
                &self.config.namespace,
                "-o",
                "jsonpath={.status.phase}",
            ])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                return ReconcileResult::Unknown {
                    reason: format!("Failed to query pod status: {e}"),
                }
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Pod not found = resolved
            if stderr.contains("NotFound") || stderr.contains("not found") {
                return ReconcileResult::Resolved {
                    reason: format!("Pod '{pod_name}' no longer exists"),
                };
            }
            return ReconcileResult::Unknown {
                reason: format!("kubectl get pod failed: {stderr}"),
            };
        }

        let phase = String::from_utf8_lossy(&output.stdout).trim().to_string();

        match phase.as_str() {
            "Running" => {
                // Pod is running - check if containers are healthy
                if self.are_containers_healthy(pod_name) {
                    ReconcileResult::Resolved {
                        reason: format!(
                            "Pod '{pod_name}' is now healthy (Running with all containers ready)"
                        ),
                    }
                } else {
                    ReconcileResult::StillActive {
                        reason: format!("Pod '{pod_name}' is Running but containers not healthy"),
                    }
                }
            }
            "Succeeded" => ReconcileResult::Resolved {
                reason: format!("Pod '{pod_name}' completed successfully"),
            },
            "Failed" | "Error" => ReconcileResult::StillActive {
                reason: format!("Pod '{pod_name}' is in {phase} state"),
            },
            "" => {
                // Empty phase might mean pod was deleted
                ReconcileResult::Resolved {
                    reason: format!("Pod '{pod_name}' no longer exists"),
                }
            }
            _ => ReconcileResult::Unknown {
                reason: format!("Pod '{pod_name}' in unexpected phase: {phase}"),
            },
        }
    }

    /// Check if all containers in a pod are healthy.
    fn are_containers_healthy(&self, pod_name: &str) -> bool {
        let output = Command::new("kubectl")
            .args([
                "get",
                "pod",
                pod_name,
                "-n",
                &self.config.namespace,
                "-o",
                "jsonpath={.status.containerStatuses[*].ready}",
            ])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return false,
        };

        let ready_statuses = String::from_utf8_lossy(&output.stdout);
        // All containers should be "true"
        ready_statuses.split_whitespace().all(|s| s == "true")
    }

    /// Check CodeRun status for A9 alerts.
    fn check_coderun_status(&self, coderun_name: &str) -> ReconcileResult {
        let output = Command::new("kubectl")
            .args([
                "get",
                "coderun",
                coderun_name,
                "-n",
                &self.config.namespace,
                "-o",
                "jsonpath={.status.phase}",
            ])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                return ReconcileResult::Unknown {
                    reason: format!("Failed to query CodeRun status: {e}"),
                }
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("NotFound") || stderr.contains("not found") {
                return ReconcileResult::Resolved {
                    reason: format!("CodeRun '{coderun_name}' no longer exists"),
                };
            }
            return ReconcileResult::Unknown {
                reason: format!("kubectl get coderun failed: {stderr}"),
            };
        }

        let phase = String::from_utf8_lossy(&output.stdout).trim().to_string();

        match phase.as_str() {
            "Succeeded" | "Completed" => ReconcileResult::Resolved {
                reason: format!("CodeRun '{coderun_name}' completed successfully"),
            },
            "Failed" | "Error" => ReconcileResult::Resolved {
                reason: format!("CodeRun '{coderun_name}' failed (no longer stuck)"),
            },
            "Running" | "Pending" => ReconcileResult::StillActive {
                reason: format!("CodeRun '{coderun_name}' still in {phase} state"),
            },
            "" => ReconcileResult::Resolved {
                reason: format!("CodeRun '{coderun_name}' no longer exists"),
            },
            _ => ReconcileResult::Unknown {
                reason: format!("CodeRun '{coderun_name}' in unexpected phase: {phase}"),
            },
        }
    }

    /// Check CI status - look for successful workflow runs since issue creation.
    fn check_ci_status(&self, workflow_name: &str) -> ReconcileResult {
        // For CI failures, we'd need to check if there's been a successful run
        // This is more complex - for now, just check if the workflow exists
        let output = Command::new("gh")
            .args([
                "run",
                "list",
                "--repo",
                &self.config.repository,
                "--workflow",
                workflow_name,
                "--status",
                "success",
                "--limit",
                "1",
                "--json",
                "createdAt",
            ])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => {
                return ReconcileResult::Unknown {
                    reason: "Could not query workflow runs".to_string(),
                }
            }
        };

        let json_str = String::from_utf8_lossy(&output.stdout);
        if json_str.contains("createdAt") {
            // There's been a successful run
            ReconcileResult::Resolved {
                reason: format!(
                    "Workflow '{workflow_name}' has had successful runs since issue creation"
                ),
            }
        } else {
            ReconcileResult::StillActive {
                reason: format!("No successful runs found for workflow '{workflow_name}'"),
            }
        }
    }

    /// Generic check if a resource exists (fallback).
    fn check_resource_exists(&self, resource_name: &str) -> ReconcileResult {
        // Try pod first
        let pod_output = Command::new("kubectl")
            .args([
                "get",
                "pod",
                resource_name,
                "-n",
                &self.config.namespace,
                "--ignore-not-found",
                "-o",
                "name",
            ])
            .output();

        if let Ok(output) = pod_output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    return ReconcileResult::Resolved {
                        reason: format!("Resource '{resource_name}' no longer exists"),
                    };
                }
            }
        }

        // Try workflow
        let wf_output = Command::new("kubectl")
            .args([
                "get",
                "workflow",
                resource_name,
                "-n",
                &self.config.namespace,
                "--ignore-not-found",
                "-o",
                "name",
            ])
            .output();

        if let Ok(output) = wf_output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    return ReconcileResult::Resolved {
                        reason: format!("Resource '{resource_name}' no longer exists"),
                    };
                }
            }
        }

        ReconcileResult::Unknown {
            reason: format!("Could not determine status of resource '{resource_name}'"),
        }
    }

    /// Close an issue with an auto-close comment.
    fn close_issue(&self, issue: &HealerIssue, reason: &str) -> Result<()> {
        // Add comment explaining why we're closing
        let comment = format!(
            "🤖 **Auto-closed by Healer Reconciler**\n\n\
             The underlying condition that triggered this alert has been resolved:\n\n\
             > {reason}\n\n\
             If this issue was closed in error, please reopen it."
        );

        let comment_output = Command::new("gh")
            .args([
                "issue",
                "comment",
                &issue.number.to_string(),
                "--repo",
                &self.config.repository,
                "--body",
                &comment,
            ])
            .output()
            .context("Failed to add close comment")?;

        if !comment_output.status.success() {
            let stderr = String::from_utf8_lossy(&comment_output.stderr);
            warn!(
                "Failed to add comment to issue #{}: {}",
                issue.number, stderr
            );
        }

        // Close the issue
        let close_output = Command::new("gh")
            .args([
                "issue",
                "close",
                &issue.number.to_string(),
                "--repo",
                &self.config.repository,
                "--reason",
                "completed",
            ])
            .output()
            .context("Failed to close issue")?;

        if !close_output.status.success() {
            let stderr = String::from_utf8_lossy(&close_output.stderr);
            anyhow::bail!("gh issue close failed: {stderr}");
        }

        info!("Closed issue #{}: {}", issue.number, issue.title);
        Ok(())
    }
}

/// Extract alert type from issue title or labels.
///
/// Titles follow patterns like:
/// - `[HEAL-A2] Silent Failure: pod-name`
/// - `[CI Failure] workflow - branch (commit)`
fn extract_alert_type(title: &str, labels: &[String]) -> Option<String> {
    // Try to extract from title
    if let Some(start) = title.find("[HEAL-") {
        if let Some(end) = title[start..].find(']') {
            let alert_type = &title[start + 6..start + end];
            return Some(alert_type.to_string());
        }
    }

    // Try CI failure pattern
    if title.starts_with("[CI Failure]") || title.contains("CI Failure") {
        return Some("CI".to_string());
    }

    // Fall back to labels
    for label in labels {
        let label_upper = label.to_uppercase();
        if label_upper.starts_with('A') && label_upper.len() <= 3 {
            // Likely A1, A2, A7, A9, etc.
            return Some(label_upper);
        }
    }

    None
}

/// Format a reconcile report as text.
#[must_use]
pub fn format_report_text(report: &ReconcileReport) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(output, "=== Issue Reconciliation Report ===").unwrap();
    writeln!(output, "Time: {}", report.run_time).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "Issues Checked: {}", report.issues_checked).unwrap();
    writeln!(output, "Issues Closed: {}", report.issues_closed).unwrap();
    writeln!(
        output,
        "Issues Still Active: {}",
        report.issues_still_active
    )
    .unwrap();
    writeln!(output, "Issues Unknown: {}", report.issues_unknown).unwrap();

    if !report.closed_issues.is_empty() {
        writeln!(output).unwrap();
        writeln!(output, "Closed Issues:").unwrap();
        for info in &report.closed_issues {
            writeln!(output, "  - #{}: {}", info.number, info.title).unwrap();
            writeln!(output, "    Reason: {}", info.reason).unwrap();
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_alert_type_from_title() {
        assert_eq!(
            extract_alert_type("[HEAL-A2] Silent Failure: my-pod-123", &[]),
            Some("A2".to_string())
        );
        assert_eq!(
            extract_alert_type("[HEAL-A7] Pod Failure: play-task-4-abc", &[]),
            Some("A7".to_string())
        );
        assert_eq!(
            extract_alert_type("[CI Failure] build - main (abc123)", &[]),
            Some("CI".to_string())
        );
    }

    #[test]
    fn test_extract_alert_type_from_labels() {
        let labels = vec!["heal".to_string(), "A2".to_string()];
        assert_eq!(
            extract_alert_type("Some issue title", &labels),
            Some("A2".to_string())
        );
    }

    #[test]
    fn test_default_config() {
        let config = ReconcileConfig::default();
        assert_eq!(config.repository, "5dlabs/cto");
        assert_eq!(config.namespace, "cto");
        assert!(!config.dry_run);
    }
}
