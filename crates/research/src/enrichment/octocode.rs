//! OctoCode/GitHub code search enrichment.
//!
//! Uses GitHub's code search API to find real implementations
//! of concepts discovered in research entries.

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::time::Duration;

const GITHUB_API_BASE: &str = "https://api.github.com";

/// A code example found via search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Repository full name (owner/repo).
    pub repo: String,
    /// File path within the repository.
    pub path: String,
    /// URL to the file on GitHub.
    pub url: String,
    /// HTML URL for viewing in browser.
    pub html_url: String,
    /// Matched code snippet (may be truncated).
    pub snippet: String,
    /// Repository stars (for ranking).
    pub stars: u32,
    /// Language of the file.
    pub language: Option<String>,
}

/// Options for code search.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Programming language to filter by.
    pub language: Option<String>,
    /// Minimum star count for repositories.
    pub min_stars: u32,
    /// Maximum number of results.
    pub max_results: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            language: None,
            min_stars: 10,
            max_results: 5,
        }
    }
}

/// GitHub code search response.
#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[allow(dead_code)]
    total_count: u64,
    items: Vec<SearchItem>,
}

/// Individual search result item.
#[derive(Debug, Deserialize)]
struct SearchItem {
    #[allow(dead_code)]
    name: String,
    path: String,
    html_url: String,
    url: String,
    repository: SearchRepo,
    #[serde(default)]
    text_matches: Vec<TextMatch>,
}

/// Repository info from search.
#[derive(Debug, Deserialize)]
struct SearchRepo {
    full_name: String,
    stargazers_count: u32,
    language: Option<String>,
}

/// Text match from search.
#[derive(Debug, Deserialize)]
struct TextMatch {
    fragment: String,
}

/// Client for searching GitHub code.
pub struct OctoCodeClient {
    client: Client,
    token: Option<String>,
}

impl OctoCodeClient {
    /// Create a new client with optional GitHub token.
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("CTO-Research/1.0")
                .build()
                .expect("Failed to build HTTP client"),
            token,
        }
    }

    /// Create from environment variable.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").ok();
        Ok(Self::new(token))
    }

    /// Search for code examples matching a query.
    ///
    /// Searches GitHub's code search API for implementations.
    pub async fn search_code(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<CodeExample>> {
        // Build search query
        let mut q = query.to_string();

        if let Some(lang) = &options.language {
            let _ = write!(q, " language:{lang}");
        }

        if options.min_stars > 0 {
            let _ = write!(q, " stars:>={}", options.min_stars);
        }

        let url = format!(
            "{}/search/code?q={}&per_page={}",
            GITHUB_API_BASE,
            urlencoding::encode(&q),
            options.max_results
        );

        let mut req = self.client.get(&url);

        // Add auth token if available
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }

        // Request text match fragments
        req = req.header("Accept", "application/vnd.github.text-match+json");

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }

        let search_result: SearchResponse = response.json().await?;

        let examples: Vec<CodeExample> = search_result
            .items
            .into_iter()
            .map(|item| {
                let snippet = item
                    .text_matches
                    .first()
                    .map_or_else(|| format!("// {}", item.path), |m| m.fragment.clone());

                CodeExample {
                    repo: item.repository.full_name,
                    path: item.path,
                    url: item.url,
                    html_url: item.html_url,
                    snippet,
                    stars: item.repository.stargazers_count,
                    language: item.repository.language,
                }
            })
            .collect();

        Ok(examples)
    }

    /// Search for code implementations of a topic.
    ///
    /// Builds a targeted query for finding real implementations.
    pub async fn find_implementations(
        &self,
        topic: &str,
        lang: Option<&str>,
    ) -> Result<Vec<CodeExample>> {
        let options = SearchOptions {
            language: lang.map(String::from),
            min_stars: 50,
            max_results: 5,
        };

        // Build implementation-focused query
        let query = format!("{topic} impl OR implementation OR example");

        self.search_code(&query, &options).await
    }

    /// Search for repositories related to a topic.
    pub async fn search_repos(&self, query: &str, max_results: usize) -> Result<Vec<RepoInfo>> {
        let url = format!(
            "{}/search/repositories?q={}&sort=stars&order=desc&per_page={}",
            GITHUB_API_BASE,
            urlencoding::encode(query),
            max_results
        );

        let mut req = self.client.get(&url);

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }

        let result: RepoSearchResponse = response.json().await?;

        Ok(result
            .items
            .into_iter()
            .map(|r| RepoInfo {
                full_name: r.full_name,
                description: r.description,
                html_url: r.html_url,
                stars: r.stargazers_count,
                language: r.language,
                topics: r.topics.unwrap_or_default(),
            })
            .collect())
    }
}

/// Repository search response.
#[derive(Debug, Deserialize)]
struct RepoSearchResponse {
    items: Vec<RepoSearchItem>,
}

/// Repository search item.
#[derive(Debug, Deserialize)]
struct RepoSearchItem {
    full_name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
    language: Option<String>,
    topics: Option<Vec<String>>,
}

/// Repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    /// Full repository name (owner/repo).
    pub full_name: String,
    /// Repository description.
    pub description: Option<String>,
    /// URL to repository.
    pub html_url: String,
    /// Star count.
    pub stars: u32,
    /// Primary language.
    pub language: Option<String>,
    /// Repository topics.
    pub topics: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.min_stars, 10);
        assert_eq!(options.max_results, 5);
        assert!(options.language.is_none());
    }

    #[test]
    fn test_client_creation() {
        let client = OctoCodeClient::new(Some("test_token".to_string()));
        assert!(client.token.is_some());

        let client_no_token = OctoCodeClient::new(None);
        assert!(client_no_token.token.is_none());
    }
}
