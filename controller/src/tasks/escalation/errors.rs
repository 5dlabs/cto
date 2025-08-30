//! # Error Classification and Critical Error Detection
//!
//! This module provides comprehensive error classification and detection
//! of critical errors that require escalation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// Classification of error criticality
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCriticality {
    /// Low impact, can be retried
    Low,
    /// Medium impact, requires attention but not immediate
    Medium,
    /// High impact, requires prompt attention
    High,
    /// Critical impact, requires immediate escalation
    Critical,
}

/// Comprehensive error classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CriticalError {
    // Authentication and Authorization
    AuthenticationFailed(String),
    AuthorizationDenied(String),
    TokenExpired(String),
    TokenInvalid(String),

    // API and Network
    GitHubRateLimitExceeded(u32), // remaining requests
    GitHubAPIUnavailable(String),
    NetworkTimeout(String),
    NetworkConnectionFailed(String),

    // Resource and Infrastructure
    KubernetesAPIFailed(String),
    ConfigMapAccessDenied(String),
    CodeRunCreationFailed(String),
    ResourceQuotaExceeded(String),

    // Data and State
    StateCorruption(String),
    DataValidationFailed(String),
    ConfigMapSizeExceeded(usize), // actual size
    SerializationFailed(String),

    // Application Logic
    InvalidWorkflowState(String),
    LabelOperationFailed(String),
    OverrideDetectionFailed(String),
    EscalationLogicFailed(String),

    // External Dependencies
    DatabaseConnectionFailed(String),
    CacheUnavailable(String),
    NotificationServiceFailed(String),

    // Security
    InputValidationFailed(String),
    XSSAttemptDetected(String),
    InjectionAttemptDetected(String),
    PrivilegeEscalationAttempt(String),

    // Generic
    UnknownError(String),
}

/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub task_id: String,
    pub pr_number: i32,
    pub operation: String,
    pub component: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Error classification result
#[derive(Debug, Clone)]
pub struct ErrorClassification {
    pub criticality: ErrorCriticality,
    pub requires_escalation: bool,
    pub retry_recommended: bool,
    pub escalation_message: String,
    pub suggested_actions: Vec<String>,
}

/// Error classifier for automated error analysis
pub struct ErrorClassifier {
    // Configuration for error classification rules
    rate_limit_threshold: u32,
    configmap_size_limit: usize,
    retryable_errors: Vec<String>,
}

impl Default for ErrorClassifier {
    fn default() -> Self {
        Self {
            rate_limit_threshold: 100, // GitHub's secondary rate limit
            configmap_size_limit: 800 * 1024, // 800KB
            retryable_errors: vec![
                "timeout".to_string(),
                "connection".to_string(),
                "temporary".to_string(),
                "rate limit".to_string(),
            ],
        }
    }
}

impl ErrorClassifier {
    /// Classify an error and determine its criticality
    pub fn classify_error(&self, error: &CriticalError, context: &ErrorContext) -> ErrorClassification {
        match error {
            // Critical errors - always require escalation
            CriticalError::AuthenticationFailed(_) |
            CriticalError::AuthorizationDenied(_) |
            CriticalError::StateCorruption(_) |
            CriticalError::XSSAttemptDetected(_) |
            CriticalError::InjectionAttemptDetected(_) |
            CriticalError::PrivilegeEscalationAttempt(_) => {
                ErrorClassification {
                    criticality: ErrorCriticality::Critical,
                    requires_escalation: true,
                    retry_recommended: false,
                    escalation_message: format!("Critical error in {}: {:?}", context.component, error),
                    suggested_actions: vec![
                        "Immediate investigation required".to_string(),
                        "Security review if applicable".to_string(),
                        "Manual intervention needed".to_string(),
                    ],
                }
            }

            // High priority errors
            CriticalError::GitHubRateLimitExceeded(remaining) if *remaining == 0 => {
                ErrorClassification {
                    criticality: ErrorCriticality::High,
                    requires_escalation: true,
                    retry_recommended: false,
                    escalation_message: format!("GitHub rate limit completely exhausted in {}", context.component),
                    suggested_actions: vec![
                        "Wait for rate limit reset".to_string(),
                        "Consider token rotation".to_string(),
                        "Review API usage patterns".to_string(),
                    ],
                }
            }

            CriticalError::ConfigMapSizeExceeded(size) => {
                ErrorClassification {
                    criticality: ErrorCriticality::High,
                    requires_escalation: true,
                    retry_recommended: false,
                    escalation_message: format!("ConfigMap size limit exceeded: {} bytes in {}", size, context.component),
                    suggested_actions: vec![
                        "Review state data cleanup".to_string(),
                        "Consider data archiving".to_string(),
                        "Optimize state storage".to_string(),
                    ],
                }
            }

            // Medium priority errors
            CriticalError::GitHubAPIUnavailable(_) |
            CriticalError::NetworkTimeout(_) |
            CriticalError::NetworkConnectionFailed(_) => {
                let should_retry = self.is_retryable_error(error);
                ErrorClassification {
                    criticality: ErrorCriticality::Medium,
                    requires_escalation: !should_retry,
                    retry_recommended: should_retry,
                    escalation_message: format!("Network/API issue in {}: {:?}", context.component, error),
                    suggested_actions: vec![
                        "Check network connectivity".to_string(),
                        "Verify GitHub API status".to_string(),
                        "Consider retry with backoff".to_string(),
                    ],
                }
            }

            // Low priority errors
            CriticalError::DataValidationFailed(_) |
            CriticalError::SerializationFailed(_) => {
                ErrorClassification {
                    criticality: ErrorCriticality::Low,
                    requires_escalation: false,
                    retry_recommended: true,
                    escalation_message: format!("Data processing issue in {}: {:?}", context.component, error),
                    suggested_actions: vec![
                        "Validate input data".to_string(),
                        "Check serialization format".to_string(),
                        "Retry operation".to_string(),
                    ],
                }
            }

            // Default case
            _ => {
                let should_retry = self.is_retryable_error(error);
                ErrorClassification {
                    criticality: ErrorCriticality::Medium,
                    requires_escalation: false,
                    retry_recommended: should_retry,
                    escalation_message: format!("Error in {}: {:?}", context.component, error),
                    suggested_actions: vec![
                        "Review error details".to_string(),
                        "Check system logs".to_string(),
                        if should_retry { "Retry operation".to_string() } else { "Manual investigation".to_string() },
                    ],
                }
            }
        }
    }

