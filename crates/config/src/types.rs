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

/// Agent skills configuration.
///
/// Skills are organized by job type. The `default` skills are always included,
/// and job-type-specific skills are merged when the agent performs that job type.
///
/// This replaces the legacy `skill-mappings.yaml` file with a unified config.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AgentSkills {
    /// Default skills always loaded for this agent.
    #[serde(default)]
    pub default: Vec<String>,

    /// Skills for coder job type (implementation tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coder: Option<Vec<String>>,

    /// Skills for healer job type (incident response, remediation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healer: Option<Vec<String>>,

    /// Skills for intake job type (PRD processing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intake: Option<Vec<String>>,

    /// Skills for quality job type (code review).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<Vec<String>>,

    /// Skills for test job type (testing tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<Vec<String>>,

    /// Skills for security job type (security analysis).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<String>>,

    /// Skills for review job type (PR review).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<Vec<String>>,

    /// Skills for deploy job type (infrastructure deployment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy: Option<Vec<String>>,

    /// Skills for integration job type (CI/merge tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration: Option<Vec<String>>,

    /// Optional skills that can be enabled on-demand.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<Vec<String>>,
}

impl AgentSkills {
    /// Get skills for a specific job type, merged with defaults.
    ///
    /// Returns default skills + job-type-specific skills (if any).
    #[must_use]
    pub fn get_skills_for_job(&self, job_type: &str) -> Vec<String> {
        let mut skills = self.default.clone();

        let job_skills = match job_type {
            "coder" => self.coder.as_ref(),
            "healer" => self.healer.as_ref(),
            "intake" => self.intake.as_ref(),
            "quality" => self.quality.as_ref(),
            "test" => self.test.as_ref(),
            "security" => self.security.as_ref(),
            "review" => self.review.as_ref(),
            "deploy" => self.deploy.as_ref(),
            "integration" => self.integration.as_ref(),
            _ => None,
        };

        if let Some(job_skills) = job_skills {
            for skill in job_skills {
                if !skills.contains(skill) {
                    skills.push(skill.clone());
                }
            }
        }

        skills
    }

    /// Check if this agent has any skills configured.
    #[must_use]
    pub fn has_skills(&self) -> bool {
        !self.default.is_empty()
            || self.coder.is_some()
            || self.healer.is_some()
            || self.intake.is_some()
            || self.quality.is_some()
            || self.test.is_some()
            || self.security.is_some()
            || self.review.is_some()
            || self.deploy.is_some()
            || self.integration.is_some()
    }
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

/// Agent communication mode for Play workflow delegations.
///
/// `a2a` is the HTTP JSON-RPC path used by OpenClaw today. `acp` remains a
/// deprecated config alias for backward compatibility when deserializing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentCommunicationMode {
    /// Native OpenClaw subagent hook invocation.
    #[default]
    Subagent,
    /// HTTP A2A JSON-RPC transport.
    #[serde(alias = "acp")]
    A2a,
}

impl AgentCommunicationMode {
    /// Get the canonical serialized value for this transport.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Subagent => "subagent",
            Self::A2a => "a2a",
        }
    }
}

/// ACP runtime transport type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AcpTransport {
    /// ACP over stdio.
    #[default]
    Stdio,
}

/// Shared ACP runtime definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpRuntimeConfig {
    /// Whether this runtime is available for selection.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// ACP transport implementation.
    #[serde(default)]
    pub transport: AcpTransport,

    /// Binary or shell command to execute.
    pub command: String,

    /// Arguments passed to the runtime command.
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional working directory override for the runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// Additional environment variables for the runtime.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl AcpRuntimeConfig {
    /// Create a stdio runtime definition.
    #[must_use]
    pub fn stdio(
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            enabled: true,
            transport: AcpTransport::Stdio,
            command: command.into(),
            args: args.into_iter().map(Into::into).collect(),
            cwd: None,
            env: HashMap::new(),
        }
    }
}

impl Default for AcpRuntimeConfig {
    fn default() -> Self {
        Self::stdio("stakpak", ["acp"])
    }
}

/// ACP service-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpServiceConfig {
    /// Whether ACP delegation is enabled for this service.
    #[serde(default)]
    pub enabled: bool,

    /// Explicit runtime IDs this service is allowed to use.
    #[serde(default, rename = "runtimeIds")]
    pub runtime_ids: Vec<String>,

    /// Default runtime ID for this service.
    #[serde(skip_serializing_if = "Option::is_none", rename = "defaultRuntime")]
    pub default_runtime: Option<String>,

    /// Internal-only caller allowlist for ACP server surfaces.
    #[serde(default, rename = "allowedCallers")]
    pub allowed_callers: Vec<String>,
}

