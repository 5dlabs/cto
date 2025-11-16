//! CLI Discovery Service
//!
//! Responsible for discovering and profiling CLI tools to understand
//! their capabilities, configuration requirements, and compatibility.

use crate::cli::types::{CLIType, CLIProfile, CLIAvailability, CLIConfiguration, ConfigFormat, CLICapabilities, SessionType, CostModel};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::info;

/// Discovery service for profiling CLI tools
pub struct DiscoveryService {
    /// Cache of discovered CLI profiles
    discovered_profiles: HashMap<CLIType, CLIProfile>,
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryService {
    /// Create a new discovery service
    #[must_use] pub fn new() -> Self {
        Self {
            discovered_profiles: HashMap::new(),
        }
    }

    /// Discover a CLI's capabilities and create a profile
    pub async fn discover_cli(&mut self, cli_type: CLIType) -> Result<CLIProfile> {
        info!("🔍 Starting discovery for {cli_type:?}");

        // Phase 1: Basic availability check
        let _availability = self.check_availability(cli_type).await?;

        // Phase 2: Configuration format discovery
        let configuration = self.discover_configuration(cli_type)?;

        // Phase 3: Capability assessment
        let capabilities = self.assess_capabilities(cli_type)?;

        // Phase 4: Performance profiling
        let cost_model = self.profile_performance(cli_type)?;

        let profile = CLIProfile {
            name: cli_type.to_string(),
            cli_type,
            capabilities,
            configuration,
            cost_model,
            discovered_at: chrono::Utc::now(),
        };

        // Cache the profile
        self.discovered_profiles.insert(cli_type, profile.clone());

        info!("✅ Discovery complete for {cli_type:?}");
        Ok(profile)
    }

