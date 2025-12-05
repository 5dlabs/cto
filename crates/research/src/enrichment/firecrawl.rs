//! Firecrawl API client for web scraping.

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const FIRECRAWL_API_BASE: &str = "https://api.firecrawl.dev/v1";

/// Options for scraping a URL.
#[derive(Debug, Clone)]
pub struct ScrapeOptions {
    /// Output formats to request.
    pub formats: Vec<String>,
    /// Only extract main content.
    pub only_main_content: bool,
    /// Request timeout.
    pub timeout: Duration,
}

impl Default for ScrapeOptions {
    fn default() -> Self {
        Self {
            formats: vec!["markdown".to_string()],
            only_main_content: true,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Response from Firecrawl scrape API.
#[derive(Debug, Deserialize)]
pub struct ScrapeResponse {
    /// Whether the request was successful.
    pub success: bool,
    /// Scraped data.
    pub data: Option<ScrapeData>,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Scraped page data.
#[derive(Debug, Deserialize)]
pub struct ScrapeData {
    /// Markdown content.
    pub markdown: Option<String>,
    /// HTML content.
    pub html: Option<String>,
    /// Page metadata.
    pub metadata: Option<ScrapeMetadata>,
}

/// Metadata for a scraped page.
#[derive(Debug, Deserialize)]
pub struct ScrapeMetadata {
    /// Page title.
    pub title: Option<String>,
    /// Page description.
    pub description: Option<String>,
    /// Source URL.
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    /// HTTP status code.
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
}

/// Request body for Firecrawl scrape.
#[derive(Debug, Serialize)]
struct ScrapeRequest {
    url: String,
    formats: Vec<String>,
    #[serde(rename = "onlyMainContent")]
    only_main_content: bool,
}

/// Firecrawl API client.
pub struct FirecrawlClient {
    api_key: String,
    client: Client,
}

impl FirecrawlClient {
    /// Create a new Firecrawl client.
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

        Ok(Self { api_key, client })
    }

    /// Create a client from environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key =
            std::env::var("FIRECRAWL_API_KEY").map_err(|_| anyhow!("FIRECRAWL_API_KEY not set"))?;
        Self::new(api_key)
    }

    /// Scrape a single URL.
    pub async fn scrape(&self, url: &str, options: &ScrapeOptions) -> Result<ScrapeResponse> {
        let request = ScrapeRequest {
            url: url.to_string(),
            formats: options.formats.clone(),
            only_main_content: options.only_main_content,
        };

        let response = self
            .client
            .post(format!("{FIRECRAWL_API_BASE}/scrape"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .timeout(options.timeout)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Firecrawl request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            return Err(anyhow!("Firecrawl API error ({status}): {error_text}"));
        }

        response
            .json::<ScrapeResponse>()
            .await
            .map_err(|e| anyhow!("Failed to parse Firecrawl response: {e}"))
    }
}
