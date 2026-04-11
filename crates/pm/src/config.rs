//! Configuration for the PM service.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

// =============================================================================
// SCM Provider Configuration
// =============================================================================

/// Which source control management platform to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScmProvider {
    GitHub,
    GitLab,
}

impl Default for ScmProvider {
    fn default() -> Self {
        Self::GitHub
    }
}

impl std::fmt::Display for ScmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::GitLab => write!(f, "gitlab"),
        }
    }
}

impl std::str::FromStr for ScmProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "gitlab" => Ok(Self::GitLab),
            other => Err(format!("unknown SCM provider: {other}")),
        }
    }
}

/// Unified SCM configuration for GitHub and GitLab side-by-side operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScmConfig {
    /// Active provider (override via `CTO_SCM_PROVIDER` env var).
    pub provider: ScmProvider,
    /// Hostname of the SCM (e.g. `github.com` or `git.5dlabs.ai`).
    pub host: String,
    /// Base URL for REST API calls.
    pub api_base: String,
    /// Organization (GitHub) or group (GitLab).
    pub org_or_group: String,
    /// Container registry prefix (e.g. `ghcr.io/5dlabs`).
    pub registry: String,
    /// Auth token for API calls.
    pub token: Option<String>,
    /// Webhook verification secret.
    pub webhook_secret: Option<String>,
}

impl Default for ScmConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ScmConfig {
    /// Build config from environment, defaulting to GitHub.
    #[must_use]
    pub fn from_env() -> Self {
        let provider: ScmProvider = env::var("CTO_SCM_PROVIDER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();

        match provider {
            ScmProvider::GitHub => Self {
                provider,
                host: env::var("GITHUB_HOST").unwrap_or_else(|_| "github.com".to_string()),
                api_base: env::var("GITHUB_API_BASE")
                    .unwrap_or_else(|_| "https://api.github.com".to_string()),
                org_or_group: env::var("GITHUB_DEFAULT_ORG")
                    .unwrap_or_else(|_| "5dlabs".to_string()),
                registry: env::var("GITHUB_REGISTRY")
                    .unwrap_or_else(|_| "ghcr.io/5dlabs".to_string()),
                token: env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty()),
                webhook_secret: env::var("GITHUB_WEBHOOK_SECRET")
                    .ok()
                    .filter(|s| !s.is_empty()),
            },
            ScmProvider::GitLab => Self {
                provider,
                host: env::var("GITLAB_HOST").unwrap_or_else(|_| "git.5dlabs.ai".to_string()),
                api_base: env::var("GITLAB_API_BASE")
                    .unwrap_or_else(|_| "https://git.5dlabs.ai/api/v4".to_string()),
                org_or_group: env::var("GITLAB_GROUP").unwrap_or_else(|_| "5dlabs".to_string()),
                registry: env::var("GITLAB_REGISTRY")
                    .unwrap_or_else(|_| "registry.5dlabs.ai/5dlabs".to_string()),
                token: env::var("GITLAB_TOKEN").ok().filter(|s| !s.is_empty()),
                webhook_secret: env::var("GITLAB_WEBHOOK_SECRET")
                    .ok()
                    .filter(|s| !s.is_empty()),
            },
        }
    }

    /// Whether the active provider is GitHub.
    #[must_use]
    pub fn is_github(&self) -> bool {
        self.provider == ScmProvider::GitHub
    }

    /// Whether the active provider is GitLab.
    #[must_use]
    pub fn is_gitlab(&self) -> bool {
        self.provider == ScmProvider::GitLab
    }
}

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
    /// GitLab webhook secret for token verification.
    pub gitlab_webhook_secret: Option<String>,
    /// Unified SCM provider configuration.
    pub scm: ScmConfig,
    /// Morgan/OpenClaw dispatch configuration for reversible webhook cutover.
    pub morgan_dispatch: MorganDispatchConfig,
}

/// How PM should route verified webhook deliveries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookDispatchMode {
    /// Keep using PM's existing direct-processing logic.
    Legacy,
    /// Forward to Morgan as a best-effort shadow copy, then continue legacy handling.
    Shadow,
    /// Forward to Morgan and skip PM's legacy direct-processing logic.
    Morgan,
}

impl Default for WebhookDispatchMode {
    fn default() -> Self {
        Self::Legacy
    }
}

impl std::str::FromStr for WebhookDispatchMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "legacy" => Ok(Self::Legacy),
            "shadow" | "morgan-shadow" => Ok(Self::Shadow),
            "morgan" | "morgan-only" => Ok(Self::Morgan),
            other => Err(format!("unknown webhook dispatch mode: {other}")),
        }
    }
}

impl WebhookDispatchMode {
    /// Whether this mode should forward deliveries to Morgan.
    #[must_use]
    pub const fn dispatches_to_morgan(self) -> bool {
        matches!(self, Self::Shadow | Self::Morgan)
    }

