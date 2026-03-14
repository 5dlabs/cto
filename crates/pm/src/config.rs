//! Configuration for the PM service.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

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
    /// Linear team ID for intake setup (e.g., "CTOPA").
    pub linear_team_id: Option<String>,
    /// Intake configuration.
    pub intake: IntakeConfig,
    /// Play configuration.
    pub play: PlayConfig,
    /// Multi-agent Linear OAuth configuration.
    ///
    /// Wrapped in `Arc<RwLock<>>` to allow updating tokens after OAuth refresh
    /// while sharing the config across axum request handlers.
    pub linear: Arc<RwLock<LinearConfig>>,
    /// Skip webhook signature verification (for local development only).
    pub skip_signature_verification: bool,
    /// GitHub webhook secret for HMAC-SHA256 signature verification.
    pub github_webhook_secret: Option<String>,
}

// =============================================================================
// Multi-Agent Linear OAuth Configuration
// =============================================================================

/// All known agent names for the CTO platform.
pub const AGENT_NAMES: &[&str] = &[
    "morgan", "rex", "blaze", "grizz", "nova", "tap", "spark", "cleo", "cipher", "tess", "atlas",
    "bolt", "stitch", "vex", "angie",
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
    /// OAuth refresh token (for obtaining new access tokens).
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    /// Unix timestamp when the access token expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
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
            refresh_token: None,
            expires_at: None,
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

    /// Check if the access token has expired.
    ///
    /// Returns true if:
    /// - There is an `expires_at` timestamp and it's in the past (or within 1 hour buffer)
    /// - There is no `expires_at` (assume expired to be safe)
    ///
    /// Returns false if there's no access token at all.
    #[must_use]
    pub fn is_token_expired(&self) -> bool {
        if self.access_token.is_none() {
            return false; // No token to expire
        }

        match self.expires_at {
            Some(expires_at) => {
                let now = Utc::now().timestamp();
                // Consider expired if less than 1 hour remaining
                expires_at - now < 3600
            }
            None => true, // No expiration info, assume expired to be safe
        }
    }

    /// Check if the token can be refreshed.
    #[must_use]
    pub fn can_refresh(&self) -> bool {
        self.refresh_token.is_some() && !self.client_id.is_empty() && !self.client_secret.is_empty()
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
    /// - `LINEAR_APP_{AGENT}_REFRESH_TOKEN` (optional)
    /// - `LINEAR_APP_{AGENT}_EXPIRES_AT` (optional, Unix timestamp)
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
            // Try standard env var first, then fallback to _DIRECT (for bypassing ExternalSecrets)
            // Note: We filter empty strings because ExternalSecrets may set empty values
            let access_token = env::var(format!("LINEAR_APP_{upper}_ACCESS_TOKEN"))
                .ok()
                .filter(|s| !s.is_empty())
                .or_else(|| env::var(format!("LINEAR_APP_{upper}_ACCESS_TOKEN_DIRECT")).ok())
                .filter(|s| !s.is_empty());

            // Load refresh token if available
            let refresh_token = env::var(format!("LINEAR_APP_{upper}_REFRESH_TOKEN"))
                .ok()
                .filter(|s| !s.is_empty());

            // Load expiration timestamp if available
            let expires_at = env::var(format!("LINEAR_APP_{upper}_EXPIRES_AT"))
                .ok()
                .and_then(|s| s.parse::<i64>().ok());

            if !client_id.is_empty() {
                let mut config = LinearAppConfig::new(client_id, client_secret, webhook_secret);
                config.access_token = access_token;
                config.refresh_token = refresh_token;
                config.expires_at = expires_at;
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
    ///
    /// Uses `prompt=consent` to force the consent screen even if previously authorized,
    /// ensuring we always get a callback with a fresh authorization code.
    ///
    /// Uses `actor=app` to enable Actor authorization, which makes the app perform
    /// actions as itself (the Agent app identity) rather than as the authorizing user.
    #[must_use]
    pub fn oauth_url(&self, agent: &str) -> Option<String> {
        self.get_app(agent).map(|app| {
            format!(
                "https://linear.app/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read,write&prompt=consent&actor=app",
                app.client_id,
                urlencoding::encode(&self.redirect_uri)
            )
        })
    }

    /// Update tokens for an agent after a successful OAuth token refresh.
    ///
    /// This updates the in-memory config to reflect the new tokens, ensuring
    /// subsequent calls see the fresh token data and don't try to refresh again
    /// with stale/rotated refresh tokens.
    ///
    /// # Arguments
    /// * `agent` - Agent name (e.g., "morgan", "rex")
    /// * `access_token` - New access token
    /// * `refresh_token` - New refresh token (may be rotated by Linear)
    /// * `expires_in` - Token lifetime in seconds (optional)
    pub fn update_tokens(
        &mut self,
        agent: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_in: Option<i64>,
    ) {
        if let Some(app) = self.apps.get_mut(&agent.to_lowercase()) {
            app.access_token = Some(access_token.to_string());
            if let Some(rt) = refresh_token {
                app.refresh_token = Some(rt.to_string());
            }
            // Always update expires_at: calculate new value if expires_in is provided,
            // otherwise clear it to avoid preserving stale expiration timestamps from
            // previous tokens that may already be expired.
            app.expires_at = expires_in.map(|secs| Utc::now().timestamp() + secs);
        }
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
            oauth_token: env::var("LINEAR_OAUTH_TOKEN")
                .or_else(|_| env::var("LINEAR_API_KEY"))
                .ok(),
            namespace: env::var("NAMESPACE").unwrap_or_else(|_| "cto".to_string()),
            linear_team_id: env::var("LINEAR_TEAM_ID").ok().filter(|s| !s.is_empty()),
            intake: IntakeConfig::default(),
            play: PlayConfig::default(),
            linear: Arc::new(RwLock::new(LinearConfig::from_env())),
            skip_signature_verification: env::var("LINEAR_SKIP_SIGNATURE_VERIFICATION")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            github_webhook_secret: env::var("GITHUB_WEBHOOK_SECRET")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}

/// Intake workflow configuration.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Config struct with boolean feature toggles by design
pub struct IntakeConfig {
    /// Container image for intake workflows (derived from CLI selection).
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
    /// Whether to enable extended thinking for better task generation.
    /// When enabled, the AI model will use extended thinking capabilities.
    pub extended_thinking: bool,
    /// Budget for extended thinking in tokens.
    /// Only used when `extended_thinking` is true.
    pub thinking_budget: Option<i32>,
}

impl Default for IntakeConfig {
    fn default() -> Self {
        Self {
            // Container image for intake workflows (derived from CLI selection)
            runtime_image: env::var("RUNTIME_IMAGE")
                .unwrap_or_else(|_| "ghcr.io/5dlabs/claude:latest".to_string()),
            // Morgan is the designated intake/PM agent
            github_app: env::var("GITHUB_APP_NAME").unwrap_or_else(|_| "5DLabs-Morgan".to_string()),
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
            // Extended thinking is enabled by default for complex intake tasks
            extended_thinking: env::var("TASKS_EXTENDED_THINKING")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            thinking_budget: env::var("TASKS_THINKING_BUDGET")
                .ok()
                .and_then(|s| s.parse().ok()),
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
    /// Infrastructure agent (bolt for DevOps/K8s).
    pub infrastructure_agent: Option<String>,
    /// Testing agent.
    pub testing_agent: String,
    /// Quality agent.
    pub quality_agent: String,
    /// Security agent.
    pub security_agent: String,
    /// Primary model (deprecated - use agent-specific model config).
    pub model: Option<String>,
    /// Whether to use parallel execution.
    pub parallel_execution: bool,
    /// Whether to auto-merge PRs.
    pub auto_merge: bool,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            // Rex is the primary implementation agent for play workflows
            github_app: env::var("GITHUB_APP_NAME").unwrap_or_else(|_| "5DLabs-Rex".to_string()),
            repository: env::var("DEFAULT_REPOSITORY").ok(),
            docs_project_directory: env::var("DOCS_PROJECT_DIRECTORY").ok(),
            implementation_agent: env::var("IMPLEMENTATION_AGENT")
                .unwrap_or_else(|_| "5DLabs-Rex".to_string()),
            frontend_agent: env::var("FRONTEND_AGENT")
                .unwrap_or_else(|_| "5DLabs-Blaze".to_string()),
            go_agent: Some(env::var("GO_AGENT").unwrap_or_else(|_| "5DLabs-Grizz".to_string())),
            node_agent: Some(env::var("NODE_AGENT").unwrap_or_else(|_| "5DLabs-Nova".to_string())),
            mobile_agent: Some(
                env::var("MOBILE_AGENT").unwrap_or_else(|_| "5DLabs-Tap".to_string()),
            ),
            desktop_agent: Some(
                env::var("DESKTOP_AGENT").unwrap_or_else(|_| "5DLabs-Spark".to_string()),
            ),
            infrastructure_agent: Some(
                env::var("INFRASTRUCTURE_AGENT").unwrap_or_else(|_| "5DLabs-Bolt".to_string()),
            ),
            testing_agent: env::var("TESTING_AGENT").unwrap_or_else(|_| "5DLabs-Tess".to_string()),
            quality_agent: env::var("QUALITY_AGENT").unwrap_or_else(|_| "5DLabs-Cleo".to_string()),
            security_agent: env::var("SECURITY_AGENT")
                .unwrap_or_else(|_| "5DLabs-Cipher".to_string()),
            model: env::var("PLAY_MODEL").ok(),
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
    /// Prompt style variant (e.g., "minimal" for Ralph-style prompts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_style: Option<String>,
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
        self.cli.is_none() && self.model.is_none() && self.prompt_style.is_none()
    }

    /// Merge another config into this one (other takes precedence).
    pub fn merge(&mut self, other: &Self) {
        if other.cli.is_some() {
            self.cli.clone_from(&other.cli);
        }
        if other.model.is_some() {
            self.model.clone_from(&other.model);
        }
        if other.prompt_style.is_some() {
            self.prompt_style.clone_from(&other.prompt_style);
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
            prompt_style: None,
        };
        assert!(!with_cli.is_empty());

        let with_model = CtoConfig {
            cli: None,
            model: Some("opus".to_string()),
            prompt_style: None,
        };
        assert!(!with_model.is_empty());
    }

    #[test]
    fn test_cto_config_merge() {
        let mut base = CtoConfig {
            cli: Some("claude".to_string()),
            model: Some("sonnet".to_string()),
            prompt_style: None,
        };

        let override_config = CtoConfig {
            cli: Some("cursor".to_string()),
            model: None,
            prompt_style: Some("minimal".to_string()),
        };

        base.merge(&override_config);

        assert_eq!(base.cli, Some("cursor".to_string()));
        assert_eq!(base.model, Some("sonnet".to_string())); // unchanged
        assert_eq!(base.prompt_style, Some("minimal".to_string()));
    }

    #[test]
    fn test_cto_config_merge_empty() {
        let mut base = CtoConfig {
            cli: Some("claude".to_string()),
            model: Some("sonnet".to_string()),
            prompt_style: Some("minimal".to_string()),
        };

        let empty = CtoConfig::default();
        base.merge(&empty);

        assert_eq!(base.cli, Some("claude".to_string()));
        assert_eq!(base.model, Some("sonnet".to_string()));
        assert_eq!(base.prompt_style, Some("minimal".to_string()));
    }

    #[test]
    fn test_cto_config_serialize() {
        let config = CtoConfig {
            cli: Some("cursor".to_string()),
            model: Some("opus".to_string()),
            prompt_style: None,
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
        assert!(url.contains("scope=read,write"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(super::capitalize("rex"), "Rex");
        assert_eq!(super::capitalize("blaze"), "Blaze");
        assert_eq!(super::capitalize(""), "");
        assert_eq!(super::capitalize("a"), "A");
    }

    // =========================================================================
    // Token Update Tests
    // =========================================================================

    #[test]
    fn test_update_tokens_with_expires_in() {
        let mut config = LinearConfig::default();
        config.apps.insert(
            "rex".to_string(),
            LinearAppConfig::new(
                "id".to_string(),
                "secret".to_string(),
                "webhook".to_string(),
            ),
        );

        config.update_tokens(
            "rex",
            "new_access_token",
            Some("new_refresh_token"),
            Some(3600),
        );

        let app = config.get_app("rex").unwrap();
        assert_eq!(app.access_token, Some("new_access_token".to_string()));
        assert_eq!(app.refresh_token, Some("new_refresh_token".to_string()));
        assert!(app.expires_at.is_some());
        // expires_at should be approximately now + 3600 seconds
        let now = Utc::now().timestamp();
        let expires_at = app.expires_at.unwrap();
        assert!(expires_at > now && expires_at <= now + 3600);
    }

    #[test]
    fn test_update_tokens_clears_expires_at_when_expires_in_is_none() {
        let mut config = LinearConfig::default();
        let mut app_config = LinearAppConfig::new(
            "id".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        // Set up an app with an old expires_at timestamp in the past
        app_config.access_token = Some("old_token".to_string());
        app_config.refresh_token = Some("old_refresh".to_string());
        app_config.expires_at = Some(Utc::now().timestamp() - 3600); // 1 hour ago (expired)
        config.apps.insert("rex".to_string(), app_config);

        // Verify the old token is considered expired
        assert!(config.get_app("rex").unwrap().is_token_expired());

        // Refresh tokens without expires_in
        config.update_tokens("rex", "new_access_token", Some("new_refresh_token"), None);

        let app = config.get_app("rex").unwrap();
        assert_eq!(app.access_token, Some("new_access_token".to_string()));
        assert_eq!(app.refresh_token, Some("new_refresh_token".to_string()));
        // expires_at should be cleared to None, not preserved as the old stale value
        assert!(
            app.expires_at.is_none(),
            "expires_at should be cleared when expires_in is None"
        );
    }

    #[test]
    fn test_update_tokens_replaces_old_expires_at() {
        let mut config = LinearConfig::default();
        let mut app_config = LinearAppConfig::new(
            "id".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        // Set up an app with an old expires_at timestamp
        app_config.access_token = Some("old_token".to_string());
        app_config.expires_at = Some(12345); // Some old timestamp
        config.apps.insert("rex".to_string(), app_config);

        // Update tokens with new expires_in
        config.update_tokens("rex", "new_access_token", None, Some(7200));

        let app = config.get_app("rex").unwrap();
        assert_eq!(app.access_token, Some("new_access_token".to_string()));
        // expires_at should be updated to a new value, not 12345
        let expires_at = app.expires_at.unwrap();
        assert_ne!(expires_at, 12345);
        let now = Utc::now().timestamp();
        assert!(expires_at > now && expires_at <= now + 7200);
    }
}
