//! Core types for the alert system.

#![allow(dead_code)] // Public API - methods used by handlers and future integrations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for each alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertId {
    A1,         // Comment order mismatch
    A2,         // Silent agent failure
    A3,         // Stale progress
    A4,         // Repeated approval loop
    A5,         // Post-Tess CI/merge failure
    A7,         // Pod failure
    A8,         // Step timeout
    Completion, // Success completion check
}

impl AlertId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::A4 => "A4",
            Self::A5 => "A5",
            Self::A7 => "A7",
            Self::A8 => "A8",
            Self::Completion => "completion",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::A1 => "Agent Comment Order Mismatch",
            Self::A2 => "Silent Agent Failure",
            Self::A3 => "Stale Progress",
            Self::A4 => "Repeated Approval Loop",
            Self::A5 => "Post-Tess CI/Merge Failure",
            Self::A7 => "Pod Failure",
            Self::A8 => "Workflow Step Timeout",
            Self::Completion => "Success Completion Check",
        }
    }
}

/// A detected alert with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: AlertId,
    pub message: String,
    pub severity: Severity,
    pub context: HashMap<String, String>,
    pub detected_at: DateTime<Utc>,
}

impl Alert {
    pub fn new(id: AlertId, message: impl Into<String>) -> Self {
        Self {
            id,
            message: message.into(),
            severity: Severity::Warning,
            context: HashMap::new(),
            detected_at: Utc::now(),
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Context available to alert handlers
pub struct AlertContext {
    pub task_id: String,
    pub repository: String,
    pub namespace: String,
    pub pr_number: Option<u32>,
    pub workflow_name: Option<String>,
    pub config: AlertConfig,
}

/// Configuration for alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Minutes before stale progress alert (A3)
    pub stale_progress_threshold_mins: u64,
    /// Number of approvals before loop alert (A4)
    pub approval_loop_threshold: u32,
    /// Step timeout thresholds by agent type (A8)
    pub step_timeouts: StepTimeouts,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            stale_progress_threshold_mins: 15,
            approval_loop_threshold: 2,
            step_timeouts: StepTimeouts::default(),
        }
    }
}

/// Timeout thresholds for each agent type (values in minutes)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // _mins suffix is intentional for clarity
pub struct StepTimeouts {
    pub implementation_mins: u64, // Rex/Blaze
    pub quality_mins: u64,        // Cleo
    pub testing_mins: u64,        // Tess
    pub security_mins: u64,       // Cipher
    pub integration_mins: u64,    // Atlas
    pub default_mins: u64,        // Unknown agents
}

impl Default for StepTimeouts {
    fn default() -> Self {
        Self {
            implementation_mins: 45,
            quality_mins: 15,
            testing_mins: 30,
            security_mins: 15,
            integration_mins: 20,
            default_mins: 60,
        }
    }
}

/// Trait for alert handlers
pub trait AlertHandler: Send + Sync {
    /// Unique identifier for this handler
    fn id(&self) -> AlertId;

    /// Human-readable name
    fn name(&self) -> &'static str {
        self.id().name()
    }

    /// Evaluate current state and return an alert if condition is met
    fn evaluate(
        &self,
        event: &crate::k8s::K8sEvent,
        github: &crate::github::GitHubState,
        ctx: &AlertContext,
    ) -> Option<Alert>;
}
