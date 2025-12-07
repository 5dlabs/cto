//! Alerting, messaging, and incident management integrations.
//!
//! This module provides a comprehensive notification and incident management system
//! organized into two main categories:
//!
//! ## Messaging (`messaging/`)
//!
//! Team messaging platforms for notifications and agent-to-human communication:
//!
//! - **Discord** - Rich embeds and webhooks (implemented)
//! - **Slack** - Enterprise messaging with webhooks (implemented)
//! - **Microsoft Teams** - Enterprise collaboration (scaffolding)
//! - **Element (Matrix)** - Self-hosted, encrypted, decentralized (scaffolding)
//! - **Mattermost** - Self-hosted Slack alternative (scaffolding)
//! - **Zulip** - Threaded Streams/Topics model (scaffolding)
//! - **Google Chat** - Google Workspace integration (scaffolding)
//! - **Rocket.Chat** - Open-source with omnichannel (scaffolding)
//!
//! ## Incidents (`incidents/`)
//!
//! Incident management and on-call alerting platforms:
//!
//! - **PagerDuty** - Industry-standard Events API v2 (implemented)
//! - **Opsgenie** - Atlassian incident management (scaffolding, EOL 2027)
//! - **incident.io** - Chat-native for Slack/Teams (scaffolding)
//! - **Better Stack** - All-in-one monitoring + incidents (scaffolding)
//! - **Zenduty** - Fast-growing PagerDuty alternative (scaffolding)
//! - **Grafana OnCall** - Free, open-source (scaffolding)
//! - **Rootly** - Comprehensive incident response (scaffolding)
//! - **Spike** - Simple 5-minute setup (scaffolding)
//!
//! # Architecture
//!
//! The notification system uses a trait-based channel design for extensibility:
//!
//! - [`NotifyChannel`] trait defines the interface for notification channels
//! - [`Notifier`] dispatches events to all enabled channels
//!
//! # Usage
//!
//! ```no_run
//! use integrations::alerts::{Notifier, NotifyEvent};
//!
//! // Create notifier from environment variables
//! let notifier = Notifier::from_env();
//!
//! // Send a notification (fire-and-forget)
//! notifier.notify(NotifyEvent::PlayStarted {
//!     task_id: "42".to_string(),
//!     repository: "5dlabs/example".to_string(),
//!     workflow_name: "play-42-abc123".to_string(),
//!     timestamp: chrono::Utc::now(),
//! });
//! ```
//!
//! # Configuration
//!
//! The notifier is configured via environment variables:
//!
//! - `DISCORD_WEBHOOK_URL`: Discord webhook URL (enables Discord channel)
//! - `SLACK_WEBHOOK_URL`: Slack webhook URL (enables Slack channel)
//! - `PAGERDUTY_ROUTING_KEY`: PagerDuty routing key (enables PagerDuty)
//! - `NOTIFY_DISABLED`: Set to "true" to disable all notifications

pub mod events;
pub mod incidents;
pub mod messaging;

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, warn};

pub use events::{NotifyEvent, Severity};

// Re-export commonly used types from submodules
pub use incidents::pagerduty::{
    EventAction, EventPayload, EventSeverity, PagerDutyClient, PagerDutyEvent,
};
pub use messaging::{DiscordChannel, SlackChannel};

/// Environment variable to disable all notifications.
const ENV_NOTIFY_DISABLED: &str = "NOTIFY_DISABLED";

