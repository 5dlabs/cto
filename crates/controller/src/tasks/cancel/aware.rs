//! # State-Aware Cancellation System
//!
//! This module provides state-aware cancellation logic that integrates with distributed
//! locking to prevent unnecessary cancellations and ensure proper coordination.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use kube::api::{Api, DeleteParams, ListParams};
use kube::{Client, Error as KubeError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::crds::CodeRun;
use crate::tasks::cancel::lock::DistributedLock;
use crate::tasks::cancel::lock::LeaseError;

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

/// State-aware cancellation manager that uses distributed locking
pub struct StateAwareCancellation {
    client: Client,
    namespace: String,
    lock_manager: DistributedLock,
    #[allow(dead_code)]
    cancellation_timeout: Duration, // For future use
    #[allow(dead_code)]
    grace_period: Duration, // For future use
}

impl StateAwareCancellation {
    /// Create a new state-aware cancellation manager
    #[must_use]
    pub fn new(client: Client, namespace: &str) -> Self {
        let lock_manager = DistributedLock::new(
            client.clone(),
            namespace,
            "cancellation-lock",
            "state-aware-cancellation",
        );

        Self {
            client,
            namespace: namespace.to_string(),
            lock_manager,
            cancellation_timeout: Duration::from_secs(300), // 5 minutes
            grace_period: Duration::from_secs(30),          // 30 seconds
        }
    }

    /// Cancel agents with distributed locking
    ///
    /// # Errors
    ///
    /// Returns `CancellationError` if the distributed lock cannot be obtained
    /// or Kubernetes API calls fail.
    ///
    /// # Panics
    ///
    /// Panics if `SystemTime::now()` returns a value earlier than `UNIX_EPOCH`
    /// while generating the correlation identifier. This is not expected in
    /// normal environments.
    #[allow(clippy::too_many_lines)]
    pub async fn cancel_agents_with_state_check(
        &self,
        task_id: &str,
        pr_number: i32,
    ) -> Result<CancellationResult, CancellationError> {
        // Allow SystemTime::now() for generating unique correlation ID (not time-dependent logic)
        #[allow(clippy::disallowed_methods)]
        let correlation_id = format!(
            "cancel-{}-{}",
            task_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        debug!(
            task_id = %task_id,
            pr_number = pr_number,
            correlation_id = %correlation_id,
            "Starting state-aware cancellation"
        );

        // Acquire distributed lock to prevent concurrent cancellations
        let lock_name = format!("cancel-{task_id}");
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
                    reason: format!("Lock held by: {holder}"),
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

        // Check if agents have already completed
        if self.agents_completed(task_id).await? {
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

        // Perform the actual cancellation
        let result = self
            .perform_cancellation(task_id, pr_number, &correlation_id)
            .await;

        // Log result
        match &result {
            Ok(cancellation_result) => {
                info!(
                    task_id = %task_id,
                    cancelled = cancellation_result.cancelled_agents.len(),
                    skipped = cancellation_result.skipped_agents.len(),
                    "Cancellation completed successfully"
                );
            }
            Err(e) => {
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
    async fn agents_completed(&self, task_id: &str) -> Result<bool, CancellationError> {
        // Check if CodeRuns exist and are in completed state
        let coderun_api: Api<CodeRun> = Api::namespaced(self.client.clone(), &self.namespace);

        let label_selector = format!("task-id={task_id}");
        let lp = ListParams::default().labels(&label_selector);

        let coderuns = coderun_api
            .list(&lp)
            .await
            .map_err(CancellationError::KubeError)?;

        if coderuns.items.is_empty() {
            debug!(task_id = %task_id, "No CodeRuns found for task");
            return Ok(false);
        }

        // Check if any CodeRuns are still running
        for coderun in &coderuns.items {
            if let Some(status) = &coderun.status {
                if status.phase == "Running" || status.phase == "Pending" {
                    debug!(
                        task_id = %task_id,
                        coderun_name = %coderun.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                        phase = ?status.phase,
                        "Found running CodeRun, cancellation needed"
                    );
                    return Ok(false);
                }
            }
        }

        debug!(task_id = %task_id, "All CodeRuns completed, no cancellation needed");
        Ok(true)
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
        let coderun_api: Api<CodeRun> = Api::namespaced(self.client.clone(), &self.namespace);
        let label_selector = format!("task-id={task_id}");
        let lp = ListParams::default().labels(&label_selector);

        let coderuns = coderun_api
            .list(&lp)
            .await
            .map_err(CancellationError::KubeError)?;

        for coderun in coderuns.items {
            let coderun_name = coderun.metadata.name.as_ref().ok_or_else(|| {
                CancellationError::ResourceNotFound {
                    resource: "CodeRun without name".to_string(),
                }
            })?;

            let agent_type = coderun
                .metadata
                .labels
                .as_ref()
                .and_then(|labels| labels.get("agent-type"))
                .map_or("unknown", |s| s.as_str());

            // Check current phase
            let should_cancel = if let Some(status) = &coderun.status {
                matches!(status.phase.as_str(), "Running" | "Pending")
            } else {
                true // Cancel if no status available
            };

            if should_cancel {
                match self.cancel_single_coderun(coderun.clone()).await {
                    Ok(()) => {
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
                        let error_msg = format!("Cancellation failed: {e}");
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

    /// Cancel a single `CodeRun` with proper error handling
    async fn cancel_single_coderun(&self, coderun: CodeRun) -> Result<(), CancellationError> {
        let coderun_name =
            coderun
                .metadata
                .name
                .as_ref()
                .ok_or_else(|| CancellationError::ResourceNotFound {
                    resource: "CodeRun without name".to_string(),
                })?;

        let coderun_api: Api<CodeRun> = Api::namespaced(self.client.clone(), &self.namespace);

        // Force delete the CodeRun
        let dp = DeleteParams {
            grace_period_seconds: Some(0),
            ..Default::default()
        };

        coderun_api.delete(coderun_name, &dp).await?;

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
            cancelled_agents: vec![AgentInfo {
                name: "coderun-1".to_string(),
                agent_type: "cleo".to_string(),
                reason: "Successfully cancelled".to_string(),
            }],
            skipped_agents: vec![],
            reason: "Test cancellation".to_string(),
            correlation_id: "test-correlation".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: CancellationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.task_id, deserialized.task_id);
        assert_eq!(result.pr_number, deserialized.pr_number);
        assert_eq!(
            result.cancelled_agents.len(),
            deserialized.cancelled_agents.len()
        );
    }
}
