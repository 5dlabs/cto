//! # Label Orchestrator
//!
//! This module implements the core state machine for workflow transitions,
//! managing the complex logic of moving between remediation states based on events.

use crate::remediation::RemediationStateManager;
use crate::tasks::label::client::{GitHubLabelClient, GitHubLabelError};
use crate::tasks::label::override_detector::{OverrideDetector, OverrideError};
use crate::tasks::label::schema::{
    LabelOperation, LabelOperationType, LabelSchema, StateTransition, WorkflowState,
};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

/// Label orchestrator for managing workflow state transitions
pub struct LabelOrchestrator {
    label_client: GitHubLabelClient,
    label_schema: LabelSchema,
    #[allow(dead_code)]
    state_manager: Arc<RemediationStateManager>,
    override_detector: OverrideDetector,
}

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("GitHub API error: {0}")]
    GitHubError(#[from] GitHubLabelError),

    #[error("Invalid transition: {0}")]
    InvalidTransition(String),

    #[error("State manager error: {0}")]
    StateManagerError(String),

    #[error("Override detected: {0}")]
    OverrideDetected(String),

    #[error("Condition evaluation failed: {0}")]
    ConditionError(String),

    #[error("Label operation failed: {0}")]
    LabelOperationError(String),
}

impl From<OverrideError> for OrchestratorError {
    fn from(error: OverrideError) -> Self {
        OrchestratorError::OverrideDetected(error.to_string())
    }
}

impl LabelOrchestrator {
    /// Create a new label orchestrator
    #[must_use]
    pub fn new(
        label_client: GitHubLabelClient,
        state_manager: Arc<RemediationStateManager>,
        override_detector: OverrideDetector,
    ) -> Self {
        Self {
            label_client,
            label_schema: LabelSchema::default(),
            state_manager,
            override_detector,
        }
    }

    /// Execute a state transition based on a trigger event
    #[instrument(skip(self), fields(pr_number = %pr_number, task_id = %task_id, trigger = %trigger))]
    pub async fn transition_state(
        &mut self,
        pr_number: i32,
        task_id: &str,
        trigger: &str,
        context: Option<serde_json::Value>,
    ) -> Result<(), OrchestratorError> {
        info!(
            "Processing state transition for PR #{} with trigger '{}'",
            pr_number, trigger
        );

        // Get current labels and check for overrides
        let current_labels = self.label_client.get_labels(pr_number).await?;
        debug!("Current labels: {:?}", current_labels);

        // Check for override labels first
        let override_status = self
            .override_detector
            .check_override_status(pr_number, task_id)
            .await?;
        if override_status.has_override {
            return Err(OrchestratorError::OverrideDetected(
                override_status
                    .message
                    .unwrap_or_else(|| "Automation override active".to_string()),
            ));
        }

        // Determine current state
        let current_state = self.label_schema.determine_workflow_state(&current_labels);
        debug!("Current workflow state: {:?}", current_state);

        // Find valid transition
        let transition = self
            .find_transition(&current_state, trigger)
            .ok_or_else(|| {
                OrchestratorError::InvalidTransition(format!(
                    "No valid transition from {current_state:?} with trigger '{trigger}'"
                ))
            })?;

        // Validate transition conditions
        self.validate_transition_conditions(&transition, task_id)?;

        // Execute the transition
        self.execute_transition(pr_number, task_id, transition.clone(), context)
            .await?;

        info!(
            "Successfully completed state transition: {:?} -> {:?}",
            transition.from, transition.to
        );

        Ok(())
    }

    /// Get the current workflow state for a PR
    pub async fn get_current_state(
        &mut self,
        pr_number: i32,
    ) -> Result<WorkflowState, OrchestratorError> {
        let labels = self.label_client.get_labels(pr_number).await?;
        Ok(self.label_schema.determine_workflow_state(&labels))
    }

