//! GitHub PR creation using octocrab.

use anyhow::{Context, Result};
use octocrab::Octocrab;

/// Creates pull requests on GitHub.
pub struct PrCreator {
    client: Octocrab,
}

impl PrCreator {
    /// Create a new PR creator with the given token.
    pub fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .context("Failed to create GitHub client")?;

        Ok(Self { client })
    }

    /// Create a pull request.
    ///
    /// Returns the PR URL.
    pub async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<String> {
        tracing::debug!(
            owner = %owner,
            repo = %repo,
            head = %head,
            base = %base,
            "Creating pull request"
        );

        let pr = self
            .client
            .pulls(owner, repo)
            .create(title, head, base)
            .body(body)
            .send()
            .await
            .context("Failed to create pull request")?;

        let pr_url = pr.html_url.map_or_else(
            || format!("https://github.com/{owner}/{repo}/pull/{}", pr.number),
            |url| url.to_string(),
        );

        // Add labels
        let labels: Vec<String> = vec!["research".to_string(), "automated".to_string()];
        if let Err(e) = self.add_labels(owner, repo, pr.number, &labels).await {
            tracing::warn!(error = %e, "Failed to add labels to PR");
        }

        Ok(pr_url)
    }

    /// Add labels to a PR.
    async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        labels: &[String],
    ) -> Result<()> {
        self.client
            .issues(owner, repo)
            .add_labels(pr_number, labels)
            .await
            .context("Failed to add labels")?;

        Ok(())
    }
}
