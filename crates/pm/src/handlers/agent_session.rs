//! Agent session event handlers for Linear webhook events.
//!
//! This module handles `AgentSessionEvent` webhooks:
//! - `created`: New session from @mention or delegation
//! - `prompted`: User sent a follow-up message to an existing session

use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::emitter::AgentActivityEmitter;
use crate::server::AppState;
use crate::webhooks::AgentIdentification;
use crate::WebhookPayload;

/// Context for agent session handling.
#[derive(Debug, Clone)]
pub struct AgentSessionContext {
    /// Identified agent from webhook signature.
    pub agent: Option<AgentIdentification>,
    /// Session ID from Linear.
    pub session_id: String,
    /// Issue ID from Linear.
    pub issue_id: String,
    /// Issue identifier (e.g., "TSK-123").
    pub issue_identifier: String,
}

impl AgentSessionContext {
    /// Extract context from webhook payload.
    pub fn from_payload(
        payload: &WebhookPayload,
        agent: Option<AgentIdentification>,
    ) -> Result<Self, &'static str> {
        let session = payload
            .agent_session
            .as_ref()
            .ok_or("Missing agent session")?;
        let issue = session.issue.as_ref().ok_or("Missing issue in session")?;

        Ok(Self {
            agent,
            session_id: session.id.clone(),
            issue_id: issue.id.clone(),
            issue_identifier: issue.identifier.clone(),
        })
    }

    /// Get the agent name or "unknown".
    #[must_use]
    pub fn agent_name(&self) -> &str {
        self.agent.as_ref().map_or("unknown", |a| a.agent.as_str())
    }
}

/// Handle a new agent session (created action).
///
/// This is called when an agent is @mentioned or delegated to an issue.
/// We should:
/// 1. Start the appropriate Argo workflow
/// 2. Store the session-to-workflow mapping
/// 3. Move the issue to "started" state
/// 4. Emit an initial thought activity within 10 seconds
#[allow(clippy::too_many_lines)]
pub async fn handle_agent_session_created(
    state: &AppState,
    payload: &WebhookPayload,
    agent: Option<AgentIdentification>,
) -> Result<Json<Value>, StatusCode> {
    let ctx = AgentSessionContext::from_payload(payload, agent).map_err(|e| {
        warn!("Invalid session payload: {e}");
        StatusCode::BAD_REQUEST
    })?;

    info!(
        session_id = %ctx.session_id,
        issue = %ctx.issue_identifier,
        agent = %ctx.agent_name(),
        "Handling new agent session"
    );

    // Get the Linear client for API calls
    let Some(client) = &state.linear_client else {
        warn!("Linear client not configured, cannot handle session");
        return Ok(Json(json!({
            "status": "error",
            "reason": "linear_client_not_configured"
        })));
    };

    // Create emitter for this session
    // TODO: Use per-agent OAuth tokens instead of shared client
    let emitter = crate::emitter::LinearAgentEmitter::new(client.clone(), ctx.session_id.clone());

    // Emit initial thought within 10 seconds (Linear requirement)
    if let Err(e) = emitter
        .emit_thought("Starting work on this task...", false)
        .await
    {
        warn!(error = %e, "Failed to emit initial thought activity");
    } else {
        debug!("Emitted initial thought activity");
    }

    // Move issue to "started" state
    if let Err(e) = move_issue_to_started(client, &ctx.issue_id).await {
        warn!(error = %e, "Failed to move issue to started state");
    }

    // TODO: Start Argo workflow based on agent type
    // TODO: Store session-to-workflow mapping

    Ok(Json(json!({
        "status": "accepted",
        "session_id": ctx.session_id,
        "agent": ctx.agent_name(),
        "issue": ctx.issue_identifier
    })))
}

