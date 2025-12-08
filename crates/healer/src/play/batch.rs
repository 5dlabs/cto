//! Batch tracking for parallel task execution.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;

use super::stage::Stage;
use super::task::TaskState;
use super::types::{BatchStatus, TaskStatus};

/// A batch of parallel tasks being executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayBatch {
    /// Project-level identifier
    pub project_name: String,
    /// Target repository
    pub repository: String,
    /// Kubernetes namespace
    pub namespace: String,
    /// All tasks in the batch
    pub tasks: Vec<TaskState>,
    /// When the batch started
    pub started_at: DateTime<Utc>,
    /// Current batch status
    pub status: BatchStatus,
}

impl PlayBatch {
    /// Create a new batch.
    #[must_use]
    pub fn new(
        project_name: impl Into<String>,
        repository: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            project_name: project_name.into(),
            repository: repository.into(),
            namespace: namespace.into(),
            tasks: Vec::new(),
            started_at: Utc::now(),
            status: BatchStatus::InProgress {
                completed: 0,
                total: 0,
            },
        }
    }

    /// Load batch state from Kubernetes `ConfigMaps`.
    ///
    /// Queries for all `play-task-*` `ConfigMaps` in the specified namespace
    /// and reconstructs the batch state.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl fails or JSON parsing fails.
    pub fn load_from_k8s(namespace: &str) -> Result<Self> {
        // Get all play-task-* ConfigMaps
        let output = Command::new("kubectl")
            .args([
                "get",
                "configmaps",
                "-n",
                namespace,
                "-l",
                "app.kubernetes.io/component=play-state",
                "-o",
                "json",
            ])
            .output()
            .context("Failed to query ConfigMaps")?;

        if !output.status.success() {
            // Try alternative: get by name pattern
            let output = Command::new("kubectl")
                .args(["get", "configmaps", "-n", namespace, "-o", "json"])
                .output()
                .context("Failed to query ConfigMaps")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("kubectl get configmaps failed: {stderr}");
            }

            return Self::parse_configmaps(&output.stdout, namespace);
        }

        Self::parse_configmaps(&output.stdout, namespace)
    }

    /// Parse `ConfigMap` JSON output into a batch.
    fn parse_configmaps(json_output: &[u8], namespace: &str) -> Result<Self> {
        let json_str = String::from_utf8_lossy(json_output);
        let value: serde_json::Value =
            serde_json::from_str(&json_str).context("Failed to parse ConfigMap JSON")?;

        let empty_vec = Vec::new();
        let items = value["items"].as_array().unwrap_or(&empty_vec);

        let mut batch = Self::new("play", "unknown", namespace);
        let mut repository = String::new();

        for item in items {
            let name = item["metadata"]["name"].as_str().unwrap_or("");

            // Filter to play-task-* ConfigMaps
            if !name.starts_with("play-task-") {
                continue;
            }

            // Extract task ID from name (play-task-{id})
            let task_id = name.strip_prefix("play-task-").unwrap_or(name).to_string();

            // Parse data fields
            let data = &item["data"];
            let stage_str = data["stage"].as_str().unwrap_or("pending");
            let status_str = data["status"].as_str().unwrap_or("in-progress");

            // Get repository from ConfigMap if available
            if let Some(repo) = data["repository"].as_str() {
                if repository.is_empty() {
                    repository = repo.to_string();
                }
            }

            // Parse stage
            let stage = Stage::from_configmap_value(stage_str).unwrap_or(Stage::Pending);

            // Create task state
            let mut task = TaskState::new(&task_id);

            match status_str {
                "completed" | "done" => {
                    task.status = TaskStatus::Completed;
                }
                "failed" | "error" => {
                    let reason = data["error"]
                        .as_str()
                        .unwrap_or("Unknown error")
                        .to_string();
                    task.status = TaskStatus::Failed {
                        stage,
                        reason,
                        remediation: None,
                    };
                }
                _ => {
                    // Parse start time if available
                    let started = data["last-updated"]
                        .as_str()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

                    task.status = TaskStatus::InProgress {
                        stage,
                        stage_started: started,
                    };
                }
            }

            // Get PR number if available
            if let Some(pr) = data["pr-number"].as_str() {
                task.pr_number = pr.parse().ok();
            }

            // Get workflow name
            if let Some(wf) = data["workflow-name"].as_str() {
                task.workflow_name = Some(wf.to_string());
            }

            // Get active CodeRun
            if let Some(cr) = data["coderun-name"].as_str() {
                task.active_coderun = Some(cr.to_string());
            }

            batch.tasks.push(task);
        }

        // Sort tasks by ID
        batch.tasks.sort_by(|a, b| a.task_id.cmp(&b.task_id));

        // Update repository
        if !repository.is_empty() {
            batch.repository = repository;
        }

        // Update batch status
        batch.update_status();

        Ok(batch)
    }

    /// Update the batch status based on task states.
    pub fn update_status(&mut self) {
        let completed = self.tasks.iter().filter(|t| t.is_completed()).count();
        let failed: Vec<String> = self
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Failed { .. }))
            .map(|t| t.task_id.clone())
            .collect();
        let total = self.tasks.len();

        if !failed.is_empty() && completed + failed.len() == total {
            self.status = BatchStatus::Failed {
                failed_tasks: failed,
            };
        } else if completed == total && total > 0 {
            self.status = BatchStatus::Completed;
        } else {
            self.status = BatchStatus::InProgress { completed, total };
        }
    }

    /// Get overall progress as percentage.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let completed = self.tasks.iter().filter(|t| t.is_completed()).count();
        (completed as f64 / self.tasks.len() as f64) * 100.0
    }

    /// Get tasks that are stuck (>30 min in current stage).
    #[must_use]
    pub fn stuck_tasks(&self) -> Vec<&TaskState> {
        self.tasks.iter().filter(|t| t.is_stuck()).collect()
    }

    /// Get tasks that need remediation.
    #[must_use]
    pub fn tasks_needing_remediation(&self) -> Vec<&TaskState> {
        self.tasks
            .iter()
            .filter(|t| t.needs_remediation())
            .collect()
    }

    /// Get tasks that are currently running.
    #[must_use]
    pub fn running_tasks(&self) -> Vec<&TaskState> {
        self.tasks.iter().filter(|t| t.is_running()).collect()
    }

    /// Get a task by ID.
    #[must_use]
    pub fn get_task(&self, task_id: &str) -> Option<&TaskState> {
        self.tasks.iter().find(|t| t.task_id == task_id)
    }

    /// Get a mutable task by ID.
    pub fn get_task_mut(&mut self, task_id: &str) -> Option<&mut TaskState> {
        self.tasks.iter_mut().find(|t| t.task_id == task_id)
    }

    /// Check if any tasks are unhealthy.
    #[must_use]
    pub fn has_unhealthy_tasks(&self) -> bool {
        self.tasks.iter().any(|t| !t.is_healthy())
    }

    /// Get count of unhealthy tasks.
    #[must_use]
    pub fn unhealthy_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.is_healthy()).count()
    }

    /// Get elapsed time since batch started.
    #[must_use]
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.started_at)
    }
}

