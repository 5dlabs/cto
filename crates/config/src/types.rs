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

/// Default max concurrent subagents (Claude supports up to 10).
fn default_max_concurrent() -> u8 {
    5
}

/// Subagent configuration for Claude Code parallel execution.
///
/// When enabled, the agent operates as a coordinator that can spawn
/// parallel subagents to work on subtasks concurrently. This is only
/// supported when `cli: "claude"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentConfig {
    /// Enable subagent parallel execution.
    /// When true, the agent receives coordinator instructions and can
    /// dispatch work to parallel subagents.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum concurrent subagents (1-10, default 5).
    /// Claude Code supports up to 10 concurrent subagents.
    #[serde(default = "default_max_concurrent", rename = "maxConcurrent")]
    pub max_concurrent: u8,
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_concurrent: default_max_concurrent(),
        }
    }
}

impl SubagentConfig {
    /// Create a new enabled subagent config with default max concurrent.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            max_concurrent: default_max_concurrent(),
        }
    }

    /// Create a new enabled subagent config with custom max concurrent.
    #[must_use]
    pub fn with_max_concurrent(max_concurrent: u8) -> Self {
        Self {
            enabled: true,
            max_concurrent: max_concurrent.clamp(1, 10),
        }
    }

    /// Check if subagents should be used (enabled and valid CLI).
    #[must_use]
    pub fn should_use(&self, cli: &str) -> bool {
        self.enabled && matches!(cli, "claude" | "opencode")
    }
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

    /// Subagent configuration for parallel execution (Claude Code only).
    /// When enabled, the agent operates as a coordinator that can spawn
    /// multiple subagents to work on subtasks concurrently.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagents: Option<SubagentConfig>,
}

/// Play workflow defaults.
///
/// Agent fields are optional - when not specified, they are constructed from
/// the top-level `orgName` and the hardcoded agent suffix (e.g., "Rex", "Blaze").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayDefaults {
    /// Override implementation agent (defaults to {orgName}-Rex).
    #[serde(
        rename = "implementationAgent",
        skip_serializing_if = "Option::is_none"
    )]
    pub implementation_agent: Option<String>,

    /// Override frontend agent (defaults to {orgName}-Blaze).
    #[serde(rename = "frontendAgent", skip_serializing_if = "Option::is_none")]
    pub frontend_agent: Option<String>,

    /// Override Go agent (defaults to {orgName}-Grizz).
    #[serde(rename = "goAgent", skip_serializing_if = "Option::is_none")]
    pub go_agent: Option<String>,

    /// Override Node agent (defaults to {orgName}-Nova).
    #[serde(rename = "nodeAgent", skip_serializing_if = "Option::is_none")]
    pub node_agent: Option<String>,

    /// Override mobile agent (defaults to {orgName}-Tap).
    #[serde(rename = "mobileAgent", skip_serializing_if = "Option::is_none")]
    pub mobile_agent: Option<String>,

    /// Override desktop agent (defaults to {orgName}-Spark).
    #[serde(rename = "desktopAgent", skip_serializing_if = "Option::is_none")]
    pub desktop_agent: Option<String>,

    /// Override VR agent (defaults to {orgName}-Vex).
    #[serde(rename = "vrAgent", skip_serializing_if = "Option::is_none")]
    pub vr_agent: Option<String>,

    /// Override infrastructure agent (defaults to {orgName}-Bolt).
    #[serde(
        rename = "infrastructureAgent",
        skip_serializing_if = "Option::is_none"
    )]
    pub infrastructure_agent: Option<String>,

    /// Override quality agent (defaults to {orgName}-Cleo).
    #[serde(rename = "qualityAgent", skip_serializing_if = "Option::is_none")]
    pub quality_agent: Option<String>,

    /// Override security agent (defaults to {orgName}-Cipher).
    #[serde(rename = "securityAgent", skip_serializing_if = "Option::is_none")]
    pub security_agent: Option<String>,

    /// Override testing agent (defaults to {orgName}-Tess).
    #[serde(rename = "testingAgent", skip_serializing_if = "Option::is_none")]
    pub testing_agent: Option<String>,

    /// Target repository.
    #[serde(default)]
    pub repository: String,

    /// Service name for workspace isolation.
    #[serde(default)]
    pub service: String,

    /// Docs repository.
    #[serde(rename = "docsRepository", default)]
    pub docs_repository: String,

    /// Docs project directory.
    #[serde(
        rename = "docsProjectDirectory",
        default = "default_docs_project_directory"
    )]
    pub docs_project_directory: String,

    /// Working directory.
    #[serde(rename = "workingDirectory", default = "default_working_directory")]
    pub working_directory: String,

    /// Healer API endpoint for session notifications.
    /// When configured, the MCP server notifies Healer when a Play starts,
    /// enabling real-time monitoring of the workflow lifecycle.
    /// Example: `http://localhost:8083` (local) or `http://cto-healer-play-api:8083` (cluster)
    #[serde(rename = "healerEndpoint", skip_serializing_if = "Option::is_none")]
    pub healer_endpoint: Option<String>,

    /// Retry count before triggering fresh start to combat drift (default: 3).
    /// Based on Cursor's research: periodic fresh starts combat drift and tunnel vision.
    /// When retry count exceeds this threshold, context is cleared and agent restarts fresh.
    #[serde(
        rename = "freshStartThreshold",
        default = "default_fresh_start_threshold"
    )]
    pub fresh_start_threshold: u32,
}

