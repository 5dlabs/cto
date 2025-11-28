//! AI integration for task management.
//!
//! This module provides:
//! - AI provider abstraction (Anthropic, OpenAI, etc.)
//! - Prompt template system with Handlebars
//! - Structured response schemas
//! - Provider registry for dynamic provider management

pub mod provider;
pub mod registry;
pub mod prompts;
pub mod schemas;

// Provider implementations
pub mod anthropic;
pub mod openai;

// Re-exports
pub use provider::{AIProvider, AIMessage, AIRole, AIResponse, GenerateOptions, TokenUsage, parse_ai_response};
pub use registry::ProviderRegistry;
pub use prompts::{PromptTemplate, PromptManager};
pub use schemas::*;