impl Default for AcpServiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            runtime_ids: Vec::new(),
            default_runtime: None,
            allowed_callers: Vec::new(),
        }
    }
}

impl AcpServiceConfig {
    /// Create a disabled service configuration bound to the given runtime IDs.
    #[must_use]
    pub fn disabled_with_runtimes(
        runtime_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            enabled: false,
            runtime_ids: runtime_ids.into_iter().map(Into::into).collect(),
            default_runtime: None,
            allowed_callers: Vec::new(),
        }
    }
}

/// Per-service ACP defaults across CTO services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpServicesConfig {
    /// Healer ACP client/server settings.
    #[serde(default)]
    pub healer: AcpServiceConfig,

    /// PM ACP bridge settings.
    #[serde(default)]
    pub pm: AcpServiceConfig,

    /// Controller ACP runtime selection settings.
    #[serde(default)]
    pub controller: AcpServiceConfig,

    /// MCP ACP caller settings.
    #[serde(default)]
    pub mcp: AcpServiceConfig,

    /// MCP Lite ACP caller settings.
    #[serde(default, rename = "mcpLite")]
    pub mcp_lite: AcpServiceConfig,
}

impl Default for AcpServicesConfig {
    fn default() -> Self {
        Self {
            healer: AcpServiceConfig {
                runtime_ids: vec!["stakpak".to_string()],
                default_runtime: Some("stakpak".to_string()),
                allowed_callers: vec!["openclaw".to_string()],
                ..AcpServiceConfig::default()
            },
            pm: AcpServiceConfig::disabled_with_runtimes(["stakpak"]),
            controller: AcpServiceConfig::disabled_with_runtimes(["stakpak"]),
            mcp: AcpServiceConfig::disabled_with_runtimes(["stakpak"]),
            mcp_lite: AcpServiceConfig::disabled_with_runtimes(["stakpak"]),
        }
    }
}

fn default_acp_bind() -> String {
    "127.0.0.1:8890".to_string()
}

/// Shared ACP server defaults for internal-only services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpServerConfig {
    /// Whether the ACP server surface is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Bind address for internal-only ACP servers.
    #[serde(default = "default_acp_bind")]
    pub bind: String,

    /// Environment variable containing a bearer token for internal callers.
    #[serde(skip_serializing_if = "Option::is_none", rename = "authTokenEnv")]
    pub auth_token_env: Option<String>,

    /// Allowlisted caller IDs accepted by the server.
    #[serde(default, rename = "allowedCallers")]
    pub allowed_callers: Vec<String>,
}

impl Default for AcpServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: default_acp_bind(),
            auth_token_env: Some("CTO_ACP_SERVER_TOKEN".to_string()),
            allowed_callers: vec!["openclaw".to_string()],
        }
    }
}

/// Shared ACP defaults for CTO services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpDefaults {
    /// Global ACP feature flag for the workspace.
    #[serde(default)]
    pub enabled: bool,

    /// Default runtime ID when a service does not specify one explicitly.
    #[serde(skip_serializing_if = "Option::is_none", rename = "defaultRuntime")]
    pub default_runtime: Option<String>,

    /// Registered ACP runtimes.
    #[serde(default)]
    pub runtimes: HashMap<String, AcpRuntimeConfig>,

    /// Per-service ACP enablement and runtime policies.
    #[serde(default)]
    pub services: AcpServicesConfig,

    /// Shared ACP server settings for internal-only services.
    #[serde(default)]
    pub server: AcpServerConfig,
}

impl Default for AcpDefaults {
    fn default() -> Self {
        let mut runtimes = HashMap::new();
        runtimes.insert("stakpak".to_string(), AcpRuntimeConfig::default());

        Self {
            enabled: false,
            default_runtime: Some("stakpak".to_string()),
            runtimes,
            services: AcpServicesConfig::default(),
            server: AcpServerConfig::default(),
        }
    }
}

/// Default watcher check interval in seconds (2 minutes).
fn default_watcher_check_interval() -> u64 {
    120
}

/// Default watcher prompt template.
fn default_watcher_template() -> String {
    "watcher/base".to_string()
}

/// Default circuit breaker threshold.
fn default_circuit_breaker_threshold() -> u32 {
    3
}

