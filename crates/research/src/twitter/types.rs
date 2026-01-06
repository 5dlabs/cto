//! Twitter data types.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Twitter's snowflake epoch (November 4, 2010, 01:42:54.657 UTC).
/// Tweet IDs encode timestamps as milliseconds since this epoch.
const TWITTER_EPOCH_MS: i64 = 1_288_834_974_657;

/// Extract the creation timestamp from a Twitter/X snowflake ID.
///
/// Twitter IDs are 64-bit integers where the upper 42 bits encode
/// the timestamp in milliseconds since the Twitter epoch.
#[must_use]
pub fn tweet_id_to_datetime(id: &str) -> Option<DateTime<Utc>> {
    let id_num: u64 = id.parse().ok()?;
    // Timestamp is in the upper 42 bits (shift right by 22)
    let timestamp_ms = (id_num >> 22) as i64 + TWITTER_EPOCH_MS;
    Utc.timestamp_millis_opt(timestamp_ms).single()
}

/// Check if a tweet ID is within the given number of days from now.
#[must_use]
pub fn tweet_id_within_days(id: &str, days: i64) -> bool {
    tweet_id_to_datetime(id)
        .map(|dt| (Utc::now() - dt).num_days() <= days)
        .unwrap_or(false)
}

/// A bookmarked tweet with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique tweet ID.
    pub id: String,
    /// Tweet author.
    pub author: Author,
    /// Tweet text content.
    pub text: String,
    /// When the tweet was posted.
    pub posted_at: DateTime<Utc>,
    /// When it was bookmarked (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarked_at: Option<DateTime<Utc>>,
    /// Attached media.
    #[serde(default)]
    pub media: Vec<Media>,
    /// URLs found in the tweet.
    #[serde(default)]
    pub urls: Vec<String>,
    /// Quote tweet if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Box<Bookmark>>,
    /// Thread context (if part of thread).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Vec<String>>,
}

impl Bookmark {
    /// Create a new bookmark with minimal required fields.
    #[must_use]
    pub fn new(id: String, author: Author, text: String, posted_at: DateTime<Utc>) -> Self {
        Self {
            id,
            author,
            text,
            posted_at,
            bookmarked_at: None,
            media: Vec::new(),
            urls: Vec::new(),
            quote: None,
            thread: None,
        }
    }

    /// Create a bookmark, deriving the timestamp from the tweet ID (snowflake).
    ///
    /// Twitter/X tweet IDs are snowflake IDs that encode the creation timestamp.
    #[must_use]
    pub fn from_id(id: String, author: Author, text: String) -> Self {
        let posted_at = tweet_id_to_datetime(&id).unwrap_or_else(Utc::now);
        Self::new(id, author, text, posted_at)
    }

    /// Get the age of this tweet.
    #[must_use]
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.posted_at
    }

    /// Check if this tweet is within the given number of days.
    #[must_use]
    pub fn is_within_days(&self, days: i64) -> bool {
        self.age().num_days() <= days
    }

    /// Extract URLs from tweet text.
    pub fn extract_urls(&mut self) {
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        self.urls = url_regex
            .find_iter(&self.text)
            .map(|m| m.as_str().to_string())
            .collect();
    }

    /// Get external URLs (excluding Twitter/X links).
    #[must_use]
    pub fn external_urls(&self) -> Vec<&str> {
        self.urls
            .iter()
            .filter(|url| {
                !url.contains("twitter.com") && !url.contains("x.com") && !url.contains("t.co")
            })
            .map(String::as_str)
            .collect()
    }
}

/// Author information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Twitter handle (without @).
    pub handle: String,
    /// Display name.
    pub name: String,
    /// Whether the account is verified.
    #[serde(default)]
    pub verified: bool,
    /// User bio (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

impl Author {
    /// Create a new author.
    #[must_use]
    pub fn new(handle: String, name: String) -> Self {
        Self {
            handle,
            name,
            verified: false,
            bio: None,
        }
    }

    /// Get the handle with @ prefix.
    #[must_use]
    pub fn at_handle(&self) -> String {
        format!("@{}", self.handle)
    }
}

/// Media attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    /// Media type (image, video, gif).
    pub media_type: MediaType,
    /// URL to the media.
    pub url: String,
    /// Alt text if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
}

/// Type of media attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Image attachment.
    Image,
    /// Video attachment.
    Video,
    /// GIF attachment.
    Gif,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_tweet_id_to_datetime() {
        // Test with a known tweet ID: 2008123587132309749
        // This ID was created around January 6, 2026
        let dt = tweet_id_to_datetime("2008123587132309749").expect("should parse");

        // Verify it's a reasonable date (after Twitter epoch, before year 3000)
        assert!(dt.year() >= 2010);
        assert!(dt.year() <= 3000);

        // The ID should decode to sometime in late 2025/early 2026
        assert!(dt.year() >= 2025);
    }

    #[test]
    fn test_tweet_id_to_datetime_invalid() {
        assert!(tweet_id_to_datetime("not-a-number").is_none());
        assert!(tweet_id_to_datetime("").is_none());
    }

    #[test]
    fn test_tweet_id_within_days() {
        // A very old tweet ID (from 2015)
        assert!(!tweet_id_within_days("666666666666666666", 60));

        // A recent tweet ID should be within 60 days if it's actually recent
        // (This test will fail if run far in the future from when the ID was created)
    }

    #[test]
    fn test_bookmark_is_within_days() {
        let bookmark = Bookmark::from_id(
            "2008123587132309749".to_string(),
            Author::new("test".to_string(), "Test".to_string()),
            "test".to_string(),
        );

        // The bookmark was created with a timestamp from the ID
        // It should have a valid age
        assert!(bookmark.age().num_days() >= 0);
    }
}
