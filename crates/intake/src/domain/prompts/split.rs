//! Task splitting functionality for Session 2 preparation.
//!
//! Splits a `tasks.json` file into individual `task-N.json` files
//! to enable bounded-context prompt generation.

use std::path::Path;

use serde_json::Value;

use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};

/// Result of splitting tasks.json into individual files.
#[derive(Debug, Clone, Default)]
pub struct SplitTasksResult {
    /// Number of task files created.
    pub files_created: usize,
    /// Paths to the created task files.
    pub task_files: Vec<String>,
}

/// Split tasks.json into individual task-N.json files.
///
/// This prepares tasks for Session 2 prompt generation by creating
/// individual files that can be processed one at a time with bounded context.
///
/// # Arguments
///
/// * `tasks_json_path` - Path to the tasks.json file
/// * `output_dir` - Directory to write individual task files
///
/// # Returns
///
/// Result containing the number of files created and their paths.
pub async fn split_tasks(
    tasks_json_path: &Path,
    output_dir: &Path,
) -> TasksResult<SplitTasksResult> {
    // Read and parse tasks.json
    let content = tokio::fs::read_to_string(tasks_json_path)
        .await
        .map_err(|e| TasksError::FileReadError {
            path: tasks_json_path.display().to_string(),
            reason: e.to_string(),
        })?;

    let json: Value = serde_json::from_str(&content).map_err(|e| TasksError::JsonParseError {
        reason: format!("Failed to parse tasks.json: {e}"),
    })?;

    // Extract tasks array
    let tasks = json
        .get("tasks")
        .and_then(|t| t.as_array())
        .ok_or_else(|| TasksError::JsonParseError {
            reason: "tasks.json must contain a 'tasks' array".to_string(),
        })?;

    // Create output directory
    tokio::fs::create_dir_all(output_dir)
        .await
        .map_err(|e| TasksError::FileWriteError {
            path: output_dir.display().to_string(),
            reason: e.to_string(),
        })?;

    let mut result = SplitTasksResult::default();

    for task_value in tasks {
        // Parse task to get ID
        let task: Task =
            serde_json::from_value(task_value.clone()).map_err(|e| TasksError::JsonParseError {
                reason: format!("Failed to parse task: {e}"),
            })?;

        // Normalize task ID for filename
        let task_id = if task.id.starts_with("task-") {
            task.id.clone()
        } else {
            format!("task-{}", task.id)
        };

        let file_name = format!("{task_id}.json");
        let file_path = output_dir.join(&file_name);

        // Write individual task file with pretty formatting
        let task_json =
            serde_json::to_string_pretty(task_value).map_err(|e| TasksError::JsonParseError {
                reason: format!("Failed to serialize task: {e}"),
            })?;

        tokio::fs::write(&file_path, task_json)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: file_path.display().to_string(),
                reason: e.to_string(),
            })?;

        result.files_created += 1;
        result.task_files.push(file_path.display().to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_split_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_json_path = temp_dir.path().join("tasks.json");
        let output_dir = temp_dir.path().join("output");

        // Create a sample tasks.json
        let tasks_json = r#"{
            "tasks": [
                {
                    "id": "1",
                    "title": "Task One",
                    "description": "First task",
                    "status": "pending",
                    "priority": "medium",
                    "dependencies": [],
                    "details": "",
                    "testStrategy": "",
                    "subtasks": []
                },
                {
                    "id": "2",
                    "title": "Task Two",
                    "description": "Second task",
                    "status": "pending",
                    "priority": "high",
                    "dependencies": ["1"],
                    "details": "",
                    "testStrategy": "",
                    "subtasks": []
                }
            ]
        }"#;

        tokio::fs::write(&tasks_json_path, tasks_json)
            .await
            .unwrap();

        let result = split_tasks(&tasks_json_path, &output_dir).await.unwrap();

        assert_eq!(result.files_created, 2);
        assert!(output_dir.join("task-1.json").exists());
        assert!(output_dir.join("task-2.json").exists());
    }

    #[tokio::test]
    async fn test_split_tasks_with_prefixed_ids() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_json_path = temp_dir.path().join("tasks.json");
        let output_dir = temp_dir.path().join("output");

        // Create tasks.json with already-prefixed IDs
        let tasks_json = r#"{
            "tasks": [
                {
                    "id": "task-1",
                    "title": "Task One",
                    "description": "First task",
                    "status": "pending",
                    "priority": "medium",
                    "dependencies": [],
                    "details": "",
                    "testStrategy": "",
                    "subtasks": []
                }
            ]
        }"#;

        tokio::fs::write(&tasks_json_path, tasks_json)
            .await
            .unwrap();

        let result = split_tasks(&tasks_json_path, &output_dir).await.unwrap();

        assert_eq!(result.files_created, 1);
        // Should not double-prefix
        assert!(output_dir.join("task-1.json").exists());
        assert!(!output_dir.join("task-task-1.json").exists());
    }
}
