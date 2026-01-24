//! Link and content enrichment module.
//!
//! Provides multiple enrichment sources:
//! - **Firecrawl**: Web scraping for linked URLs
//! - **OctoCode**: GitHub code search for implementations
//! - **Tavily**: Web research for competitive analysis

mod firecrawl;
mod links;
mod octocode;
mod tavily;

pub use firecrawl::{FirecrawlClient, ScrapeOptions};
pub use links::{EnrichedLink, LinkEnricher};
pub use octocode::{CodeExample, OctoCodeClient, RepoInfo, SearchOptions};
pub use tavily::{ResearchReport, ResearchResult, TavilyClient, TavilySearchOptions};

use serde::{Deserialize, Serialize};

/// Enhanced enrichment result combining all sources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrichedContent {
    /// Content scraped from linked URLs.
    pub link_content: Vec<EnrichedLink>,
    /// Code examples from GitHub.
    pub code_examples: Vec<CodeExample>,
    /// Research report from web search.
    pub research: Option<ResearchReport>,
}

impl EnrichedContent {
    /// Check if any enrichment was successful.
    pub fn has_content(&self) -> bool {
        !self.link_content.is_empty()
            || !self.code_examples.is_empty()
            || self
                .research
                .as_ref()
                .is_some_and(tavily::ResearchReport::has_results)
    }

    /// Get total number of enrichment items.
    pub fn total_items(&self) -> usize {
        self.link_content.len()
            + self.code_examples.len()
            + self
                .research
                .as_ref()
                .map_or(0, tavily::ResearchReport::total_results)
    }
}

/// Configuration for enhanced enrichment.
#[derive(Debug, Clone)]
pub struct EnrichmentConfig {
    /// Enable Firecrawl link scraping.
    pub enable_firecrawl: bool,
    /// Enable OctoCode GitHub search.
    pub enable_octocode: bool,
    /// Maximum code examples to fetch.
    pub max_code_examples: usize,
    /// Enable Tavily web research.
    pub enable_tavily: bool,
    /// Maximum Tavily results per category.
    pub max_tavily_results: u32,
}

impl Default for EnrichmentConfig {
    fn default() -> Self {
        Self {
            enable_firecrawl: true,
            enable_octocode: true,
            max_code_examples: 3,
            enable_tavily: true,
            max_tavily_results: 3,
        }
    }
}

/// Enhanced enrichment pipeline that combines multiple sources.
pub struct EnhancedEnricher {
    link_enricher: Option<LinkEnricher>,
    octocode: Option<OctoCodeClient>,
    tavily: Option<TavilyClient>,
    config: EnrichmentConfig,
}

impl EnhancedEnricher {
    /// Create a new enhanced enricher from environment.
    pub fn from_env(config: EnrichmentConfig) -> Self {
        let link_enricher = if config.enable_firecrawl {
            LinkEnricher::from_env().ok()
        } else {
            None
        };

        let octocode = if config.enable_octocode {
            OctoCodeClient::from_env().ok()
        } else {
            None
        };

        let tavily = if config.enable_tavily {
            TavilyClient::from_env().ok()
        } else {
            None
        };

        Self {
            link_enricher,
            octocode,
            tavily,
            config,
        }
    }

    /// Create with specific clients (for testing).
    pub fn new(
        link_enricher: Option<LinkEnricher>,
        octocode: Option<OctoCodeClient>,
        tavily: Option<TavilyClient>,
        config: EnrichmentConfig,
    ) -> Self {
        Self {
            link_enricher,
            octocode,
            tavily,
            config,
        }
    }

    /// Enrich content with all available sources.
    ///
    /// # Arguments
    /// * `urls` - URLs from the original content to scrape
    /// * `topics` - Topics to search for code examples and research
    /// * `language` - Optional programming language filter for code search
    pub async fn enrich(
        &self,
        urls: &[String],
        topics: &[String],
        language: Option<&str>,
    ) -> EnrichedContent {
        let mut result = EnrichedContent::default();

        // Enrich links with Firecrawl
        if let Some(enricher) = &self.link_enricher {
            if !urls.is_empty() {
                match enricher.enrich_urls(urls).await {
                    Ok(links) => result.link_content = links,
                    Err(e) => tracing::warn!("Firecrawl enrichment failed: {}", e),
                }
            }
        }

        // Search for code examples with OctoCode
        if let Some(octocode) = &self.octocode {
            if !topics.is_empty() {
                let topic = topics.join(" ");
                match octocode.find_implementations(&topic, language).await {
                    Ok(mut examples) => {
                        examples.truncate(self.config.max_code_examples);
                        result.code_examples = examples;
                    }
                    Err(e) => tracing::warn!("OctoCode search failed: {}", e),
                }
            }
        }

        // Research topics with Tavily
        if let Some(tavily) = &self.tavily {
            if !topics.is_empty() {
                let topic = topics.first().map_or("", String::as_str);
                match tavily.research_topic(topic).await {
                    Ok(report) => result.research = Some(report),
                    Err(e) => tracing::warn!("Tavily research failed: {}", e),
                }
            }
        }

        result
    }

    /// Check which enrichment sources are available.
    pub fn available_sources(&self) -> Vec<&'static str> {
        let mut sources = Vec::new();
        if self.link_enricher.is_some() {
            sources.push("firecrawl");
        }
        if self.octocode.is_some() {
            sources.push("octocode");
        }
        if self.tavily.is_some() {
            sources.push("tavily");
        }
        sources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enriched_content_default() {
        let content = EnrichedContent::default();
        assert!(!content.has_content());
        assert_eq!(content.total_items(), 0);
    }

    #[test]
    fn test_enrichment_config_default() {
        let config = EnrichmentConfig::default();
        assert!(config.enable_firecrawl);
        assert!(config.enable_octocode);
        assert!(config.enable_tavily);
        assert_eq!(config.max_code_examples, 3);
    }

    #[test]
    fn test_enhanced_enricher_creation() {
        let config = EnrichmentConfig {
            enable_firecrawl: false,
            enable_octocode: false,
            enable_tavily: false,
            ..Default::default()
        };
        let enricher = EnhancedEnricher::new(None, None, None, config);
        assert!(enricher.available_sources().is_empty());
    }
}
