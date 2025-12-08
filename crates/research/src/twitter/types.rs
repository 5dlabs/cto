//! Twitter data types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
