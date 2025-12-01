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

/// Extract org and repo name from a GitHub URL
/// Returns (org, repo) tuple or error if URL is invalid
pub fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Handle URLs like:
    // https://github.com/org/repo
    // https://github.com/org/repo.git
    // https://github.com/org/repo/tree/main/...
    let url = url.trim_end_matches('/').trim_end_matches(".git");

    let parts: Vec<&str> = url.split('/').collect();

    // Find github.com in the path and get the next two parts
    let github_idx = parts.iter().position(|&p| p == "github.com");
    match github_idx {
        Some(idx) if parts.len() > idx + 2 => {
            let org = parts[idx + 1].to_string();
            let repo = parts[idx + 2].to_string();
            if org.is_empty() || repo.is_empty() {
                return Err(anyhow!(
                    "Invalid GitHub URL: could not extract org/repo from '{url}'"
                ));
            }
            Ok((org, repo))
        }
        _ => Err(anyhow!(
            "Invalid GitHub URL format: '{url}'. Expected https://github.com/org/repo"
        )),
    }
}

/// Derive a server key from a GitHub repository name
/// Converts repo name to lowercase kebab-case suitable for use as a server key
pub fn derive_server_key(repo_name: &str) -> String {
    // Remove common prefixes/suffixes
    let key = repo_name
        .trim_start_matches("mcp-")
        .trim_start_matches("server-")
        .trim_end_matches("-mcp")
        .trim_end_matches("-server");

    // Convert to lowercase and replace underscores with hyphens
    key.to_lowercase().replace('_', "-")
}

/// Scrape README content from a GitHub repository
/// Returns the README markdown content
pub fn scrape_readme(github_url: &str) -> Result<String> {
    let api_key = std::env::var("FIRECRAWL_API_KEY")
        .map_err(|_| anyhow!("FIRECRAWL_API_KEY environment variable not set"))?;

    let client = FirecrawlClient::new(api_key);

    // Scrape the main GitHub repo page which includes the README
    let clean_url = github_url.trim_end_matches('/').trim_end_matches(".git");
    let scrape_resp = client.scrape(clean_url)?;

    if !scrape_resp.success {
        return Err(anyhow!(
            "Failed to scrape README: {}",
            scrape_resp
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    let data = scrape_resp
        .data
        .ok_or_else(|| anyhow!("No data returned from Firecrawl scrape"))?;

    data.markdown
        .ok_or_else(|| anyhow!("No markdown content in scraped README"))
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

    #[test]
    fn test_parse_github_url() {
        // Standard URL
        let (org, repo) = parse_github_url("https://github.com/anthropics/github-mcp").unwrap();
        assert_eq!(org, "anthropics");
        assert_eq!(repo, "github-mcp");

        // URL with trailing slash
        let (org, repo) =
            parse_github_url("https://github.com/modelcontextprotocol/server-slack/").unwrap();
        assert_eq!(org, "modelcontextprotocol");
        assert_eq!(repo, "server-slack");

        // URL with .git suffix
        let (org, repo) = parse_github_url("https://github.com/org/repo.git").unwrap();
        assert_eq!(org, "org");
        assert_eq!(repo, "repo");

        // URL with tree path
        let (org, repo) = parse_github_url("https://github.com/org/repo/tree/main/docs").unwrap();
        assert_eq!(org, "org");
        assert_eq!(repo, "repo");

        // Invalid URL
        assert!(parse_github_url("https://gitlab.com/org/repo").is_err());
        assert!(parse_github_url("not-a-url").is_err());
    }

    #[test]
    fn test_derive_server_key() {
        // Remove mcp- prefix
        assert_eq!(derive_server_key("mcp-slack"), "slack");

        // Remove server- prefix
        assert_eq!(derive_server_key("server-github"), "github");

        // Remove -mcp suffix
        assert_eq!(derive_server_key("brave-search-mcp"), "brave-search");

        // Remove -server suffix
        assert_eq!(derive_server_key("kubernetes-server"), "kubernetes");

        // Multiple transformations
        assert_eq!(derive_server_key("mcp-brave-search-server"), "brave-search");

        // No transformation needed
        assert_eq!(derive_server_key("slack"), "slack");

        // Underscores to hyphens
        assert_eq!(derive_server_key("brave_search"), "brave-search");

        // Uppercase to lowercase
        assert_eq!(derive_server_key("GitHub"), "github");
    }
}
