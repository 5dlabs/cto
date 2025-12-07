//! Slack webhook notification channel.

use async_trait::async_trait;
use serde::Serialize;
use tracing::{debug, warn};

use super::events::NotifyEvent;
use super::{ChannelError, NotifyChannel};

/// Environment variable for Slack webhook URL.
const ENV_SLACK_WEBHOOK_URL: &str = "SLACK_WEBHOOK_URL";

/// Slack webhook notification channel.
pub struct SlackChannel {
    webhook_url: Option<String>,
    client: reqwest::Client,
}

impl SlackChannel {
    /// Create a new Slack channel from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        let webhook_url = std::env::var(ENV_SLACK_WEBHOOK_URL).ok();

        if webhook_url.is_some() {
            debug!("Slack notifications enabled");
        } else {
            debug!("Slack notifications disabled (SLACK_WEBHOOK_URL not set)");
        }

        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }

    /// Create a Slack channel with a specific webhook URL.
    #[must_use]
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url: Some(webhook_url),
            client: reqwest::Client::new(),
        }
    }

    /// Format an event as a Slack webhook payload using Block Kit.
    fn format_payload(event: &NotifyEvent) -> SlackPayload {
        let severity = event.severity();

        let mut blocks = vec![
            // Header with emoji and title
            SlackBlock::Section {
                text: SlackText::mrkdwn(format!("{} *{}*", severity.emoji(), event.title())),
            },
            // Description
            SlackBlock::Section {
                text: SlackText::mrkdwn(Self::format_description(event)),
            },
        ];

        // Add fields section if we have fields
        let fields = Self::format_fields(event);
        if !fields.is_empty() {
            blocks.push(SlackBlock::Divider);
            blocks.push(SlackBlock::Section {
                text: SlackText::mrkdwn(fields.join(" • ")),
            });
        }

        // Add context with timestamp
        blocks.push(SlackBlock::Context {
            elements: vec![SlackText::mrkdwn(format!(
                "CTO Platform • {}",
                event.timestamp().format("%Y-%m-%d %H:%M:%S UTC")
            ))],
        });

        SlackPayload {
            text: event.title(), // Fallback for notifications
            blocks,
            attachments: vec![SlackAttachment {
                color: format!("#{:06x}", severity.color()),
                fallback: None,
            }],
        }
    }

    /// Format the description for an event.
    fn format_description(event: &NotifyEvent) -> String {
        match event {
            NotifyEvent::PlayStarted {
                repository,
                workflow_name,
                ..
            } => {
                format!("Started workflow `{workflow_name}` for repository `{repository}`")
            }

            NotifyEvent::TaskStarted {
                repository, agent, ..
            } => {
                format!("Agent *{agent}* started working on `{repository}`")
            }

            NotifyEvent::TaskCompleted {
                repository,
                agent,
                success,
                duration_secs,
                ..
            } => {
                let status = if *success {
                    ":white_check_mark: Success"
                } else {
                    ":x: Failed"
                };
                let duration = format_duration(*duration_secs);
                format!("Agent *{agent}* finished on `{repository}`\n{status} in {duration}")
            }

            NotifyEvent::AgentStarted {
                agent,
                coderun_name,
                ..
            } => {
                format!("CodeRun `{coderun_name}` started for agent *{agent}*")
            }

            NotifyEvent::AgentCompleted {
                agent,
                coderun_name,
                success,
                duration_secs,
                ..
            } => {
                let status = if *success {
                    ":white_check_mark: Success"
                } else {
                    ":x: Failed"
                };
                let duration = format_duration(*duration_secs);
                format!("CodeRun `{coderun_name}` for agent *{agent}*\n{status} in {duration}")
            }

            NotifyEvent::HealAlert { message, .. } => message.clone(),

            NotifyEvent::HealRemediation {
                repository, reason, ..
            } => {
                format!("Starting remediation on `{repository}`\n*Reason:* {reason}")
            }
        }
    }

    /// Format additional fields for an event as inline text.
    fn format_fields(event: &NotifyEvent) -> Vec<String> {
        match event {
            NotifyEvent::PlayStarted {
                task_id,
                repository,
                workflow_name,
                ..
            } => vec![
                format!("*Task:* {task_id}"),
                format!("*Repo:* {repository}"),
                format!("*Workflow:* {workflow_name}"),
            ],

            NotifyEvent::TaskStarted {
                task_id,
                repository,
                agent,
                ..
            }
            | NotifyEvent::TaskCompleted {
                task_id,
                repository,
                agent,
                ..
            } => vec![
                format!("*Task:* {task_id}"),
                format!("*Repo:* {repository}"),
                format!("*Agent:* {agent}"),
            ],

            NotifyEvent::AgentStarted {
                agent,
                task_id,
                coderun_name,
                ..
            }
            | NotifyEvent::AgentCompleted {
                agent,
                task_id,
                coderun_name,
                ..
            } => vec![
                format!("*Agent:* {agent}"),
                format!("*Task:* {task_id}"),
                format!("*CodeRun:* {coderun_name}"),
            ],

            NotifyEvent::HealAlert {
                alert_id,
                severity,
                context,
                ..
            } => {
                let mut fields = vec![
                    format!("*Alert:* {alert_id}"),
                    format!("*Severity:* {}", severity.as_str()),
                ];

                for (key, value) in context {
                    fields.push(format!("*{key}:* {value}"));
                }

                fields
            }

            NotifyEvent::HealRemediation {
                task_id,
                iteration,
                repository,
                ..
            } => vec![
                format!("*Task:* {task_id}"),
                format!("*Iteration:* {iteration}"),
                format!("*Repo:* {repository}"),
            ],
        }
    }
}

