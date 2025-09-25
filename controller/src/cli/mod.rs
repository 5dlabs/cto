//! CLI-Agnostic Platform
//!
//! This module provides support for multiple AI CLI tools through a
//! standardized middleware layer that translates between universal
//! configuration and CLI-specific requirements.

// Legacy adapter system (maintained for backward compatibility)
pub mod adapter;
pub mod bridge;
pub mod discovery;
pub mod router;
pub mod session;

// New trait-based adapter system
pub mod trait_adapter;
pub mod base_adapter;
pub mod factory;
pub mod adapters;

pub mod types;

// Integration tests
#[cfg(test)]
pub mod integration_tests;

// Re-export legacy system for backward compatibility
pub use adapter::{CLIExecutionAdapter, CommandBuilder, ResultProcessor};
pub use bridge::{
    CLIAdapter as LegacyCLIAdapter, ConfigurationBridge, JsonCLIAdapter, MarkdownCLIAdapter, TomlCLIAdapter,
};
pub use discovery::DiscoveryService;
pub use router::CLIRouter;
pub use session::{SessionManager, SessionState};

// Re-export new trait system
pub use trait_adapter::{CliAdapter, AdapterError, AgentConfig, ParsedResponse, CliCapabilities};
pub use base_adapter::{BaseAdapter, AdapterConfig, AdapterMetrics};
pub use factory::{AdapterFactory, ConfigRegistry, HealthMonitor};
pub use adapters::*;

pub use types::*;
