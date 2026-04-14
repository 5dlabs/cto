//! Configuration for PM Lite

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    pub github: GitHubConfig,
    #[serde(default)]
    pub linear: LinearOAuthConfig,
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubConfig {
    pub app_id: Option<String>,
    pub private_key: Option<String>,
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LinearOAuthConfig {
    #[serde(default)]
    pub apps: HashMap<String, LinearAppConfig>,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LinearAppConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(skip_serializing)]
    pub webhook_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

fn default_port() -> u16 {
    8080
}

fn default_namespace() -> String {
    "cto-lite".to_string()
}

impl Config {
    #[must_use]
    pub fn from_env() -> Self {
        let redirect_uri = env::var("LINEAR_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8080/oauth/callback".to_string());

        let mut apps = HashMap::new();

        // Load Morgan
        if let (Ok(app_id), Ok(client_secret)) = (
            env::var("LINEAR_APP_MORGAN_CLIENT_ID"),
            env::var("LINEAR_APP_MORGAN_CLIENT_SECRET"),
        ) {
            apps.insert(
                "morgan".to_string(),
                LinearAppConfig {
                    client_id: app_id,
                    client_secret,
                    webhook_secret: String::new(),
                    access_token: None,
                    refresh_token: env::var("LINEAR_REFRESH_TOKEN_MORGAN").ok(),
                    expires_at: None,
                },
            );
        }

        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            github: GitHubConfig {
                app_id: env::var("GITHUB_APP_ID").ok(),
                private_key: env::var("GITHUB_PRIVATE_KEY").ok(),
                webhook_secret: env::var("GITHUB_WEBHOOK_SECRET").ok(),
            },
            linear: LinearOAuthConfig { apps, redirect_uri },
            namespace: env::var("NAMESPACE").unwrap_or_else(|_| default_namespace()),
        }
    }

    pub fn load() -> Self {
        let config_dir = if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".config/cto-lite")
        } else {
            PathBuf::from(".config/cto-lite")
        };
        let config_file = config_dir.join("pm-lite.json");

        if config_file.exists() {
            match std::fs::read_to_string(&config_file) {
                Ok(content) => {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                    tracing::warn!("Failed to parse config file");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to read config file");
                }
            }
        }

        Self::from_env()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_dir = if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".config/cto-lite")
        } else {
            PathBuf::from(".config/cto-lite")
        };

        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            return Err(format!("Failed to create dir: {e}"));
        }

        let config_file = config_dir.join("pm-lite.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| format!("Serialize: {e}"))?;
        std::fs::write(&config_file, content).map_err(|e| format!("Write: {e}"))?;

        tracing::info!(path = %config_file.display(), "Config saved");
        Ok(())
    }
}
