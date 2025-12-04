//! Remediation tracker for managing retry loops and state.
//!
//! This module tracks the lifecycle of CI remediation attempts:
//! - Monitors CodeRun completion status
//! - Manages retry logic with different agents
//! - Coordinates with OpenMemory for learning
//! - Triggers escalation after max attempts

use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::memory::{MemoryClient, MemoryConfig};
use super::router::CiRouter;
use super::spawner::CodeRunSpawner;
use super::types::{
    Agent, AttemptOutcome, CiFailure, CiFailureType, RemediationConfig, RemediationContext,
    RemediationState, RemediationStatus,
};

/// Remediation tracker state.
pub struct RemediationTracker {
    /// Active remediation states by workflow run ID
    active: Arc<RwLock<HashMap<u64, TrackedRemediation>>>,
    /// Memory client for historical context
    memory: Option<MemoryClient>,
    /// Router for agent selection
    router: CiRouter,
    /// Configuration
    config: RemediationConfig,
}

/// Tracked remediation with full context.
#[derive(Debug, Clone)]
pub struct TrackedRemediation {
    /// Workflow run ID (primary key)
    pub workflow_run_id: u64,
    /// The original failure
    pub failure: CiFailure,
    /// Classified failure type
    pub failure_type: CiFailureType,
    /// Current remediation state
    pub state: RemediationState,
    /// Active CodeRun name (if any)
    pub active_coderun: Option<String>,
    /// Repository
    pub repository: String,
    /// Branch
    pub branch: String,
    /// PR number if available
    pub pr_number: Option<u32>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// CodeRun completion event.
#[derive(Debug, Clone, Deserialize)]
pub struct CodeRunCompletion {
    /// CodeRun name
    pub name: String,
    /// Workflow run ID from label
    pub workflow_run_id: u64,
    /// Task ID from label
    pub task_id: String,
    /// Completion status
    pub status: CodeRunStatus,
    /// Agent that ran
    pub agent: String,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Duration in seconds
    pub duration_secs: u64,
}

/// CodeRun completion status.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeRunStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
}

impl RemediationTracker {
    /// Create a new tracker.
    pub fn new(config: RemediationConfig, memory_config: Option<MemoryConfig>) -> Result<Self> {
        let memory = memory_config
            .map(MemoryClient::new)
            .transpose()
            .context("Failed to create memory client")?;

        Ok(Self {
            active: Arc::new(RwLock::new(HashMap::new())),
            memory,
            router: CiRouter::new(),
            config,
        })
    }

    /// Track a new remediation.
    pub async fn track(
        &self,
        failure: CiFailure,
        failure_type: CiFailureType,
        pr_number: Option<u32>,
    ) -> u64 {
        let workflow_run_id = failure.workflow_run_id;
        let now = Utc::now();

        let mut state = RemediationState::new(failure.clone(), failure_type.clone());
        state.pr_number = pr_number;

        let tracked = TrackedRemediation {
            workflow_run_id,
            repository: failure.repository.clone(),
            branch: failure.branch.clone(),
            failure,
            failure_type,
            state,
            active_coderun: None,
            pr_number,
            created_at: now,
            updated_at: now,
        };

        let mut active = self.active.write().await;
        active.insert(workflow_run_id, tracked);

        info!("Tracking remediation for workflow run {workflow_run_id}");

        workflow_run_id
    }

    /// Record a CodeRun spawn.
    pub async fn record_spawn(&self, workflow_run_id: u64, coderun_name: &str, agent: Agent) {
        let mut active = self.active.write().await;
        if let Some(tracked) = active.get_mut(&workflow_run_id) {
            tracked.active_coderun = Some(coderun_name.to_string());
            tracked.state.status = RemediationStatus::InProgress;
            tracked.updated_at = Utc::now();

            debug!(
                "Recorded spawn of {} ({}) for workflow {}",
                coderun_name,
                agent.name(),
                workflow_run_id
            );
        }
    }

