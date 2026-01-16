//! Embedding-based skill search.

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{SearchMode, Skill};

use super::SkillSearcher;

/// Embedding-based skill searcher.
pub struct EmbeddingSearcher {
    /// API key for embedding service.
    #[allow(dead_code)]
    api_key: String,

    /// Model to use for embeddings.
    #[allow(dead_code)]
    model: String,
}

impl EmbeddingSearcher {
    /// Create a new embedding searcher.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl SkillSearcher for EmbeddingSearcher {
    async fn search(
        &self,
        _query: &str,
        _space_id: Uuid,
        _mode: SearchMode,
        _limit: usize,
    ) -> Result<Vec<Skill>> {
        // TODO: Implement actual embedding search with vector DB
        // 1. Generate embedding for query
        // 2. Search vector DB for similar skills
        // 3. Return top matches
        Ok(Vec::new())
    }
}
