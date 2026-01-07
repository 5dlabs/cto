//! Domain facades for intake workflow.
//!
//! These facades provide high-level operations that combine
//! storage operations with business logic.
//!
//! ## Two-Session Architecture
//!
//! - **Session 1 (Task Planning)**: Uses `IntakeDomain` to parse PRD and generate tasks.json
//!   - `tasks/` module: Task CRUD, dependency analysis, agent routing
//! - **Session 2 (Prompt Generation)**: Uses `prompts::PromptGenerator` to create per-task prompts
//!   - `prompts/` module: AI-based prompt generation, templates

mod ai;
mod config;
pub mod cto_config;
pub mod intake;
pub mod prompts;
pub mod tasks;

// Re-export from tasks module
pub use tasks::{
    compute_subtask_execution_levels, infer_agent_hint, infer_agent_hint_str,
    infer_agent_hint_with_deps, infer_agent_hint_with_deps_str, is_implementation_agent,
    parse_agent, Agent, DependencyDomain, ExecutionLevels, ExecutionStats, TasksDomain,
};

// Re-export from prompts module (including docs for backward compatibility)
pub use prompts::{
    generate_all_docs, split_tasks, GeneratePromptsConfig, PromptFiles, PromptGenerator,
    PromptGeneratorResult, SplitTasksResult,
};
// Backward compatibility: expose templates as docs
pub mod docs {
    pub use super::prompts::templates::*;
}

pub use ai::AIDomain;
pub use config::ConfigDomain;
pub use cto_config::{generate_cto_config, save_cto_config, CtoConfig};
pub use intake::{IntakeConfig, IntakeDomain, IntakeResult};
