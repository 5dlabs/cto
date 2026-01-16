//! Data models for the experience learning system.

mod session;
mod skill;
mod space;
mod task;

pub use session::{MessageRecord, SessionRecord, SessionStatus};
pub use skill::{AgentType, Skill, ToolStep};
pub use space::Space;
pub use task::{TaskRecord, TaskStatus, ToolCallRecord};

/// Search mode for skill retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    /// Fast embedding-based similarity search.
    #[default]
    Fast,
    /// LLM-powered exploration for comprehensive coverage.
    Agentic,
}
