//! Agent Activity Emitter - Workflow-agnostic abstraction for emitting Linear activities.
//!
//! This module provides a trait-based abstraction for emitting Linear Agent Activities
//! that can be used by any workflow (intake, play, GitHub handlers, etc.).
//!
//! # Design Principles
//!
//! - **Workflow-agnostic**: No workflow-specific terminology
//! - **Generic activities**: Works for any workflow phase
//! - **Environment-based config**: Session ID and token from environment
//! - **Two consumption methods**: Library API and CLI binary

use anyhow::Result;
use async_trait::async_trait;

use crate::activities::PlanStep;

/// Trait for emitting Linear Agent Activities.
///
/// This abstraction allows any workflow container or handler to emit activities
/// without being coupled to the `LinearClient` implementation directly.
///
/// # Example
///
/// ```rust,ignore
/// let emitter = LinearAgentEmitter::new(client, session_id);
///
/// // Emit a thought (internal reasoning)
/// emitter.emit_thought("Analyzing the PRD...", true).await?;
///
/// // Emit an action in progress
/// emitter.emit_action("Parsing", "prd.md").await?;
///
/// // Emit action completion
/// emitter.emit_action_complete("Parsed", "prd.md", "Found 15 requirements").await?;
///
/// // Update the plan checklist
/// emitter.update_plan(&[
///     PlanStep::completed("Parse PRD"),
///     PlanStep::in_progress("Generate tasks"),
///     PlanStep::pending("Expand subtasks"),
/// ]).await?;
///
/// // Emit final response
/// emitter.emit_response("Generated 12 tasks from PRD").await?;
/// ```
#[async_trait]
pub trait AgentActivityEmitter: Send + Sync {
    /// Emit a thought activity (internal reasoning/status).
    ///
    /// # Arguments
    /// * `body` - The thought content (markdown supported)
    /// * `ephemeral` - If true, will be replaced by the next activity
    ///
    /// # Returns
    /// The activity ID if successful
    async fn emit_thought(&self, body: &str, ephemeral: bool) -> Result<String>;

    /// Emit an action activity (tool/step in progress).
    ///
    /// # Arguments
    /// * `action` - Action name (e.g., "Parsing", "Running tests")
    /// * `parameter` - Action parameter (e.g., file path, query)
    ///
    /// # Returns
    /// The activity ID if successful
    async fn emit_action(&self, action: &str, parameter: &str) -> Result<String>;

    /// Emit an action completion with result.
    ///
    /// # Arguments
    /// * `action` - Action name (past tense, e.g., "Parsed", "Tests passed")
    /// * `parameter` - Action parameter
    /// * `result` - Action result (markdown supported)
    ///
    /// # Returns
    /// The activity ID if successful
    async fn emit_action_complete(
        &self,
        action: &str,
        parameter: &str,
        result: &str,
    ) -> Result<String>;

    /// Emit a response activity (work completed).
    ///
    /// # Arguments
    /// * `body` - The response content (markdown supported)
    ///
    /// # Returns
    /// The activity ID if successful
    async fn emit_response(&self, body: &str) -> Result<String>;

    /// Emit an error activity.
    ///
    /// # Arguments
    /// * `body` - The error message (markdown supported)
    ///
    /// # Returns
    /// The activity ID if successful
    async fn emit_error(&self, body: &str) -> Result<String>;

    /// Emit an elicitation activity (request user input).
    ///
    /// # Arguments
    /// * `body` - The prompt for the user (markdown supported)
    ///
    /// # Returns
    /// The activity ID if successful
    async fn emit_elicitation(&self, body: &str) -> Result<String>;

    /// Update the session plan (visual checklist).
    ///
    /// Plans are shown in Linear UI as a checklist of steps.
    /// The plan array **replaces** the existing plan entirely.
    ///
    /// # Arguments
    /// * `steps` - Array of plan steps with content and status
    ///
    /// # Returns
    /// `true` if the update succeeded
    async fn update_plan(&self, steps: &[PlanStep]) -> Result<bool>;

    /// Get the session ID this emitter is bound to.
    fn session_id(&self) -> &str;
}

/// Linear-backed implementation of `AgentActivityEmitter`.
///
/// Wraps a `LinearClient` with a bound session ID for convenient activity emission.
pub struct LinearAgentEmitter {
    client: crate::LinearClient,
    session_id: String,
}

impl LinearAgentEmitter {
    /// Create a new emitter bound to a session.
    ///
    /// # Arguments
    /// * `client` - The Linear API client
    /// * `session_id` - The agent session ID to emit activities to
    #[must_use]
    pub fn new(client: crate::LinearClient, session_id: impl Into<String>) -> Self {
        Self {
            client,
            session_id: session_id.into(),
        }
    }

    /// Create an emitter from environment variables.
    ///
    /// Reads:
    /// - `LINEAR_API_TOKEN` - OAuth access token or API key
    /// - `LINEAR_SESSION_ID` - Agent session ID
    ///
    /// # Errors
    /// Returns error if environment variables are missing or client creation fails.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("LINEAR_API_TOKEN")
            .map_err(|_| anyhow::anyhow!("LINEAR_API_TOKEN not set"))?;
        let session_id = std::env::var("LINEAR_SESSION_ID")
            .map_err(|_| anyhow::anyhow!("LINEAR_SESSION_ID not set"))?;

        let client = crate::LinearClient::new(&token)?;
        Ok(Self::new(client, session_id))
    }

    /// Create an emitter with explicit token and session ID.
    ///
    /// # Arguments
    /// * `token` - Linear API token
    /// * `session_id` - Agent session ID
    ///
    /// # Errors
    /// Returns error if client creation fails.
    pub fn with_credentials(token: &str, session_id: impl Into<String>) -> Result<Self> {
        let client = crate::LinearClient::new(token)?;
        Ok(Self::new(client, session_id))
    }
}

#[async_trait]
impl AgentActivityEmitter for LinearAgentEmitter {
    async fn emit_thought(&self, body: &str, ephemeral: bool) -> Result<String> {
        if ephemeral {
            self.client.emit_ephemeral_thought(&self.session_id, body).await
        } else {
            self.client.emit_thought(&self.session_id, body).await
        }
    }

    async fn emit_action(&self, action: &str, parameter: &str) -> Result<String> {
        self.client
            .emit_action(&self.session_id, action, parameter)
            .await
    }

    async fn emit_action_complete(
        &self,
        action: &str,
        parameter: &str,
        result: &str,
    ) -> Result<String> {
        self.client
            .emit_action_with_result(&self.session_id, action, parameter, result)
            .await
    }

    async fn emit_response(&self, body: &str) -> Result<String> {
        self.client.emit_response(&self.session_id, body).await
    }

    async fn emit_error(&self, body: &str) -> Result<String> {
        self.client.emit_error(&self.session_id, body).await
    }

    async fn emit_elicitation(&self, body: &str) -> Result<String> {
        self.client.emit_elicitation(&self.session_id, body).await
    }

    async fn update_plan(&self, steps: &[PlanStep]) -> Result<bool> {
        self.client
            .update_plan(&self.session_id, steps.to_vec())
            .await
    }

    fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activities::PlanStepStatus;

    #[test]
    fn test_plan_step_creation() {
        let step = PlanStep::pending("Test step");
        assert_eq!(step.content, "Test step");
        assert_eq!(step.status, PlanStepStatus::Pending);

        let step = PlanStep::in_progress("Active step");
        assert_eq!(step.status, PlanStepStatus::InProgress);

        let step = PlanStep::completed("Done step");
        assert_eq!(step.status, PlanStepStatus::Completed);
    }
}
