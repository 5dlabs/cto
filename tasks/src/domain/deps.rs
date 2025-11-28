//! Dependency domain facade.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};
use crate::storage::Storage;

/// Dependency domain facade providing dependency management operations
pub struct DependencyDomain {
    storage: Arc<dyn Storage>,
}

/// Result of dependency validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub invalid_deps: Vec<InvalidDep>,
    pub cycles: Vec<Vec<String>>,
}

/// Invalid dependency information
#[derive(Debug, Clone)]
pub struct InvalidDep {
    pub task_id: String,
    pub dep_id: String,
    pub reason: String,
}

impl DependencyDomain {
    /// Create a new dependency domain
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Add a dependency to a task
    pub async fn add_dependency(
        &self,
        task_id: &str,
        depends_on: &str,
        tag: Option<&str>,
    ) -> TasksResult<()> {
        let mut tasks = self.storage.load_tasks(tag).await?;

        // Verify both tasks exist
        let task_exists = tasks.iter().any(|t| t.id == task_id);
        let dep_exists = tasks.iter().any(|t| t.id == depends_on);

        if !task_exists {
            return Err(TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            });
        }

        if !dep_exists {
            return Err(TasksError::InvalidDependency {
                task_id: task_id.to_string(),
                dep_id: depends_on.to_string(),
            });
        }

        // Check for cycles
        if Self::would_create_cycle(&tasks, task_id, depends_on) {
            return Err(TasksError::CircularDependency {
                cycle: vec![task_id.to_string(), depends_on.to_string()],
            });
        }

