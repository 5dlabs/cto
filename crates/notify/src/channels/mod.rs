//! Notification channel implementations.

pub mod discord;

use async_trait::async_trait;

use crate::error::ChannelError;
use crate::events::NotifyEvent;

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