    /// Whether this mode keeps the legacy PM logic active after dispatch.
    #[must_use]
    pub const fn keeps_legacy_processing(self) -> bool {
        matches!(self, Self::Legacy | Self::Shadow)
    }
}

/// Morgan/OpenClaw hook settings used for webhook forwarding.
#[derive(Debug, Clone)]
pub struct MorganDispatchConfig {
    /// Dispatch mode for verified webhooks.
    pub mode: WebhookDispatchMode,
    /// Base URL for the Morgan/OpenClaw gateway (without the `/hooks/agent` suffix).
    pub base_url: Option<String>,
    /// Bearer token used to authenticate to OpenClaw hooks.
    pub token: Option<String>,
    /// Target agent ID for OpenClaw hook routing.
    pub agent_id: String,
    /// Prefix for generated session keys.
    pub session_key_prefix: String,
    /// Timeout for hook-triggered runs.
    pub timeout_seconds: u64,
}

impl Default for MorganDispatchConfig {
    fn default() -> Self {
        Self {
            mode: env::var("PM_WEBHOOK_DISPATCH_MODE")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or_default(),
            base_url: env::var("MORGAN_HOOKS_BASE_URL")
                .ok()
                .map(|s| s.trim().trim_end_matches('/').to_string())
                .filter(|s| !s.is_empty()),
            token: env::var("MORGAN_HOOKS_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
            agent_id: env::var("MORGAN_HOOKS_AGENT_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "morgan".to_string()),
            session_key_prefix: env::var("MORGAN_HOOKS_SESSION_KEY_PREFIX")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "hook:pm".to_string()),
            timeout_seconds: env::var("MORGAN_HOOKS_TIMEOUT_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120),
        }
    }
}

impl MorganDispatchConfig {
    /// Whether Morgan dispatch is active for any verified webhook deliveries.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.mode.dispatches_to_morgan()
    }
}

// =============================================================================
// Multi-Agent Linear OAuth Configuration
// =============================================================================

/// All known agent names for the CTO platform.
pub const AGENT_NAMES: &[&str] = &[
    "morgan", "rex", "blaze", "grizz", "nova", "tap", "spark", "cleo", "cipher", "tess", "atlas",
    "bolt", "stitch", "vex", "angie", "pixel",
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
    /// Runtime access token minted or refreshed by PM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Refresh token for legacy authorization_code apps.
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
            None => false, // No expiration info — token may be long-lived (e.g. loaded from K8s secret without expires_at)
        }
    }

    /// Check if the token can be refreshed.
    #[must_use]
    pub fn can_refresh(&self) -> bool {
        self.refresh_token.is_some() && !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    /// Check if the app has enough credentials to mint a token via client_credentials.
    #[must_use]
    pub fn can_mint_client_credentials(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    /// Whether PM should proactively mint a client_credentials token.
    ///
    /// We only do this when:
    /// - the app has client credentials
    /// - the app is not already using refresh-token flow
    /// - there is no access token yet, or
    /// - the token has a known expiration and is within the 1 hour renewal buffer
    ///
    /// If an access token exists but `expires_at` is missing, we avoid blind
    /// replacement to preserve legacy long-lived tokens until they actually fail.
    #[must_use]
    pub fn should_proactively_mint_client_credentials(&self) -> bool {
        if !self.can_mint_client_credentials() || self.can_refresh() {
            return false;
        }

        if self.access_token.is_none() {
            return true;
        }

        self.expires_at
            .is_some_and(|expires_at| expires_at - Utc::now().timestamp() < 3600)
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
            gitlab_webhook_secret: env::var("GITLAB_WEBHOOK_SECRET")
                .ok()
                .filter(|s| !s.is_empty()),
            scm: ScmConfig::from_env(),
            morgan_dispatch: MorganDispatchConfig::default(),
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
    /// Default skills-release repo URL stamped into every CodeRun spec.
    /// Format: "https://github.com/{owner}/{repo}"
    pub skills_repo: Option<String>,
}

impl Default for IntakeConfig {
    fn default() -> Self {
        Self {
            // Container image for intake workflows (derived from CLI selection)
            runtime_image: env::var("RUNTIME_IMAGE")
                .unwrap_or_else(|_| "registry.5dlabs.ai/5dlabs/claude:latest".to_string()),
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
            skills_repo: env::var("SKILLS_REPO").ok(),
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
        env::remove_var("PM_WEBHOOK_DISPATCH_MODE");
        env::remove_var("MORGAN_HOOKS_BASE_URL");
        env::remove_var("MORGAN_HOOKS_TOKEN");
        env::remove_var("MORGAN_HOOKS_AGENT_ID");
        env::remove_var("MORGAN_HOOKS_SESSION_KEY_PREFIX");
        env::remove_var("MORGAN_HOOKS_TIMEOUT_SECONDS");

        let config = Config::default();
        assert!(!config.enabled);
        assert!(config.webhook_secret.is_none());
        assert_eq!(config.max_timestamp_age_ms, 60_000);
        assert_eq!(config.port, 8081);
        assert_eq!(config.morgan_dispatch.mode, WebhookDispatchMode::Legacy);
        assert_eq!(config.morgan_dispatch.agent_id, "morgan");
        assert_eq!(config.morgan_dispatch.session_key_prefix, "hook:pm");
        assert_eq!(config.morgan_dispatch.timeout_seconds, 120);
    }

    #[test]
    fn test_config_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();

        // Clean first
        env::remove_var("LINEAR_ENABLED");
        env::remove_var("LINEAR_WEBHOOK_SECRET");
        env::remove_var("LINEAR_PORT");
        env::remove_var("PM_WEBHOOK_DISPATCH_MODE");
        env::remove_var("MORGAN_HOOKS_BASE_URL");
        env::remove_var("MORGAN_HOOKS_TOKEN");
        env::remove_var("MORGAN_HOOKS_AGENT_ID");
        env::remove_var("MORGAN_HOOKS_SESSION_KEY_PREFIX");
        env::remove_var("MORGAN_HOOKS_TIMEOUT_SECONDS");

        env::set_var("LINEAR_ENABLED", "true");
        env::set_var("LINEAR_WEBHOOK_SECRET", "test-secret");
        env::set_var("LINEAR_PORT", "9000");
        env::set_var("PM_WEBHOOK_DISPATCH_MODE", "morgan");
        env::set_var("MORGAN_HOOKS_BASE_URL", "https://morgan.example.com/");
        env::set_var("MORGAN_HOOKS_TOKEN", "hook-token");
        env::set_var("MORGAN_HOOKS_AGENT_ID", "atlas");
        env::set_var("MORGAN_HOOKS_SESSION_KEY_PREFIX", "hook:pm:test");
        env::set_var("MORGAN_HOOKS_TIMEOUT_SECONDS", "45");

        let config = Config::default();
        assert!(config.enabled);
        assert_eq!(config.webhook_secret, Some("test-secret".to_string()));
        assert_eq!(config.port, 9000);
        assert_eq!(config.morgan_dispatch.mode, WebhookDispatchMode::Morgan);
        assert_eq!(
            config.morgan_dispatch.base_url,
            Some("https://morgan.example.com".to_string())
        );
        assert_eq!(config.morgan_dispatch.token, Some("hook-token".to_string()));
        assert_eq!(config.morgan_dispatch.agent_id, "atlas");
        assert_eq!(config.morgan_dispatch.session_key_prefix, "hook:pm:test");
        assert_eq!(config.morgan_dispatch.timeout_seconds, 45);

        // Clean up
        env::remove_var("LINEAR_ENABLED");
        env::remove_var("LINEAR_WEBHOOK_SECRET");
        env::remove_var("LINEAR_PORT");
        env::remove_var("PM_WEBHOOK_DISPATCH_MODE");
        env::remove_var("MORGAN_HOOKS_BASE_URL");
        env::remove_var("MORGAN_HOOKS_TOKEN");
        env::remove_var("MORGAN_HOOKS_AGENT_ID");
        env::remove_var("MORGAN_HOOKS_SESSION_KEY_PREFIX");
        env::remove_var("MORGAN_HOOKS_TIMEOUT_SECONDS");
    }

    #[test]
    fn test_webhook_dispatch_mode_parsing() {
        assert_eq!(
            "legacy".parse::<WebhookDispatchMode>().unwrap(),
            WebhookDispatchMode::Legacy
        );
        assert_eq!(
            "shadow".parse::<WebhookDispatchMode>().unwrap(),
            WebhookDispatchMode::Shadow
        );
        assert_eq!(
            "morgan".parse::<WebhookDispatchMode>().unwrap(),
            WebhookDispatchMode::Morgan
        );
        assert!("unknown".parse::<WebhookDispatchMode>().is_err());
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

    #[test]
    fn test_linear_app_can_mint_client_credentials() {
        let app = LinearAppConfig::new(
            "id".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        assert!(app.can_mint_client_credentials());
        assert!(!LinearAppConfig::default().can_mint_client_credentials());
    }

    #[test]
    fn test_should_proactively_mint_client_credentials_when_token_missing() {
        let app = LinearAppConfig::new(
            "id".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        assert!(app.should_proactively_mint_client_credentials());
    }

    #[test]
    fn test_should_proactively_mint_client_credentials_when_token_expiring() {
        let mut app = LinearAppConfig::new(
            "id".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        app.access_token = Some("token".to_string());
        app.expires_at = Some(Utc::now().timestamp() + 1800);

        assert!(app.should_proactively_mint_client_credentials());
    }

    #[test]
    fn test_should_not_proactively_mint_client_credentials_without_expiry() {
        let mut app = LinearAppConfig::new(
            "id".to_string(),
            "secret".to_string(),
            "webhook".to_string(),
        );
        app.access_token = Some("legacy_token".to_string());

        assert!(!app.should_proactively_mint_client_credentials());
    }
}
