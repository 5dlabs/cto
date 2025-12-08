//! # PR Review Comments
//!
//! Fetch review comments from GitHub PRs using the `gh` CLI.
//!
//! This module retrieves inline review comments from code reviewers like
//! Bugbot (Cursor's reviewer) and Stitch (5DLabs' reviewer) with structured
//! data including file path, line number, and suggestion blocks.
//!
//! ## Example
//!
//! ```no_run
//! use utils::PrReviews;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let reviews = PrReviews::new("5dlabs", "cto");
//!
//! // Get all review comments from Bugbot and Stitch
//! let comments = reviews.fetch(1956).await?;
//!
//! for comment in comments {
//!     println!("{}: {} at {}:{}", comment.author, comment.body, comment.path, comment.line.unwrap_or(0));
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Known code review bot usernames
pub const BUGBOT_AUTHORS: &[&str] = &["bug-bot", "cursor[bot]", "cursor"];
pub const STITCH_AUTHORS: &[&str] = &["stitch-5dlabs[bot]", "5DLabs-Stitch[bot]", "stitch-5dlabs"];

/// A review comment from a PR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    /// Comment ID
    pub id: u64,

    /// Author username
    pub author: String,

    /// File path (for inline comments)
    #[serde(default)]
    pub path: String,

    /// Line number in the file
    #[serde(default)]
    pub line: Option<u32>,

    /// Starting line for multi-line comments
    #[serde(default)]
    pub start_line: Option<u32>,

    /// Comment body/content
    pub body: String,

    /// Extracted suggestion block (if present)
    #[serde(default)]
    pub suggestion: Option<String>,

    /// When the comment was created
    #[serde(default)]
    pub created_at: String,

    /// Diff hunk context
    #[serde(default)]
    pub diff_hunk: Option<String>,

    /// Whether this is from Bugbot
    #[serde(default)]
    pub is_bugbot: bool,

    /// Whether this is from Stitch
    #[serde(default)]
    pub is_stitch: bool,
}

/// Raw review comment from GitHub API
#[derive(Debug, Deserialize)]
struct RawReviewComment {
    id: u64,
    user: RawUser,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    start_line: Option<u32>,
    body: String,
    created_at: String,
    #[serde(default)]
    diff_hunk: Option<String>,
}

/// Raw user from GitHub API
#[derive(Debug, Deserialize)]
struct RawUser {
    login: String,
}

/// Client for fetching PR review comments from GitHub
#[derive(Debug, Clone)]
pub struct PrReviews {
    owner: String,
    repo: String,
}

impl PrReviews {
    /// Create a new PR reviews client
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

    /// Fetch all review comments from Bugbot and Stitch
    ///
    /// # Arguments
    ///
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A vector of review comments from known reviewers
    pub async fn fetch(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        info!(pr = pr_number, "Fetching review comments for PR");

        let comments = self.get_review_comments(pr_number).await?;

        // Filter for Bugbot and Stitch comments
        let filtered: Vec<ReviewComment> = comments
            .into_iter()
            .filter(|c| c.is_bugbot || c.is_stitch)
            .collect();

        info!(
            pr = pr_number,
            total = filtered.len(),
            "Found review comments from Bugbot/Stitch"
        );

        Ok(filtered)
    }

    /// Fetch all review comments (unfiltered)
    pub async fn fetch_all(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        info!(pr = pr_number, "Fetching all review comments for PR");
        self.get_review_comments(pr_number).await
    }

    /// Fetch only Bugbot comments
    pub async fn fetch_bugbot(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        let all = self.fetch_all(pr_number).await?;
        Ok(all.into_iter().filter(|c| c.is_bugbot).collect())
    }

    /// Fetch only Stitch comments
    pub async fn fetch_stitch(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        let all = self.fetch_all(pr_number).await?;
        Ok(all.into_iter().filter(|c| c.is_stitch).collect())
    }