#[async_trait]
impl NotifyChannel for SlackChannel {
    fn name(&self) -> &'static str {
        "slack"
    }

    fn enabled(&self) -> bool {
        self.webhook_url.is_some()
    }

    async fn send(&self, event: &NotifyEvent) -> Result<(), ChannelError> {
        let webhook_url = self
            .webhook_url
            .as_ref()
            .ok_or_else(|| ChannelError::NotConfigured("SLACK_WEBHOOK_URL".to_string()))?;

        let payload = Self::format_payload(event);

        debug!(channel = "slack", event_type = ?event.title(), "Sending notification");

        let response = self.client.post(webhook_url).json(&payload).send().await?;

        if response.status().is_success() {
            debug!(channel = "slack", "Notification sent successfully");
            Ok(())
        } else if response.status() == 429 {
            // Rate limited
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(5);

            warn!(
                channel = "slack",
                retry_after_secs = retry_after,
                "Rate limited by Slack"
            );

            Err(ChannelError::RateLimited {
                retry_after_secs: retry_after,
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            warn!(
                channel = "slack",
                status = %status,
                body = %body,
                "Slack webhook request failed"
            );

            Err(ChannelError::Other(format!(
                "Slack returned {status}: {body}"
            )))
        }
    }
}

// =============================================================================
// Slack API types (Block Kit)
// =============================================================================

#[derive(Debug, Serialize)]
struct SlackPayload {
    /// Fallback text for notifications
    text: String,
    /// Block Kit blocks
    blocks: Vec<SlackBlock>,
    /// Attachments (for color strip)
    attachments: Vec<SlackAttachment>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SlackBlock {
    /// Section block with text
    Section { text: SlackText },
    /// Divider line
    Divider,
    /// Context block for metadata
    Context { elements: Vec<SlackText> },
}

#[derive(Debug, Serialize)]
struct SlackText {
    #[serde(rename = "type")]
    text_type: &'static str,
    text: String,
}

impl SlackText {
    fn mrkdwn(text: impl Into<String>) -> Self {
        Self {
            text_type: "mrkdwn",
            text: text.into(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SlackAttachment {
    /// Hex color for the attachment strip
    color: String,
    /// Fallback text (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback: Option<String>,
}

/// Format seconds into a human-readable duration.
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        if remaining_secs == 0 {
            format!("{mins}m")
        } else {
            format!("{mins}m {remaining_secs}s")
        }
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if mins == 0 {
            format!("{hours}h")
        } else {
            format!("{hours}h {mins}m")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_slack_channel_disabled_without_env() {
        // Clear any existing env var
        std::env::remove_var("SLACK_WEBHOOK_URL");

        let channel = SlackChannel::from_env();
        assert!(!channel.enabled());
    }

    #[test]
    fn test_slack_payload_format() {
        let event = NotifyEvent::PlayStarted {
            task_id: "42".to_string(),
            repository: "test/repo".to_string(),
            workflow_name: "wf-123".to_string(),
            timestamp: Utc::now(),
        };

        let payload = SlackChannel::format_payload(&event);

        assert_eq!(payload.text, "Play Started: Task #42");
        assert!(!payload.blocks.is_empty());
    }
}

