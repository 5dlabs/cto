//! Documentation proxy module for ingesting docs via Firecrawl API v2.
//!
//! Provides a unified interface for crawling GitHub repositories and websites.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const FIRECRAWL_API_BASE: &str = "https://api.firecrawl.dev/v2";

/// Type of documentation source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocType {
    /// GitHub repository documentation
    Repo,
    /// Website documentation (scrape)
    Scrape,
}

impl DocType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "repo" => Ok(Self::Repo),
            "scrape" => Ok(Self::Scrape),
            _ => Err(anyhow!(
                "Invalid doc type: '{s}'. Must be 'repo' or 'scrape'"
            )),
        }
    }
}

/// Response from Firecrawl crawl initiation
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CrawlResponse {
    pub success: bool,
    pub id: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
}

/// Response from Firecrawl crawl status check
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CrawlStatusResponse {
    pub success: bool,
    pub status: String,
    pub total: Option<u32>,
    pub completed: Option<u32>,
    #[serde(rename = "creditsUsed")]
    pub credits_used: Option<u32>,
    pub data: Option<Vec<CrawlData>>,
    pub error: Option<String>,
}

/// Individual crawled page data
#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct CrawlData {
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub metadata: Option<CrawlMetadata>,
}

/// Metadata for a crawled page
#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct CrawlMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
}

/// Response from Firecrawl scrape (single page)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ScrapeResponse {
    pub success: bool,
    pub data: Option<ScrapeData>,
    pub error: Option<String>,
}

/// Single page scrape data
#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct ScrapeData {
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub metadata: Option<CrawlMetadata>,
}

/// Firecrawl API client
pub struct FirecrawlClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl FirecrawlClient {
    /// Create a new Firecrawl client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Start a crawl job for a URL
    pub fn crawl(&self, url: &str, limit: u32, query: Option<&str>) -> Result<CrawlResponse> {
        let mut body = json!({
            "url": url,
            "limit": limit,
            "scrapeOptions": {
                "formats": ["markdown"],
                "onlyMainContent": true
            }
        });

        // Add search filter if query provided
        if let Some(q) = query {
            body["includePaths"] = json!([format!("*{q}*")]);
        }

        let api_key = &self.api_key;
        let resp = self
            .client
            .post(format!("{FIRECRAWL_API_BASE}/crawl"))
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| anyhow!("Firecrawl crawl request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Firecrawl API error ({status}): {error_text}"));
        }

        resp.json::<CrawlResponse>()
            .map_err(|e| anyhow!("Failed to parse Firecrawl response: {e}"))
    }

    /// Check the status of a crawl job
    #[allow(dead_code)]
    pub fn check_crawl_status(&self, crawl_id: &str) -> Result<CrawlStatusResponse> {
        let api_key = &self.api_key;
        let resp = self
            .client
            .get(format!("{FIRECRAWL_API_BASE}/crawl/{crawl_id}"))
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .map_err(|e| anyhow!("Firecrawl status request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Firecrawl API error ({status}): {error_text}"));
        }

        resp.json::<CrawlStatusResponse>()
            .map_err(|e| anyhow!("Failed to parse Firecrawl status response: {e}"))
    }

    /// Scrape a single page
    #[allow(dead_code)]
    pub fn scrape(&self, url: &str) -> Result<ScrapeResponse> {
        let body = json!({
            "url": url,
            "formats": ["markdown"],
            "onlyMainContent": true
        });

        let api_key = &self.api_key;
        let resp = self
            .client
            .post(format!("{FIRECRAWL_API_BASE}/scrape"))
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| anyhow!("Firecrawl scrape request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Firecrawl API error ({status}): {error_text}"));
        }

        resp.json::<ScrapeResponse>()
            .map_err(|e| anyhow!("Failed to parse Firecrawl scrape response: {e}"))
    }
}

/// Process URL for repo type - extract docs path if needed
pub fn process_repo_url(url: &str) -> String {
    // If it's a GitHub URL without /tree/ path, try to construct docs URL
    if url.contains("github.com") && !url.contains("/tree/") && !url.contains("/blob/") {
        // Try common docs paths: /tree/main/docs, /tree/master/docs
        format!("{}/tree/main/docs", url.trim_end_matches('/'))
    } else {
        url.to_string()
    }
}

/// Handle the `add_docs` tool call
pub fn handle_add_docs(
    url: &str,
    doc_type: DocType,
    query: Option<&str>,
    limit: u32,
) -> Result<Value> {
    let api_key = std::env::var("FIRECRAWL_API_KEY")
        .map_err(|_| anyhow!("FIRECRAWL_API_KEY environment variable not set"))?;

    let client = FirecrawlClient::new(api_key);

    // Process URL based on type
    let target_url = match doc_type {
        DocType::Repo => process_repo_url(url),
        DocType::Scrape => url.to_string(),
    };

    // Start the crawl
    let crawl_resp = client.crawl(&target_url, limit, query)?;

    if !crawl_resp.success {
        return Err(anyhow!(
            "Firecrawl crawl failed: {}",
            crawl_resp
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    let crawl_id = crawl_resp
        .id
        .ok_or_else(|| anyhow!("No crawl ID returned from Firecrawl"))?;

    // Return immediately with the crawl ID - crawling is async
    Ok(json!({
        "success": true,
        "message": "Crawl job started successfully",
        "crawl_id": crawl_id,
        "status_url": format!("{}/crawl/{}", FIRECRAWL_API_BASE, crawl_id),
        "target_url": target_url,
        "type": match doc_type {
            DocType::Repo => "repo",
            DocType::Scrape => "scrape",
        },
        "limit": limit,
        "note": "Use the crawl_id to check status. Crawling runs asynchronously and results expire after 24 hours."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_type_from_str() {
        assert_eq!(DocType::from_str("repo").unwrap(), DocType::Repo);
        assert_eq!(DocType::from_str("scrape").unwrap(), DocType::Scrape);
        assert_eq!(DocType::from_str("REPO").unwrap(), DocType::Repo);
        assert!(DocType::from_str("invalid").is_err());
    }

    #[test]
    fn test_process_repo_url() {
        assert_eq!(
            process_repo_url("https://github.com/vercel/next.js"),
            "https://github.com/vercel/next.js/tree/main/docs"
        );
        assert_eq!(
            process_repo_url("https://github.com/vercel/next.js/tree/canary/docs"),
            "https://github.com/vercel/next.js/tree/canary/docs"
        );
    }
}
