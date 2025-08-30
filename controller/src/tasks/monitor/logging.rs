//! # Structured Logging Framework
//!
//! This module provides structured logging capabilities for the Agent Remediation Loop,
//! with JSON output format and context-aware logging functions.

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, info, warn, Level};
use chrono::{DateTime, Utc};

/// Logging errors
#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("Logging initialization error: {0}")]
    InitializationError(String),

    #[error("Log serialization error: {0}")]
    SerializationError(String),

    #[error("Log write error: {0}")]
    WriteError(String),
}

/// Result type for logging operations
pub type LoggingResult<T> = Result<T, LoggingError>;

/// Log levels for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub operation: Option<String>,
    pub task_id: Option<String>,
    pub pr_number: Option<i32>,
    pub correlation_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

/// Structured logger for consistent logging across the system
pub struct StructuredLogger {
    component_name: String,
    log_level: Level,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new() -> LoggingResult<Self> {
        Ok(Self {
            component_name: "agent-remediation-loop".to_string(),
            log_level: Level::INFO,
        })
    }

    /// Initialize the logger with production settings
    pub async fn initialize(&self) -> LoggingResult<()> {
        info!("Initializing structured logger for component: {}", self.component_name);
        // In a real implementation, this would configure the global logger
        // with JSON format, appropriate log levels, and output destinations
        Ok(())
    }

    /// Set the component name for this logger
    pub fn with_component_name(mut self, name: String) -> Self {
        self.component_name = name;
        self
    }

