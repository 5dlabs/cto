//! Zulip integration for threaded team messaging.
//!
//! This module will provide integration with Zulip:
//!
//! - Stream and topic-based threading model
//! - Bot integration for notifications
//! - Excellent async communication support
//! - Self-hosted and cloud options
//!
//! Zulip's unique Streams/Topics model makes it excellent for organizing
//! conversations and async communication across time zones.
//!
//! # Status
//!
//! This integration is currently scaffolding only. Implementation coming soon.
//!
//! # Example (Future API)
//!
//! ```ignore
//! use integrations::alerts::messaging::zulip::ZulipClient;
//!
//! let client = ZulipClient::new(
//!     "https://zulip.example.com",
//!     "bot@example.com",
//!     "api_key"
//! )?;
//!
//! // Send to a stream with topic
//! client.send_message("alerts", "cto-platform", "New deployment started").await?;
//! ```
//!
//! # Configuration
//!
//! - `ZULIP_SITE`: Zulip server URL
//! - `ZULIP_EMAIL`: Bot email address
//! - `ZULIP_API_KEY`: Bot API key
//! - `ZULIP_DEFAULT_STREAM`: Default stream for notifications
//! - `ZULIP_DEFAULT_TOPIC`: Default topic within the stream

// TODO: Implement Zulip REST API client
// TODO: Implement stream/topic message sending
// TODO: Implement message formatting (Zulip markdown)
// TODO: Implement webhook incoming support
