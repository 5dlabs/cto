//! AI integration for task management.
//!
//! This module provides:
//! - AI provider abstraction (Claude Agent SDK via TypeScript binary)
//! - Prompt template system with Handlebars
//! - Structured response schemas
//! - Provider registry for dynamic provider management
//!
//! # Design Philosophy
//!
//! - Single Claude Agent SDK provider with MCP support - no fallbacks
//! - Fail fast to surface problems immediately
//! - Clean JSON protocol over subprocess (no CLI output parsing)

pub mod prompts;
pub mod provider;
pub mod registry;
pub mod schemas;

// Claude Agent SDK provider (TypeScript binary with MCP support)
pub mod sdk_provider;

// Re-exports
pub use prompts::{PromptManager, PromptTemplate};
pub use provider::{
    extract_json_continuation, parse_ai_response, validate_json_continuation, AIMessage,
    AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage,
};
pub use registry::ProviderRegistry;
pub use schemas::*;
pub use sdk_provider::AgentUsage;
// Export both names for compatibility
pub use sdk_provider::{AgentSdkProvider, AnthropicProvider};