/// Watcher configuration for dual-model execution pattern.
///
/// When enabled, a second "watcher" `CodeRun` is spawned alongside the executor
/// that monitors progress, detects issues, and writes them to a coordination
/// file for the executor to self-correct.
///
/// CLI-agnostic: supports any CLI (claude, codex, factory, droid, gemini, opencode, cursor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatcherDefaults {
    /// Enable watcher mode for play workflows.
    /// When true, a paired watcher `CodeRun` is created alongside the executor.
    #[serde(default)]
    pub enabled: bool,

    /// CLI to use for the watcher (e.g., "factory", "droid", "claude").
    /// Any supported CLI works - defaults to "factory" for cost efficiency.
    #[serde(default = "default_watcher_cli")]
    pub cli: String,

    /// Model to use for the watcher.
    /// Typically a cheaper model since watcher does monitoring, not code generation.
    #[serde(default = "default_watcher_model")]
    pub model: String,

    /// Interval between watcher checks in seconds.
    /// Default: 120 (2 minutes).
    #[serde(
        default = "default_watcher_check_interval",
        rename = "checkIntervalSecs"
    )]
    pub check_interval_secs: u64,

    /// Prompt template for the watcher.
    /// Default: "watcher/base".
    #[serde(default = "default_watcher_template")]
    pub template: String,

    /// Circuit breaker threshold - after this many failures on the same step,
    /// escalate to human intervention.
    /// Default: 3.
    #[serde(
        default = "default_circuit_breaker_threshold",
        rename = "circuitBreakerThreshold"
    )]
    pub circuit_breaker_threshold: u32,
}

fn default_watcher_cli() -> String {
    "factory".to_string()
}

fn default_watcher_model() -> String {
    "glm-4-plus".to_string()
}

impl Default for WatcherDefaults {
    fn default() -> Self {
        Self {
            enabled: false,
            cli: default_watcher_cli(),
            model: default_watcher_model(),
            check_interval_secs: default_watcher_check_interval(),
            template: default_watcher_template(),
            circuit_breaker_threshold: default_circuit_breaker_threshold(),
        }
    }
}

impl WatcherDefaults {
    /// Create an enabled watcher config with default settings.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    /// Create an enabled watcher config with a specific CLI and model.
    #[must_use]
    pub fn with_cli_and_model(cli: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            enabled: true,
            cli: cli.into(),
            model: model.into(),
            ..Self::default()
        }
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

    /// Skills configuration by job type.
    /// When present, these skills are used instead of skill-mappings.yaml.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<AgentSkills>,

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

    /// Per-agent watcher configuration override.
    /// When set, this agent will use these watcher settings instead of the global defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watcher: Option<WatcherDefaults>,
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

    /// Agent-to-agent communication mode used by Play workflows.
    #[serde(
        rename = "agentCommunication",
        default = "default_agent_communication_mode"
    )]
    pub agent_communication: AgentCommunicationMode,

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

    /// Watcher configuration for dual-model execution pattern.
    /// When enabled, a second `CodeRun` monitors the executor and provides real-time feedback.
    #[serde(default)]
    pub watcher: WatcherDefaults,
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

fn default_agent_communication_mode() -> AgentCommunicationMode {
    AgentCommunicationMode::Subagent
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
            agent_communication: default_agent_communication_mode(),
            healer_endpoint: None,
            fresh_start_threshold: default_fresh_start_threshold(),
            watcher: WatcherDefaults::default(),
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

/// Default max refinements for multi-model critic/validator pattern.
fn default_max_refinements() -> u32 {
    2
}

/// Default critic threshold (0.0-1.0).
fn default_critic_threshold() -> f32 {
    0.8
}

/// Default generator provider.
fn default_generator() -> String {
    "claude".to_string()
}

/// Default critic provider.
fn default_critic() -> String {
    "minimax".to_string()
}

/// Multi-model configuration for critic/validator collaboration pattern.
///
/// When enabled, intake uses a two-model approach:
/// - Generator (optimistic planner): Creates initial content
/// - Critic (pessimistic validator): Reviews and identifies issues
///
/// The generator refines content based on critic feedback until approved
/// or max refinements is reached.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiModelConfig {
    /// Enable multi-model collaboration.
    /// When false, uses single-model generation (default Claude).
    #[serde(default)]
    pub enabled: bool,

    /// Generator provider name (claude, minimax, codex).
    /// The optimistic model that produces initial content.
    #[serde(default = "default_generator")]
    pub generator: String,

    /// Critic provider name (claude, minimax, codex).
    /// The pessimistic model that reviews and validates content.
    #[serde(default = "default_critic")]
    pub critic: String,

    /// Maximum refinement iterations.
    /// After this many rounds, output is returned even if critic hasn't approved.
    #[serde(default = "default_max_refinements", rename = "maxRefinements")]
    pub max_refinements: u32,

    /// Critic confidence threshold (0.0-1.0).
    /// Content is approved when critic confidence exceeds this threshold.
    #[serde(default = "default_critic_threshold", rename = "criticThreshold")]
    pub critic_threshold: f32,
}