    /// Check if a CLI is available and get version info
    async fn check_availability(&self, cli_type: CLIType) -> Result<CLIAvailability> {
        let (command, args) = self.get_version_command(cli_type);

        let output = Command::new(command)
            .args(&args)
            .output()
            .await
            .map_err(|e| DiscoveryError::CommandFailed(format!("Failed to run {command}: {e}")))?;

        Ok(CLIAvailability {
            available: output.status.success(),
            version: if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "unknown".to_string()
            },
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Discover configuration format and requirements
    fn discover_configuration(&self, cli_type: CLIType) -> Result<CLIConfiguration> {
        match cli_type {
            CLIType::Claude => Ok(CLIConfiguration {
                config_format: ConfigFormat::Markdown,
                config_location: "/workspace/CLAUDE.md".to_string(),
                required_env_vars: vec![],
                init_commands: vec![],
                cleanup_commands: vec![],
            }),
            CLIType::Codex => Ok(CLIConfiguration {
                config_format: ConfigFormat::TOML,
                config_location: "/home/node/.codex/config.toml".to_string(),
                required_env_vars: vec!["OPENAI_API_KEY".to_string()],
                init_commands: vec![],
                cleanup_commands: vec![],
            }),
            CLIType::Cursor => Ok(CLIConfiguration {
                config_format: ConfigFormat::JSON,
                config_location: "/workspace/.cursor/cli.json".to_string(),
                required_env_vars: vec!["CURSOR_API_KEY".to_string()],
                init_commands: vec![],
                cleanup_commands: vec![],
            }),
            CLIType::Factory => Ok(CLIConfiguration {
                config_format: ConfigFormat::JSON,
                config_location: "/workspace/.factory/cli.json".to_string(),
                required_env_vars: vec!["FACTORY_API_KEY".to_string()],
                init_commands: vec![],
                cleanup_commands: vec![],
            }),
            CLIType::OpenCode => Ok(CLIConfiguration {
                config_format: ConfigFormat::JSON,
                config_location: "/home/node/.config/opencode/config.json".to_string(),
                required_env_vars: vec!["OPENAI_API_KEY".to_string()],
                init_commands: vec![],
                cleanup_commands: vec![],
            }),
            _ => Ok(CLIConfiguration {
                config_format: ConfigFormat::JSON,
                config_location: "/workspace/config.json".to_string(),
                required_env_vars: vec![],
                init_commands: vec![],
                cleanup_commands: vec![],
            }),
        }
    }

    /// Assess CLI capabilities
    fn assess_capabilities(&self, cli_type: CLIType) -> Result<CLICapabilities> {
        // For now, return known capabilities based on our research
        match cli_type {
            CLIType::Claude => Ok(CLICapabilities {
                max_context_window: 200_000,
                supports_tools: true,
                supports_vision: false,
                supports_web_search: false,
                supports_code_execution: true,
                supports_file_operations: true,
                session_persistence: SessionType::Persistent,
            }),
            CLIType::Codex => Ok(CLICapabilities {
                max_context_window: 128_000,
                supports_tools: true,
                supports_vision: false,
                supports_web_search: true,
                supports_code_execution: true,
                supports_file_operations: true,
                session_persistence: SessionType::Persistent,
            }),
            CLIType::Cursor => Ok(CLICapabilities {
                max_context_window: 128_000,
                supports_tools: true,
                supports_vision: false,
                supports_web_search: true,
                supports_code_execution: true,
                supports_file_operations: true,
                session_persistence: SessionType::Persistent,
            }),
            CLIType::Factory => Ok(CLICapabilities {
                max_context_window: 128_000,
                supports_tools: true,
                supports_vision: false,
                supports_web_search: true,
                supports_code_execution: true,
                supports_file_operations: true,
                session_persistence: SessionType::Persistent,
            }),
            CLIType::OpenCode => Ok(CLICapabilities {
                max_context_window: 128_000,
                supports_tools: true,
                supports_vision: true,
                supports_web_search: true,
                supports_code_execution: true,
                supports_file_operations: true,
                session_persistence: SessionType::Persistent,
            }),
            _ => Ok(CLICapabilities {
                max_context_window: 8000,
                supports_tools: false,
                supports_vision: false,
                supports_web_search: false,
                supports_code_execution: false,
                supports_file_operations: true,
                session_persistence: SessionType::Stateless,
            }),
        }
    }

    /// Profile performance characteristics
    fn profile_performance(&self, cli_type: CLIType) -> Result<CostModel> {
        // Return estimated cost models based on our research
        match cli_type {
            CLIType::Claude => Ok(CostModel {
                input_token_cost: 0.003,
                output_token_cost: 0.015,
                free_tier_tokens: None,
            }),
            CLIType::Codex => Ok(CostModel {
                input_token_cost: 0.0015,
                output_token_cost: 0.006,
                free_tier_tokens: None,
            }),
            CLIType::Cursor => Ok(CostModel {
                input_token_cost: 0.0015,
                output_token_cost: 0.006,
                free_tier_tokens: None,
            }),
            CLIType::Factory => Ok(CostModel {
                input_token_cost: 0.0015,
                output_token_cost: 0.006,
                free_tier_tokens: None,
            }),
            CLIType::OpenCode => Ok(CostModel {
                input_token_cost: 0.003,
                output_token_cost: 0.015,
                free_tier_tokens: None,
            }),
            _ => Ok(CostModel {
                input_token_cost: 0.002,
                output_token_cost: 0.010,
                free_tier_tokens: None,
            }),
        }
    }

    /// Get the version command for a CLI type
    fn get_version_command(&self, cli_type: CLIType) -> (&str, Vec<&str>) {
        match cli_type {
            CLIType::Claude => ("claude-code", vec!["--version"]),
            CLIType::Codex => ("codex", vec!["--version"]),
            CLIType::OpenCode => ("opencode", vec!["--version"]),
            CLIType::Cursor => ("cursor-agent", vec!["--version"]),
            CLIType::Factory => ("droid", vec!["--version"]),
            CLIType::OpenHands => (
                "python3",
                vec!["-c", "import openhands; print(openhands.__version__)"],
            ),
            CLIType::Grok => ("grok-cli", vec!["--version"]),
            CLIType::Gemini => ("gemini-cli", vec!["--version"]),
            CLIType::Qwen => ("qwen-cli", vec!["--version"]),
        }
    }

    /// Get a cached profile if available
    #[must_use] pub fn get_profile(&self, cli_type: CLIType) -> Option<&CLIProfile> {
        self.discovered_profiles.get(&cli_type)
    }

    /// Check if a CLI is available
    pub async fn is_available(&self, cli_type: CLIType) -> bool {
        self.check_availability(cli_type)
            .await
            .map(|avail| avail.available)
            .unwrap_or(false)
    }
}

/// Errors that can occur during discovery
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Configuration discovery failed: {0}")]
    ConfigDiscoveryFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;
