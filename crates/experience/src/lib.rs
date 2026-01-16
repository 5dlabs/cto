//! Agent Experience Learning System
//!
//! This crate provides self-learning capabilities for CTO agents, enabling them to:
//! - Learn skills (SOPs) from successful Play workflows
//! - Apply context editing strategies to manage token limits
//! - Search and retrieve learned skills for agent guidance
//!
//! Inspired by Acontext's "Context Data Platform" approach, adapted for CTO's
//! Kubernetes-native architecture.
//!
//! # Core Concepts
//!
//! - **Skill**: A learned Standard Operating Procedure (SOP) extracted from successful
//!   task completions, including tool-call patterns and user preferences.
//!
//! - **Space**: A scope for skills (per project or user) that isolates learned patterns.
//!
//! - **Session**: A tracked workflow execution that may contribute to learning.
//!
//! - **Context Editing**: Declarative strategies for managing LLM context windows.
//!
//! # Example
//!
//! ```rust,ignore
//! use experience::{ExperienceClient, SearchMode};
//!
//! // Search for relevant skills
//! let skills = client.search_skills(
//!     "implementing Rust HTTP handler with authentication",
//!     space_id,
//!     SearchMode::Fast,
//! ).await?;
//!
//! // Apply context editing
//! let edited = client.edit_context(
//!     session_id,
//!     vec![
//!         EditStrategy::TokenLimit { limit: 20000 },
//!         EditStrategy::RemoveToolResult { keep_recent: 3 },
//!     ],
//! ).await?;
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod editing;
pub mod learning;
pub mod models;
pub mod search;
#[cfg(feature = "postgres")]
pub mod storage;
pub mod tools;

// Re-export primary types
pub use editing::{EditParams, EditResult, EditStrategy};
pub use learning::{ComplexityFilter, SessionCollector, SkillLearner, TaskExtractor};
pub use models::{
    AgentType, MessageRecord, SearchMode, SessionRecord, SessionStatus, Skill, Space, TaskRecord,
    TaskStatus, ToolCallRecord, ToolStep,
};
pub use search::SkillSearcher;

/// Experience client configuration.
#[derive(Debug, Clone)]
pub struct ExperienceConfig {
    /// Database URL for `PostgreSQL` storage.
    #[cfg(feature = "postgres")]
    pub database_url: String,

    /// Model to use for generating embeddings.
    pub embedding_model: String,

    /// API key for embedding service.
    pub embedding_api_key: String,

    /// Minimum complexity score for learning (0.0 - 1.0).
    pub complexity_threshold: f32,

    /// Minimum number of tool calls for a task to be learnable.
    pub min_tool_calls: usize,

    /// Minimum duration in seconds for a task to be learnable.
    pub min_duration_secs: u64,
}

impl Default for ExperienceConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "postgres")]
            database_url: String::new(),
            embedding_model: "text-embedding-3-small".to_string(),
            embedding_api_key: String::new(),
            complexity_threshold: 0.5,
            min_tool_calls: 3,
            min_duration_secs: 300, // 5 minutes
        }
    }
}
