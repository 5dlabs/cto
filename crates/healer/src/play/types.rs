//! Shared types for play orchestration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::stage::Stage;

/// Status of a batch of parallel tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch is currently executing
    InProgress {
        /// Number of completed tasks
        completed: usize,
        /// Total number of tasks
        total: usize,
    },
    /// All tasks completed successfully
    Completed,
    /// One or more tasks failed
    Failed {
        /// Task IDs that failed
        failed_tasks: Vec<String>,
    },
}

impl BatchStatus {
    /// Check if the batch is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::InProgress { .. })
    }

    /// Check if the batch completed successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Status of an individual task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to start
    Pending,
    /// Task is currently executing
    InProgress {
        /// Current stage
        stage: Stage,
        /// When this stage started
        stage_started: DateTime<Utc>,
    },
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed {
        /// Stage where failure occurred
        stage: Stage,
        /// Reason for failure
        reason: String,
        /// Active remediation, if any
        remediation: Option<RemediationState>,
    },
}

impl TaskStatus {
    /// Check if the task is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::InProgress { .. })
    }

    /// Get the current stage if in progress.
    #[must_use]
    pub fn current_stage(&self) -> Option<Stage> {
        match self {
            Self::InProgress { stage, .. } => Some(*stage),
            Self::Failed { stage, .. } => Some(*stage),
            _ => None,
        }
    }

    /// Get the stage start time if in progress.
    #[must_use]
    pub fn stage_started(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::InProgress { stage_started, .. } => Some(*stage_started),
            _ => None,
        }
    }
}

/// State of an active remediation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationState {
    /// Name of the Healer CodeRun
    pub coderun_name: String,
    /// Summary of the diagnosis
    pub diagnosis: String,
    /// When remediation started
    pub started_at: DateTime<Utc>,
}

/// An issue detected during play execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Issue {
    /// Task stuck in stage for >30 minutes
    StageTimeout {
        /// Task identifier
        task_id: String,
        /// Stage where stuck
        stage: Stage,
        /// Time elapsed in this stage
        elapsed: Duration,
    },
    /// Task failed and needs code fix
    NeedsRemediation {
        /// Task identifier
        task_id: String,
        /// Stage where failure occurred
        stage: Stage,
        /// Reason for failure
        failure_reason: String,
    },
    /// Agent behaving suboptimally (optimization opportunity)
    OptimizationOpportunity {
        /// Task identifier
        task_id: String,
        /// Agent name
        agent: String,
        /// What was observed
        observation: String,
        /// Suggested prompt improvement
        suggested_prompt_change: String,
    },
}

impl Issue {
    /// Get the task ID associated with this issue.
    #[must_use]
    pub fn task_id(&self) -> &str {
        match self {
            Self::StageTimeout { task_id, .. }
            | Self::NeedsRemediation { task_id, .. }
            | Self::OptimizationOpportunity { task_id, .. } => task_id,
        }
    }

    /// Get a human-readable description of the issue.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::StageTimeout {
                task_id,
                stage,
                elapsed,
            } => {
                format!(
                    "Task {} stuck in {:?} for {}m (>30m threshold)",
                    task_id,
                    stage,
                    elapsed.as_secs() / 60
                )
            }
            Self::NeedsRemediation {
                task_id,
                stage,
                failure_reason,
            } => {
                format!(
                    "Task {} failed in {:?}: {}",
                    task_id, stage, failure_reason
                )
            }
            Self::OptimizationOpportunity {
                task_id,
                agent,
                observation,
                ..
            } => {
                format!(
                    "Task {} ({}) optimization: {}",
                    task_id, agent, observation
                )
            }
        }
    }
}

/// Context gathered for diagnosing an issue.
#[derive(Debug, Clone, Default)]
pub struct DiagnosisContext {
    /// Logs from the failed pod
    pub logs: String,
    /// Agent's last actions/output
    pub agent_output: String,
    /// Relevant source code snippets
    pub code_snippets: Vec<String>,
    /// PR state (if applicable)
    pub pr_state: Option<PrContext>,
    /// CI results (if applicable)
    pub ci_results: Option<String>,
}

/// PR context for diagnosis.
#[derive(Debug, Clone)]
pub struct PrContext {
    /// PR number
    pub number: u32,
    /// PR state (open, closed, merged)
    pub state: String,
    /// Whether PR is mergeable
    pub mergeable: bool,
    /// Check status
    pub checks_status: String,
}

/// Diagnosis result from analyzing an issue.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    /// Summary of what went wrong
    pub summary: String,
    /// Root cause category
    pub category: DiagnosisCategory,
    /// Suggested fix approach
    pub suggested_fix: String,
    /// Relevant files to examine/modify
    pub relevant_files: Vec<String>,
}

/// Category of diagnosed issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosisCategory {
    /// Issue in the source code being modified
    CodeIssue,
    /// Issue in agent prompts/templates
    PromptIssue,
    /// Infrastructure/environment issue
    InfraIssue,
    /// Git/version control issue
    GitIssue,
    /// Unknown/needs investigation
    Unknown,
}

