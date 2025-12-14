//! Types for platform self-healing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alertmanager webhook payload.
///
/// Reference: <https://prometheus.io/docs/alerting/latest/configuration/#webhook_config>
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertmanagerPayload {
    /// Version of the payload format
    pub version: String,
    /// Unique identifier for this group of alerts
    pub group_key: String,
    /// Number of truncated alerts (if any)
    #[serde(default)]
    pub truncated_alerts: u32,
    /// Status: "firing" or "resolved"
    pub status: String,
    /// Receiver that matched this alert
    pub receiver: String,
    /// Labels common to all alerts in this group
    #[serde(default)]
    pub group_labels: HashMap<String, String>,
    /// Labels common to all alerts (may include group labels)
    #[serde(default)]
    pub common_labels: HashMap<String, String>,
    /// Annotations common to all alerts
    #[serde(default)]
    pub common_annotations: HashMap<String, String>,
    /// External URL for Alertmanager
    #[serde(default)]
    pub external_url: String,
    /// List of alerts in this notification
    pub alerts: Vec<AlertmanagerAlert>,
}

/// Individual alert from Alertmanager.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertmanagerAlert {
    /// Status: "firing" or "resolved"
    pub status: String,
    /// Alert labels
    pub labels: HashMap<String, String>,
    /// Alert annotations
    pub annotations: HashMap<String, String>,
    /// When the alert started firing
    pub starts_at: DateTime<Utc>,
    /// When the alert was resolved (if resolved)
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    /// URL to the alert in Alertmanager
    #[serde(default)]
    pub generator_url: String,
    /// Unique fingerprint for this alert
    #[serde(default)]
    pub fingerprint: String,
}

impl AlertmanagerAlert {
    /// Get the alert name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.labels.get("alertname").map_or("unknown", String::as_str)
    }

    /// Get the severity.
    #[must_use]
    pub fn severity(&self) -> &str {
        self.labels.get("severity").map_or("unknown", String::as_str)
    }

    /// Get the component label.
    #[must_use]
    pub fn component(&self) -> Option<&str> {
        self.labels.get("component").map(String::as_str)
    }

    /// Get the platform label.
    #[must_use]
    pub fn platform(&self) -> Option<&str> {
        self.labels.get("platform").map(String::as_str)
    }

    /// Get the namespace label.
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.labels.get("namespace").map(String::as_str)
    }

    /// Get the pod label.
    #[must_use]
    pub fn pod(&self) -> Option<&str> {
        self.labels.get("pod").map(String::as_str)
    }

    /// Check if this is a firing alert.
    #[must_use]
    pub fn is_firing(&self) -> bool {
        self.status == "firing"
    }

    /// Get the summary annotation.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.annotations
            .get("summary")
            .map_or("No summary", String::as_str)
    }

    /// Get the description annotation.
    #[must_use]
    pub fn description(&self) -> &str {
        self.annotations
            .get("description")
            .map_or("No description", String::as_str)
    }
}

/// Processed platform alert with context.
#[derive(Debug, Clone, Serialize)]
pub struct PlatformAlert {
    /// Alert name
    pub name: String,
    /// Severity (critical, warning, info)
    pub severity: String,
    /// Affected component (controller, healer, pm, etc.)
    pub component: String,
    /// Target platform (cto, argo)
    pub platform: String,
    /// Namespace where issue occurred
    pub namespace: String,
    /// Pod name (if applicable)
    pub pod: Option<String>,
    /// Summary of the issue
    pub summary: String,
    /// Detailed description
    pub description: String,
    /// When the alert started
    pub started_at: DateTime<Utc>,
    /// Unique fingerprint
    pub fingerprint: String,
    /// All labels for context
    pub labels: HashMap<String, String>,
}

impl From<AlertmanagerAlert> for PlatformAlert {
    fn from(alert: AlertmanagerAlert) -> Self {
        Self {
            name: alert.name().to_string(),
            severity: alert.severity().to_string(),
            component: alert.component().unwrap_or("unknown").to_string(),
            platform: alert.platform().unwrap_or("unknown").to_string(),
            namespace: alert.namespace().unwrap_or("cto").to_string(),
            pod: alert.pod().map(ToString::to_string),
            summary: alert.summary().to_string(),
            description: alert.description().to_string(),
            started_at: alert.starts_at,
            fingerprint: alert.fingerprint.clone(),
            labels: alert.labels.clone(),
        }
    }
}

/// A platform issue that needs remediation.
#[derive(Debug, Clone, Serialize)]
pub struct PlatformIssue {
    /// Type of issue
    pub issue_type: PlatformIssueType,
    /// The triggering alert
    pub alert: PlatformAlert,
    /// Logs from affected pods (fetched from Loki)
    pub logs: String,
    /// Diagnosis summary
    pub diagnosis: Option<String>,
    /// Remediation target
    pub target: RemediationTarget,
}

