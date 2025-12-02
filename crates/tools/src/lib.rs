#![warn(clippy::pedantic)]
// =============================================================================
// Clippy Pedantic Lint Configuration
// =============================================================================
// This crate enables clippy::pedantic for high code quality. The allows below
// are intentional choices documented by category.

// -----------------------------------------------------------------------------
// Documentation: To be addressed in a separate documentation PR
// -----------------------------------------------------------------------------
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
// -----------------------------------------------------------------------------
// API Design Choices: Intentional for consistency and future extensibility
// -----------------------------------------------------------------------------
// Methods take &self for consistency even when not currently needed
#![allow(clippy::unused_self)]
// Functions return Result for consistent error handling patterns
#![allow(clippy::unnecessary_wraps)]
// Parameters taken by value for API flexibility
#![allow(clippy::needless_pass_by_value)]
// Not all functions need #[must_use]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
// -----------------------------------------------------------------------------
// Code Style: Acceptable pedantic style choices
// -----------------------------------------------------------------------------
// Allow similar names when context makes them clear
#![allow(clippy::similar_names)]
// Allow longer functions for complex logic that shouldn't be split
#![allow(clippy::too_many_lines)]
// Allow module_name in type names for clarity in public API
#![allow(clippy::module_name_repetitions)]
// Allow struct fields named after their type
#![allow(clippy::struct_field_names)]
// Allow wildcard imports in specific contexts
#![allow(clippy::wildcard_imports)]
// Single match arms are sometimes clearer than if-let
#![allow(clippy::single_match_else)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::match_same_arms)]
// Allow match on bool for clarity in complex conditions
#![allow(clippy::match_bool)]
// Allow option_if_let_else for readability in some cases
#![allow(clippy::option_if_let_else)]
// -----------------------------------------------------------------------------
// Cast Safety: Casts are intentional and checked at runtime where needed
// -----------------------------------------------------------------------------
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
// -----------------------------------------------------------------------------
// Minor Style: Low-impact style preferences
// -----------------------------------------------------------------------------
#![allow(clippy::items_after_statements)]
#![allow(clippy::needless_continue)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::ref_option)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::assigning_clones)]
// -----------------------------------------------------------------------------
// Disallowed Items: Configuration-based restrictions (clippy.toml)
// -----------------------------------------------------------------------------
// SystemTime::now usage - acceptable for now, Clock abstraction planned
#![allow(clippy::disallowed_methods)]
// println/eprintln converted to tracing throughout codebase
#![allow(clippy::disallowed_macros)]

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
