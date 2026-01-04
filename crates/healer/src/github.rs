//! GitHub types and state management for the monitor.

#![allow(dead_code)] // Public API - methods used for GitHub polling integration

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};

/// Merge method for pull requests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeMethod {
    /// Standard merge commit
    Merge,
    /// Squash and merge
    #[default]
    Squash,
    /// Rebase and merge
    Rebase,
}

impl MergeMethod {
    /// Get the gh CLI flag for this merge method.
    #[must_use]
    pub fn as_flag(self) -> &'static str {
        match self {
            Self::Merge => "--merge",
            Self::Squash => "--squash",
            Self::Rebase => "--rebase",
        }
    }
}

/// Status of a PR's mergeability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStatus {
    /// PR is ready to merge (all checks passed, no conflicts)
    Ready,
    /// PR has merge conflicts
    Conflicting,
    /// PR is blocked (checks pending or failing, or waiting for review)
    Blocked { reason: String },
    /// PR state is unknown
    Unknown,
}

/// GitHub client for interacting with GitHub via the `gh` CLI
#[derive(Debug, Clone)]
pub struct GitHubClient {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
}

impl GitHubClient {
    /// Create a new GitHub client
    #[must_use]
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        }
    }

    /// Get the full repository path (owner/repo)
    #[must_use]
    pub fn repo_path(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// Create a GitHub issue
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn create_issue(&self, title: &str, body: &str, labels: &[&str]) -> Result<String> {
        let labels_str = labels.join(",");

        debug!(
            repo = %self.repo_path(),
            title = %title,
            "Creating GitHub issue"
        );

        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                &self.repo_path(),
                "--title",
                title,
                "--body",
                body,
                "--label",
                &labels_str,
            ])
            .output()
            .context("Failed to execute gh issue create")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue create failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issue_url = stdout.trim().to_string();

        info!(issue_url = %issue_url, "Created GitHub issue");
        Ok(issue_url)
    }

    /// Add a comment to an issue
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn add_issue_comment(&self, issue_number: u32, body: &str) -> Result<()> {
        let output = Command::new("gh")
            .args([
                "issue",
                "comment",
                &issue_number.to_string(),
                "--repo",
                &self.repo_path(),
                "--body",
                body,
            ])
            .output()
            .context("Failed to execute gh issue comment")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue comment failed: {stderr}");
        }

        Ok(())
    }

    /// Close an issue
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn close_issue(&self, issue_number: u32, reason: Option<&str>) -> Result<()> {
        let mut args = vec![
            "issue".to_string(),
            "close".to_string(),
            issue_number.to_string(),
            "--repo".to_string(),
            self.repo_path(),
        ];

        if let Some(r) = reason {
            args.push("--reason".to_string());
            args.push(r.to_string());
        }

        let output = Command::new("gh")
            .args(&args)
            .output()
            .context("Failed to execute gh issue close")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue close failed: {stderr}");
        }

        Ok(())
    }

    /// Merge a pull request.
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails or the PR cannot be merged.
    pub fn merge_pr(&self, pr_number: u32, method: MergeMethod) -> Result<()> {
        debug!(
            repo = %self.repo_path(),
            pr = pr_number,
            method = ?method,
            "Merging pull request"
        );

        let output = Command::new("gh")
            .args([
                "pr",
                "merge",
                &pr_number.to_string(),
                "--repo",
                &self.repo_path(),
                method.as_flag(),
                "--delete-branch",
            ])
            .output()
            .context("Failed to execute gh pr merge")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr merge failed: {stderr}");
        }

        info!(
            repo = %self.repo_path(),
            pr = pr_number,
            "Successfully merged pull request"
        );
        Ok(())
    }

    /// Check the merge status of a pull request.
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn check_mergeable(&self, pr_number: u32) -> Result<MergeStatus> {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &pr_number.to_string(),
                "--repo",
                &self.repo_path(),
                "--json",
                "mergeable,mergeStateStatus,statusCheckRollup",
            ])
            .output()
            .context("Failed to execute gh pr view")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("gh pr view failed: {stderr}");
            return Ok(MergeStatus::Unknown);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).context("Failed to parse PR JSON")?;

        // Check mergeable field
        let mergeable = json
            .get("mergeable")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        if mergeable == "CONFLICTING" {
            return Ok(MergeStatus::Conflicting);
        }

        // Check merge state status
        let merge_state = json
            .get("mergeStateStatus")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        match merge_state {
            "CLEAN" => Ok(MergeStatus::Ready),
            "BLOCKED" => {
                // Check why it's blocked
                let checks = json.get("statusCheckRollup").and_then(|v| v.as_array());
                let pending = checks.is_some_and(|c| {
                    c.iter().any(|check| {
                        check
                            .get("status")
                            .and_then(|s| s.as_str())
                            .is_some_and(|s| s != "COMPLETED")
                    })
                });
                let reason = if pending {
                    "checks pending".to_string()
                } else {
                    "blocked by branch protection".to_string()
                };
                Ok(MergeStatus::Blocked { reason })
            }
            "DIRTY" => Ok(MergeStatus::Conflicting),
            "UNSTABLE" => Ok(MergeStatus::Blocked {
                reason: "checks failing".to_string(),
            }),
            _ => Ok(MergeStatus::Unknown),
        }
    }

    /// Update a PR branch with the latest base branch changes.
    ///
    /// This uses `gh pr update-branch` which performs a merge (not rebase)
    /// of the base branch into the PR branch.
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn update_branch(&self, pr_number: u32) -> Result<()> {
        debug!(
            repo = %self.repo_path(),
            pr = pr_number,
            "Updating PR branch"
        );

        let output = Command::new("gh")
            .args([
                "pr",
                "update-branch",
                &pr_number.to_string(),
                "--repo",
                &self.repo_path(),
            ])
            .output()
            .context("Failed to execute gh pr update-branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr update-branch failed: {stderr}");
        }

        info!(
            repo = %self.repo_path(),
            pr = pr_number,
            "Updated PR branch"
        );
        Ok(())
    }

    /// Get the list of conflicting files for a PR.
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn get_conflicting_files(&self, pr_number: u32) -> Result<Vec<String>> {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &pr_number.to_string(),
                "--repo",
                &self.repo_path(),
                "--json",
                "files",
            ])
            .output()
            .context("Failed to execute gh pr view")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).context("Failed to parse PR files JSON")?;

        let files = json
            .get("files")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f.get("path").and_then(|p| p.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(files)
    }

    /// Add a comment to a pull request.
    ///
    /// # Errors
    /// Returns an error if the `gh` CLI command fails.
    pub fn add_pr_comment(&self, pr_number: u32, body: &str) -> Result<()> {
        let output = Command::new("gh")
            .args([
                "pr",
                "comment",
                &pr_number.to_string(),
                "--repo",
                &self.repo_path(),
                "--body",
                body,
            ])
            .output()
            .context("Failed to execute gh pr comment")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr comment failed: {stderr}");
        }

        Ok(())
    }
}

