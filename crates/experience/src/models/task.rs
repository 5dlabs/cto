//! Task record model - tracking individual task execution.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is pending.
    #[default]
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Success,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "running" | "in_progress" | "in-progress" => Ok(Self::Running),
            "success" | "done" | "completed" => Ok(Self::Success),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown task status: {s}")),
        }
    }
}

/// Record of a tool call made during task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Unique ID of this tool call.
    pub id: String,

    /// Name of the tool.
    pub tool_name: String,

    /// Arguments passed to the tool (as JSON string).
    pub arguments: String,

    /// Result of the tool call (may be truncated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,

    /// Whether the tool call succeeded.
    pub success: bool,

    /// Duration of the tool call in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// When the tool was called.
    pub timestamp: DateTime<Utc>,
}

impl ToolCallRecord {
    /// Create a new tool call record.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        tool_name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            tool_name: tool_name.into(),
            arguments: arguments.into(),
            result: None,
            success: true,
            duration_ms: None,
            timestamp: Utc::now(),
        }
    }

    /// Set the result.
    #[must_use]
    pub fn with_result(mut self, result: impl Into<String>, success: bool) -> Self {
        self.result = Some(result.into());
        self.success = success;
        self
    }

    /// Set the duration.
    #[must_use]
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Record of a task within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Internal UUID for database storage.
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,

    /// Task ID from the workflow (e.g., "1", "INT-001").
    pub id: String,

    /// Task description.
    pub description: String,

    /// Current status.
    pub status: TaskStatus,

    /// Progress updates from the agent.
    #[serde(default)]
    pub progresses: Vec<String>,

    /// User preferences captured during execution.
    #[serde(default)]
    pub user_preferences: Vec<String>,

    /// Tool calls made during this task.
    #[serde(default)]
    pub tool_calls: Vec<ToolCallRecord>,

    /// Agent that worked on this task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// When the task started.
    pub started_at: DateTime<Utc>,

    /// When the task completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl TaskRecord {
    /// Create a new task record.
    #[must_use]
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            id: id.into(),
            description: description.into(),
            status: TaskStatus::Pending,
            progresses: Vec::new(),
            user_preferences: Vec::new(),
            tool_calls: Vec::new(),
            agent: None,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Set the agent.
    #[must_use]
    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// Add a progress update.
    pub fn add_progress(&mut self, progress: impl Into<String>) {
        self.progresses.push(progress.into());
    }

    /// Add a user preference.
    pub fn add_preference(&mut self, preference: impl Into<String>) {
        self.user_preferences.push(preference.into());
    }

    /// Add a tool call.
    pub fn add_tool_call(&mut self, tool_call: ToolCallRecord) {
        self.tool_calls.push(tool_call);
    }

    /// Start the task.
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Utc::now();
    }

    /// Complete the task.
    pub fn complete(&mut self, success: bool) {
        self.status = if success {
            TaskStatus::Success
        } else {
            TaskStatus::Failed
        };
        self.completed_at = Some(Utc::now());
    }

    /// Get task duration.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }

    /// Check if this task is learnable (has enough tool calls, succeeded, etc.).
    #[must_use]
    pub fn is_learnable(&self, min_tool_calls: usize, min_duration_secs: u64) -> bool {
        if self.status != TaskStatus::Success {
            return false;
        }

        if self.tool_calls.len() < min_tool_calls {
            return false;
        }

        if let Some(duration) = self.duration() {
            let duration_secs = duration.num_seconds();
            let min_duration_secs_i64 = i64::try_from(min_duration_secs).unwrap_or(i64::MAX);
            if duration_secs < min_duration_secs_i64 {
                return false;
            }
        } else {
            return false;
        }

        true
    }

    /// Get unique tool names used in this task.
    #[must_use]
    pub fn unique_tools(&self) -> Vec<String> {
        let mut tools: Vec<String> = self
            .tool_calls
            .iter()
            .map(|tc| tc.tool_name.clone())
            .collect();
        tools.sort();
        tools.dedup();
        tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_parsing() {
        assert_eq!(
            "pending".parse::<TaskStatus>().unwrap(),
            TaskStatus::Pending
        );
        assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Success);
        assert_eq!(
            "in_progress".parse::<TaskStatus>().unwrap(),
            TaskStatus::Running
        );
    }

    #[test]
    fn test_task_creation() {
        let task = TaskRecord::new("1", "Implement feature X");
        assert_eq!(task.id, "1");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.tool_calls.is_empty());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = TaskRecord::new("1", "Test task");

        task.start();
        assert_eq!(task.status, TaskStatus::Running);

        task.add_tool_call(ToolCallRecord::new("call-1", "read_file", "{}"));
        task.add_progress("Read the config file");

        task.complete(true);
        assert_eq!(task.status, TaskStatus::Success);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_learnability() {
        let mut task = TaskRecord::new("1", "Complex task");
        task.start();

        // Add enough tool calls
        for i in 0..5 {
            task.add_tool_call(ToolCallRecord::new(
                format!("call-{i}"),
                format!("tool_{i}"),
                "{}",
            ));
        }

        // Not learnable yet - not completed
        assert!(!task.is_learnable(3, 0));

        task.complete(true);

        // Now learnable (0 duration threshold)
        assert!(task.is_learnable(3, 0));

        // Not learnable with high duration threshold
        assert!(!task.is_learnable(3, 3600));
    }

    #[test]
    fn test_unique_tools() {
        let mut task = TaskRecord::new("1", "Task");
        task.add_tool_call(ToolCallRecord::new("1", "read_file", "{}"));
        task.add_tool_call(ToolCallRecord::new("2", "write_file", "{}"));
        task.add_tool_call(ToolCallRecord::new("3", "read_file", "{}"));

        let unique = task.unique_tools();
        assert_eq!(unique.len(), 2);
        assert!(unique.contains(&"read_file".to_string()));
        assert!(unique.contains(&"write_file".to_string()));
    }
}
