//! Adapter Factory
//!
//! Factory for creating CLI adapters based on CLI type.

use crate::adapter::{AdapterError, AdapterResult, CliAdapter};
use crate::adapters::{
    ClaudeAdapter, CodexAdapter, CursorAdapter, FactoryAdapter, GeminiAdapter, OpenCodeAdapter,
};
use crate::base_adapter::AdapterConfig;
use crate::types::CLIType;
use std::sync::Arc;
use tracing::{debug, info};

/// Factory for creating CLI adapters
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create an adapter for the specified CLI type
    pub fn create(cli_type: CLIType) -> AdapterResult<Arc<dyn CliAdapter>> {
        Self::create_with_config(cli_type, AdapterConfig::new(cli_type))
    }

    /// Create an adapter with custom configuration
    pub fn create_with_config(
        cli_type: CLIType,
        config: AdapterConfig,
    ) -> AdapterResult<Arc<dyn CliAdapter>> {
        info!(cli_type = %cli_type, "Creating adapter");

        let adapter: Arc<dyn CliAdapter> = match cli_type {
            CLIType::Claude => Arc::new(ClaudeAdapter::with_config(config)?),
            CLIType::Codex => Arc::new(CodexAdapter::with_config(config)?),
            CLIType::Cursor => Arc::new(CursorAdapter::with_config(config)?),
            CLIType::Factory => Arc::new(FactoryAdapter::with_config(config)?),
            CLIType::Gemini => Arc::new(GeminiAdapter::with_config(config)?),
            CLIType::OpenCode => Arc::new(OpenCodeAdapter::with_config(config)?),
            CLIType::OpenHands | CLIType::Grok | CLIType::Qwen => {
                return Err(AdapterError::UnsupportedCliType(format!(
                    "{cli_type} adapter not yet implemented"
                )));
            }
        };

        debug!(cli_type = %cli_type, "Adapter created successfully");
        Ok(adapter)
    }

    /// Get all supported CLI types
    pub fn supported_types() -> Vec<CLIType> {
        vec![
            CLIType::Claude,
            CLIType::Codex,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
            CLIType::OpenCode,
        ]
    }

    /// Check if a CLI type is supported
    pub fn is_supported(cli_type: CLIType) -> bool {
        Self::supported_types().contains(&cli_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_claude_adapter() {
        let adapter = AdapterFactory::create(CLIType::Claude);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_create_codex_adapter() {
        let adapter = AdapterFactory::create(CLIType::Codex);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_supported_types() {
        let types = AdapterFactory::supported_types();
        assert!(types.contains(&CLIType::Claude));
        assert!(types.contains(&CLIType::Codex));
    }

    #[test]
    fn test_unsupported_type() {
        let adapter = AdapterFactory::create(CLIType::OpenHands);
        assert!(adapter.is_err());
    }
}
