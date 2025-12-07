//! Linear entity type definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Linear Issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// Unique identifier
    pub id: String,
    /// Human-readable identifier (e.g., "TSK-1")
    pub identifier: String,
    /// Issue title
    pub title: String,
    /// Issue description (markdown)
    #[serde(default)]
    pub description: Option<String>,
    /// URL to the issue
    #[serde(default)]
    pub url: Option<String>,
    /// Current workflow state
    #[serde(default)]
    pub state: Option<WorkflowState>,
    /// Team the issue belongs to
    #[serde(default)]
    pub team: Option<Team>,
    /// Parent issue ID (for sub-issues)
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Priority (0 = no priority, 1 = urgent, 2 = high, 3 = normal, 4 = low)
    #[serde(default)]
    pub priority: i32,
    /// Labels on the issue
    #[serde(default)]
    pub labels: Vec<Label>,
    /// Issue delegate (agent assignment)
    #[serde(default)]
    pub delegate: Option<User>,
    /// Issue assignee (human owner)
    #[serde(default)]
    pub assignee: Option<User>,
    /// Created timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Updated timestamp
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Linear workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    /// Unique identifier
    pub id: String,
    /// State name (e.g., "In Progress")
    pub name: String,
    /// State type: backlog, unstarted, started, completed, canceled
    #[serde(rename = "type")]
    pub state_type: String,
    /// Position for ordering
    #[serde(default)]
    pub position: f64,
}

/// Linear team
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Unique identifier
    pub id: String,
    /// Team name
    pub name: String,
    /// Team key (used in issue identifiers)
    pub key: String,
}

/// Linear label
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    /// Unique identifier
    pub id: String,
    /// Label name
    pub name: String,
    /// Label color
    #[serde(default)]
    pub color: Option<String>,
}

/// Linear user (human or app)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Email address
    #[serde(default)]
    pub email: Option<String>,
    /// User type
    #[serde(default)]
    pub user_type: Option<String>,
}

/// Linear document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Unique identifier
    pub id: String,
    /// Document title
    pub title: String,
    /// Document content (markdown)
    #[serde(default)]
    pub content: Option<String>,
    /// URL to the document
    #[serde(default)]
    pub url: Option<String>,
}

/// Linear project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Unique identifier
    pub id: String,
    /// Project name
    pub name: String,
    /// Project description
    #[serde(default)]
    pub description: Option<String>,
    /// URL to the project
    #[serde(default)]
    pub url: Option<String>,
}

/// Linear attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    /// Unique identifier
    pub id: String,
    /// Attachment title
    pub title: String,
    /// Attachment URL
    pub url: String,
}

/// Linear comment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    /// Unique identifier
    pub id: String,
    /// Comment body (markdown)
    pub body: String,
    /// Comment author
    #[serde(default)]
    pub user: Option<User>,
    /// Created timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

/// Agent session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentSessionState {
    /// Session created, awaiting agent response
    Pending,
    /// Agent is actively working
    Active,
    /// Agent encountered an error
    Error,
    /// Agent is waiting for user input
    AwaitingInput,
    /// Agent has completed work
    Complete,
}

/// Linear agent session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    /// Unique identifier
    pub id: String,
    /// Session state
    #[serde(default)]
    pub state: Option<AgentSessionState>,
    /// Associated issue
    #[serde(default)]
    pub issue: Option<Issue>,
    /// Associated comment (if triggered by comment)
    #[serde(default)]
    pub comment: Option<Comment>,
    /// Previous comments for context
    #[serde(default)]
    pub previous_comments: Vec<Comment>,
    /// Guidance/system prompt from workspace settings
    #[serde(default)]
    pub guidance: Option<String>,
    /// External URL for session dashboard
    #[serde(default)]
    pub external_url: Option<String>,
}

/// Issue relation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueRelationType {
    /// This issue blocks the related issue
    Blocks,
    /// This issue is blocked by the related issue
    BlockedBy,
    /// This issue is related to the related issue
    Related,
    /// This issue duplicates the related issue
    Duplicate,
}

/// Input for creating an issue
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateInput {
    /// Team ID
    pub team_id: String,
    /// Issue title
    pub title: String,
    /// Issue description (markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Parent issue ID (for sub-issues)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Priority (1-4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    /// Label IDs to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,
    /// Project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Workflow state ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
}

