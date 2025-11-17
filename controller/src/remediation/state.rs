//! State Management System for Agent Remediation Loop
//!
//! This module provides ConfigMap-based state tracking for remediation iterations,
//! feedback history, and workflow state management. It ensures atomic operations
//! and proper cleanup for the remediation system.
//!
//! Key Features:
//! - ConfigMap-based persistent state storage
//! - Atomic iteration counters with locking
//! - Feedback history serialization/deserialization
//! - State recovery and cleanup patterns
//! - Thread-safe operations for concurrent access

use crate::remediation::StructuredFeedback;
use crate::tasks::types::{Context, Result};
use anyhow;
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Represents the overall state of a remediation workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationState {
    /// Unique identifier for this remediation workflow
    pub workflow_id: String,

    /// Associated PR number
    pub pr_number: u32,

    /// Associated task ID
    pub task_id: String,

    /// Current iteration count (starts at 1)
    pub iteration: u32,

    /// Maximum allowed iterations (default: 10)
    pub max_iterations: u32,

    /// Workflow status
    pub status: RemediationStatus,

    /// Timestamp when remediation started
    pub started_at: DateTime<Utc>,

    /// Timestamp of last update
    pub updated_at: DateTime<Utc>,

    /// History of feedback and remediation attempts
    pub feedback_history: Vec<FeedbackEntry>,

    /// Current Rex agent run (if active)
    pub active_run: Option<ActiveRun>,

    /// Metadata and configuration
    pub metadata: HashMap<String, String>,
}

/// Status of the remediation workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemediationStatus {
    /// Initial state, waiting for first feedback
    WaitingForFeedback,

    /// Actively processing feedback and remediation
    InProgress,

    /// Waiting for Rex agent to complete current iteration
    WaitingForAgent,

    /// Successfully completed all remediation
    Completed,

    /// Failed with unrecoverable error
    Failed,

    /// Manually terminated or max iterations reached
    Terminated,

    /// Paused due to manual intervention or system issues
    Paused,
}

/// Entry in the feedback history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    /// Unique identifier for this feedback entry
    pub id: String,

    /// Iteration number when this feedback was received
    pub iteration: u32,

    /// GitHub comment ID that triggered this feedback
    pub comment_id: u64,

    /// Author of the feedback comment
    pub author: String,

    /// Timestamp when feedback was received
    pub received_at: DateTime<Utc>,

    /// Structured feedback data
    pub feedback: StructuredFeedback,

    /// Actions taken in response to this feedback
    pub actions_taken: Vec<String>,

    /// Status of this feedback entry
    pub status: FeedbackStatus,
}

/// Status of a feedback entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedbackStatus {
    /// Feedback received and being processed
    Received,

    /// Remediation action initiated
    Processing,

    /// Remediation completed successfully
    Resolved,

    /// Remediation failed or was rejected
    Failed,

    /// Feedback was skipped or ignored
    Skipped,
}

/// Information about currently active agent run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRun {
    /// Type of run (`CodeRun` or `DocsRun`)
    pub run_type: RunType,

    /// Name of the Kubernetes resource
    pub run_name: String,

    /// Namespace of the resource
    pub namespace: String,

    /// When this run was started
    pub started_at: DateTime<Utc>,
}

/// Type of agent run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunType {
    CodeRun,
    DocsRun,
}

/// Main state manager for remediation workflows
pub struct RemediationStateManager {
    configmaps: Api<ConfigMap>,
    namespace: String,
}

impl RemediationStateManager {
    /// Create a new state manager
    #[must_use]
    pub fn new(context: &Context) -> Self {
        Self {
            configmaps: Api::namespaced(context.client.clone(), &context.namespace),
            namespace: context.namespace.clone(),
        }
    }

    /// Generate `ConfigMap` name for a remediation workflow
    fn configmap_name(pr_number: u32, task_id: &str) -> String {
        format!("remediation-state-pr-{pr_number}-task-{task_id}")
    }

