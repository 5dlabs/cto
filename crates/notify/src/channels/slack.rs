//! Slack webhook notification channel.

use async_trait::async_trait;
use serde::Serialize;
use tracing::{debug, warn};

use crate::error::ChannelError;
use crate::events::{NotifyEvent, Severity};
use crate::NotifyChannel;

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

    /// Format an event as a Slack webhook payload.
    fn format_payload(event: &NotifyEvent) -> SlackPayload {
        let color = match event.severity() {
            Severity::Info => "#3498db",     // Blue
            Severity::Warning => "#f39c12",  // Orange
            Severity::Critical => "#e74c3c", // Red
        };

        let mut fields = vec![];
        for (name, value) in Self::format_fields(event) {
            fields.push(SlackField {
                title: name,
                value,
                short: true,
            });
        }

        let attachment = SlackAttachment {
            fallback: event.title(),
            color: color.to_string(),
            pretext: None,
            author_name: Some("CTO Platform".to_string()),
            title: event.title(),
            text: Self::format_description(event),
            fields,
            footer: Some(format!(
                "{} | {}",
                event.severity().as_str(),
                event.timestamp().format("%Y-%m-%d %H:%M:%S UTC")
            )),
            ts: Some(event.timestamp().timestamp()),
        };

        SlackPayload {
            attachments: vec![attachment],
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
                    "✅ Success"
                } else {
                    "❌ Failed"
                };
                let duration = format_duration(*duration_secs);
                format!("Agent *{agent}* finished on `{repository}`\n{status} in {duration}")
            }

            NotifyEvent::AgentStarted {
                agent,
                coderun_name,
                ..
            } => {
                format!("`{coderun_name}` started for agent *{agent}*")
            }

            NotifyEvent::AgentCompleted {
                agent,
                coderun_name,
                success,
                duration_secs,
                ..
            } => {
                let status = if *success {
                    "✅ Success"
                } else {
                    "❌ Failed"
                };
                let duration = format_duration(*duration_secs);
                format!("`{coderun_name}` for agent *{agent}*\n{status} in {duration}")
            }

            NotifyEvent::HealAlert { message, .. } => message.clone(),

            NotifyEvent::HealRemediation {
                repository, reason, ..
            } => {
                format!("Starting remediation on `{repository}`\n*Reason:* {reason}")
            }
        }
    }

    /// Format additional fields for an event.
    fn format_fields(event: &NotifyEvent) -> Vec<(String, String)> {
        match event {
            NotifyEvent::PlayStarted {
                task_id,
                repository,
                workflow_name,
                ..
            } => vec![
                ("Task ID".to_string(), task_id.clone()),
                ("Repository".to_string(), repository.clone()),
                ("Workflow".to_string(), workflow_name.clone()),
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
                ("Task ID".to_string(), task_id.clone()),
                ("Repository".to_string(), repository.clone()),
                ("Agent".to_string(), agent.clone()),
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
                ("Agent".to_string(), agent.clone()),
                ("Task ID".to_string(), task_id.clone()),
                ("CodeRun".to_string(), coderun_name.clone()),
            ],

            NotifyEvent::HealAlert {
                alert_id,
                severity,
                context,
                ..
            } => {
                let mut fields = vec![
                    ("Alert ID".to_string(), alert_id.clone()),
                    ("Severity".to_string(), severity.as_str().to_string()),
                ];

                // Add context fields
                for (key, value) in context {
                    fields.push((key.clone(), value.clone()));
                }

                fields
            }

            NotifyEvent::HealRemediation {
                task_id,
                iteration,
                repository,
                ..
            } => vec![
                ("Task ID".to_string(), task_id.clone()),
                ("Iteration".to_string(), iteration.to_string()),
                ("Repository".to_string(), repository.clone()),
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
// Slack API types
// =============================================================================

#[derive(Debug, Serialize)]
struct SlackPayload {
    attachments: Vec<SlackAttachment>,
}

#[derive(Debug, Serialize)]
struct SlackAttachment {
    fallback: String,
    color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pretext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author_name: Option<String>,
    title: String,
    text: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<SlackField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
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
