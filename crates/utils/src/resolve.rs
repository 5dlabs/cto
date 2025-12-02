//! # PR Conversation Resolution
//!
//! Resolve PR review thread conversations via GitHub's GraphQL API.
//!
//! This module allows reviewers (like Stitch) to programmatically resolve
//! conversations after confirming that issues have been addressed.
//!
//! ## Example
//!
//! ```no_run
//! use utils::PrConversations;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = PrConversations::new("5dlabs", "cto");
//!
//! // List unresolved conversations
//! let threads = client.list_unresolved(1956).await?;
//! println!("Found {} unresolved conversations", threads.len());
//!
//! // Resolve a specific thread after confirming fix
//! client.resolve(&threads[0].id).await?;
//!
//! // Or resolve all at once
//! let resolved = client.resolve_all(1956).await?;
//! println!("Resolved {} conversations", resolved);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// A PR review thread (conversation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewThread {
    /// Thread ID (GraphQL node ID)
    pub id: String,

    /// Whether the thread is resolved
    pub is_resolved: bool,

    /// File path (if inline comment)
    #[serde(default)]
    pub path: Option<String>,

    /// Line number (if inline comment)
    #[serde(default)]
    pub line: Option<u32>,

    /// First comment body (preview)
    #[serde(default)]
    pub body_preview: String,

    /// Author of the first comment
    #[serde(default)]
    pub author: String,
}

/// Client for managing PR conversations
#[derive(Debug, Clone)]
pub struct PrConversations {
    owner: String,
    repo: String,
}

impl PrConversations {
    /// Create a new conversations client
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// List all review threads for a PR
    pub async fn list(&self, pr_number: u32) -> Result<Vec<ReviewThread>> {
        let query = r"
            query GetReviewThreads($owner: String!, $repo: String!, $pullNumber: Int!) {
                repository(owner: $owner, name: $repo) {
                    pullRequest(number: $pullNumber) {
                        reviewThreads(first: 100) {
                            nodes {
                                id
                                isResolved
                                path
                                line
                                comments(first: 1) {
                                    nodes {
                                        body
                                        author {
                                            login
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ";

        let output = Command::new("gh")
            .args([
                "api",
                "graphql",
                "-f",
                &format!("query={query}"),
                "-f",
                &format!("owner={}", self.owner),
                "-f",
                &format!("repo={}", self.repo),
                "-F",
                &format!("pullNumber={pr_number}"),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api graphql")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("GraphQL query failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: GraphQLResponse =
            serde_json::from_str(&stdout).context("Failed to parse GraphQL response")?;

        let threads = response
            .data
            .repository
            .pull_request
            .review_threads
            .nodes
            .into_iter()
            .map(|node| {
                let first_comment = node.comments.nodes.first();
                ReviewThread {
                    id: node.id,
                    is_resolved: node.is_resolved,
                    path: node.path,
                    line: node.line,
                    body_preview: first_comment
                        .map(|c| truncate_body(&c.body, 100))
                        .unwrap_or_default(),
                    author: first_comment
                        .and_then(|c| c.author.as_ref())
                        .map(|a| a.login.clone())
                        .unwrap_or_default(),
                }
            })
            .collect();

        Ok(threads)
    }

    /// List only unresolved review threads
    pub async fn list_unresolved(&self, pr_number: u32) -> Result<Vec<ReviewThread>> {
        let all = self.list(pr_number).await?;
        Ok(all.into_iter().filter(|t| !t.is_resolved).collect())
    }

    /// Resolve a single review thread by ID
    pub async fn resolve(&self, thread_id: &str) -> Result<bool> {
        let mutation = r"
            mutation ResolveReviewThread($threadId: ID!) {
                resolveReviewThread(input: {threadId: $threadId}) {
                    thread {
                        isResolved
                    }
                }
            }
        ";

        let output = Command::new("gh")
            .args([
                "api",
                "graphql",
                "-f",
                &format!("query={mutation}"),
                "-f",
                &format!("threadId={thread_id}"),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api graphql")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Some threads can't be resolved (not a conversation)
            if stderr.contains("not a conversation") {
                warn!(thread_id, "Thread is not a conversation, skipping");
                return Ok(false);
            }
            anyhow::bail!("Failed to resolve thread {thread_id}: {stderr}");
        }

        debug!(thread_id, "Successfully resolved thread");
        Ok(true)
    }

    /// Resolve all unresolved threads for a PR
    ///
    /// Returns the number of threads successfully resolved
    pub async fn resolve_all(&self, pr_number: u32) -> Result<u32> {
        let threads = self.list_unresolved(pr_number).await?;
        info!(
            pr = pr_number,
            count = threads.len(),
            "Resolving all unresolved threads"
        );

        let mut resolved_count = 0;
        for thread in threads {
            match self.resolve(&thread.id).await {
                Ok(true) => resolved_count += 1,
                Ok(false) => {} // Skipped (not a conversation)
                Err(e) => warn!(thread_id = thread.id, error = %e, "Failed to resolve thread"),
            }
        }

        info!(pr = pr_number, resolved = resolved_count, "Done resolving");
        Ok(resolved_count)
    }

    /// Resolve threads by a specific author
    pub async fn resolve_by_author(&self, pr_number: u32, author: &str) -> Result<u32> {
        let threads = self.list_unresolved(pr_number).await?;
        let author_lower = author.to_lowercase();

        let matching: Vec<_> = threads
            .into_iter()
            .filter(|t| t.author.to_lowercase().contains(&author_lower))
            .collect();

        info!(
            pr = pr_number,
            author,
            count = matching.len(),
            "Resolving threads by author"
        );

        let mut resolved_count = 0;
        for thread in matching {
            match self.resolve(&thread.id).await {
                Ok(true) => resolved_count += 1,
                Ok(false) => {}
                Err(e) => warn!(thread_id = thread.id, error = %e, "Failed to resolve thread"),
            }
        }

        Ok(resolved_count)
    }
}

/// Truncate a string to max length with ellipsis
fn truncate_body(body: &str, max_len: usize) -> String {
    let first_line = body.lines().next().unwrap_or(body);
    if first_line.len() <= max_len {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max_len - 3])
    }
}

// GraphQL response types
#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    repository: Repository,
}

#[derive(Debug, Deserialize)]
struct Repository {
    #[serde(rename = "pullRequest")]
    pull_request: PullRequest,
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    #[serde(rename = "reviewThreads")]
    review_threads: ReviewThreads,
}

#[derive(Debug, Deserialize)]
struct ReviewThreads {
    nodes: Vec<ReviewThreadNode>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadNode {
    id: String,
    #[serde(rename = "isResolved")]
    is_resolved: bool,
    path: Option<String>,
    line: Option<u32>,
    comments: Comments,
}

#[derive(Debug, Deserialize)]
struct Comments {
    nodes: Vec<CommentNode>,
}

#[derive(Debug, Deserialize)]
struct CommentNode {
    body: String,
    author: Option<Author>,
}

#[derive(Debug, Deserialize)]
struct Author {
    login: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_body() {
        assert_eq!(truncate_body("short", 10), "short");
        assert_eq!(
            truncate_body("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_body("line 1\nline 2\nline 3", 100), "line 1");
    }

    #[test]
    fn test_client_new() {
        let client = PrConversations::new("owner", "repo");
        assert_eq!(client.owner, "owner");
        assert_eq!(client.repo, "repo");
    }
}
