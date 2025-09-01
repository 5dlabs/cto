//! CLI-Agnostic Platform
//!
//! This module provides support for multiple AI CLI tools through a
//! standardized middleware layer that translates between universal
//! configuration and CLI-specific requirements.

pub mod adapter;
pub mod bridge;
pub mod discovery;
pub mod router;
pub mod session;
pub mod types;

// Re-export commonly used types and structs
pub use adapter::{CLIExecutionAdapter, CommandBuilder, ResultProcessor};
pub use bridge::{
    CLIAdapter, ConfigurationBridge, JsonCLIAdapter, MarkdownCLIAdapter, TomlCLIAdapter,
};
pub use discovery::DiscoveryService;
pub use router::CLIRouter;
pub use session::{SessionManager, SessionState};
pub use types::*;
