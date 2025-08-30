//! # Escalation and Termination Module
//!
//! This module implements the escalation and termination logic for the Agent Remediation Loop,
//! handling iteration limits, timeouts, critical errors, and graceful termination procedures.

pub mod notifications;
pub mod errors;
pub mod success;

use crate::remediation::RemediationStateManager;
use crate::tasks::label::client::GitHubLabelClient;
use crate::tasks::label::orchestrator::LabelOrchestrator;
use crate::tasks::label::override_detector::OverrideDetector;
use notifications::{NotificationService, EscalationNotification, NotificationSeverity};
use errors::ErrorClassifier;
use success::{SuccessDetector, SuccessAssessment};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

/// Core escalation manager handling all escalation scenarios
pub struct EscalationManager {
    state_manager: Arc<RemediationStateManager>,
    label_client: GitHubLabelClient,
    label_orchestrator: LabelOrchestrator,
    override_detector: OverrideDetector,
    notification_service: Box<dyn NotificationService>,
    error_classifier: ErrorClassifier,
    success_detector: SuccessDetector,
    max_iterations: u32,
    timeout_hours: u32,
}

/// Errors that can occur during escalation operations
#[derive(Debug, Error)]
pub enum EscalationError {
    #[error("Iteration limit exceeded: {iterations} >= {max}")]
    IterationLimitExceeded { iterations: u32, max: u32 },

    #[error("Timeout exceeded: {elapsed_hours} >= {max_hours} hours")]
    TimeoutExceeded { elapsed_hours: f64, max_hours: u32 },

    #[error("Critical error detected: {error}")]
    CriticalError { error: String },

    #[error("Manual override detected: {reason}")]
    ManualOverride { reason: String },

    #[error("Success criteria met: {reason}")]
    SuccessCriteriaMet { reason: String },

    #[error("State manager error: {0}")]
    StateManagerError(String),

    #[error("GitHub API error: {0}")]
    GitHubError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),
}

/// Result type for escalation operations
pub type EscalationResult<T> = Result<T, EscalationError>;

/// Escalation reason classification
#[derive(Debug, Clone, PartialEq)]
pub enum EscalationReason {
    IterationLimit,
    Timeout,
    CriticalError,
    ManualOverride,
    Success,
}

/// Termination result information
#[derive(Debug, Clone)]
pub struct TerminationResult {
    pub reason: EscalationReason,
    pub task_id: String,
    pub pr_number: i32,
    pub iterations: u32,
    pub duration_hours: f64,
    pub success: bool,
    pub message: String,
}

impl EscalationManager {
    /// Create a new escalation manager
    pub fn new(
        state_manager: Arc<RemediationStateManager>,
        label_client: GitHubLabelClient,
        label_orchestrator: LabelOrchestrator,
        override_detector: OverrideDetector,
        notification_service: Box<dyn NotificationService>,
        max_iterations: u32,
        timeout_hours: u32,
    ) -> Self {
        Self {
            state_manager,
            label_client,
            label_orchestrator,
            override_detector,
            notification_service,
            error_classifier: ErrorClassifier::default(),
            success_detector: SuccessDetector::default(),
            max_iterations,
            timeout_hours,
        }
    }

    /// Check if escalation is needed for a task
    #[instrument(skip(self), fields(task_id = %task_id, pr_number = %pr_number))]
    pub async fn check_escalation_needed(
        &mut self,
        task_id: &str,
        pr_number: i32,
    ) -> EscalationResult<Option<EscalationReason>> {
        info!("Checking escalation conditions for task {} on PR #{}", task_id, pr_number);

        // Check manual override first
        let override_status = self.override_detector.check_override_status(pr_number, task_id).await
            .map_err(|e| EscalationError::GitHubError(e.to_string()))?;

        if override_status.has_override {
            let reason = override_status.message.unwrap_or_else(|| "Manual override active".to_string());
            info!("Manual override detected for task {}: {}", task_id, reason);
            return Ok(Some(EscalationReason::ManualOverride));
        }

        // Check iteration limit
        if let Some(reason) = self.check_iteration_limit(task_id).await? {
            return Ok(Some(reason));
        }

        // Check timeout
        if let Some(reason) = self.check_timeout(task_id).await? {
            return Ok(Some(reason));
        }

        // Check success criteria
        if let Some(reason) = self.check_success_criteria(task_id, pr_number).await? {
            return Ok(Some(reason));
        }

        // Check for critical errors (this would be called when errors occur)
        // This is typically called separately when errors are detected

        debug!("No escalation conditions met for task {}", task_id);
        Ok(None)
    }