    /// Handle CodeRun completion.
    ///
    /// Returns the next action to take.
    pub async fn handle_completion(
        &self,
        completion: CodeRunCompletion,
        spawner: &CodeRunSpawner,
    ) -> Result<CompletionAction> {
        let workflow_run_id = completion.workflow_run_id;

        let mut active = self.active.write().await;
        let tracked = match active.get_mut(&workflow_run_id) {
            Some(t) => t,
            None => {
                warn!(
                    "Received completion for unknown workflow run {}",
                    workflow_run_id
                );
                return Ok(CompletionAction::Unknown);
            }
        };

        tracked.active_coderun = None;
        tracked.updated_at = Utc::now();

        // Parse agent from completion
        let agent = match completion.agent.as_str() {
            "rex" => Agent::Rex,
            "blaze" => Agent::Blaze,
            "bolt" => Agent::Bolt,
            "cipher" => Agent::Cipher,
            "atlas" => Agent::Atlas,
            _ => Agent::Atlas,
        };

        // Map completion status to attempt outcome
        let outcome = match completion.status {
            CodeRunStatus::Success => AttemptOutcome::Success,
            CodeRunStatus::Failed => AttemptOutcome::AgentFailed,
            CodeRunStatus::Timeout => AttemptOutcome::Timeout,
            CodeRunStatus::Cancelled => AttemptOutcome::Escalated,
        };

        // Record the attempt
        tracked
            .state
            .record_attempt(outcome, &completion.name, agent);

        // Store outcome in memory
        if let Some(memory) = &self.memory {
            let description = completion
                .error_message
                .as_deref()
                .unwrap_or("No details");
            let _ = memory
                .store_remediation_outcome(
                    &tracked.failure,
                    Some(&tracked.failure_type),
                    agent,
                    &format!("{outcome:?}").to_lowercase(),
                    description,
                )
                .await;
        }

        // Determine next action
        match completion.status {
            CodeRunStatus::Success => {
                tracked.state.status = RemediationStatus::Succeeded;

                info!(
                    "Remediation succeeded for workflow {} after {} attempts",
                    workflow_run_id,
                    tracked.state.attempts.len()
                );

                // Store routing success
                if let Some(memory) = &self.memory {
                    let _ = memory
                        .store_routing_decision(
                            Some(&tracked.failure_type),
                            agent,
                            None,
                            true,
                        )
                        .await;
                }

                Ok(CompletionAction::Success)
            }
            _ => {
                // Check if we should retry
                let attempts_made = tracked.state.attempts.len() as u32;
                let should_retry = attempts_made < self.config.max_attempts;

                if should_retry {
                    let next_agent = self.select_next_agent(tracked, agent).await;

                    info!(
                        "Retrying workflow {} with agent {} (attempt {})",
                        workflow_run_id,
                        next_agent.name(),
                        attempts_made + 1
                    );

                    // Build retry context
                    let ctx = self.build_retry_context(tracked).await?;

                    // Spawn will happen outside this function
                    drop(active); // Release lock before spawning

                    // Spawn new CodeRun
                    match spawner.spawn(next_agent, &ctx) {
                        Ok(coderun_name) => {
                            // Re-acquire lock to update
                            let mut active = self.active.write().await;
                            if let Some(tracked) = active.get_mut(&workflow_run_id) {
                                tracked.active_coderun = Some(coderun_name.clone());
                            }

                            Ok(CompletionAction::Retry {
                                agent: next_agent,
                                coderun: coderun_name,
                            })
                        }
                        Err(e) => {
                            warn!("Failed to spawn retry CodeRun: {e}");
                            Ok(CompletionAction::SpawnFailed(e.to_string()))
                        }
                    }
                } else {
                    // Max attempts reached, escalate
                    tracked.state.status = RemediationStatus::Escalated;

                    warn!(
                        "Max attempts reached for workflow {}, escalating",
                        workflow_run_id
                    );

                    // Store escalation in memory
                    if let Some(memory) = &self.memory {
                        let _ = memory
                            .store_escalation(
                                &tracked.failure,
                                Some(&tracked.failure_type),
                                tracked.state.attempts.len() as u32,
                                &completion.error_message.unwrap_or_default(),
                            )
                            .await;
                    }

                    let failure = tracked.failure.clone();
                    let attempts = tracked.state.attempts.clone();
                    let pr_number = tracked.pr_number;

                    Ok(CompletionAction::Escalate {
                        failure,
                        attempts,
                        pr_number,
                    })
                }
            }
        }
    }

