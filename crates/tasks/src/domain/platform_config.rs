//! Platform Configuration - Global settings for CTO platform workflows.
//!
//! This module defines the platform-level configuration that controls:
//! - Intake workflow execution (API mode vs CLI mode)
//! - Default images for workflows
//! - Model preferences
//!
//! **File Locations** (checked in order):
//! 1. `PLATFORM_CONFIG` environment variable
//! 2. `.cto/platform-config.json` in current directory
//! 3. `platform-config.json` in current directory
//! 4. `~/.cto/platform-config.json`
//!
//! **Separate from cto-config.json:**
//! - `platform-config.json` - Global platform settings (how intake runs)
//! - `cto-config.json` - Per-project settings (which agents, tools, etc.)

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::errors::{TasksError, TasksResult};

/// Platform config version
pub const PLATFORM_CONFIG_VERSION: &str = "1.0";

/// Intake execution mode
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IntakeMode {
    /// Direct API calls via `tasks intake` binary (default, faster)
    /// Uses ANTHROPIC_API_KEY/OPENAI_API_KEY directly
    #[default]
    Api,
    /// Use AI CLI (claude, codex, etc.) for intake
    /// Requires the CLI to be installed in the container image
    Cli,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model identifier (e.g., "claude-opus-4-5-20250929")
    pub model: String,

    /// Provider name (e.g., "anthropic", "openai")
    pub provider: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: "claude-opus-4-5-20250929".to_string(),
            provider: "anthropic".to_string(),
        }
    }
}

/// Intake workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeSettings {
    /// Execution mode: "api" (direct API calls) or "cli" (use AI CLI)
    #[serde(default)]
    pub mode: IntakeMode,

    /// CLI to use when mode = "cli"
    /// Options: claude, codex, gemini, opencode
    #[serde(default = "default_intake_cli")]
    pub cli: String,

    /// Container image for intake workflow
    /// - API mode: use "runtime" image (smaller, has tasks CLI)
    /// - CLI mode: use "factory" image (has all AI CLIs)
    #[serde(default = "default_intake_image")]
    pub image: String,

    /// Primary model for task generation
    #[serde(default)]
    pub primary_model: ModelConfig,

    /// Research model for context enrichment
    #[serde(default)]
    pub research_model: ModelConfig,

    /// Fallback model when primary fails
    #[serde(default = "default_fallback_model")]
    pub fallback_model: ModelConfig,

    /// Maximum retries for intake
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable research/context enrichment via Firecrawl
    #[serde(default = "default_enable_research")]
    pub enable_research: bool,
}

fn default_intake_cli() -> String {
    "claude".to_string()
}

fn default_intake_image() -> String {
    "ghcr.io/5dlabs/runtime:latest".to_string()
}

fn default_fallback_model() -> ModelConfig {
    ModelConfig {
        model: "claude-sonnet-4-20250514".to_string(),
        provider: "anthropic".to_string(),
    }
}

fn default_max_retries() -> u32 {
    3
}

fn default_enable_research() -> bool {
    true
}

impl Default for IntakeSettings {
    fn default() -> Self {
        Self {
            mode: IntakeMode::Api,
            cli: default_intake_cli(),
            image: default_intake_image(),
            primary_model: ModelConfig::default(),
            research_model: ModelConfig::default(),
            fallback_model: default_fallback_model(),
            max_retries: default_max_retries(),
            enable_research: default_enable_research(),
        }
    }
}

/// Play workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaySettings {
    /// Default CLI for play workflows
    #[serde(default = "default_play_cli")]
    pub cli: String,

    /// Container image for play workflows (agents)
    #[serde(default = "default_play_image")]
    pub image: String,

    /// Default model for agents
    #[serde(default = "default_play_model")]
    pub model: String,

    /// Enable parallel execution of independent tasks
    #[serde(default = "default_parallel")]
    pub parallel_execution: bool,

    /// Auto-merge PRs after all checks pass
    #[serde(default)]
    pub auto_merge: bool,

    /// Maximum retries per agent
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_play_cli() -> String {
    "claude".to_string()
}

fn default_play_image() -> String {
    "ghcr.io/5dlabs/factory:latest".to_string()
}

fn default_play_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_parallel() -> bool {
    true
}

impl Default for PlaySettings {
    fn default() -> Self {
        Self {
            cli: default_play_cli(),
            image: default_play_image(),
            model: default_play_model(),
            parallel_execution: default_parallel(),
            auto_merge: false,
            max_retries: default_max_retries(),
        }
    }
}

