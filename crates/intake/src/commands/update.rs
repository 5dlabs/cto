//! Update command - re-parses PRD/architecture changes and generates a delta PR.
//!
//! This command implements the `intake update` workflow:
//! 1. Load existing tasks from storage
//! 2. Re-run PRD parsing with updated content
//! 3. Compute diff between old and new tasks
//! 4. Output changed files for delta PR generation

use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::delta::{compute_task_delta, TaskDelta};
use crate::domain::AIDomain;
use crate::errors::TasksResult;
use crate::storage::Storage;

/// Configuration for the update command.
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// Project name/identifier.
    pub project_name: String,

    /// Optional PRD content (if not provided, reads from default path).
    pub prd_content: Option<String>,

    /// Path to the PRD file (used if prd_content is None).
    pub prd_path: PathBuf,

    /// Path to the architecture file (optional).
    pub architecture_path: Option<PathBuf>,

    /// AI model to use (None = default).
    pub model: Option<String>,

    /// Whether to use research mode for AI operations.
    pub research: bool,

    /// Target number of tasks to generate (0 = auto).
    pub num_tasks: i32,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            prd_content: None,
            prd_path: PathBuf::from(".tasks/docs/prd.txt"),
            architecture_path: None,
            model: None,
            research: true,
            num_tasks: 0, // Auto-detect
        }
    }
}

/// Result of the update command.
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// The computed delta between old and new tasks.
    pub delta: TaskDelta,

    /// Files that were changed (paths relative to project root).
    pub changed_files: Vec<PathBuf>,

    /// Total input tokens used.
    pub total_input_tokens: u32,

    /// Total output tokens used.
    pub total_output_tokens: u32,
}

/// Update domain for orchestrating PRD re-parsing and delta computation.
pub struct UpdateDomain {
    storage: Arc<dyn Storage>,
    ai_domain: AIDomain,
}

impl UpdateDomain {
    /// Create a new update domain.
    #[must_use]
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            ai_domain: AIDomain::new(Arc::clone(&storage)),
            storage,
        }
    }

    /// Run the update workflow.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to load existing tasks
    /// - Failed to read PRD content
    /// - Failed to parse PRD
    pub async fn run(&self, config: &UpdateConfig) -> TasksResult<UpdateResult> {
        let mut total_input_tokens = 0u32;
        let mut total_output_tokens = 0u32;

        // 1. Load existing tasks
        tracing::info!(
            "Loading existing tasks for project: {}",
            config.project_name
        );
        let existing_tasks = self.storage.load_tasks(None).await?;
        tracing::info!("Loaded {} existing tasks", existing_tasks.len());

        // 2. Get PRD content
        let prd_content = if let Some(ref content) = config.prd_content {
            content.clone()
        } else {
            tokio::fs::read_to_string(&config.prd_path)
                .await
                .map_err(|e| crate::errors::TasksError::FileReadError {
                    path: config.prd_path.display().to_string(),
                    reason: e.to_string(),
                })?
        };

        // 3. Read architecture content if provided
        let architecture_content = if let Some(arch_path) = &config.architecture_path {
            if arch_path.exists() {
                Some(tokio::fs::read_to_string(arch_path).await.map_err(|e| {
                    crate::errors::TasksError::FileReadError {
                        path: arch_path.display().to_string(),
                        reason: e.to_string(),
                    }
                })?)
            } else {
                None
            }
        } else {
            None
        };

        // 4. Combine PRD with architecture context
        let full_prd = if let Some(arch) = &architecture_content {
            format!(
                "{}\n\n---\n\n## Architecture Context\n\n{}",
                prd_content, arch
            )
        } else {
            prd_content
        };

        // 5. Re-parse PRD to generate new tasks
        tracing::info!("Re-parsing PRD to generate tasks...");
        let (new_tasks, prd_usage) = self
            .ai_domain
            .parse_prd(
                &full_prd,
                config.prd_path.to_str().unwrap_or(""),
                if config.num_tasks > 0 {
                    Some(config.num_tasks)
                } else {
                    None
                },
                config.research,
                config.model.as_deref(),
            )
            .await?;

        total_input_tokens += prd_usage.input_tokens;
        total_output_tokens += prd_usage.output_tokens;

        tracing::info!("Generated {} new tasks", new_tasks.len());

        // 6. Compute delta between old and new tasks
        tracing::info!("Computing task delta...");
        let delta = compute_task_delta(&existing_tasks, &new_tasks);

        tracing::info!(
            "Delta: {} added, {} removed, {} modified, {} unchanged",
            delta.added.len(),
            delta.removed.len(),
            delta.modified.len(),
            delta.unchanged.len()
        );

        // 7. Compute changed files
        let changed_files = self.compute_changed_files(&delta);

        Ok(UpdateResult {
            delta,
            changed_files,
            total_input_tokens,
            total_output_tokens,
        })
    }

    /// Compute the list of files that would be changed by applying the delta.
    fn compute_changed_files(&self, delta: &TaskDelta) -> Vec<PathBuf> {
        let mut files = Vec::new();

        // Tasks file is always changed if there's any delta
        if !delta.added.is_empty() || !delta.removed.is_empty() || !delta.modified.is_empty() {
            files.push(PathBuf::from(".tasks/tasks/tasks.json"));
        }

        // Individual task doc files for added/modified tasks
        for task in &delta.added {
            files.push(PathBuf::from(format!(".tasks/docs/{}/prompt.md", task.id)));
            files.push(PathBuf::from(format!(".tasks/docs/{}/prompt.xml", task.id)));
            files.push(PathBuf::from(format!(
                ".tasks/docs/{}/acceptance.md",
                task.id
            )));
        }

        for (_, new_task) in &delta.modified {
            files.push(PathBuf::from(format!(
                ".tasks/docs/{}/prompt.md",
                new_task.id
            )));
            files.push(PathBuf::from(format!(
                ".tasks/docs/{}/prompt.xml",
                new_task.id
            )));
            files.push(PathBuf::from(format!(
                ".tasks/docs/{}/acceptance.md",
                new_task.id
            )));
        }

        // Mark removed task docs (they would be deleted)
        for task in &delta.removed {
            files.push(PathBuf::from(format!(".tasks/docs/{}/", task.id)));
        }

        files
    }
}

/// Execute the update command with the given configuration.
///
/// # Errors
///
/// Returns an error if the update workflow fails.
pub async fn execute(storage: Arc<dyn Storage>, config: UpdateConfig) -> TasksResult<UpdateResult> {
    let domain = UpdateDomain::new(storage);
    domain.run(&config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert!(config.project_name.is_empty());
        assert!(config.prd_content.is_none());
        assert!(config.research);
        assert_eq!(config.num_tasks, 0);
    }
}
