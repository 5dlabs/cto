use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Task from TaskMaster tasks.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub dependencies: Option<Vec<u32>>,
    pub details: Option<String>,
    #[serde(rename = "testStrategy")]
    pub test_strategy: Option<String>,
    pub subtasks: Option<Vec<SubTask>>,
}

/// Subtask from TaskMaster tasks.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub details: Option<String>,
}

/// TaskMaster file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TasksFile {
    tasks: Vec<Task>,
}

/// Find the tasks.json file in a repository
/// Looks in common locations: .taskmaster/tasks/tasks.json, tasks.json
fn find_tasks_file(repo_path: &Path) -> Option<PathBuf> {
    let candidates = vec![
        repo_path
            .join(".taskmaster")
            .join("tasks")
            .join("tasks.json"),
        repo_path.join(".taskmaster").join("tasks.json"),
        repo_path.join("tasks.json"),
    ];

    candidates.into_iter().find(|candidate| candidate.exists())
}

/// Read tasks from tasks.json file
fn read_tasks_file(repo_path: &Path) -> Result<Vec<Task>> {
    let tasks_file =
        find_tasks_file(repo_path).ok_or_else(|| anyhow!("tasks.json not found in repository"))?;

    info!("Reading tasks from: {}", tasks_file.display());

    let content = fs::read_to_string(&tasks_file)
        .with_context(|| format!("Failed to read tasks file: {}", tasks_file.display()))?;

    let tasks_data: TasksFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse tasks.json: {}", tasks_file.display()))?;

    Ok(tasks_data.tasks)
}

/// Get the next available task based on dependencies, status, and priority
pub fn get_next_task(repo_path: &Path) -> Result<Option<Task>> {
    let tasks = read_tasks_file(repo_path)?;

    // Build a map of task ID -> Task for dependency checking
    let task_map: HashMap<u32, &Task> = tasks.iter().map(|t| (t.id, t)).collect();

    // Filter tasks that are:
    // 1. Not done
    // 2. Have all dependencies satisfied (all deps have status="done")
    let mut available_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|task| {
            // Skip done/completed tasks
            if task.status == "done" || task.status == "completed" {
                return false;
            }

            // Check dependencies
            if let Some(deps) = &task.dependencies {
                for dep_id in deps {
                    if let Some(dep_task) = task_map.get(dep_id) {
                        if dep_task.status != "done" && dep_task.status != "completed" {
                            // Dependency not satisfied
                            return false;
                        }
                    } else {
                        warn!(
                            "Task {} references non-existent dependency {}",
                            task.id, dep_id
                        );
                        // Treat as unsatisfied dependency
                        return false;
                    }
                }
            }

            true
        })
        .collect();

    if available_tasks.is_empty() {
        return Ok(None);
    }

    // Sort by priority (high > medium > low), then by ID (ascending)
    available_tasks.sort_by(|a, b| {
        let priority_order = |p: &Option<String>| match p.as_deref() {
            Some("high") => 0,
            Some("medium") => 1,
            Some("low") => 2,
            _ => 1, // Default to medium
        };

        let a_priority = priority_order(&a.priority);
        let b_priority = priority_order(&b.priority);

        match a_priority.cmp(&b_priority) {
            std::cmp::Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        }
    });

    Ok(available_tasks.first().map(|&t| t.clone()))
}

/// Update task status in tasks.json
pub fn update_task_status(repo_path: &Path, task_id: u32, status: &str) -> Result<()> {
    let tasks_file =
        find_tasks_file(repo_path).ok_or_else(|| anyhow!("tasks.json not found in repository"))?;

    info!(
        "Updating task {} status to {} in: {}",
        task_id,
        status,
        tasks_file.display()
    );

    // Read current tasks
    let content = fs::read_to_string(&tasks_file)
        .with_context(|| format!("Failed to read tasks file: {}", tasks_file.display()))?;

    let mut tasks_data: TasksFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse tasks.json: {}", tasks_file.display()))?;

    // Find and update the task
    let mut found = false;
    for task in &mut tasks_data.tasks {
        if task.id == task_id {
            task.status = status.to_string();
            found = true;
            break;
        }
    }

    if !found {
        return Err(anyhow!("Task {task_id} not found in tasks.json"));
    }

    // Write back to file
    let updated_content =
        serde_json::to_string_pretty(&tasks_data).context("Failed to serialize updated tasks")?;

    fs::write(&tasks_file, updated_content)
        .with_context(|| format!("Failed to write tasks file: {}", tasks_file.display()))?;

    info!("Successfully updated task {} status to {}", task_id, status);

    Ok(())
}

