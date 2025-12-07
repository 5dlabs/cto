//! Configuration for the Linear service.

use serde::{Deserialize, Serialize};
use std::env;

/// Linear webhook handler configuration.
#[derive(Clone)]
pub struct Config {
    /// HTTP server port.
    pub port: u16,
    /// Webhook signing secret for signature verification.
    pub webhook_secret: Option<String>,
    /// Maximum age for webhook timestamps (default: 60 seconds).
    pub max_timestamp_age_ms: i64,
    /// Whether Linear integration is enabled.
    pub enabled: bool,
    /// Linear OAuth token for API calls.
    pub oauth_token: Option<String>,
    /// GitHub token for API calls (fetching files from repos).
    pub github_token: Option<String>,
    /// Kubernetes namespace.
    pub namespace: String,
    /// Webhook callback URL (for GitHub webhooks to call back to this service).
    pub webhook_callback_url: Option<String>,
    /// GitHub repositories to auto-configure webhooks on (comma-separated, e.g., "5dlabs/cto,5dlabs/other").
    pub github_webhook_repos: Vec<String>,
    /// Intake configuration.
    pub intake: IntakeConfig,
    /// Play configuration.
    pub play: PlayConfig,
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
            github_token: env::var("GITHUB_TOKEN").ok(),
            namespace: env::var("NAMESPACE").unwrap_or_else(|_| "cto".to_string()),
            webhook_callback_url: env::var("WEBHOOK_CALLBACK_URL").ok(),
            github_webhook_repos: env::var("GITHUB_WEBHOOK_REPOS")
                .ok()
                .map(|s| s.split(',').map(|r| r.trim().to_string()).collect())
                .unwrap_or_default(),
            intake: IntakeConfig::default(),
            play: PlayConfig::default(),
        }
    }
}

/// Intake workflow configuration.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct IntakeConfig {
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
}

impl Default for IntakeConfig {
    fn default() -> Self {
        Self {
            github_app: env::var("GITHUB_APP_NAME").unwrap_or_else(|_| "cto-dev".to_string()),
            primary_model: env::var("PRIMARY_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            primary_provider: env::var("PRIMARY_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
            research_model: env::var("RESEARCH_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            research_provider: env::var("RESEARCH_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
            fallback_model: env::var("FALLBACK_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            fallback_provider: env::var("FALLBACK_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
            docs_model: env::var("DOCS_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            num_tasks: env::var("NUM_TASKS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            expand_tasks: true,
            analyze_complexity: true,
            enrich_context: true,
            include_codebase: false,
            cli: env::var("INTAKE_CLI").unwrap_or_else(|_| "claude".to_string()),
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
    /// Implementation agent (rex, blaze, etc).
    pub implementation_agent: String,
    /// Testing agent.
    pub testing_agent: String,
    /// Quality agent.
    pub quality_agent: String,
    /// Frontend agent.
    pub frontend_agent: String,
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
                .unwrap_or_else(|_| "rex".to_string()),
            testing_agent: env::var("TESTING_AGENT").unwrap_or_else(|_| "tess".to_string()),
            quality_agent: env::var("QUALITY_AGENT").unwrap_or_else(|_| "cleo".to_string()),
            frontend_agent: env::var("FRONTEND_AGENT").unwrap_or_else(|_| "blaze".to_string()),
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
}
