//! Link enrichment logic.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::firecrawl::{FirecrawlClient, ScrapeOptions};
use crate::twitter::Bookmark;

/// Enriched content from a scraped link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedLink {
    /// Original URL.
    pub url: String,
    /// Resolved page title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Markdown content.
    pub content: String,
    /// Key excerpts from the content.
    #[serde(default)]
    pub excerpts: Vec<String>,
}

/// Configuration for link enrichment.
#[derive(Debug, Clone)]
pub struct EnrichConfig {
    /// Max links to enrich per tweet.
    pub max_links: usize,
    /// Timeout per scrape.
    pub timeout: Duration,
    /// Maximum content length to store.
    pub max_content_length: usize,
}

impl Default for EnrichConfig {
    fn default() -> Self {
        Self {
            max_links: 3,
            timeout: Duration::from_secs(30),
            max_content_length: 50_000,
        }
    }
}

/// Enriches bookmarks with scraped link content.
pub struct LinkEnricher {
    client: FirecrawlClient,
    config: EnrichConfig,
}

impl LinkEnricher {
    /// Create a new link enricher.
    pub fn new(client: FirecrawlClient, config: EnrichConfig) -> Self {
        Self { client, config }
    }

    /// Create from environment with default config.
    pub fn from_env() -> Result<Self> {
        let client = FirecrawlClient::from_env()?;
        Ok(Self::new(client, EnrichConfig::default()))
    }

    /// Enrich a bookmark with scraped link content.
    pub async fn enrich(&self, bookmark: &Bookmark) -> Result<Vec<EnrichedLink>> {
        let mut enriched = Vec::new();
        let external_urls = bookmark.external_urls();

        for url in external_urls.iter().take(self.config.max_links) {
            match self.scrape_url(url).await {
                Ok(link) => {
                    tracing::info!(url, "Successfully enriched link");
                    enriched.push(link);
                }
                Err(e) => {
                    tracing::warn!(url, error = %e, "Failed to enrich link");
                }
            }
        }

        Ok(enriched)
    }

    /// Scrape a single URL and create an enriched link.
    async fn scrape_url(&self, url: &str) -> Result<EnrichedLink> {
        let options = ScrapeOptions {
            formats: vec!["markdown".to_string()],
            only_main_content: true,
            timeout: self.config.timeout,
        };

        let response = self.client.scrape(url, &options).await?;

        let data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in scrape response"))?;

        let markdown = data.markdown.unwrap_or_default();

        let content = truncate_content(&markdown, self.config.max_content_length);
        let excerpts = extract_excerpts(&markdown);
        let title = data.metadata.and_then(|m| m.title);

        Ok(EnrichedLink {
            url: url.to_string(),
            title,
            content,
            excerpts,
        })
    }
}

/// Truncate content to max length, trying to break at paragraph boundaries.
fn truncate_content(content: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        return content.to_string();
    }

    // Find a safe character boundary at or before max_length
    let safe_end = content
        .char_indices()
        .take_while(|(i, _)| *i < max_length)
        .last()
        .map_or(0, |(i, c)| i + c.len_utf8());

    // Try to find a paragraph break near the limit
    let truncate_at = content[..safe_end].rfind("\n\n").unwrap_or(safe_end);

    format!("{}...", &content[..truncate_at])
}

/// Extract key excerpts from content (first few paragraphs).
fn extract_excerpts(content: &str) -> Vec<String> {
    content
        .split("\n\n")
        .filter(|p| !p.trim().is_empty() && p.len() > 50)
        .take(3)
        .map(|s| s.trim().to_string())
        .collect()
}
