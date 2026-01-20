//! Dependency domain facade.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::entities::{Subtask, Task};
use crate::errors::{TasksError, TasksResult};
use crate::storage::Storage;

/// Result of execution level computation for subtasks.
#[derive(Debug, Clone)]
pub struct ExecutionLevels {
    /// Subtasks grouped by execution level.
    /// Level 0 contains subtasks with no dependencies, level 1 contains subtasks
    /// that depend only on level 0, etc.
    pub levels: Vec<Vec<u32>>,
    /// Statistics about the computed levels.
    pub stats: ExecutionStats,
}

/// Statistics about execution levels.
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total number of subtasks processed.
    pub total_subtasks: usize,
    /// Number of execution levels.
    pub total_levels: usize,
    /// Maximum parallelism (largest level size).
    pub max_parallelism: usize,
    /// Average parallelism across levels.
    pub avg_parallelism: f64,
}

/// Compute execution levels for subtasks within a task.
///
/// This function analyzes subtask dependencies and groups subtasks into parallel
/// execution levels. Subtasks with no unmet dependencies can run in parallel
/// within the same level.
///
/// # Arguments
/// * `subtasks` - Mutable slice of subtasks to compute and set execution levels for
///
/// # Returns
/// * `ExecutionLevels` containing the grouped subtasks and statistics
///
/// # Example
/// ```ignore
/// let mut subtasks = vec![
///     Subtask::new(1, "task-1", "Setup database", "..."),  // no deps
///     Subtask::new(2, "task-1", "Add migrations", "..."),  // depends on 1
///     Subtask::new(3, "task-1", "Setup cache", "..."),     // no deps
/// ];
/// subtasks[1].dependencies = vec!["1".to_string()];
///
/// let levels = compute_subtask_execution_levels(&mut subtasks);
/// // Level 0: [1, 3] (can run in parallel)
/// // Level 1: [2] (runs after level 0)
/// ```
pub fn compute_subtask_execution_levels(subtasks: &mut [Subtask]) -> ExecutionLevels {
    if subtasks.is_empty() {
        return ExecutionLevels {
            levels: Vec::new(),
            stats: ExecutionStats {
                total_subtasks: 0,
                total_levels: 0,
                max_parallelism: 0,
                avg_parallelism: 0.0,
            },
        };
    }

    // Build dependency graph: subtask_id -> set of dependency ids
    let mut deps: HashMap<u32, HashSet<u32>> = HashMap::new();
    let subtask_ids: HashSet<u32> = subtasks.iter().map(|s| s.id).collect();

    for subtask in subtasks.iter() {
        let mut subtask_deps = HashSet::new();
        for dep_str in &subtask.dependencies {
            // Parse dependency - could be "1" or "task-1.1" format
            if let Some(dep_id) = parse_subtask_dep(dep_str, &subtask_ids) {
                subtask_deps.insert(dep_id);
            }
        }
        deps.insert(subtask.id, subtask_deps);
    }

    // Compute levels using topological sort
    let mut task_levels: HashMap<u32, u32> = HashMap::new();
    let mut remaining: HashSet<u32> = subtask_ids.clone();
    let mut levels: Vec<Vec<u32>> = Vec::new();

    while !remaining.is_empty() {
        // Find subtasks whose dependencies are all resolved
        let mut ready: Vec<u32> = Vec::new();

        for &subtask_id in &remaining {
            let subtask_deps = deps.get(&subtask_id).cloned().unwrap_or_default();
            let unmet_deps: HashSet<_> = subtask_deps
                .difference(&task_levels.keys().copied().collect())
                .copied()
                .collect();

            if unmet_deps.is_empty() {
                ready.push(subtask_id);
            }
        }

        if ready.is_empty() {
            // Circular dependency detected - add remaining to final level with warning
            tracing::warn!(
                remaining = ?remaining,
                "Circular or unresolvable dependencies detected in subtasks"
            );
            ready = remaining.iter().copied().collect();
        }

        // Sort for consistent ordering
        ready.sort_unstable();

        // Current level index (subtask execution levels won't exceed u32::MAX in practice)
        #[allow(clippy::cast_possible_truncation)] // Task levels are small positive integers
        let level_idx = levels.len() as u32;

        // Mark these subtasks as resolved at this level
        for &subtask_id in &ready {
            task_levels.insert(subtask_id, level_idx);
            remaining.remove(&subtask_id);
        }

        levels.push(ready);
    }

    // Update subtasks with their execution levels and parallelizable flag
    for subtask in subtasks.iter_mut() {
        if let Some(&level) = task_levels.get(&subtask.id) {
            subtask.execution_level = Some(level);
            // Subtasks are parallelizable if they're in a level with multiple items
            // or if they have no dependencies (level 0)
            let level_size = levels.get(level as usize).map_or(0, std::vec::Vec::len);
            subtask.parallelizable = level_size > 1 || level == 0;
        }
    }

    // Compute statistics
    let total_subtasks = subtasks.len();
    let total_levels = levels.len();
    let max_parallelism = levels.iter().map(Vec::len).max().unwrap_or(0);
    #[allow(clippy::cast_precision_loss)] // Precision loss acceptable for statistics
    let avg_parallelism = if total_levels > 0 {
        total_subtasks as f64 / total_levels as f64
    } else {
        0.0
    };

    ExecutionLevels {
        levels,
        stats: ExecutionStats {
            total_subtasks,
            total_levels,
            max_parallelism,
            avg_parallelism,
        },
    }
}

