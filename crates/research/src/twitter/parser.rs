//! Twitter bookmark HTML parser.

use anyhow::Result;
use scraper::{Html, Selector};

use super::types::{Author, Bookmark};
use chrono::Utc;

/// Parser for Twitter bookmark page HTML.
pub struct BookmarkParser;

impl BookmarkParser {
    /// Parse bookmarks from HTML content.
    ///
    /// Twitter's HTML structure uses data-testid="tweet" for tweet containers.
    pub fn parse(html: &str) -> Result<Vec<Bookmark>> {
        let document = Html::parse_document(html);
        let mut bookmarks = Vec::new();

        // Prefer the stable testid selector; fall back to article containers if X changes markup.
        let tweet_selector =
            Selector::parse("[data-testid='tweet']").expect("Invalid tweet selector");
        let article_selector = Selector::parse("article").expect("Invalid article selector");

        // Extract tweet links and text
        let link_selector = Selector::parse("a[href*='/status/']").expect("Invalid link selector");
        let text_selector =
            Selector::parse("[data-testid='tweetText']").expect("Invalid text selector");
        let lang_text_selector = Selector::parse("div[lang]").expect("Invalid lang text selector");

        let mut tweets: Vec<_> = document.select(&tweet_selector).collect();
        if tweets.is_empty() {
            tweets = document.select(&article_selector).collect();
        }
        tracing::debug!(tweet_count = tweets.len(), "Found tweet containers");

        if tweets.is_empty() {
            tracing::warn!(
                "No tweet containers found in HTML (selectors: data-testid=tweet, article). \
                 The page may not have rendered or X markup may have changed."
            );
            return Ok(bookmarks);
        }

        for tweet in tweets {
            // Extract the tweet link (contains username and tweet ID)
            let tweet_link = tweet.select(&link_selector).find(|el| {
                el.value()
                    .attr("href")
                    .is_some_and(|h| h.contains("/status/"))
            });

            let Some(link) = tweet_link else {
                tracing::debug!("Tweet missing status link, skipping");
                continue;
            };

            let href = link.value().attr("href").unwrap_or_default();
            let Some((username, tweet_id)) = Self::parse_tweet_url(href) else {
                tracing::debug!(href, "Could not parse tweet URL");
                continue;
            };

            // Extract the tweet text.
            // Prefer the tweetText testid; fall back to div[lang] which often holds the rendered text.
            let text = tweet
                .select(&text_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .or_else(|| {
                    tweet
                        .select(&lang_text_selector)
                        .next()
                        .map(|el| el.text().collect::<String>())
                })
                .unwrap_or_default();

            if text.is_empty() {
                tracing::debug!(tweet_id, "Tweet has no text, skipping");
                continue;
            }

            // Create the bookmark
            let mut bookmark = Bookmark::new(
                tweet_id.clone(),
                Author::new(username.clone(), username), // Display name same as handle for now
                text,
                Utc::now(), // We don't have the actual timestamp easily
            );

            // Extract URLs from the tweet text
            bookmark.extract_urls();
            if !bookmark.urls.is_empty() {
                tracing::debug!(id = %tweet_id, urls = ?bookmark.urls, "Found URLs in tweet");
            }

            tracing::debug!(id = %tweet_id, "Parsed bookmark");
            bookmarks.push(bookmark);
        }

        tracing::info!(count = bookmarks.len(), "Parsed bookmarks from HTML");
        Ok(bookmarks)
    }

    /// Parse a tweet URL to extract username and tweet ID.
    /// URLs are like: /username/status/1234567890 or /username/status/1234567890?s=20
    fn parse_tweet_url(url: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = url.trim_start_matches('/').split('/').collect();
        if parts.len() >= 3 && parts[1] == "status" {
            // Strip query parameters and fragments from the ID
            let id = parts[2]
                .split('?')
                .next()
                .unwrap_or(parts[2])
                .split('#')
                .next()
                .unwrap_or(parts[2]);
            Some((parts[0].to_string(), id.to_string()))
        } else {
            None
        }
    }

    /// Extract tweet ID from a URL (full or relative).
    pub fn extract_tweet_id(url: &str) -> Option<String> {
        // Handle both full URLs and relative paths
        // Full: https://x.com/user/status/123 or https://twitter.com/user/status/123
        // Relative: /user/status/123
        let path = if url.starts_with("http") {
            url.split('/').skip(3).collect::<Vec<_>>().join("/")
        } else {
            url.trim_start_matches('/').to_string()
        };
        Self::parse_tweet_url(&format!("/{path}")).map(|(_, id)| id)
    }

    /// Create a placeholder bookmark for testing.
    #[cfg(test)]
    pub fn placeholder(id: &str, text: &str) -> Bookmark {
        Bookmark::new(
            id.to_string(),
            Author::new("test".to_string(), "Test User".to_string()),
            text.to_string(),
            Utc::now(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tweet_id() {
        assert_eq!(
            BookmarkParser::extract_tweet_id("https://x.com/user/status/123456"),
            Some("123456".to_string())
        );
        assert_eq!(
            BookmarkParser::extract_tweet_id("https://twitter.com/user/status/789"),
            Some("789".to_string())
        );
        assert_eq!(BookmarkParser::extract_tweet_id("https://google.com"), None);
    }
}