impl std::fmt::Display for PlayBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match &self.status {
            BatchStatus::InProgress { completed, total } => {
                format!("In Progress ({completed}/{total} tasks)")
            }
            BatchStatus::Completed => "Completed".to_string(),
            BatchStatus::Failed { failed_tasks } => {
                format!("Failed ({} tasks)", failed_tasks.len())
            }
        };

        write!(
            f,
            "Batch: {} | {} | {} | {}m elapsed",
            self.project_name,
            self.repository,
            status_str,
            self.elapsed().num_minutes()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_batch() {
        let batch = PlayBatch::new("test-project", "5dlabs/test", "cto");
        assert_eq!(batch.project_name, "test-project");
        assert_eq!(batch.repository, "5dlabs/test");
        assert!(batch.tasks.is_empty());
        assert!((batch.progress() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_batch_progress() {
        let mut batch = PlayBatch::new("test", "repo", "ns");
        batch.tasks.push(TaskState::new("1"));
        batch.tasks.push(TaskState::new("2"));

        assert!((batch.progress() - 0.0).abs() < f64::EPSILON);

        batch.tasks[0].complete();
        assert!((batch.progress() - 50.0).abs() < f64::EPSILON);

        batch.tasks[1].complete();
        assert!((batch.progress() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_batch_status_update() {
        let mut batch = PlayBatch::new("test", "repo", "ns");
        batch.tasks.push(TaskState::new("1"));
        batch.tasks.push(TaskState::new("2"));

        batch.update_status();
        assert!(matches!(
            batch.status,
            BatchStatus::InProgress {
                completed: 0,
                total: 2
            }
        ));

        batch.tasks[0].complete();
        batch.update_status();
        assert!(matches!(
            batch.status,
            BatchStatus::InProgress {
                completed: 1,
                total: 2
            }
        ));

        batch.tasks[1].complete();
        batch.update_status();
        assert!(matches!(batch.status, BatchStatus::Completed));
    }
}
