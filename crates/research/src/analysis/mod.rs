//! AI-powered content analysis module.
//!
//! Uses LLMs to analyze tweet relevance and categorize content.

mod categories;
mod prompts;
mod relevance;

pub use categories::Category;
pub use prompts::PromptManager;
pub use relevance::{RelevanceAnalyzer, RelevanceResult};
