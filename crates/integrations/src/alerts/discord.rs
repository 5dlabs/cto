//! Discord webhook notification channel.

use async_trait::async_trait;
use serde::Serialize;
use tracing::{debug, warn};

use super::events::NotifyEvent;
use super::{ChannelError, NotifyChannel};

/// Environment variable for Discord webhook URL.
const ENV_DISCORD_WEBHOOK_URL: &str = "DISCORD_WEBHOOK_URL";

/// Discord webhook notification channel.
pub struct DiscordChannel {
    webhook_url: Option<String>,
    client: reqwest::Client,
}

impl DiscordChannel {
    /// Create a new Discord channel from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        let webhook_url = std::env::var(ENV_DISCORD_WEBHOOK_URL).ok();

        if webhook_url.is_some() {
            debug!("Discord notifications enabled");
        } else {
            debug!("Discord notifications disabled (DISCORD_WEBHOOK_URL not set)");
        }

        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }

    /// Create a Discord channel with a specific webhook URL.
    #[must_use]
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url: Some(webhook_url),
            client: reqwest::Client::new(),
        }
    }

    /// Format an event as a Discord webhook payload.
    fn format_payload(event: &NotifyEvent) -> DiscordPayload {
        let embed = DiscordEmbed {
            title: event.title(),
            description: Self::format_description(event),
            color: event.severity().color(),
            timestamp: event.timestamp().to_rfc3339(),
            footer: Some(DiscordFooter {
                text: "CTO Platform".to_string(),
            }),
            fields: Self::format_fields(event),
        };

        DiscordPayload {
            embeds: vec![embed],
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
                format!("Agent **{agent}** started working on `{repository}`")
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
                format!("Agent **{agent}** finished on `{repository}`\n{status} in {duration}")
            }

            NotifyEvent::AgentStarted {
                agent,
                coderun_name,
                ..
            } => {
                format!("CodeRun `{coderun_name}` started for agent **{agent}**")
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
                format!("CodeRun `{coderun_name}` for agent **{agent}**\n{status} in {duration}")
            }

            NotifyEvent::HealAlert { message, .. } => message.clone(),

            NotifyEvent::HealRemediation {
                repository, reason, ..
            } => {
                format!("Starting remediation on `{repository}`\n**Reason:** {reason}")
            }
        }
    }

    /// Format additional fields for an event.
    fn format_fields(event: &NotifyEvent) -> Vec<DiscordField> {
        match event {
            NotifyEvent::PlayStarted {
                task_id,
                repository,
                workflow_name,
                ..
            } => vec![
                DiscordField::inline("Task ID", task_id),
                DiscordField::inline("Repository", repository),
                DiscordField::inline("Workflow", workflow_name),
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
                DiscordField::inline("Task ID", task_id),
                DiscordField::inline("Repository", repository),
                DiscordField::inline("Agent", agent),
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
                DiscordField::inline("Agent", agent),
                DiscordField::inline("Task ID", task_id),
                DiscordField::inline("CodeRun", coderun_name),
            ],

            NotifyEvent::HealAlert {
                alert_id,
                severity,
                context,
                ..
            } => {
                let mut fields = vec![
                    DiscordField::inline("Alert ID", alert_id),
                    DiscordField::inline("Severity", severity.as_str()),
                ];

                // Add context fields
                for (key, value) in context {
                    fields.push(DiscordField::inline(key, value));
                }

                fields
            }

            NotifyEvent::HealRemediation {
                task_id,
                iteration,
                repository,
                ..
            } => vec![
                DiscordField::inline("Task ID", task_id),
                DiscordField::inline("Iteration", iteration.to_string()),
                DiscordField::inline("Repository", repository),
            ],
        }
    }
}

#[async_trait]
impl NotifyChannel for DiscordChannel {
    fn name(&self) -> &'static str {
        "discord"
    }

    fn enabled(&self) -> bool {
        self.webhook_url.is_some()
    }

    async fn send(&self, event: &NotifyEvent) -> Result<(), ChannelError> {
        let webhook_url = self
            .webhook_url
            .as_ref()
            .ok_or_else(|| ChannelError::NotConfigured("DISCORD_WEBHOOK_URL".to_string()))?;

        let payload = Self::format_payload(event);

        debug!(channel = "discord", event_type = ?event.title(), "Sending notification");

        let response = self.client.post(webhook_url).json(&payload).send().await?;

        if response.status().is_success() {
            debug!(channel = "discord", "Notification sent successfully");
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
                channel = "discord",
                retry_after_secs = retry_after,
                "Rate limited by Discord"
            );

            Err(ChannelError::RateLimited {
                retry_after_secs: retry_after,
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            warn!(
                channel = "discord",
                status = %status,
                body = %body,
                "Discord webhook request failed"
            );

            Err(ChannelError::Other(format!(
                "Discord returned {status}: {body}"
            )))
        }
    }
}

// =============================================================================
// Discord API types
// =============================================================================

#[derive(Debug, Serialize)]
struct DiscordPayload {
    embeds: Vec<DiscordEmbed>,
}

#[derive(Debug, Serialize)]
struct DiscordEmbed {
    title: String,
    description: String,
    color: u32,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<DiscordFooter>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<DiscordField>,
}

#[derive(Debug, Serialize)]
struct DiscordFooter {
    text: String,
}

#[derive(Debug, Serialize)]
struct DiscordField {
    name: String,
    value: String,
    inline: bool,
}

impl DiscordField {
    fn inline(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            inline: true,
        }
    }
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

