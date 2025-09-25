//! CLI Adapter Implementations
//!
//! Concrete implementations of the `CliAdapter` trait for different CLI tools.

pub mod claude;

// Re-export adapter implementations
pub use claude::ClaudeAdapter;

// Stub modules for future implementations
pub mod codex {
    //! Codex CLI adapter (placeholder for Task 4)
    // TODO: Implement CodexAdapter in Task 4
}

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

pub mod cursor {
    //! Cursor CLI adapter (placeholder for future task)
    // TODO: Implement CursorAdapter
}

pub mod openhands {
    //! OpenHands CLI adapter (placeholder for future task)
    // TODO: Implement OpenHandsAdapter
}
