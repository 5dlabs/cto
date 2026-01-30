//! Error types for CTO Lite

use serde::Serialize;
use thiserror::Error;

/// Application-wide error type
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Container runtime not found: {0}")]
    RuntimeNotFound(String),

    #[error("Container runtime not running: {0}")]
    RuntimeNotRunning(String),

    #[error("Cluster error: {0}")]
    ClusterError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Keychain error: {0}")]
    KeychainError(String),

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Tunnel error: {0}")]
    TunnelError(String),

    #[error("Not configured: {0}")]
    NotConfigured(String),
}

/// Serializable error response for Tauri commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(err: AppError) -> Self {
        let code = match &err {
            AppError::RuntimeNotFound(_) => "RUNTIME_NOT_FOUND",
            AppError::RuntimeNotRunning(_) => "RUNTIME_NOT_RUNNING",
            AppError::ClusterError(_) => "CLUSTER_ERROR",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::KeychainError(_) => "KEYCHAIN_ERROR",
            AppError::OAuthError(_) => "OAUTH_ERROR",
            AppError::HttpError(_) => "HTTP_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::JsonError(_) => "JSON_ERROR",
            AppError::CommandFailed(_) => "COMMAND_FAILED",
            AppError::ConfigError(_) => "CONFIG_ERROR",
            AppError::TunnelError(_) => "TUNNEL_ERROR",
            AppError::NotConfigured(_) => "NOT_CONFIGURED",
        };

        CommandError {
            code: code.to_string(),
            message: err.to_string(),
        }
    }
}

// Make AppError serializable for Tauri
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        CommandError::from(AppError::from(self.to_string())).serialize(serializer)
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::CommandFailed(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::CommandFailed(s.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
