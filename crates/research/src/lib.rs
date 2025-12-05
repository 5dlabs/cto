//! Research crate for Twitter/X bookmark monitoring and analysis.
//!
//! This crate provides:
//! - Twitter/X bookmark polling using browser automation
//! - AI-powered relevance analysis using Claude/GPT
//! - Link enrichment via Firecrawl
//! - Markdown storage with YAML frontmatter

pub mod analysis;
pub mod auth;
pub mod enrichment;
pub mod pipeline;
pub mod storage;
pub mod twitter;

// Re-export main types
pub use analysis::{Category, RelevanceResult};
pub use auth::Session;
pub use enrichment::EnrichedLink;
pub use pipeline::{Pipeline, PipelineConfig, PollCycleResult};
pub use storage::ResearchEntry;
pub use twitter::{Author, Bookmark};
