//! # PR Check Status
//!
//! Fetch and monitor the status of GitHub check runs on a PR.
//! Use this to verify all CI checks pass before completing remediation.
//!
//! ## Example
//!
//! ```no_run
//! use utils::PrChecks;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let checks = PrChecks::new("5dlabs", "cto");
//!
//! // Get all check status
//! let status = checks.fetch(2622).await?;
//! println!("All passed: {}", status.all_passed());
//!
//! // Wait for checks to complete (with timeout)
//! let final_status = checks.wait_for_completion(2622, 600, None).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

/// Status of an individual check run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    /// Check is queued but not started
    Queued,
    /// Check is currently running
    InProgress,
    /// Check completed
    Completed,
}

/// Conclusion of a completed check run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckConclusion {
    /// Check passed
    Success,
    /// Check failed
    Failure,
    /// Check was skipped
    Skipped,
    /// Check was cancelled
    Cancelled,
    /// Check timed out
    TimedOut,
    /// Neutral (informational)
    Neutral,
    /// Action required
    ActionRequired,
    /// Stale check
    Stale,
    /// Startup failure
    StartupFailure,
}

impl std::fmt::Display for CheckConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::Failure => write!(f, "FAILURE"),
            Self::Skipped => write!(f, "SKIPPED"),
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::TimedOut => write!(f, "TIMED_OUT"),
            Self::Neutral => write!(f, "NEUTRAL"),
            Self::ActionRequired => write!(f, "ACTION_REQUIRED"),
            Self::Stale => write!(f, "STALE"),
            Self::StartupFailure => write!(f, "STARTUP_FAILURE"),
        }
    }
}

/// An individual check run with its status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCheck {
    /// Check name (e.g., "lint-rust", "CodeQL")
    pub name: String,

    /// Workflow name (e.g., "Controller CI")
    pub workflow: String,

    /// Current status
    pub status: CheckStatus,

    /// Conclusion (only present if completed)
    pub conclusion: Option<CheckConclusion>,

    /// Whether this check is required (based on branch protection)
    #[serde(default)]
    pub required: bool,

    /// Details URL
    pub url: String,
}

impl CiCheck {
    /// Check if this run passed or was skipped
    #[must_use]
    pub fn passed(&self) -> bool {
        matches!(
            self.conclusion,
            Some(CheckConclusion::Success | CheckConclusion::Skipped | CheckConclusion::Neutral)
        )
    }

    /// Check if this run failed
    #[must_use]
    pub fn failed(&self) -> bool {
        matches!(
            self.conclusion,
            Some(
                CheckConclusion::Failure
                    | CheckConclusion::TimedOut
                    | CheckConclusion::Cancelled
                    | CheckConclusion::StartupFailure
            )
        )
    }

    /// Check if this run is still pending
    #[must_use]
    pub fn pending(&self) -> bool {
        matches!(self.status, CheckStatus::Queued | CheckStatus::InProgress)
    }
}

/// Summary of all PR check runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrCheckStatus {
    /// All check runs
    pub checks: Vec<CiCheck>,

    /// Total number of checks
    pub total: usize,

    /// Number of passed checks
    pub passed: usize,

    /// Number of failed checks
    pub failed: usize,

    /// Number of pending checks
    pub pending: usize,

    /// Number of skipped checks
    pub skipped: usize,

    /// Overall merge state (from GitHub)
    pub merge_state: String,
}

impl PrCheckStatus {
    /// Build summary from check runs
    fn from_checks(checks: Vec<CiCheck>, merge_state: String) -> Self {
        let total = checks.len();
        let passed = checks.iter().filter(|c| c.passed()).count();
        let failed = checks.iter().filter(|c| c.failed()).count();
        let pending = checks.iter().filter(|c| c.pending()).count();
        let skipped = checks
            .iter()
            .filter(|c| c.conclusion == Some(CheckConclusion::Skipped))
            .count();

        Self {
            checks,
            total,
            passed,
            failed,
            pending,
            skipped,
            merge_state,
        }
    }

