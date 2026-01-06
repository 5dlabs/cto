//! Git operations using shell commands.
//!
//! Uses tokio::process::Command for async git operations.

use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Handles git operations via shell commands.
pub struct GitPublisher {
    /// Git author name.
    author_name: String,
    /// Git author email.
    author_email: String,
}

impl GitPublisher {
    /// Create a new git publisher.
    pub fn new() -> Self {
        Self {
            author_name: std::env::var("GIT_AUTHOR_NAME")
                .unwrap_or_else(|_| "CTO Research Bot".to_string()),
            author_email: std::env::var("GIT_AUTHOR_EMAIL")
                .unwrap_or_else(|_| "research@5dlabs.io".to_string()),
        }
    }

    /// Clone a repository with a specific base branch.
    ///
    /// Uses `--branch` to clone the exact branch we need, which is more
    /// reliable than shallow cloning default and then fetching.
    pub async fn clone_repo_with_branch(
        &self,
        owner: &str,
        repo: &str,
        token: &str,
        target_dir: &Path,
        base_branch: &str,
    ) -> Result<()> {
        let url = format!("https://x-access-token:{token}@github.com/{owner}/{repo}.git");

        tracing::debug!(
            target = %target_dir.display(),
            branch = %base_branch,
            "Cloning repository"
        );

        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                base_branch,
                "--single-branch",
                &url,
            ])
            .arg(target_dir)
            .output()
            .await
            .context("Failed to execute git clone")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Redact token from error message
            let safe_err = stderr.replace(token, "[REDACTED]");

            // Provide helpful error message for missing branch
            if safe_err.contains("not found") || safe_err.contains("Could not find remote branch") {
                return Err(anyhow::anyhow!(
                    "Branch '{base_branch}' not found on remote. \
                     Try setting RESEARCH_BASE_BRANCH=main if your repo uses main as the default branch."
                ));
            }

            return Err(anyhow::anyhow!("git clone failed: {safe_err}"));
        }

        // Configure git user
        self.configure_user(target_dir).await?;

        Ok(())
    }

    /// Clone a repository (deprecated - use `clone_repo_with_branch`).
    #[deprecated(note = "Use clone_repo_with_branch for reliable branch handling")]
    pub async fn clone_repo(
        &self,
        owner: &str,
        repo: &str,
        token: &str,
        target_dir: &Path,
    ) -> Result<()> {
        // Default to main for backwards compatibility
        self.clone_repo_with_branch(owner, repo, token, target_dir, "main")
            .await
    }

    /// Configure git user for the repository.
    async fn configure_user(&self, repo_dir: &Path) -> Result<()> {
        Command::new("git")
            .args(["config", "user.name", &self.author_name])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to set git user.name")?;

        Command::new("git")
            .args(["config", "user.email", &self.author_email])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to set git user.email")?;

        Ok(())
    }

    /// Create and checkout a new branch from the current HEAD.
    ///
    /// Assumes the repo was cloned with `clone_repo_with_branch` so we're
    /// already on the correct base branch.
    pub async fn create_branch(&self, repo_dir: &Path, branch_name: &str) -> Result<()> {
        tracing::debug!(branch = %branch_name, "Creating branch from HEAD");

        // Create and checkout new branch from current HEAD
        // (HEAD is already the base branch from clone_repo_with_branch)
        let output = Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to create branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git checkout failed: {stderr}"));
        }

        Ok(())
    }

    /// Stage all changes and commit.
    pub async fn commit(&self, repo_dir: &Path, message: &str) -> Result<()> {
        tracing::debug!("Committing changes");

        // Stage all changes
        let output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to stage changes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git add failed: {stderr}"));
        }

        // Check if there are changes to commit
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to check git status")?;

        if status.stdout.is_empty() {
            return Err(anyhow::anyhow!("No changes to commit"));
        }

        // Commit
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to commit")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git commit failed: {stderr}"));
        }

        Ok(())
    }

    /// Push branch to remote.
    pub async fn push(&self, repo_dir: &Path, branch_name: &str, token: &str) -> Result<()> {
        tracing::debug!(branch = %branch_name, "Pushing to remote");

        // Set the push URL with token
        let output = Command::new("git")
            .args(["push", "-u", "origin", branch_name])
            .current_dir(repo_dir)
            .output()
            .await
            .context("Failed to push")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Redact token from error
            let safe_err = stderr.replace(token, "[REDACTED]");
            return Err(anyhow::anyhow!("git push failed: {safe_err}"));
        }

        Ok(())
    }
}

impl Default for GitPublisher {
    fn default() -> Self {
        Self::new()
    }
}
