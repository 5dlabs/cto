//! Publishing module for creating PRs with research entries.
//!
//! Handles git operations and GitHub PR creation.

mod git;
mod github;

pub use git::GitPublisher;
pub use github::PrCreator;

use anyhow::Result;
use std::path::Path;

/// Configuration for publishing research entries.
#[derive(Debug, Clone)]
pub struct PublishConfig {
    /// GitHub repository (owner/repo format).
    pub repo: String,
    /// Target branch for the PR (defaults to "develop").
    pub base_branch: String,
    /// Directory in the repo where research files go.
    pub research_dir: String,
    /// GitHub token for API access.
    pub github_token: String,
}

impl PublishConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Result<Self> {
        let repo = std::env::var("RESEARCH_REPO").unwrap_or_else(|_| "5dlabs/cto".to_string());
        let base_branch =
            std::env::var("RESEARCH_BASE_BRANCH").unwrap_or_else(|_| "develop".to_string());
        let research_dir =
            std::env::var("RESEARCH_DIR").unwrap_or_else(|_| "docs/research".to_string());
        let github_token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .map_err(|_| anyhow::anyhow!("GITHUB_TOKEN or GH_TOKEN not set"))?;

        Ok(Self {
            repo,
            base_branch,
            research_dir,
            github_token,
        })
    }
}

/// Publisher that commits research entries and creates PRs.
pub struct Publisher {
    config: PublishConfig,
    git: GitPublisher,
    github: PrCreator,
}

impl Publisher {
    /// Create a new publisher.
    pub fn new(config: PublishConfig) -> Result<Self> {
        let github = PrCreator::new(&config.github_token)?;
        let git = GitPublisher::new();

        Ok(Self {
            config,
            git,
            github,
        })
    }

    /// Publish research entries to GitHub as a PR.
    ///
    /// Returns the PR URL if successful.
    pub async fn publish(&self, source_dir: &Path, entries: &[String]) -> Result<String> {
        if entries.is_empty() {
            return Err(anyhow::anyhow!("No entries to publish"));
        }

        let (owner, repo) = self.parse_repo()?;
        let branch_name = format!("research/{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));

        tracing::info!(
            repo = %self.config.repo,
            branch = %branch_name,
            entries = entries.len(),
            "Publishing research entries"
        );

        // Clone the repo
        let work_dir = tempfile::tempdir()?;
        let repo_path = work_dir.path().join("repo");

        self.git
            .clone_repo(&owner, &repo, &self.config.github_token, &repo_path)
            .await?;

        // Create branch
        self.git
            .create_branch(&repo_path, &branch_name, &self.config.base_branch)
            .await?;

        // Copy research files
        let target_dir = repo_path.join(&self.config.research_dir);
        std::fs::create_dir_all(&target_dir)?;

        for entry_id in entries {
            // Find the source file (could be in dated subdirectory)
            let source_file = Self::find_entry_file(source_dir, entry_id)?;
            let target_file = target_dir.join(source_file.file_name().unwrap());

            tracing::debug!(
                source = %source_file.display(),
                target = %target_file.display(),
                "Copying research entry"
            );

            // Ensure parent directories exist
            if let Some(parent) = target_file.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::copy(&source_file, &target_file)?;
        }

        // Commit and push
        let commit_msg = format!(
            "docs(research): add {} new research entries\n\n{}",
            entries.len(),
            entries
                .iter()
                .map(|id| format!("- {id}"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        self.git.commit(&repo_path, &commit_msg).await?;
        self.git
            .push(&repo_path, &branch_name, &self.config.github_token)
            .await?;

        // Create PR
        let pr_title = format!(
            "docs(research): {} new entries ({})",
            entries.len(),
            chrono::Utc::now().format("%Y-%m-%d")
        );

        let pr_body = Self::generate_pr_body(entries);

        let pr_url = self
            .github
            .create_pr(
                &owner,
                &repo,
                &pr_title,
                &pr_body,
                &branch_name,
                &self.config.base_branch,
            )
            .await?;

        tracing::info!(pr_url = %pr_url, "Created pull request");

        Ok(pr_url)
    }

    fn parse_repo(&self) -> Result<(String, String)> {
        let parts: Vec<&str> = self.config.repo.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid repo format '{}', expected 'owner/repo'",
                self.config.repo
            ));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    fn find_entry_file(source_dir: &Path, entry_id: &str) -> Result<std::path::PathBuf> {
        // Search for the file in source_dir and subdirectories
        for entry in walkdir::WalkDir::new(source_dir)
            .max_depth(5)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                if name.starts_with(entry_id) && name.ends_with(".md") {
                    return Ok(entry.path().to_path_buf());
                }
            }
        }

        Err(anyhow::anyhow!("Entry file not found: {}", entry_id))
    }

    fn generate_pr_body(entries: &[String]) -> String {
        use std::fmt::Write;

        let mut body = String::new();
        body.push_str("## 🔬 New Research Entries\n\n");
        body.push_str("This PR adds research entries curated from Twitter/X bookmarks.\n\n");
        body.push_str("### Entries\n\n");

        for entry in entries {
            let _ = writeln!(body, "- `{entry}`");
        }

        body.push_str("\n---\n");
        body.push_str("*Generated by CTO Research Pipeline*\n");

        body
    }
}