    /// Fetch comments by specific author
    pub async fn fetch_by_author(
        &self,
        pr_number: u32,
        author: &str,
    ) -> Result<Vec<ReviewComment>> {
        let all = self.fetch_all(pr_number).await?;
        let author_lower = author.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|c| c.author.to_lowercase().contains(&author_lower))
            .collect())
    }

    /// Fetch only inline comments (comments on specific lines)
    pub async fn fetch_inline(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        let comments = self.fetch(pr_number).await?;
        Ok(comments
            .into_iter()
            .filter(|c| c.line.is_some() && !c.path.is_empty())
            .collect())
    }

    /// Get review comments from GitHub API
    async fn get_review_comments(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/pulls/{pr_number}/comments",
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
            anyhow::bail!("gh api pulls/comments failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse paginated response
        let mut all_comments = Vec::new();
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as array first (normal response)
            if let Ok(raw_comments) = serde_json::from_str::<Vec<RawReviewComment>>(line) {
                for raw in raw_comments {
                    all_comments.push(self.convert_comment(raw));
                }
            } else {
                // Try single object
                if let Ok(raw) = serde_json::from_str::<RawReviewComment>(line) {
                    all_comments.push(self.convert_comment(raw));
                } else {
                    debug!(line = line, "Skipping unparseable line");
                }
            }
        }

        // Also fetch issue comments (general PR comments, not inline)
        let issue_comments = self.get_issue_comments(pr_number).await?;
        all_comments.extend(issue_comments);

        debug!(
            pr = pr_number,
            count = all_comments.len(),
            "Fetched review comments"
        );

        Ok(all_comments)
    }

    /// Get issue comments (general PR comments)
    async fn get_issue_comments(&self, pr_number: u32) -> Result<Vec<ReviewComment>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/issues/{pr_number}/comments",
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
            warn!(error = %stderr, "Failed to fetch issue comments");
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut all_comments = Vec::new();
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(raw_comments) = serde_json::from_str::<Vec<RawIssueComment>>(line) {
                for raw in raw_comments {
                    all_comments.push(self.convert_issue_comment(raw));
                }
            }
        }

        Ok(all_comments)
    }

    /// Convert raw API response to ReviewComment
    fn convert_comment(&self, raw: RawReviewComment) -> ReviewComment {
        let author = raw.user.login.clone();
        let is_bugbot = is_bugbot_author(&author);
        let is_stitch = is_stitch_author(&author);
        let suggestion = extract_suggestion(&raw.body);

        ReviewComment {
            id: raw.id,
            author,
            path: raw.path.unwrap_or_default(),
            line: raw.line,
            start_line: raw.start_line,
            body: raw.body,
            suggestion,
            created_at: raw.created_at,
            diff_hunk: raw.diff_hunk,
            is_bugbot,
            is_stitch,
        }
    }

    /// Convert issue comment to ReviewComment
    fn convert_issue_comment(&self, raw: RawIssueComment) -> ReviewComment {
        let author = raw.user.login.clone();
        let is_bugbot = is_bugbot_author(&author);
        let is_stitch = is_stitch_author(&author);
        let suggestion = extract_suggestion(&raw.body);

        ReviewComment {
            id: raw.id,
            author,
            path: String::new(), // Issue comments don't have a path
            line: None,
            start_line: None,
            body: raw.body,
            suggestion,
            created_at: raw.created_at,
            diff_hunk: None,
            is_bugbot,
            is_stitch,
        }
    }
}

/// Raw issue comment from GitHub API
#[derive(Debug, Deserialize)]
struct RawIssueComment {
    id: u64,
    user: RawUser,
    body: String,
    created_at: String,
}

/// Check if author is Bugbot
fn is_bugbot_author(author: &str) -> bool {
    let author_lower = author.to_lowercase();
    BUGBOT_AUTHORS
        .iter()
        .any(|&b| author_lower.contains(&b.to_lowercase()))
}

/// Check if author is Stitch
fn is_stitch_author(author: &str) -> bool {
    let author_lower = author.to_lowercase();
    STITCH_AUTHORS
        .iter()
        .any(|&s| author_lower.contains(&s.to_lowercase()))
}

/// Extract suggestion block from comment body
///
/// Looks for GitHub suggestion blocks:
/// ```suggestion
/// suggested code here
/// ```
fn extract_suggestion(body: &str) -> Option<String> {
    // Look for ```suggestion blocks
    let suggestion_start = "```suggestion";
    let code_end = "```";

    if let Some(start_idx) = body.find(suggestion_start) {
        let after_marker = &body[start_idx + suggestion_start.len()..];
        // Skip any newline after the marker
        let content_start = after_marker.find('\n').map_or(0, |i| i + 1);
        let content = &after_marker[content_start..];

        if let Some(end_idx) = content.find(code_end) {
            let suggestion = content[..end_idx].trim().to_string();
            if !suggestion.is_empty() {
                return Some(suggestion);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bugbot_author() {
        assert!(is_bugbot_author("bug-bot"));
        assert!(is_bugbot_author("cursor[bot]"));
        assert!(is_bugbot_author("cursor"));
        assert!(!is_bugbot_author("some-user"));
        assert!(!is_bugbot_author("stitch-5dlabs"));
    }

    #[test]
    fn test_is_stitch_author() {
        assert!(is_stitch_author("stitch-5dlabs[bot]"));
        assert!(is_stitch_author("5DLabs-Stitch[bot]"));
        assert!(is_stitch_author("stitch-5dlabs"));
        assert!(!is_stitch_author("some-user"));
        assert!(!is_stitch_author("bug-bot"));
    }

    #[test]
    fn test_extract_suggestion() {
        let body = r"This code has an issue.

```suggestion
let fixed = better_code();
```

Please fix it.";

        let suggestion = extract_suggestion(body);
        assert_eq!(suggestion, Some("let fixed = better_code();".to_string()));
    }

    #[test]
    fn test_extract_suggestion_none() {
        let body = "This is a regular comment without a suggestion block.";
        assert_eq!(extract_suggestion(body), None);
    }

    #[test]
    fn test_extract_suggestion_multiline() {
        let body = r"
```suggestion
fn example() {
    do_something();
}
```
";

        let suggestion = extract_suggestion(body);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert!(s.contains("fn example()"));
        assert!(s.contains("do_something()"));
    }

    #[test]
    fn test_pr_reviews_new() {
        let reviews = PrReviews::new("owner", "repo");
        assert_eq!(reviews.owner, "owner");
        assert_eq!(reviews.repo, "repo");
    }

    #[test]
    fn test_review_comment_serialize() {
        let comment = ReviewComment {
            id: 123,
            author: "bug-bot".to_string(),
            path: "src/main.rs".to_string(),
            line: Some(42),
            start_line: Some(40),
            body: "Fix this".to_string(),
            suggestion: Some("let x = 1;".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            diff_hunk: None,
            is_bugbot: true,
            is_stitch: false,
        };

        let json = serde_json::to_string(&comment).unwrap();
        assert!(json.contains("\"author\":\"bug-bot\""));
        assert!(json.contains("\"is_bugbot\":true"));
    }
}
