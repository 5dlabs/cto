//! Error types for CTO

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

    #[error("Other error: {0}")]
    Other(String),
}

/// Serializable error response for Tauri commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(err: AppError) -> Self {
        let (code, message) = match &err {
            AppError::RuntimeNotFound(msg) => ("RUNTIME_NOT_FOUND", msg.clone()),
            AppError::RuntimeNotRunning(msg) => ("RUNTIME_NOT_RUNNING", msg.clone()),
            AppError::ClusterError(msg) => ("CLUSTER_ERROR", msg.clone()),
            // Sanitize internal errors - don't expose database/system details
            AppError::DatabaseError(_) => {
                tracing::error!("Database error: {}", err);
                ("DATABASE_ERROR", "A database error occurred".to_string())
            }
            AppError::KeychainError(msg) => ("KEYCHAIN_ERROR", msg.clone()),
            AppError::OAuthError(msg) => ("OAUTH_ERROR", msg.clone()),
            // Sanitize HTTP errors - don't expose internal URLs or auth details
            AppError::HttpError(e) => {
                tracing::error!("HTTP error: {}", e);
                let sanitized = if e.is_timeout() {
                    "Request timed out".to_string()
                } else if e.is_connect() {
                    "Connection failed".to_string()
                } else {
                    "Network request failed".to_string()
                };
                ("HTTP_ERROR", sanitized)
            }
            // Sanitize IO errors - don't expose file paths
            AppError::IoError(e) => {
                tracing::error!("IO error: {}", e);
                let sanitized = match e.kind() {
                    std::io::ErrorKind::NotFound => "File not found".to_string(),
                    std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
                    _ => "An I/O error occurred".to_string(),
                };
                ("IO_ERROR", sanitized)
            }
            AppError::JsonError(_) => {
                tracing::error!("JSON error: {}", err);
                ("JSON_ERROR", "Invalid data format".to_string())
            }
            AppError::CommandFailed(msg) => ("COMMAND_FAILED", msg.clone()),
            AppError::ConfigError(msg) => ("CONFIG_ERROR", msg.clone()),
            AppError::TunnelError(msg) => ("TUNNEL_ERROR", msg.clone()),
            AppError::NotConfigured(msg) => ("NOT_CONFIGURED", msg.clone()),
            AppError::Other(msg) => ("OTHER", msg.clone()),
        };

        CommandError {
            code: code.to_string(),
            message,
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