    /// Check if all checks have passed (or are skipped/neutral)
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.pending == 0
    }

    /// Check if there are any failures
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }

    /// Check if checks are still running
    #[must_use]
    pub fn has_pending(&self) -> bool {
        self.pending > 0
    }

    /// Get only the failed checks
    #[must_use]
    pub fn failed_checks(&self) -> Vec<&CiCheck> {
        self.checks.iter().filter(|c| c.failed()).collect()
    }

    /// Get only the pending checks
    #[must_use]
    pub fn pending_checks(&self) -> Vec<&CiCheck> {
        self.checks.iter().filter(|c| c.pending()).collect()
    }
}

/// Client for fetching PR check status from GitHub
#[derive(Debug, Clone)]
pub struct PrChecks {
    owner: String,
    repo: String,
}

impl PrChecks {
    /// Create a new PR checks client
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Fetch the current check status for a PR
    pub async fn fetch(&self, pr_number: u32) -> Result<PrCheckStatus> {
        info!(pr = pr_number, "Fetching check status");

        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &pr_number.to_string(),
                "--repo",
                &format!("{}/{}", self.owner, self.repo),
                "--json",
                "statusCheckRollup,mergeStateStatus",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr view failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: PrViewResponse =
            serde_json::from_str(&stdout).context("Failed to parse PR view response")?;

        let checks = response
            .status_check_rollup
            .into_iter()
            .filter_map(|item| self.parse_check_item(item))
            .collect();

        let status = PrCheckStatus::from_checks(checks, response.merge_state_status);

        info!(
            pr = pr_number,
            total = status.total,
            passed = status.passed,
            failed = status.failed,
            pending = status.pending,
            "Check status fetched"
        );

        Ok(status)
    }

    /// Wait for all checks to complete, polling periodically
    ///
    /// # Arguments
    /// * `pr_number` - The PR number
    /// * `timeout_secs` - Maximum seconds to wait
    /// * `poll_interval_secs` - Seconds between polls (default 30)
    pub async fn wait_for_completion(
        &self,
        pr_number: u32,
        timeout_secs: u64,
        poll_interval_secs: Option<u64>,
    ) -> Result<PrCheckStatus> {
        let poll_interval = Duration::from_secs(poll_interval_secs.unwrap_or(30));
        let timeout = Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();

        info!(
            pr = pr_number,
            timeout_secs, "Waiting for checks to complete"
        );

        loop {
            let status = self.fetch(pr_number).await?;

            if !status.has_pending() {
                info!(
                    pr = pr_number,
                    all_passed = status.all_passed(),
                    failed = status.failed,
                    "All checks complete"
                );
                return Ok(status);
            }

            let elapsed = start.elapsed();
            if elapsed >= timeout {
                warn!(
                    pr = pr_number,
                    pending = status.pending,
                    elapsed_secs = elapsed.as_secs(),
                    "Timeout waiting for checks"
                );
                return Ok(status);
            }

            debug!(
                pr = pr_number,
                pending = status.pending,
                elapsed_secs = elapsed.as_secs(),
                "Checks still pending, waiting..."
            );

            sleep(poll_interval).await;
        }
    }

    /// Parse a status check item from the GraphQL response
    fn parse_check_item(&self, item: StatusCheckItem) -> Option<CiCheck> {
        match item {
            StatusCheckItem::CheckRun {
                name,
                workflow_name,
                status,
                conclusion,
                details_url,
            } => Some(CiCheck {
                name,
                workflow: workflow_name.unwrap_or_default(),
                status: parse_status(&status),
                conclusion: conclusion.as_deref().and_then(parse_conclusion),
                required: false, // Would need additional API call for branch protection rules
                url: details_url.unwrap_or_default(),
            }),
            StatusCheckItem::StatusContext { context, state, .. } => {
                // Status contexts are from commit status API (legacy)
                Some(CiCheck {
                    name: context,
                    workflow: String::new(),
                    status: CheckStatus::Completed,
                    conclusion: Some(match state.as_str() {
                        "SUCCESS" | "success" => CheckConclusion::Success,
                        "FAILURE" | "failure" | "ERROR" | "error" => CheckConclusion::Failure,
                        "PENDING" | "pending" => return None, // Will be handled as pending
                        _ => CheckConclusion::Neutral,
                    }),
                    required: false,
                    url: String::new(),
                })
            }
        }
    }

