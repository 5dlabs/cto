//! GitHub API integration for fallback PR detection

use anyhow::{Context as AnyhowContext, Result};
use octocrab::{models::pulls::PullRequest, Octocrab};
use tracing::{info, warn};

use crate::crds::coderun::CodeRun;

/// Check GitHub API for PR by branch name
pub async fn check_github_for_pr_by_branch(
    code_run: &CodeRun,
    github_token: Option<&str>,
) -> Result<Option<String>> {
    let task_id = code_run.spec.task_id;

    info!(
        "Checking GitHub API for PR containing branch pattern: task-{}",
        task_id
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

    // Search for PRs whose head ref matches task-specific branch patterns
    let mut page = octocrab
        .pulls(&owner, &repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(50)
        .send()
        .await
        .with_context(|| format!("Failed to search for PRs in {owner}/{repo}"))?;

    let expected_full_name = format!("{owner}/{repo}");

    loop {
        if let Some(pr) = page.items.iter().find(|pr| {
            branch_matches(task_id, &pr.head.ref_field)
                && pr_origin_matches(pr, &owner, &repo, &expected_full_name)
        }) {
            let pr_url = pr
                .html_url
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_else(|| format!("https://github.com/{owner}/{repo}/pull/{}", pr.number));
            info!("Found PR via GitHub API for task {}: {}", task_id, pr_url);
            return Ok(Some(pr_url));
        }

        if let Some(next) = next_page(&octocrab, &page).await? {
            page = next
        } else {
            info!("No PR found for task branch patterns: task-{}", task_id);
            return Ok(None);
        }
    }
}

async fn next_page(
    client: &Octocrab,
    current: &octocrab::Page<PullRequest>,
) -> Result<Option<octocrab::Page<PullRequest>>> {
    client
        .get_page(&current.next)
        .await
        .with_context(|| "Failed to fetch next page of pull requests".to_string())
}

fn branch_matches(task_id: u32, head_ref: &str) -> bool {
    let base = format!("task-{task_id}");

    head_ref == base
        || head_ref == format!("feature/{base}")
        || head_ref.starts_with(&format!("{base}-"))
        || head_ref.starts_with(&format!("feature/{base}-"))
}

fn pr_origin_matches(
    pr: &PullRequest,
    expected_owner: &str,
    expected_repo: &str,
    expected_full_name: &str,
) -> bool {
    pr.head.repo.as_ref().is_some_and(|repo| {
        repo_identity_matches(
            repo.owner.as_ref().map(|owner| owner.login.as_str()),
            repo.name.as_str(),
            repo.full_name.as_deref(),
            expected_owner,
            expected_repo,
            expected_full_name,
        )
    })
}

fn repo_identity_matches(
    owner_login: Option<&str>,
    repo_name: &str,
    full_name: Option<&str>,
    expected_owner: &str,
    expected_repo: &str,
    expected_full_name: &str,
) -> bool {
    let owner_matches = owner_login.is_some_and(|login| login.eq_ignore_ascii_case(expected_owner));

    if !owner_matches {
        return false;
    }

    if let Some(full) = full_name {
        full.eq_ignore_ascii_case(expected_full_name)
    } else {
        repo_name.eq_ignore_ascii_case(expected_repo)
    }
}

/// Parse repository URL to extract owner and repo name
/// Supports formats like:
/// - <https://github.com/owner/repo>
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
        Err(anyhow::anyhow!("Invalid repository URL format: {repo_url}"))
    }
}

/// Update `CodeRun` status with found PR URL
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

    #[test]
    fn test_branch_matches() {
        assert!(branch_matches(1, "task-1"));
        assert!(branch_matches(1, "task-1-implementation"));
        assert!(branch_matches(1, "feature/task-1"));
        assert!(branch_matches(1, "feature/task-1-implementation"));
        assert!(branch_matches(
            1,
            "feature/task-1-implementation-20250101121212"
        ));
        assert!(!branch_matches(1, "task-10"));
        assert!(!branch_matches(1, "feature/task-10-implementation"));
        assert!(!branch_matches(1, "main"));
    }

    #[test]
    fn test_repo_identity_matches_full_name() {
        assert!(repo_identity_matches(
            Some("5dlabs"),
            "rust-basic-api-2",
            Some("5dlabs/rust-basic-api-2"),
            "5dlabs",
            "rust-basic-api-2",
            "5dlabs/rust-basic-api-2"
        ));
    }

    #[test]
    fn test_repo_identity_matches_repo_name_only() {
        assert!(repo_identity_matches(
            Some("5dlabs"),
            "rust-basic-api-2",
            None,
            "5dlabs",
            "rust-basic-api-2",
            "5dlabs/rust-basic-api-2"
        ));
    }

    #[test]
    fn test_repo_identity_matches_owner_mismatch() {
        assert!(!repo_identity_matches(
            Some("someone-else"),
            "rust-basic-api-2",
            Some("someone-else/rust-basic-api-2"),
            "5dlabs",
            "rust-basic-api-2",
            "5dlabs/rust-basic-api-2"
        ));
    }
}
