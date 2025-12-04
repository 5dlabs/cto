//! Play tracker for health monitoring and remediation triggering.

use anyhow::Result;
use chrono::Utc;

use super::batch::PlayBatch;
use super::insights::InsightCollector;
use super::remediate::RemediationEngine;
use super::stage::STAGE_TIMEOUT;
use super::task::TaskState;
use super::types::{Issue, RemediationState};

/// Watches a batch and triggers remediation when things go wrong.
pub struct PlayTracker {
    /// The batch being tracked
    pub batch: PlayBatch,
    /// Insight collector for optimization suggestions
    pub insights: InsightCollector,
    /// Remediation engine
    remediation: RemediationEngine,
}

impl PlayTracker {
    /// Create a new tracker for a batch.
    #[must_use]
    pub fn new(batch: PlayBatch) -> Self {
        Self {
            batch,
            insights: InsightCollector::new(),
            remediation: RemediationEngine::new(),
        }
    }

    /// Load tracker from K8s state.
    pub fn load(namespace: &str) -> Result<Self> {
        let batch = PlayBatch::load_from_k8s(namespace)?;
        Ok(Self::new(batch))
    }

    /// Check all tasks and return any that need intervention.
    #[must_use]
    pub fn check_health(&self) -> Vec<Issue> {
        let mut issues = vec![];

        // Check for stuck tasks (>30 min)
        for task in self.batch.stuck_tasks() {
            if let Some(stage) = task.current_stage() {
                if let Some(duration) = task.stage_duration() {
                    issues.push(Issue::StageTimeout {
                        task_id: task.task_id.clone(),
                        stage,
                        elapsed: duration.to_std().unwrap_or(STAGE_TIMEOUT),
                    });
                }
            }
        }

        // Check for failed tasks needing remediation
        for task in self.batch.tasks_needing_remediation() {
            if let super::types::TaskStatus::Failed { stage, reason, .. } = &task.status {
                issues.push(Issue::NeedsRemediation {
                    task_id: task.task_id.clone(),
                    stage: *stage,
                    failure_reason: reason.clone(),
                });
            }
        }

        issues
    }

    /// Spawn a code-fixing remediation for an issue.
    pub async fn remediate(&self, issue: &Issue) -> Result<RemediationState> {
        // 1. Gather context (logs, code, agent output)
        let context = self.remediation.gather_context(issue, &self.batch).await?;

        // 2. Diagnose root cause
        let diagnosis = self.remediation.diagnose(&context).await?;

        // 3. Spawn Healer CodeRun to fix the code
        let coderun_name = self.remediation.spawn_fix_coderun(&diagnosis).await?;

        Ok(RemediationState {
            coderun_name,
            diagnosis: diagnosis.summary,
            started_at: Utc::now(),
        })
    }

    /// Get a summary of the current batch health.
    #[must_use]
    pub fn health_summary(&self) -> HealthSummary {
        let total = self.batch.tasks.len();
        let completed = self.batch.tasks.iter().filter(|t| t.is_completed()).count();
        let running = self.batch.running_tasks().len();
        let stuck = self.batch.stuck_tasks().len();
        let failed = self
            .batch
            .tasks
            .iter()
            .filter(|t| matches!(t.status, super::types::TaskStatus::Failed { .. }))
            .count();
        let pending = self
            .batch
            .tasks
            .iter()
            .filter(|t| matches!(t.status, super::types::TaskStatus::Pending))
            .count();

        let issues = self.check_health();

        HealthSummary {
            total,
            completed,
            running,
            stuck,
            failed,
            pending,
            issues,
            progress: self.batch.progress(),
            elapsed_mins: self.batch.elapsed().num_minutes(),
        }
    }

    /// Get tasks grouped by health status.
    #[must_use]
    pub fn tasks_by_health(&self) -> TasksByHealth {
        let mut healthy = vec![];
        let mut stuck = vec![];
        let mut failed = vec![];
        let mut pending = vec![];

        for task in &self.batch.tasks {
            if task.is_completed() {
                healthy.push(task);
            } else if task.is_stuck() {
                stuck.push(task);
            } else if task.needs_remediation() || task.has_active_remediation() {
                failed.push(task);
            } else if matches!(task.status, super::types::TaskStatus::Pending) {
                pending.push(task);
            } else {
                healthy.push(task);
            }
        }

        TasksByHealth {
            healthy,
            stuck,
            failed,
            pending,
        }
    }

    /// Get a specific task by ID.
    #[must_use]
    pub fn get_task(&self, task_id: &str) -> Option<&TaskState> {
        self.batch.get_task(task_id)
    }
}

/// Summary of batch health status.
#[derive(Debug)]
pub struct HealthSummary {
    /// Total number of tasks
    pub total: usize,
    /// Completed tasks
    pub completed: usize,
    /// Currently running tasks
    pub running: usize,
    /// Stuck tasks (>30 min in stage)
    pub stuck: usize,
    /// Failed tasks
    pub failed: usize,
    /// Pending tasks
    pub pending: usize,
    /// Active issues
    pub issues: Vec<Issue>,
    /// Progress percentage
    pub progress: f64,
    /// Elapsed time in minutes
    pub elapsed_mins: i64,
}

impl HealthSummary {
    /// Check if the batch is healthy.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.stuck == 0 && self.issues.is_empty()
    }

    /// Get the overall status string.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        if self.completed == self.total && self.total > 0 {
            "Completed"
        } else if self.stuck > 0 || !self.issues.is_empty() {
            "Critical"
        } else if self.failed > 0 {
            "Warning"
        } else {
            "Healthy"
        }
    }
}

/// Tasks grouped by health status.
pub struct TasksByHealth<'a> {
    /// Healthy/completed tasks
    pub healthy: Vec<&'a TaskState>,
    /// Stuck tasks
    pub stuck: Vec<&'a TaskState>,
    /// Failed tasks
    pub failed: Vec<&'a TaskState>,
    /// Pending tasks
    pub pending: Vec<&'a TaskState>,
}

