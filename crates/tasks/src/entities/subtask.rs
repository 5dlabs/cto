//! Subtask entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::task::{TaskPriority, TaskStatus};

/// Subagent type for routing subtasks to specialized workers.
///
/// Each subagent type corresponds to a specialized Claude Code subagent
/// with focused context, tools, and system prompt for its role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SubagentType {
    /// Write/implement code - the default for most subtasks
    #[default]
    Implementer,
    /// Review code quality, patterns, and best practices
    Reviewer,
    /// Write and run tests
    Tester,
    /// Write documentation
    Documenter,
    /// Research and exploration
    Researcher,
    /// Debug issues and fix bugs
    Debugger,
}

impl std::fmt::Display for SubagentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Implementer => write!(f, "implementer"),
            Self::Reviewer => write!(f, "reviewer"),
            Self::Tester => write!(f, "tester"),
            Self::Documenter => write!(f, "documenter"),
            Self::Researcher => write!(f, "researcher"),
            Self::Debugger => write!(f, "debugger"),
        }
    }
}

impl std::str::FromStr for SubagentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "implementer" | "implement" | "coder" => Ok(Self::Implementer),
            "reviewer" | "review" => Ok(Self::Reviewer),
            "tester" | "test" => Ok(Self::Tester),
            "documenter" | "docs" | "documentation" => Ok(Self::Documenter),
            "researcher" | "research" => Ok(Self::Researcher),
            "debugger" | "debug" => Ok(Self::Debugger),
            _ => Err(format!("Unknown subagent type: {s}")),
        }
    }
}

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

    #[serde(default, skip_serializing_if = "Option::is_none", rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none", rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,

    // ===== Subagent execution fields =====

    /// Role/type hint for subagent routing.
    /// Determines which specialized subagent handles this subtask.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "subagentType")]
    pub subagent_type: Option<SubagentType>,

    /// Execution level for parallel grouping.
    /// Level 0 = no dependencies, higher levels depend on lower levels.
    /// Subtasks at the same level can run in parallel.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "executionLevel")]
    pub execution_level: Option<u32>,

    /// Whether this subtask can run in parallel with others at the same execution level.
    /// Default is true if execution_level is set.
    #[serde(default, rename = "parallelizable")]
    pub parallelizable: bool,
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
            subagent_type: None,
            execution_level: None,
            parallelizable: false,
        }
    }

    /// Create a new subtask with subagent configuration
    pub fn new_with_subagent(
        id: u32,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        subagent_type: SubagentType,
    ) -> Self {
        let mut subtask = Self::new(id, parent_id, title, description);
        subtask.subagent_type = Some(subagent_type);
        subtask.parallelizable = true; // Subagent tasks are parallelizable by default
        subtask
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

    /// Set execution level for parallel grouping
    pub fn set_execution_level(&mut self, level: u32) {
        self.execution_level = Some(level);
        self.updated_at = Some(Utc::now());
    }

    /// Set subagent type for routing
    pub fn set_subagent_type(&mut self, subagent_type: SubagentType) {
        self.subagent_type = Some(subagent_type);
        self.updated_at = Some(Utc::now());
    }

    /// Check if this subtask can run in parallel with others
    pub fn can_parallelize(&self) -> bool {
        self.parallelizable && self.execution_level.is_some()
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
        assert!(subtask.subagent_type.is_none());
        assert!(subtask.execution_level.is_none());
        assert!(!subtask.parallelizable);
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

    #[test]
    fn test_subtask_with_subagent() {
        let subtask =
            Subtask::new_with_subagent(1, "task-1", "Implement handler", "Desc", SubagentType::Implementer);
        assert_eq!(subtask.subagent_type, Some(SubagentType::Implementer));
        assert!(subtask.parallelizable);
    }

    #[test]
    fn test_subtask_execution_level() {
        let mut subtask = Subtask::new(1, "task-1", "Sub", "Desc");
        subtask.set_execution_level(2);
        assert_eq!(subtask.execution_level, Some(2));
    }

    #[test]
    fn test_subtask_can_parallelize() {
        let mut subtask = Subtask::new(1, "task-1", "Sub", "Desc");
        assert!(!subtask.can_parallelize()); // No level set

        subtask.execution_level = Some(0);
        assert!(!subtask.can_parallelize()); // Not marked parallelizable

        subtask.parallelizable = true;
        assert!(subtask.can_parallelize()); // Both conditions met
    }

    #[test]
    fn test_subagent_type_display() {
        assert_eq!(SubagentType::Implementer.to_string(), "implementer");
        assert_eq!(SubagentType::Reviewer.to_string(), "reviewer");
        assert_eq!(SubagentType::Tester.to_string(), "tester");
        assert_eq!(SubagentType::Documenter.to_string(), "documenter");
        assert_eq!(SubagentType::Researcher.to_string(), "researcher");
        assert_eq!(SubagentType::Debugger.to_string(), "debugger");
    }

    #[test]
    fn test_subagent_type_from_str() {
        assert_eq!(
            "implementer".parse::<SubagentType>().unwrap(),
            SubagentType::Implementer
        );
        assert_eq!(
            "coder".parse::<SubagentType>().unwrap(),
            SubagentType::Implementer
        );
        assert_eq!(
            "reviewer".parse::<SubagentType>().unwrap(),
            SubagentType::Reviewer
        );
        assert_eq!(
            "tester".parse::<SubagentType>().unwrap(),
            SubagentType::Tester
        );
        assert!("unknown".parse::<SubagentType>().is_err());
    }

    #[test]
    fn test_subagent_type_default() {
        assert_eq!(SubagentType::default(), SubagentType::Implementer);
    }

    #[test]
    fn test_subtask_serde_with_subagent_fields() {
        let json = r#"{
            "id": 1,
            "parentId": "task-1",
            "title": "Test subtask",
            "description": "Test description",
            "subagentType": "reviewer",
            "executionLevel": 1,
            "parallelizable": true
        }"#;
        let subtask: Subtask = serde_json::from_str(json).unwrap();
        assert_eq!(subtask.subagent_type, Some(SubagentType::Reviewer));
        assert_eq!(subtask.execution_level, Some(1));
        assert!(subtask.parallelizable);
    }
}
