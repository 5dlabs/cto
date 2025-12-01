//! # PR Alerts (Annotations)
//!
//! Fetch check run annotations from GitHub PRs using the `gh` CLI.
//!
//! The "Alerts" button in GitHub's PR UI shows check run annotations -
//! warnings, errors, and notices from CI/CD checks. This module retrieves
//! them programmatically.
//!
//! ## Example
//!
//! ```no_run
//! use utils::PrAlerts;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let alerts = PrAlerts::new("5dlabs", "cto");
//!
//! // Get all annotations for a PR
//! let annotations = alerts.fetch(1864).await?;
//!
//! for ann in annotations {
//!     println!("{}: {} at {}:{}", ann.level, ann.message, ann.path, ann.start_line);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Annotation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationLevel {
    /// Informational notice
    Notice,
    /// Warning that should be addressed
    Warning,
    /// Failure that blocks the check
    Failure,
}

impl std::fmt::Display for AnnotationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notice => write!(f, "notice"),
            Self::Warning => write!(f, "warning"),
            Self::Failure => write!(f, "failure"),
        }
    }
}

/// A check run annotation from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// File path (e.g., `src/main.rs`)
    pub path: String,

    /// Starting line number
    pub start_line: u32,

    /// Ending line number
    pub end_line: u32,

    /// Severity level
    #[serde(rename = "annotation_level")]
    pub level: AnnotationLevel,

    /// The alert message
    pub message: String,

    /// Optional title
    #[serde(default)]
    pub title: String,

    /// Additional details
    #[serde(default)]
    pub raw_details: String,
}

/// Summary of a check run with annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRun {
    /// Check run ID
    pub id: u64,

    /// Check name (e.g., "Clippy", "Test")
    pub name: String,

    /// Number of annotations
    #[serde(rename = "annotations_count")]
    pub count: u32,
}

/// Check runs API response
#[derive(Debug, Deserialize)]
struct CheckRunsResponse {
    check_runs: Vec<CheckRunRaw>,
}

/// Raw check run from API
#[derive(Debug, Deserialize)]
struct CheckRunRaw {
    id: u64,
    name: String,
    output: CheckRunOutput,
}

/// Check run output containing annotation count
#[derive(Debug, Deserialize)]
struct CheckRunOutput {
    annotations_count: u32,
}

/// Client for fetching PR alerts from GitHub
#[derive(Debug, Clone)]
pub struct PrAlerts {
    owner: String,
    repo: String,
}

impl PrAlerts {
    /// Create a new PR alerts client
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (e.g., "5dlabs")
    /// * `repo` - Repository name (e.g., "cto")
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Fetch all annotations for a PR
    ///
    /// # Arguments
    ///
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A vector of all annotations across all check runs
    pub async fn fetch(&self, pr_number: u32) -> Result<Vec<Annotation>> {
        let head_sha = self.get_head_sha(pr_number).await?;
        info!(pr = pr_number, sha = %head_sha, "Fetching alerts for PR");

        let check_runs = self.get_check_runs_with_annotations(&head_sha).await?;

        if check_runs.is_empty() {
            debug!(pr = pr_number, "No check runs with annotations found");
            return Ok(Vec::new());
        }

        info!(
            pr = pr_number,
            count = check_runs.len(),
            "Found check runs with annotations"
        );

        let mut all_annotations = Vec::new();
        for run in check_runs {
            let annotations = self.get_annotations(run.id).await?;
            debug!(
                check_run = run.name,
                count = annotations.len(),
                "Fetched annotations"
            );
            all_annotations.extend(annotations);
        }

        info!(
            pr = pr_number,
            total = all_annotations.len(),
            "Total annotations fetched"
        );
        Ok(all_annotations)
    }

    /// Fetch annotations filtered by level
    ///
    /// # Arguments
    ///
    /// * `pr_number` - The pull request number
    /// * `level` - Only return annotations at this level
    pub async fn fetch_by_level(
        &self,
        pr_number: u32,
        level: AnnotationLevel,
    ) -> Result<Vec<Annotation>> {
        let all = self.fetch(pr_number).await?;
        Ok(all.into_iter().filter(|a| a.level == level).collect())
    }

