use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Per-repo play configuration (`.tasks/play-config.yaml`)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayConfig {
    #[serde(default)]
    pub kubeconfig: KubeConfig,
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub defaults: PlayDefaults,
    #[serde(default)]
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KubeConfig {
    /// K8s namespace for CRD deployment
    #[serde(default = "default_namespace")]
    pub namespace: String,
    /// kubectl context name (empty = current context)
    #[serde(default)]
    pub context: String,
    /// Path to kubeconfig file (empty = default ~/.kube/config or in-cluster)
    #[serde(default)]
    pub path: String,
}

fn default_namespace() -> String {
    "cto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    /// Git repository URL
    #[serde(default)]
    pub repo_url: String,
    /// Service name for CRD
    #[serde(default)]
    pub service: String,
    /// Base branch
    #[serde(default = "default_branch")]
    pub base_branch: String,
    /// Docs repository URL (if separate)
    #[serde(default)]
    pub docs_repository_url: String,
}

fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayDefaults {
    /// Inference provider (e.g. "fireworks")
    #[serde(default)]
    pub provider: String,
    /// Default model
    #[serde(default)]
    pub model: String,
    /// Coding CLI (e.g. "claude", "codex")
    #[serde(default)]
    pub cli: String,
    /// Harness agent ("openclaw" or "hermes")
    #[serde(default)]
    pub harness_agent: String,
    /// GitHub App prefix
    #[serde(default)]
    pub github_app_prefix: String,
    /// Enable Docker in CodeRun pods
    #[serde(default = "default_true")]
    pub enable_docker: bool,
    /// Quality gate
    #[serde(default = "default_true")]
    pub quality: bool,
    /// Security gate
    #[serde(default = "default_true")]
    pub security: bool,
    /// Testing gate
    #[serde(default = "default_true")]
    pub testing: bool,
    /// Deployment gate
    #[serde(default)]
    pub deployment: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bridge_url: String,
}

/// CTO-level config subset (from cto-config.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CtoConfig {
    #[serde(default)]
    pub defaults: Option<CtoDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CtoDefaults {
    #[serde(default)]
    pub play: Option<CtoPlayDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CtoPlayDefaults {
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub service: String,
    #[serde(default)]
    pub auto_merge: Option<bool>,
    #[serde(default)]
    pub quality: Option<bool>,
    #[serde(default)]
    pub security: Option<bool>,
    #[serde(default)]
    pub testing: Option<bool>,
    #[serde(default)]
    pub deployment: Option<bool>,
}

impl PlayConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

impl CtoConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
}

/// Build the merged args-json for lobster run
pub fn build_args_json(
    play_config: &PlayConfig,
    cto_config: &Option<CtoConfig>,
    overrides: &HashMap<String, String>,
) -> serde_json::Value {
    let pc = play_config;
    let cto_play = cto_config
        .as_ref()
        .and_then(|c| c.defaults.as_ref())
        .and_then(|d| d.play.as_ref());

    // Helper: override > play-config > cto-default > fallback
    let resolve_str = |key: &str, play_val: &str, cto_val: Option<&str>, fallback: &str| -> String {
        if let Some(v) = overrides.get(key) {
            return v.clone();
        }
        if !play_val.is_empty() {
            return play_val.to_string();
        }
        if let Some(v) = cto_val {
            if !v.is_empty() {
                return v.to_string();
            }
        }
        fallback.to_string()
    };

    let repo_url = resolve_str(
        "repo_url",
        &pc.project.repo_url,
        cto_play.map(|c| c.repository.as_str()),
        "",
    );
    let namespace = resolve_str("namespace", &pc.kubeconfig.namespace, None, "cto");
    let base_branch = resolve_str("base_branch", &pc.project.base_branch, None, "main");
    let cli = resolve_str("cli", &pc.defaults.cli, None, "claude");
    let provider = resolve_str("provider", &pc.defaults.provider, None, "fireworks");
    let model = resolve_str(
        "model",
        &pc.defaults.model,
        None,
        "accounts/fireworks/routers/kimi-k2p5-turbo",
    );
    let harness_agent = resolve_str("harness_agent", &pc.defaults.harness_agent, None, "openclaw");
    let github_app_prefix = resolve_str(
        "github_app_prefix",
        &pc.defaults.github_app_prefix,
        None,
        "5DLabs",
    );
    let _service = resolve_str(
        "service",
        &pc.project.service,
        cto_play.map(|c| c.service.as_str()),
        "",
    );
    let docs_repo = resolve_str(
        "docs_repository_url",
        &pc.project.docs_repository_url,
        None,
        "",
    );
    let discord_enabled = if overrides.contains_key("discord_enabled") {
        overrides["discord_enabled"] == "true"
    } else {
        pc.discord.enabled
    };
    let discord_bridge_url = resolve_str("discord_bridge_url", &pc.discord.bridge_url, None, "");

    serde_json::json!({
        "repo_url": repo_url,
        "namespace": namespace,
        "base_branch": base_branch,
        "cli": cli,
        "provider": provider,
        "model": model,
        "harness_agent": harness_agent,
        "github_app_prefix": github_app_prefix,
        "enable_docker": if pc.defaults.enable_docker { "true" } else { "false" },
        "linear_session_id": overrides.get("linear_session_id").cloned().unwrap_or_default(),
        "linear_team_id": overrides.get("linear_team_id").cloned().unwrap_or_default(),
        "docs_repository_url": docs_repo,
        "discord_enabled": if discord_enabled { "true" } else { "false" },
        "discord_bridge_url": discord_bridge_url,
    })
}
