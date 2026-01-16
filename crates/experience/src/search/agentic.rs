//! Agentic skill search using LLM exploration.

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{SearchMode, Skill};

use super::SkillSearcher;

/// Agentic searcher that uses LLM to explore and find relevant skills.
pub struct AgenticSearcher {
    /// Base searcher for initial retrieval.
    embedding_searcher: Box<dyn SkillSearcher>,
}

impl AgenticSearcher {
    /// Create a new agentic searcher.
    #[must_use]
    pub fn new(embedding_searcher: Box<dyn SkillSearcher>) -> Self {
        Self { embedding_searcher }
    }
}

#[async_trait]
impl SkillSearcher for AgenticSearcher {
    async fn search(
        &self,
        query: &str,
        space_id: Uuid,
        mode: SearchMode,
        limit: usize,
    ) -> Result<Vec<Skill>> {
        // For now, delegate to embedding search
        // Future: Add LLM-powered exploration and refinement
        self.embedding_searcher
            .search(query, space_id, mode, limit)
            .await
    }
}
