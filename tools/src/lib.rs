#![warn(clippy::pedantic)]
// Allow common pedantic lints that don't affect correctness
// TODO: Gradually address these during future refactoring
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::ref_option)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::unused_self)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::disallowed_methods)] // SystemTime::now - will refactor later
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::if_not_else)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::match_bool)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_else)]
#![allow(clippy::needless_continue)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::disallowed_macros)] // println/eprintln - converted to tracing

use std::path::PathBuf;

// Re-export the MCP client module
pub mod client;

// Comprehensive error handling system
pub mod errors;

// Server health monitoring and recovery
pub mod health_monitor;

// Comprehensive server recovery system
pub mod recovery;

// Context-based user configuration management
pub mod context;

// Configuration management
pub mod config;

// Re-export key types for convenience
pub use client::McpClient;
pub use config::{ClientInfo, ServerConfig, SystemConfigManager};
pub use context::{ContextConfig, ContextManager};

/// Helper function to resolve working directory patterns
#[must_use]
pub fn resolve_working_directory(working_dir: &str, project_dir: &std::path::Path) -> PathBuf {
    match working_dir {
        "project_root" | "project" => project_dir.to_path_buf(),
        path if path.starts_with('/') => PathBuf::from(path), // Absolute path
        path => project_dir.join(path),                       // Relative to project directory
    }
}
