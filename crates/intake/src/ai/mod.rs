//! AI integration for task management.
//!
//! This module provides:
//! - AI provider abstraction (Anthropic, OpenAI, etc.)
//! - Prompt template system with Handlebars
//! - Structured response schemas
//! - Provider registry for dynamic provider management

pub mod prompts;
pub mod provider;
pub mod registry;
pub mod schemas;

// Provider implementations
pub mod anthropic;
pub mod cli_provider;
pub mod openai;

// Re-exports
pub use prompts::{PromptManager, PromptTemplate};
pub use provider::{
    parse_ai_response, AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage,
};
pub use registry::ProviderRegistry;
pub use schemas::*;