/// Parse a subtask dependency string to get the subtask ID.
///
/// Handles both formats:
/// - "1" -> subtask ID 1
/// - "task-1.1" -> subtask ID 1 (extracts after the dot)
fn parse_subtask_dep(dep_str: &str, valid_ids: &HashSet<u32>) -> Option<u32> {
    // Try direct parse first
    if let Ok(id) = dep_str.parse::<u32>() {
        if valid_ids.contains(&id) {
            return Some(id);
        }
    }

    // Try parsing as "parent.subtask" format
    if let Some(dot_idx) = dep_str.rfind('.') {
        if let Ok(id) = dep_str[dot_idx + 1..].parse::<u32>() {
            if valid_ids.contains(&id) {
                return Some(id);
            }
        }
    }

    None
}

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
    fn would_create_cycle(tasks: &[Task], task_id: &str, depends_on: &str) -> bool {
        // Build dependency graph
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for task in tasks {
            graph.insert(
                task.id.as_str(),
                task.dependencies.iter().map(String::as_str).collect(),
            );
        }

        // Temporarily add the new dependency
        graph.entry(task_id).or_default().push(depends_on);

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
        let task =
            tasks
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

    // ===== Subtask Execution Level Tests =====

    #[test]
    fn test_compute_subtask_execution_levels_empty() {
        let mut subtasks: Vec<Subtask> = vec![];
        let result = compute_subtask_execution_levels(&mut subtasks);
        assert!(result.levels.is_empty());
        assert_eq!(result.stats.total_subtasks, 0);
        assert_eq!(result.stats.total_levels, 0);
    }

    #[test]
    fn test_compute_subtask_execution_levels_no_deps() {
        let mut subtasks = vec![
            Subtask::new(1, "task-1", "Subtask 1", "First subtask"),
            Subtask::new(2, "task-1", "Subtask 2", "Second subtask"),
            Subtask::new(3, "task-1", "Subtask 3", "Third subtask"),
        ];

        let result = compute_subtask_execution_levels(&mut subtasks);

        // All subtasks should be at level 0 (no dependencies)
        assert_eq!(result.levels.len(), 1);
        assert_eq!(result.levels[0].len(), 3);
        assert_eq!(result.stats.max_parallelism, 3);

        // All subtasks should be parallelizable
        for subtask in &subtasks {
            assert_eq!(subtask.execution_level, Some(0));
            assert!(subtask.parallelizable);
        }
    }

    #[test]
    fn test_compute_subtask_execution_levels_linear_deps() {
        let mut subtasks = vec![
            Subtask::new(1, "task-1", "Subtask 1", "First"),
            Subtask::new(2, "task-1", "Subtask 2", "Second"),
            Subtask::new(3, "task-1", "Subtask 3", "Third"),
        ];
        subtasks[1].dependencies = vec!["1".to_string()]; // 2 depends on 1
        subtasks[2].dependencies = vec!["2".to_string()]; // 3 depends on 2

        let result = compute_subtask_execution_levels(&mut subtasks);

        // Should have 3 levels: [1], [2], [3]
        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0], vec![1]);
        assert_eq!(result.levels[1], vec![2]);
        assert_eq!(result.levels[2], vec![3]);

        assert_eq!(subtasks[0].execution_level, Some(0));
        assert_eq!(subtasks[1].execution_level, Some(1));
        assert_eq!(subtasks[2].execution_level, Some(2));
    }

    #[test]
    fn test_compute_subtask_execution_levels_parallel_branches() {
        // Create a diamond pattern:
        //     1
        //    / \
        //   2   3
        //    \ /
        //     4
        let mut subtasks = vec![
            Subtask::new(1, "task-1", "Root", "Root task"),
            Subtask::new(2, "task-1", "Branch A", "Branch A"),
            Subtask::new(3, "task-1", "Branch B", "Branch B"),
            Subtask::new(4, "task-1", "Merge", "Merge point"),
        ];
        subtasks[1].dependencies = vec!["1".to_string()]; // 2 depends on 1
        subtasks[2].dependencies = vec!["1".to_string()]; // 3 depends on 1
        subtasks[3].dependencies = vec!["2".to_string(), "3".to_string()]; // 4 depends on 2,3

        let result = compute_subtask_execution_levels(&mut subtasks);

        // Should have 3 levels: [1], [2, 3], [4]
        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0], vec![1]);
        assert!(result.levels[1].contains(&2));
        assert!(result.levels[1].contains(&3));
        assert_eq!(result.levels[2], vec![4]);

        // Level 1 should have parallelism of 2
        assert_eq!(result.stats.max_parallelism, 2);

        // Subtasks 2 and 3 should be parallelizable
        assert!(subtasks[1].parallelizable);
        assert!(subtasks[2].parallelizable);
    }

    #[test]
    fn test_compute_subtask_execution_levels_dotted_format() {
        let mut subtasks = vec![
            Subtask::new(1, "task-1", "Subtask 1", "First"),
            Subtask::new(2, "task-1", "Subtask 2", "Second"),
        ];
        // Use dotted format for dependency
        subtasks[1].dependencies = vec!["task-1.1".to_string()];

        let result = compute_subtask_execution_levels(&mut subtasks);

        // Should correctly parse dotted format
        assert_eq!(result.levels.len(), 2);
        assert_eq!(subtasks[0].execution_level, Some(0));
        assert_eq!(subtasks[1].execution_level, Some(1));
    }

    #[test]
    fn test_parse_subtask_dep() {
        let valid_ids: HashSet<u32> = [1, 2, 3].into_iter().collect();

        // Direct numeric format
        assert_eq!(parse_subtask_dep("1", &valid_ids), Some(1));
        assert_eq!(parse_subtask_dep("2", &valid_ids), Some(2));

        // Dotted format
        assert_eq!(parse_subtask_dep("task-1.1", &valid_ids), Some(1));
        assert_eq!(parse_subtask_dep("task-1.2", &valid_ids), Some(2));

        // Invalid IDs
        assert_eq!(parse_subtask_dep("99", &valid_ids), None);
        assert_eq!(parse_subtask_dep("task-1.99", &valid_ids), None);
        assert_eq!(parse_subtask_dep("invalid", &valid_ids), None);
    }
}
