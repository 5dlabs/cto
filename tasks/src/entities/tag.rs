//! Tag and tagged task list entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Task;

/// Tagged task collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedTaskList {
    /// Tasks in this tag context
    pub tasks: Vec<Task>,

    /// Tag metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TagMetadata>,
}

impl Default for TaggedTaskList {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            metadata: Some(TagMetadata {
                created: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                description: None,
                version: Some("1.0.0".to_string()),
                project_name: None,
            }),
        }
    }
}

impl TaggedTaskList {
    /// Create a new empty tagged task list
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial tasks
    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            metadata: Some(TagMetadata {
                created: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                description: None,
                version: Some("1.0.0".to_string()),
                project_name: None,
            }),
        }
    }

    /// Get task count
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Get completed task count
    pub fn completed_count(&self) -> usize {
        use super::TaskStatus;
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count()
    }
}

/// Tag metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMetadata {
    /// Creation timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "updatedAt"
    )]
    pub updated_at: Option<DateTime<Utc>>,

    /// Tag description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Version info
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Project name
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "projectName"
    )]
    pub project_name: Option<String>,
}

impl Default for TagMetadata {
    fn default() -> Self {
        Self {
            created: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            description: None,
            version: Some("1.0.0".to_string()),
            project_name: None,
        }
    }
}

/// Tag statistics
#[derive(Debug, Clone)]
pub struct TagStats {
    /// Tag name
    pub name: String,

    /// Whether this is the currently active tag
    pub is_current: bool,

    /// Total number of tasks
    pub task_count: usize,

    /// Number of completed tasks
    pub completed_tasks: usize,

    /// Status breakdown
    pub status_breakdown: HashMap<String, usize>,

    /// Subtask counts
    pub subtask_counts: Option<SubtaskCounts>,

    /// Creation date
    pub created: Option<DateTime<Utc>>,

    /// Description
    pub description: Option<String>,
}

impl TagStats {
    /// Create new tag stats
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_current: false,
            task_count: 0,
            completed_tasks: 0,
            status_breakdown: HashMap::new(),
            subtask_counts: None,
            created: None,
            description: None,
        }
    }

    /// Calculate completion percentage
    #[allow(clippy::cast_precision_loss)]
    pub fn completion_percent(&self) -> f64 {
        if self.task_count == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / self.task_count as f64) * 100.0
        }
    }
}

/// Subtask statistics
#[derive(Debug, Clone)]
pub struct SubtaskCounts {
    pub total: usize,
    pub by_status: HashMap<String, usize>,
}

impl SubtaskCounts {
    /// Create new subtask counts
    pub fn new() -> Self {
        Self {
            total: 0,
            by_status: HashMap::new(),
        }
    }
}

impl Default for SubtaskCounts {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged_task_list_default() {
        let list = TaggedTaskList::default();
        assert!(list.tasks.is_empty());
        assert!(list.metadata.is_some());
    }

    #[test]
    fn test_tag_stats_completion_percent() {
        let mut stats = TagStats::new("test");
        stats.task_count = 10;
        stats.completed_tasks = 3;
        assert!((stats.completion_percent() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_tag_stats_zero_tasks() {
        let stats = TagStats::new("empty");
        assert!((stats.completion_percent() - 0.0).abs() < 0.001);
    }
}

