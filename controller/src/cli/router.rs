//! CLI Router
//!
//! Handles CLI selection, fallback logic, and execution context preparation.
//! This component decides which CLI to use and prepares the execution environment.

use crate::cli::bridge::ConfigurationBridge;
use crate::cli::discovery::DiscoveryService;
use crate::cli::types::{CLIExecutionContext, CLIProfile, CLIType, SessionType, UniversalConfig};
use std::collections::HashMap;

/// CLI selection preferences
#[derive(Debug, Clone)]
pub struct CLISelectionCriteria {
    /// Preferred CLI type
    pub preferred_cli: Option<CLIType>,
    /// Fallback chain (ordered list of CLI types to try)
    pub fallback_chain: Vec<CLIType>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Cost sensitivity (prefer cheaper CLIs)
    pub cost_sensitive: bool,
    /// Performance priority (prefer faster CLIs)
    pub performance_priority: bool,
}

/// CLI router for selecting and preparing CLI execution
pub struct CLIRouter {
    /// Discovery service for CLI availability
    discovery: DiscoveryService,
    /// Configuration bridge for format translation
    bridge: ConfigurationBridge,
    /// Default fallback chain
    default_fallback_chain: Vec<CLIType>,
}

impl Default for CLIRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CLIRouter {
    /// Create a new CLI router
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery: DiscoveryService::new(),
            bridge: ConfigurationBridge::new(),
            default_fallback_chain: vec![
                CLIType::Claude,
                CLIType::Codex,
                CLIType::Cursor,
                CLIType::Factory,
                CLIType::OpenCode,
            ],
        }
    }

    /// Prepare a CLI execution context for a task
    pub async fn prepare_execution(
        &mut self,
        task: &str,
        universal_config: &UniversalConfig,
        criteria: &CLISelectionCriteria,
    ) -> Result<CLIExecutionContext> {
        // 1. Select the best available CLI
        let selected_cli = self.select_cli(criteria).await?;

        // 2. Translate configuration to CLI-specific format
        let translation = self
            .bridge
            .translate(universal_config, selected_cli)
            .await?;

        // 3. Generate CLI-specific command
        let command = self
            .bridge
            .generate_command(task, universal_config, selected_cli)
            .await?;

        // 4. Prepare execution context
        let context = CLIExecutionContext {
            cli_type: selected_cli,
            working_dir: "/workspace".to_string(),
            env_vars: Self::prepare_environment(selected_cli, &translation.env_vars),
            config_files: translation.config_files,
            command,
        };

        Ok(context)
    }

    /// Select the best CLI based on criteria and availability
    async fn select_cli(&mut self, criteria: &CLISelectionCriteria) -> Result<CLIType> {
        let candidates = self.build_candidate_list(criteria);

        // Try each candidate in order
        for cli_type in candidates {
            if self.discovery.is_available(cli_type).await {
                // Check if CLI meets capability requirements
                if self.meets_requirements(cli_type, criteria).await {
                    return Ok(cli_type);
                }
            }
        }

        // If no candidates work, return the most compatible fallback
        Err(RouterError::NoSuitableCLI(
            "No suitable CLI available".to_string(),
        ))
    }

    /// Build ordered list of CLI candidates based on criteria
    fn build_candidate_list(&self, criteria: &CLISelectionCriteria) -> Vec<CLIType> {
        let mut candidates = Vec::new();

        // Start with preferred CLI if specified
        if let Some(preferred) = criteria.preferred_cli {
            candidates.push(preferred);
        }

        // Add fallback chain
        candidates.extend(&criteria.fallback_chain);

        // Add default fallback chain for any missing CLIs
        for cli in &self.default_fallback_chain {
            if !candidates.contains(cli) {
                candidates.push(*cli);
            }
        }

        candidates
    }

    /// Check if a CLI meets the selection criteria
    async fn meets_requirements(
        &mut self,
        cli_type: CLIType,
        criteria: &CLISelectionCriteria,
    ) -> bool {
        // Discover CLI if we haven't already
        if self.discovery.get_profile(cli_type).is_none()
            && self.discovery.discover_cli(cli_type).await.is_err()
        {
            return false; // Can't discover = not available
        }

        let Some(profile) = self.discovery.get_profile(cli_type) else {
            return false;
        };

        // Check required capabilities
        for capability in &criteria.required_capabilities {
            if !Self::cli_has_capability(profile, capability) {
                return false;
            }
        }

        // Check if bridge supports this CLI
        if !self.bridge.supports_cli(cli_type) {
            return false;
        }

        true
    }

    /// Check if CLI has a specific capability
    fn cli_has_capability(profile: &CLIProfile, capability: &str) -> bool {
        match capability {
            "tools" => profile.capabilities.supports_tools,
            "vision" => profile.capabilities.supports_vision,
            "web_search" => profile.capabilities.supports_web_search,
            "code_execution" => profile.capabilities.supports_code_execution,
            "file_operations" => profile.capabilities.supports_file_operations,
            "persistent_sessions" => matches!(
                profile.capabilities.session_persistence,
                SessionType::Persistent
            ),
            _ => false,
        }
    }

    /// Prepare environment variables for CLI execution
    fn prepare_environment(cli_type: CLIType, required_vars: &[String]) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Add CLI-specific environment setup
        match cli_type {
            CLIType::Claude => {
                // Claude doesn't need special env vars
            }
            CLIType::Codex | CLIType::Cursor | CLIType::Factory | CLIType::OpenCode => {
                env.insert("HOME".to_string(), "/home/node".to_string());
                // Provider-specific keys are added from required_vars
            }
            _ => {
                env.insert("HOME".to_string(), "/workspace".to_string());
            }
        }

        // Add required environment variables
        for var in required_vars {
            if let Ok(value) = std::env::var(var) {
                env.insert(var.clone(), value);
            }
        }

        env
    }

    /// Get CLI profile information
    #[must_use]
    pub fn get_cli_profile(&self, cli_type: CLIType) -> Option<&CLIProfile> {
        self.discovery.get_profile(cli_type)
    }

    /// Check if a CLI is currently available
    pub async fn is_cli_available(&mut self, cli_type: CLIType) -> bool {
        self.discovery.is_available(cli_type).await
    }

    /// Get all supported CLI types
    #[must_use]
    pub fn supported_clis(&self) -> Vec<CLIType> {
        self.bridge.supported_clis()
    }

    /// Set custom fallback chain
    pub fn set_fallback_chain(&mut self, chain: Vec<CLIType>) {
        self.default_fallback_chain = chain;
    }
}

