//! Team messaging integrations.
//!
//! This module provides integrations with various team messaging and chat platforms
//! for sending notifications, alerts, and enabling agent-to-human communication.
//!
//! ## Fully Implemented
//!
//! - **Discord** - Gaming-origin team chat with rich embeds and webhooks
//! - **Slack** - Enterprise team messaging with webhooks and bot integration
//!
//! ## Scaffolding (Coming Soon)
//!
//! - **Microsoft Teams** - Enterprise collaboration (270M+ daily users)
//! - **Element (Matrix)** - Self-hosted, encrypted, decentralized messaging
//! - **Mattermost** - Self-hosted Slack alternative for enterprises
//! - **Zulip** - Threaded messaging with Streams/Topics model
//! - **Google Chat** - Google Workspace integration
//! - **Rocket.Chat** - Open-source with omnichannel support
//!
//! # Example
//!
//! ```no_run
//! use integrations::alerts::messaging::{DiscordChannel, SlackChannel};
//! use integrations::alerts::NotifyChannel;
//!
//! // Create channels from environment
//! let discord = DiscordChannel::from_env();
//! let slack = SlackChannel::from_env();
//!
//! // Check if configured
//! if discord.enabled() {
//!     println!("Discord notifications ready");
//! }
//! ```

pub mod discord;
pub mod element;
pub mod google_chat;
pub mod mattermost;
pub mod rocket_chat;
pub mod slack;
pub mod teams;
pub mod zulip;

// Re-export implemented channels
pub use discord::DiscordChannel;
pub use slack::SlackChannel;
