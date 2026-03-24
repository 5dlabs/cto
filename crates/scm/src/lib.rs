pub mod github;
pub mod gitlab;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Which SCM platform is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScmProvider {
    GitHub,
    GitLab,
}

impl Default for ScmProvider {
    fn default() -> Self {
        Self::GitHub
    }
}

impl std::fmt::Display for ScmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::GitLab => write!(f, "gitlab"),
        }
    }
}

impl std::str::FromStr for ScmProvider {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "gitlab" => Ok(Self::GitLab),
            other => Err(format!("unknown SCM provider: {other}")),
        }
    }
}

/// Configuration needed to construct an SCM client.
#[derive(Debug, Clone)]
pub struct ScmClientConfig {
    pub provider: ScmProvider,
    pub host: String,
    pub api_base: String,
    pub org_or_group: String,
    pub token: Option<String>,
}

/// A merge/pull request in provider-neutral form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub url: String,
    pub author: String,
}

/// Basic repository info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub full_name: String,
    pub url: String,
    pub clone_url: String,
    pub default_branch: String,
}

/// A code search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub path: String,
    pub repository: String,
    pub url: String,
    pub fragment: Option<String>,
}

/// Provider-agnostic SCM operations.
///
/// Implementations exist for GitHub (REST v3 via reqwest) and GitLab (REST v4).
#[async_trait]
pub trait ScmClient: Send + Sync {
    /// List open merge/pull requests, optionally filtered by head branch.
    async fn list_open_mrs(
        &self,
        owner: &str,
        repo: &str,
        head_branch: Option<&str>,
    ) -> Result<Vec<MergeRequest>>;

    /// Create a new merge/pull request.
    async fn create_mr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<MergeRequest>;

    /// Fetch file contents (returned as raw bytes, decoded from base64).
    async fn get_file_contents(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        ref_: &str,
    ) -> Result<Vec<u8>>;

    /// Create a new repository/project.
    async fn create_repo(&self, org: &str, name: &str, private: bool) -> Result<RepoInfo>;

    /// Register a webhook on a repository.
    async fn create_webhook(
        &self,
        owner: &str,
        repo: &str,
        url: &str,
        events: &[&str],
        secret: Option<&str>,
    ) -> Result<()>;

    /// Search code across the org/group.
    async fn search_code(&self, query: &str) -> Result<Vec<CodeSearchResult>>;

    /// Create an issue on the repository.
    async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<String>;

    /// Build the web URL for a merge/pull request.
    fn mr_url(&self, owner: &str, repo: &str, number: u64) -> String;

    /// Build the web URL for a repository.
    fn repo_url(&self, owner: &str, repo: &str) -> String;

    /// Build a clone URL, optionally embedding a token.
    fn clone_url(&self, owner: &str, repo: &str, token: Option<&str>) -> String;

    /// Parse owner and repo from a full URL (HTTPS or SSH).
    fn parse_repo_from_url(&self, url: &str) -> Result<(String, String)>;

    /// The provider type.
    fn provider(&self) -> ScmProvider;
}

/// Build the appropriate SCM client from config.
pub fn create_scm_client(config: &ScmClientConfig) -> Box<dyn ScmClient> {
    match config.provider {
        ScmProvider::GitHub => Box::new(github::GitHubClient::new(config)),
        ScmProvider::GitLab => Box::new(gitlab::GitLabClient::new(config)),
    }
}
