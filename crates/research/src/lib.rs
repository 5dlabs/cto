//! Research crate for Twitter/X bookmark monitoring and analysis.
//!
//! This crate provides:
//! - Twitter/X bookmark polling using browser automation
//! - AI-powered relevance analysis using Claude/GPT
//! - Multi-dimensional feature scoring for prioritization
//! - Link enrichment via Firecrawl, OctoCode, and Tavily
//! - Markdown storage with YAML frontmatter
//! - GitHub PR creation for research entries
//! - Platform-aware context for analysis

// Allow pedantic lints that are overly strict for this utility crate
#![allow(clippy::missing_errors_doc)] // Error documentation not critical for research tool
#![allow(clippy::must_use_candidate)] // Return values in this crate are often optional to use
#![allow(clippy::similar_names)] // Variable names like enricher/enriched are fine
#![allow(clippy::too_many_lines)] // Complex async functions are naturally long
#![allow(clippy::doc_markdown)] // Module docs don't need backticks
#![allow(clippy::missing_panics_doc)] // Panics are rare and documented via expect messages
#![allow(clippy::cast_possible_wrap)] // Known safe casts in retry logic
#![allow(clippy::single_match_else)] // Match can be clearer than if-let in some cases
#![allow(clippy::unused_async)] // Async signature needed for interface consistency

pub mod analysis;
pub mod auth;
pub mod context;
pub mod digest;
pub mod enrichment;
pub mod pipeline;
pub mod publish;
pub mod storage;
pub mod tooling;
pub mod twitter;

// Re-export main types
pub use analysis::{
    Category, FeatureScore, InstallableAsset, Priority, RelevanceAnalyzer, RelevanceResult,
};
pub use auth::Session;
pub use context::{AgentProfile, AgentRole, PlatformContext};
pub use digest::{
    ActionItem, DigestAnalysis, DigestAnalyzer, DigestConfig, DigestGenerator, DigestState,
    EmailSender,
};
pub use enrichment::{
    CodeExample, EnhancedEnricher, EnrichedContent, EnrichedLink, EnrichmentConfig, ResearchReport,
    ResearchResult,
};
pub use pipeline::{Pipeline, PipelineConfig, PollCycleResult};
pub use publish::{PublishConfig, Publisher};
pub use storage::ResearchEntry;
pub use tooling::{InstallResult, ToolingClient, ToolingConfig};
pub use twitter::{Author, Bookmark};
