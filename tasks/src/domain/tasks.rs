//! Tasks domain facade.

use std::sync::Arc;

use crate::entities::{Subtask, Task, TaskStatus};
use crate::errors::{TasksError, TasksResult};
use crate::storage::Storage;

/// Tasks domain facade providing high-level task operations
pub struct TasksDomain {
    storage: Arc<dyn Storage>,
}

impl TasksDomain {
    /// Create a new tasks domain
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Initialize the project
    pub async fn init(&self) -> TasksResult<()> {
        self.storage.initialize().await
    }

    /// Check if project is initialized
    pub async fn is_initialized(&self) -> TasksResult<bool> {
        self.storage.is_initialized().await
    }

    /// List all tasks with optional status filter
    pub async fn list_tasks(
        &self,
        tag: Option<&str>,
        status_filter: Option<TaskStatus>,
    ) -> TasksResult<Vec<Task>> {
        let tasks = self.storage.load_tasks(tag).await?;

        if let Some(status) = status_filter {
            Ok(tasks.into_iter().filter(|t| t.status == status).collect())
        } else {
            Ok(tasks)
        }
    }

    /// Get a specific task by ID
    pub async fn get_task(&self, task_id: &str, tag: Option<&str>) -> TasksResult<Task> {
        self.storage
            .load_task(task_id, tag)
            .await?
            .ok_or_else(|| TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            })
    }

    /// Get multiple tasks by IDs
    pub async fn get_tasks(&self, task_ids: &[&str], tag: Option<&str>) -> TasksResult<Vec<Task>> {
        let mut tasks = Vec::new();
        for id in task_ids {
            if let Some(task) = self.storage.load_task(id, tag).await? {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    /// Get the next task to work on
    pub async fn next_task(&self, tag: Option<&str>) -> TasksResult<Option<Task>> {
        let tasks = self.storage.load_tasks(tag).await?;

        // Get list of completed task IDs for dependency checking
        let done_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .map(|t| t.id.as_str())
            .collect();

        // Find first pending task without blocking dependencies
        // Priority order: Critical > High > Medium > Low
        let mut candidates: Vec<&Task> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .filter(|t| !t.has_blocking_deps(&done_ids))
            .collect();

        // Sort by priority (Critical first)
        candidates.sort_by(|a, b| {
            use crate::entities::TaskPriority::{Critical, High, Low, Medium};
            let priority_order = |p: &crate::entities::TaskPriority| match p {
                Critical => 0,
                High => 1,
                Medium => 2,
                Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        Ok(candidates.first().copied().cloned())
    }

    /// Add a new task
    pub async fn add_task(
        &self,
        title: impl Into<String>,
        description: impl Into<String>,
        tag: Option<&str>,
    ) -> TasksResult<Task> {
        let id = self.storage.next_task_id(tag).await?;
        let task = Task::new(id, title, description);
        self.storage.add_task(task.clone(), tag).await?;
        Ok(task)
    }

    /// Add a task with full details
    pub async fn add_task_full(&self, task: Task, tag: Option<&str>) -> TasksResult<()> {
        self.storage.add_task(task, tag).await
    }

    /// Update task status
    pub async fn set_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        tag: Option<&str>,
    ) -> TasksResult<()> {
        self.storage
            .update_task_status(task_id, status, tag)
            .await?;
        Ok(())
    }

    /// Update task details
    pub async fn update_task(&self, task: &Task, tag: Option<&str>) -> TasksResult<()> {
        self.storage.update_task(&task.id, task, tag).await
    }

    /// Remove a task
    pub async fn remove_task(&self, task_id: &str, tag: Option<&str>) -> TasksResult<()> {
        self.storage.delete_task(task_id, tag).await
    }

    /// Add a subtask to a task
    pub async fn add_subtask(
        &self,
        task_id: &str,
        title: impl Into<String>,
        description: impl Into<String>,
        tag: Option<&str>,
    ) -> TasksResult<Subtask> {
        let mut task = self.get_task(task_id, tag).await?;
        let subtask_id = task.next_subtask_id();
        let subtask = Subtask::new(subtask_id, task_id, title, description);
        task.add_subtask(subtask.clone());
        self.storage.update_task(task_id, &task, tag).await?;
        Ok(subtask)
    }

    /// Clear subtasks from a task
    pub async fn clear_subtasks(&self, task_id: &str, tag: Option<&str>) -> TasksResult<usize> {
        let mut task = self.get_task(task_id, tag).await?;
        let count = task.subtasks.len();
        task.clear_subtasks();
        self.storage.update_task(task_id, &task, tag).await?;
        Ok(count)
    }

    /// Move a task to a new position (reorder)
    pub async fn move_task(
        &self,
        task_id: &str,
        new_position: usize,
        tag: Option<&str>,
    ) -> TasksResult<()> {
        let mut tasks = self.storage.load_tasks(tag).await?;

        let current_idx =
            tasks
                .iter()
                .position(|t| t.id == task_id)
                .ok_or_else(|| TasksError::TaskNotFound {
                    task_id: task_id.to_string(),
                })?;

        let task = tasks.remove(current_idx);
        let insert_idx = new_position.min(tasks.len());
        tasks.insert(insert_idx, task);

        self.storage.save_tasks(&tasks, tag).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::FileStorage;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, TasksDomain) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path()));
        storage.initialize().await.unwrap();
        let domain = TasksDomain::new(storage);
        (temp_dir, domain)
    }

    #[tokio::test]
    async fn test_add_and_list_tasks() {
        let (_temp, domain) = setup().await;

        domain
            .add_task("Task 1", "Description 1", None)
            .await
            .unwrap();
        domain
            .add_task("Task 2", "Description 2", None)
            .await
            .unwrap();

        let tasks = domain.list_tasks(None, None).await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_next_task() {
        let (_temp, domain) = setup().await;

        domain
            .add_task("Task 1", "Description 1", None)
            .await
            .unwrap();

        let next = domain.next_task(None).await.unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().title, "Task 1");
    }

    #[tokio::test]
    async fn test_set_status() {
        let (_temp, domain) = setup().await;

        let task = domain
            .add_task("Task 1", "Description 1", None)
            .await
            .unwrap();
        domain
            .set_status(&task.id, TaskStatus::InProgress, None)
            .await
            .unwrap();

        let updated = domain.get_task(&task.id, None).await.unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);
    }
}
