//! Configuration for the email digest system.

use anyhow::{Context, Result};

/// Default Gmail SMTP host.
pub const DEFAULT_SMTP_HOST: &str = "smtp.gmail.com";

/// Default Gmail SMTP port (STARTTLS).
pub const DEFAULT_SMTP_PORT: u16 = 587;

/// Default burst threshold - send immediately if this many entries accumulate.
pub const DEFAULT_BURST_THRESHOLD: usize = 15;

/// Default minimum entries for a scheduled digest (skip if fewer).
pub const DEFAULT_MIN_FOR_DIGEST: usize = 3;

/// Default recipient email.
pub const DEFAULT_TO_EMAIL: &str = "jay@jonathonfritz.com";

/// Configuration for the email digest system.
#[derive(Debug, Clone)]
pub struct DigestConfig {
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port.
    pub smtp_port: u16,
    /// SMTP username (Gmail address).
    pub smtp_username: String,
    /// SMTP password (Gmail app password).
    pub smtp_password: String,
    /// Recipient email address.
    pub to_email: String,
    /// Sender email address (usually same as username).
    pub from_email: String,
    /// Send immediately if this many entries accumulate since last digest.
    pub burst_threshold: usize,
    /// Minimum entries required for a scheduled digest.
    pub min_for_digest: usize,
}

impl DigestConfig {
    /// Create configuration from environment variables.
    ///
    /// # Required Environment Variables
    /// - `GMAIL_USERNAME`: Gmail address for sending
    /// - `GMAIL_APP_PASSWORD`: Gmail app password (not regular password)
    ///
    /// # Optional Environment Variables
    /// - `DIGEST_TO_EMAIL`: Recipient (default: jay@jonathonfritz.com)
    /// - `DIGEST_BURST_THRESHOLD`: Burst threshold (default: 15)
    /// - `DIGEST_MIN_ENTRIES`: Minimum for scheduled digest (default: 3)
    pub fn from_env() -> Result<Self> {
        let smtp_username = std::env::var("GMAIL_USERNAME")
            .context("GMAIL_USERNAME environment variable not set")?;

        let smtp_password = std::env::var("GMAIL_APP_PASSWORD")
            .context("GMAIL_APP_PASSWORD environment variable not set")?;

        let to_email =
            std::env::var("DIGEST_TO_EMAIL").unwrap_or_else(|_| DEFAULT_TO_EMAIL.to_string());

        let burst_threshold = std::env::var("DIGEST_BURST_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_BURST_THRESHOLD);

        let min_for_digest = std::env::var("DIGEST_MIN_ENTRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_MIN_FOR_DIGEST);

        Ok(Self {
            smtp_host: DEFAULT_SMTP_HOST.to_string(),
            smtp_port: DEFAULT_SMTP_PORT,
            smtp_username: smtp_username.clone(),
            smtp_password,
            to_email,
            from_email: smtp_username,
            burst_threshold,
            min_for_digest,
        })
    }

    /// Check if the burst threshold has been reached.
    #[must_use]
    pub fn should_burst_send(&self, pending_count: usize) -> bool {
        pending_count >= self.burst_threshold
    }

    /// Check if there are enough entries for a scheduled digest.
    #[must_use]
    pub fn has_enough_for_digest(&self, pending_count: usize) -> bool {
        pending_count >= self.min_for_digest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burst_threshold() {
        let config = DigestConfig {
            smtp_host: "test".to_string(),
            smtp_port: 587,
            smtp_username: "test@gmail.com".to_string(),
            smtp_password: "test".to_string(),
            to_email: "test@example.com".to_string(),
            from_email: "test@gmail.com".to_string(),
            burst_threshold: 15,
            min_for_digest: 3,
        };

        assert!(!config.should_burst_send(14));
        assert!(config.should_burst_send(15));
        assert!(config.should_burst_send(20));
    }

    #[test]
    fn test_min_for_digest() {
        let config = DigestConfig {
            smtp_host: "test".to_string(),
            smtp_port: 587,
            smtp_username: "test@gmail.com".to_string(),
            smtp_password: "test".to_string(),
            to_email: "test@example.com".to_string(),
            from_email: "test@gmail.com".to_string(),
            burst_threshold: 15,
            min_for_digest: 3,
        };

        assert!(!config.has_enough_for_digest(2));
        assert!(config.has_enough_for_digest(3));
        assert!(config.has_enough_for_digest(10));
    }
}


