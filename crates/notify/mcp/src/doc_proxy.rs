//! Documentation proxy module for GitHub utilities.
//!
//! Provides helpers for parsing GitHub URLs and scraping README content.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::json;

const FIRECRAWL_API_BASE: &str = "https://api.firecrawl.dev/v2";

/// Response from Firecrawl scrape (single page)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ScrapeResponse {
    pub success: bool,
    pub data: Option<ScrapeData>,
    pub error: Option<String>,
}

/// Single page scrape data
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ScrapeData {
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub metadata: Option<ScrapeMetadata>,
}

/// Metadata for a scraped page
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ScrapeMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
}

/// Firecrawl API client for scraping
struct FirecrawlClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl FirecrawlClient {
    /// Create a new Firecrawl client
    fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Scrape a single page
    fn scrape(&self, url: &str) -> Result<ScrapeResponse> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