    /// Set the log level
    pub fn with_log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Log remediation cycle started
    pub fn log_remediation_started(
        &self,
        task_id: &str,
        pr_number: i32,
        correlation_id: &str,
    ) -> LoggingResult<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: format!("Remediation cycle started for task {}", task_id),
            component: self.component_name.clone(),
            operation: Some("remediation_start".to_string()),
            task_id: Some(task_id.to_string()),
            pr_number: Some(pr_number),
            correlation_id: Some(correlation_id.to_string()),
            metadata: HashMap::new(),
            duration_ms: None,
            error: None,
        };

        self.write_log_entry(&entry)?;
        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Log remediation cycle completed
    pub fn log_remediation_completed(
        &self,
        task_id: &str,
        pr_number: i32,
        success: bool,
        duration_seconds: f64,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("success".to_string(), success.into());
        metadata.insert("duration_seconds".to_string(), duration_seconds.into());

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: if success { LogLevel::Info } else { LogLevel::Warn },
            message: format!("Remediation cycle completed for task {}", task_id),
            component: self.component_name.clone(),
            operation: Some("remediation_complete".to_string()),
            task_id: Some(task_id.to_string()),
            pr_number: Some(pr_number),
            correlation_id: None,
            metadata,
            duration_ms: Some((duration_seconds * 1000.0) as u64),
            error: None,
        };

        self.write_log_entry(&entry)?;
        if success {
            info!("{}", serde_json::to_string(&entry).unwrap_or_default());
        } else {
            warn!("{}", serde_json::to_string(&entry).unwrap_or_default());
        }
        Ok(())
    }

    /// Log agent cancellation event
    pub fn log_agent_cancellation(
        &self,
        task_id: &str,
        pr_number: i32,
        reason: &str,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("reason".to_string(), reason.into());

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: format!("Agent cancelled for task {}", task_id),
            component: self.component_name.clone(),
            operation: Some("agent_cancellation".to_string()),
            task_id: Some(task_id.to_string()),
            pr_number: Some(pr_number),
            correlation_id: None,
            metadata,
            duration_ms: None,
            error: None,
        };

        self.write_log_entry(&entry)?;
        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Log escalation event
    pub fn log_escalation(
        &self,
        task_id: &str,
        pr_number: i32,
        escalation_type: &str,
        severity: &str,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("escalation_type".to_string(), escalation_type.into());
        metadata.insert("severity".to_string(), severity.into());

        let level = match severity {
            "critical" => LogLevel::Error,
            "high" => LogLevel::Warn,
            _ => LogLevel::Info,
        };

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: format!("Escalation triggered for task {}: {}", task_id, escalation_type),
            component: self.component_name.clone(),
            operation: Some("escalation".to_string()),
            task_id: Some(task_id.to_string()),
            pr_number: Some(pr_number),
            correlation_id: None,
            metadata,
            duration_ms: None,
            error: None,
        };

        self.write_log_entry(&entry)?;
        match severity {
            "critical" => error!("{}", serde_json::to_string(&entry).unwrap_or_default()),
            "high" => warn!("{}", serde_json::to_string(&entry).unwrap_or_default()),
            _ => info!("{}", serde_json::to_string(&entry).unwrap_or_default()),
        }
        Ok(())
    }

    /// Log state operation
    pub fn log_state_operation(
        &self,
        operation: &str,
        task_id: Option<&str>,
        pr_number: Option<i32>,
        duration_ms: u64,
        success: bool,
        error: Option<&str>,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), operation.into());
        metadata.insert("success".to_string(), success.into());

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: if success { LogLevel::Debug } else { LogLevel::Warn },
            message: format!("State operation {} completed", operation),
            component: self.component_name.clone(),
            operation: Some("state_operation".to_string()),
            task_id: task_id.map(|s| s.to_string()),
            pr_number,
            correlation_id: None,
            metadata,
            duration_ms: Some(duration_ms),
            error: error.map(|s| s.to_string()),
        };

        self.write_log_entry(&entry)?;
        debug!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Log label operation
    pub fn log_label_operation(
        &self,
        operation: &str,
        pr_number: i32,
        duration_ms: u64,
        success: bool,
        error: Option<&str>,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), operation.into());
        metadata.insert("success".to_string(), success.into());

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: if success { LogLevel::Debug } else { LogLevel::Warn },
            message: format!("Label operation {} completed", operation),
            component: self.component_name.clone(),
            operation: Some("label_operation".to_string()),
            task_id: None,
            pr_number: Some(pr_number),
            correlation_id: None,
            metadata,
            duration_ms: Some(duration_ms),
            error: error.map(|s| s.to_string()),
        };

        self.write_log_entry(&entry)?;
        debug!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Log GitHub API call
    pub fn log_github_api_call(
        &self,
        endpoint: &str,
        method: &str,
        status_code: u16,
        duration_ms: u64,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("endpoint".to_string(), endpoint.into());
        metadata.insert("method".to_string(), method.into());
        metadata.insert("status_code".to_string(), status_code.into());

        let level = if status_code >= 400 {
            LogLevel::Warn
        } else {
            LogLevel::Debug
        };

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: format!("GitHub API call to {} {}", method, endpoint),
            component: self.component_name.clone(),
            operation: Some("github_api_call".to_string()),
            task_id: None,
            pr_number: None,
            correlation_id: None,
            metadata,
            duration_ms: Some(duration_ms),
            error: None,
        };

        self.write_log_entry(&entry)?;
        debug!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Log system error with comprehensive context
    pub fn log_system_error(
        &self,
        component: &str,
        operation: &str,
        error: &str,
        task_id: Option<&str>,
        pr_number: Option<i32>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> LoggingResult<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: format!("System error in {}: {}", component, error),
            component: self.component_name.clone(),
            operation: Some(operation.to_string()),
            task_id: task_id.map(|s| s.to_string()),
            pr_number,
            correlation_id: None,
            metadata,
            duration_ms: None,
            error: Some(error.to_string()),
        };

        self.write_log_entry(&entry)?;
        error!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Log system health event
    pub fn log_system_health(
        &self,
        component_scores: HashMap<String, f64>,
        overall_score: f64,
    ) -> LoggingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("overall_score".to_string(), overall_score.into());

        for (component, score) in component_scores {
            metadata.insert(format!("{}_score", component), score.into());
        }

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: format!("System health check completed with score {:.2}", overall_score),
            component: self.component_name.clone(),
            operation: Some("health_check".to_string()),
            task_id: None,
            pr_number: None,
            correlation_id: None,
            metadata,
            duration_ms: None,
            error: None,
        };

        self.write_log_entry(&entry)?;
        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
        Ok(())
    }

    /// Write a log entry (internal method)
    fn write_log_entry(&self, entry: &LogEntry) -> LoggingResult<()> {
        // In a real implementation, this would write to configured log sinks
        // (files, databases, external services, etc.)

        // For now, we just validate the JSON serialization
        let _json = serde_json::to_string(entry)
            .map_err(|e| LoggingError::SerializationError(e.to_string()))?;

        Ok(())
    }

    /// Get recent log entries (for debugging/testing)
    pub fn get_recent_entries(&self, _limit: usize) -> Vec<LogEntry> {
        // In a real implementation, this would return recent log entries
        // from a buffer or external storage
        Vec::new()
    }
}

