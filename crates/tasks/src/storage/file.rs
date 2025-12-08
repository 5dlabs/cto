//! File-based storage implementation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use tokio::fs;

use super::traits::{Storage, UpdateStatusResult};
use crate::entities::{
    RuntimeState, SubtaskCounts, TagMetadata, TagStats, TaggedTaskList, Task, TaskStatus,
};
use crate::errors::{TasksError, TasksResult};

/// File-based storage implementation
pub struct FileStorage {
    /// Project root path
    project_path: PathBuf,

    /// Path to tasks directory (.tasks/)
    tasks_dir: PathBuf,

    /// Path to tasks.json
    tasks_file: PathBuf,

    /// Path to config.json (reserved for future use)
    #[allow(dead_code)]
    config_file: PathBuf,

    /// Path to state.json
    state_file: PathBuf,
}

impl FileStorage {
    /// Create a new file storage instance
    ///
    /// Uses `.tasks/` directory for project task storage.
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        let project_path = project_path.as_ref().to_path_buf();
        let tasks_dir = project_path.join(".tasks");
        let tasks_subdir = tasks_dir.join("tasks");
        let tasks_file = tasks_subdir.join("tasks.json");
        let config_file = tasks_dir.join("config.json");
        let state_file = tasks_dir.join("state.json");

