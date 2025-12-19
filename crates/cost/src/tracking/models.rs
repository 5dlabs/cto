//! Tracking models for task and agent cost analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identifies an agent making API calls.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new agent ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Identifies a task being worked on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    /// Create a new task ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Identifies a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl ProjectId {
    /// Create a new project ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Identifies a session/run (multiple iterations within a task).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    /// Create a new session ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new random session ID.
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Context for tracking an API call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackingContext {
    /// The project this call belongs to.
    pub project_id: Option<ProjectId>,
    /// The task this call is working on.
    pub task_id: Option<TaskId>,
    /// The agent making this call.
    pub agent_id: Option<AgentId>,
    /// The session/run ID (for grouping iterations).
    pub session_id: Option<SessionId>,
    /// Iteration number within the session.
    pub iteration: Option<u32>,
    /// Additional custom tags.
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl TrackingContext {
    /// Create a new empty tracking context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the project ID.
    #[must_use]
    pub fn with_project(mut self, project_id: impl Into<ProjectId>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Set the task ID.
    #[must_use]
    pub fn with_task(mut self, task_id: impl Into<TaskId>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Set the agent ID.
    #[must_use]
    pub fn with_agent(mut self, agent_id: impl Into<AgentId>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Set the session ID.
    #[must_use]
    pub fn with_session(mut self, session_id: impl Into<SessionId>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set the iteration number.
    #[must_use]
    pub fn with_iteration(mut self, iteration: u32) -> Self {
        self.iteration = Some(iteration);
        self
    }

    /// Add a custom tag.
    #[must_use]
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}

/// A recorded API call with tracking context and usage data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedCall {
    /// Unique ID for this call.
    pub id: String,
    /// When the call was made.
    pub timestamp: DateTime<Utc>,
    /// Tracking context (task, agent, etc.).
    pub context: TrackingContext,
    /// The provider used (e.g., "openai", "anthropic").
    pub provider: String,
    /// The model used.
    pub model: String,
    /// Input tokens used.
    pub input_tokens: i64,
    /// Output tokens used.
    pub output_tokens: i64,
    /// Cached input tokens.
    pub cached_tokens: i64,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
    /// Duration of the call in milliseconds.
    pub duration_ms: Option<u64>,
    /// Whether the call succeeded.
    pub success: bool,
    /// Error message if the call failed.
    pub error: Option<String>,
}

impl TrackedCall {
    /// Calculate total tokens used.
    #[must_use]
    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }

    /// Calculate billable input tokens (excluding cached).
    #[must_use]
    pub fn billable_input_tokens(&self) -> i64 {
        self.input_tokens - self.cached_tokens
    }
}

/// Filter criteria for querying tracked calls.
#[derive(Debug, Clone, Default)]
pub struct TrackingFilter {
    /// Filter by project ID.
    pub project_id: Option<ProjectId>,
    /// Filter by task ID.
    pub task_id: Option<TaskId>,
    /// Filter by agent ID.
    pub agent_id: Option<AgentId>,
    /// Filter by session ID.
    pub session_id: Option<SessionId>,
    /// Filter by provider.
    pub provider: Option<String>,
    /// Filter by model.
    pub model: Option<String>,
    /// Filter by start time (inclusive).
    pub start_time: Option<DateTime<Utc>>,
    /// Filter by end time (exclusive).
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by success status.
    pub success: Option<bool>,
    /// Filter by custom tag.
    pub tag: Option<(String, String)>,
}

impl TrackingFilter {
    /// Create a new empty filter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by project ID.
    #[must_use]
    pub fn with_project(mut self, project_id: impl Into<ProjectId>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Filter by task ID.
    #[must_use]
    pub fn with_task(mut self, task_id: impl Into<TaskId>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Filter by agent ID.
    #[must_use]
    pub fn with_agent(mut self, agent_id: impl Into<AgentId>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Filter by session ID.
    #[must_use]
    pub fn with_session(mut self, session_id: impl Into<SessionId>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Filter by provider.
    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Filter by model.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Filter by time range.
    #[must_use]
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by success status.
    #[must_use]
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    /// Check if a tracked call matches this filter.
    #[must_use]
    pub fn matches(&self, call: &TrackedCall) -> bool {
        if let Some(ref project_id) = self.project_id {
            if call.context.project_id.as_ref() != Some(project_id) {
                return false;
            }
        }

        if let Some(ref task_id) = self.task_id {
            if call.context.task_id.as_ref() != Some(task_id) {
                return false;
            }
        }

        if let Some(ref agent_id) = self.agent_id {
            if call.context.agent_id.as_ref() != Some(agent_id) {
                return false;
            }
        }

        if let Some(ref session_id) = self.session_id {
            if call.context.session_id.as_ref() != Some(session_id) {
                return false;
            }
        }

        if let Some(ref provider) = self.provider {
            if &call.provider != provider {
                return false;
            }
        }

        if let Some(ref model) = self.model {
            if &call.model != model {
                return false;
            }
        }

        if let Some(start_time) = self.start_time {
            if call.timestamp < start_time {
                return false;
            }
        }

        if let Some(end_time) = self.end_time {
            if call.timestamp >= end_time {
                return false;
            }
        }

        if let Some(success) = self.success {
            if call.success != success {
                return false;
            }
        }

        if let Some((ref key, ref value)) = self.tag {
            if call.context.tags.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}
















