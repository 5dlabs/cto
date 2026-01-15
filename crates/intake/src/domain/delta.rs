//! Task diff computation for the update command.
//!
//! This module provides functionality to compute the difference between
//! two sets of tasks, identifying added, removed, modified, and unchanged tasks.

use crate::Task;

/// Represents the delta between two task sets.
#[derive(Debug, Clone, Default)]
pub struct TaskDelta {
    /// Tasks that exist in new set but not in old set.
    pub added: Vec<Task>,

    /// Tasks that exist in old set but not in new set.
    pub removed: Vec<Task>,

    /// Tasks that exist in both but have changes (old, new).
    pub modified: Vec<(Task, Task)>,

    /// Tasks that are identical in both sets.
    pub unchanged: Vec<Task>,
}

impl TaskDelta {
    /// Create a new empty delta.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the delta represents no changes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }

    /// Get the total number of changes.
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }

    /// Get a summary string of the delta.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_empty() {
            return "No changes detected".to_string();
        }

        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("{} added", self.added.len()));
        }
        if !self.removed.is_empty() {
            parts.push(format!("{} removed", self.removed.len()));
        }
        if !self.modified.is_empty() {
            parts.push(format!("{} modified", self.modified.len()));
        }
        if !self.unchanged.is_empty() {
            parts.push(format!("{} unchanged", self.unchanged.len()));
        }
        parts.join(", ")
    }
}

/// Represents what changed in a modified task.
#[derive(Debug, Clone, Default)]
pub struct TaskChanges {
    /// Whether the title changed.
    pub title_changed: bool,

    /// Whether the description changed.
    pub description_changed: bool,

    /// Whether the details changed.
    pub details_changed: bool,

    /// Whether the test strategy changed.
    pub test_strategy_changed: bool,

    /// Whether dependencies changed.
    pub dependencies_changed: bool,

    /// Whether the agent hint changed.
    pub agent_hint_changed: bool,

    /// Whether subtasks changed.
    pub subtasks_changed: bool,

    /// Whether priority changed.
    pub priority_changed: bool,
}

impl TaskChanges {
    /// Check if any significant content changed (not just metadata).
    #[must_use]
    pub fn has_content_changes(&self) -> bool {
        self.title_changed
            || self.description_changed
            || self.details_changed
            || self.test_strategy_changed
            || self.subtasks_changed
    }

    /// Check if any structural changes occurred.
    #[must_use]
    pub fn has_structural_changes(&self) -> bool {
        self.dependencies_changed || self.agent_hint_changed
    }
}

/// Compute the delta between old and new task sets.
///
/// Tasks are matched by ID. The algorithm:
/// 1. Build a map of old tasks by ID
/// 2. For each new task, check if it exists in old tasks
/// 3. If exists, compare for modifications
/// 4. If not exists, mark as added
/// 5. Any old tasks not in new set are marked as removed
#[must_use]
pub fn compute_task_delta(old_tasks: &[Task], new_tasks: &[Task]) -> TaskDelta {
    use std::collections::HashMap;

    let mut delta = TaskDelta::new();

    // Build map of old tasks by ID
    let old_map: HashMap<&str, &Task> = old_tasks.iter().map(|t| (t.id.as_str(), t)).collect();

    // Track which old tasks we've seen
    let mut seen_old_ids: std::collections::HashSet<&str> = std::collections::HashSet::new();

    // Process new tasks
    for new_task in new_tasks {
        if let Some(old_task) = old_map.get(new_task.id.as_str()) {
            seen_old_ids.insert(new_task.id.as_str());

            if tasks_are_equal(old_task, new_task) {
                delta.unchanged.push(new_task.clone());
            } else {
                delta.modified.push(((*old_task).clone(), new_task.clone()));
            }
        } else {
            delta.added.push(new_task.clone());
        }
    }

    // Find removed tasks (in old but not in new)
    for old_task in old_tasks {
        if !seen_old_ids.contains(old_task.id.as_str()) {
            delta.removed.push(old_task.clone());
        }
    }

    delta
}

/// Compare two tasks for equality (ignoring status and timestamps).
///
/// We compare content fields that would affect the generated documentation:
/// - title
/// - description
/// - details
/// - test_strategy
/// - dependencies
/// - agent_hint
/// - subtasks (by count and content, not status)
#[must_use]
pub fn tasks_are_equal(a: &Task, b: &Task) -> bool {
    // Compare core content fields
    if a.title != b.title
        || a.description != b.description
        || a.details != b.details
        || a.test_strategy != b.test_strategy
    {
        return false;
    }

    // Compare dependencies (order-independent)
    let mut a_deps = a.dependencies.clone();
    let mut b_deps = b.dependencies.clone();
    a_deps.sort();
    b_deps.sort();
    if a_deps != b_deps {
        return false;
    }

    // Compare agent hint
    if a.agent_hint != b.agent_hint {
        return false;
    }

    // Compare subtasks (by content, not status)
    if a.subtasks.len() != b.subtasks.len() {
        return false;
    }

    for (a_sub, b_sub) in a.subtasks.iter().zip(b.subtasks.iter()) {
        if a_sub.title != b_sub.title || a_sub.description != b_sub.description {
            return false;
        }
    }

    true
}

