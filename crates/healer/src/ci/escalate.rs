//! Human escalation for CI remediation failures.
//!
//! Handles escalation when automated remediation fails:
//! - Posts PR comments with failure summaries
//! - Sends Discord notifications via the notify crate
//! - Creates GitHub issues for tracking

use anyhow::{Context as _, Result};
use std::process::Command;
use tracing::{info, warn};

use super::types::{CiFailure, RemediationAttempt};

/// Escalation configuration.
#[derive(Debug, Clone)]
pub struct EscalationConfig {
    /// Discord webhook URL (via notify crate)
    pub discord_enabled: bool,
    /// GitHub issue creation enabled
    pub github_issue_enabled: bool,
    /// PR comment enabled
    pub pr_comment_enabled: bool,
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            discord_enabled: true,
            github_issue_enabled: false,
            pr_comment_enabled: true,
        }
    }
}

/// Escalator for handling human escalation.
pub struct Escalator {
    config: EscalationConfig,
}

impl Escalator {
    /// Create a new escalator.
    #[must_use]
    pub fn new(config: EscalationConfig) -> Self {
        Self { config }
    }

    /// Escalate a failed remediation to humans.
    pub async fn escalate(
        &self,
        failure: &CiFailure,
        attempts: &[RemediationAttempt],
        pr_number: Option<u64>,
    ) -> Result<()> {
        info!(
            "Escalating failed remediation for workflow {} after {} attempts",
            failure.workflow_name,
            attempts.len()
        );

        // Build the escalation message
        let message = self.build_escalation_message(failure, attempts);

        // Post PR comment if we have a PR
        if self.config.pr_comment_enabled {
            if let Some(pr) = pr_number {
                if let Err(e) = self.post_pr_comment(&failure.repository, pr, &message) {
                    warn!("Failed to post PR comment: {e}");
                }
            }
        }

        // Send Discord notification
        if self.config.discord_enabled {
            if let Err(e) = self.send_discord_notification(failure, attempts).await {
                warn!("Failed to send Discord notification: {e}");
            }
        }

        // Create GitHub issue if enabled
        if self.config.github_issue_enabled {
            if let Err(e) = self.create_github_issue(&failure.repository, failure, attempts) {
                warn!("Failed to create GitHub issue: {e}");
            }
        }

        Ok(())
    }

    /// Build the escalation message for PR comments.
    fn build_escalation_message(&self, failure: &CiFailure, attempts: &[RemediationAttempt]) -> String {
        let mut msg = String::new();

        msg.push_str("## 🚨 CI Remediation Escalation\n\n");
        msg.push_str(&format!(
            "Automated remediation failed after **{} attempts**.\n\n",
            attempts.len()
        ));

        msg.push_str("### Failure Details\n\n");
        msg.push_str(&format!("- **Workflow**: {}\n", failure.workflow_name));
        if let Some(job) = &failure.job_name {
            msg.push_str(&format!("- **Job**: {job}\n"));
        }
        msg.push_str(&format!("- **Branch**: `{}`\n", failure.branch));
        msg.push_str(&format!("- **Commit**: `{}`\n", &failure.head_sha[..7.min(failure.head_sha.len())]));
        msg.push_str(&format!("- **[View Workflow]({})**\n\n", failure.html_url));

        msg.push_str("### Remediation Attempts\n\n");
        msg.push_str("| # | Agent | Outcome | Duration |\n");
        msg.push_str("|---|-------|---------|----------|\n");

        for attempt in attempts {
            let duration = attempt
                .completed_at
                .map(|c| {
                    let dur = c - attempt.started_at;
                    format!("{}s", dur.num_seconds())
                })
                .unwrap_or_else(|| "N/A".to_string());

            msg.push_str(&format!(
                "| {} | {} | {:?} | {} |\n",
                attempt.attempt_number,
                attempt.agent.name(),
                attempt.outcome,
                duration
            ));
        }

        msg.push('\n');

        // Add last error if available
        if let Some(last) = attempts.last() {
            if let Some(error) = &last.error_message {
                msg.push_str("### Last Error\n\n");
                msg.push_str("```\n");
                // Truncate very long errors
                if error.len() > 2000 {
                    msg.push_str(&error[..2000]);
                    msg.push_str("...(truncated)");
                } else {
                    msg.push_str(error);
                }
                msg.push_str("\n```\n\n");
            }
        }

        msg.push_str("---\n");
        msg.push_str("*This issue requires manual intervention. ");
        msg.push_str("Please investigate and fix the root cause.*\n");

        msg
    }

