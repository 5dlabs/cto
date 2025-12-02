//! # Comprehensive Audit Logging
//!
//! This module provides comprehensive audit logging for security events,
//! access control, and compliance tracking in the Agent Remediation Loop.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

/// Audit logging errors
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Audit logging error: {0}")]
    LoggingError(String),

    #[error("Audit storage error: {0}")]
    StorageError(String),

    #[error("Audit configuration error: {0}")]
    ConfigurationError(String),
}

/// Result type for audit operations
pub type AuditResult<T> = Result<T, AuditError>;

/// Audit event severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub success: bool,
    pub severity: AuditSeverity,
    pub error_message: Option<String>,
    pub resource_id: Option<String>,
    pub task_id: Option<String>,
    pub pr_number: Option<i32>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Audit logger for comprehensive security event tracking
pub struct AuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_events: usize,
    log_to_console: bool,
    log_to_file: bool,
    log_file_path: Option<String>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> AuditResult<Self> {
        Ok(Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 10000, // Keep last 10k events in memory
            log_to_console: true,
            log_to_file: false,
            log_file_path: None,
        })
    }

    /// Initialize the audit logger
    pub async fn initialize(&self) -> AuditResult<()> {
        info!("Initializing audit logger");
        Ok(())
    }

    /// Log an audit event
    pub async fn log_event(&mut self, event: AuditEvent) -> AuditResult<()> {
        // Validate event
        self.validate_event(&event)?;

        // Add to in-memory store
        {
            let mut events = self.events.write().await;
            events.push(event.clone());

            // Maintain size limit
            if events.len() > self.max_events {
                let remove_count = events.len() - self.max_events;
                events.drain(0..remove_count);
            }
        }

        // Log to console if enabled
        if self.log_to_console {
            self.log_to_console(&event);
        }

        // Log to file if enabled
        if self.log_to_file {
            if let Some(file_path) = &self.log_file_path {
                self.log_to_file(&event, file_path).await?;
            }
        }

        Ok(())
    }

    /// Log authentication event
    pub async fn log_authentication_event(
        &mut self,
        actor: &str,
        success: bool,
        auth_method: &str,
        error_message: Option<&str>,
    ) -> AuditResult<()> {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: "authentication".to_string(),
            actor: actor.to_string(),
            action: if success { "login_success" } else { "login_failure" }.to_string(),
            resource: "system".to_string(),
            success,
            severity: if success { AuditSeverity::Info } else { AuditSeverity::Warning },
            error_message: error_message.map(|s| s.to_string()),
            resource_id: None,
            task_id: None,
            pr_number: None,
            ip_address: None,
            user_agent: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("auth_method".to_string(), auth_method.into());
                meta
            },
        };

        self.log_event(event).await
    }

    /// Log authorization event
    pub async fn log_authorization_event(
        &mut self,
        actor: &str,
        action: &str,
        resource: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> AuditResult<()> {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: "authorization".to_string(),
            actor: actor.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            success,
            severity: if success { AuditSeverity::Info } else { AuditSeverity::Warning },
            error_message: error_message.map(|s| s.to_string()),
            resource_id: None,
            task_id: None,
            pr_number: None,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
        };

        self.log_event(event).await
    }

    /// Log input validation event
    pub async fn log_input_validation_event(
        &mut self,
        actor: &str,
        input_type: &str,
        success: bool,
        violations: Vec<String>,
    ) -> AuditResult<()> {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: "input_validation".to_string(),
            actor: actor.to_string(),
            action: "validate_input".to_string(),
            resource: input_type.to_string(),
            success,
            severity: if success { AuditSeverity::Info } else { AuditSeverity::Warning },
            error_message: if violations.is_empty() {
                None
            } else {
                Some(format!("Violations: {}", violations.join(", ")))
            },
            resource_id: None,
            task_id: None,
            pr_number: None,
            ip_address: None,
            user_agent: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("violation_count".to_string(), violations.len().into());
                meta
            },
        };

        self.log_event(event).await
    }

    /// Log rate limiting event
    pub async fn log_rate_limit_event(
        &mut self,
        actor: &str,
        resource: &str,
        limit_exceeded: bool,
        current_count: u32,
        limit: u32,
    ) -> AuditResult<()> {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: "rate_limiting".to_string(),
            actor: actor.to_string(),
            action: if limit_exceeded { "limit_exceeded" } else { "within_limit" }.to_string(),
            resource: resource.to_string(),
            success: !limit_exceeded,
            severity: if limit_exceeded { AuditSeverity::Warning } else { AuditSeverity::Info },
            error_message: if limit_exceeded {
                Some(format!("Rate limit exceeded: {}/{}", current_count, limit))
            } else {
                None
            },
            resource_id: None,
            task_id: None,
            pr_number: None,
            ip_address: None,
            user_agent: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("current_count".to_string(), current_count.into());
                meta.insert("limit".to_string(), limit.into());
                meta
            },
        };

        self.log_event(event).await
    }

    /// Log privilege escalation attempt
    pub async fn log_privilege_escalation(
        &mut self,
        actor: &str,
        attempted_action: &str,
        error_message: &str,
    ) -> AuditResult<()> {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: "privilege_escalation".to_string(),
            actor: actor.to_string(),
            action: attempted_action.to_string(),
            resource: "system".to_string(),
            success: false,
            severity: AuditSeverity::Critical,
            error_message: Some(error_message.to_string()),
            resource_id: None,
            task_id: None,
            pr_number: None,
            ip_address: None,
            user_agent: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("escalation_attempt".to_string(), true.into());
                meta
            },
        };

        self.log_event(event).await
    }

    /// Validate audit event
    fn validate_event(&self, event: &AuditEvent) -> AuditResult<()> {
        if event.actor.is_empty() {
            return Err(AuditError::LoggingError("Actor cannot be empty".to_string()));
        }

        if event.action.is_empty() {
            return Err(AuditError::LoggingError("Action cannot be empty".to_string()));
        }

        if event.resource.is_empty() {
            return Err(AuditError::LoggingError("Resource cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Log event to console
    fn log_to_console(&self, event: &AuditEvent) {
        let level = match event.severity {
            AuditSeverity::Info => "INFO",
            AuditSeverity::Warning => "WARN",
            AuditSeverity::Error => "ERROR",
            AuditSeverity::Critical => "CRIT",
        };

        let status = if event.success { "SUCCESS" } else { "FAILURE" };

        let message = format!(
            "[AUDIT] {} {} {} {} {} {} - {}",
            event.timestamp.format("%Y-%m-%d %H:%M:%S"),
            level,
            event.event_type,
            event.actor,
            event.action,
            status,
            event.resource
        );

        match event.severity {
            AuditSeverity::Info => info!("{}", message),
            AuditSeverity::Warning => warn!("{}", message),
            AuditSeverity::Error | AuditSeverity::Critical => error!("{}", message),
        }

        if let Some(error) = &event.error_message {
            warn!("[AUDIT] Error: {}", error);
        }
    }

    /// Log event to file
    async fn log_to_file(&self, event: &AuditEvent, file_path: &str) -> AuditResult<()> {
        // In a real implementation, this would write to a file
        // For now, just log that we would write to file
        debug!("Would write audit event to file: {}", file_path);
        Ok(())
    }

    /// Get audit statistics
    pub async fn get_statistics(&self) -> AuditResult<HashMap<String, u64>> {
        let events = self.events.read().await;
        let mut stats = HashMap::new();

        stats.insert("total_events".to_string(), events.len() as u64);

        let successful_events = events.iter().filter(|e| e.success).count() as u64;
        stats.insert("successful_events".to_string(), successful_events);

        let failed_events = events.len() as u64 - successful_events;
        stats.insert("failed_events".to_string(), failed_events);

        // Count by severity
        let critical_events = events.iter()
            .filter(|e| matches!(e.severity, AuditSeverity::Critical))
            .count() as u64;
        stats.insert("critical_events".to_string(), critical_events);

        let error_events = events.iter()
            .filter(|e| matches!(e.severity, AuditSeverity::Error))
            .count() as u64;
        stats.insert("error_events".to_string(), error_events);

        let warning_events = events.iter()
            .filter(|e| matches!(e.severity, AuditSeverity::Warning))
            .count() as u64;
        stats.insert("warning_events".to_string(), warning_events);

        // Count by event type
        let mut event_types = HashMap::new();
        for event in events.iter() {
            *event_types.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        for (event_type, count) in event_types {
            stats.insert(format!("{}_events", event_type), count);
        }

        Ok(stats)
    }

    /// Get recent audit events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        let start = if events.len() > limit {
            events.len() - limit
        } else {
            0
        };

        events[start..].to_vec()
    }

    /// Search audit events
    pub async fn search_events(&self, filters: HashMap<String, String>) -> Vec<AuditEvent> {
        let events = self.events.read().await;

        events.iter().filter(|event| {
            for (key, value) in &filters {
                match key.as_str() {
                    "actor" => if event.actor != *value { return false; }
                    "action" => if event.action != *value { return false; }
                    "resource" => if event.resource != *value { return false; }
                    "event_type" => if event.event_type != *value { return false; }
                    "success" => {
                        let success_filter = value.parse::<bool>().unwrap_or(true);
                        if event.success != success_filter { return false; }
                    }
                    "severity" => {
                        let severity_matches = match value.as_str() {
                            "info" => matches!(event.severity, AuditSeverity::Info),
                            "warning" => matches!(event.severity, AuditSeverity::Warning),
                            "error" => matches!(event.severity, AuditSeverity::Error),
                            "critical" => matches!(event.severity, AuditSeverity::Critical),
                            _ => false,
                        };
                        if !severity_matches { return false; }
                    }
                    _ => {} // Unknown filter, ignore
                }
            }
            true
        }).cloned().collect()
    }

    /// Check if audit logger is healthy
    pub async fn is_healthy(&self) -> bool {
        // Check if we can access the events store
        self.events.try_read().is_ok()
    }

    /// Clear audit events (for testing or maintenance)
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
        info!("Cleared all audit events");
    }

    /// Export audit events to JSON
    pub async fn export_events(&self) -> AuditResult<String> {
        let events = self.events.read().await;
        serde_json::to_string(&*events)
            .map_err(|e| AuditError::LoggingError(format!("Export failed: {}", e)))
    }

    /// Configure file logging
    pub fn enable_file_logging(&mut self, file_path: String) {
        self.log_to_file = true;
        self.log_file_path = Some(file_path);
    }

    /// Disable console logging
    pub fn disable_console_logging(&mut self) {
        self.log_to_console = false;
    }

    /// Update maximum events to keep in memory
    pub fn set_max_events(&mut self, max_events: usize) {
        self.max_events = max_events;
    }
}