        // Add the dependency
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            if !task.dependencies.contains(&depends_on.to_string()) {
                task.dependencies.push(depends_on.to_string());
            }
        }

        self.storage.save_tasks(&tasks, tag).await
    }

    /// Remove a dependency from a task
    pub async fn remove_dependency(
        &self,
        task_id: &str,
        depends_on: &str,
        tag: Option<&str>,
    ) -> TasksResult<()> {
        let mut tasks = self.storage.load_tasks(tag).await?;

        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.dependencies.retain(|d| d != depends_on);
        } else {
            return Err(TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            });
        }

        self.storage.save_tasks(&tasks, tag).await
    }

    /// Validate all dependencies
    pub async fn validate(&self, tag: Option<&str>) -> TasksResult<ValidationResult> {
        let tasks = self.storage.load_tasks(tag).await?;
        let task_ids: HashSet<_> = tasks.iter().map(|t| t.id.as_str()).collect();

        let mut invalid_deps = Vec::new();
        let mut cycles = Vec::new();

        // Check for invalid references
        for task in &tasks {
            for dep in &task.dependencies {
                if !task_ids.contains(dep.as_str()) {
                    invalid_deps.push(InvalidDep {
                        task_id: task.id.clone(),
                        dep_id: dep.clone(),
                        reason: "Task does not exist".to_string(),
                    });
                }
            }
        }

        // Check for cycles
        let found_cycles = Self::find_cycles(&tasks);
        cycles.extend(found_cycles);

        Ok(ValidationResult {
            is_valid: invalid_deps.is_empty() && cycles.is_empty(),
            invalid_deps,
            cycles,
        })
    }

    /// Fix invalid dependencies by removing them
    pub async fn fix(&self, tag: Option<&str>) -> TasksResult<usize> {
        let mut tasks = self.storage.load_tasks(tag).await?;
        let task_ids: HashSet<_> = tasks.iter().map(|t| t.id.clone()).collect();

        let mut fixed_count = 0;

        for task in &mut tasks {
            let before_len = task.dependencies.len();
            task.dependencies.retain(|d| task_ids.contains(d));
            fixed_count += before_len - task.dependencies.len();
        }

        if fixed_count > 0 {
            self.storage.save_tasks(&tasks, tag).await?;
        }

        Ok(fixed_count)
    }

    /// Check if adding a dependency would create a cycle
    fn would_create_cycle(
        tasks: &[Task],
        task_id: &str,
        depends_on: &str,
    ) -> bool {
        // Build dependency graph
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for task in tasks {
            graph.insert(
                task.id.as_str(),
                task.dependencies.iter().map(String::as_str).collect(),
            );
        }

        // Temporarily add the new dependency
        graph
            .entry(task_id)
            .or_default()
            .push(depends_on);

        // Check if depends_on can reach task_id (which would mean a cycle)
        let mut visited = HashSet::new();
        let mut stack = vec![depends_on];

        while let Some(current) = stack.pop() {
            if current == task_id {
                return true;
            }

            if visited.insert(current) {
                if let Some(deps) = graph.get(current) {
                    stack.extend(deps.iter());
                }
            }
        }

        false
    }

    /// Find all cycles in the dependency graph
    fn find_cycles(tasks: &[Task]) -> Vec<Vec<String>> {
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for task in tasks {
            graph.insert(
                task.id.as_str(),
                task.dependencies.iter().map(String::as_str).collect(),
            );
        }

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task in tasks {
            if !visited.contains(task.id.as_str()) {
                let mut path = Vec::new();
                Self::dfs_cycle(
                    &graph,
                    task.id.as_str(),
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_cycle<'a>(
        graph: &HashMap<&'a str, Vec<&'a str>>,
        node: &'a str,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
        path: &mut Vec<&'a str>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    Self::dfs_cycle(graph, dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&n| n == *dep).unwrap();
                    let cycle: Vec<String> = path[cycle_start..]
                        .iter()
                        .map(|s| (*s).to_string())
                        .collect();
                    if !cycles.contains(&cycle) {
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Get tasks that depend on a given task
    pub async fn get_dependents(&self, task_id: &str, tag: Option<&str>) -> TasksResult<Vec<Task>> {
        let tasks = self.storage.load_tasks(tag).await?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.dependencies.contains(&task_id.to_string()))
            .collect())
    }

    /// Get tasks that a given task depends on
    pub async fn get_dependencies(
        &self,
        task_id: &str,
        tag: Option<&str>,
    ) -> TasksResult<Vec<Task>> {
        let tasks = self.storage.load_tasks(tag).await?;
        let task = tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| TasksError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;

        let dep_ids: Vec<String> = task.dependencies.clone();
        Ok(tasks
            .into_iter()
            .filter(|t| dep_ids.contains(&t.id))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::FileStorage;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, DependencyDomain, Arc<dyn Storage>) {
        let temp_dir = TempDir::new().unwrap();
        let storage: Arc<dyn Storage> = Arc::new(FileStorage::new(temp_dir.path()));
        storage.initialize().await.unwrap();
        let domain = DependencyDomain::new(Arc::clone(&storage));
        (temp_dir, domain, storage)
    }

    #[tokio::test]
    async fn test_add_dependency() {
        let (_temp, domain, storage) = setup().await;

        storage
            .add_task(Task::new("1", "Task 1", "Desc"), None)
            .await
            .unwrap();
        storage
            .add_task(Task::new("2", "Task 2", "Desc"), None)
            .await
            .unwrap();

        domain.add_dependency("2", "1", None).await.unwrap();

        let tasks = storage.load_tasks(None).await.unwrap();
        let task2 = tasks.iter().find(|t| t.id == "2").unwrap();
        assert!(task2.dependencies.contains(&"1".to_string()));
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let (_temp, domain, storage) = setup().await;

        storage
            .add_task(Task::new("1", "Task 1", "Desc"), None)
            .await
            .unwrap();
        storage
            .add_task(Task::new("2", "Task 2", "Desc"), None)
            .await
            .unwrap();

        domain.add_dependency("2", "1", None).await.unwrap();

        // This should fail due to cycle
        let result = domain.add_dependency("1", "2", None).await;
        assert!(matches!(result, Err(TasksError::CircularDependency { .. })));
    }

    #[tokio::test]
    async fn test_validate() {
        let (_temp, domain, storage) = setup().await;

        let mut task = Task::new("1", "Task 1", "Desc");
        task.dependencies = vec!["nonexistent".to_string()];
        storage.add_task(task, None).await.unwrap();

        let result = domain.validate(None).await.unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.invalid_deps.len(), 1);
    }

    #[tokio::test]
    async fn test_fix() {
        let (_temp, domain, storage) = setup().await;

        let mut task = Task::new("1", "Task 1", "Desc");
        task.dependencies = vec!["nonexistent".to_string()];
        storage.add_task(task, None).await.unwrap();

        let fixed = domain.fix(None).await.unwrap();
        assert_eq!(fixed, 1);

        let result = domain.validate(None).await.unwrap();
        assert!(result.is_valid);
    }
}

