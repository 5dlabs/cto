#![warn(clippy::pedantic)]
// Allow common pedantic lints that don't affect correctness
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::module_inception)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_self)]
#![allow(clippy::if_not_else)]
#![allow(clippy::map_unwrap_or)]

//! # Tasks
//!
//! A Rust-native task management system for AI-driven development workflows.
//!
//! This crate provides:
//! - Task and subtask management with dependencies
//! - Tag-based task organization (like git branches for tasks)
//! - File-based storage in `.tasks/` directory
//! - AI-powered task generation and analysis
//! - CLI and MCP server interfaces
//!
//! ## Example
//!
//! ```rust,ignore
//! use tasks::{TasksDomain, FileStorage};
//!
//! let storage = FileStorage::new(".");
//! let domain = TasksDomain::new(storage);
//!
//! // List all tasks
//! let tasks = domain.list_tasks(None).await?;
//! ```

// Core entities
pub mod entities;

// Error types
pub mod errors;

// Storage layer
pub mod storage;

// Domain facades
pub mod domain;

// Terminal UI helpers
pub mod ui;

// AI integration
pub mod ai;

// Re-export key types for convenience
pub use entities::{
    ComplexityInfo, GlobalConfig, ModelConfig, ModelSettings, RuntimeState, Subtask, TagMetadata,
    TagStats, TaggedTaskList, Task, TaskComplexity, TaskPriority, TaskStatus, TasksConfig,
};
pub use errors::{TasksError, TasksResult};
pub use storage::{FileStorage, Storage};

// Re-export AI types
pub use ai::{
    AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, PromptManager, PromptTemplate,
    ProviderRegistry, TokenUsage,
};
