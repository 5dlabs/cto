//! Play workflow state transitions.
//!
//! This module handles automatic state transitions for Linear issues
//! based on which agent is working on them.

use anyhow::Result;
use tracing::info;

use crate::client::LinearClient;

/// Bolt has two distinct stages in the workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoltStage {
    /// Task 1: Initial infrastructure setup (DB, cache, storage)
    Infrastructure,
    /// Final: Production deployment (probes, telemetry, DNS)
    Deployment,
}

impl BoltStage {
    /// Get the Linear workflow state name for this Bolt stage.
    #[must_use]
    pub const fn state_name(&self) -> &'static str {
        match self {
            Self::Infrastructure => "Infrastructure",
            Self::Deployment => "Deployment",
        }
    }
}

/// Determine which Bolt stage based on task context.
///
/// Task 1 or tasks containing "infrastructure" are Infrastructure stage.
/// Tasks containing "deploy" are Deployment stage.
#[must_use]
pub fn determine_bolt_stage(task_id: &str, task_title: &str) -> BoltStage {
    let title_lower = task_title.to_lowercase();

    if task_id == "1" || title_lower.contains("infrastructure") {
        BoltStage::Infrastructure
    } else if title_lower.contains("deploy") {
        BoltStage::Deployment
    } else {
        // Default to Infrastructure for ambiguous Bolt tasks
        BoltStage::Infrastructure
    }
}

/// Get the workflow state name for an agent.
///
/// Maps agent names to their corresponding Linear workflow states.
#[must_use]
pub fn get_state_for_agent(agent: &str, bolt_stage: Option<BoltStage>) -> Option<&'static str> {
    match agent.to_lowercase().as_str() {
        "bolt" => Some(bolt_stage.unwrap_or(BoltStage::Infrastructure).state_name()),
        "rex" | "grizz" | "nova" | "blaze" | "tap" | "spark" => Some("Implementation"),
        "cleo" => Some("Quality"),
        "cipher" => Some("Security"),
        "tess" => Some("Testing"),
        "atlas" => Some("Integration"),
        // Morgan and unknown agents don't trigger state changes
        _ => None,
    }
}

/// Update a Linear issue to the appropriate play stage based on agent.
///
/// This is called when a `CodeRun` starts to move the issue to the correct
/// workflow state for the agent handling it.
pub async fn update_play_stage(
    client: &LinearClient,
    issue_id: &str,
    agent: &str,
    bolt_stage: Option<BoltStage>,
) -> Result<()> {
    let Some(new_state) = get_state_for_agent(agent, bolt_stage) else {
        info!(
            agent = %agent,
            issue_id = %issue_id,
            "No state transition for this agent"
        );
        return Ok(());
    };

    info!(
        issue_id = %issue_id,
        agent = %agent,
        new_state = %new_state,
        "Updating issue to play stage"
    );

    client.update_issue_state(issue_id, new_state).await?;

    Ok(())
}

/// Mark a task as done (typically when PR is merged).
pub async fn mark_task_done(client: &LinearClient, issue_id: &str) -> Result<()> {
    info!(issue_id = %issue_id, "Marking task as Done");
    client.update_issue_state(issue_id, "Done").await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_bolt_stage() {
        assert_eq!(
            determine_bolt_stage("1", "Set up infrastructure"),
            BoltStage::Infrastructure
        );
        assert_eq!(
            determine_bolt_stage("5", "Deploy to production"),
            BoltStage::Deployment
        );
        assert_eq!(
            determine_bolt_stage("2", "Infrastructure setup for databases"),
            BoltStage::Infrastructure
        );
        assert_eq!(
            determine_bolt_stage("10", "Final deployment verification"),
            BoltStage::Deployment
        );
        // Default to Infrastructure for ambiguous
        assert_eq!(
            determine_bolt_stage("3", "Some other task"),
            BoltStage::Infrastructure
        );
    }

    #[test]
    fn test_get_state_for_agent() {
        assert_eq!(
            get_state_for_agent("bolt", Some(BoltStage::Infrastructure)),
            Some("Infrastructure")
        );
        assert_eq!(
            get_state_for_agent("bolt", Some(BoltStage::Deployment)),
            Some("Deployment")
        );
        assert_eq!(get_state_for_agent("rex", None), Some("Implementation"));
        assert_eq!(get_state_for_agent("blaze", None), Some("Implementation"));
        assert_eq!(get_state_for_agent("grizz", None), Some("Implementation"));
        assert_eq!(get_state_for_agent("nova", None), Some("Implementation"));
        assert_eq!(get_state_for_agent("tap", None), Some("Implementation"));
        assert_eq!(get_state_for_agent("spark", None), Some("Implementation"));
        assert_eq!(get_state_for_agent("cleo", None), Some("Quality"));
        assert_eq!(get_state_for_agent("cipher", None), Some("Security"));
        assert_eq!(get_state_for_agent("tess", None), Some("Testing"));
        assert_eq!(get_state_for_agent("atlas", None), Some("Integration"));
        assert_eq!(get_state_for_agent("morgan", None), None);
        assert_eq!(get_state_for_agent("unknown", None), None);
    }

    #[test]
    fn test_bolt_stage_state_name() {
        assert_eq!(BoltStage::Infrastructure.state_name(), "Infrastructure");
        assert_eq!(BoltStage::Deployment.state_name(), "Deployment");
    }
}
