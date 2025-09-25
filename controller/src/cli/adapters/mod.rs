//! CLI Adapter Implementations
//!
//! This module contains concrete implementations of the CliAdapter trait
//! for different CLI providers.

pub mod claude;
pub mod codex;
pub mod opencode;
pub mod gemini;
pub mod grok;
pub mod qwen;
pub mod cursor;
pub mod openhands;

// Re-export adapter implementations
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use opencode::OpencodeAdapter;
pub use gemini::GeminiAdapter;
pub use grok::GrokAdapter;
pub use qwen::QwenAdapter;
pub use cursor::CursorAdapter;
pub use openhands::OpenHandsAdapter;