/// Prepared CLI execution context
#[derive(Debug, Clone)]
pub struct PreparedExecution {
    /// Selected CLI type
    pub cli_type: CLIType,
    /// Execution context
    pub context: CLIExecutionContext,
    /// Selection reason
    pub selection_reason: String,
    /// Fallback information
    pub fallback_info: Option<String>,
}

/// Router-specific errors
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("No suitable CLI available: {0}")]
    NoSuitableCLI(String),

    #[error("CLI discovery failed: {0}")]
    DiscoveryError(String),

    #[error("Configuration translation failed: {0}")]
    BridgeError(String),

    #[error("Command generation failed: {0}")]
    CommandError(String),
}

pub type Result<T> = std::result::Result<T, RouterError>;

// Implement From conversions for error handling
impl From<crate::cli::discovery::DiscoveryError> for RouterError {
    fn from(err: crate::cli::discovery::DiscoveryError) -> Self {
        RouterError::DiscoveryError(err.to_string())
    }
}

impl From<crate::cli::bridge::BridgeError> for RouterError {
    fn from(err: crate::cli::bridge::BridgeError) -> Self {
        RouterError::BridgeError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let router = CLIRouter::new();
        let supported = router.supported_clis();
        assert!(supported.contains(&CLIType::Claude));
        assert!(supported.contains(&CLIType::Codex));
        assert!(supported.contains(&CLIType::Cursor));
        assert!(supported.contains(&CLIType::Factory));
    }

    #[test]
    fn test_candidate_list_building() {
        let router = CLIRouter::new();
        let criteria = CLISelectionCriteria {
            preferred_cli: Some(CLIType::Codex),
            fallback_chain: vec![CLIType::OpenCode],
            required_capabilities: vec![],
            cost_sensitive: false,
            performance_priority: false,
        };

        let candidates = router.build_candidate_list(&criteria);
        assert_eq!(candidates[0], CLIType::Codex);
        assert_eq!(candidates[1], CLIType::OpenCode);
        // Default fallbacks should follow
        assert!(candidates.contains(&CLIType::Claude));
        assert!(candidates.contains(&CLIType::Cursor));
        assert!(candidates.contains(&CLIType::Factory));
    }

    #[test]
    fn test_environment_preparation() {
        let env = CLIRouter::prepare_environment(CLIType::Codex, &["OPENAI_API_KEY".to_string()]);

        assert_eq!(env.get("HOME").unwrap(), "/home/node");
        // OPENAI_API_KEY would be added if available in actual environment

        let cursor_env =
            CLIRouter::prepare_environment(CLIType::Cursor, &["CURSOR_API_KEY".to_string()]);
        assert_eq!(cursor_env.get("HOME").unwrap(), "/home/node");

        let factory_env =
            CLIRouter::prepare_environment(CLIType::Factory, &["FACTORY_API_KEY".to_string()]);
        assert_eq!(factory_env.get("HOME").unwrap(), "/home/node");
    }
}