fn default_docs_project_directory() -> String {
    "docs".to_string()
}

fn default_fresh_start_threshold() -> u32 {
    3
}

fn default_working_directory() -> String {
    ".".to_string()
}

impl Default for PlayDefaults {
    fn default() -> Self {
        Self {
            implementation_agent: None,
            frontend_agent: None,
            go_agent: None,
            node_agent: None,
            mobile_agent: None,
            desktop_agent: None,
            vr_agent: None,
            infrastructure_agent: None,
            quality_agent: None,
            security_agent: None,
            testing_agent: None,
            repository: String::new(),
            service: String::new(),
            docs_repository: String::new(),
            docs_project_directory: default_docs_project_directory(),
            working_directory: default_working_directory(),
            healer_endpoint: None,
            fresh_start_threshold: default_fresh_start_threshold(),
        }
    }
}

impl PlayDefaults {
    /// Get the implementation agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_implementation_agent(&self, org_name: &str) -> String {
        self.implementation_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_REX))
    }

    /// Get the frontend agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_frontend_agent(&self, org_name: &str) -> String {
        self.frontend_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_BLAZE))
    }

    /// Get the Go agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_go_agent(&self, org_name: &str) -> String {
        self.go_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_GRIZZ))
    }

    /// Get the Node agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_node_agent(&self, org_name: &str) -> String {
        self.node_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_NOVA))
    }

    /// Get the mobile agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_mobile_agent(&self, org_name: &str) -> String {
        self.mobile_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_TAP))
    }

    /// Get the desktop agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_desktop_agent(&self, org_name: &str) -> String {
        self.desktop_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_SPARK))
    }

    /// Get the VR agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_vr_agent(&self, org_name: &str) -> String {
        self.vr_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_VEX))
    }

    /// Get the infrastructure agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_infrastructure_agent(&self, org_name: &str) -> String {
        self.infrastructure_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_BOLT))
    }

    /// Get the quality agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_quality_agent(&self, org_name: &str) -> String {
        self.quality_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_CLEO))
    }

    /// Get the security agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_security_agent(&self, org_name: &str) -> String {
        self.security_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_CIPHER))
    }

    /// Get the testing agent name, using the org name if not overridden.
    #[must_use]
    pub fn get_testing_agent(&self, org_name: &str) -> String {
        self.testing_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_TESS))
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

    /// Per-CLI model overrides. Maps CLI name (claude, codex, cursor, etc.)
    /// to the model name that CLI should use.
    #[serde(default, rename = "cliModels")]
    pub cli_models: HashMap<String, String>,
}

impl IntakeModels {
    /// Get the model for a specific CLI type.
    /// Falls back to the primary model if no CLI-specific model is configured.
    #[must_use]
    pub fn get_model_for_cli(&self, cli_type: &str) -> &str {
        self.cli_models
            .get(cli_type)
            .map_or(&self.primary, String::as_str)
    }
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