/// Errors that can occur when sending notifications.
#[derive(Debug, Error)]
pub enum ChannelError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Channel is not configured
    #[error("Channel not configured: {0}")]
    NotConfigured(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Rate limited by the service
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited {
        /// Seconds to wait before retrying
        retry_after_secs: u64,
    },

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Trait for notification channels (Discord, Slack, etc.).
#[async_trait]
pub trait NotifyChannel: Send + Sync {
    /// Get the name of this channel.
    fn name(&self) -> &'static str;

    /// Check if this channel is enabled/configured.
    fn enabled(&self) -> bool;

    /// Send a notification event to this channel.
    async fn send(&self, event: &NotifyEvent) -> Result<(), ChannelError>;
}

/// Central notification dispatcher.
///
/// The `Notifier` manages multiple notification channels and dispatches
/// events to all enabled channels in a fire-and-forget manner.
pub struct Notifier {
    channels: Vec<Arc<dyn NotifyChannel>>,
    disabled: bool,
}

impl Notifier {
    /// Create a new notifier from environment variables.
    ///
    /// This will auto-detect which channels are configured based on
    /// environment variables and enable them accordingly.
    #[must_use]
    pub fn from_env() -> Self {
        let disabled = std::env::var(ENV_NOTIFY_DISABLED)
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        if disabled {
            info!("Notifications disabled via NOTIFY_DISABLED");
            return Self {
                channels: vec![],
                disabled: true,
            };
        }

        let mut channels: Vec<Arc<dyn NotifyChannel>> = vec![];

        // Add Discord channel
        let discord = DiscordChannel::from_env();
        if discord.enabled() {
            info!("Discord notifications enabled");
            channels.push(Arc::new(discord));
        }

        // Add Slack channel
        let slack = SlackChannel::from_env();
        if slack.enabled() {
            info!("Slack notifications enabled");
            channels.push(Arc::new(slack));
        }

        if channels.is_empty() {
            warn!("No notification channels configured");
        } else {
            info!(
                channel_count = channels.len(),
                "Notification system initialized"
            );
        }

        Self {
            channels,
            disabled: false,
        }
    }

    /// Create a notifier with specific channels.
    #[must_use]
    pub fn with_channels(channels: Vec<Arc<dyn NotifyChannel>>) -> Self {
        Self {
            channels,
            disabled: false,
        }
    }

    /// Create a disabled notifier (for testing or when notifications are off).
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            channels: vec![],
            disabled: true,
        }
    }

    /// Check if any notification channels are enabled.
    #[must_use]
    pub fn has_channels(&self) -> bool {
        !self.disabled && !self.channels.is_empty()
    }

    /// Get the number of enabled channels.
    #[must_use]
    pub fn channel_count(&self) -> usize {
        if self.disabled {
            0
        } else {
            self.channels.len()
        }
    }

    /// Send a notification to all enabled channels (fire-and-forget).
    ///
    /// This method spawns async tasks for each channel and returns immediately.
    /// Errors are logged but not propagated to the caller.
    pub fn notify(&self, event: NotifyEvent) {
        if self.disabled {
            debug!("Notifications disabled, skipping event");
            return;
        }

        if self.channels.is_empty() {
            debug!("No channels configured, skipping event");
            return;
        }

        let event = Arc::new(event);

        for channel in &self.channels {
            let channel = Arc::clone(channel);
            let event = Arc::clone(&event);

            tokio::spawn(async move {
                let channel_name = channel.name();

                if !channel.enabled() {
                    debug!(channel = channel_name, "Channel disabled, skipping");
                    return;
                }

                match channel.send(&event).await {
                    Ok(()) => {
                        debug!(channel = channel_name, "Notification sent");
                    }
                    Err(e) => {
                        error!(
                            channel = channel_name,
                            error = %e,
                            "Failed to send notification"
                        );
                    }
                }
            });
        }
    }

    /// Send a notification and wait for all channels to complete.
    ///
    /// Unlike `notify()`, this method waits for all notifications to be sent
    /// and collects any errors. Useful for testing or when delivery confirmation
    /// is needed.
    pub async fn notify_and_wait(
        &self,
        event: NotifyEvent,
    ) -> Vec<(String, Result<(), ChannelError>)> {
        if self.disabled || self.channels.is_empty() {
            return vec![];
        }

        let mut results = vec![];

        for channel in &self.channels {
            let channel_name = channel.name().to_string();
            let result = channel.send(&event).await;
            results.push((channel_name, result));
        }

        results
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_notifier() {
        let notifier = Notifier::disabled();
        assert!(!notifier.has_channels());
        assert_eq!(notifier.channel_count(), 0);
    }

    #[test]
    fn test_severity_colors() {
        assert_eq!(Severity::Info.color(), 0x0034_98db);
        assert_eq!(Severity::Warning.color(), 0x00f3_9c12);
        assert_eq!(Severity::Critical.color(), 0x00e7_4c3c);
    }

    #[test]
    fn test_event_titles() {
        let event = NotifyEvent::PlayStarted {
            task_id: "42".to_string(),
            repository: "test/repo".to_string(),
            workflow_name: "wf-123".to_string(),
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(event.title(), "Play Started: Task #42");

        let event = NotifyEvent::HealAlert {
            alert_id: "A7".to_string(),
            severity: Severity::Critical,
            message: "Pod failed".to_string(),
            context: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(event.title(), "HEAL Alert: A7");
    }
}