    /// Get list of failing check names for a PR
    pub async fn get_failing_checks(&self, pr_number: u32) -> Result<Vec<String>> {
        let status = self.fetch(pr_number).await?;
        Ok(status
            .failed_checks()
            .into_iter()
            .map(|c| c.name.clone())
            .collect())
    }

    /// Check if a specific check has passed
    pub async fn check_passed(&self, pr_number: u32, check_name: &str) -> Result<bool> {
        let status = self.fetch(pr_number).await?;
        Ok(status
            .checks
            .iter()
            .any(|c| c.name == check_name && c.passed()))
    }

    /// Check if specific checks have all passed
    pub async fn checks_passed(&self, pr_number: u32, check_names: &[&str]) -> Result<bool> {
        let status = self.fetch(pr_number).await?;
        let names: HashSet<&str> = check_names.iter().copied().collect();

        let relevant_checks: Vec<_> = status
            .checks
            .iter()
            .filter(|c| names.contains(c.name.as_str()))
            .collect();

        // All specified checks must exist and pass
        if relevant_checks.len() != check_names.len() {
            return Ok(false);
        }

        Ok(relevant_checks.iter().all(|c| c.passed()))
    }
}

/// Parse status string to enum
fn parse_status(s: &str) -> CheckStatus {
    match s {
        "QUEUED" | "queued" => CheckStatus::Queued,
        "IN_PROGRESS" | "in_progress" => CheckStatus::InProgress,
        "COMPLETED" | "completed" => CheckStatus::Completed,
        _ => CheckStatus::Completed, // Default to completed for unknown
    }
}

/// Parse conclusion string to enum
fn parse_conclusion(s: &str) -> Option<CheckConclusion> {
    match s {
        "SUCCESS" | "success" => Some(CheckConclusion::Success),
        "FAILURE" | "failure" => Some(CheckConclusion::Failure),
        "SKIPPED" | "skipped" => Some(CheckConclusion::Skipped),
        "CANCELLED" | "cancelled" => Some(CheckConclusion::Cancelled),
        "TIMED_OUT" | "timed_out" => Some(CheckConclusion::TimedOut),
        "NEUTRAL" | "neutral" => Some(CheckConclusion::Neutral),
        "ACTION_REQUIRED" | "action_required" => Some(CheckConclusion::ActionRequired),
        "STALE" | "stale" => Some(CheckConclusion::Stale),
        "STARTUP_FAILURE" | "startup_failure" => Some(CheckConclusion::StartupFailure),
        _ => None,
    }
}

/// Response from gh pr view --json statusCheckRollup
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrViewResponse {
    #[serde(default)]
    status_check_rollup: Vec<StatusCheckItem>,
    #[serde(default)]
    merge_state_status: String,
}

