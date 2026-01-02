//! Configuration for the PM service.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// PM webhook handler configuration.
#[derive(Clone)]
pub struct Config {
    /// HTTP server port.
    pub port: u16,
    /// Webhook signing secret for signature verification (legacy single-app).
    pub webhook_secret: Option<String>,
    /// Maximum age for webhook timestamps (default: 60 seconds).
    pub max_timestamp_age_ms: i64,
    /// Whether Linear integration is enabled.
    pub enabled: bool,
    /// Linear OAuth token for API calls (legacy single-app).
    pub oauth_token: Option<String>,
    /// Kubernetes namespace.
    pub namespace: String,
    /// Intake configuration.
    pub intake: IntakeConfig,
    /// Play configuration.
    pub play: PlayConfig,
    /// Multi-agent Linear OAuth configuration.
    pub linear: LinearConfig,
}

// =============================================================================
// Multi-Agent Linear OAuth Configuration
// =============================================================================

/// All known agent names for the CTO platform.
pub const AGENT_NAMES: &[&str] = &[
    "morgan", "rex", "blaze", "grizz", "nova", "tap", "spark", "cleo", "cipher", "tess", "atlas",
    "bolt",
];

/// Configuration for a single Linear OAuth application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinearAppConfig {
    /// OAuth Client ID (32 hex characters).
    pub client_id: String,
    /// OAuth Client Secret (32 hex characters).
    #[serde(skip_serializing)]
    pub client_secret: String,
    /// Webhook signing secret for this app.
    #[serde(skip_serializing)]
    pub webhook_secret: String,
    /// OAuth access token (obtained after installation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Human-readable display name (e.g., "5DLabs-Rex").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl LinearAppConfig {
    /// Create a new app config with the given credentials.
    #[must_use]
    pub fn new(client_id: String, client_secret: String, webhook_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            webhook_secret,
            access_token: None,
            display_name: None,
        }
    }

    /// Check if this app has been installed (has an access token).
    #[must_use]
    pub fn is_installed(&self) -> bool {
        self.access_token.is_some()
    }

    /// Check if this app has all required credentials.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.client_id.is_empty()
            && !self.client_secret.is_empty()
            && !self.webhook_secret.is_empty()
    }
}

/// Configuration for all Linear OAuth applications.
#[derive(Debug, Clone, Default)]
pub struct LinearConfig {
    /// Map of agent name to app configuration.
    pub apps: HashMap<String, LinearAppConfig>,
    /// Common webhook URL for all apps.
    pub webhook_url: String,
    /// Common OAuth redirect URI.
    pub redirect_uri: String,
}

impl LinearConfig {
    /// Load configuration from environment variables.
    ///
    /// Environment variable format:
    /// - `LINEAR_APP_{AGENT}_CLIENT_ID`
    /// - `LINEAR_APP_{AGENT}_CLIENT_SECRET`
    /// - `LINEAR_APP_{AGENT}_WEBHOOK_SECRET`
    /// - `LINEAR_APP_{AGENT}_ACCESS_TOKEN` (optional)
    #[must_use]
    pub fn from_env() -> Self {
        let mut apps = HashMap::new();

        for agent in AGENT_NAMES {
            let upper = agent.to_uppercase();
            let client_id = env::var(format!("LINEAR_APP_{upper}_CLIENT_ID")).unwrap_or_default();
            let client_secret =
                env::var(format!("LINEAR_APP_{upper}_CLIENT_SECRET")).unwrap_or_default();
            let webhook_secret =
                env::var(format!("LINEAR_APP_{upper}_WEBHOOK_SECRET")).unwrap_or_default();
            let access_token = env::var(format!("LINEAR_APP_{upper}_ACCESS_TOKEN")).ok();

            if !client_id.is_empty() {
                let mut config = LinearAppConfig::new(client_id, client_secret, webhook_secret);
                config.access_token = access_token;
                config.display_name = Some(format!("5DLabs-{}", capitalize(agent)));
                apps.insert((*agent).to_string(), config);
            }
        }

        Self {
            apps,
            webhook_url: env::var("LINEAR_WEBHOOK_URL")
                .unwrap_or_else(|_| "https://cto.5dlabs.ai/webhooks/linear".to_string()),
            redirect_uri: env::var("LINEAR_REDIRECT_URI")
                .unwrap_or_else(|_| "https://cto.5dlabs.ai/oauth/callback".to_string()),
        }
    }

