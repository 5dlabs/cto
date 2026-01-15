//! Sync task command - synchronizes a task with a Linear issue.
//!
//! This command implements the `intake sync-task` workflow:
//! 1. Fetch the Linear issue by ID
//! 2. Parse the issue content into task structure
//! 3. Update local task files based on Linear issue edits

use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::linear_parser::{parse_linear_issue, ParsedLinearTask};
use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};
use crate::storage::Storage;

/// Configuration for the sync-task command.
#[derive(Debug, Clone, Default)]
pub struct SyncTaskConfig {
    /// Linear issue ID to sync from.
    pub issue_id: String,

    /// Project name/identifier.
    pub project_name: String,

    /// Local task ID to update (if syncing to existing task).
    pub task_id: Option<String>,

    /// Linear API token (from environment or config).
    pub linear_token: Option<String>,

    /// Tag context for task storage.
    pub tag: Option<String>,
}

/// Result of the sync-task command.
#[derive(Debug, Clone)]
pub struct SyncTaskResult {
    /// The synced task.
    pub task: Task,

    /// Whether the task was created (true) or updated (false).
    pub created: bool,

    /// Files that were changed.
    pub changed_files: Vec<PathBuf>,

    /// The parsed Linear task data.
    pub parsed: ParsedLinearTask,
}

/// Sync task domain for fetching Linear issues and updating local tasks.
pub struct SyncTaskDomain {
    storage: Arc<dyn Storage>,
}

impl SyncTaskDomain {
    /// Create a new sync task domain.
    #[must_use]
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Run the sync workflow.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Linear API token is not configured
    /// - Failed to fetch the Linear issue
    /// - Failed to parse the issue content
    /// - Failed to save the task
    pub async fn run(&self, config: &SyncTaskConfig) -> TasksResult<SyncTaskResult> {
        // Get Linear API token
        let token = config
            .linear_token
            .clone()
            .or_else(|| std::env::var("LINEAR_API_KEY").ok())
            .ok_or_else(|| TasksError::ConfigError {
                reason: "LINEAR_API_KEY environment variable not set".to_string(),
            })?;

        tracing::info!(issue_id = %config.issue_id, "Fetching Linear issue");

        // Create Linear client and fetch issue
        let client = pm::LinearClient::new(&token).map_err(|e| TasksError::ConfigError {
            reason: format!("Failed to create Linear client: {e}"),
        })?;

        let issue = client
            .get_issue(&config.issue_id)
            .await
            .map_err(|e| TasksError::Internal {
                reason: format!("Failed to fetch Linear issue: {e}"),
            })?;

        tracing::info!(
            identifier = %issue.identifier,
            title = %issue.title,
            "Fetched Linear issue"
        );

        // Extract labels as strings
        let labels: Vec<String> = issue.labels.iter().map(|l| l.name.clone()).collect();

        // Parse the issue content
        let parsed = parse_linear_issue(
            &issue.title,
            issue.description.as_deref(),
            &labels,
            issue.priority,
        );

        // Determine task ID
        let task_id = config
            .task_id
            .clone()
            .unwrap_or_else(|| issue.identifier.clone());

        // Check if task already exists
        let existing_task = self
            .storage
            .load_tasks(config.tag.as_deref())
            .await
            .ok()
            .and_then(|tasks| tasks.into_iter().find(|t| t.id == task_id));

        let (task, created) = if let Some(mut existing) = existing_task {
            // Update existing task
            tracing::info!(task_id = %task_id, "Updating existing task");

            existing.title.clone_from(&parsed.title);
            existing.description.clone_from(&parsed.description);

            // Update details with acceptance criteria
            if !parsed.acceptance_criteria.is_empty() {
                existing.details = parsed.acceptance_criteria.join("\n- ");
                if !existing.details.is_empty() {
                    existing.details = format!("- {}", existing.details);
                }
            }

            // Update test strategy if present
            if let Some(ref strategy) = parsed.test_strategy {
                existing.test_strategy.clone_from(strategy);
            }

            // Update priority if present
            if let Some(linear_priority) = parsed.priority {
                existing.priority = map_linear_priority(linear_priority);
            }

            // Update agent hint if present
            if let Some(ref hint) = parsed.agent_hint {
                existing.agent_hint = Some(hint.clone());
            }

            existing.updated_at = Some(chrono::Utc::now());

            self.storage
                .update_task(&task_id, &existing, config.tag.as_deref())
                .await?;
            (existing, false)
        } else {
            // Create new task
            tracing::info!(task_id = %task_id, "Creating new task");

            let mut task = Task::new(&task_id, &parsed.title, &parsed.description);

            // Set details from acceptance criteria
            if !parsed.acceptance_criteria.is_empty() {
                task.details = parsed.acceptance_criteria.join("\n- ");
                if !task.details.is_empty() {
                    task.details = format!("- {}", task.details);
                }
            }

            // Set test strategy if present
            if let Some(ref strategy) = parsed.test_strategy {
                task.test_strategy.clone_from(strategy);
            }

            // Set priority if present
            if let Some(linear_priority) = parsed.priority {
                task.priority = map_linear_priority(linear_priority);
            }

            // Set agent hint if present
            if let Some(ref hint) = parsed.agent_hint {
                task.agent_hint = Some(hint.clone());
            }

            self.storage
                .add_task(task.clone(), config.tag.as_deref())
                .await?;
            (task, true)
        };

        // Compute changed files
        let changed_files = vec![
            PathBuf::from(".tasks/tasks/tasks.json"),
            PathBuf::from(format!(".tasks/docs/{}/prompt.md", task.id)),
            PathBuf::from(format!(".tasks/docs/{}/prompt.xml", task.id)),
            PathBuf::from(format!(".tasks/docs/{}/acceptance.md", task.id)),
        ];

        Ok(SyncTaskResult {
            task,
            created,
            changed_files,
            parsed,
        })
    }
}

/// Map Linear priority to task priority.
///
/// Linear priority: 1 = urgent, 2 = high, 3 = normal, 4 = low, 0 = no priority
fn map_linear_priority(linear_priority: i32) -> crate::entities::TaskPriority {
    use crate::entities::TaskPriority;

    match linear_priority {
        1 => TaskPriority::Critical,
        2 => TaskPriority::High,
        4 => TaskPriority::Low,
        _ => TaskPriority::Medium,
    }
}

/// Execute the sync-task command with the given configuration.
///
/// # Errors
///
/// Returns an error if the sync workflow fails.
pub async fn execute(
    storage: Arc<dyn Storage>,
    config: SyncTaskConfig,
) -> TasksResult<SyncTaskResult> {
    let domain = SyncTaskDomain::new(storage);
    domain.run(&config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncTaskConfig::default();
        assert!(config.issue_id.is_empty());
        assert!(config.project_name.is_empty());
        assert!(config.task_id.is_none());
        assert!(config.linear_token.is_none());
    }

    #[test]
    fn test_map_linear_priority() {
        use crate::entities::TaskPriority;

        assert!(matches!(map_linear_priority(1), TaskPriority::Critical));
        assert!(matches!(map_linear_priority(2), TaskPriority::High));
        assert!(matches!(map_linear_priority(3), TaskPriority::Medium));
        assert!(matches!(map_linear_priority(4), TaskPriority::Low));
        assert!(matches!(map_linear_priority(0), TaskPriority::Medium));
    }
}
