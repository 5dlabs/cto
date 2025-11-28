//! Storage trait definitions.

use async_trait::async_trait;

use crate::entities::{TagStats, Task, TaskStatus};
use crate::errors::TasksResult;

/// Result of status update operation
#[derive(Debug, Clone)]
pub struct UpdateStatusResult {
    pub success: bool,
    pub task_id: String,
    pub old_status: TaskStatus,
    pub new_status: TaskStatus,
}

/// Storage interface for task persistence
#[async_trait]
pub trait Storage: Send + Sync {
    /// Initialize storage (create directories, etc.)
    async fn initialize(&self) -> TasksResult<()>;

    /// Close and cleanup resources
    async fn close(&self) -> TasksResult<()>;

    /// Get storage type identifier
    fn storage_type(&self) -> &'static str;

    /// Check if storage is initialized
    async fn is_initialized(&self) -> TasksResult<bool>;

    // === Task Operations ===

    /// Load all tasks for a tag
    async fn load_tasks(&self, tag: Option<&str>) -> TasksResult<Vec<Task>>;

    /// Load a single task by ID
    async fn load_task(&self, task_id: &str, tag: Option<&str>) -> TasksResult<Option<Task>>;

    /// Save all tasks for a tag
    async fn save_tasks(&self, tasks: &[Task], tag: Option<&str>) -> TasksResult<()>;

    /// Add a new task
    async fn add_task(&self, task: Task, tag: Option<&str>) -> TasksResult<()>;

    /// Update a single task
    async fn update_task(&self, task_id: &str, task: &Task, tag: Option<&str>) -> TasksResult<()>;

    /// Update task status
    async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        tag: Option<&str>,
    ) -> TasksResult<UpdateStatusResult>;

    /// Delete a task
    async fn delete_task(&self, task_id: &str, tag: Option<&str>) -> TasksResult<()>;

    /// Get next available task ID
    async fn next_task_id(&self, tag: Option<&str>) -> TasksResult<String>;

    // === Tag Operations ===

    /// Get all available tags
    async fn get_all_tags(&self) -> TasksResult<Vec<String>>;

    /// Get tags with detailed statistics
    async fn get_tags_with_stats(&self) -> TasksResult<Vec<TagStats>>;

    /// Create a new tag
    async fn create_tag(
        &self,
        name: &str,
        copy_from: Option<&str>,
        description: Option<&str>,
    ) -> TasksResult<()>;

    /// Delete a tag
    async fn delete_tag(&self, name: &str) -> TasksResult<()>;

    /// Rename a tag
    async fn rename_tag(&self, old_name: &str, new_name: &str) -> TasksResult<()>;

    /// Copy a tag
    async fn copy_tag(
        &self,
        source: &str,
        target: &str,
        description: Option<&str>,
    ) -> TasksResult<()>;

    /// Check if tag exists
    async fn tag_exists(&self, name: &str) -> TasksResult<bool>;

    // === State Operations ===

    /// Get current active tag
    async fn get_current_tag(&self) -> TasksResult<String>;

    /// Set current active tag
    async fn set_current_tag(&self, tag: &str) -> TasksResult<()>;
}