    /// Check if an error is retryable
    fn is_retryable_error(&self, error: &CriticalError) -> bool {
        let error_string = format!("{:?}", error).to_lowercase();
        self.retryable_errors.iter().any(|retryable| error_string.contains(retryable))
    }

    /// Create a critical error from a generic error
    pub fn classify_generic_error(&self, error: &anyhow::Error, _context: &ErrorContext) -> CriticalError {
        let error_message = error.to_string().to_lowercase();

        if error_message.contains("401") || error_message.contains("unauthorized") {
            CriticalError::AuthenticationFailed(error.to_string())
        } else if error_message.contains("403") || error_message.contains("forbidden") {
            CriticalError::AuthorizationDenied(error.to_string())
        } else if error_message.contains("rate limit") {
            CriticalError::GitHubRateLimitExceeded(0) // Assume exhausted
        } else if error_message.contains("timeout") {
            CriticalError::NetworkTimeout(error.to_string())
        } else if error_message.contains("connection") {
            CriticalError::NetworkConnectionFailed(error.to_string())
        } else if error_message.contains("configmap") {
            CriticalError::ConfigMapAccessDenied(error.to_string())
        } else if error_message.contains("coderun") {
            CriticalError::CodeRunCreationFailed(error.to_string())
        } else if error_message.contains("serializ") {
            CriticalError::SerializationFailed(error.to_string())
        } else if error_message.contains("validation") {
            CriticalError::DataValidationFailed(error.to_string())
        } else if error_message.contains("xss") || error_message.contains("script") {
            CriticalError::XSSAttemptDetected(error.to_string())
        } else if error_message.contains("inject") {
            CriticalError::InjectionAttemptDetected(error.to_string())
        } else {
            CriticalError::UnknownError(error.to_string())
        }
    }

    /// Check if an error should trigger immediate escalation
    pub fn should_escalate_immediately(&self, error: &CriticalError) -> bool {
        matches!(error,
            CriticalError::AuthenticationFailed(_) |
            CriticalError::AuthorizationDenied(_) |
            CriticalError::StateCorruption(_) |
            CriticalError::XSSAttemptDetected(_) |
            CriticalError::InjectionAttemptDetected(_) |
            CriticalError::PrivilegeEscalationAttempt(_) |
            CriticalError::ConfigMapSizeExceeded(_) |
            CriticalError::GitHubRateLimitExceeded(0)
        )
    }

    /// Get escalation priority for an error
    pub fn get_escalation_priority(&self, error: &CriticalError) -> &'static str {
        match error {
            CriticalError::AuthenticationFailed(_) |
            CriticalError::AuthorizationDenied(_) |
            CriticalError::StateCorruption(_) |
            CriticalError::XSSAttemptDetected(_) |
            CriticalError::InjectionAttemptDetected(_) |
            CriticalError::PrivilegeEscalationAttempt(_) => "P0 - Critical",

            CriticalError::ConfigMapSizeExceeded(_) |
            CriticalError::GitHubRateLimitExceeded(0) => "P1 - High",

            CriticalError::GitHubAPIUnavailable(_) |
            CriticalError::NetworkTimeout(_) |
            CriticalError::NetworkConnectionFailed(_) |
            CriticalError::KubernetesAPIFailed(_) => "P2 - Medium",

            _ => "P3 - Low",
        }
    }
}

/// Helper function to create error context
pub fn create_error_context(
    task_id: String,
    pr_number: i32,
    operation: String,
    component: String,
    metadata: HashMap<String, String>,
) -> ErrorContext {
    ErrorContext {
        task_id,
        pr_number,
        operation,
        component,
        timestamp: chrono::Utc::now(),
        metadata,
    }
}
