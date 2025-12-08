use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Comprehensive error types for the MCP Bridge Proxy
#[derive(Error, Debug, Clone)]
pub enum BridgeError {
    // Connection and Server Errors
    #[error("Server '{name}' failed to start: {reason}")]
    ServerStartupFailed { name: String, reason: String },

    #[error("Server '{name}' connection lost: {reason}")]
    ServerConnectionLost { name: String, reason: String },

    #[error("Server '{name}' is unresponsive (timeout after {timeout_secs}s)")]
    ServerTimeout { name: String, timeout_secs: u64 },

    #[error("Server '{name}' process crashed with exit code {exit_code:?}")]
    ServerCrashed {
        name: String,
        exit_code: Option<i32>,
    },

    #[error("Server '{name}' not found in configuration")]
    ServerNotFound { name: String },

    #[error("Failed to initialize server '{name}': {reason}")]
    ServerInitializationFailed { name: String, reason: String },

    // Tool Errors
    #[error("Tool '{tool}' not found on server '{server}'")]
    ToolNotFound { server: String, tool: String },

    #[error("Tool '{tool}' is disabled")]
    ToolDisabled { tool: String },

    #[error("Invalid tool name format: '{name}' (expected 'server_tool' format)")]
    InvalidToolFormat { name: String },

    #[error("Tool call failed for '{tool}' on server '{server}': {reason}")]
    ToolCallFailed {
        server: String,
        tool: String,
        reason: String,
    },

    // Protocol and Communication Errors
    #[error("Invalid JSON-RPC request: {reason}")]
    InvalidJsonRpc { reason: String },

    #[error("Malformed request payload: {reason}")]
    MalformedRequest { reason: String },

    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolMismatch { expected: String, actual: String },

    #[error("Communication error with server '{server}': {reason}")]
    CommunicationError { server: String, reason: String },

    // Configuration Errors
    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },

    #[error("Failed to save configuration: {reason}")]
    ConfigSaveFailed { reason: String },

    #[error("Invalid server configuration for '{server}': {reason}")]
    InvalidServerConfig { server: String, reason: String },

    // Resource and System Errors
    #[error("Resource limit exceeded: {resource} ({limit})")]
    ResourceLimitExceeded { resource: String, limit: String },

    #[error("Insufficient system resources: {reason}")]
    InsufficientResources { reason: String },

    #[error("File system error: {reason}")]
    FileSystemError { reason: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    // Recovery and Fallback Errors
    #[error("Server restart failed for '{server}': {reason}")]
    ServerRestartFailed { server: String, reason: String },

    #[error("Recovery attempt failed: {reason}")]
    RecoveryFailed { reason: String },

    #[error("All fallback servers failed for tool '{tool}'")]
    AllFallbacksFailed { tool: String },

    #[error("Health check failed for server '{server}': {reason}")]
    HealthCheckFailed { server: String, reason: String },

    // Internal Errors
    #[error("Internal error: {reason}")]
    Internal { reason: String },

    #[error("Unexpected state: {description}")]
    UnexpectedState { description: String },
}

/// Error severity levels for categorization and response strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - informational, operation can continue
    Info,
    /// Medium severity - warning, may impact performance but service continues
    Warning,
    /// High severity - error, operation failed but system is stable
    Error,
    /// Critical severity - system stability at risk, immediate action required
    Critical,
}

/// Error recovery strategies for different types of failures
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// No recovery needed - error is informational
    None,
    /// Retry the operation with backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
    /// Restart the failed server
    RestartServer { server_name: String },
    /// Switch to fallback server
    UseFallback { fallback_servers: Vec<String> },
    /// Notify user and wait for manual intervention
    ManualIntervention { message: String },
    /// Graceful degradation - disable feature
    GracefulDegradation { feature: String },
}

/// Comprehensive error context for better debugging and recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error: BridgeError,
    pub severity: ErrorSeverity,
    pub recovery_strategy: RecoveryStrategy,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context_data: std::collections::HashMap<String, String>,
    pub correlation_id: String,
}