    /// Check if a transition is valid without executing it
    pub async fn validate_transition(
        &mut self,
        pr_number: i32,
        task_id: &str,
        from_state: &WorkflowState,
        to_state: &WorkflowState,
        trigger: &str,
    ) -> Result<bool, OrchestratorError> {
        // Check for overrides
        let override_status = self
            .override_detector
            .check_override_status(pr_number, task_id)
            .await?;
        if override_status.has_override {
            return Ok(false);
        }

        // Check if transition exists
        let transition_exists = self
            .label_schema
            .is_valid_transition(from_state, to_state, trigger);
        if !transition_exists {
            return Ok(false);
        }

        // Get transition and validate conditions
        if let Some(transition) = self
            .label_schema
            .get_transition(from_state, to_state, trigger)
        {
            self.validate_transition_conditions(transition, task_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Find a valid transition for the given state and trigger
    fn find_transition(
        &self,
        current_state: &WorkflowState,
        trigger: &str,
    ) -> Option<StateTransition> {
        self.label_schema
            .state_transitions
            .iter()
            .find(|transition| &transition.from == current_state && transition.trigger == trigger)
            .cloned()
    }

    /// Validate all conditions for a transition
    fn validate_transition_conditions(
        &self,
        transition: &StateTransition,
        task_id: &str,
    ) -> Result<(), OrchestratorError> {
        for condition in &transition.conditions {
            if !self.evaluate_condition(condition, task_id)? {
                return Err(OrchestratorError::ConditionError(format!(
                    "Condition '{condition}' not satisfied for task {task_id}"
                )));
            }
        }
        Ok(())
    }

    /// Evaluate a single condition
    fn evaluate_condition(
        &self,
        condition: &str,
        task_id: &str,
    ) -> Result<bool, OrchestratorError> {
        if condition.starts_with("iteration ") {
            let current_iteration = Self::get_current_iteration(task_id);
            self.evaluate_iteration_condition(condition, current_iteration)
        } else {
            warn!("Unknown condition type: {}", condition);
            Ok(false)
        }
    }

    /// Get the current iteration for a task
    fn get_current_iteration(_task_id: &str) -> i32 {
        // This would integrate with the state manager from Task 4
        // For now, return a placeholder
        1
    }

    /// Evaluate iteration-based conditions
    #[allow(clippy::unused_self)]
    fn evaluate_iteration_condition(
        &self,
        condition: &str,
        current_iteration: i32,
    ) -> Result<bool, OrchestratorError> {
        let pattern = regex::Regex::new(r"iteration\s*(>=|<=|>|<|==)\s*(\d+)").map_err(|e| {
            OrchestratorError::ConditionError(format!("Invalid iteration condition pattern: {e}"))
        })?;

        if let Some(captures) = pattern.captures(condition) {
            let operator = &captures[1];
            let value: i32 = captures[2].parse().map_err(|_| {
                OrchestratorError::ConditionError(format!(
                    "Invalid iteration value in condition: {condition}"
                ))
            })?;

            match operator {
                ">=" => Ok(current_iteration >= value),
                "<=" => Ok(current_iteration <= value),
                ">" => Ok(current_iteration > value),
                "<" => Ok(current_iteration < value),
                "==" => Ok(current_iteration == value),
                _ => Ok(false),
            }
        } else {
            Err(OrchestratorError::ConditionError(format!(
                "Invalid iteration condition: {condition}"
            )))
        }
    }

    /// Execute a transition by performing the required actions
    async fn execute_transition(
        &mut self,
        pr_number: i32,
        task_id: &str,
        transition: StateTransition,
        context: Option<serde_json::Value>,
    ) -> Result<(), OrchestratorError> {
        info!("Executing transition actions: {:?}", transition.actions);

        let mut operations = Vec::new();
        let mut iteration_update = None;

        // Process each action
        for action in &transition.actions {
            Self::process_action(action, task_id, &mut operations, &mut iteration_update);
        }

        // Execute label operations atomically
        if !operations.is_empty() {
            debug!("Executing {} label operations", operations.len());
            self.label_client
                .update_labels_atomic(pr_number, &operations)
                .await?;
        }

        // Log the transition
        Self::log_transition(pr_number, task_id, &transition, iteration_update, context);

        Ok(())
    }

    /// Process a single transition action
    fn process_action(
        action: &str,
        task_id: &str,
        operations: &mut Vec<LabelOperation>,
        iteration_update: &mut Option<i32>,
    ) {
        match action {
            "add_needs_fixes" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Add,
                    labels: vec!["needs-fixes".to_string()],
                    from_label: None,
                });
            }
            "remove_needs_fixes" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Remove,
                    labels: vec!["needs-fixes".to_string()],
                    from_label: None,
                });
            }
            "add_fixing_in_progress" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Add,
                    labels: vec!["fixing-in-progress".to_string()],
                    from_label: None,
                });
            }
            "remove_fixing_in_progress" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Remove,
                    labels: vec!["fixing-in-progress".to_string()],
                    from_label: None,
                });
            }
            "add_needs_cleo" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Add,
                    labels: vec!["needs-cleo".to_string()],
                    from_label: None,
                });
            }
            "remove_needs_cleo" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Remove,
                    labels: vec!["needs-cleo".to_string()],
                    from_label: None,
                });
            }
            "add_needs_tess" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Add,
                    labels: vec!["needs-tess".to_string()],
                    from_label: None,
                });
            }
            "remove_needs_tess" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Remove,
                    labels: vec!["needs-tess".to_string()],
                    from_label: None,
                });
            }
            "add_approved" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Add,
                    labels: vec!["approved".to_string()],
                    from_label: None,
                });
            }
            "add_failed_remediation" => {
                operations.push(LabelOperation {
                    operation_type: LabelOperationType::Add,
                    labels: vec!["failed-remediation".to_string()],
                    from_label: None,
                });
            }
            "increment_iteration" => {
                let new_iteration = Self::increment_iteration(task_id);
                *iteration_update = Some(new_iteration);
            }
            _ => {
                warn!("Unknown transition action: {}", action);
            }
        }
    }

    /// Increment the iteration counter for a task
    fn increment_iteration(_task_id: &str) -> i32 {
        // This would integrate with the state manager
        // For now, return a placeholder
        1
    }

    /// Log a completed transition
    fn log_transition(
        pr_number: i32,
        task_id: &str,
        transition: &StateTransition,
        iteration_update: Option<i32>,
        context: Option<serde_json::Value>,
    ) {
        let mut log_fields = serde_json::json!({
            "pr_number": pr_number,
            "task_id": task_id,
            "from_state": format!("{:?}", transition.from),
            "to_state": format!("{:?}", transition.to),
            "trigger": transition.trigger,
            "actions": transition.actions,
        });

        if let Some(iteration) = iteration_update {
            log_fields["iteration"] = serde_json::json!(iteration);
        }

        if let Some(ctx) = context {
            log_fields["context"] = ctx;
        }

        info!("Workflow state transition completed: {}", log_fields);
    }

    /// Force a specific state (for recovery scenarios)
    ///
    /// # Errors
    /// Returns `OrchestratorError::GitHubError` if label operations fail
    pub async fn force_state(
        &mut self,
        pr_number: i32,
        task_id: &str,
        target_state: WorkflowState,
    ) -> Result<(), OrchestratorError> {
        info!(
            "Forcing PR #{} to state {:?} for task {}",
            pr_number, target_state, task_id
        );

        // Get current labels
        let current_labels = self.label_client.get_labels(pr_number).await?;
        let _current_state = self.label_schema.determine_workflow_state(&current_labels);

        // Calculate required operations to reach target state
        let operations = self.calculate_force_operations(&current_labels, &target_state);

        // Execute operations
        if !operations.is_empty() {
            self.label_client
                .update_labels_atomic(pr_number, &operations)
                .await?;
        }

        info!(
            "Successfully forced PR #{} to state {:?}",
            pr_number, target_state
        );
        Ok(())
    }

    /// Calculate operations needed to force a state
    #[allow(clippy::unused_self)]
    fn calculate_force_operations(
        &self,
        current_labels: &[String],
        target_state: &WorkflowState,
    ) -> Vec<LabelOperation> {
        let mut operations = Vec::new();

        // Remove all status labels first
        let status_labels = [
            "needs-fixes",
            "fixing-in-progress",
            "needs-cleo",
            "needs-tess",
            "approved",
            "failed-remediation",
        ];

        let labels_to_remove: Vec<String> = current_labels
            .iter()
            .filter(|label| status_labels.contains(&label.as_str()))
            .cloned()
            .collect();

        if !labels_to_remove.is_empty() {
            operations.push(LabelOperation {
                operation_type: LabelOperationType::Remove,
                labels: labels_to_remove,
                from_label: None,
            });
        }

        // Add the target state label
        let target_label = match target_state {
            WorkflowState::NeedsFixes => "needs-fixes",
            WorkflowState::FixingInProgress => "fixing-in-progress",
            WorkflowState::NeedsCleo => "needs-cleo",
            WorkflowState::NeedsTess => "needs-tess",
            WorkflowState::Approved => "approved",
            WorkflowState::Failed => "failed-remediation",
            WorkflowState::Initial | WorkflowState::ManualOverride => return operations,
        };

        operations.push(LabelOperation {
            operation_type: LabelOperationType::Add,
            labels: vec![target_label.to_string()],
            from_label: None,
        });

        operations
    }
}
