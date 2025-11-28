//! Subtask entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::task::{TaskPriority, TaskStatus};

/// Subtask structure (nested within tasks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// Numeric ID within parent task
    pub id: u32,

    /// Parent task ID
    #[serde(rename = "parentId")]
    pub parent_id: String,

    /// Brief, descriptive title
    pub title: String,

    /// Concise description
    pub description: String,

    /// Current status
    #[serde(default)]
    pub status: TaskStatus,

    /// Priority (inherits from parent if not set)
    #[serde(default)]
    pub priority: TaskPriority,

    /// Dependencies (can reference other subtasks or tasks)
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Implementation details
    #[serde(default)]
    pub details: String,

    /// Test strategy
    #[serde(default, rename = "testStrategy")]
    pub test_strategy: String,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "createdAt"
    )]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "updatedAt"
    )]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
}

impl Subtask {
    /// Create a new subtask
    pub fn new(
        id: u32,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            parent_id: parent_id.into(),
            title: title.into(),
            description: description.into(),
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            dependencies: Vec::new(),
            details: String::new(),
            test_strategy: String::new(),
            created_at: Some(now),
            updated_at: Some(now),
            assignee: None,
        }
    }

    /// Get full ID (parentId.subtaskId format)
    pub fn full_id(&self) -> String {
        format!("{}.{}", self.parent_id, self.id)
    }

    /// Update subtask status
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtask_new() {
        let subtask = Subtask::new(1, "task-1", "Subtask Title", "Subtask description");
        assert_eq!(subtask.id, 1);
        assert_eq!(subtask.parent_id, "task-1");
        assert_eq!(subtask.title, "Subtask Title");
        assert_eq!(subtask.status, TaskStatus::Pending);
    }

    #[test]
    fn test_subtask_full_id() {
        let subtask = Subtask::new(2, "task-1", "Sub", "Desc");
        assert_eq!(subtask.full_id(), "task-1.2");
    }

    #[test]
    fn test_subtask_set_status() {
        let mut subtask = Subtask::new(1, "task-1", "Sub", "Desc");
        subtask.set_status(TaskStatus::InProgress);
        assert_eq!(subtask.status, TaskStatus::InProgress);
        assert!(subtask.updated_at.is_some());
    }
}

