//! CLI Adapter Implementations
//!
//! Concrete implementations of the `CliAdapter` trait for different CLI tools.

pub mod claude;
pub mod codex;
pub mod cursor;
pub mod factory;
pub mod gemini;
pub mod opencode;

// Re-export adapter implementations
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use cursor::CursorAdapter;
pub use factory::FactoryAdapter;
pub use gemini::GeminiAdapter;
pub use opencode::OpenCodeAdapter;

