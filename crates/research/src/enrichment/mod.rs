//! Link enrichment module using Firecrawl.
//!
//! Scrapes linked URLs to extract additional context for research items.

mod firecrawl;
mod links;

pub use firecrawl::{FirecrawlClient, ScrapeOptions};
pub use links::{EnrichedLink, LinkEnricher};