/// Get detailed changes between two tasks.
#[must_use]
pub fn get_task_changes(old: &Task, new: &Task) -> TaskChanges {
    let mut a_deps = old.dependencies.clone();
    let mut b_deps = new.dependencies.clone();
    a_deps.sort();
    b_deps.sort();

    TaskChanges {
        title_changed: old.title != new.title,
        description_changed: old.description != new.description,
        details_changed: old.details != new.details,
        test_strategy_changed: old.test_strategy != new.test_strategy,
        dependencies_changed: a_deps != b_deps,
        agent_hint_changed: old.agent_hint != new.agent_hint,
        subtasks_changed: !subtasks_are_equal(&old.subtasks, &new.subtasks),
        priority_changed: old.priority != new.priority,
    }
}

/// Compare two subtask lists for equality (by content, not status).
fn subtasks_are_equal(a: &[crate::entities::Subtask], b: &[crate::entities::Subtask]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for (a_sub, b_sub) in a.iter().zip(b.iter()) {
        if a_sub.title != b_sub.title || a_sub.description != b_sub.description {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, title: &str) -> Task {
        Task::new(id, title, format!("Description for {}", title))
    }

    #[test]
    fn test_empty_delta() {
        let delta = TaskDelta::new();
        assert!(delta.is_empty());
        assert_eq!(delta.change_count(), 0);
        assert_eq!(delta.summary(), "No changes detected");
    }

    #[test]
    fn test_compute_delta_no_changes() {
        let tasks = vec![make_task("1", "Task 1"), make_task("2", "Task 2")];

        let delta = compute_task_delta(&tasks, &tasks);
        assert!(delta.is_empty());
        assert_eq!(delta.unchanged.len(), 2);
    }

    #[test]
    fn test_compute_delta_added_task() {
        let old_tasks = vec![make_task("1", "Task 1")];
        let new_tasks = vec![make_task("1", "Task 1"), make_task("2", "Task 2")];

        let delta = compute_task_delta(&old_tasks, &new_tasks);
        assert_eq!(delta.added.len(), 1);
        assert_eq!(delta.added[0].id, "2");
        assert_eq!(delta.removed.len(), 0);
        assert_eq!(delta.modified.len(), 0);
        assert_eq!(delta.unchanged.len(), 1);
    }

    #[test]
    fn test_compute_delta_removed_task() {
        let old_tasks = vec![make_task("1", "Task 1"), make_task("2", "Task 2")];
        let new_tasks = vec![make_task("1", "Task 1")];

        let delta = compute_task_delta(&old_tasks, &new_tasks);
        assert_eq!(delta.added.len(), 0);
        assert_eq!(delta.removed.len(), 1);
        assert_eq!(delta.removed[0].id, "2");
        assert_eq!(delta.modified.len(), 0);
        assert_eq!(delta.unchanged.len(), 1);
    }

    #[test]
    fn test_compute_delta_modified_task() {
        let old_tasks = vec![make_task("1", "Task 1")];
        let mut new_task = make_task("1", "Task 1 Updated");
        new_task.description = "Updated description".to_string();
        let new_tasks = vec![new_task];

        let delta = compute_task_delta(&old_tasks, &new_tasks);
        assert_eq!(delta.added.len(), 0);
        assert_eq!(delta.removed.len(), 0);
        assert_eq!(delta.modified.len(), 1);
        assert_eq!(delta.modified[0].0.title, "Task 1");
        assert_eq!(delta.modified[0].1.title, "Task 1 Updated");
        assert_eq!(delta.unchanged.len(), 0);
    }

    #[test]
    fn test_delta_summary() {
        let mut delta = TaskDelta::new();
        delta.added.push(make_task("1", "New"));
        delta.removed.push(make_task("2", "Old"));
        delta
            .modified
            .push((make_task("3", "Before"), make_task("3", "After")));

        let summary = delta.summary();
        assert!(summary.contains("1 added"));
        assert!(summary.contains("1 removed"));
        assert!(summary.contains("1 modified"));
    }

    #[test]
    fn test_tasks_are_equal_ignores_status() {
        let mut task1 = make_task("1", "Task");
        let mut task2 = make_task("1", "Task");

        task1.status = crate::TaskStatus::Pending;
        task2.status = crate::TaskStatus::Done;

        assert!(tasks_are_equal(&task1, &task2));
    }

    #[test]
    fn test_tasks_are_equal_compares_dependencies() {
        let mut task1 = make_task("1", "Task");
        let mut task2 = make_task("1", "Task");

        task1.dependencies = vec!["2".to_string(), "3".to_string()];
        task2.dependencies = vec!["3".to_string(), "2".to_string()]; // Different order

        assert!(tasks_are_equal(&task1, &task2));

        task2.dependencies = vec!["2".to_string()]; // Different content
        assert!(!tasks_are_equal(&task1, &task2));
    }

    #[test]
    fn test_get_task_changes() {
        let mut old = make_task("1", "Old Title");
        old.details = "Old details".to_string();

        let mut new = make_task("1", "New Title");
        new.details = "Old details".to_string();

        let changes = get_task_changes(&old, &new);
        assert!(changes.title_changed);
        assert!(changes.description_changed); // Different because of title in description
        assert!(!changes.details_changed);
    }
}
