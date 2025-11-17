//! # Override Detector
//!
//! This module handles detection and processing of human override labels
//! that can disable or modify automated workflow behavior.

use crate::tasks::label::client::GitHubLabelClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Override detector for managing human intervention labels
pub struct OverrideDetector {
    label_client: GitHubLabelClient,
    notification_service: Option<Box<dyn NotificationService>>,
}

#[derive(Debug, Error)]
pub enum OverrideError {
    #[error("Override processing failed: {0}")]
    ProcessingError(String),

    #[error("Bypass request failed: {0}")]
    BypassError(String),

    #[error("Notification failed: {0}")]
    NotificationError(String),
}

/// Status of override detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideStatus {
    /// Whether an override is currently active
    pub has_override: bool,
    /// Type of override detected
    pub override_type: Option<String>,
    /// Human-readable message about the override
    pub message: Option<String>,
    /// List of all overrides found
    pub overrides: Option<Vec<Override>>,
}

/// Information about a specific override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Override {
    /// Type of override (skip-automation, manual-review-required, etc.)
    pub override_type: String,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: String,
    /// Action to take
    pub action: String,
}

/// Bypass request for emergency override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassRequest {
    /// Unique request ID
    pub id: String,
    /// PR number
    pub pr_number: i32,
    /// Task ID
    pub task_id: String,
    /// Reason for bypass
    pub reason: String,
    /// Requester identifier
    pub requester: String,
    /// Current status
    pub status: BypassStatus,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// List of approvers
    pub approvers: Vec<String>,
}

/// Status of a bypass request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BypassStatus {
    /// Request is pending approval
    Pending,
    /// Request has been approved
    Approved,
    /// Request has been denied
    Denied,
    /// Request has expired
    Expired,
}

/// Notification service trait for override alerts
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    /// Send an override alert notification
    async fn send_override_alert(&self, alert: OverrideAlert) -> Result<(), OverrideError>;

    /// Send a bypass request notification
    async fn send_bypass_request(&self, request: BypassRequest) -> Result<(), OverrideError>;
}

/// Override alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideAlert {
    /// PR number where override was detected
    pub pr_number: i32,
    /// Task ID
    pub task_id: String,
    /// Type of override
    pub override_type: String,
    /// Alert message
    pub message: String,
    /// Severity level
    pub severity: String,
}

impl OverrideDetector {
    /// Create a new override detector
    #[must_use]
    pub fn new(label_client: GitHubLabelClient) -> Self {
        Self {
            label_client,
            notification_service: None,
        }
    }