/// Check if any tasks are blocked by unsatisfied dependencies
/// Returns a list of task IDs that have all pending dependencies
#[allow(dead_code)]
pub fn find_blocked_tasks(repo_path: &Path) -> Result<Vec<u32>> {
    let tasks = read_tasks_file(repo_path)?;
    let task_map: HashMap<u32, &Task> = tasks.iter().map(|t| (t.id, t)).collect();

    let mut blocked = Vec::new();

    for task in &tasks {
        // Skip done tasks
        if task.status == "done" || task.status == "completed" {
            continue;
        }

        // Check if this task has dependencies
        if let Some(deps) = &task.dependencies {
            if deps.is_empty() {
                continue;
            }

            // Check if ANY dependency is still pending/in-progress
            let has_incomplete_deps = deps.iter().any(|dep_id| {
                task_map
                    .get(dep_id)
                    .map(|dep| dep.status != "done" && dep.status != "completed")
                    .unwrap_or(true)
            });

            if has_incomplete_deps {
                blocked.push(task.id);
            }
        }
    }

    Ok(blocked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_tasks_file(dir: &TempDir, tasks: &TasksFile) -> PathBuf {
        let taskmaster_dir = dir.path().join(".taskmaster").join("tasks");
        fs::create_dir_all(&taskmaster_dir).unwrap();

        let tasks_file = taskmaster_dir.join("tasks.json");
        let content = serde_json::to_string_pretty(tasks).unwrap();

        let mut file = fs::File::create(&tasks_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        dir.path().to_path_buf()
    }

    #[test]
    fn test_get_next_task_simple() {
        let temp_dir = TempDir::new().unwrap();
        let tasks = TasksFile {
            tasks: vec![
                Task {
                    id: 1,
                    title: "Task 1".to_string(),
                    description: None,
                    status: "done".to_string(),
                    priority: Some("high".to_string()),
                    dependencies: None,
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
                Task {
                    id: 2,
                    title: "Task 2".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: Some("high".to_string()),
                    dependencies: Some(vec![1]),
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
                Task {
                    id: 3,
                    title: "Task 3".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: Some("medium".to_string()),
                    dependencies: None,
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
            ],
        };

        let repo_path = create_test_tasks_file(&temp_dir, &tasks);
        let next = get_next_task(&repo_path).unwrap();

        assert!(next.is_some());
        let next_task = next.unwrap();
        // Task 2 has higher priority than 3, and its dependency is satisfied
        assert_eq!(next_task.id, 2);
    }

    #[test]
    fn test_get_next_task_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let tasks = TasksFile {
            tasks: vec![
                Task {
                    id: 1,
                    title: "Task 1".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: Some("high".to_string()),
                    dependencies: None,
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
                Task {
                    id: 2,
                    title: "Task 2".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: Some("high".to_string()),
                    dependencies: Some(vec![1]),
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
            ],
        };

        let repo_path = create_test_tasks_file(&temp_dir, &tasks);
        let next = get_next_task(&repo_path).unwrap();

        assert!(next.is_some());
        let next_task = next.unwrap();
        // Task 1 has no dependencies, should be selected
        assert_eq!(next_task.id, 1);
    }

    #[test]
    fn test_find_blocked_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let tasks = TasksFile {
            tasks: vec![
                Task {
                    id: 1,
                    title: "Task 1".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: None,
                    dependencies: None,
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
                Task {
                    id: 2,
                    title: "Task 2".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: None,
                    dependencies: Some(vec![1]),
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
                Task {
                    id: 3,
                    title: "Task 3".to_string(),
                    description: None,
                    status: "pending".to_string(),
                    priority: None,
                    dependencies: Some(vec![1, 2]),
                    details: None,
                    test_strategy: None,
                    subtasks: None,
                },
            ],
        };

        let repo_path = create_test_tasks_file(&temp_dir, &tasks);
        let blocked = find_blocked_tasks(&repo_path).unwrap();

        // Tasks 2 and 3 are blocked (all their deps are pending)
        assert_eq!(blocked.len(), 2);
        assert!(blocked.contains(&2));
        assert!(blocked.contains(&3));
    }
}