    /// Fetch only failure annotations
    pub async fn fetch_failures(&self, pr_number: u32) -> Result<Vec<Annotation>> {
        self.fetch_by_level(pr_number, AnnotationLevel::Failure)
            .await
    }

    /// Fetch only warning annotations
    pub async fn fetch_warnings(&self, pr_number: u32) -> Result<Vec<Annotation>> {
        self.fetch_by_level(pr_number, AnnotationLevel::Warning)
            .await
    }

    /// Get summary of check runs with annotations
    ///
    /// Returns information about each check run that has annotations,
    /// without fetching the actual annotation content.
    pub async fn get_summary(&self, pr_number: u32) -> Result<Vec<CheckRun>> {
        let head_sha = self.get_head_sha(pr_number).await?;
        self.get_check_runs_with_annotations(&head_sha).await
    }

    /// Get the head SHA for a PR
    pub async fn get_head_sha(&self, pr_number: u32) -> Result<String> {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &pr_number.to_string(),
                "--repo",
                &format!("{}/{}", self.owner, self.repo),
                "--json",
                "headRefOid",
                "-q",
                ".headRefOid",
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

        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if sha.is_empty() {
            anyhow::bail!("Empty SHA returned for PR {pr_number}");
        }

        Ok(sha)
    }

    /// Get check runs that have annotations
    async fn get_check_runs_with_annotations(&self, sha: &str) -> Result<Vec<CheckRun>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/commits/{sha}/check-runs",
                    self.owner, self.repo
                ),
                "--paginate",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh api check-runs failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse paginated response - gh api --paginate returns concatenated JSON
        let mut check_runs = Vec::new();
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let response: CheckRunsResponse =
                serde_json::from_str(line).context("Failed to parse check runs response")?;

            for run in response.check_runs {
                if run.output.annotations_count > 0 {
                    check_runs.push(CheckRun {
                        id: run.id,
                        name: run.name,
                        count: run.output.annotations_count,
                    });
                }
            }
        }

        Ok(check_runs)
    }

    /// Get annotations for a specific check run
    async fn get_annotations(&self, check_run_id: u64) -> Result<Vec<Annotation>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/check-runs/{check_run_id}/annotations",
                    self.owner, self.repo
                ),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                check_run_id,
                error = %stderr,
                "Failed to fetch annotations for check run"
            );
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let annotations: Vec<Annotation> =
            serde_json::from_str(&stdout).context("Failed to parse annotations")?;

        Ok(annotations)
    }
}

/// Parse repository string "owner/repo" into tuple
pub fn parse_repo(repo_str: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = repo_str.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid repository format. Expected 'owner/repo', got: {repo_str}");
    }
    Ok((parts[0], parts[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo() {
        let (owner, repo) = parse_repo("5dlabs/cto").unwrap();
        assert_eq!(owner, "5dlabs");
        assert_eq!(repo, "cto");
    }

    #[test]
    fn test_parse_repo_invalid() {
        assert!(parse_repo("invalid").is_err());
        assert!(parse_repo("too/many/parts").is_err());
    }

    #[test]
    fn test_annotation_level_display() {
        assert_eq!(format!("{}", AnnotationLevel::Notice), "notice");
        assert_eq!(format!("{}", AnnotationLevel::Warning), "warning");
        assert_eq!(format!("{}", AnnotationLevel::Failure), "failure");
    }

    #[test]
    fn test_annotation_deserialize() {
        let json = r#"{
            "path": "src/main.rs",
            "start_line": 10,
            "end_line": 10,
            "annotation_level": "failure",
            "message": "unused variable",
            "title": "",
            "raw_details": ""
        }"#;

        let ann: Annotation = serde_json::from_str(json).unwrap();
        assert_eq!(ann.path, "src/main.rs");
        assert_eq!(ann.start_line, 10);
        assert_eq!(ann.level, AnnotationLevel::Failure);
        assert_eq!(ann.message, "unused variable");
    }

    #[test]
    fn test_pr_alerts_new() {
        let alerts = PrAlerts::new("owner", "repo");
        assert_eq!(alerts.owner, "owner");
        assert_eq!(alerts.repo, "repo");
    }
}