/// Current state of a GitHub PR, fetched via `gh pr view --json`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubState {
    pub pr_number: Option<u32>,
    pub comments: Vec<Comment>,
    pub commits: Vec<Commit>,
    pub checks: Vec<Check>,
    pub reviews: Vec<Review>,
    pub labels: Vec<String>,
    pub mergeable: bool,
    pub merge_state_status: String,
}

/// A PR comment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comment {
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

/// A commit on the PR branch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub committed_at: DateTime<Utc>,
}

/// A CI check
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub conclusion: CheckConclusion,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_summary: Option<String>,
}

/// CI check conclusion
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckConclusion {
    #[default]
    Pending,
    Success,
    Failure,
    Cancelled,
    Skipped,
}

/// A PR review
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Review {
    pub author: String,
    pub state: ReviewState,
    pub submitted_at: Option<DateTime<Utc>>,
}

/// Review state
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewState {
    #[default]
    Pending,
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
}

impl GitHubState {
    /// Parse from `gh pr view --json` output
    pub fn from_gh_output(json: &str) -> anyhow::Result<Self> {
        // TODO: Implement actual parsing
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_method_flags() {
        assert_eq!(MergeMethod::Merge.as_flag(), "--merge");
        assert_eq!(MergeMethod::Squash.as_flag(), "--squash");
        assert_eq!(MergeMethod::Rebase.as_flag(), "--rebase");
    }

    #[test]
    fn test_merge_method_default() {
        let method = MergeMethod::default();
        assert_eq!(method, MergeMethod::Squash);
    }

    #[test]
    fn test_merge_status_equality() {
        assert_eq!(MergeStatus::Ready, MergeStatus::Ready);
        assert_eq!(MergeStatus::Conflicting, MergeStatus::Conflicting);
        assert_ne!(MergeStatus::Ready, MergeStatus::Conflicting);
    }

    #[test]
    fn test_github_client_repo_path() {
        let client = GitHubClient::new("5dlabs", "cto");
        assert_eq!(client.repo_path(), "5dlabs/cto");
    }
}