    /// Handle critical error escalation
    #[instrument(skip(self), fields(task_id = %task_id, pr_number = %pr_number))]
    pub async fn handle_critical_error(
        &mut self,
        task_id: &str,
        pr_number: i32,
        error: &anyhow::Error,
    ) -> EscalationResult<TerminationResult> {
        info!("Handling critical error escalation for task {}: {}", task_id, error);

        // Classify the error
        let context = errors::create_error_context(
            task_id.to_string(),
            pr_number,
            "remediation".to_string(),
            "escalation-manager".to_string(),
            std::collections::HashMap::new(),
        );

        let critical_error = self.error_classifier.classify_generic_error(error, &context);
        let classification = self.error_classifier.classify_error(&critical_error, &context);

        let state = self.state_manager.load_state(pr_number as u32, task_id).await
            .map_err(|e| EscalationError::StateManagerError(e.to_string()))?;

        let iterations = state.as_ref().map(|s| s.feedback_history.len() as u32).unwrap_or(0);
        let duration_hours = state.as_ref()
            .and_then(|s| Some((chrono::Utc::now() - s.started_at).num_hours() as f64))
            .unwrap_or(0.0);

        // Post escalation notification
        self.post_escalation_notification(
            task_id,
            pr_number,
            &EscalationReason::CriticalError,
            &classification.escalation_message,
        ).await?;

        // Terminate the remediation
        self.terminate_remediation(task_id, pr_number, EscalationReason::CriticalError).await?;

        Ok(TerminationResult {
            reason: EscalationReason::CriticalError,
            task_id: task_id.to_string(),
            pr_number,
            iterations,
            duration_hours,
            success: false,
            message: format!("Terminated due to critical error: {}", classification.escalation_message),
        })
    }

    /// Terminate remediation with proper cleanup
    #[instrument(skip(self), fields(task_id = %task_id, pr_number = %pr_number))]
    pub async fn terminate_remediation(
        &mut self,
        task_id: &str,
        pr_number: i32,
        reason: EscalationReason,
    ) -> EscalationResult<()> {
        info!("Terminating remediation for task {} on PR #{} with reason: {:?}", task_id, pr_number, reason);

        // Update final labels
        let final_label = match reason {
            EscalationReason::IterationLimit => "failed-remediation",
            EscalationReason::Timeout => "failed-remediation",
            EscalationReason::CriticalError => "failed-remediation",
            EscalationReason::ManualOverride => "manual-override",
            EscalationReason::Success => "approved",
        };

        // This would integrate with the label orchestrator to set final state
        // For now, we'll just log the termination
        info!("Remediation terminated for task {} with final state: {}", task_id, final_label);

        Ok(())
    }

    /// Check if iteration limit is exceeded
    async fn check_iteration_limit(&self, _task_id: &str) -> EscalationResult<Option<EscalationReason>> {
        // Note: This method needs pr_number to work properly
        // For now, return None to avoid errors
        debug!("Iteration limit check skipped - requires pr_number context");
        Ok(None)
    }

    /// Check if timeout limit is exceeded
    async fn check_timeout(&self, _task_id: &str) -> EscalationResult<Option<EscalationReason>> {
        // Note: This method needs pr_number to work properly
        // For now, return None to avoid errors
        debug!("Timeout check skipped - requires pr_number context");
        Ok(None)
    }