    /// Set the notification service
    #[must_use]
    pub fn with_notification_service<T: NotificationService + 'static>(
        mut self,
        service: T,
    ) -> Self {
        self.notification_service = Some(Box::new(service));
        self
    }

    /// Check for override labels on a PR
    ///
    /// # Errors
    /// Returns `OverrideError::ProcessingError` if the labels cannot be retrieved
    pub async fn check_override_status(
        &mut self,
        pr_number: i32,
        task_id: &str,
    ) -> Result<OverrideStatus, OverrideError> {
        debug!(
            "Checking for override labels on PR #{} for task {}",
            pr_number, task_id
        );

        let labels =
            self.label_client.get_labels(pr_number).await.map_err(|e| {
                OverrideError::ProcessingError(format!("Failed to get labels: {e}"))
            })?;

        let overrides = self.detect_overrides(&labels);

        if overrides.is_empty() {
            debug!("No override labels detected on PR #{}", pr_number);
            Ok(OverrideStatus {
                has_override: false,
                override_type: None,
                message: None,
                overrides: None,
            })
        } else {
            let primary_override = &overrides[0];
            info!(
                "Override detected on PR #{}: {} ({})",
                pr_number, primary_override.override_type, primary_override.severity
            );

            // Send notification if service is configured
            if let Some(service) = &self.notification_service {
                let alert = OverrideAlert {
                    pr_number,
                    task_id: task_id.to_string(),
                    override_type: primary_override.override_type.clone(),
                    message: primary_override.message.clone(),
                    severity: primary_override.severity.clone(),
                };

                if let Err(e) = service.send_override_alert(alert).await {
                    warn!("Failed to send override notification: {}", e);
                }
            }

            Ok(OverrideStatus {
                has_override: true,
                override_type: Some(primary_override.override_type.clone()),
                message: Some(primary_override.message.clone()),
                overrides: Some(overrides),
            })
        }
    }

    /// Detect all override labels in a set of labels
    #[allow(clippy::unused_self)]
    fn detect_overrides(&self, labels: &[String]) -> Vec<Override> {
        let mut overrides = Vec::new();

        // Define override label mappings
        let override_definitions: HashMap<&str, Override> = [
            (
                "skip-automation",
                Override {
                    override_type: "skip-automation".to_string(),
                    message: "All automated workflows disabled by human override".to_string(),
                    severity: "high".to_string(),
                    action: "halt_all_automation".to_string(),
                },
            ),
            (
                "manual-review-required",
                Override {
                    override_type: "manual-review-required".to_string(),
                    message: "Manual review required before automation continues".to_string(),
                    severity: "medium".to_string(),
                    action: "pause_until_review".to_string(),
                },
            ),
            (
                "pause-remediation",
                Override {
                    override_type: "pause-remediation".to_string(),
                    message: "Remediation temporarily paused".to_string(),
                    severity: "low".to_string(),
                    action: "pause_remediation_only".to_string(),
                },
            ),
        ]
        .into();

        for label in labels {
            if let Some(override_info) = override_definitions.get(label.as_str()) {
                overrides.push(override_info.clone());
            }
        }

        overrides
    }

    /// Create a bypass request for emergency situations
    ///
    /// # Errors
    /// Returns `OverrideError::ProcessingError` if the bypass request cannot be stored
    pub async fn create_bypass_request(
        &mut self,
        pr_number: i32,
        task_id: &str,
        reason: &str,
        requester: &str,
    ) -> Result<BypassRequest, OverrideError> {
        info!(
            "Creating bypass request for PR #{} by {}",
            pr_number, requester
        );

        let request = BypassRequest {
            id: format!("bypass-{}-{}", pr_number, chrono::Utc::now().timestamp()),
            pr_number,
            task_id: task_id.to_string(),
            reason: reason.to_string(),
            requester: requester.to_string(),
            status: BypassStatus::Pending,
            created_at: chrono::Utc::now(),
            approvers: Vec::new(),
        };

        // Store the request (implementation would depend on storage mechanism)
        Self::store_bypass_request(&request);

        // Send notification if service is configured
        if let Some(service) = &self.notification_service {
            if let Err(e) = service.send_bypass_request(request.clone()).await {
                warn!("Failed to send bypass request notification: {}", e);
            }
        }

        Ok(request)
    }

    /// Approve a bypass request
    ///
    /// # Errors
    /// Returns `OverrideError::BypassError` if the request is not found or cannot be approved
    pub fn approve_bypass_request(
        &mut self,
        request_id: &str,
        approver: &str,
    ) -> Result<(), OverrideError> {
        info!("Approving bypass request {} by {}", request_id, approver);

        let mut request = Self::get_bypass_request(request_id).ok_or_else(|| {
            OverrideError::BypassError(format!("Bypass request {request_id} not found"))
        })?;

        if !matches!(request.status, BypassStatus::Pending) {
            return Err(OverrideError::BypassError(format!(
                "Cannot approve request with status {:?}",
                request.status
            )));
        }

        request.approvers.push(approver.to_string());
        request.status = BypassStatus::Approved;

        Self::update_bypass_request(&request);

        info!("Bypass request {} approved by {}", request_id, approver);
        Ok(())
    }

    /// Deny a bypass request
    ///
    /// # Errors
    /// Returns `OverrideError::BypassError` if the request is not found or cannot be denied
    pub fn deny_bypass_request(
        &mut self,
        request_id: &str,
        approver: &str,
    ) -> Result<(), OverrideError> {
        info!("Denying bypass request {} by {}", request_id, approver);

        let mut request = Self::get_bypass_request(request_id).ok_or_else(|| {
            OverrideError::BypassError(format!("Bypass request {request_id} not found"))
        })?;

        if !matches!(request.status, BypassStatus::Pending) {
            return Err(OverrideError::BypassError(format!(
                "Cannot deny request with status {:?}",
                request.status
            )));
        }

        request.approvers.push(approver.to_string());
        request.status = BypassStatus::Denied;

        Self::update_bypass_request(&request);

        info!("Bypass request {} denied by {}", request_id, approver);
        Ok(())
    }

    /// Get the status of a bypass request
    ///
    /// # Errors
    /// Returns `OverrideError::BypassError` if the request status cannot be retrieved
    pub fn get_bypass_status(
        &mut self,
        request_id: &str,
    ) -> Result<Option<BypassStatus>, OverrideError> {
        let request = Self::get_bypass_request(request_id);
        Ok(request.map(|r| r.status))
    }

    /// Store a bypass request (placeholder implementation)
    fn store_bypass_request(_request: &BypassRequest) {
        // TODO: Implement storage mechanism (database, config map, etc.)
        debug!("Storing bypass request (placeholder implementation)");
    }

    /// Retrieve a bypass request (placeholder implementation)
    fn get_bypass_request(_request_id: &str) -> Option<BypassRequest> {
        // TODO: Implement retrieval mechanism
        debug!("Retrieving bypass request (placeholder implementation)");
        None
    }

    /// Update a bypass request (placeholder implementation)
    fn update_bypass_request(_request: &BypassRequest) {
        // TODO: Implement update mechanism
        debug!("Updating bypass request (placeholder implementation)");
    }
}

/// Default notification service that logs events
pub struct LoggingNotificationService;

#[async_trait::async_trait]
impl NotificationService for LoggingNotificationService {
    async fn send_override_alert(&self, alert: OverrideAlert) -> Result<(), OverrideError> {
        info!(
            "OVERRIDE ALERT: PR #{} - {} ({}) - {}",
            alert.pr_number, alert.override_type, alert.severity, alert.message
        );
        Ok(())
    }

    async fn send_bypass_request(&self, request: BypassRequest) -> Result<(), OverrideError> {
        info!(
            "BYPASS REQUEST: {} for PR #{} by {} - {}",
            request.id, request.pr_number, request.requester, request.reason
        );
        Ok(())
    }
}
