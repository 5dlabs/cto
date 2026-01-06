//! CTO Configuration types.
//!
//! Defines the structure of `cto-config.json` files used by Play workflows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CTO Config version.
pub const CTO_CONFIG_VERSION: &str = "1.0";

/// Agent tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AgentTools {
    /// Remote tools from platform tools-server.
    #[serde(default)]
    pub remote: Vec<String>,

    /// Local MCP servers to spawn per-agent.
    #[serde(default, rename = "localServers")]
    pub local_servers: HashMap<String, serde_json::Value>,
}

/// Individual agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// GitHub App name for this agent.
    #[serde(rename = "githubApp")]
    pub github_app: String,

    /// CLI to use (claude, codex, gemini, opencode).
    pub cli: String,

    /// AI model to use.
    pub model: String,

    /// MCP tools configuration.
    #[serde(default)]
    pub tools: AgentTools,

    /// Frontend stack (for Blaze only).
    #[serde(skip_serializing_if = "Option::is_none", rename = "frontendStack")]
    pub frontend_stack: Option<String>,

    /// Feature flags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

/// Play workflow defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayDefaults {
    /// Implementation agent GitHub App.
    #[serde(rename = "implementationAgent")]
    pub implementation_agent: String,

    /// Frontend agent GitHub App.
    #[serde(rename = "frontendAgent")]
    pub frontend_agent: String,

    /// Go agent GitHub App.
    #[serde(rename = "goAgent")]
    pub go_agent: String,

    /// Node agent GitHub App.
    #[serde(rename = "nodeAgent")]
    pub node_agent: String,

    /// Mobile agent GitHub App.
    #[serde(rename = "mobileAgent")]
    pub mobile_agent: String,

    /// Desktop agent GitHub App.
    #[serde(rename = "desktopAgent")]
    pub desktop_agent: String,

    /// Infrastructure agent GitHub App.
    #[serde(rename = "infrastructureAgent")]
    pub infrastructure_agent: String,

    /// Quality agent GitHub App.
    #[serde(rename = "qualityAgent")]
    pub quality_agent: String,

    /// Security agent GitHub App.
    #[serde(rename = "securityAgent")]
    pub security_agent: String,

    /// Testing agent GitHub App.
    #[serde(rename = "testingAgent")]
    pub testing_agent: String,

    /// Target repository.
    pub repository: String,

    /// Service name for workspace isolation.
    pub service: String,

    /// Docs repository.
    #[serde(rename = "docsRepository")]
    pub docs_repository: String,

    /// Docs project directory.
    #[serde(rename = "docsProjectDirectory")]
    pub docs_project_directory: String,

    /// Working directory.
    #[serde(rename = "workingDirectory")]
    pub working_directory: String,
}

impl Default for PlayDefaults {
    fn default() -> Self {
        Self {
            implementation_agent: "5DLabs-Rex".to_string(),
            frontend_agent: "5DLabs-Blaze".to_string(),
            go_agent: "5DLabs-Grizz".to_string(),
            node_agent: "5DLabs-Nova".to_string(),
            mobile_agent: "5DLabs-Tap".to_string(),
            desktop_agent: "5DLabs-Spark".to_string(),
            infrastructure_agent: "5DLabs-Bolt".to_string(),
            quality_agent: "5DLabs-Cleo".to_string(),
            security_agent: "5DLabs-Cipher".to_string(),
            testing_agent: "5DLabs-Tess".to_string(),
            repository: String::new(),
            service: String::new(),
            docs_repository: String::new(),
            docs_project_directory: "docs".to_string(),
            working_directory: ".".to_string(),
        }
    }
}

/// Simplified model configuration for intake.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct IntakeModels {
    /// Primary model for task generation.
    #[serde(default)]
    pub primary: String,

    /// Research model for context enrichment.
    #[serde(default)]
    pub research: String,

    /// Fallback model if primary fails.
    #[serde(default)]
    pub fallback: String,
}