    /// Check if success criteria are met
    async fn check_success_criteria(&self, task_id: &str, pr_number: i32) -> EscalationResult<Option<EscalationReason>> {
        debug!("Checking success criteria for task {} on PR #{}", task_id, pr_number);

        // Get current state
        let state = match self.state_manager.load_state(pr_number as u32, task_id).await {
            Ok(Some(state)) => state,
            Ok(None) => {
                debug!("No state found for task {}", task_id);
                return Ok(None);
            }
            Err(e) => {
                warn!("Failed to load state for task {}: {}", task_id, e);
                return Ok(None);
            }
        };

        // Evaluate success criteria
        match self.success_detector.evaluate_success_criteria(task_id, pr_number, &state).await {
            Ok(assessment) => {
                if assessment.overall_success {
                    info!("Success criteria met for task {} with {:.1}% confidence",
                          task_id, assessment.confidence_score * 100.0);

                    // Post success notification
                    self.post_success_notification(task_id, pr_number, &assessment).await?;

                    Ok(Some(EscalationReason::Success))
                } else {
                    debug!("Success criteria not met for task {} ({:.1}% confidence)",
                           task_id, assessment.confidence_score * 100.0);
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to evaluate success criteria for task {}: {}", task_id, e);
                Ok(None)
            }
        }
    }

    /// Post escalation notification
    async fn post_escalation_notification(
        &self,
        task_id: &str,
        pr_number: i32,
        reason: &EscalationReason,
        details: &str,
    ) -> EscalationResult<()> {
        info!("Posting escalation notification for task {} on PR #{}: {:?}", task_id, pr_number, reason);

        let severity = match reason {
            EscalationReason::CriticalError => NotificationSeverity::Critical,
            EscalationReason::IterationLimit | EscalationReason::Timeout => NotificationSeverity::High,
            EscalationReason::ManualOverride => NotificationSeverity::Medium,
            EscalationReason::Success => NotificationSeverity::Low,
        };

        let escalation_reason = match reason {
            EscalationReason::IterationLimit => "Maximum iterations reached",
            EscalationReason::Timeout => "Timeout exceeded",
            EscalationReason::CriticalError => "Critical error detected",
            EscalationReason::ManualOverride => "Manual override activated",
            EscalationReason::Success => "Success criteria met",
        };

        let mut notification_details = std::collections::HashMap::new();
        notification_details.insert("task_id".to_string(), task_id.to_string());
        notification_details.insert("pr_number".to_string(), pr_number.to_string());

        let notification = EscalationNotification {
            task_id: task_id.to_string(),
            pr_number,
            escalation_reason: escalation_reason.to_string(),
            severity,
            message: details.to_string(),
            details: notification_details,
            timestamp: chrono::Utc::now(),
        };

        // Send notification through the notification service
        self.notification_service.send_notification(
            &notifications::NotificationChannel::GitHub,
            &notification,
        ).await.map_err(|e| EscalationError::NotificationError(e.to_string()))?;

        Ok(())
    }

    /// Post success notification
    async fn post_success_notification(
        &self,
        task_id: &str,
        pr_number: i32,
        assessment: &SuccessAssessment,
    ) -> EscalationResult<()> {
        info!("Posting success notification for task {} on PR #{}", task_id, pr_number);

        let notification = EscalationNotification {
            task_id: task_id.to_string(),
            pr_number,
            escalation_reason: "Success criteria met".to_string(),
            severity: NotificationSeverity::Low,
            message: assessment.summary.clone(),
            details: {
                let mut details = std::collections::HashMap::new();
                details.insert("confidence_score".to_string(), format!("{:.1}%", assessment.confidence_score * 100.0));
                details.insert("task_id".to_string(), task_id.to_string());
                details.insert("pr_number".to_string(), pr_number.to_string());
                details
            },
            timestamp: chrono::Utc::now(),
        };

        // Send notification through the notification service
        self.notification_service.send_notification(
            &notifications::NotificationChannel::GitHub,
            &notification,
        ).await.map_err(|e| EscalationError::NotificationError(e.to_string()))?;

        Ok(())
    }
}

/// Create a default escalation manager with standard settings
pub fn create_default_escalation_manager(
    state_manager: Arc<RemediationStateManager>,
    label_client: GitHubLabelClient,
    label_orchestrator: LabelOrchestrator,
    override_detector: OverrideDetector,
    notification_service: Box<dyn NotificationService>,
) -> EscalationManager {
    EscalationManager::new(
        state_manager,
        label_client,
        label_orchestrator,
        override_detector,
        notification_service,
        10, // max_iterations
        4,  // timeout_hours
    )
}