    /// Post a comment to a GitHub PR.
    fn post_pr_comment(&self, repository: &str, pr_number: u64, message: &str) -> Result<()> {
        let output = Command::new("gh")
            .args([
                "pr",
                "comment",
                &pr_number.to_string(),
                "--repo",
                repository,
                "--body",
                message,
            ])
            .output()
            .context("Failed to execute gh pr comment")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr comment failed: {stderr}");
        }

        info!("Posted escalation comment to PR #{pr_number}");

        Ok(())
    }

    /// Send a Discord notification via the notify crate.
    async fn send_discord_notification(
        &self,
        failure: &CiFailure,
        attempts: &[RemediationAttempt],
    ) -> Result<()> {
        // Use the notify crate for Discord notifications
        use notify::{DiscordConfig, NotifyEvent};

        let config = DiscordConfig::default();
        
        // Build a summary for Discord
        let agents_tried: Vec<&str> = attempts.iter().map(|a| a.agent.name()).collect();
        let summary = format!(
            "**CI Remediation Failed**\n\
            Workflow: `{}`\n\
            Branch: `{}`\n\
            Attempts: {} (agents: {})\n\
            [View Workflow]({})",
            failure.workflow_name,
            failure.branch,
            attempts.len(),
            agents_tried.join(" → "),
            failure.html_url
        );

        // Create and send the event
        let event = NotifyEvent::Custom {
            title: "🚨 CI Remediation Escalation".to_string(),
            description: summary,
            color: Some(0xFF0000), // Red
            fields: vec![
                ("Repository".to_string(), failure.repository.clone()),
                ("Commit".to_string(), failure.head_sha[..7.min(failure.head_sha.len())].to_string()),
            ],
        };

        notify::send_discord(&config, &event).await?;

        info!("Sent Discord escalation notification");

        Ok(())
    }

    /// Create a GitHub issue for tracking.
    fn create_github_issue(
        &self,
        repository: &str,
        failure: &CiFailure,
        attempts: &[RemediationAttempt],
    ) -> Result<()> {
        let title = format!(
            "[Healer] CI Remediation Failed: {}",
            failure.workflow_name
        );

        let body = self.build_escalation_message(failure, attempts);

        let labels = "healer,ci-failure,needs-attention";

        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                repository,
                "--title",
                &title,
                "--body",
                &body,
                "--label",
                labels,
            ])
            .output()
            .context("Failed to execute gh issue create")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue create failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("Created GitHub issue: {}", stdout.trim());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::types::{Agent, AttemptOutcome};
    use chrono::Utc;

    fn create_test_failure() -> CiFailure {
        CiFailure {
            workflow_run_id: 12345,
            workflow_name: "CI".to_string(),
            job_name: Some("build".to_string()),
            conclusion: "failure".to_string(),
            branch: "feat/test".to_string(),
            head_sha: "abc123def456".to_string(),
            commit_message: "Test commit".to_string(),
            html_url: "https://github.com/test/repo/actions/runs/12345".to_string(),
            repository: "test/repo".to_string(),
            sender: "testuser".to_string(),
            detected_at: Utc::now(),
            raw_event: None,
        }
    }

    fn create_test_attempts() -> Vec<RemediationAttempt> {
        vec![
            RemediationAttempt {
                attempt_number: 1,
                agent: Agent::Rex,
                outcome: AttemptOutcome::Failed,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
                error_message: Some("Compilation error".to_string()),
                changes_made: Vec::new(),
            },
            RemediationAttempt {
                attempt_number: 2,
                agent: Agent::Atlas,
                outcome: AttemptOutcome::Failed,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
                error_message: Some("Test failure".to_string()),
                changes_made: Vec::new(),
            },
        ]
    }

    #[test]
    fn test_build_escalation_message() {
        let escalator = Escalator::new(EscalationConfig::default());
        let failure = create_test_failure();
        let attempts = create_test_attempts();

        let message = escalator.build_escalation_message(&failure, &attempts);

        assert!(message.contains("CI Remediation Escalation"));
        assert!(message.contains("2 attempts"));
        assert!(message.contains("Rex"));
        assert!(message.contains("Atlas"));
        assert!(message.contains("feat/test"));
    }
}