/// Intake workflow defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntakeDefaults {
    /// GitHub App for intake.
    #[serde(rename = "githubApp")]
    pub github_app: String,

    /// CLI to use for intake.
    pub cli: String,

    /// Whether to include codebase context.
    #[serde(rename = "includeCodebase", default)]
    pub include_codebase: bool,

    /// Source branch for PRs.
    #[serde(rename = "sourceBranch", default = "default_source_branch")]
    pub source_branch: String,

    /// Model configuration.
    pub models: IntakeModels,
}

fn default_source_branch() -> String {
    "main".to_string()
}

impl Default for IntakeDefaults {
    fn default() -> Self {
        Self {
            github_app: "5DLabs-Morgan".to_string(),
            cli: "claude".to_string(),
            include_codebase: false,
            source_branch: "main".to_string(),
            models: IntakeModels {
                primary: "claude-opus-4-5-20251101".to_string(),
                research: "claude-opus-4-5-20251101".to_string(),
                fallback: "claude-opus-4-5-20251101".to_string(),
            },
        }
    }
}

/// Linear integration settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinearIntakeSettings {
    /// Whether to create a project.
    #[serde(rename = "createProject", default = "default_true")]
    pub create_project: bool,

    /// Project template to use.
    #[serde(rename = "projectTemplate", default = "default_project_template")]
    pub project_template: String,
}

fn default_true() -> bool {
    true
}

fn default_project_template() -> String {
    "Play Workflow".to_string()
}

impl Default for LinearIntakeSettings {
    fn default() -> Self {
        Self {
            create_project: true,
            project_template: "Play Workflow".to_string(),
        }
    }
}

/// Linear integration defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinearDefaults {
    /// Linear team ID.
    #[serde(rename = "teamId")]
    pub team_id: String,

    /// PM server URL.
    #[serde(rename = "pmServerUrl", default = "default_pm_server_url")]
    pub pm_server_url: String,

    /// Intake-specific settings.
    #[serde(default)]
    pub intake: LinearIntakeSettings,
}

fn default_pm_server_url() -> String {
    "https://pm.5dlabs.ai".to_string()
}

impl Default for LinearDefaults {
    fn default() -> Self {
        Self {
            team_id: String::new(),
            pm_server_url: "https://pm.5dlabs.ai".to_string(),
            intake: LinearIntakeSettings::default(),
        }
    }
}

/// All default configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Defaults {
    /// Intake workflow defaults.
    #[serde(default)]
    pub intake: IntakeDefaults,

    /// Linear integration defaults.
    #[serde(default)]
    pub linear: LinearDefaults,

    /// Play workflow defaults.
    #[serde(default)]
    pub play: PlayDefaults,
}

/// Complete CTO Config structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CtoConfig {
    /// Config version.
    pub version: String,

    /// Default configurations for workflows.
    pub defaults: Defaults,

    /// Agent configurations.
    pub agents: HashMap<String, AgentConfig>,
}

impl Default for CtoConfig {
    fn default() -> Self {
        Self {
            version: CTO_CONFIG_VERSION.to_string(),
            defaults: Defaults::default(),
            agents: HashMap::new(),
        }
    }
}

impl CtoConfig {
    /// Create a new CTO config with the given version.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize the config to a JSON string.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize the config to a JSON string (compact).
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse a CTO config from a JSON string.
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CtoConfig::default();
        assert_eq!(config.version, CTO_CONFIG_VERSION);
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_roundtrip_serialization() {
        let config = CtoConfig::default();
        let json = config.to_json().unwrap();
        let parsed = CtoConfig::from_json(&json).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_play_defaults() {
        let defaults = PlayDefaults::default();
        assert_eq!(defaults.implementation_agent, "5DLabs-Rex");
        assert_eq!(defaults.frontend_agent, "5DLabs-Blaze");
        assert_eq!(defaults.quality_agent, "5DLabs-Cleo");
    }

    #[test]
    fn test_intake_defaults() {
        let defaults = IntakeDefaults::default();
        assert_eq!(defaults.github_app, "5DLabs-Morgan");
        assert_eq!(defaults.cli, "claude");
        assert_eq!(defaults.source_branch, "main");
    }
}