        Self {
            project_path,
            tasks_dir,
            tasks_file,
            config_file,
            state_file,
        }
    }

    /// Get the project path
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// Get the tasks directory path
    pub fn tasks_dir(&self) -> &Path {
        &self.tasks_dir
    }

    /// Read and parse the tasks file
    async fn read_tasks_file(&self) -> TasksResult<Value> {
        match fs::read_to_string(&self.tasks_file).await {
            Ok(content) => {
                let data: Value = serde_json::from_str(&content)?;
                Ok(data)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
            Err(e) => Err(TasksError::FileReadError {
                path: self.tasks_file.display().to_string(),
                reason: e.to_string(),
            }),
        }
    }

    /// Write the tasks file
    async fn write_tasks_file(&self, data: &Value) -> TasksResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.tasks_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(data)?;
        fs::write(&self.tasks_file, content)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: self.tasks_file.display().to_string(),
                reason: e.to_string(),
            })
    }

    /// Read the state file
    async fn read_state(&self) -> TasksResult<RuntimeState> {
        match fs::read_to_string(&self.state_file).await {
            Ok(content) => {
                let state: RuntimeState = serde_json::from_str(&content)?;
                Ok(state)
            }
            Err(_) => Ok(RuntimeState::default()),
        }
    }

    /// Write the state file
    async fn write_state(&self, state: &RuntimeState) -> TasksResult<()> {
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_file, content)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: self.state_file.display().to_string(),
                reason: e.to_string(),
            })
    }

    /// Detect format of tasks.json (legacy vs tagged)
    fn detect_format(data: &Value) -> TasksFormat {
        if data.get("tasks").is_some() && data.get("metadata").is_some() {
            TasksFormat::Standard
        } else if data.is_object() {
            // Check if keys look like tag names
            let obj = data.as_object();
            if let Some(obj) = obj {
                // If we have any key that is not "tasks" or "metadata", it's tagged format
                let has_tag_keys = obj.keys().any(|k| k != "tasks" && k != "metadata");
                if has_tag_keys {
                    return TasksFormat::Tagged;
                }
            }
            TasksFormat::Standard
        } else {
            TasksFormat::Standard
        }
    }

    /// Extract tasks from JSON data for a specific tag
    fn extract_tasks(data: &Value, tag: &str) -> Vec<Task> {
        match Self::detect_format(data) {
            TasksFormat::Standard if tag == "master" => data
                .get("tasks")
                .and_then(|v| serde_json::from_value::<Vec<Task>>(v.clone()).ok())
                .unwrap_or_default(),
            TasksFormat::Tagged => data
                .get(tag)
                .and_then(|t| t.get("tasks"))
                .and_then(|v| serde_json::from_value::<Vec<Task>>(v.clone()).ok())
                .unwrap_or_default(),
            TasksFormat::Standard => Vec::new(),
        }
    }

    /// Build metadata for saving
    fn build_metadata(tasks: &[Task]) -> Value {
        let completed_count = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count();

        serde_json::json!({
            "version": "1.0.0",
            "lastModified": Utc::now().to_rfc3339(),
            "taskCount": tasks.len(),
            "completedCount": completed_count,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum TasksFormat {
    Standard, // { "tasks": [...], "metadata": {...} }
    Tagged,   // { "master": { "tasks": [...] }, "feature": { "tasks": [...] } }
}

#[async_trait]
impl Storage for FileStorage {
    async fn initialize(&self) -> TasksResult<()> {
        // Create directory structure
        fs::create_dir_all(self.tasks_dir.join("tasks")).await?;
        fs::create_dir_all(self.tasks_dir.join("reports")).await?;

        // Create initial empty tasks file if it doesn't exist
        if !self.tasks_file.exists() {
            let initial = TaggedTaskList::default();
            let data = serde_json::json!({
                "tasks": initial.tasks,
                "metadata": initial.metadata,
            });
            self.write_tasks_file(&data).await?;
        }

        // Create initial state file if it doesn't exist
        if !self.state_file.exists() {
            let state = RuntimeState::default();
            self.write_state(&state).await?;
        }

        Ok(())
    }

    async fn close(&self) -> TasksResult<()> {
        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        "file"
    }

    async fn is_initialized(&self) -> TasksResult<bool> {
        Ok(self.tasks_dir.exists() && self.tasks_file.exists())
    }

    async fn load_tasks(&self, tag: Option<&str>) -> TasksResult<Vec<Task>> {
        let tag = tag.unwrap_or("master");
        let data = self.read_tasks_file().await?;
        Ok(Self::extract_tasks(&data, tag))
    }

    async fn load_task(&self, task_id: &str, tag: Option<&str>) -> TasksResult<Option<Task>> {
        let tasks = self.load_tasks(tag).await?;

        // Handle subtask notation (e.g., "1.2")
        if task_id.contains('.') {
            let parts: Vec<&str> = task_id.split('.').collect();
            if parts.len() == 2 {
                let parent_id = parts[0];
                let subtask_id: u32 = parts[1].parse().map_err(|_| TasksError::InvalidId {
                    id: task_id.to_string(),
                })?;

                if let Some(parent) = tasks.iter().find(|t| t.id == parent_id) {
                    if let Some(subtask) = parent.subtasks.iter().find(|s| s.id == subtask_id) {
                        // Return subtask as a task-like structure
                        return Ok(Some(Task {
                            id: task_id.to_string(),
                            title: subtask.title.clone(),
                            description: subtask.description.clone(),
                            status: subtask.status,
                            priority: subtask.priority,
                            dependencies: subtask.dependencies.clone(),
                            details: subtask.details.clone(),
                            test_strategy: subtask.test_strategy.clone(),
                            subtasks: Vec::new(),
                            created_at: subtask.created_at,
                            updated_at: subtask.updated_at,
                            effort: None,
                            actual_effort: None,
                            tags: Vec::new(),
                            assignee: subtask.assignee.clone(),
                            complexity: None,
                            agent_hint: None,
                        }));
                    }
                }
                return Ok(None);
            }
        }

        Ok(tasks.into_iter().find(|t| t.id == task_id))
    }

    async fn save_tasks(&self, tasks: &[Task], tag: Option<&str>) -> TasksResult<()> {
        let tag = tag.unwrap_or("master");
        let mut data = self.read_tasks_file().await?;
        let metadata = Self::build_metadata(tasks);

        let format = Self::detect_format(&data);

        match format {
            TasksFormat::Standard if tag == "master" => {
                data = serde_json::json!({
                    "tasks": tasks,
                    "metadata": metadata,
                });
            }
            TasksFormat::Standard => {
                // Need to migrate from standard to tagged format
                // First, extract existing master tasks
                let master_tasks = data.get("tasks").cloned();
                let master_metadata = data.get("metadata").cloned();

                // Create new tagged format structure
                let mut new_data = serde_json::Map::new();

                // Preserve master tag if it had tasks
                if let Some(mt) = master_tasks {
                    new_data.insert(
                        "master".to_string(),
                        serde_json::json!({
                            "tasks": mt,
                            "metadata": master_metadata,
                        }),
                    );
                }

                // Add the new tag
                new_data.insert(
                    tag.to_string(),
                    serde_json::json!({
                        "tasks": tasks,
                        "metadata": metadata,
                    }),
                );

                data = Value::Object(new_data);
            }
            TasksFormat::Tagged => {
                // Update tagged format
                if let Some(obj) = data.as_object_mut() {
                    obj.insert(
                        tag.to_string(),
                        serde_json::json!({
                            "tasks": tasks,
                            "metadata": metadata,
                        }),
                    );
                }
            }
        }

        self.write_tasks_file(&data).await
    }

    async fn add_task(&self, task: Task, tag: Option<&str>) -> TasksResult<()> {
        let mut tasks = self.load_tasks(tag).await?;
        tasks.push(task);
        self.save_tasks(&tasks, tag).await
    }

    async fn update_task(&self, task_id: &str, task: &Task, tag: Option<&str>) -> TasksResult<()> {
        let mut tasks = self.load_tasks(tag).await?;

        if let Some(idx) = tasks.iter().position(|t| t.id == task_id) {
            tasks[idx] = task.clone();
            self.save_tasks(&tasks, tag).await?;
            Ok(())
        } else {
            Err(TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            })
        }
    }

    async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        tag: Option<&str>,
    ) -> TasksResult<UpdateStatusResult> {
        let mut tasks = self.load_tasks(tag).await?;

        // Handle subtask
        if task_id.contains('.') {
            let parts: Vec<&str> = task_id.split('.').collect();
            let parent_id = parts[0];
            let subtask_id: u32 = parts[1].parse().map_err(|_| TasksError::InvalidId {
                id: task_id.to_string(),
            })?;

            if let Some(parent) = tasks.iter_mut().find(|t| t.id == parent_id) {
                if let Some(subtask) = parent.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                    let old_status = subtask.status;
                    subtask.status = status;
                    subtask.updated_at = Some(Utc::now());

                    // Auto-update parent status based on subtasks
                    let all_done = parent
                        .subtasks
                        .iter()
                        .all(|s| matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled));
                    let any_in_progress = parent
                        .subtasks
                        .iter()
                        .any(|s| s.status == TaskStatus::InProgress);
                    let any_done = parent.subtasks.iter().any(|s| s.status == TaskStatus::Done);

                    if all_done && !parent.subtasks.is_empty() {
                        parent.status = TaskStatus::Done;
                    } else if any_in_progress || any_done {
                        parent.status = TaskStatus::InProgress;
                    }

                    parent.updated_at = Some(Utc::now());
                    self.save_tasks(&tasks, tag).await?;

                    return Ok(UpdateStatusResult {
                        success: true,
                        task_id: task_id.to_string(),
                        old_status,
                        new_status: status,
                    });
                }
            }
            return Err(TasksError::SubtaskNotFound {
                task_id: parent_id.to_string(),
                subtask_id: subtask_id.to_string(),
            });
        }

        // Handle regular task
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            let old_status = task.status;
            task.status = status;
            task.updated_at = Some(Utc::now());

            self.save_tasks(&tasks, tag).await?;

            Ok(UpdateStatusResult {
                success: true,
                task_id: task_id.to_string(),
                old_status,
                new_status: status,
            })
        } else {
            Err(TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            })
        }
    }

    async fn delete_task(&self, task_id: &str, tag: Option<&str>) -> TasksResult<()> {
        let mut tasks = self.load_tasks(tag).await?;
        let len_before = tasks.len();
        tasks.retain(|t| t.id != task_id);

        if tasks.len() == len_before {
            return Err(TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            });
        }

        self.save_tasks(&tasks, tag).await
    }

    async fn next_task_id(&self, tag: Option<&str>) -> TasksResult<String> {
        let tasks = self.load_tasks(tag).await?;

        // Find the highest numeric ID
        let max_id = tasks
            .iter()
            .filter_map(|t| t.id.parse::<u32>().ok())
            .max()
            .unwrap_or(0);

        Ok((max_id + 1).to_string())
    }

    async fn get_all_tags(&self) -> TasksResult<Vec<String>> {
        let data = self.read_tasks_file().await?;

        match Self::detect_format(&data) {
            TasksFormat::Standard => Ok(vec!["master".to_string()]),
            TasksFormat::Tagged => {
                let tags: Vec<String> = data.as_object().map_or_else(
                    || vec!["master".to_string()],
                    |o| o.keys().cloned().collect(),
                );
                Ok(tags)
            }
        }
    }

    async fn get_tags_with_stats(&self) -> TasksResult<Vec<TagStats>> {
        let tags = self.get_all_tags().await?;
        let current_tag = self.get_current_tag().await?;
        let data = self.read_tasks_file().await?;

        let mut stats = Vec::new();
        for tag_name in tags {
            let tasks = Self::extract_tasks(&data, &tag_name);

            let mut status_breakdown = HashMap::new();
            let mut completed = 0;
            let mut total_subtasks = 0;
            let mut subtasks_by_status: HashMap<String, usize> = HashMap::new();

            for task in &tasks {
                let status_key = task.status.to_string();
                *status_breakdown.entry(status_key).or_insert(0) += 1;

                if task.status == TaskStatus::Done {
                    completed += 1;
                }

                for subtask in &task.subtasks {
                    total_subtasks += 1;
                    let sub_status = subtask.status.to_string();
                    *subtasks_by_status.entry(sub_status).or_insert(0) += 1;
                }
            }

            // Get metadata
            let tag_data = match Self::detect_format(&data) {
                TasksFormat::Standard if tag_name == "master" => data.get("metadata"),
                TasksFormat::Tagged => data.get(&tag_name).and_then(|t| t.get("metadata")),
                TasksFormat::Standard => None,
            };

            let metadata: Option<TagMetadata> =
                tag_data.and_then(|m| serde_json::from_value(m.clone()).ok());

            stats.push(TagStats {
                name: tag_name.clone(),
                is_current: tag_name == current_tag,
                task_count: tasks.len(),
                completed_tasks: completed,
                status_breakdown,
                subtask_counts: if total_subtasks > 0 {
                    Some(SubtaskCounts {
                        total: total_subtasks,
                        by_status: subtasks_by_status,
                    })
                } else {
                    None
                },
                created: metadata.as_ref().and_then(|m| m.created),
                description: metadata.and_then(|m| m.description),
            });
        }

        Ok(stats)
    }

    async fn create_tag(
        &self,
        name: &str,
        copy_from: Option<&str>,
        _description: Option<&str>,
    ) -> TasksResult<()> {
        let existing_tags = self.get_all_tags().await?;
        if existing_tags.contains(&name.to_string()) {
            return Err(TasksError::TagAlreadyExists {
                name: name.to_string(),
            });
        }

        let tasks_to_copy = if let Some(source) = copy_from {
            self.load_tasks(Some(source)).await?
        } else {
            Vec::new()
        };

        self.save_tasks(&tasks_to_copy, Some(name)).await
    }

    async fn delete_tag(&self, name: &str) -> TasksResult<()> {
        if name == "master" {
            return Err(TasksError::CannotDeleteMasterTag);
        }

        let mut data = self.read_tasks_file().await?;

        if let Some(obj) = data.as_object_mut() {
            if obj.remove(name).is_none() {
                return Err(TasksError::TagNotFound {
                    name: name.to_string(),
                });
            }
        }

        self.write_tasks_file(&data).await
    }

    async fn rename_tag(&self, old_name: &str, new_name: &str) -> TasksResult<()> {
        if old_name == "master" {
            return Err(TasksError::CannotRenameMasterTag);
        }

        let mut data = self.read_tasks_file().await?;

        if let Some(obj) = data.as_object_mut() {
            if let Some(tag_data) = obj.remove(old_name) {
                obj.insert(new_name.to_string(), tag_data);
            } else {
                return Err(TasksError::TagNotFound {
                    name: old_name.to_string(),
                });
            }
        }

        self.write_tasks_file(&data).await?;

        // Update current tag if needed
        let mut state = self.read_state().await?;
        if state.current_tag == old_name {
            state.current_tag = new_name.to_string();
            self.write_state(&state).await?;
        }

        Ok(())
    }

    async fn copy_tag(
        &self,
        source: &str,
        target: &str,
        _description: Option<&str>,
    ) -> TasksResult<()> {
        let tasks = self.load_tasks(Some(source)).await?;
        self.save_tasks(&tasks, Some(target)).await
    }

    async fn tag_exists(&self, name: &str) -> TasksResult<bool> {
        let tags = self.get_all_tags().await?;
        Ok(tags.contains(&name.to_string()))
    }

    async fn get_current_tag(&self) -> TasksResult<String> {
        let state = self.read_state().await?;
        Ok(state.current_tag)
    }

    async fn set_current_tag(&self, tag: &str) -> TasksResult<()> {
        // Verify tag exists
        if !self.tag_exists(tag).await? {
            return Err(TasksError::TagNotFound {
                name: tag.to_string(),
            });
        }

        let mut state = self.read_state().await?;
        state.switch_tag(tag);
        self.write_state(&state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_storage() -> (TempDir, FileStorage) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        storage.initialize().await.unwrap();
        (temp_dir, storage)
    }

    #[tokio::test]
    async fn test_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        assert!(!storage.is_initialized().await.unwrap());

        storage.initialize().await.unwrap();

        assert!(storage.is_initialized().await.unwrap());
        assert!(temp_dir.path().join(".tasks/tasks/tasks.json").exists());
        assert!(temp_dir.path().join(".tasks/state.json").exists());
    }

    #[tokio::test]
    async fn test_load_save_tasks() {
        let (_temp_dir, storage) = setup_storage().await;

        let task = Task::new("1", "Test Task", "Test description");
        storage.add_task(task.clone(), None).await.unwrap();

        let tasks = storage.load_tasks(None).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "1");
        assert_eq!(tasks[0].title, "Test Task");
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let (_temp_dir, storage) = setup_storage().await;

        let task = Task::new("1", "Test", "Desc");
        storage.add_task(task, None).await.unwrap();

        let result = storage
            .update_task_status("1", TaskStatus::InProgress, None)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.old_status, TaskStatus::Pending);
        assert_eq!(result.new_status, TaskStatus::InProgress);

        let loaded = storage.load_task("1", None).await.unwrap().unwrap();
        assert_eq!(loaded.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let (_temp_dir, storage) = setup_storage().await;

        let task = Task::new("1", "Test", "Desc");
        storage.add_task(task, None).await.unwrap();

        storage.delete_task("1", None).await.unwrap();

        let tasks = storage.load_tasks(None).await.unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_tags() {
        let (_temp_dir, storage) = setup_storage().await;

        // Create a new tag
        storage.create_tag("feature-1", None, None).await.unwrap();

        let tags = storage.get_all_tags().await.unwrap();
        assert!(tags.contains(&"master".to_string()));
        assert!(tags.contains(&"feature-1".to_string()));

        // Switch tag
        storage.set_current_tag("feature-1").await.unwrap();
        assert_eq!(storage.get_current_tag().await.unwrap(), "feature-1");

        // Delete tag
        storage.delete_tag("feature-1").await.unwrap();
        let tags = storage.get_all_tags().await.unwrap();
        assert!(!tags.contains(&"feature-1".to_string()));
    }

    #[tokio::test]
    async fn test_next_task_id() {
        let (_temp_dir, storage) = setup_storage().await;

        assert_eq!(storage.next_task_id(None).await.unwrap(), "1");

        storage
            .add_task(Task::new("1", "T1", "D1"), None)
            .await
            .unwrap();
        assert_eq!(storage.next_task_id(None).await.unwrap(), "2");

        storage
            .add_task(Task::new("5", "T5", "D5"), None)
            .await
            .unwrap();
        assert_eq!(storage.next_task_id(None).await.unwrap(), "6");
    }
}
