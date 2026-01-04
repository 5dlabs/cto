//! Auto-merge functionality for healer CI remediation.
//!
//! This module handles:
//! - Monitoring PR status after successful remediation
//! - Auto-merging PRs when checks pass
//! - Detecting and routing merge conflicts

use std::time::Duration;

use anyhow::Result;
use tracing::{debug, error, info, warn};

use crate::github::{GitHubClient, MergeMethod, MergeStatus};

use super::tracker::{CompletionAction, TrackedRemediation};
use super::types::RemediationConfig;

/// Interval between check status polls (in seconds).
const CHECK_POLL_INTERVAL_SECS: u64 = 30;

/// Auto-merge handler for managing PR merges after successful remediation.
pub struct AutoMergeHandler {
    /// GitHub client
    github: GitHubClient,
    /// Configuration
    config: RemediationConfig,
}

impl AutoMergeHandler {
    /// Create a new auto-merge handler.
    #[must_use]
    pub fn new(github: GitHubClient, config: RemediationConfig) -> Self {
        Self { github, config }
    }

    /// Handle a successful remediation by attempting to merge the PR.
    ///
    /// This function will:
    /// 1. Check if auto-merge is enabled
    /// 2. Poll PR status until checks complete or timeout
    /// 3. Merge if ready, or return appropriate action
    ///
    /// # Errors
    ///
    /// Returns an error if the GitHub API calls fail.
    pub async fn handle_success(&self, tracked: &TrackedRemediation) -> Result<CompletionAction> {
        let Some(pr_number) = tracked.pr_number else {
            debug!("No PR number associated with remediation, skipping auto-merge");
            return Ok(CompletionAction::Success);
        };

        if !self.config.auto_merge_enabled {
            debug!("Auto-merge disabled, skipping");
            return Ok(CompletionAction::Success);
        }

        info!(pr = pr_number, "Checking PR status for auto-merge");

        // Poll for PR status
        let timeout = Duration::from_secs(u64::from(self.config.check_timeout_mins) * 60);
        let start = std::time::Instant::now();

        loop {
            // Check if we've exceeded timeout
            if start.elapsed() > timeout {
                warn!(
                    pr = pr_number,
                    "Timed out waiting for checks after {:?}",
                    start.elapsed()
                );
                return Ok(CompletionAction::AwaitingChecks { pr_number });
            }

            // Check PR status
            let status = self.github.check_mergeable(pr_number)?;

            match status {
                MergeStatus::Ready => {
                    info!(pr = pr_number, "PR is ready to merge");
                    return self.attempt_merge(pr_number).await;
                }
                MergeStatus::Conflicting => {
                    warn!(pr = pr_number, "PR has merge conflicts");
                    let files = self.github.get_conflicting_files(pr_number)?;
                    return Ok(CompletionAction::MergeConflict {
                        pr_number,
                        branch: tracked.branch.clone(),
                        conflicting_files: files,
                    });
                }
                MergeStatus::Blocked { reason } => {
                    debug!(
                        pr = pr_number,
                        reason = %reason,
                        "PR is blocked, waiting..."
                    );
                    // Continue polling
                }
                MergeStatus::Unknown => {
                    warn!(pr = pr_number, "PR status unknown, skipping auto-merge");
                    return Ok(CompletionAction::Success);
                }
            }

            // Wait before next poll
            tokio::time::sleep(Duration::from_secs(CHECK_POLL_INTERVAL_SECS)).await;
        }
    }

    /// Attempt to merge the PR.
    async fn attempt_merge(&self, pr_number: u32) -> Result<CompletionAction> {
        let method = parse_merge_method(&self.config.merge_method);

        info!(
            pr = pr_number,
            method = ?method,
            "Attempting to merge PR"
        );

        match self.github.merge_pr(pr_number, method) {
            Ok(()) => {
                info!(pr = pr_number, "Successfully merged PR");

                // Add a comment to the PR
                let _ = self.github.add_pr_comment(
                    pr_number,
                    "🤖 **Healer Auto-Merge**: PR automatically merged after CI checks passed.",
                );

                Ok(CompletionAction::Merged { pr_number })
            }
            Err(e) => {
                error!(pr = pr_number, error = %e, "Failed to merge PR");

                // Check if it's a conflict issue
                if e.to_string().contains("conflict") {
                    let files = self.github.get_conflicting_files(pr_number)?;
                    return Ok(CompletionAction::MergeConflict {
                        pr_number,
                        branch: String::new(), // Will be filled by caller
                        conflicting_files: files,
                    });
                }

                // Return success to indicate remediation worked, merge just failed
                Ok(CompletionAction::ReadyToMerge { pr_number })
            }
        }
    }

    /// Check if a PR needs conflict resolution and route to Atlas if so.
    ///
    /// # Errors
    ///
    /// Returns an error if the GitHub API calls fail.
    pub fn check_for_conflicts(&self, pr_number: u32) -> Result<Option<CompletionAction>> {
        let status = self.github.check_mergeable(pr_number)?;

        if let MergeStatus::Conflicting = status {
            let files = self.github.get_conflicting_files(pr_number)?;
            return Ok(Some(CompletionAction::MergeConflict {
                pr_number,
                branch: String::new(),
                conflicting_files: files,
            }));
        }

        Ok(None)
    }

    /// Try to update the PR branch with the latest base branch.
    ///
    /// This can resolve simple conflicts by merging the base branch.
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub fn try_update_branch(&self, pr_number: u32) -> Result<bool> {
        info!(pr = pr_number, "Attempting to update PR branch");

        match self.github.update_branch(pr_number) {
            Ok(()) => {
                info!(pr = pr_number, "Successfully updated PR branch");
                Ok(true)
            }
            Err(e) => {
                warn!(
                    pr = pr_number,
                    error = %e,
                    "Failed to update PR branch - conflicts may require manual resolution"
                );
                Ok(false)
            }
        }
    }
}

/// Parse merge method from config string.
fn parse_merge_method(method: &str) -> MergeMethod {
    match method.to_lowercase().as_str() {
        "merge" => MergeMethod::Merge,
        "rebase" => MergeMethod::Rebase,
        _ => MergeMethod::Squash, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_merge_method() {
        assert_eq!(parse_merge_method("squash"), MergeMethod::Squash);
        assert_eq!(parse_merge_method("merge"), MergeMethod::Merge);
        assert_eq!(parse_merge_method("rebase"), MergeMethod::Rebase);
        assert_eq!(parse_merge_method("SQUASH"), MergeMethod::Squash);
        assert_eq!(parse_merge_method("unknown"), MergeMethod::Squash);
    }
}
