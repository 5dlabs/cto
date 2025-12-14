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
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
// -----------------------------------------------------------------------------
// Type Conversion: Intentional due to API constraints
// -----------------------------------------------------------------------------
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
// -----------------------------------------------------------------------------
// Code Style: Acceptable pedantic style choices
// -----------------------------------------------------------------------------
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::single_match_else)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::match_bool)]
#![allow(clippy::option_if_let_else)]
// -----------------------------------------------------------------------------
// Minor Style: Low-impact style preferences
// -----------------------------------------------------------------------------
#![allow(clippy::items_after_statements)]
#![allow(clippy::needless_continue)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::trivially_copy_pass_by_ref)]

//! # CLI
//!
//! Shared CLI types and adapters for AI coding assistants.
//!
//! This crate provides a unified interface for interacting with multiple CLI-based
//! AI coding tools:
//!
//! - Claude Code CLI (Anthropic)
//! - Codex CLI (OpenAI)
//! - Cursor Agent
//! - OpenCode
//! - Factory Droid
//! - Gemini CLI (Google)
//! - Grok CLI (xAI)
//! - Qwen CLI (Alibaba)
//! - OpenHands
//!
//! ## Key Components
//!
//! - **Types**: Core type definitions (`CLIType`, capabilities, configuration)
//! - **Adapter Trait**: Unified interface for CLI interactions
//! - **Adapters**: Concrete implementations for each CLI
//! - **Factory**: Adapter creation

// Core types
pub mod types;

// Adapter trait and error types
pub mod adapter;

// Concrete adapter implementations
pub mod adapters;

// Base adapter utilities
pub mod base_adapter;

// Adapter factory
pub mod factory;

// Re-export key types for convenience
pub use adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, StreamingDelta, ToolCall, ToolConfiguration,
};
pub use adapters::{
    ClaudeAdapter, CodexAdapter, CursorAdapter, FactoryAdapter, GeminiAdapter, OpenCodeAdapter,
};
pub use base_adapter::{AdapterConfig, BaseAdapter};
pub use factory::AdapterFactory;
pub use types::{
    CLIExecutionContext, CLIExecutionResult, CLISelectionCriteria, CLIType, ConfigFile,
    UniversalConfig,
};