/// Status check item - can be CheckRun or StatusContext
#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
enum StatusCheckItem {
    CheckRun {
        name: String,
        #[serde(rename = "workflowName")]
        workflow_name: Option<String>,
        status: String,
        conclusion: Option<String>,
        #[serde(rename = "detailsUrl")]
        details_url: Option<String>,
    },
    StatusContext {
        context: String,
        state: String,
        #[serde(rename = "targetUrl")]
        #[allow(dead_code)]
        target_url: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_check_passed() {
        let check = CiCheck {
            name: "test".to_string(),
            workflow: "CI".to_string(),
            status: CheckStatus::Completed,
            conclusion: Some(CheckConclusion::Success),
            required: false,
            url: String::new(),
        };
        assert!(check.passed());
        assert!(!check.failed());
        assert!(!check.pending());
    }

    #[test]
    fn test_ci_check_failed() {
        let check = CiCheck {
            name: "test".to_string(),
            workflow: "CI".to_string(),
            status: CheckStatus::Completed,
            conclusion: Some(CheckConclusion::Failure),
            required: false,
            url: String::new(),
        };
        assert!(!check.passed());
        assert!(check.failed());
        assert!(!check.pending());
    }

    #[test]
    fn test_ci_check_pending() {
        let check = CiCheck {
            name: "test".to_string(),
            workflow: "CI".to_string(),
            status: CheckStatus::InProgress,
            conclusion: None,
            required: false,
            url: String::new(),
        };
        assert!(!check.passed());
        assert!(!check.failed());
        assert!(check.pending());
    }

    #[test]
    fn test_ci_check_skipped() {
        let check = CiCheck {
            name: "test".to_string(),
            workflow: "CI".to_string(),
            status: CheckStatus::Completed,
            conclusion: Some(CheckConclusion::Skipped),
            required: false,
            url: String::new(),
        };
        assert!(check.passed()); // Skipped counts as passed
        assert!(!check.failed());
    }

    #[test]
    fn test_pr_check_status_all_passed() {
        let checks = vec![
            CiCheck {
                name: "test1".to_string(),
                workflow: "CI".to_string(),
                status: CheckStatus::Completed,
                conclusion: Some(CheckConclusion::Success),
                required: false,
                url: String::new(),
            },
            CiCheck {
                name: "test2".to_string(),
                workflow: "CI".to_string(),
                status: CheckStatus::Completed,
                conclusion: Some(CheckConclusion::Skipped),
                required: false,
                url: String::new(),
            },
        ];

        let status = PrCheckStatus::from_checks(checks, "CLEAN".to_string());
        assert!(status.all_passed());
        assert!(!status.has_failures());
        assert!(!status.has_pending());
    }

    #[test]
    fn test_pr_check_status_with_failure() {
        let checks = vec![
            CiCheck {
                name: "test1".to_string(),
                workflow: "CI".to_string(),
                status: CheckStatus::Completed,
                conclusion: Some(CheckConclusion::Success),
                required: false,
                url: String::new(),
            },
            CiCheck {
                name: "test2".to_string(),
                workflow: "CI".to_string(),
                status: CheckStatus::Completed,
                conclusion: Some(CheckConclusion::Failure),
                required: false,
                url: String::new(),
            },
        ];

        let status = PrCheckStatus::from_checks(checks, "UNSTABLE".to_string());
        assert!(!status.all_passed());
        assert!(status.has_failures());
        assert_eq!(status.failed, 1);
        assert_eq!(status.failed_checks().len(), 1);
        assert_eq!(status.failed_checks()[0].name, "test2");
    }

    #[test]
    fn test_pr_check_status_with_pending() {
        let checks = vec![
            CiCheck {
                name: "test1".to_string(),
                workflow: "CI".to_string(),
                status: CheckStatus::InProgress,
                conclusion: None,
                required: false,
                url: String::new(),
            },
            CiCheck {
                name: "test2".to_string(),
                workflow: "CI".to_string(),
                status: CheckStatus::Completed,
                conclusion: Some(CheckConclusion::Success),
                required: false,
                url: String::new(),
            },
        ];

        let status = PrCheckStatus::from_checks(checks, "BLOCKED".to_string());
        assert!(!status.all_passed());
        assert!(status.has_pending());
        assert_eq!(status.pending, 1);
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(parse_status("QUEUED"), CheckStatus::Queued);
        assert_eq!(parse_status("IN_PROGRESS"), CheckStatus::InProgress);
        assert_eq!(parse_status("COMPLETED"), CheckStatus::Completed);
        assert_eq!(parse_status("unknown"), CheckStatus::Completed);
    }

    #[test]
    fn test_parse_conclusion() {
        assert_eq!(parse_conclusion("SUCCESS"), Some(CheckConclusion::Success));
        assert_eq!(parse_conclusion("FAILURE"), Some(CheckConclusion::Failure));
        assert_eq!(parse_conclusion("SKIPPED"), Some(CheckConclusion::Skipped));
        assert!(parse_conclusion("unknown").is_none());
    }

    #[test]
    fn test_pr_checks_new() {
        let client = PrChecks::new("owner", "repo");
        assert_eq!(client.owner, "owner");
        assert_eq!(client.repo, "repo");
    }
}
