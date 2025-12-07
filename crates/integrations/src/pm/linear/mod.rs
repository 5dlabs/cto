//! Linear API client and webhook service for CTO platform integration.
//!
//! This module provides:
//! - GraphQL client for Linear API
//! - Webhook payload parsing and signature verification
//! - Agent Activity emission for Linear's agent system
//! - Type definitions for Linear entities
//!
//! # Example
//!
//! ```no_run
//! use integrations::pm::linear::{LinearClient, ActivityContent};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = LinearClient::new("your-token")?;
//!
//! // Get an issue
//! let issue = client.get_issue("issue-id").await?;
//!
//! // Emit an activity
//! client.emit_thought("session-id", "Processing your request...").await?;
//! # Ok(())
//! # }
//! ```

// Re-export from parent integrations crate root
pub use crate::activities::{
    ActivityContent, ActivitySignal, AgentActivityCreateInput, AgentActivityCreateResponse,
    AgentSessionUpdateInput, AgentSessionUpdateResponse, AuthSignalMetadata, PlanStep,
    PlanStepStatus, SelectOption, SelectSignalMetadata, SignalMetadata,
    AGENT_ACTIVITY_CREATE_MUTATION, AGENT_SESSION_UPDATE_MUTATION,
};
pub use crate::client::LinearClient;
pub use crate::config::{Config, ConfigSource, CtoConfig, IntakeConfig, PlayConfig};
pub use crate::models::{
    AgentSession, AgentSessionState, AgentStatus, Attachment, AttachmentCreateInput, Comment,
    CommentCreateInput, Document, Issue, IssueCreateInput, IssueRelationCreateInput,
    IssueRelationType, IssueUpdateInput, Label, Project, ProjectCreateInput, TaskStatus, Team,
    User, WorkflowState,
};
pub use crate::webhooks::{
    validate_webhook_timestamp, verify_webhook_signature, WebhookAction, WebhookHeaders,
    WebhookPayload, WebhookType,
};
