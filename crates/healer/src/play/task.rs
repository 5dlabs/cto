//! Individual task state tracking.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use super::stage::Stage;
use super::types::{Issue, RemediationState, TaskStatus};

/// Individual task within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    /// Task identifier (e.g., "1", "2", "3")
    pub task_id: String,
    /// Current status
    pub status: TaskStatus,
    /// PR number once created
    pub pr_number: Option<u32>,
    /// Active `CodeRun` name
    pub active_coderun: Option<String>,
    /// Detected issues
    pub issues: Vec<Issue>,
    /// Workflow name in K8s
    pub workflow_name: Option<String>,
}

impl TaskState {
    /// Create a new pending task.
    #[must_use]
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::Pending,
            pr_number: None,
            active_coderun: None,
            issues: Vec::new(),
            workflow_name: None,
        }
    }

    /// Create a task in progress at a specific stage.
    #[must_use]
    pub fn in_progress(task_id: impl Into<String>, stage: Stage) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::InProgress {
                stage,
                stage_started: Utc::now(),
            },
            pr_number: None,
            active_coderun: None,
            issues: Vec::new(),
            workflow_name: None,
        }
    }

    /// Get the current stage if in progress or failed.
    #[must_use]
    pub fn current_stage(&self) -> Option<Stage> {
        self.status.current_stage()
    }

    /// Get how long the task has been in the current stage.
    #[must_use]
    pub fn stage_duration(&self) -> Option<Duration> {
        if let TaskStatus::InProgress { stage_started, .. } = &self.status {
            Some(Utc::now().signed_duration_since(*stage_started))
        } else {
            None
        }
    }

    /// Check if this task is stuck (>30 min in current stage).
    #[must_use]
    pub fn is_stuck(&self) -> bool {
        if let Some(duration) = self.stage_duration() {
            duration > Duration::minutes(30)
        } else {
            false
        }
    }

    /// Check if this task needs remediation.
    #[must_use]
    pub fn needs_remediation(&self) -> bool {
        matches!(
            &self.status,
            TaskStatus::Failed {
                remediation: None,
                ..
            }
        )
    }

    /// Check if this task has an active remediation.
    #[must_use]
    pub fn has_active_remediation(&self) -> bool {
        matches!(
            &self.status,
            TaskStatus::Failed {
                remediation: Some(_),
                ..
            }
        )
    }

    /// Check if this task is completed.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }

    /// Check if this task is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    /// Check if this task is healthy (not stuck, not failed without remediation).
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        match &self.status {
            TaskStatus::Pending | TaskStatus::Completed => true,
            TaskStatus::InProgress { .. } => !self.is_stuck(),
            TaskStatus::Failed { remediation, .. } => remediation.is_some(),
        }
    }

    /// Get a health indicator string.
    #[must_use]
    pub fn health_indicator(&self) -> &'static str {
        if self.is_completed() {
            "healthy"
        } else if self.is_stuck() || self.needs_remediation() {
            "critical"
        } else if self.has_active_remediation() {
            "warning"
        } else if matches!(self.status, TaskStatus::Pending) {
            "pending"
        } else {
            "healthy"
        }
    }

    /// Set the task as failed with a reason.
    pub fn fail(&mut self, stage: Stage, reason: impl Into<String>) {
        self.status = TaskStatus::Failed {
            stage,
            reason: reason.into(),
            remediation: None,
        };
    }

    /// Set remediation state for a failed task.
    pub fn set_remediation(&mut self, remediation: RemediationState) {
        if let TaskStatus::Failed {
            remediation: ref mut r,
            ..
        } = &mut self.status
        {
            *r = Some(remediation);
        }
    }

    /// Mark the task as completed.
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
    }

    /// Transition to a new stage.
    pub fn transition_to(&mut self, stage: Stage) {
        self.status = TaskStatus::InProgress {
            stage,
            stage_started: Utc::now(),
        };
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stage_str = self
            .current_stage()
            .map_or_else(|| "-".to_string(), |s| s.to_string());

        let duration_str = self.stage_duration().map_or_else(
            || "-".to_string(),
            |d| format!("{}m", d.num_minutes()),
        );

        let pr_str = self
            .pr_number
            .map_or_else(|| "-".to_string(), |n| format!("#{n}"));

        write!(
            f,
            "Task {} | {} | {} | PR: {}",
            self.task_id, stage_str, duration_str, pr_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let task = TaskState::new("1");
        assert_eq!(task.task_id, "1");
        assert!(matches!(task.status, TaskStatus::Pending));
        assert!(!task.is_stuck());
        assert!(task.is_healthy());
    }

    #[test]
    fn test_task_in_progress() {
        let task = TaskState::in_progress("2", Stage::ImplementationInProgress);
        assert_eq!(task.current_stage(), Some(Stage::ImplementationInProgress));
        assert!(task.is_running());
        assert!(!task.is_completed());
    }

    #[test]
    fn test_task_fail_and_remediate() {
        let mut task = TaskState::in_progress("3", Stage::QualityInProgress);
        task.fail(Stage::QualityInProgress, "Test failure");

        assert!(task.needs_remediation());
        assert!(!task.is_healthy());

        task.set_remediation(RemediationState {
            coderun_name: "healer-fix-123".to_string(),
            diagnosis: "Test diagnosis".to_string(),
            started_at: Utc::now(),
        });

        assert!(!task.needs_remediation());
        assert!(task.has_active_remediation());
    }
}

