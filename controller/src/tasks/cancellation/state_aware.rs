//! # State-Aware Cancellation System
//!
//! This module provides state-aware cancellation logic that integrates with the remediation
//! state management system to prevent unnecessary cancellations and ensure proper state tracking.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, DeleteParams, ListParams, PatchParams, Patch};
use kube::core::ObjectMeta;
use kube::{Client, Error as KubeError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn, error};

use crate::crds::platformv1;
use crate::remediation::{RemediationState, RemediationStateManager};
use crate::tasks::cancellation::distributed_lock::{DistributedLock, Lease};
use crate::tasks::cancellation::LeaseError;

/// Errors that can occur during state-aware cancellation operations
#[derive(Error, Debug)]
pub enum CancellationError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] KubeError),

    #[error("Distributed lock error: {0}")]
    LockError(#[from] LeaseError),

    #[error("State management error: {0}")]
    StateError(String),

    #[error("Cancellation failed: {message}")]
    CancellationFailed { message: String },

    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },

    #[error("Operation timeout: {operation}")]
    Timeout { operation: String },
}

/// Cancellation request with task and PR information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationRequest {
    pub task_id: String,
    pub pr_number: i32,
    pub correlation_id: String,
    pub requester: String,
    pub reason: String,
    pub force: bool,
}

/// State-aware cancellation manager that integrates with remediation state
#[derive(Clone)]
pub struct StateAwareCancellation {
    client: Client,
    namespace: String,
    state_manager: RemediationStateManager,
    lock_manager: DistributedLock,
    cancellation_timeout: Duration,
    grace_period: Duration,
}

impl StateAwareCancellation {
    /// Create a new state-aware cancellation manager
    pub fn new(
        client: Client,
        namespace: &str,
        state_manager: RemediationStateManager,
    ) -> Self {
        let lock_manager = DistributedLock::new(
            client.clone(),
            namespace,
            "cancellation-lock",
            "state-aware-cancellation",
        );

        Self {
            client,
            namespace: namespace.to_string(),
            state_manager,
            lock_manager,
            cancellation_timeout: Duration::from_secs(300), // 5 minutes
            grace_period: Duration::from_secs(30), // 30 seconds
        }
    }

    /// Cancel agents with state awareness and distributed locking
    pub async fn cancel_agents_with_state_check(
        &self,
        task_id: &str,
        pr_number: i32,
    ) -> Result<CancellationResult, CancellationError> {
        let correlation_id = format!("cancel-{}-{}", task_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());

        debug!(
            task_id = %task_id,
            pr_number = pr_number,
            correlation_id = %correlation_id,
            "Starting state-aware cancellation"
        );

        // Acquire distributed lock to prevent concurrent cancellations
        let lock_name = format!("cancel-{}", task_id);
        let lease = match self.lock_manager.try_acquire().await {
            Ok(lease) => {
                info!(
                    task_id = %task_id,
                    lock_name = %lock_name,
                    holder = %lease.holder(),
                    "Acquired distributed lock for cancellation"
                );
                lease
            }
            Err(LeaseError::LockHeld { holder }) => {
                warn!(
                    task_id = %task_id,
                    holder = %holder,
                    "Cancellation lock held by another process, skipping"
                );
                return Ok(CancellationResult {
                    task_id: task_id.to_string(),
                    pr_number,
                    cancelled_agents: vec![],
                    skipped_agents: vec![],
                    reason: format!("Lock held by: {}", holder),
                    correlation_id,
                });
            }
            Err(e) => {
                error!(
                    task_id = %task_id,
                    error = %e,
                    "Failed to acquire cancellation lock"
                );
                return Err(CancellationError::LockError(e));
            }
        };

        // Ensure lease is released when function exits
        let _lease_guard = lease;

        // Check current remediation state
        let state_result = self.state_manager.get_state(task_id).await;

        match state_result {
            Ok(Some(state)) => {
                if state.cancellation_in_progress {
                    info!(
                        task_id = %task_id,
                        "Cancellation already in progress, skipping"
                    );
                    return Ok(CancellationResult {
                        task_id: task_id.to_string(),
                        pr_number,
                        cancelled_agents: vec![],
                        skipped_agents: vec![],
                        reason: "Cancellation already in progress".to_string(),
                        correlation_id,
                    });
                }

                // Check if agents have already completed
                if self.agents_completed(&state).await? {
                    info!(
                        task_id = %task_id,
                        "Agents have already completed, skipping cancellation"
                    );
                    return Ok(CancellationResult {
                        task_id: task_id.to_string(),
                        pr_number,
                        cancelled_agents: vec![],
                        skipped_agents: vec![],
                        reason: "Agents already completed".to_string(),
                        correlation_id,
                    });
                }

                // Mark cancellation as in progress
                self.mark_cancellation_started(task_id).await?;
            }
            Ok(None) => {
                debug!(
                    task_id = %task_id,
                    "No existing state found, proceeding with cancellation"
                );
            }
            Err(e) => {
                warn!(
                    task_id = %task_id,
                    error = %e,
                    "Failed to check remediation state, proceeding cautiously"
                );
            }
        }

        // Perform the actual cancellation
        let result = self.perform_cancellation(task_id, pr_number, &correlation_id).await;

        // Update state after cancellation
        match &result {
            Ok(cancellation_result) => {
                self.mark_cancellation_completed(task_id, cancellation_result).await?;
                info!(
                    task_id = %task_id,
                    cancelled = cancellation_result.cancelled_agents.len(),
                    skipped = cancellation_result.skipped_agents.len(),
                    "Cancellation completed successfully"
                );
            }
            Err(e) => {
                self.mark_cancellation_failed(task_id, e).await?;
                error!(
                    task_id = %task_id,
                    error = %e,
                    "Cancellation failed"
                );
            }
        }

        result
    }

