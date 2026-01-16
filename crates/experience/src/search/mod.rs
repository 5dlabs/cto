//! Skill search functionality.
//!
//! Provides both fast embedding-based search and agentic exploration.

mod agentic;
mod embedding;

pub use agentic::AgenticSearcher;
pub use embedding::EmbeddingSearcher;

use crate::models::{SearchMode, Skill};
use anyhow::Result;
use uuid::Uuid;

/// Skill searcher interface.
#[async_trait::async_trait]
pub trait SkillSearcher: Send + Sync {
    /// Search for skills matching a query.
    async fn search(
        &self,
        query: &str,
        space_id: Uuid,
        mode: SearchMode,
        limit: usize,
    ) -> Result<Vec<Skill>>;
}