impl ErrorContext {
    pub fn new(error: BridgeError) -> Self {
        let (severity, recovery_strategy) = Self::determine_severity_and_strategy(&error);

        Self {
            error,
            severity,
            recovery_strategy,
            timestamp: chrono::Utc::now(),
            context_data: std::collections::HashMap::new(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context_data.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = id.to_string();
        self
    }

    /// Determine appropriate severity and recovery strategy based on error type
    fn determine_severity_and_strategy(error: &BridgeError) -> (ErrorSeverity, RecoveryStrategy) {
        match error {
            BridgeError::ServerNotFound { name } => (
                ErrorSeverity::Error,
                RecoveryStrategy::ManualIntervention {
                    message: format!("Server '{name}' not found in configuration"),
                },
            ),

            BridgeError::ServerTimeout { .. } => (
                ErrorSeverity::Warning,
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    backoff_ms: 1000,
                },
            ),

            BridgeError::ServerCrashed { name, .. } => (
                ErrorSeverity::Error,
                RecoveryStrategy::RestartServer {
                    server_name: name.clone(),
                },
            ),

            BridgeError::ServerConnectionLost { name, .. } => (
                ErrorSeverity::Warning,
                RecoveryStrategy::RestartServer {
                    server_name: name.clone(),
                },
            ),

            BridgeError::ToolNotFound { .. } => (ErrorSeverity::Info, RecoveryStrategy::None),

            BridgeError::ToolDisabled { .. } => (ErrorSeverity::Info, RecoveryStrategy::None),

            BridgeError::InvalidJsonRpc { .. } => (ErrorSeverity::Warning, RecoveryStrategy::None),

            BridgeError::ConfigurationError { .. } => (
                ErrorSeverity::Error,
                RecoveryStrategy::ManualIntervention {
                    message: "Configuration issue requires manual review".to_string(),
                },
            ),

            BridgeError::ResourceLimitExceeded { .. } => (
                ErrorSeverity::Critical,
                RecoveryStrategy::GracefulDegradation {
                    feature: "New server connections".to_string(),
                },
            ),

            BridgeError::AllFallbacksFailed { tool } => (
                ErrorSeverity::Error,
                RecoveryStrategy::ManualIntervention {
                    message: format!("All servers failed for tool '{tool}'"),
                },
            ),

            _ => (
                ErrorSeverity::Warning,
                RecoveryStrategy::Retry {
                    max_attempts: 2,
                    backoff_ms: 500,
                },
            ),
        }
    }

    /// Generate user-friendly error message
    pub fn user_message(&self) -> String {
        match &self.error {
            BridgeError::ServerTimeout { name, timeout_secs } => {
                format!("⏱️ Server '{name}' is taking longer than expected ({timeout_secs}s). This might be temporary - please try again.")
            }

            BridgeError::ServerCrashed { name, .. } => {
                format!(
                    "🔄 Server '{name}' encountered an issue and is being restarted automatically."
                )
            }

            BridgeError::ToolNotFound { server, tool } => {
                format!("🔧 Tool '{tool}' is not available on server '{server}'. Check if the tool exists or try enabling it first.")
            }

            BridgeError::ToolDisabled { tool } => {
                format!(
                    "⚠️ Tool '{tool}' is currently disabled. Use enable_tool to make it available."
                )
            }

            BridgeError::ConfigurationError { reason } => {
                format!(
                    "⚙️ Configuration issue: {reason}. Please check your mcp-servers.json file."
                )
            }

            BridgeError::InvalidJsonRpc { reason } => {
                format!("📝 Request format issue: {reason}. Please check the request structure.")
            }

            _ => {
                format!(
                    "❌ An error occurred: {}. Please try again or contact support.",
                    self.error
                )
            }
        }
    }

    /// Generate technical error message for logs
    pub fn technical_message(&self) -> String {
        format!(
            "[{}] {} | Severity: {:?} | Recovery: {:?} | Correlation: {} | Context: {:?}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.error,
            self.severity,
            self.recovery_strategy,
            self.correlation_id,
            self.context_data
        )
    }
}

/// Result type with our custom error context
pub type BridgeResult<T> = Result<T, Box<ErrorContext>>;

/// Helper trait for converting errors to our error context
pub trait IntoBridgeError<T> {
    fn into_bridge_error(self) -> BridgeResult<T>;
    fn into_bridge_error_with_context(
        self,
        context_fn: impl FnOnce() -> ErrorContext,
    ) -> BridgeResult<T>;
}

impl<T, E> IntoBridgeError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_bridge_error(self) -> BridgeResult<T> {
        self.map_err(|e| {
            Box::new(ErrorContext::new(BridgeError::Internal {
                reason: e.to_string(),
            }))
        })
    }

    fn into_bridge_error_with_context(
        self,
        context_fn: impl FnOnce() -> ErrorContext,
    ) -> BridgeResult<T> {
        self.map_err(|_| Box::new(context_fn()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_assignment() {
        let error = BridgeError::ServerTimeout {
            name: "test".to_string(),
            timeout_secs: 10,
        };
        let context = ErrorContext::new(error);

        matches!(context.severity, ErrorSeverity::Warning);
        matches!(context.recovery_strategy, RecoveryStrategy::Retry { .. });
    }

    #[test]
    fn test_user_message_formatting() {
        let error = BridgeError::ToolNotFound {
            server: "git".to_string(),
            tool: "commit".to_string(),
        };
        let context = ErrorContext::new(error);

        let message = context.user_message();
        assert!(message.contains("Tool 'commit' is not available"));
        assert!(message.contains("server 'git'"));
    }
}