    /// Check if agents have already completed their work
    async fn agents_completed(&self, state: &RemediationState) -> Result<bool, CancellationError> {
        // Check if CodeRuns exist and are in completed state
        let coderun_api: Api<platformv1::CodeRun> = Api::namespaced(self.client.clone(), &self.namespace);

        let label_selector = format!("task-id={}", state.task_id);
        let lp = ListParams::default().labels(&label_selector);

        let coderuns = coderun_api.list(&lp).await
            .map_err(|e| CancellationError::KubeError(e))?;

        if coderuns.items.is_empty() {
            debug!(task_id = %state.task_id, "No CodeRuns found for task");
            return Ok(false);
        }

        // Check if any CodeRuns are still running
        for coderun in &coderuns.items {
            if let Some(status) = &coderun.status {
                if status.phase == platformv1::CodeRunPhase::Running ||
                   status.phase == platformv1::CodeRunPhase::Pending {
                    debug!(
                        task_id = %state.task_id,
                        coderun_name = %coderun.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                        phase = ?status.phase,
                        "Found running CodeRun, cancellation needed"
                    );
                    return Ok(false);
                }
            }
        }

        debug!(task_id = %state.task_id, "All CodeRuns completed, no cancellation needed");
        Ok(true)
    }

    /// Mark cancellation as started in state
    async fn mark_cancellation_started(&self, task_id: &str) -> Result<(), CancellationError> {
        let mut state = self.state_manager.get_state(task_id).await?
            .unwrap_or_else(|| RemediationState::new(task_id.to_string()));

        state.cancellation_in_progress = true;
        state.last_updated = chrono::Utc::now();

        if let Err(e) = self.state_manager.update_state(state).await {
            warn!(
                task_id = %task_id,
                error = %e,
                "Failed to mark cancellation as started in state"
            );
        }

        Ok(())
    }

    /// Perform the actual cancellation of agents
    async fn perform_cancellation(
        &self,
        task_id: &str,
        pr_number: i32,
        correlation_id: &str,
    ) -> Result<CancellationResult, CancellationError> {
        let mut cancelled_agents = Vec::new();
        let mut skipped_agents = Vec::new();

        // Find CodeRuns to cancel
        let coderun_api: Api<platformv1::CodeRun> = Api::namespaced(self.client.clone(), &self.namespace);
        let label_selector = format!("task-id={}", task_id);
        let lp = ListParams::default().labels(&label_selector);

        let coderuns = coderun_api.list(&lp).await
            .map_err(|e| CancellationError::KubeError(e))?;

        for coderun in coderuns.items {
            let coderun_name = coderun.metadata.name.as_ref()
                .ok_or_else(|| CancellationError::ResourceNotFound {
                    resource: "CodeRun without name".to_string()
                })?;

            let agent_type = coderun.metadata.labels
                .as_ref()
                .and_then(|labels| labels.get("agent-type"))
                .unwrap_or("unknown");

            // Check current phase
            let should_cancel = if let Some(status) = &coderun.status {
                matches!(status.phase,
                    platformv1::CodeRunPhase::Running |
                    platformv1::CodeRunPhase::Pending
                )
            } else {
                true // Cancel if no status available
            };

            if should_cancel {
                match self.cancel_single_coderun(coderun).await {
                    Ok(_) => {
                        cancelled_agents.push(AgentInfo {
                            name: coderun_name.clone(),
                            agent_type: agent_type.to_string(),
                            reason: "Successfully cancelled".to_string(),
                        });
                        info!(
                            task_id = %task_id,
                            coderun_name = %coderun_name,
                            agent_type = %agent_type,
                            "Successfully cancelled CodeRun"
                        );
                    }
                    Err(e) => {
                        let error_msg = format!("Cancellation failed: {}", e);
                        skipped_agents.push(AgentInfo {
                            name: coderun_name.clone(),
                            agent_type: agent_type.to_string(),
                            reason: error_msg.clone(),
                        });
                        warn!(
                            task_id = %task_id,
                            coderun_name = %coderun_name,
                            agent_type = %agent_type,
                            error = %e,
                            "Failed to cancel CodeRun"
                        );
                    }
                }
            } else {
                skipped_agents.push(AgentInfo {
                    name: coderun_name.clone(),
                    agent_type: agent_type.to_string(),
                    reason: "Already completed".to_string(),
                });
                debug!(
                    task_id = %task_id,
                    coderun_name = %coderun_name,
                    agent_type = %agent_type,
                    "Skipping completed CodeRun"
                );
            }
        }

        Ok(CancellationResult {
            task_id: task_id.to_string(),
            pr_number,
            cancelled_agents,
            skipped_agents,
            reason: "Cancellation completed".to_string(),
            correlation_id: correlation_id.to_string(),
        })
    }

