//! GitHub types and state management for the monitor.

#![allow(dead_code)] // Public API - methods used for GitHub polling integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
