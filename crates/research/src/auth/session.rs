//! Session management for Twitter authentication.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Twitter session cookies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The long-lived auth token cookie.
    pub auth_token: String,
    /// The CSRF token (regenerates automatically).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct0: Option<String>,
    /// When this session was created/updated.
    pub created_at: DateTime<Utc>,
    /// When this session was last validated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated: Option<DateTime<Utc>>,
}

impl Session {
    /// Create a new session with the given auth token.
    #[must_use]
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            ct0: None,
            created_at: Utc::now(),
            last_validated: None,
        }
    }

    /// Create a session with both auth token and CSRF token.
    #[must_use]
    pub fn with_ct0(auth_token: String, ct0: String) -> Self {
        Self {
            auth_token,
            ct0: Some(ct0),
            created_at: Utc::now(),
            last_validated: None,
        }
    }

    /// Load session from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let session: Self = serde_json::from_str(&content)?;
        Ok(session)
    }

    /// Save session to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load session from environment variables.
    pub fn from_env() -> Result<Self> {
        let auth_token = std::env::var("TWITTER_AUTH_TOKEN")
            .map_err(|_| anyhow::anyhow!("TWITTER_AUTH_TOKEN not set"))?;
        let ct0 = std::env::var("TWITTER_CT0").ok();

        Ok(Self {
            auth_token,
            ct0,
            created_at: Utc::now(),
            last_validated: None,
        })
    }

    /// Mark the session as validated.
    pub fn mark_validated(&mut self) {
        self.last_validated = Some(Utc::now());
    }

    /// Get cookie string for HTTP requests.
    #[must_use]
    pub fn cookie_string(&self) -> String {
        if let Some(ct0) = &self.ct0 {
            format!("auth_token={}; ct0={}", self.auth_token, ct0)
        } else {
            format!("auth_token={}", self.auth_token)
        }
    }
}