    /// Get app configuration by agent name.
    #[must_use]
    pub fn get_app(&self, agent: &str) -> Option<&LinearAppConfig> {
        self.apps.get(&agent.to_lowercase())
    }

    /// Get mutable app configuration by agent name.
    pub fn get_app_mut(&mut self, agent: &str) -> Option<&mut LinearAppConfig> {
        self.apps.get_mut(&agent.to_lowercase())
    }

    /// Find which agent app has the given webhook secret.
    ///
    /// This is used to identify which agent sent a webhook by validating
    /// the signature against each app's webhook secret.
    #[must_use]
    pub fn find_agent_by_webhook_secret(&self, secret: &str) -> Option<&str> {
        for (agent, config) in &self.apps {
            if config.webhook_secret == secret {
                return Some(agent.as_str());
            }
        }
        None
    }

    /// Get the access token for an agent, if installed.
    #[must_use]
    pub fn get_access_token(&self, agent: &str) -> Option<&str> {
        self.get_app(agent)
            .and_then(|app| app.access_token.as_deref())
    }

    /// Check if all apps are configured (have credentials).
    #[must_use]
    pub fn all_configured(&self) -> bool {
        AGENT_NAMES.iter().all(|agent| {
            self.get_app(agent)
                .is_some_and(LinearAppConfig::is_configured)
        })
    }

    /// Check if all apps are installed (have access tokens).
    #[must_use]
    pub fn all_installed(&self) -> bool {
        AGENT_NAMES.iter().all(|agent| {
            self.get_app(agent)
                .is_some_and(LinearAppConfig::is_installed)
        })
    }

