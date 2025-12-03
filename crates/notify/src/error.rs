//! Error types for the notification system.

use thiserror::Error;

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
    RateLimited { retry_after_secs: u64 },

    /// Other error
    #[error("{0}")]
    Other(String),
}

