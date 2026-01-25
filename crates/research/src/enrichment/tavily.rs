//! Tavily web search enrichment.
//!
//! Uses Tavily's AI-powered search API to find competitive analysis,
//! industry trends, and technical documentation.

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::time::Duration;

const TAVILY_API_BASE: &str = "https://api.tavily.com";

/// A web research result from Tavily.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    /// Title of the page.
    pub title: String,
    /// URL of the source.
    pub url: String,
    /// Extracted content/snippet.
    pub content: String,
    /// Relevance score (0.0-1.0).
    pub score: f32,
    /// Published date if available.
    pub published_date: Option<String>,
}

/// Search options for Tavily.
#[derive(Debug, Clone)]
pub struct TavilySearchOptions {
    /// Search depth: "basic" or "advanced".
    pub search_depth: String,
    /// Include raw content in response.
    pub include_raw_content: bool,
    /// Include images in results.
    pub include_images: bool,
    /// Maximum number of results.
    pub max_results: u32,
    /// Include domains to search.
    pub include_domains: Vec<String>,
    /// Exclude domains from search.
    pub exclude_domains: Vec<String>,
}

impl Default for TavilySearchOptions {
    fn default() -> Self {
        Self {
            search_depth: "basic".to_string(),
            include_raw_content: false,
            include_images: false,
            max_results: 5,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
        }
    }
}

/// Tavily search request body.
#[derive(Debug, Serialize)]
struct SearchRequest {
    api_key: String,
    query: String,
    search_depth: String,
    include_raw_content: bool,
    include_images: bool,
    max_results: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    include_domains: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exclude_domains: Vec<String>,
}

/// Tavily search response.
#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<TavilyResult>,
    #[allow(dead_code)]
    #[serde(default)]
    answer: Option<String>,
}

/// Individual Tavily result.
#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f32,
    #[serde(default)]
    published_date: Option<String>,
}

/// Client for Tavily web search.
pub struct TavilyClient {
    client: Client,
    api_key: String,
}

impl TavilyClient {
    /// Create a new Tavily client.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client"),
            api_key,
        }
    }

    /// Create from environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("TAVILY_API_KEY")
            .map_err(|_| anyhow!("TAVILY_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key))
    }

    /// Check if the client is configured.
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Search the web for a query.
    pub async fn search(
        &self,
        query: &str,
        options: &TavilySearchOptions,
    ) -> Result<Vec<ResearchResult>> {
        let request = SearchRequest {
            api_key: self.api_key.clone(),
            query: query.to_string(),
            search_depth: options.search_depth.clone(),
            include_raw_content: options.include_raw_content,
            include_images: options.include_images,
            max_results: options.max_results,
            include_domains: options.include_domains.clone(),
            exclude_domains: options.exclude_domains.clone(),
        };

        let response = self
            .client
            .post(format!("{TAVILY_API_BASE}/search"))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Tavily API error {status}: {body}"));
        }

        let search_result: SearchResponse = response.json().await?;

        Ok(search_result
            .results
            .into_iter()
            .map(|r| ResearchResult {
                title: r.title,
                url: r.url,
                content: r.content,
                score: r.score,
                published_date: r.published_date,
            })
            .collect())
    }

    /// Research a topic for competitive analysis.
    pub async fn research_topic(&self, topic: &str) -> Result<ResearchReport> {
        let general_query = format!("{topic} best practices implementation");
        let general_results = self
            .search(
                &general_query,
                &TavilySearchOptions {
                    search_depth: "advanced".to_string(),
                    max_results: 3,
                    ..Default::default()
                },
            )
            .await?;

        let competitive_query = format!("{topic} alternatives comparison");
        let competitive_results = self
            .search(
                &competitive_query,
                &TavilySearchOptions {
                    max_results: 3,
                    ..Default::default()
                },
            )
            .await?;

        let trends_query = format!("{topic} 2026 trends updates");
        let trends_results = self
            .search(
                &trends_query,
                &TavilySearchOptions {
                    max_results: 2,
                    ..Default::default()
                },
            )
            .await?;

        Ok(ResearchReport {
            topic: topic.to_string(),
            general_info: general_results,
            competitive_analysis: competitive_results,
            trends: trends_results,
        })
    }

    /// Search technical documentation.
    pub async fn search_docs(
        &self,
        topic: &str,
        include_domains: Vec<String>,
    ) -> Result<Vec<ResearchResult>> {
        let options = TavilySearchOptions {
            search_depth: "advanced".to_string(),
            max_results: 5,
            include_domains,
            ..Default::default()
        };

        let query = format!("{topic} documentation guide tutorial");
        self.search(&query, &options).await
    }
}

/// Comprehensive research report on a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    /// The researched topic.
    pub topic: String,
    /// General information and best practices.
    pub general_info: Vec<ResearchResult>,
    /// Competitive analysis and alternatives.
    pub competitive_analysis: Vec<ResearchResult>,
    /// Recent trends and updates.
    pub trends: Vec<ResearchResult>,
}

impl ResearchReport {
    /// Get total number of results.
    pub fn total_results(&self) -> usize {
        self.general_info.len() + self.competitive_analysis.len() + self.trends.len()
    }

    /// Check if the report has any results.
    pub fn has_results(&self) -> bool {
        self.total_results() > 0
    }

    /// Get all results flattened into a single list.
    pub fn all_results(&self) -> Vec<&ResearchResult> {
        let mut all = Vec::new();
        all.extend(self.general_info.iter());
        all.extend(self.competitive_analysis.iter());
        all.extend(self.trends.iter());
        all
    }

    /// Format the report as markdown for inclusion in research entries.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        if !self.general_info.is_empty() {
            md.push_str("### Best Practices & Documentation\n\n");
            for result in &self.general_info {
                let _ = writeln!(
                    md,
                    "- **[{}]({})**: {}",
                    result.title,
                    result.url,
                    truncate(&result.content, 150)
                );
            }
            md.push('\n');
        }

        if !self.competitive_analysis.is_empty() {
            md.push_str("### Competitive Landscape\n\n");
            for result in &self.competitive_analysis {
                let _ = writeln!(
                    md,
                    "- **[{}]({})**: {}",
                    result.title,
                    result.url,
                    truncate(&result.content, 150)
                );
            }
            md.push('\n');
        }

        if !self.trends.is_empty() {
            md.push_str("### Recent Trends\n\n");
            for result in &self.trends {
                let _ = writeln!(
                    md,
                    "- **[{}]({})**: {}",
                    result.title,
                    result.url,
                    truncate(&result.content, 150)
                );
            }
        }

        md
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = TavilySearchOptions::default();
        assert_eq!(options.search_depth, "basic");
        assert_eq!(options.max_results, 5);
        assert!(!options.include_raw_content);
    }

    #[test]
    fn test_research_report_total() {
        let report = ResearchReport {
            topic: "test".to_string(),
            general_info: vec![ResearchResult {
                title: "Test".to_string(),
                url: "https://test.com".to_string(),
                content: "Content".to_string(),
                score: 0.9,
                published_date: None,
            }],
            competitive_analysis: vec![],
            trends: vec![],
        };
        assert_eq!(report.total_results(), 1);
        assert!(report.has_results());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is a ...");
    }
}