    /// Get list of configured agent names.
    #[must_use]
    pub fn configured_agents(&self) -> Vec<&str> {
        self.apps
            .iter()
            .filter(|(_, config)| config.is_configured())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get list of installed agent names.
    #[must_use]
    pub fn installed_agents(&self) -> Vec<&str> {
        self.apps
            .iter()
            .filter(|(_, config)| config.is_installed())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Generate OAuth authorization URL for an agent.
    #[must_use]
    pub fn oauth_url(&self, agent: &str) -> Option<String> {
        self.get_app(agent).map(|app| {
            format!(
                "https://linear.app/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read,write,app:assignable,app:mentionable&actor=app",
                app.client_id,
                urlencoding::encode(&self.redirect_uri)
            )
        })
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: env::var("LINEAR_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8081),
            webhook_secret: env::var("LINEAR_WEBHOOK_SECRET")
                .ok()
                .filter(|s| !s.is_empty()),
            max_timestamp_age_ms: env::var("LINEAR_MAX_TIMESTAMP_AGE_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60_000),
            enabled: env::var("LINEAR_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            oauth_token: env::var("LINEAR_OAUTH_TOKEN").ok(),
            namespace: env::var("NAMESPACE").unwrap_or_else(|_| "cto".to_string()),
            intake: IntakeConfig::default(),
            play: PlayConfig::default(),
            linear: LinearConfig::from_env(),
        }
    }
}

/// Intake workflow configuration.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct IntakeConfig {
    /// Intake mode: "api" (direct API calls) or "cli" (use AI CLI).
    pub mode: String,
    /// Container image for intake workflows.
    pub runtime_image: String,
    /// GitHub App for authentication.
    pub github_app: String,
    /// Primary model for task generation.
    pub primary_model: String,
    /// Primary provider (anthropic, openai, etc).
    pub primary_provider: String,
    /// Research model for analysis.
    pub research_model: String,
    /// Research provider.
    pub research_provider: String,
    /// Fallback model.
    pub fallback_model: String,
    /// Fallback provider.
    pub fallback_provider: String,
    /// Docs model for documentation generation.
    pub docs_model: String,
    /// Number of tasks to generate.
    pub num_tasks: i32,
    /// Whether to expand tasks with subtasks.
    pub expand_tasks: bool,
    /// Whether to analyze complexity.
    pub analyze_complexity: bool,
    /// Whether to enrich context via Firecrawl.
    pub enrich_context: bool,
    /// Whether to include codebase in context.
    pub include_codebase: bool,
    /// CLI for documentation generation (claude, cursor, codex).
    pub cli: String,
    /// Webhook callback URL for GitHub webhooks (e.g., PR merge notifications).
    /// If not set, GitHub webhook setup will be skipped.
    pub webhook_callback_url: Option<String>,
    /// Default GitHub organization for creating new repositories.
    /// Used when no repository URL is provided in the intake request.
    pub github_default_org: Option<String>,
}

impl Default for IntakeConfig {
    fn default() -> Self {
        Self {
            // Intake mode: "api" (direct API calls via tasks CLI) or "cli" (use AI CLI)
            mode: env::var("INTAKE_MODE").unwrap_or_else(|_| "api".to_string()),
            // Container image for intake workflows
            runtime_image: env::var("RUNTIME_IMAGE")
                .unwrap_or_else(|_| "ghcr.io/5dlabs/runtime:latest".to_string()),
            github_app: env::var("GITHUB_APP_NAME").unwrap_or_else(|_| "cto-dev".to_string()),
            // Opus 4.5 with extended thinking for complex intake tasks
            primary_model: env::var("PRIMARY_MODEL")
                .unwrap_or_else(|_| "claude-opus-4-5-20251101".to_string()),
            primary_provider: env::var("PRIMARY_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
            research_model: env::var("RESEARCH_MODEL")
                .unwrap_or_else(|_| "claude-opus-4-5-20251101".to_string()),
            research_provider: env::var("RESEARCH_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
            // Fallback to Sonnet 4 (faster, cheaper) if Opus fails
            fallback_model: env::var("FALLBACK_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            fallback_provider: env::var("FALLBACK_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
            docs_model: env::var("DOCS_MODEL")
                .unwrap_or_else(|_| "claude-opus-4-5-20251101".to_string()),
            num_tasks: env::var("NUM_TASKS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            expand_tasks: true,
            analyze_complexity: true,
            enrich_context: true,
            include_codebase: false,
            cli: env::var("INTAKE_CLI").unwrap_or_else(|_| "claude".to_string()),
            // These should be configured via environment variables for each deployment
            webhook_callback_url: env::var("WEBHOOK_CALLBACK_URL").ok(),
            github_default_org: env::var("GITHUB_DEFAULT_ORG").ok(),
        }
    }
}

/// Play workflow configuration.
#[derive(Debug, Clone)]
pub struct PlayConfig {
    /// GitHub App for authentication.
    pub github_app: String,
    /// Default repository URL.
    pub repository: Option<String>,
    /// Default docs project directory.
    pub docs_project_directory: Option<String>,
    /// Implementation agent (rex for Rust).
    pub implementation_agent: String,
    /// Frontend agent (blaze for React/TS).
    pub frontend_agent: String,
    /// Go agent (grizz for Go/gRPC).
    pub go_agent: Option<String>,
    /// Node agent (nova for Node.js/Bun).
    pub node_agent: Option<String>,
    /// Mobile agent (tap for Expo/React Native).
    pub mobile_agent: Option<String>,
    /// Desktop agent (spark for Electron).
    pub desktop_agent: Option<String>,
    /// Testing agent.
    pub testing_agent: String,
    /// Quality agent.
    pub quality_agent: String,
    /// Security agent.
    pub security_agent: String,
    /// Primary model.
    pub model: String,
    /// Whether to use parallel execution.
    pub parallel_execution: bool,
    /// Whether to auto-merge PRs.
    pub auto_merge: bool,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            github_app: env::var("GITHUB_APP_NAME").unwrap_or_else(|_| "cto-dev".to_string()),
            repository: env::var("DEFAULT_REPOSITORY").ok(),
            docs_project_directory: env::var("DOCS_PROJECT_DIRECTORY").ok(),
            implementation_agent: env::var("IMPLEMENTATION_AGENT")
                .unwrap_or_else(|_| "5DLabs-Rex".to_string()),
            frontend_agent: env::var("FRONTEND_AGENT")
                .unwrap_or_else(|_| "5DLabs-Blaze".to_string()),
            go_agent: env::var("GO_AGENT").ok(),
            node_agent: env::var("NODE_AGENT").ok(),
            mobile_agent: env::var("MOBILE_AGENT").ok(),
            desktop_agent: env::var("DESKTOP_AGENT").ok(),
            testing_agent: env::var("TESTING_AGENT").unwrap_or_else(|_| "5DLabs-Tess".to_string()),
            quality_agent: env::var("QUALITY_AGENT").unwrap_or_else(|_| "5DLabs-Cleo".to_string()),
            security_agent: env::var("SECURITY_AGENT")
                .unwrap_or_else(|_| "5DLabs-Cipher".to_string()),
            model: env::var("PLAY_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            parallel_execution: env::var("PARALLEL_EXECUTION")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            auto_merge: env::var("AUTO_MERGE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        }
    }
}

// =========================================================================
// CTO Issue-Level Configuration
// =========================================================================

/// CTO platform configuration extracted from Linear issue labels/frontmatter.
///
/// This configuration allows per-issue overrides of CLI and model settings.
/// Resolution order: Description frontmatter > Labels > Environment defaults
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CtoConfig {
    /// CLI to use (claude, cursor, codex, dexter, opencode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    /// Model to use (e.g., claude-sonnet-4-20250514, claude-opus-4-20250514).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl CtoConfig {
    /// Create a new empty CTO config.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if config has any values set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cli.is_none() && self.model.is_none()
    }

    /// Merge another config into this one (other takes precedence).
    pub fn merge(&mut self, other: &Self) {
        if other.cli.is_some() {
            self.cli.clone_from(&other.cli);
        }
        if other.model.is_some() {
            self.model.clone_from(&other.model);
        }
    }
}

/// Source of a configuration value (for logging/debugging).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Value from environment/default configuration.
    Default,
    /// Value from Linear issue label.
    Label,
    /// Value from issue description frontmatter.
    Frontmatter,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Use a mutex to serialize tests that modify environment variables
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_default_config() {
        let _lock = ENV_MUTEX.lock().unwrap();

        // Clear env vars for test
        env::remove_var("LINEAR_WEBHOOK_SECRET");
        env::remove_var("LINEAR_ENABLED");
        env::remove_var("LINEAR_PORT");
        env::remove_var("LINEAR_MAX_TIMESTAMP_AGE_MS");

        let config = Config::default();
        assert!(!config.enabled);
        assert!(config.webhook_secret.is_none());
        assert_eq!(config.max_timestamp_age_ms, 60_000);
        assert_eq!(config.port, 8081);
    }

    #[test]
    fn test_config_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();

        // Clean first
        env::remove_var("LINEAR_ENABLED");
        env::remove_var("LINEAR_WEBHOOK_SECRET");
        env::remove_var("LINEAR_PORT");

        env::set_var("LINEAR_ENABLED", "true");
        env::set_var("LINEAR_WEBHOOK_SECRET", "test-secret");
        env::set_var("LINEAR_PORT", "9000");

        let config = Config::default();
        assert!(config.enabled);
        assert_eq!(config.webhook_secret, Some("test-secret".to_string()));
        assert_eq!(config.port, 9000);

        // Clean up
        env::remove_var("LINEAR_ENABLED");
        env::remove_var("LINEAR_WEBHOOK_SECRET");
        env::remove_var("LINEAR_PORT");
    }

    // =========================================================================
    // CtoConfig Tests
    // =========================================================================

    #[test]
    fn test_cto_config_default() {
        let config = CtoConfig::default();
        assert!(config.cli.is_none());
        assert!(config.model.is_none());
        assert!(config.is_empty());
    }

    #[test]
    fn test_cto_config_is_empty() {
        let empty = CtoConfig::default();
        assert!(empty.is_empty());

        let with_cli = CtoConfig {
            cli: Some("claude".to_string()),
            model: None,
        };
        assert!(!with_cli.is_empty());

        let with_model = CtoConfig {
            cli: None,
            model: Some("opus".to_string()),
        };
        assert!(!with_model.is_empty());
    }

    #[test]
    fn test_cto_config_merge() {
        let mut base = CtoConfig {
            cli: Some("claude".to_string()),
            model: Some("sonnet".to_string()),
        };

        let override_config = CtoConfig {
            cli: Some("cursor".to_string()),
            model: None,
        };

        base.merge(&override_config);

        assert_eq!(base.cli, Some("cursor".to_string()));
        assert_eq!(base.model, Some("sonnet".to_string())); // unchanged
    }

    #[test]
    fn test_cto_config_merge_empty() {
        let mut base = CtoConfig {
            cli: Some("claude".to_string()),
            model: Some("sonnet".to_string()),
        };

        let empty = CtoConfig::default();
        base.merge(&empty);

        assert_eq!(base.cli, Some("claude".to_string()));
        assert_eq!(base.model, Some("sonnet".to_string()));
    }

    #[test]
    fn test_cto_config_serialize() {
        let config = CtoConfig {
            cli: Some("cursor".to_string()),
            model: Some("opus".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("cursor"));
        assert!(json.contains("opus"));
    }

    #[test]
    fn test_cto_config_deserialize() {
        let json = r#"{"cli": "codex", "model": "gpt-4.1"}"#;
        let config: CtoConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.cli, Some("codex".to_string()));
        assert_eq!(config.model, Some("gpt-4.1".to_string()));
    }

    // =========================================================================
    // LinearAppConfig Tests
    // =========================================================================

    #[test]
    fn test_linear_app_config_new() {
        let config = LinearAppConfig::new(
            "abc123".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        assert_eq!(config.client_id, "abc123");
        assert_eq!(config.client_secret, "secret");
        assert_eq!(config.webhook_secret, "webhook");
        assert!(config.access_token.is_none());
        assert!(!config.is_installed());
        assert!(config.is_configured());
    }

    #[test]
    fn test_linear_app_config_is_installed() {
        let mut config = LinearAppConfig::new(
            "abc123".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        assert!(!config.is_installed());

        config.access_token = Some("token".to_string());
        assert!(config.is_installed());
    }

    #[test]
    fn test_linear_app_config_is_configured() {
        let empty = LinearAppConfig::default();
        assert!(!empty.is_configured());

        let partial = LinearAppConfig {
            client_id: "abc".to_string(),
            ..Default::default()
        };
        assert!(!partial.is_configured());

        let complete = LinearAppConfig::new(
            "abc".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        assert!(complete.is_configured());
    }

    // =========================================================================
    // LinearConfig Tests
    // =========================================================================

    #[test]
    fn test_linear_config_get_app() {
        let mut config = LinearConfig::default();
        config.apps.insert(
            "rex".to_string(),
            LinearAppConfig::new(
                "id".to_string(),
                "secret".to_string(),
                "webhook".to_string(),
            ),
        );

        assert!(config.get_app("rex").is_some());
        assert!(config.get_app("Rex").is_some()); // case-insensitive
        assert!(config.get_app("REX").is_some());
        assert!(config.get_app("unknown").is_none());
    }

    #[test]
    fn test_linear_config_find_agent_by_webhook_secret() {
        let mut config = LinearConfig::default();
        config.apps.insert(
            "rex".to_string(),
            LinearAppConfig::new(
                "id1".to_string(),
                "s1".to_string(),
                "webhook_rex".to_string(),
            ),
        );
        config.apps.insert(
            "blaze".to_string(),
            LinearAppConfig::new(
                "id2".to_string(),
                "s2".to_string(),
                "webhook_blaze".to_string(),
            ),
        );

        assert_eq!(
            config.find_agent_by_webhook_secret("webhook_rex"),
            Some("rex")
        );
        assert_eq!(
            config.find_agent_by_webhook_secret("webhook_blaze"),
            Some("blaze")
        );
        assert_eq!(config.find_agent_by_webhook_secret("unknown"), None);
    }

    #[test]
    fn test_linear_config_oauth_url() {
        let mut config = LinearConfig {
            redirect_uri: "https://example.com/callback".to_string(),
            ..Default::default()
        };
        config.apps.insert(
            "rex".to_string(),
            LinearAppConfig::new(
                "client123".to_string(),
                "secret".to_string(),
                "webhook".to_string(),
            ),
        );

        let url = config.oauth_url("rex").unwrap();
        assert!(url.contains("client_id=client123"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("actor=app"));
        assert!(url.contains("app:assignable"));
        assert!(url.contains("app:mentionable"));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(super::capitalize("rex"), "Rex");
        assert_eq!(super::capitalize("blaze"), "Blaze");
        assert_eq!(super::capitalize(""), "");
        assert_eq!(super::capitalize("a"), "A");
    }
}