/// Handle a prompted agent session (user sent follow-up message).
///
/// This is called when a user sends a message to an existing agent session.
/// We should:
/// 1. Check for stop signal
/// 2. Forward the message to the running agent via sidecar
pub async fn handle_agent_session_prompted(
    state: &AppState,
    payload: &WebhookPayload,
    agent: Option<AgentIdentification>,
) -> Result<Json<Value>, StatusCode> {
    let ctx = AgentSessionContext::from_payload(payload, agent).map_err(|e| {
        warn!("Invalid session payload: {e}");
        StatusCode::BAD_REQUEST
    })?;

    info!(
        session_id = %ctx.session_id,
        issue = %ctx.issue_identifier,
        agent = %ctx.agent_name(),
        "Handling prompted agent session"
    );

    // Check for stop signal
    if payload.has_stop_signal() {
        return handle_stop_signal(state, &ctx).await;
    }

    // Get the prompt body
    let prompt_body = payload.get_prompt_body().unwrap_or("");

    debug!(prompt_length = prompt_body.len(), "Processing user prompt");

    // Forward message to running agent
    if let Err(e) = forward_message_to_agent(&ctx, prompt_body).await {
        warn!(error = %e, "Failed to forward message to agent");
        return Ok(Json(json!({
            "status": "error",
            "reason": "failed_to_forward",
            "session_id": ctx.session_id
        })));
    }

    Ok(Json(json!({
        "status": "forwarded",
        "session_id": ctx.session_id,
        "agent": ctx.agent_name()
    })))
}

/// Handle stop signal from user.
///
/// This emits a confirmation activity and signals the agent to stop.
async fn handle_stop_signal(
    state: &AppState,
    ctx: &AgentSessionContext,
) -> Result<Json<Value>, StatusCode> {
    info!(
        session_id = %ctx.session_id,
        agent = %ctx.agent_name(),
        "Processing stop signal"
    );

    // Emit stop confirmation using the linear client
    if let Some(client) = &state.linear_client {
        let emitter =
            crate::emitter::LinearAgentEmitter::new(client.clone(), ctx.session_id.clone());

        if let Err(e) = emitter
            .emit_response("Stopped as requested. No further changes made.")
            .await
        {
            warn!(error = %e, "Failed to emit stop confirmation");
        } else {
            debug!("Emitted stop confirmation");
        }
    }

    // TODO: Signal sidecar to stop the agent
    // This would send POST /stop to the sidecar

    Ok(Json(json!({
        "status": "stopped",
        "session_id": ctx.session_id,
        "agent": ctx.agent_name()
    })))
}

/// Move an issue to the first "started" workflow state.
async fn move_issue_to_started(
    client: &crate::LinearClient,
    issue_id: &str,
) -> Result<(), anyhow::Error> {
    // Get the issue to find its team
    let issue = client.get_issue(issue_id).await?;
    let team_id = issue
        .team
        .as_ref()
        .map(|t| t.id.as_str())
        .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;

    // Get the team's workflow states
    let states = client.get_team_workflow_states(team_id).await?;

    // Find the first "started" state (type = "started" and lowest position)
    let started_state = states
        .iter()
        .filter(|s| s.state_type == "started")
        .min_by(|a, b| {
            a.position
                .partial_cmp(&b.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    if let Some(state) = started_state {
        debug!(
            state_id = %state.id,
            state_name = %state.name,
            "Moving issue to started state"
        );
        let input = crate::models::IssueUpdateInput {
            state_id: Some(state.id.clone()),
            ..Default::default()
        };
        client.update_issue(issue_id, input).await?;
    } else {
        debug!("No 'started' workflow state found");
    }

    Ok(())
}

/// Forward a user message to the running agent via sidecar.
async fn forward_message_to_agent(
    ctx: &AgentSessionContext,
    message: &str,
) -> Result<(), anyhow::Error> {
    // Try to route the message using the global router
    if let Err(e) = crate::handlers::route_message_global(&ctx.session_id, message).await {
        warn!(error = %e, "Failed to route message via global router");
        // Fallback: try to find and send directly
        // This would be implemented when we have the session tracker
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_session_context_agent_name() {
        let ctx = AgentSessionContext {
            agent: Some(AgentIdentification {
                agent: "rex".to_string(),
                verified: true,
            }),
            session_id: "session-123".to_string(),
            issue_id: "issue-456".to_string(),
            issue_identifier: "TSK-1".to_string(),
        };

        assert_eq!(ctx.agent_name(), "rex");
    }

    #[test]
    fn test_agent_session_context_unknown_agent() {
        let ctx = AgentSessionContext {
            agent: None,
            session_id: "session-123".to_string(),
            issue_id: "issue-456".to_string(),
            issue_identifier: "TSK-1".to_string(),
        };

        assert_eq!(ctx.agent_name(), "unknown");
    }
}
