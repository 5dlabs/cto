//! Notification event types for the CTO platform.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity levels for alerts and notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational - normal operations
    Info,
    /// Warning - something needs attention
    Warning,
    /// Critical - immediate action required
    Critical,
}

impl Severity {
    /// Get the color for this severity (for embeds).
    #[must_use]
    pub const fn color(&self) -> u32 {
        match self {
            Self::Info => 0x0034_98db,     // Blue
            Self::Warning => 0x00f3_9c12,  // Orange
            Self::Critical => 0x00e7_4c3c, // Red
        }
    }

    /// Get display name for this severity.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
        }
    }

    /// Get emoji for this severity (for Slack).
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Critical => "🚨",
        }
    }
}

/// Events that can trigger notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotifyEvent {
    // =========================================================================
    // Play/Task lifecycle events (from Controller watching Argo workflows)
    // =========================================================================
    /// A play workflow has started
    PlayStarted {
        /// Task ID being worked on
        task_id: String,
        /// Repository being modified
        repository: String,
        /// Argo workflow name
        workflow_name: String,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },

    /// A task within a play has started
    TaskStarted {
        /// Task ID
        task_id: String,
        /// Repository being modified
        repository: String,
        /// Agent name (e.g., "cleo", "rex")
        agent: String,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },

    /// A task within a play has completed
    TaskCompleted {
        /// Task ID
        task_id: String,
        /// Repository that was modified
        repository: String,
        /// Agent name
        agent: String,
        /// Whether the task succeeded
        success: bool,
        /// Duration in seconds
        duration_secs: u64,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },

    // =========================================================================
    // Agent lifecycle events (from Controller watching CodeRuns)
    // =========================================================================
    /// An agent (`CodeRun`) has started running
    AgentStarted {
        /// Agent name
        agent: String,
        /// Task ID being worked on
        task_id: String,
        /// `CodeRun` resource name
        coderun_name: String,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },

    /// An agent (`CodeRun`) has completed
    AgentCompleted {
        /// Agent name
        agent: String,
        /// Task ID that was completed
        task_id: String,
        /// `CodeRun` resource name
        coderun_name: String,
        /// Whether the agent succeeded
        success: bool,
        /// Duration in seconds
        duration_secs: u64,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },

    // =========================================================================
    // HEAL events
    // =========================================================================
    /// A HEAL alert was detected (A1-A9, etc.)
    HealAlert {
        /// Alert identifier (e.g., "A7")
        alert_id: String,
        /// Alert severity
        severity: Severity,
        /// Alert message
        message: String,
        /// Additional context
        #[serde(default)]
        context: HashMap<String, String>,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },

    /// A HEAL remediation cycle is starting
    HealRemediation {
        /// Task ID for remediation
        task_id: String,
        /// Remediation iteration number
        iteration: u32,
        /// Repository being remediated
        repository: String,
        /// Reason for remediation
        reason: String,
        /// Event timestamp
        #[serde(default = "Utc::now")]
        timestamp: DateTime<Utc>,
    },
}

impl NotifyEvent {
    /// Get a short title for this event type.
    #[must_use]
    pub fn title(&self) -> String {
        match self {
            Self::PlayStarted { task_id, .. } => format!("Play Started: Task #{task_id}"),
            Self::TaskStarted { agent, task_id, .. } => {
                format!("{agent} Started: Task #{task_id}")
            }
            Self::TaskCompleted {
                agent,
                task_id,
                success,
                ..
            } => {
                let status = if *success { "Completed" } else { "Failed" };
                format!("{agent} {status}: Task #{task_id}")
            }
            Self::AgentStarted { agent, task_id, .. } => {
                format!("Agent {agent} Started: Task #{task_id}")
            }
            Self::AgentCompleted {
                agent,
                task_id,
                success,
                ..
            } => {
                let status = if *success { "Completed" } else { "Failed" };
                format!("Agent {agent} {status}: Task #{task_id}")
            }
            Self::HealAlert { alert_id, .. } => format!("HEAL Alert: {alert_id}"),
            Self::HealRemediation {
                task_id, iteration, ..
            } => {
                format!("HEAL Remediation #{iteration}: Task #{task_id}")
            }
        }
    }

    /// Get the severity/color for this event.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Self::PlayStarted { .. } | Self::TaskStarted { .. } | Self::AgentStarted { .. } => {
                Severity::Info
            }

            Self::TaskCompleted { success, .. } | Self::AgentCompleted { success, .. } => {
                if *success {
                    Severity::Info
                } else {
                    Severity::Warning
                }
            }

            Self::HealAlert { severity, .. } => *severity,
            Self::HealRemediation { .. } => Severity::Warning,
        }
    }

    /// Get the timestamp for this event.
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::PlayStarted { timestamp, .. }
            | Self::TaskStarted { timestamp, .. }
            | Self::TaskCompleted { timestamp, .. }
            | Self::AgentStarted { timestamp, .. }
            | Self::AgentCompleted { timestamp, .. }
            | Self::HealAlert { timestamp, .. }
            | Self::HealRemediation { timestamp, .. } => *timestamp,
        }
    }
}