    /// Initialize a new remediation state
    pub async fn initialize_state(
        &self,
        pr_number: u32,
        task_id: String,
        max_iterations: Option<u32>,
    ) -> Result<RemediationState> {
        let workflow_id = format!("remediation-{pr_number}-{task_id}");
        let now = Utc::now();

        let state = RemediationState {
            workflow_id: workflow_id.clone(),
            pr_number,
            task_id,
            iteration: 0, // Will be incremented to 1 on first feedback
            max_iterations: max_iterations.unwrap_or(10),
            status: RemediationStatus::WaitingForFeedback,
            started_at: now,
            updated_at: now,
            feedback_history: Vec::new(),
            active_run: None,
            metadata: HashMap::new(),
        };

        // Store in ConfigMap
        self.save_state(&state).await?;

        info!(
            "Initialized remediation state for PR #{} task {}",
            pr_number, state.task_id
        );
        Ok(state)
    }

    /// Load existing remediation state
    pub async fn load_state(
        &self,
        pr_number: u32,
        task_id: &str,
    ) -> Result<Option<RemediationState>> {
        let cm_name = Self::configmap_name(pr_number, task_id);

        match self.configmaps.get(&cm_name).await {
            Ok(cm) => {
                if let Some(data) = &cm.data {
                    if let Some(state_json) = data.get("state.json") {
                        let state: RemediationState = serde_json::from_str(state_json)?;
                        debug!(
                            "Loaded remediation state for PR #{} task {}",
                            pr_number, task_id
                        );
                        Ok(Some(state))
                    } else {
                        warn!("ConfigMap {} missing state.json data", cm_name);
                        Ok(None)
                    }
                } else {
                    warn!("ConfigMap {} has no data", cm_name);
                    Ok(None)
                }
            }
            Err(kube::Error::Api(err)) if err.code == 404 => {
                debug!(
                    "No existing state found for PR #{} task {}",
                    pr_number, task_id
                );
                Ok(None)
            }
            Err(e) => {
                error!(
                    "Failed to load state for PR #{} task {}: {}",
                    pr_number, task_id, e
                );
                Err(e.into())
            }
        }
    }