    /// Select the next agent for retry.
    async fn select_next_agent(&self, tracked: &TrackedRemediation, failed_agent: Agent) -> Agent {
        // Try a different agent
        let ctx = RemediationContext {
            failure: Some(tracked.failure.clone()),
            failure_type: Some(tracked.failure_type.clone()),
            changed_files: Vec::new(),
            workflow_logs: String::new(),
            ..Default::default()
        };

        self.router.try_different_agent(failed_agent, &ctx)
    }

    /// Build context for a retry attempt.
    async fn build_retry_context(&self, tracked: &TrackedRemediation) -> Result<RemediationContext> {
        // Get historical context from memory
        let historical = if let Some(memory) = &self.memory {
            memory
                .query_similar_failures(&tracked.failure, Some(&tracked.failure_type))
                .await
                .unwrap_or_default()
        } else {
            super::types::HistoricalContext::default()
        };

        Ok(RemediationContext {
            failure: Some(tracked.failure.clone()),
            failure_type: Some(tracked.failure_type.clone()),
            pr: tracked.pr_number.map(|n| super::types::PullRequest {
                number: n,
                title: String::new(),
                head_ref: tracked.branch.clone(),
                base_ref: "main".to_string(),
                html_url: format!(
                    "https://github.com/{}/pull/{}",
                    tracked.repository, n
                ),
                state: "open".to_string(),
                mergeable: None,
                checks_status: String::new(),
            }),
            previous_attempts: tracked.state.attempts.clone(),
            historical: Some(historical),
            ..Default::default()
        })
    }

    /// Get active remediations.
    pub async fn get_active(&self) -> Vec<TrackedRemediation> {
        let active = self.active.read().await;
        active.values().cloned().collect()
    }

    /// Get remediation by workflow run ID.
    pub async fn get(&self, workflow_run_id: u64) -> Option<TrackedRemediation> {
        let active = self.active.read().await;
        active.get(&workflow_run_id).cloned()
    }

    /// Remove completed/escalated remediations.
    pub async fn cleanup(&self) {
        let mut active = self.active.write().await;
        let before = active.len();

        active.retain(|_, v| {
            matches!(
                v.state.status,
                RemediationStatus::Pending | RemediationStatus::InProgress
            )
        });

        let removed = before - active.len();
        if removed > 0 {
            info!("Cleaned up {removed} completed remediations");
        }
    }
}

/// Action to take after a CodeRun completes.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum CompletionAction {
    /// Remediation succeeded
    Success,
    /// Retry with a different agent
    Retry {
        agent: Agent,
        coderun: String,
    },
    /// Escalate to human
    Escalate {
        failure: CiFailure,
        attempts: Vec<super::types::RemediationAttempt>,
        pr_number: Option<u32>,
    },
    /// Unknown workflow run ID
    Unknown,
    /// Failed to spawn retry
    SpawnFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coderun_status_deserialize() {
        let success: CodeRunStatus = serde_json::from_str("\"success\"").unwrap();
        assert_eq!(success, CodeRunStatus::Success);

        let failed: CodeRunStatus = serde_json::from_str("\"failed\"").unwrap();
        assert_eq!(failed, CodeRunStatus::Failed);
    }
}
