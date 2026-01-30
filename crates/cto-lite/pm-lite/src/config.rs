//! Configuration for PM Lite

use serde::Deserialize;

/// PM Lite configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// GitHub App configuration
    pub github: GitHubConfig,

    /// Kubernetes namespace for workflows
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

/// GitHub App configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubConfig {
    /// GitHub App ID
    pub app_id: Option<String>,

    /// GitHub App private key (PEM format)
    pub private_key: Option<String>,

    /// Webhook secret for signature verification
    pub webhook_secret: Option<String>,
}

fn default_port() -> u16 {
    8080
}

fn default_namespace() -> String {
    "cto-lite".to_string()
}

impl Config {
    /// Load config from environment variables
    ///
    /// # Errors
    /// Returns error if required env vars are missing
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(default_port),
            github: GitHubConfig {
                app_id: std::env::var("GITHUB_APP_ID").ok(),
                private_key: std::env::var("GITHUB_PRIVATE_KEY").ok(),
                webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET").ok(),
            },
            namespace: std::env::var("NAMESPACE").unwrap_or_else(|_| default_namespace()),
        })
    }
}