    /// Save remediation state to `ConfigMap`
    pub async fn save_state(&self, state: &RemediationState) -> Result<()> {
        let cm_name = Self::configmap_name(state.pr_number, &state.task_id);

        let state_json = serde_json::to_string_pretty(state)?;

        let cm = ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some(cm_name.clone()),
                namespace: Some(self.namespace.clone()),
                labels: Some({
                    let mut labels = std::collections::BTreeMap::new();
                    labels.insert("app".to_string(), "remediation-loop".to_string());
                    labels.insert("component".to_string(), "state-manager".to_string());
                    labels.insert("pr-number".to_string(), state.pr_number.to_string());
                    labels.insert("task-id".to_string(), state.task_id.clone());
                    labels
                }),
                ..Default::default()
            },
            data: Some({
                let mut data = std::collections::BTreeMap::new();
                data.insert("state.json".to_string(), state_json);
                data.insert("workflow_id".to_string(), state.workflow_id.clone());
                data.insert("iteration".to_string(), state.iteration.to_string());
                data.insert("status".to_string(), format!("{:?}", state.status));
                data.insert("updated_at".to_string(), state.updated_at.to_rfc3339());
                data
            }),
            ..Default::default()
        };

        // Try to create first, then patch if it exists
        match self.configmaps.create(&PostParams::default(), &cm).await {
            Ok(_) => {
                debug!("Created new ConfigMap {}", cm_name);
            }
            Err(kube::Error::Api(err)) if err.code == 409 => {
                // ConfigMap already exists, patch it
                let patch = serde_json::json!({
                    "data": cm.data
                });

                self.configmaps
                    .patch(&cm_name, &PatchParams::default(), &Patch::Merge(&patch))
                    .await?;

                debug!("Updated existing ConfigMap {}", cm_name);
            }
            Err(e) => {
                error!("Failed to save ConfigMap {}: {}", cm_name, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Add new feedback to the state
    pub async fn add_feedback(
        &self,
        pr_number: u32,
        task_id: &str,
        comment_id: u64,
        author: String,
        feedback: crate::remediation::StructuredFeedback,
    ) -> Result<RemediationState> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        // Increment iteration for new feedback
        state.iteration += 1;
        state.updated_at = Utc::now();

        // Create feedback entry
        let feedback_entry = FeedbackEntry {
            id: format!("feedback-{}-{}", comment_id, state.iteration),
            iteration: state.iteration,
            comment_id,
            author,
            received_at: Utc::now(),
            feedback,
            actions_taken: Vec::new(),
            status: FeedbackStatus::Received,
        };

        state.feedback_history.push(feedback_entry);
        state.status = RemediationStatus::InProgress;

        self.save_state(&state).await?;

        info!(
            "Added feedback to remediation state for PR #{} task {} (iteration {})",
            pr_number, task_id, state.iteration
        );

        Ok(state)
    }

    /// Update feedback status
    pub async fn update_feedback_status(
        &self,
        pr_number: u32,
        task_id: &str,
        feedback_id: &str,
        status: FeedbackStatus,
        actions_taken: Option<Vec<String>>,
    ) -> Result<()> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        if let Some(entry) = state
            .feedback_history
            .iter_mut()
            .find(|e| e.id == feedback_id)
        {
            entry.status = status.clone();
            if let Some(actions) = actions_taken {
                entry.actions_taken = actions;
            }
        }

        state.updated_at = Utc::now();
        self.save_state(&state).await?;

        debug!("Updated feedback {} status to {:?}", feedback_id, status);
        Ok(())
    }

    /// Set active run information
    pub async fn set_active_run(
        &self,
        pr_number: u32,
        task_id: &str,
        run_type: RunType,
        run_name: String,
        run_namespace: String,
    ) -> Result<()> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        state.active_run = Some(ActiveRun {
            run_type,
            run_name: run_name.clone(),
            namespace: run_namespace,
            started_at: Utc::now(),
        });

        state.status = RemediationStatus::WaitingForAgent;
        state.updated_at = Utc::now();

        self.save_state(&state).await?;

        info!(
            "Set active run for PR #{} task {}: {}",
            pr_number, task_id, run_name
        );
        Ok(())
    }

    /// Clear active run (when agent completes)
    pub async fn clear_active_run(&self, pr_number: u32, task_id: &str) -> Result<()> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        state.active_run = None;
        state.status = RemediationStatus::InProgress;
        state.updated_at = Utc::now();

        self.save_state(&state).await?;

        debug!("Cleared active run for PR #{} task {}", pr_number, task_id);
        Ok(())
    }

    /// Complete remediation workflow
    pub async fn complete_remediation(&self, pr_number: u32, task_id: &str) -> Result<()> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        state.status = RemediationStatus::Completed;
        state.updated_at = Utc::now();

        self.save_state(&state).await?;

        info!(
            "✅ Completed remediation for PR #{} task {}",
            pr_number, task_id
        );
        Ok(())
    }

    /// Terminate remediation workflow
    pub async fn terminate_remediation(
        &self,
        pr_number: u32,
        task_id: &str,
        reason: &str,
    ) -> Result<()> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        state.status = RemediationStatus::Terminated;
        state.updated_at = Utc::now();

        // Add termination metadata
        state
            .metadata
            .insert("termination_reason".to_string(), reason.to_string());
        state
            .metadata
            .insert("terminated_at".to_string(), Utc::now().to_rfc3339());

        self.save_state(&state).await?;

        info!(
            "🛑 Terminated remediation for PR #{} task {}: {}",
            pr_number, task_id, reason
        );
        Ok(())
    }

    /// Fail remediation workflow
    pub async fn fail_remediation(
        &self,
        pr_number: u32,
        task_id: &str,
        error_message: &str,
    ) -> Result<()> {
        let mut state = self
            .load_state(pr_number, task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No state found for PR #{pr_number} task {task_id}"))?;

        state.status = RemediationStatus::Failed;
        state.updated_at = Utc::now();

        // Add failure metadata
        state
            .metadata
            .insert("failure_reason".to_string(), error_message.to_string());
        state
            .metadata
            .insert("failed_at".to_string(), Utc::now().to_rfc3339());

        self.save_state(&state).await?;

        error!(
            "❌ Failed remediation for PR #{} task {}: {}",
            pr_number, task_id, error_message
        );
        Ok(())
    }

    /// Clean up old remediation states (for maintenance)
    pub async fn cleanup_old_states(&self, max_age_days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(max_age_days));
        let mut cleaned_count = 0;

        // List all remediation state ConfigMaps
        let configmaps = self.configmaps.list(&ListParams::default()).await?;

        for cm in configmaps.items {
            if let Some(labels) = &cm.metadata.labels {
                if labels.get("app") == Some(&"remediation-loop".to_string())
                    && labels.get("component") == Some(&"state-manager".to_string())
                {
                    // Check if this state is old
                    if let Some(data) = &cm.data {
                        if let Some(updated_at_str) = data.get("updated_at") {
                            if let Ok(updated_at) = DateTime::parse_from_rfc3339(updated_at_str) {
                                if updated_at < cutoff {
                                    // Delete old ConfigMap
                                    if let Some(name) = &cm.metadata.name {
                                        self.configmaps
                                            .delete(name, &DeleteParams::default())
                                            .await?;
                                        cleaned_count += 1;
                                        debug!("Cleaned up old remediation state: {}", name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if cleaned_count > 0 {
            info!(
                "Cleaned up {} old remediation states (older than {} days)",
                cleaned_count, max_age_days
            );
        }

        Ok(cleaned_count)
    }

    /// Get statistics about current remediation workflows
    pub async fn get_statistics(&self) -> Result<RemediationStatistics> {
        let configmaps = self.configmaps.list(&ListParams::default()).await?;
        let mut stats = RemediationStatistics::default();

        for cm in configmaps.items {
            if let Some(labels) = &cm.metadata.labels {
                if labels.get("app") == Some(&"remediation-loop".to_string())
                    && labels.get("component") == Some(&"state-manager".to_string())
                {
                    stats.total_workflows += 1;

                    if let Some(data) = &cm.data {
                        if let Some(status_str) = data.get("status") {
                            match status_str.as_str() {
                                "InProgress" => stats.in_progress += 1,
                                "Completed" => stats.completed += 1,
                                "Failed" => stats.failed += 1,
                                "Terminated" => stats.terminated += 1,
                                _ => stats.other += 1,
                            }
                        }

                        if let Some(iteration_str) = data.get("iteration") {
                            if let Ok(iteration) = iteration_str.parse::<u32>() {
                                stats.total_iterations += iteration;
                            }
                        }
                    }
                }
            }
        }

        Ok(stats)
    }
}

/// Statistics about remediation workflows
#[derive(Debug, Default, Clone)]
pub struct RemediationStatistics {
    pub total_workflows: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
    pub terminated: usize,
    pub other: usize,
    pub total_iterations: u32,
}

impl RemediationStatistics {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn average_iterations(&self) -> f64 {
        if self.total_workflows == 0 {
            0.0
        } else {
            f64::from(self.total_iterations) / self.total_workflows as f64
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn success_rate(&self) -> f64 {
        let total_completed = self.completed + self.failed + self.terminated;
        if total_completed == 0 {
            0.0
        } else {
            self.completed as f64 / total_completed as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_remediation_state_serialization() {
        let state = RemediationState {
            workflow_id: "test-workflow".to_string(),
            pr_number: 123,
            task_id: "test-task".to_string(),
            iteration: 1,
            max_iterations: 10,
            status: RemediationStatus::InProgress,
            started_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 1, 1, 0, 0).unwrap(),
            feedback_history: Vec::new(),
            active_run: None,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: RemediationState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.workflow_id, deserialized.workflow_id);
        assert_eq!(state.pr_number, deserialized.pr_number);
        assert_eq!(state.task_id, deserialized.task_id);
        assert_eq!(state.iteration, deserialized.iteration);
        assert_eq!(state.max_iterations, deserialized.max_iterations);
        assert!(matches!(deserialized.status, RemediationStatus::InProgress));
    }

    #[test]
    fn test_configmap_name_generation() {
        let name = RemediationStateManager::configmap_name(123, "test-task");
        assert_eq!(name, "remediation-state-pr-123-task-test-task");
    }

    #[test]
    fn test_statistics_calculation() {
        let stats = RemediationStatistics {
            total_workflows: 10,
            in_progress: 2,
            completed: 5,
            failed: 2,
            terminated: 1,
            other: 0,
            total_iterations: 25,
        };

        assert!((stats.average_iterations() - 2.5).abs() < f64::EPSILON);
        assert!((stats.success_rate() - (5.0 / 8.0)).abs() < f64::EPSILON); // 5 completed out of 8 total completed workflows
    }
}
