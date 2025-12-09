//! Agent communication for two-way interaction with running agents.
//!
//! This module handles forwarding user messages to running Claude agents
//! and receiving responses back.

use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    Client as KubeClient,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Message types that can be sent to a running agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentMessage {
    /// A message from the user (Linear mention/prompt).
    UserMessage {
        /// Message content from the user.
        content: String,
        /// Optional session ID for tracking.
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        /// Optional issue identifier for context.
        #[serde(skip_serializing_if = "Option::is_none")]
        issue_identifier: Option<String>,
    },
    /// A stop/cancel signal.
    Stop {
        /// Reason for stopping.
        reason: String,
    },
}

impl AgentMessage {
    /// Create a user message.
    #[must_use]
    pub fn user_message(content: impl Into<String>) -> Self {
        Self::UserMessage {
            content: content.into(),
            session_id: None,
            issue_identifier: None,
        }
    }

    /// Create a user message with context.
    #[must_use]
    pub fn user_message_with_context(
        content: impl Into<String>,
        session_id: impl Into<String>,
        issue_identifier: impl Into<String>,
    ) -> Self {
        Self::UserMessage {
            content: content.into(),
            session_id: Some(session_id.into()),
            issue_identifier: Some(issue_identifier.into()),
        }
    }

    /// Create a stop message.
    #[must_use]
    pub fn stop(reason: impl Into<String>) -> Self {
        Self::Stop {
            reason: reason.into(),
        }
    }
}

/// Information about a running agent pod.
#[derive(Debug, Clone)]
pub struct RunningAgent {
    /// Pod name.
    pub pod_name: String,
    /// Pod namespace.
    pub namespace: String,
    /// Container name (usually "main" or "agent").
    pub container_name: String,
    /// Session ID the agent is handling.
    pub session_id: String,
    /// Issue identifier (e.g., "CTOPA-21").
    pub issue_identifier: Option<String>,
    /// Agent type (intake, play, etc).
    pub agent_type: String,
}

/// Find running agent pods for a Linear session.
///
/// Looks for pods with the `linear-session` label matching the session ID.
///
/// # Errors
/// Returns error if Kubernetes API call fails.
pub async fn find_running_agents(
    kube_client: &KubeClient,
    namespace: &str,
    session_id: &str,
) -> Result<Vec<RunningAgent>> {
    let pods: Api<Pod> = Api::namespaced(kube_client.clone(), namespace);

    // Look for pods with linear-session label
    let label_selector = format!("linear-session={session_id}");
    let lp = ListParams::default().labels(&label_selector);

    let pod_list = pods
        .list(&lp)
        .await
        .context("Failed to list pods for session")?;

    let mut agents = Vec::new();

    for pod in pod_list {
        let pod_name = pod.metadata.name.clone().unwrap_or_default();
        let labels = pod.metadata.labels.clone().unwrap_or_default();

        // Check if pod is running
        let phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_ref())
            .map_or("Unknown", String::as_str);

        if phase != "Running" {
            debug!(pod = %pod_name, phase = %phase, "Skipping non-running pod");
            continue;
        }

        // Determine container name and agent type
        let container_name = determine_main_container(&pod);
        let agent_type = labels
            .get("cto.5dlabs.io/agent-type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let issue_identifier = labels.get("cto.5dlabs.io/linear-issue").cloned();

        agents.push(RunningAgent {
            pod_name,
            namespace: namespace.to_string(),
            container_name,
            session_id: session_id.to_string(),
            issue_identifier,
            agent_type,
        });
    }

    info!(
        session_id = %session_id,
        count = agents.len(),
        "Found running agents for session"
    );

    Ok(agents)
}

/// Find running agents by issue identifier.
///
/// # Errors
/// Returns error if Kubernetes API call fails.
pub async fn find_agents_by_issue(
    kube_client: &KubeClient,
    namespace: &str,
    issue_identifier: &str,
) -> Result<Vec<RunningAgent>> {
    let pods: Api<Pod> = Api::namespaced(kube_client.clone(), namespace);

    // Look for pods with linear-issue label
    let label_selector = format!("cto.5dlabs.io/linear-issue={issue_identifier}");
    let lp = ListParams::default().labels(&label_selector);

    let pod_list = pods
        .list(&lp)
        .await
        .context("Failed to list pods for issue")?;

    let mut agents = Vec::new();

    for pod in pod_list {
        let pod_name = pod.metadata.name.clone().unwrap_or_default();
        let labels = pod.metadata.labels.clone().unwrap_or_default();

        // Check if pod is running
        let phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_ref())
            .map_or("Unknown", String::as_str);

        if phase != "Running" {
            continue;
        }

        let container_name = determine_main_container(&pod);
        let agent_type = labels
            .get("cto.5dlabs.io/agent-type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let session_id = labels
            .get("linear-session")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        agents.push(RunningAgent {
            pod_name,
            namespace: namespace.to_string(),
            container_name,
            session_id,
            issue_identifier: Some(issue_identifier.to_string()),
            agent_type,
        });
    }

    Ok(agents)
}

