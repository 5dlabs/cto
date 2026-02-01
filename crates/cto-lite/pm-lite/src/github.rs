//! GitHub webhook handling

use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// GitHub webhook event types we care about
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubEvent {
    /// Issue opened or edited
    Issues,
    /// Issue comment created
    IssueComment,
    /// Pull request opened, edited, or synchronized
    PullRequest,
    /// PR review comment
    PullRequestReview,
    /// Push to repository
    Push,
    /// Unknown event type
    Unknown(String),
}

impl From<&str> for GitHubEvent {
    fn from(s: &str) -> Self {
        match s {
            "issues" => Self::Issues,
            "issue_comment" => Self::IssueComment,
            "pull_request" => Self::PullRequest,
            "pull_request_review" => Self::PullRequestReview,
            "push" => Self::Push,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// Issue webhook payload
#[derive(Debug, Clone, Deserialize)]
pub struct IssuePayload {
    pub action: String,
    pub issue: Issue,
    pub repository: Repository,
    pub sender: User,
}

/// Issue comment webhook payload
#[derive(Debug, Clone, Deserialize)]
pub struct IssueCommentPayload {
    pub action: String,
    pub issue: Issue,
    pub comment: Comment,
    pub repository: Repository,
    pub sender: User,
}

/// Pull request webhook payload
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

/// GitHub Issue
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Issue {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub html_url: String,
}

/// GitHub Pull Request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PullRequest {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: User,
    pub head: GitRef,
    pub base: GitRef,
    pub html_url: String,
    pub draft: Option<bool>,
}

/// Git reference (branch)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

/// GitHub Comment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Comment {
    pub id: i64,
    pub body: String,
    pub user: User,
    pub html_url: String,
}

/// GitHub Repository
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub default_branch: String,
    pub html_url: String,
    pub clone_url: String,
}

/// GitHub User
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub login: String,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
}

/// GitHub Label
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Label {
    pub name: String,
    pub color: Option<String>,
}

/// Verify GitHub webhook signature
///
/// # Errors
/// Returns error if signature is invalid or missing
pub fn verify_signature(secret: &str, signature: &str, body: &[u8]) -> Result<()> {
    // GitHub signature format: sha256=<hex>
    let expected = signature
        .strip_prefix("sha256=")
        .ok_or_else(|| anyhow!("Invalid signature format"))?;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| anyhow!("HMAC error: {e}"))?;
    mac.update(body);

    let result = mac.finalize();
    let computed = hex::encode(result.into_bytes());

    if computed == expected {
        Ok(())
    } else {
        Err(anyhow!("Signature mismatch"))
    }
}

/// Check if an issue/PR should trigger CTO workflow
#[must_use]
pub fn should_trigger_workflow(issue: &Issue) -> bool {
    // Trigger on issues with "cto" label
    issue.labels.iter().any(|l| l.name.to_lowercase() == "cto")
}

/// Check if a comment is a command to trigger workflow
#[must_use]
pub fn is_trigger_command(body: &str) -> bool {
    let body_lower = body.to_lowercase();
    body_lower.contains("/cto") || body_lower.contains("@cto")
}

/// Extract prompt from issue body or comment
#[must_use]
pub fn extract_prompt(body: &str) -> String {
    // Remove common prefixes
    let cleaned = body
        .replace("/cto", "")
        .replace("@cto", "")
        .trim()
        .to_string();

    if cleaned.is_empty() {
        "Implement the feature described in this issue".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature() {
        let secret = "test-secret";
        let body = b"test body";

        // Generate valid signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        assert!(verify_signature(secret, &signature, body).is_ok());
    }

    #[test]
    fn test_is_trigger_command() {
        assert!(is_trigger_command("/cto implement this"));
        assert!(is_trigger_command("@cto please fix"));
        assert!(!is_trigger_command("just a normal comment"));
    }
}
