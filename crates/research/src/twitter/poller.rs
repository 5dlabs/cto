//! Twitter bookmark poller.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use crate::auth::Session;

use super::parser::BookmarkParser;
use super::types::Bookmark;

/// Configuration for bookmark polling.
#[derive(Debug, Clone)]
pub struct PollConfig {
    /// Max bookmarks to process per poll.
    pub batch_size: usize,
    /// Backoff multiplier on errors.
    pub backoff_multiplier: f32,
    /// Max backoff duration.
    pub max_backoff: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(30 * 60), // 30 minutes
        }
    }
}

/// Tracks polling state across runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PollState {
    /// Last successful poll timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_poll: Option<DateTime<Utc>>,
    /// Last seen bookmark ID (for incremental fetching).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Consecutive failure count.
    #[serde(default)]
    pub failures: u32,
    /// IDs of already processed bookmarks.
    #[serde(default)]
    pub processed: HashSet<String>,
}

impl PollState {
    /// Load state from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let state: Self = serde_json::from_str(&content)?;
            Ok(state)
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Mark a bookmark as processed.
    pub fn mark_processed(&mut self, id: &str) {
        self.processed.insert(id.to_string());
    }

    /// Check if a bookmark has been processed.
    pub fn is_processed(&self, id: &str) -> bool {
        self.processed.contains(id)
    }

    /// Record a successful poll.
    pub fn record_success(&mut self) {
        self.last_poll = Some(Utc::now());
        self.failures = 0;
    }

    /// Record a failed poll.
    pub fn record_failure(&mut self) {
        self.failures += 1;
    }

    /// Get the current backoff duration based on failure count.
    pub fn backoff_duration(&self, config: &PollConfig) -> Duration {
        if self.failures == 0 {
            return Duration::ZERO;
        }

        let base_secs = 60.0; // 1 minute base
        let backoff_secs = base_secs * config.backoff_multiplier.powi(self.failures as i32 - 1);
        let capped_secs = backoff_secs.min(config.max_backoff.as_secs_f32());

        Duration::from_secs_f32(capped_secs)
    }
}

/// Bookmark poller that fetches new bookmarks from Twitter.
pub struct BookmarkPoller {
    #[allow(dead_code)]
    session: Session,
    config: PollConfig,
}

impl BookmarkPoller {
    /// Create a new poller with the given session.
    #[must_use]
    pub fn new(session: Session, config: PollConfig) -> Self {
        Self { session, config }
    }

    /// Fetch bookmarks page and return new (unprocessed) bookmarks.
    pub async fn poll(&self, state: &mut PollState) -> Result<Vec<Bookmark>> {
        tracing::info!("Fetching bookmarks page");

        let html = self.fetch_bookmarks_page().await?;
        let all_bookmarks = BookmarkParser::parse(&html)?;

        // Filter to only new bookmarks
        let new_bookmarks: Vec<Bookmark> = all_bookmarks
            .into_iter()
            .filter(|b| !state.is_processed(&b.id))
            .take(self.config.batch_size)
            .collect();

        tracing::info!(
            total = new_bookmarks.len(),
            batch_size = self.config.batch_size,
            "Found new bookmarks"
        );

        state.record_success();
        Ok(new_bookmarks)
    }

    /// Fetch the bookmarks page HTML.
    async fn fetch_bookmarks_page(&self) -> Result<String> {
        use chromiumoxide::browser::{Browser, BrowserConfig};
        use chromiumoxide::cdp::browser_protocol::network::CookieParam;
        use futures::StreamExt;

        let config = BrowserConfig::builder()
            .arg("--no-sandbox") // Required for containerized environments
            .arg("--disable-dev-shm-usage") // Avoid /dev/shm size issues in containers
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build browser config: {e}"))?;
        let (mut browser, mut handler) = Browser::launch(config).await?;

        // Spawn handler task
        let handle = tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        // Navigate to x.com first to establish domain context for cookies
        tracing::debug!("Navigating to x.com to set cookies");
        let page = browser.new_page("https://x.com").await?;

        // Wait briefly for initial page load
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Set auth cookies
        tracing::debug!("Setting Twitter auth cookies");

        let auth_cookie = CookieParam::builder()
            .name("auth_token")
            .value(&self.session.auth_token)
            .domain(".x.com")
            .path("/")
            .secure(true)
            .http_only(true)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build auth cookie: {e}"))?;
        page.set_cookie(auth_cookie).await?;

        // Set ct0 if available
        if let Some(ct0) = &self.session.ct0 {
            let ct0_cookie = CookieParam::builder()
                .name("ct0")
                .value(ct0)
                .domain(".x.com")
                .path("/")
                .secure(true)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build ct0 cookie: {e}"))?;
            page.set_cookie(ct0_cookie).await?;
        }

        // Navigate to bookmarks
        tracing::debug!("Navigating to bookmarks page");
        page.goto("https://x.com/i/bookmarks").await?;

        // Wait for content to load (Twitter is JS-heavy)
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Check if we got redirected to login
        let url = page.url().await?.unwrap_or_default();
        if url.contains("login") || url.contains("flow") {
            tracing::error!(
                url,
                "Redirected to login - auth cookies are invalid or expired"
            );
            browser.close().await?;
            handle.await?;
            anyhow::bail!("Authentication failed: redirected to login page. Twitter session cookies are invalid or expired.");
        }

        let html = page.content().await?;
        tracing::debug!(len = html.len(), "Got page content");

        // Debug: dump HTML to file for inspection
        if std::env::var("RESEARCH_DUMP_HTML").is_ok() {
            let dump_path = std::env::var("RESEARCH_DUMP_PATH")
                .unwrap_or_else(|_| "/tmp/twitter-bookmarks.html".to_string());
            if let Err(e) = std::fs::write(&dump_path, &html) {
                tracing::warn!(path = %dump_path, error = %e, "Failed to dump HTML");
            } else {
                tracing::info!(path = %dump_path, "Dumped HTML for inspection");
            }
        }

        browser.close().await?;
        handle.await?;

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_state_default() {
        let state = PollState::default();
        assert!(state.last_poll.is_none());
        assert!(state.cursor.is_none());
        assert_eq!(state.failures, 0);
        assert!(state.processed.is_empty());
    }

    #[test]
    fn test_poll_state_processed() {
        let mut state = PollState::default();
        assert!(!state.is_processed("123"));

        state.mark_processed("123");
        assert!(state.is_processed("123"));
    }

    #[test]
    fn test_backoff_duration() {
        let config = PollConfig::default();
        let mut state = PollState::default();

        // No failures = no backoff
        assert_eq!(state.backoff_duration(&config), Duration::ZERO);

        // First failure = base backoff
        state.failures = 1;
        assert!(state.backoff_duration(&config) > Duration::ZERO);

        // More failures = exponential backoff (capped)
        state.failures = 10;
        assert!(state.backoff_duration(&config) <= config.max_backoff);
    }
}
