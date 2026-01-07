//! Task planning domain for intake workflow Session 1.
//!
//! This module handles task management and organization:
//! - Task CRUD operations (manager.rs)
//! - Dependency analysis and execution levels (deps.rs)
//! - Agent routing based on task content (routing.rs)

mod deps;
mod manager;
pub mod routing;

pub use deps::{
    compute_subtask_execution_levels, DependencyDomain, ExecutionLevels, ExecutionStats,
};
pub use manager::TasksDomain;
pub use routing::{
    infer_agent_hint, infer_agent_hint_str, infer_agent_hint_with_deps,
    infer_agent_hint_with_deps_str, is_implementation_agent, parse_agent, Agent,
};
