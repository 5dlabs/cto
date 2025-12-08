//! State cleanup for ephemeral play tracking.

use anyhow::{Context, Result};
use std::process::Command;

use super::batch::PlayBatch;

/// Clean up play state after a batch completes.
pub struct PlayCleanup {
    /// Namespace for cleanup
    namespace: String,
    /// Whether to force cleanup (even if tasks still running)
    force: bool,
}

impl PlayCleanup {
    /// Create a new cleanup handler.
    #[must_use]
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            force: false,
        }
    }

    /// Enable force cleanup mode.
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Check if cleanup is safe (no running tasks).
    ///
    /// # Errors
    ///
    /// This function currently does not return errors but may in the future.
    pub fn can_cleanup(&self, batch: &PlayBatch) -> Result<bool> {
        if self.force {
            return Ok(true);
        }

        // Check if any tasks are still running
        if batch.running_tasks().is_empty() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Perform cleanup of all play state.
    ///
    /// # Errors
    ///
    /// Returns an error if tasks are still running (without force) or kubectl fails.
    pub fn cleanup(&self, batch: &PlayBatch) -> Result<CleanupReport> {
        let mut report = CleanupReport::default();

        if !self.can_cleanup(batch)? {
            anyhow::bail!(
                "Cannot cleanup: {} tasks still running. Use --force to override.",
                batch.running_tasks().len()
            );
        }

        // Delete play-task-* `ConfigMaps`
        report.configmaps_deleted = self.delete_task_configmaps()?;

        // Delete any remediation `CodeRuns`
        report.coderuns_deleted = self.delete_remediation_coderuns();

        // Delete any play-related workflows
        report.workflows_deleted = self.delete_play_workflows();

        Ok(report)
    }

    /// Delete play-task-* `ConfigMaps`.
    fn delete_task_configmaps(&self) -> Result<usize> {
        // First, list the ConfigMaps
        let output = Command::new("kubectl")
            .args(["get", "configmaps", "-n", &self.namespace, "-o", "name"])
            .output()
            .context("Failed to list ConfigMaps")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let configmaps: Vec<&str> = stdout
            .lines()
            .filter(|l| l.contains("play-task-"))
            .collect();

        if configmaps.is_empty() {
            return Ok(0);
        }

        // Delete each ConfigMap
        let mut deleted = 0;
        for cm in &configmaps {
            let name = cm.trim_start_matches("configmap/");
            let status = Command::new("kubectl")
                .args(["delete", "configmap", name, "-n", &self.namespace])
                .status()
                .context("Failed to delete ConfigMap")?;

            if status.success() {
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// Delete healer remediation `CodeRuns`.
    fn delete_remediation_coderuns(&self) -> usize {
        // List healer-fix-* CodeRuns
        let output = Command::new("kubectl")
            .args([
                "get",
                "coderuns",
                "-n",
                &self.namespace,
                "-l",
                "app.kubernetes.io/name=healer",
                "-o",
                "name",
            ])
            .output();

        // CodeRun CRD might not exist, that's okay
        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return 0,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let coderuns: Vec<&str> = stdout.lines().collect();

        if coderuns.is_empty() {
            return 0;
        }

        // Delete each CodeRun
        let mut deleted = 0;
        for cr in &coderuns {
            let name = cr.trim_start_matches("coderun.cto.5dlabs.io/");
            let status = Command::new("kubectl")
                .args(["delete", "coderun", name, "-n", &self.namespace])
                .status();

            if matches!(status, Ok(s) if s.success()) {
                deleted += 1;
            }
        }

        deleted
    }

    /// Delete play-related workflows.
    fn delete_play_workflows(&self) -> usize {
        // List play-* workflows
        let output = Command::new("kubectl")
            .args(["get", "workflows", "-n", &self.namespace, "-o", "name"])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return 0,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let workflows: Vec<&str> = stdout.lines().filter(|l| l.contains("play-")).collect();

        if workflows.is_empty() {
            return 0;
        }

        // Delete each workflow
        let mut deleted = 0;
        for wf in &workflows {
            let name = wf.trim_start_matches("workflow.argoproj.io/");
            let status = Command::new("kubectl")
                .args(["delete", "workflow", name, "-n", &self.namespace])
                .status();

            if matches!(status, Ok(s) if s.success()) {
                deleted += 1;
            }
        }

        deleted
    }

    /// Cleanup a specific task's state.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl fails.
    pub fn cleanup_task(&self, task_id: &str) -> Result<()> {
        let configmap_name = format!("play-task-{task_id}");

        let status = Command::new("kubectl")
            .args([
                "delete",
                "configmap",
                &configmap_name,
                "-n",
                &self.namespace,
                "--ignore-not-found",
            ])
            .status()
            .context("Failed to delete task ConfigMap")?;

        if !status.success() {
            tracing::warn!("Failed to delete ConfigMap {}", configmap_name);
        }

        Ok(())
    }
}

/// Report of what was cleaned up.
#[derive(Debug, Default)]
pub struct CleanupReport {
    /// Number of `ConfigMaps` deleted
    pub configmaps_deleted: usize,
    /// Number of `CodeRuns` deleted
    pub coderuns_deleted: usize,
    /// Number of Workflows deleted
    pub workflows_deleted: usize,
}

impl CleanupReport {
    /// Get total items cleaned up.
    #[must_use]
    pub fn total(&self) -> usize {
        self.configmaps_deleted + self.coderuns_deleted + self.workflows_deleted
    }

    /// Check if anything was cleaned up.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

impl std::fmt::Display for CleanupReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cleaned up: {} ConfigMaps, {} CodeRuns, {} Workflows",
            self.configmaps_deleted, self.coderuns_deleted, self.workflows_deleted
        )
    }
}