/// Complete platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Config version
    #[serde(default = "default_version")]
    pub version: String,

    /// Intake workflow settings
    #[serde(default)]
    pub intake: IntakeSettings,

    /// Play workflow settings
    #[serde(default)]
    pub play: PlaySettings,
}

fn default_version() -> String {
    PLATFORM_CONFIG_VERSION.to_string()
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            version: PLATFORM_CONFIG_VERSION.to_string(),
            intake: IntakeSettings::default(),
            play: PlaySettings::default(),
        }
    }
}

impl PlatformConfig {
    /// Load platform config from standard locations.
    ///
    /// Checks locations in order:
    /// 1. `PLATFORM_CONFIG` environment variable
    /// 2. `.cto/platform-config.json` in current directory
    /// 3. `platform-config.json` in current directory
    /// 4. `~/.cto/platform-config.json`
    ///
    /// Returns default config if no file found.
    pub async fn load() -> TasksResult<Self> {
        // Check env var first
        if let Ok(path) = std::env::var("PLATFORM_CONFIG") {
            if let Ok(config) = Self::load_from_path(Path::new(&path)).await {
                tracing::info!("Loaded platform config from PLATFORM_CONFIG={}", path);
                return Ok(config);
            }
        }

        // Check standard locations
        let locations: Vec<PathBuf> = vec![
            PathBuf::from(".cto/platform-config.json"),
            PathBuf::from("platform-config.json"),
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".cto/platform-config.json"))
                .unwrap_or_default(),
        ];

        for path in &locations {
            if path.exists() {
                if let Ok(config) = Self::load_from_path(path).await {
                    tracing::info!("Loaded platform config from {}", path.display());
                    return Ok(config);
                }
            }
        }

        // Return default config
        tracing::info!("No platform config found, using defaults");
        Ok(Self::default())
    }

    /// Load platform config from a specific path.
    pub async fn load_from_path(path: &Path) -> TasksResult<Self> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| TasksError::FileReadError {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        serde_json::from_str(&content).map_err(|e| TasksError::JsonParseError {
            reason: format!("platform config at {}: {}", path.display(), e),
        })
    }

    /// Save platform config to a path.
    pub async fn save(&self, path: &Path) -> TasksResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| TasksError::FileWriteError {
                    path: parent.display().to_string(),
                    reason: e.to_string(),
                })?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Get the appropriate image for intake based on mode.
    pub fn intake_image(&self) -> &str {
        // If image is explicitly set, use it
        if !self.intake.image.is_empty()
            && self.intake.image != "ghcr.io/5dlabs/runtime:latest"
        {
            return &self.intake.image;
        }

        // Otherwise, derive from mode
        match self.intake.mode {
            IntakeMode::Api => "ghcr.io/5dlabs/runtime:latest",
            IntakeMode::Cli => "ghcr.io/5dlabs/factory:latest",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PlatformConfig::default();
        assert_eq!(config.version, PLATFORM_CONFIG_VERSION);
        assert_eq!(config.intake.mode, IntakeMode::Api);
        assert_eq!(config.intake.cli, "claude");
        assert!(config.intake.enable_research);
        assert!(config.play.parallel_execution);
    }

    #[test]
    fn test_intake_image_api_mode() {
        let config = PlatformConfig::default();
        assert_eq!(config.intake_image(), "ghcr.io/5dlabs/runtime:latest");
    }

    #[test]
    fn test_intake_image_cli_mode() {
        let mut config = PlatformConfig::default();
        config.intake.mode = IntakeMode::Cli;
        config.intake.image = "ghcr.io/5dlabs/runtime:latest".to_string(); // Reset to trigger derivation
        assert_eq!(config.intake_image(), "ghcr.io/5dlabs/factory:latest");
    }

    #[test]
    fn test_intake_image_explicit() {
        let mut config = PlatformConfig::default();
        config.intake.image = "custom/image:v1".to_string();
        assert_eq!(config.intake_image(), "custom/image:v1");
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = PlatformConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: PlatformConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, config.version);
        assert_eq!(parsed.intake.mode, config.intake.mode);
    }

    #[test]
    fn test_deserialize_api_mode() {
        let json = r#"{"intake": {"mode": "api"}}"#;
        let config: PlatformConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.intake.mode, IntakeMode::Api);
    }

    #[test]
    fn test_deserialize_cli_mode() {
        let json = r#"{"intake": {"mode": "cli", "cli": "codex"}}"#;
        let config: PlatformConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.intake.mode, IntakeMode::Cli);
        assert_eq!(config.intake.cli, "codex");
    }
}