/// Input for updating an issue
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateInput {
    /// New title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New workflow state ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
    /// New priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    /// New delegate ID (agent assignment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate_id: Option<String>,
}

/// Input for creating an issue relation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationCreateInput {
    /// Source issue ID
    pub issue_id: String,
    /// Related issue ID
    pub related_issue_id: String,
    /// Relation type
    #[serde(rename = "type")]
    pub relation_type: IssueRelationType,
}

/// Input for creating an attachment
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentCreateInput {
    /// Issue ID to attach to
    pub issue_id: String,
    /// Attachment URL
    pub url: String,
    /// Attachment title
    pub title: String,
    /// Optional subtitle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
}

/// Input for creating a comment
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreateInput {
    /// Issue ID to comment on
    pub issue_id: String,
    /// Comment body (markdown)
    pub body: String,
}

/// Mapping task status to Linear workflow state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending/not started
    Pending,
    /// Task is in progress
    InProgress,
    /// Task is in review
    InReview,
    /// Task is blocked
    Blocked,
    /// Task is complete
    Done,
    /// Task is cancelled
    Cancelled,
}

impl TaskStatus {
    /// Map workflow stage to task status
    #[must_use]
    pub fn from_workflow_stage(stage: &str, phase: &str) -> Self {
        match (stage, phase) {
            (_, "Succeeded") => Self::Done,
            (_, "Failed" | "Error") => Self::Blocked,
            ("pending", _) => Self::Pending,
            (
                "quality-in-progress"
                | "security-in-progress"
                | "testing-in-progress"
                | "waiting-pr-created"
                | "waiting-ready-for-qa",
                _,
            ) => Self::InReview,
            _ => Self::InProgress,
        }
    }

    /// Get Linear workflow state type for this status
    #[must_use]
    pub const fn to_state_type(&self) -> &'static str {
        match self {
            Self::Pending => "unstarted",
            // Linear doesn't have a blocked type, so use "started"
            Self::InProgress | Self::InReview | Self::Blocked => "started",
            Self::Done => "completed",
            Self::Cancelled => "canceled",
        }
    }
}

/// Agent status labels for Linear issues
/// 
/// These labels indicate the current state of agent processing on an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStatus {
    /// Waiting for agent to start processing
    Pending,
    /// Agent is actively working on the task
    Working,
    /// Agent is blocked waiting for user input
    Blocked,
    /// Agent has created a PR for review
    PrCreated,
    /// Agent has successfully completed the task
    Complete,
    /// Agent encountered an error
    Error,
}

impl fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_label_name())
    }
}

impl AgentStatus {
    /// Convert status to Linear label name
    #[must_use]
    pub const fn to_label_name(&self) -> &'static str {
        match self {
            Self::Pending => "agent:pending",
            Self::Working => "agent:working",
            Self::Blocked => "agent:blocked",
            Self::PrCreated => "agent:pr-created",
            Self::Complete => "agent:complete",
            Self::Error => "agent:error",
        }
    }

    /// Get label color for this status (hex)
    #[must_use]
    pub const fn to_color(&self) -> &'static str {
        match self {
            Self::Pending => "#9CA3AF",    // Gray
            Self::Working => "#3B82F6",    // Blue
            Self::Blocked => "#F59E0B",    // Amber
            Self::PrCreated => "#8B5CF6",  // Purple
            Self::Complete => "#10B981",   // Green
            Self::Error => "#EF4444",      // Red
        }
    }

    /// Create from sidecar status string
    #[must_use]
    pub fn from_sidecar_status(status: &str) -> Self {
        match status {
            "pending" | "queued" => Self::Pending,
            "blocked" | "awaiting_input" | "elicitation" => Self::Blocked,
            "review" | "pr_created" | "pr-created" => Self::PrCreated,
            "complete" | "done" | "success" | "succeeded" => Self::Complete,
            "failed" | "error" | "errored" => Self::Error,
            // Default to working for unknown states (including in_progress, working, running)
            _ => Self::Working,
        }
    }
}

/// Input for creating a Linear project
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateInput {
    /// Project name
    pub name: String,
    /// Project description (markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Team IDs to associate with the project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_ids: Option<Vec<String>>,
    /// Lead user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_id: Option<String>,
    /// Target completion date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
}
