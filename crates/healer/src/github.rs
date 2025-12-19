//! GitHub types and state management for the monitor.

#![allow(dead_code)] // Public API - methods used for GitHub polling integration

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info};

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
            anyhow::bail!("gh issue create failed: {}", stderr);
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
            anyhow::bail!("gh issue comment failed: {}", stderr);
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
            anyhow::bail!("gh issue close failed: {}", stderr);
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
