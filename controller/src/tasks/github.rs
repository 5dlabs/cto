//! GitHub API integration for fallback PR detection

use anyhow::{Context as AnyhowContext, Result};
use octocrab::Octocrab;
use tracing::{info, warn};

use crate::crds::coderun::CodeRun;

/// Check GitHub API for PR by branch name
pub async fn check_github_for_pr_by_branch(
    code_run: &CodeRun,
    github_token: Option<&str>,
) -> Result<Option<String>> {
    let task_id = code_run.spec.task_id;
    let expected_branch = format!("task-{task_id}");

    info!(
        "Checking GitHub API for PR with branch: {}",
        expected_branch
    );

    // Parse repository URL to extract owner/repo
    let (owner, repo) = parse_repository_url(&code_run.spec.repository_url)?;

    // Create GitHub client
    let octocrab = if let Some(token) = github_token {
        Octocrab::builder()
            .personal_token(token.to_string())
            .build()?
    } else {
        // Try to use GitHub App authentication if available
        // For now, we'll use unauthenticated requests (rate limited)
        warn!("No GitHub token provided, using unauthenticated requests");
        Octocrab::builder().build()?
    };

    // Search for PRs with the expected branch
    let pulls = octocrab
        .pulls(&owner, &repo)
        .list()
        .state(octocrab::params::State::Open)
        .head(format!("{owner}:{expected_branch}"))
        .send()
        .await
        .with_context(|| format!("Failed to search for PRs in {owner}/{repo}"))?;

    if let Some(pr) = pulls.items.first() {
        let pr_url = pr.html_url.as_ref().map(|url| url.to_string());
        info!("Found PR via GitHub API: {:?}", pr_url);
        Ok(pr_url)
    } else {
        info!("No PR found for branch: {}", expected_branch);
        Ok(None)
    }
}

/// Parse repository URL to extract owner and repo name
/// Supports formats like:
/// - https://github.com/owner/repo
/// - git@github.com:owner/repo.git
/// - owner/repo
fn parse_repository_url(repo_url: &str) -> Result<(String, String)> {
    // Handle different URL formats
    let cleaned_url = repo_url
        .trim_end_matches(".git")
        .replace("git@github.com:", "https://github.com/")
        .replace("https://github.com/", "");

    let parts: Vec<&str> = cleaned_url.split('/').collect();
    if parts.len() >= 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(anyhow::anyhow!(
            "Invalid repository URL format: {}",
            repo_url
        ))
    }
}

/// Update CodeRun status with found PR URL
pub async fn update_code_run_pr_url(
    client: &kube::Client,
    namespace: &str,
    code_run_name: &str,
    pr_url: &str,
) -> Result<()> {
    use kube::{
        api::{Patch, PatchParams},
        Api,
    };
    use serde_json::json;

    info!("Updating CodeRun {} with PR URL: {}", code_run_name, pr_url);

    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

    let status_patch = json!({
        "status": {
            "pullRequestUrl": pr_url,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
        }
    });

    coderuns
        .patch_status(
            code_run_name,
            &PatchParams::default(),
            &Patch::Merge(&status_patch),
        )
        .await
        .with_context(|| format!("Failed to update CodeRun {code_run_name} with PR URL"))?;

    info!(
        "✅ Successfully updated CodeRun {} with PR URL",
        code_run_name
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repository_url() {
        // HTTPS format
        assert_eq!(
            parse_repository_url("https://github.com/owner/repo").unwrap(),
            ("owner".to_string(), "repo".to_string())
        );

        // HTTPS with .git
        assert_eq!(
            parse_repository_url("https://github.com/owner/repo.git").unwrap(),
            ("owner".to_string(), "repo".to_string())
        );

        // SSH format
        assert_eq!(
            parse_repository_url("git@github.com:owner/repo.git").unwrap(),
            ("owner".to_string(), "repo".to_string())
        );

        // Simple format
        assert_eq!(
            parse_repository_url("owner/repo").unwrap(),
            ("owner".to_string(), "repo".to_string())
        );

        // Invalid format
        assert!(parse_repository_url("invalid").is_err());
    }
}