impl Default for MultiModelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generator: default_generator(),
            critic: default_critic(),
            max_refinements: default_max_refinements(),
            critic_threshold: default_critic_threshold(),
        }
    }
}

impl MultiModelConfig {
    /// Create an enabled multi-model config with default settings.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    /// Create an enabled config with custom generator and critic.
    #[must_use]
    pub fn with_providers(generator: impl Into<String>, critic: impl Into<String>) -> Self {
        Self {
            enabled: true,
            generator: generator.into(),
            critic: critic.into(),
            ..Self::default()
        }
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

    /// Multi-model critic/validator configuration.
    /// When enabled, uses a generator-critic pattern for higher quality output.
    #[serde(rename = "multiModel", default)]
    pub multi_model: MultiModelConfig,
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
            multi_model: MultiModelConfig::default(),
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

    /// Shared ACP defaults.
    #[serde(default)]
    pub acp: AcpDefaults,

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
pub const AGENT_ANGIE: &str = "Angie";

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

    #[test]
    fn test_watcher_defaults_default() {
        let watcher = WatcherDefaults::default();
        assert!(!watcher.enabled);
        assert_eq!(watcher.cli, "factory");
        assert_eq!(watcher.model, "glm-4-plus");
        assert_eq!(watcher.check_interval_secs, 120);
        assert_eq!(watcher.template, "watcher/base");
        assert_eq!(watcher.circuit_breaker_threshold, 3);
    }

    #[test]
    fn test_watcher_defaults_enabled() {
        let watcher = WatcherDefaults::enabled();
        assert!(watcher.enabled);
        assert_eq!(watcher.cli, "factory");
        assert_eq!(watcher.model, "glm-4-plus");
    }

    #[test]
    fn test_watcher_defaults_with_cli_and_model() {
        let watcher = WatcherDefaults::with_cli_and_model("claude", "claude-sonnet-4-20250514");
        assert!(watcher.enabled);
        assert_eq!(watcher.cli, "claude");
        assert_eq!(watcher.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_watcher_defaults_from_json() {
        let json = r#"{
            "enabled": true,
            "cli": "droid",
            "model": "gpt-4o-mini",
            "checkIntervalSecs": 60,
            "template": "watcher/custom",
            "circuitBreakerThreshold": 5
        }"#;
        let watcher: WatcherDefaults = serde_json::from_str(json).unwrap();
        assert!(watcher.enabled);
        assert_eq!(watcher.cli, "droid");
        assert_eq!(watcher.model, "gpt-4o-mini");
        assert_eq!(watcher.check_interval_secs, 60);
        assert_eq!(watcher.template, "watcher/custom");
        assert_eq!(watcher.circuit_breaker_threshold, 5);
    }

    #[test]
    fn test_play_defaults_with_watcher() {
        let json = r#"{
            "repository": "test-repo",
            "service": "test-service",
            "watcher": {
                "enabled": true,
                "cli": "factory",
                "model": "glm-4-plus"
            }
        }"#;
        let defaults: PlayDefaults = serde_json::from_str(json).unwrap();
        assert!(defaults.watcher.enabled);
        assert_eq!(defaults.watcher.cli, "factory");
    }

    #[test]
    fn test_agent_config_with_watcher_override() {
        let json = r#"{
            "githubApp": "5DLabs-Rex",
            "cli": "claude",
            "model": "claude-opus-4-5-20251101",
            "watcher": {
                "enabled": true,
                "cli": "droid",
                "model": "glm-4-plus",
                "checkIntervalSecs": 90
            }
        }"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.watcher.is_some());
        let watcher = config.watcher.unwrap();
        assert!(watcher.enabled);
        assert_eq!(watcher.cli, "droid");
        assert_eq!(watcher.check_interval_secs, 90);
    }