/// Determine the main container name for an agent pod.
fn determine_main_container(pod: &Pod) -> String {
    let Some(spec) = &pod.spec else {
        return "main".to_string();
    };

    // Look for specific container names in order of preference
    let preferred_names = ["agent", "main", "claude", "opencode"];

    for name in preferred_names {
        if spec.containers.iter().any(|c| c.name == name) {
            return name.to_string();
        }
    }

    // Fall back to first container
    spec.containers
        .first()
        .map_or_else(|| "main".to_string(), |c| c.name.clone())
}

/// Send a message to a running agent.
///
/// For Claude Code, this writes to the agent-input JSONL file.
/// The agent reads this file and processes incoming messages.
///
/// # Errors
/// Returns error if the message cannot be sent.
pub async fn send_message_to_agent(agent: &RunningAgent, message: &AgentMessage) -> Result<()> {
    let message_json = serde_json::to_string(message).context("Failed to serialize message")?;

    info!(
        pod = %agent.pod_name,
        container = %agent.container_name,
        agent_type = %agent.agent_type,
        "Sending message to agent"
    );

    // Use kubectl exec to write to the agent input file
    // Claude Code expects JSONL input on stdin or via --input-file
    let shell_script = format!(
        r"
        # Try multiple possible input locations
        if [ -p /agent-input/commands.jsonl ]; then
            echo '{message_json}' >> /agent-input/commands.jsonl
        elif [ -f /workspace/.agent-input ]; then
            echo '{message_json}' >> /workspace/.agent-input
        elif [ -d /agent-input ]; then
            echo '{message_json}' >> /agent-input/commands.jsonl
        else
            # Create the input file if it doesn't exist
            mkdir -p /agent-input
            echo '{message_json}' >> /agent-input/commands.jsonl
        fi
        "
    );

    let output = tokio::process::Command::new("kubectl")
        .args([
            "exec",
            "-n",
            &agent.namespace,
            &agent.pod_name,
            "-c",
            &agent.container_name,
            "--",
            "sh",
            "-c",
            &shell_script,
        ])
        .output()
        .await
        .context("Failed to execute kubectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(
            pod = %agent.pod_name,
            error = %stderr,
            "Failed to send message to agent"
        );
        return Err(anyhow!("kubectl exec failed: {stderr}"));
    }

    debug!(
        pod = %agent.pod_name,
        "Message sent successfully"
    );

    Ok(())
}

/// Send a user message to all running agents for a session.
///
/// # Errors
/// Returns error if no agents are found or message delivery fails.
pub async fn broadcast_to_session(
    kube_client: &KubeClient,
    namespace: &str,
    session_id: &str,
    content: &str,
    issue_identifier: Option<&str>,
) -> Result<usize> {
    let agents = find_running_agents(kube_client, namespace, session_id).await?;

    if agents.is_empty() {
        return Err(anyhow!("No running agents found for session {session_id}"));
    }

    let message = AgentMessage::UserMessage {
        content: content.to_string(),
        session_id: Some(session_id.to_string()),
        issue_identifier: issue_identifier.map(String::from),
    };

    let mut sent_count = 0;
    for agent in &agents {
        match send_message_to_agent(agent, &message).await {
            Ok(()) => sent_count += 1,
            Err(e) => {
                warn!(
                    pod = %agent.pod_name,
                    error = %e,
                    "Failed to send message to agent"
                );
            }
        }
    }

    info!(
        session_id = %session_id,
        sent = sent_count,
        total = agents.len(),
        "Broadcast message to session agents"
    );

    Ok(sent_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_message_serialization() {
        let msg = AgentMessage::user_message("Hello agent!");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user_message"));
        assert!(json.contains("Hello agent!"));

        let msg = AgentMessage::user_message_with_context("Do this task", "session-123", "TSK-1");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("session-123"));
        assert!(json.contains("issue_identifier"));
        assert!(json.contains("TSK-1"));
    }

    #[test]
    fn test_stop_message() {
        let msg = AgentMessage::stop("User requested cancellation");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stop"));
        assert!(json.contains("User requested cancellation"));
    }
}