    /// Whether to auto-append a deploy task after all other tasks.
    /// When enabled, a Bolt deploy task is added that depends on all other tasks.
    /// Only applies to deployable projects (web apps, APIs), not libraries.
    /// Default: false.
    #[serde(rename = "autoAppendDeployTask", default)]
    pub auto_append_deploy_task: bool,
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
                cli_models: HashMap::new(), // Will be populated from config
            },
            auto_append_deploy_task: false,
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

/// Default org name for agent GitHub App names.
pub const DEFAULT_ORG_NAME: &str = "5DLabs";

/// Hardcoded agent suffixes - these are the canonical agent names.
pub const AGENT_REX: &str = "Rex";
pub const AGENT_BLAZE: &str = "Blaze";
pub const AGENT_GRIZZ: &str = "Grizz";
pub const AGENT_NOVA: &str = "Nova";
pub const AGENT_TAP: &str = "Tap";
pub const AGENT_SPARK: &str = "Spark";
pub const AGENT_BOLT: &str = "Bolt";
pub const AGENT_CLEO: &str = "Cleo";
pub const AGENT_CIPHER: &str = "Cipher";
pub const AGENT_TESS: &str = "Tess";
pub const AGENT_MORGAN: &str = "Morgan";
pub const AGENT_ATLAS: &str = "Atlas";
pub const AGENT_VEX: &str = "Vex";

/// Construct a full agent GitHub App name from org name and agent suffix.
#[must_use]
pub fn make_agent_name(org_name: &str, agent_suffix: &str) -> String {
    format!("{org_name}-{agent_suffix}")
}

fn default_org_name() -> String {
    DEFAULT_ORG_NAME.to_string()
}

/// Complete CTO Config structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CtoConfig {
    /// Config version.
    pub version: String,

    /// Organization name used to construct agent GitHub App names.
    /// For example, "5DLabs" results in agent names like "5DLabs-Rex", "5DLabs-Blaze".
    #[serde(rename = "orgName", default = "default_org_name")]
    pub org_name: String,

    /// Default configurations for workflows.
    pub defaults: Defaults,

    /// Agent configurations.
    pub agents: HashMap<String, AgentConfig>,
}

