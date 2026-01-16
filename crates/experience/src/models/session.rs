//! Session record model - tracking workflow context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::task::TaskRecord;

/// Status of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is active.
    #[default]
    Active,
    /// Session completed successfully.
    Completed,
    /// Session failed.
    Failed,
    /// Session was cancelled.
    Cancelled,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown session status: {s}")),
        }
    }
}

/// A message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    /// Role: user, assistant, tool
    pub role: String,

    /// Message content (may be truncated for storage).
    pub content: String,

    /// Tool call ID if this is a tool message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    /// Tool name if this is a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Token count for this message.
    pub token_count: u32,

    /// When this message was recorded.
    pub timestamp: DateTime<Utc>,
}

impl MessageRecord {
    /// Create a user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_call_id: None,
            tool_name: None,
            token_count: 0,
            timestamp: Utc::now(),
        }
    }

    /// Create an assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_call_id: None,
            tool_name: None,
            token_count: 0,
            timestamp: Utc::now(),
        }
    }

    /// Create a tool result message.
    #[must_use]
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            tool_name: None,
            token_count: 0,
            timestamp: Utc::now(),
        }
    }

    /// Set the token count.
    #[must_use]
    pub fn with_tokens(mut self, count: u32) -> Self {
        self.token_count = count;
        self
    }

    /// Check if this is a tool-related message.
    #[must_use]
    pub fn is_tool_message(&self) -> bool {
        self.role == "tool" || self.tool_call_id.is_some() || self.tool_name.is_some()
    }
}

/// A session record for tracking workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Unique identifier.
    pub id: Uuid,

    /// Play ID (from Healer/workflow).
    pub play_id: String,

    /// Space this session belongs to.
    pub space_id: Uuid,

    /// Current status.
    pub status: SessionStatus,

    /// Tasks in this session.
    #[serde(default)]
    pub tasks: Vec<TaskRecord>,

    /// Message history (may be truncated).
    #[serde(default)]
    pub messages: Vec<MessageRecord>,

    /// Repository URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Service name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,

    /// When the session started.
    pub started_at: DateTime<Utc>,

    /// When the session completed (if completed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl SessionRecord {
    /// Create a new session record.
    #[must_use]
    pub fn new(play_id: impl Into<String>, space_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            play_id: play_id.into(),
            space_id,
            status: SessionStatus::Active,
            tasks: Vec::new(),
            messages: Vec::new(),
            repository: None,
            service: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Set repository.
    #[must_use]
    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repository = Some(repo.into());
        self
    }

    /// Set service.
    #[must_use]
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    /// Add a task to the session.
    pub fn add_task(&mut self, task: TaskRecord) {
        self.tasks.push(task);
    }

    /// Add a message to the history.
    pub fn add_message(&mut self, message: MessageRecord) {
        self.messages.push(message);
    }

    /// Mark as completed.
    pub fn complete(&mut self, success: bool) {
        self.status = if success {
            SessionStatus::Completed
        } else {
            SessionStatus::Failed
        };
        self.completed_at = Some(Utc::now());
    }

    /// Get successful tasks.
    #[must_use]
    pub fn successful_tasks(&self) -> Vec<&TaskRecord> {
        use super::task::TaskStatus;
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Success)
            .collect()
    }

    /// Get total duration if completed.
    #[must_use]
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }

    /// Calculate total token count.
    #[must_use]
    pub fn total_tokens(&self) -> u32 {
        self.messages.iter().map(|m| m.token_count).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let space_id = Uuid::new_v4();
        let session = SessionRecord::new("play-123", space_id);

        assert_eq!(session.play_id, "play-123");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.tasks.is_empty());
    }

    #[test]
    fn test_session_completion() {
        let space_id = Uuid::new_v4();
        let mut session = SessionRecord::new("play-123", space_id);

        session.complete(true);
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_message_records() {
        let user_msg = MessageRecord::user("Hello");
        assert_eq!(user_msg.role, "user");
        assert!(!user_msg.is_tool_message());

        let tool_msg = MessageRecord::tool("call-123", "Result");
        assert!(tool_msg.is_tool_message());
    }
}
