//! CLI Adapter Implementations
//!
//! Concrete implementations of the `CliAdapter` trait for different CLI tools.

pub mod claude;
pub mod codex;
pub mod cursor;

// Re-export adapter implementations
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use cursor::CursorAdapter;

pub mod opencode {
    //! OpenCode CLI adapter (placeholder for future task)
    // TODO: Implement OpencodeAdapter
}

pub mod gemini {
    //! Gemini CLI adapter (placeholder for future task)
    // TODO: Implement GeminiAdapter
}

pub mod grok {
    //! Grok CLI adapter (placeholder for future task)
    // TODO: Implement GrokAdapter
}

pub mod qwen {
    //! Qwen CLI adapter (placeholder for future task)
    // TODO: Implement QwenAdapter
}

pub mod openhands {
    //! OpenHands CLI adapter (placeholder for future task)
    // TODO: Implement OpenHandsAdapter
}