impl Default for CtoConfig {
    fn default() -> Self {
        Self {
            version: CTO_CONFIG_VERSION.to_string(),
            org_name: default_org_name(),
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
        // Agent fields are None by default, resolved via org_name
        assert!(defaults.implementation_agent.is_none());
        assert!(defaults.frontend_agent.is_none());
        assert!(defaults.quality_agent.is_none());

        // Test the getter methods with org name
        assert_eq!(defaults.get_implementation_agent("5DLabs"), "5DLabs-Rex");
        assert_eq!(defaults.get_frontend_agent("5DLabs"), "5DLabs-Blaze");
        assert_eq!(defaults.get_quality_agent("5DLabs"), "5DLabs-Cleo");

        // Test with custom org name
        assert_eq!(defaults.get_implementation_agent("Acme"), "Acme-Rex");
        assert_eq!(defaults.get_frontend_agent("Acme"), "Acme-Blaze");

        // Test fresh_start_threshold default
        assert_eq!(defaults.fresh_start_threshold, 3);
    }

    #[test]
    fn test_fresh_start_threshold_from_json() {
        // Test parsing freshStartThreshold from JSON
        let json = r#"{
            "freshStartThreshold": 5,
            "repository": "test-repo",
            "service": "test-service"
        }"#;
        let defaults: PlayDefaults = serde_json::from_str(json).unwrap();
        assert_eq!(defaults.fresh_start_threshold, 5);

        // Test default value when not specified
        let json_no_threshold = r#"{
            "repository": "test-repo",
            "service": "test-service"
        }"#;
        let defaults_default: PlayDefaults = serde_json::from_str(json_no_threshold).unwrap();
        assert_eq!(defaults_default.fresh_start_threshold, 3);
    }

    #[test]
    fn test_make_agent_name() {
        assert_eq!(make_agent_name("5DLabs", AGENT_REX), "5DLabs-Rex");
        assert_eq!(make_agent_name("Acme", AGENT_BLAZE), "Acme-Blaze");
        assert_eq!(make_agent_name("MyOrg", AGENT_TESS), "MyOrg-Tess");
    }

    #[test]
    fn test_org_name_default() {
        let config = CtoConfig::default();
        assert_eq!(config.org_name, "5DLabs");
    }

    #[test]
    fn test_intake_defaults() {
        let defaults = IntakeDefaults::default();
        assert_eq!(defaults.github_app, "5DLabs-Morgan");
        assert_eq!(defaults.cli, "claude");
        assert_eq!(defaults.source_branch, "main");
        assert!(!defaults.auto_append_deploy_task);
    }

    #[test]
    fn test_intake_auto_append_deploy_task_from_json() {
        // Test parsing autoAppendDeployTask from JSON (enabled)
        let json = r#"{
            "githubApp": "5DLabs-Morgan",
            "cli": "claude",
            "sourceBranch": "main",
            "models": {
                "primary": "claude-opus-4-5-20251101",
                "research": "claude-opus-4-5-20251101",
                "fallback": "claude-opus-4-5-20251101"
            },
            "autoAppendDeployTask": true
        }"#;
        let defaults: IntakeDefaults = serde_json::from_str(json).unwrap();
        assert!(defaults.auto_append_deploy_task);

        // Test default value when not specified (disabled)
        let json_no_flag = r#"{
            "githubApp": "5DLabs-Morgan",
            "cli": "claude",
            "sourceBranch": "main",
            "models": {
                "primary": "claude-opus-4-5-20251101",
                "research": "claude-opus-4-5-20251101",
                "fallback": "claude-opus-4-5-20251101"
            }
        }"#;
        let defaults_default: IntakeDefaults = serde_json::from_str(json_no_flag).unwrap();
        assert!(!defaults_default.auto_append_deploy_task);
    }

    #[test]
    fn test_subagent_config_default() {
        let config = SubagentConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_concurrent, 5);
    }

    #[test]
    fn test_subagent_config_enabled() {
        let config = SubagentConfig::enabled();
        assert!(config.enabled);
        assert_eq!(config.max_concurrent, 5);
    }

    #[test]
    fn test_subagent_config_with_max_concurrent() {
        let config = SubagentConfig::with_max_concurrent(8);
        assert!(config.enabled);
        assert_eq!(config.max_concurrent, 8);

        // Test clamping to max 10
        let config_max = SubagentConfig::with_max_concurrent(15);
        assert_eq!(config_max.max_concurrent, 10);

        // Test clamping to min 1
        let config_min = SubagentConfig::with_max_concurrent(0);
        assert_eq!(config_min.max_concurrent, 1);
    }

    #[test]
    fn test_subagent_config_should_use() {
        let config = SubagentConfig::enabled();
        // Supported CLIs
        assert!(config.should_use("claude"));
        assert!(config.should_use("opencode"));
        // Unsupported CLIs
        assert!(!config.should_use("codex"));
        assert!(!config.should_use("gemini"));

        let disabled = SubagentConfig::default();
        assert!(!disabled.should_use("claude"));
        assert!(!disabled.should_use("opencode"));
    }

    #[test]
    fn test_subagent_config_serde() {
        let json = r#"{
            "enabled": true,
            "maxConcurrent": 7
        }"#;
        let config: SubagentConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.max_concurrent, 7);
    }

    #[test]
    fn test_agent_config_with_subagents() {
        let json = r#"{
            "githubApp": "5DLabs-Rex",
            "cli": "claude",
            "model": "claude-opus-4-5-20251101",
            "subagents": {
                "enabled": true,
                "maxConcurrent": 5
            }
        }"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.subagents.is_some());
        let subagents = config.subagents.unwrap();
        assert!(subagents.enabled);
        assert_eq!(subagents.max_concurrent, 5);
    }

    #[test]
    fn test_agent_config_without_subagents() {
        let json = r#"{
            "githubApp": "5DLabs-Rex",
            "cli": "claude",
            "model": "claude-opus-4-5-20251101"
        }"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.subagents.is_none());
    }
}