    #[test]
    fn test_multi_model_config_default() {
        let config = MultiModelConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.generator, "claude");
        assert_eq!(config.critic, "minimax");
        assert_eq!(config.max_refinements, 2);
        assert!((config.critic_threshold - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_multi_model_config_enabled() {
        let config = MultiModelConfig::enabled();
        assert!(config.enabled);
        assert_eq!(config.generator, "claude");
        assert_eq!(config.critic, "minimax");
    }

    #[test]
    fn test_multi_model_config_with_providers() {
        let config = MultiModelConfig::with_providers("codex", "claude");
        assert!(config.enabled);
        assert_eq!(config.generator, "codex");
        assert_eq!(config.critic, "claude");
    }

    #[test]
    fn test_multi_model_config_from_json() {
        let json = r#"{
            "enabled": true,
            "generator": "claude",
            "critic": "minimax",
            "maxRefinements": 3,
            "criticThreshold": 0.9
        }"#;
        let config: MultiModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.generator, "claude");
        assert_eq!(config.critic, "minimax");
        assert_eq!(config.max_refinements, 3);
        assert!((config.critic_threshold - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_intake_defaults_with_multi_model() {
        let json = r#"{
            "githubApp": "5DLabs-Morgan",
            "cli": "claude",
            "sourceBranch": "main",
            "models": {
                "primary": "claude-opus-4-5-20251101",
                "research": "claude-opus-4-5-20251101",
                "fallback": "claude-opus-4-5-20251101"
            },
            "multiModel": {
                "enabled": true,
                "generator": "claude",
                "critic": "minimax",
                "maxRefinements": 2,
                "criticThreshold": 0.8
            }
        }"#;
        let defaults: IntakeDefaults = serde_json::from_str(json).unwrap();
        assert!(defaults.multi_model.enabled);
        assert_eq!(defaults.multi_model.generator, "claude");
        assert_eq!(defaults.multi_model.critic, "minimax");
    }

    #[test]
    fn test_agent_skills_default() {
        let skills = AgentSkills::default();
        assert!(skills.default.is_empty());
        assert!(skills.coder.is_none());
        assert!(skills.healer.is_none());
        assert!(!skills.has_skills());
    }

    #[test]
    fn test_agent_skills_get_skills_for_job() {
        let skills = AgentSkills {
            default: vec![
                "context-fundamentals".to_string(),
                "rust-patterns".to_string(),
            ],
            coder: Some(vec![
                "tool-design".to_string(),
                "compound-engineering".to_string(),
            ]),
            healer: Some(vec!["incident-response".to_string()]),
            ..Default::default()
        };

        // Default skills only for unknown job type
        let unknown = skills.get_skills_for_job("unknown");
        assert_eq!(unknown.len(), 2);
        assert!(unknown.contains(&"context-fundamentals".to_string()));
        assert!(unknown.contains(&"rust-patterns".to_string()));

        // Coder job type merges with defaults
        let coder = skills.get_skills_for_job("coder");
        assert_eq!(coder.len(), 4);
        assert!(coder.contains(&"context-fundamentals".to_string()));
        assert!(coder.contains(&"rust-patterns".to_string()));
        assert!(coder.contains(&"tool-design".to_string()));
        assert!(coder.contains(&"compound-engineering".to_string()));

        // Healer job type merges with defaults
        let healer = skills.get_skills_for_job("healer");
        assert_eq!(healer.len(), 3);
        assert!(healer.contains(&"context-fundamentals".to_string()));
        assert!(healer.contains(&"incident-response".to_string()));
    }

    #[test]
    fn test_agent_skills_has_skills() {
        // Empty skills
        let empty = AgentSkills::default();
        assert!(!empty.has_skills());

        // With default skills
        let with_default = AgentSkills {
            default: vec!["some-skill".to_string()],
            ..Default::default()
        };
        assert!(with_default.has_skills());

        // With only job-type skills
        let with_coder = AgentSkills {
            coder: Some(vec!["coder-skill".to_string()]),
            ..Default::default()
        };
        assert!(with_coder.has_skills());
    }

    #[test]
    fn test_agent_skills_from_json() {
        let json = r#"{
            "default": ["context-fundamentals", "rust-patterns"],
            "coder": ["tool-design", "compound-engineering"],
            "healer": ["incident-response", "observability"]
        }"#;
        let skills: AgentSkills = serde_json::from_str(json).unwrap();
        assert_eq!(skills.default.len(), 2);
        assert!(skills.coder.is_some());
        assert_eq!(skills.coder.as_ref().unwrap().len(), 2);
        assert!(skills.healer.is_some());
        assert!(skills.quality.is_none());
    }

    #[test]
    fn test_agent_config_with_skills() {
        let json = r#"{
            "githubApp": "5DLabs-Rex",
            "cli": "claude",
            "model": "claude-opus-4-5-20251101",
            "skills": {
                "default": ["context-fundamentals", "rust-patterns"],
                "coder": ["tool-design"]
            }
        }"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.skills.is_some());
        let skills = config.skills.unwrap();
        assert_eq!(skills.default.len(), 2);
        assert!(skills.coder.is_some());
    }

    #[test]
    fn test_agent_config_without_skills() {
        let json = r#"{
            "githubApp": "5DLabs-Rex",
            "cli": "claude",
            "model": "claude-opus-4-5-20251101"
        }"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.skills.is_none());
    }
}
