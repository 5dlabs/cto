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
    /// Maximum age of bookmarks to fetch (in days).
    /// The poller will keep scrolling until it reaches bookmarks older than this.
    pub max_age_days: i64,
    /// Maximum scroll attempts to prevent infinite loops.
    pub max_scroll_attempts: usize,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(30 * 60), // 30 minutes
            max_age_days: 60,                          // 60 days by default
            max_scroll_attempts: 50,                   // ~50 scrolls max
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

/// Result of fetching bookmarks with stats.
#[derive(Debug, Default)]
pub struct FetchResult {
    /// All bookmarks fetched (within time window).
    pub all_bookmarks: Vec<Bookmark>,
    /// New bookmarks (not yet processed).
    pub new_bookmarks: Vec<Bookmark>,
    /// Number of bookmarks already processed.
    pub already_processed: usize,
    /// Number of scroll operations performed.
    pub scroll_count: usize,
    /// Whether we reached the time window boundary.
    pub reached_time_boundary: bool,
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

    /// Fetch bookmarks and return new (unprocessed) ones.
    ///
    /// Scrolls the timeline until we've loaded all bookmarks from the past
    /// `max_age_days` days, then returns the unprocessed ones (up to batch_size).
    pub async fn poll(&self, state: &mut PollState) -> Result<Vec<Bookmark>> {
        let result = self.fetch_all_bookmarks(state).await?;

        tracing::info!(
            total = result.all_bookmarks.len(),
            new = result.new_bookmarks.len(),
            already_processed = result.already_processed,
            scrolls = result.scroll_count,
            reached_boundary = result.reached_time_boundary,
            "Bookmark fetch complete"
        );

        // Return up to batch_size new bookmarks for processing
        let to_process: Vec<Bookmark> = result
            .new_bookmarks
            .into_iter()
            .take(self.config.batch_size)
            .collect();

        tracing::info!(
            total = result.all_bookmarks.len(),
            batch_size = self.config.batch_size,
            "Found new bookmarks"
        );

        state.record_success();
        Ok(to_process)
    }

    /// Fetch all bookmarks within the time window, with full stats.
    pub async fn fetch_all_bookmarks(&self, state: &PollState) -> Result<FetchResult> {
        tracing::info!(
            max_age_days = self.config.max_age_days,
            "Fetching all bookmarks within time window"
        );

        let (all_bookmarks, scroll_count, reached_boundary) =
            self.fetch_bookmarks_with_scrolling().await?;

        let mut result = FetchResult {
            scroll_count,
            reached_time_boundary: reached_boundary,
            ..Default::default()
        };

        // Partition into new and already processed
        for bookmark in all_bookmarks {
            if state.is_processed(&bookmark.id) {
                tracing::debug!(
                    id = %bookmark.id,
                    "Bookmark already processed, skipping"
                );
                result.already_processed += 1;
            } else {
                result.new_bookmarks.push(bookmark.clone());
            }
            result.all_bookmarks.push(bookmark);
        }

        // Log summary of partition
        tracing::info!(
            total = result.all_bookmarks.len(),
            new = result.new_bookmarks.len(),
            already_processed = result.already_processed,
            "Partitioned bookmarks by processing status"
        );

        Ok(result)
    }