/// Types of platform issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformIssueType {
    /// Component pod is down
    ComponentDown,
    /// Component is crash-looping
    CrashLoop,
    /// Component has high resource usage
    HighResourceUsage,
    /// CodeRun is stuck
    CodeRunStuck,
    /// CodeRun failed
    CodeRunFailed,
    /// Workflow step is stuck
    WorkflowStepStuck,
    /// Workflow failed
    WorkflowFailed,
    /// High workflow failure rate
    HighFailureRate,
    /// Unknown/other
    Unknown,
}

impl PlatformIssueType {
    /// Parse from alert name.
    #[must_use]
    pub fn from_alert_name(name: &str) -> Self {
        match name {
            "CTOControllerDown" | "CTOHealerDown" | "CTOPMDown" => Self::ComponentDown,
            "CTOControllerCrashLoop" => Self::CrashLoop,
            "CTOHighMemory" | "CTOBuildKitNotReady" => Self::HighResourceUsage,
            "CTOCodeRunStuck" => Self::CodeRunStuck,
            "CTOCodeRunFailed" => Self::CodeRunFailed,
            "ArgoWorkflowStepStuck" | "ArgoWorkflowPendingTooLong" => Self::WorkflowStepStuck,
            "ArgoWorkflowFailed" | "ArgoWorkflowControllerDown" => Self::WorkflowFailed,
            "ArgoWorkflowHighFailureRate" => Self::HighFailureRate,
            _ => Self::Unknown,
        }
    }

    /// Get the remediation agent for this issue type.
    #[must_use]
    pub fn remediation_agent(&self) -> &'static str {
        match self {
            Self::ComponentDown | Self::CrashLoop | Self::HighResourceUsage => "bolt",
            Self::CodeRunStuck | Self::CodeRunFailed => "rex",
            Self::WorkflowStepStuck | Self::WorkflowFailed | Self::HighFailureRate => "bolt",
            Self::Unknown => "atlas",
        }
    }
}

/// Target for remediation.
#[derive(Debug, Clone, Serialize)]
pub struct RemediationTarget {
    /// Repository to remediate
    pub repository: String,
    /// GitHub App to use
    pub github_app: String,
    /// Branch to target (usually main)
    pub branch: String,
    /// Agent to use for remediation
    pub agent: String,
    /// CLI tool to use
    pub cli: String,
    /// Model to use
    pub model: String,
}

impl Default for RemediationTarget {
    fn default() -> Self {
        Self {
            repository: "5dlabs/cto".to_string(),
            github_app: "5DLabs-Bolt".to_string(),
            branch: "main".to_string(),
            agent: "bolt".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

impl RemediationTarget {
    /// Create a target for a specific agent.
    #[must_use]
    pub fn for_agent(agent: &str) -> Self {
        let github_app = match agent {
            "rex" => "5DLabs-Rex",
            "blaze" => "5DLabs-Blaze",
            "bolt" => "5DLabs-Bolt",
            "cipher" => "5DLabs-Cipher",
            "atlas" => "5DLabs-Atlas",
            _ => "5DLabs-Bolt",
        };

        Self {
            agent: agent.to_string(),
            github_app: github_app.to_string(),
            ..Default::default()
        }
    }
}

/// Status of a platform remediation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    /// Remediation detected, pending spawn
    Pending,
    /// CodeRun spawned, in progress
    InProgress,
    /// Remediation succeeded
    Succeeded,
    /// Remediation failed
    Failed,
    /// Escalated to human
    Escalated,
}

/// Tracked platform remediation.
#[derive(Debug, Clone, Serialize)]
pub struct TrackedRemediation {
    /// Alert fingerprint (used for deduplication)
    pub fingerprint: String,
    /// The issue being remediated
    pub issue: PlatformIssue,
    /// Name of the spawned CodeRun
    pub coderun_name: Option<String>,
    /// Current status
    pub status: RemediationStatus,
    /// When remediation started
    pub started_at: DateTime<Utc>,
    /// When remediation completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of attempts
    pub attempts: u32,
}

impl TrackedRemediation {
    /// Create a new tracked remediation.
    #[must_use]
    pub fn new(issue: PlatformIssue) -> Self {
        Self {
            fingerprint: issue.alert.fingerprint.clone(),
            issue,
            coderun_name: None,
            status: RemediationStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            attempts: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_type_from_alert_name() {
        assert_eq!(
            PlatformIssueType::from_alert_name("CTOControllerDown"),
            PlatformIssueType::ComponentDown
        );
        assert_eq!(
            PlatformIssueType::from_alert_name("CTOControllerCrashLoop"),
            PlatformIssueType::CrashLoop
        );
        assert_eq!(
            PlatformIssueType::from_alert_name("ArgoWorkflowFailed"),
            PlatformIssueType::WorkflowFailed
        );
    }

    #[test]
    fn test_remediation_agent() {
        assert_eq!(
            PlatformIssueType::ComponentDown.remediation_agent(),
            "bolt"
        );
        assert_eq!(
            PlatformIssueType::CodeRunStuck.remediation_agent(),
            "rex"
        );
    }

    #[test]
    fn test_remediation_target() {
        let target = RemediationTarget::for_agent("rex");
        assert_eq!(target.agent, "rex");
        assert_eq!(target.github_app, "5DLabs-Rex");
        assert_eq!(target.repository, "5dlabs/cto");
    }
}
