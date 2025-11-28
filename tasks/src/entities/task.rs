//! Task entity and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Subtask;
use crate::errors::TasksError;

/// Task status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Done,
    Deferred,
    Cancelled,
    Blocked,
    Review,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in-progress"),
            Self::Done => write!(f, "done"),
            Self::Deferred => write!(f, "deferred"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Blocked => write!(f, "blocked"),
            Self::Review => write!(f, "review"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = TasksError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "in-progress" | "inprogress" | "in_progress" => Ok(Self::InProgress),
            "done" | "completed" => Ok(Self::Done),
            "deferred" => Ok(Self::Deferred),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            "blocked" => Ok(Self::Blocked),
            "review" => Ok(Self::Review),
            _ => Err(TasksError::InvalidStatus {
                status: s.to_string(),
            }),
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = TasksError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" | "med" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" | "crit" => Ok(Self::Critical),
            _ => Err(TasksError::InvalidPriority {
                priority: s.to_string(),
            }),
        }
    }
}

/// Task complexity levels (from complexity analysis)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl std::fmt::Display for TaskComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "simple"),
            Self::Moderate => write!(f, "moderate"),
            Self::Complex => write!(f, "complex"),
            Self::VeryComplex => write!(f, "very-complex"),
        }
    }
}

/// Complexity information from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityInfo {
    /// Complexity score (1-10)
    pub score: u8,

    /// Recommended number of subtasks
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "recommendedSubtasks"
    )]
    pub recommended_subtasks: Option<u8>,

    /// AI-generated expansion prompt
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "expansionPrompt"
    )]
    pub expansion_prompt: Option<String>,

    /// Reasoning for complexity score
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

/// Core task structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier (string to support alphanumeric IDs like "TAS-123")
    pub id: String,

    /// Brief, descriptive title
    pub title: String,

    /// Concise description of what the task involves
    pub description: String,

    /// Current task status
    #[serde(default)]
    pub status: TaskStatus,

    /// Task priority level
    #[serde(default)]
    pub priority: TaskPriority,

    /// IDs of prerequisite tasks
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// In-depth implementation instructions
    #[serde(default)]
    pub details: String,

    /// Verification approach
    #[serde(default, rename = "testStrategy")]
    pub test_strategy: String,

    /// List of subtasks
    #[serde(default)]
    pub subtasks: Vec<Subtask>,

    // Optional enhanced properties
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

    /// Estimated effort (hours or story points)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<u32>,

    /// Actual effort spent
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "actualEffort"
    )]
    pub actual_effort: Option<u32>,

    /// Task tags (not to be confused with task list tags)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Assigned user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,

    // Complexity analysis fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity: Option<ComplexityInfo>,
}

impl Task {
    /// Create a new task with minimal required fields
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            dependencies: Vec::new(),
            details: String::new(),
            test_strategy: String::new(),
            subtasks: Vec::new(),
            created_at: Some(now),
            updated_at: Some(now),
            effort: None,
            actual_effort: None,
            tags: Vec::new(),
            assignee: None,
            complexity: None,
        }
    }

    /// Check if task can be marked as complete
    pub fn can_complete(&self) -> bool {
        // Cannot complete if already done or cancelled
        if matches!(self.status, TaskStatus::Done | TaskStatus::Cancelled) {
            return false;
        }

        // Cannot complete if blocked
        if self.status == TaskStatus::Blocked {
            return false;
        }

        // All subtasks must be complete
        self.subtasks
            .iter()
            .all(|s| matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled))
    }

    /// Check if task has unmet dependencies
    pub fn has_blocking_deps(&self, done_tasks: &[&str]) -> bool {
        self.dependencies
            .iter()
            .any(|dep| !done_tasks.contains(&dep.as_str()))
    }

    /// Mark task as complete (returns error if cannot complete)
    pub fn mark_complete(&mut self) -> Result<(), TasksError> {
        if !self.can_complete() {
            let reason = if self
                .subtasks
                .iter()
                .all(|s| matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled))
            {
                format!("current status is {}", self.status)
            } else {
                "incomplete subtasks".to_string()
            };

            return Err(TasksError::CannotComplete {
                task_id: self.id.clone(),
                reason,
            });
        }

        self.status = TaskStatus::Done;
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Update task status with validation
    pub fn set_status(&mut self, new_status: TaskStatus) -> Result<(), TasksError> {
        // Business rule: Cannot move from done to pending
        if self.status == TaskStatus::Done && new_status == TaskStatus::Pending {
            return Err(TasksError::InvalidTransition {
                task_id: self.id.clone(),
                from: self.status.to_string(),
                to: new_status.to_string(),
            });
        }

        self.status = new_status;
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// Get subtask by ID
    pub fn get_subtask(&self, subtask_id: u32) -> Option<&Subtask> {
        self.subtasks.iter().find(|s| s.id == subtask_id)
    }

    /// Get mutable subtask by ID
    pub fn get_subtask_mut(&mut self, subtask_id: u32) -> Option<&mut Subtask> {
        self.subtasks.iter_mut().find(|s| s.id == subtask_id)
    }

    /// Add a subtask
    pub fn add_subtask(&mut self, subtask: Subtask) {
        self.subtasks.push(subtask);
        self.updated_at = Some(Utc::now());
    }

    /// Remove a subtask by ID
    pub fn remove_subtask(&mut self, subtask_id: u32) -> Option<Subtask> {
        if let Some(idx) = self.subtasks.iter().position(|s| s.id == subtask_id) {
            self.updated_at = Some(Utc::now());
            Some(self.subtasks.remove(idx))
        } else {
            None
        }
    }

    /// Clear all subtasks
    pub fn clear_subtasks(&mut self) {
        self.subtasks.clear();
        self.updated_at = Some(Utc::now());
    }

    /// Get next available subtask ID
    pub fn next_subtask_id(&self) -> u32 {
        self.subtasks.iter().map(|s| s.id).max().unwrap_or(0) + 1
    }

    /// Check if all subtasks are complete
    pub fn all_subtasks_done(&self) -> bool {
        self.subtasks.is_empty()
            || self
                .subtasks
                .iter()
                .all(|s| matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled))
    }

    /// Count subtasks by status
    pub fn subtask_counts(&self) -> std::collections::HashMap<TaskStatus, usize> {
        let mut counts = std::collections::HashMap::new();
        for subtask in &self.subtasks {
            *counts.entry(subtask.status).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new() {
        let task = Task::new("1", "Test Task", "A test task description");
        assert_eq!(task.id, "1");
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert!(task.created_at.is_some());
    }

    #[test]
    fn test_task_status_parsing() {
        assert_eq!(
            "pending".parse::<TaskStatus>().unwrap(),
            TaskStatus::Pending
        );
        assert_eq!(
            "in-progress".parse::<TaskStatus>().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert!("invalid".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn test_task_can_complete() {
        let mut task = Task::new("1", "Test", "Test");
        assert!(task.can_complete());

        task.status = TaskStatus::Done;
        assert!(!task.can_complete());
    }

    #[test]
    fn test_task_with_subtasks_completion() {
        let mut task = Task::new("1", "Test", "Test");
        task.add_subtask(Subtask::new(1, "1", "Sub 1", "Description"));
        assert!(!task.can_complete());

        task.subtasks[0].status = TaskStatus::Done;
        assert!(task.can_complete());
    }

    #[test]
    fn test_status_transition_validation() {
        let mut task = Task::new("1", "Test", "Test");
        task.status = TaskStatus::Done;

        let result = task.set_status(TaskStatus::Pending);
        assert!(result.is_err());
    }
}