    /// Fetch bookmarks by scrolling until we've loaded all within time window.
    ///
    /// Returns (bookmarks, scroll_count, reached_time_boundary).
    async fn fetch_bookmarks_with_scrolling(&self) -> Result<(Vec<Bookmark>, usize, bool)> {
        use chromiumoxide::browser::{Browser, BrowserConfig};
        use chromiumoxide::cdp::browser_protocol::network::CookieParam;
        use futures::StreamExt;
        use std::collections::HashMap;

        let config = BrowserConfig::builder()
            // Container compatibility
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            // Anti-detection
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-features=IsolateOrigins,site-per-process")
            // Realistic browser setup
            .arg("--window-size=1920,1080")
            .arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            // Performance
            .arg("--disable-gpu")
            .arg("--disable-software-rasterizer")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build browser config: {e}"))?;

        let (mut browser, mut handler) = Browser::launch(config).await?;

        let handle = tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        // Navigate and set cookies
        tracing::debug!("Navigating to x.com to set cookies");
        let page = browser.new_page("https://x.com").await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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
        tokio::time::sleep(tokio::time::Duration::from_secs(8)).await;

        // Check for login redirect
        let url = page.url().await.ok().flatten().unwrap_or_default();
        if url.contains("login") || url.contains("flow") {
            browser.close().await?;
            handle.await?;
            anyhow::bail!("Authentication failed: redirected to login page.");
        }

        // Scroll and collect bookmarks
        let mut all_bookmarks: HashMap<String, Bookmark> = HashMap::new();
        let mut scroll_count = 0;
        let mut reached_time_boundary = false;
        let mut consecutive_no_new = 0;

        loop {
            // Get current page content
            let html = page.content().await?;
            let bookmarks = BookmarkParser::parse(&html)?;

            let before_count = all_bookmarks.len();
            let mut oldest_in_batch: Option<DateTime<Utc>> = None;

            let batch_count = bookmarks.len();
            let mut skipped_age = 0usize;
            let mut added = 0usize;

            for bookmark in bookmarks {
                // Track the oldest tweet in this batch
                if oldest_in_batch.is_none() || bookmark.posted_at < oldest_in_batch.unwrap() {
                    oldest_in_batch = Some(bookmark.posted_at);
                }

                // Log bookmark details for debugging
                let age_days = bookmark.age().num_days();
                tracing::debug!(
                    id = %bookmark.id,
                    posted_at = %bookmark.posted_at,
                    age_days,
                    max_age_days = self.config.max_age_days,
                    "Processing bookmark"
                );

                // Check if this bookmark is too old
                if !bookmark.is_within_days(self.config.max_age_days) {
                    tracing::debug!(
                        id = %bookmark.id,
                        posted_at = %bookmark.posted_at,
                        age_days,
                        max_age_days = self.config.max_age_days,
                        "Bookmark older than max_age_days, skipping"
                    );
                    skipped_age += 1;
                    reached_time_boundary = true;
                    continue; // Don't add to collection
                }

                // Add to collection (HashMap ensures no duplicates)
                all_bookmarks.entry(bookmark.id.clone()).or_insert(bookmark);
                added += 1;
            }

            // Log batch summary at INFO level for better visibility
            tracing::info!(
                batch_count,
                skipped_age,
                added,
                total_collected = all_bookmarks.len(),
                max_age_days = self.config.max_age_days,
                "Batch processing complete"
            );

            let new_in_scroll = all_bookmarks.len() - before_count;
            tracing::debug!(
                scroll = scroll_count,
                new = new_in_scroll,
                total = all_bookmarks.len(),
                oldest = ?oldest_in_batch,
                "Scroll iteration"
            );

            // Stop conditions
            if reached_time_boundary {
                tracing::info!(
                    scroll_count,
                    total = all_bookmarks.len(),
                    "Reached time boundary ({} days)",
                    self.config.max_age_days
                );
                break;
            }

            if new_in_scroll == 0 {
                consecutive_no_new += 1;
                if consecutive_no_new >= 3 {
                    tracing::info!(
                        scroll_count,
                        total = all_bookmarks.len(),
                        "No new bookmarks after 3 scrolls, likely reached end"
                    );
                    break;
                }
            } else {
                consecutive_no_new = 0;
            }

            if scroll_count >= self.config.max_scroll_attempts {
                tracing::warn!(
                    max = self.config.max_scroll_attempts,
                    total = all_bookmarks.len(),
                    "Reached max scroll attempts"
                );
                break;
            }

            // Scroll down
            scroll_count += 1;
            if let Err(e) = page
                .evaluate("window.scrollTo(0, document.body.scrollHeight)")
                .await
            {
                tracing::warn!(error = %e, "Scroll failed");
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        // Debug dump if requested
        if std::env::var("RESEARCH_DUMP_HTML").is_ok() {
            let dump_path = std::env::var("RESEARCH_DUMP_PATH")
                .unwrap_or_else(|_| "/tmp/twitter-bookmarks.html".to_string());
            if let Ok(html) = page.content().await {
                let _ = std::fs::write(&dump_path, &html);
                tracing::info!(path = %dump_path, "Dumped final HTML");
            }
        }

        browser.close().await?;
        handle.await?;

        let bookmarks: Vec<Bookmark> = all_bookmarks.into_values().collect();
        Ok((bookmarks, scroll_count, reached_time_boundary))
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
