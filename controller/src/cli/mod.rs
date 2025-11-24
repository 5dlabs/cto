//! CLI-Agnostic Platform
//!
//! This module provides support for multiple AI CLI tools through a
//! standardized middleware layer that translates between universal
//! configuration and CLI-specific requirements.
//!
//! The new CLI Adapter Trait System provides a unified interface for all CLI providers
//! while preserving their unique capabilities and requirements.

pub mod adapter;
pub mod adapter_factory;
pub mod adapters;
pub mod base_adapter;
pub mod bridge;
pub mod discovery;
pub mod router;
pub mod session;
pub mod types;

// Re-export new trait system components
pub use adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata, StreamingDelta, ToolCall,
};
pub use adapter_factory::{AdapterFactory, FactoryConfig, FactoryStats};
pub use adapters::{ClaudeAdapter, CodexAdapter, CursorAdapter, FactoryAdapter, OpenCodeAdapter};
pub use base_adapter::{AdapterConfig, AdapterMetrics, BaseAdapter};

// Re-export legacy components for backward compatibility
pub use adapter::{CLIExecutionAdapter, CommandBuilder, ResultProcessor};
pub use bridge::{
    CLIAdapter as BridgeCLIAdapter, ConfigurationBridge, JsonCLIAdapter, MarkdownCLIAdapter,
    TomlCLIAdapter,
};
pub use discovery::DiscoveryService;
pub use router::CLIRouter;
pub use session::{SessionManager, SessionState};
pub use types::*;