    /// Cancel a single CodeRun with proper error handling
    async fn cancel_single_coderun(&self, coderun: platformv1::CodeRun) -> Result<(), CancellationError> {
        let coderun_name = coderun.metadata.name.as_ref()
            .ok_or_else(|| CancellationError::ResourceNotFound {
                resource: "CodeRun without name".to_string()
            })?;

        let coderun_api: Api<platformv1::CodeRun> = Api::namespaced(self.client.clone(), &self.namespace);

        // First, try graceful termination by updating spec
        if let Some(mut spec) = coderun.spec.clone() {
            spec.terminate = Some(true);

            let patch = serde_json::json!({
                "spec": {
                    "terminate": true
                }
            });

            let patch_data = serde_json::to_vec(&patch)
                .map_err(|e| CancellationError::CancellationFailed {
                    message: format!("Failed to serialize patch: {}", e)
                })?;

            coderun_api
                .patch(coderun_name, &PatchParams::apply("cancellation-controller"), &Patch::Apply(&patch_data))
                .await?;

            // Wait for graceful termination
            tokio::time::sleep(self.grace_period).await;
        }

        // Force delete if still running
        let dp = DeleteParams {
            grace_period_seconds: Some(0),
            ..Default::default()
        };

        coderun_api
            .delete(coderun_name, &dp)
            .await?;

        Ok(())
    }

    /// Mark cancellation as completed in state
    async fn mark_cancellation_completed(
        &self,
        task_id: &str,
        result: &CancellationResult,
    ) -> Result<(), CancellationError> {
        let mut state = self.state_manager.get_state(task_id).await?
            .unwrap_or_else(|| RemediationState::new(task_id.to_string()));

        state.cancellation_in_progress = false;
        state.last_updated = chrono::Utc::now();

        // Record cancellation statistics
        if let Some(stats) = &mut state.cancellation_stats {
            stats.total_cancellations += 1;
            stats.successful_cancellations += result.cancelled_agents.len() as u32;
            stats.last_cancellation_time = Some(chrono::Utc::now());
        }

        if let Err(e) = self.state_manager.update_state(state).await {
            warn!(
                task_id = %task_id,
                error = %e,
                "Failed to mark cancellation as completed in state"
            );
        }

        Ok(())
    }

    /// Mark cancellation as failed in state
    async fn mark_cancellation_failed(
        &self,
        task_id: &str,
        error: &CancellationError,
    ) -> Result<(), CancellationError> {
        let mut state = self.state_manager.get_state(task_id).await?
            .unwrap_or_else(|| RemediationState::new(task_id.to_string()));

        state.cancellation_in_progress = false;
        state.last_updated = chrono::Utc::now();

        // Record failure
        if let Some(stats) = &mut state.cancellation_stats {
            stats.failed_cancellations += 1;
            stats.last_cancellation_error = Some(error.to_string());
        }

        if let Err(e) = self.state_manager.update_state(state).await {
            warn!(
                task_id = %task_id,
                error = %e,
                "Failed to mark cancellation as failed in state"
            );
        }

        Ok(())
    }
}

/// Information about a cancelled or skipped agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub agent_type: String,
    pub reason: String,
}

/// Result of a cancellation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationResult {
    pub task_id: String,
    pub pr_number: i32,
    pub cancelled_agents: Vec<AgentInfo>,
    pub skipped_agents: Vec<AgentInfo>,
    pub reason: String,
    pub correlation_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_result_serialization() {
        let result = CancellationResult {
            task_id: "test-task".to_string(),
            pr_number: 123,
            cancelled_agents: vec![
                AgentInfo {
                    name: "coderun-1".to_string(),
                    agent_type: "cleo".to_string(),
                    reason: "Successfully cancelled".to_string(),
                }
            ],
            skipped_agents: vec![],
            reason: "Test cancellation".to_string(),
            correlation_id: "test-correlation".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: CancellationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.task_id, deserialized.task_id);
        assert_eq!(result.pr_number, deserialized.pr_number);
        assert_eq!(result.cancelled_agents.len(), deserialized.cancelled_agents.len());
    }
}
