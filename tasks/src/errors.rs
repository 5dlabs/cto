//! Error types for the tasks crate.

use thiserror::Error;

/// Comprehensive error types for task management
#[derive(Error, Debug, Clone)]
pub enum TasksError {
    // Task errors
    #[error("Task '{task_id}' not found")]
    TaskNotFound { task_id: String },

    #[error("Subtask '{subtask_id}' not found in task '{task_id}'")]
    SubtaskNotFound { task_id: String, subtask_id: String },

    #[error("Task '{task_id}' cannot be completed: {reason}")]
    CannotComplete { task_id: String, reason: String },

    #[error("Invalid status transition for task '{task_id}': {from} -> {to}")]
    InvalidTransition {
        task_id: String,
        from: String,
        to: String,
    },

    #[error("Invalid status: '{status}'")]
    InvalidStatus { status: String },

    #[error("Invalid priority: '{priority}'")]
    InvalidPriority { priority: String },

    #[error("Invalid task ID format: '{id}'")]
    InvalidId { id: String },

    // Dependency errors
    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },

    #[error("Invalid dependency: task '{task_id}' depends on non-existent task '{dep_id}'")]
    InvalidDependency { task_id: String, dep_id: String },

    #[error("Task '{task_id}' is blocked by incomplete dependencies: {deps:?}")]
    BlockedByDeps { task_id: String, deps: Vec<String> },

    // Tag errors
    #[error("Tag '{name}' not found")]
    TagNotFound { name: String },

    #[error("Tag '{name}' already exists")]
    TagAlreadyExists { name: String },

    #[error("Cannot delete the master tag")]
    CannotDeleteMasterTag,

    #[error("Cannot rename the master tag")]
    CannotRenameMasterTag,

    // Storage errors
    #[error("Storage error: {reason}")]
    StorageError { reason: String },

    #[error("Failed to read file '{path}': {reason}")]
    FileReadError { path: String, reason: String },

    #[error("Failed to write file '{path}': {reason}")]
    FileWriteError { path: String, reason: String },

    #[error("Failed to parse JSON: {reason}")]
    JsonParseError { reason: String },

    #[error("Project not initialized. Run 'tasks init' first.")]
    NotInitialized,

    #[error("Project already initialized at '{path}'")]
    AlreadyInitialized { path: String },

    // Configuration errors
    #[error("Configuration error: {reason}")]
    ConfigError { reason: String },

    #[error("Invalid configuration value for '{key}': {reason}")]
    InvalidConfigValue { key: String, reason: String },

    // AI errors
    #[error("AI error: {0}")]
    Ai(String),

    #[error("AI provider not configured: {provider}")]
    ProviderNotConfigured { provider: String },

    #[error("AI model not supported: {model}")]
    ModelNotSupported { model: String },

    #[error("AI response parse error: {reason}")]
    AiResponseParseError { reason: String },

    #[error("AI rate limit exceeded")]
    AiRateLimitExceeded,

    #[error("AI request timeout")]
    AiTimeout,

    // General errors
    #[error("Operation cancelled")]
    Cancelled,

    #[error("Invalid argument: {reason}")]
    InvalidArgument { reason: String },

    #[error("Internal error: {reason}")]
    Internal { reason: String },
}

impl From<std::io::Error> for TasksError {
    fn from(err: std::io::Error) -> Self {
        Self::StorageError {
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for TasksError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParseError {
            reason: err.to_string(),
        }
    }
}

/// Result type alias for tasks operations
pub type TasksResult<T> = Result<T, TasksError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TasksError::TaskNotFound {
            task_id: "123".to_string(),
        };
        assert_eq!(err.to_string(), "Task '123' not found");
    }

    #[test]
    fn test_circular_dependency_error() {
        let err = TasksError::CircularDependency {
            cycle: vec!["1".to_string(), "2".to_string(), "1".to_string()],
        };
        assert!(err.to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tasks_err: TasksError = io_err.into();
        matches!(tasks_err, TasksError::StorageError { .. });
    }
}
