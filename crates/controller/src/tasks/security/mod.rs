//! # Security and RBAC Controls Module
//!
//! This module provides comprehensive security controls for agent workflows,
//! including GitHub token management, Kubernetes RBAC, input validation, rate limiting,
//! and comprehensive audit logging.

pub mod audit;
pub mod rate_limit;
pub mod rbac;
pub mod tokens;
pub mod validation;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Security errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Token validation error: {0}")]
    TokenValidationError(String),

    #[error("RBAC validation error: {0}")]
    RBACValidationError(String),

    #[error("Input validation error: {0}")]
    InputValidationError(String),

    #[error("Rate limiting error: {0}")]
    RateLimitError(String),

    #[error("Audit logging error: {0}")]
    AuditError(String),
}

impl From<tokens::TokenError> for SecurityError {
    fn from(err: tokens::TokenError) -> Self {
        SecurityError::TokenValidationError(err.to_string())
    }
}

impl From<rbac::RBACError> for SecurityError {
    fn from(err: rbac::RBACError) -> Self {
        SecurityError::RBACValidationError(err.to_string())
    }
}

impl From<validation::ValidationError> for SecurityError {
    fn from(err: validation::ValidationError) -> Self {
        SecurityError::InputValidationError(err.to_string())
    }
}

impl From<rate_limit::RateLimitError> for SecurityError {
    fn from(err: rate_limit::RateLimitError) -> Self {
        SecurityError::RateLimitError(err.to_string())
    }
}

impl From<audit::AuditError> for SecurityError {
    fn from(err: audit::AuditError) -> Self {
        SecurityError::AuditError(err.to_string())
    }
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security operation context
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub task_id: Option<String>,
    pub pr_number: Option<i32>,
    pub user: Option<String>,
    pub correlation_id: Option<String>,
    pub operation: String,
    pub component: String,
    pub timestamp: DateTime<Utc>,
}

/// Security operation result
#[derive(Debug, Clone)]
pub struct SecurityValidation {
    pub passed: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Main security manager coordinating all security controls
pub struct SecurityManager {
    token_manager: tokens::GitHubTokenManager,
    rbac_validator: rbac::RBACValidator,
    input_validator: validation::InputValidator,
    rate_limiter: rate_limit::RateLimiter,
    audit_logger: audit::AuditLogger,
    authorized_users: Vec<String>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> SecurityResult<Self> {
        Ok(Self {
            token_manager: tokens::GitHubTokenManager::new()?,
            rbac_validator: rbac::RBACValidator::new()?,
            input_validator: validation::InputValidator::new()?,
            rate_limiter: rate_limit::RateLimiter::new()?,
            audit_logger: audit::AuditLogger::new()?,
            authorized_users: vec![
                "5DLabs-Tess".to_string(),
                "5DLabs-Cleo".to_string(),
                "5DLabs-Rex".to_string(),
                "5DLabs-Blaze".to_string(),
                "5DLabs-Morgan".to_string(),
                "5DLabs-Cipher".to_string(),
            ],
        })
    }

    /// Perform comprehensive security validation for an operation
    pub async fn validate_operation(
        &mut self,
        context: &SecurityContext,
        input_data: Option<&str>,
    ) -> SecurityResult<SecurityValidation> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metadata = HashMap::new();

        // 1. Rate limiting check
        if let Err(e) = self.check_rate_limit(context).await {
            errors.push(format!("Rate limiting failed: {e}"));
            self.audit_security_event(context, "rate_limit_exceeded", false, Some(&e.to_string()))
                .await?;
        } else {
            metadata.insert("rate_limit_status".to_string(), "passed".to_string());
        }

        // 2. Input validation (if input provided)
        if let Some(input) = input_data {
            match self.validate_input(context, input).await {
                Ok(validation_result) => {
                    warnings.extend(validation_result.warnings);
                    let has_errors = !validation_result.errors.is_empty();
                    if has_errors {
                        errors.extend(validation_result.errors);
                    }
                    metadata.insert(
                        "input_validation".to_string(),
                        if has_errors { "failed" } else { "passed" }.to_string(),
                    );
                }
                Err(e) => {
                    errors.push(format!("Input validation error: {e}"));
                }
            }
        }

        // 3. GitHub token validation (if applicable)
        if let Err(e) = self.validate_github_token(context).await {
            errors.push(format!("GitHub token validation failed: {e}"));
            self.audit_security_event(
                context,
                "token_validation_failed",
                false,
                Some(&e.to_string()),
            )
            .await?;
        } else {
            metadata.insert("token_validation".to_string(), "passed".to_string());
        }

        // 4. RBAC validation (if applicable)
        if let Err(e) = self.validate_rbac(context).await {
            errors.push(format!("RBAC validation failed: {e}"));
            self.audit_security_event(context, "rbac_denied", false, Some(&e.to_string()))
                .await?;
        } else {
            metadata.insert("rbac_validation".to_string(), "passed".to_string());
        }

        // Log successful validation
        if errors.is_empty() {
            self.audit_security_event(context, "security_validation_passed", true, None)
                .await?;
        }

        let passed = errors.is_empty();

        Ok(SecurityValidation {
            passed,
            warnings,
            errors,
            metadata,
        })
    }

    /// Check rate limiting
    async fn check_rate_limit(&self, context: &SecurityContext) -> SecurityResult<()> {
        if let Some(task_id) = &context.task_id {
            self.rate_limiter.check_limit(task_id).await?;
        }
        Ok(())
    }

    /// Validate input data
    async fn validate_input(
        &self,
        context: &SecurityContext,
        input: &str,
    ) -> SecurityResult<validation::InputValidationResult> {
        // Check if user is authorized
        if let Some(user) = &context.user {
            if !self.authorized_users.contains(user) {
                return Err(SecurityError::AuthorizationError(format!(
                    "User {user} is not authorized to perform this operation"
                )));
            }
        }

        // Validate input content
        Ok(self.input_validator.validate_input(input).await?)
    }

    /// Validate GitHub token permissions
    async fn validate_github_token(&self, context: &SecurityContext) -> SecurityResult<()> {
        // For operations that require GitHub API access
        if matches!(
            context.operation.as_str(),
            "github_api_call" | "pr_comment" | "label_operation" | "status_check"
        ) {
            self.token_manager.validate_token_permissions().await?;
        }
        Ok(())
    }

    /// Validate RBAC permissions
    async fn validate_rbac(&self, context: &SecurityContext) -> SecurityResult<()> {
        // For operations that require Kubernetes resource access
        if matches!(
            context.operation.as_str(),
            "coderun_create" | "configmap_access" | "state_operation"
        ) {
            self.rbac_validator
                .validate_permissions(&context.operation)
                .await?;
        }
        Ok(())
    }

    /// Log security events
    async fn audit_security_event(
        &mut self,
        context: &SecurityContext,
        event_type: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> SecurityResult<()> {
        let audit_event = audit::AuditEvent {
            timestamp: context.timestamp,
            event_type: event_type.to_string(),
            actor: context
                .user
                .clone()
                .unwrap_or_else(|| "system".to_string()),
            action: context.operation.clone(),
            resource: context
                .task_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            success,
            severity: if success {
                audit::AuditSeverity::Info
            } else {
                audit::AuditSeverity::Warning
            },
            error_message: error_message.map(|s| s.to_string()),
            resource_id: context.task_id.clone(),
            task_id: context.task_id.clone(),
            pr_number: context.pr_number,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
        };

        self.audit_logger.log_event(audit_event).await?;
        Ok(())
    }

    /// Get security statistics
    pub async fn get_security_statistics(&self) -> SecurityResult<HashMap<String, u64>> {
        let mut stats = HashMap::new();

        // Rate limiting stats
        if let Ok(rate_stats) = self.rate_limiter.get_statistics().await {
            stats.extend(rate_stats);
        }

        // Audit stats
        if let Ok(audit_stats) = self.audit_logger.get_statistics().await {
            stats.extend(audit_stats);
        }

        // Token validation stats
        if let Ok(token_stats) = self.token_manager.get_statistics().await {
            stats.extend(token_stats);
        }

        Ok(stats)
    }

    /// Perform security health check
    pub async fn security_health_check(&self) -> SecurityResult<bool> {
        // Check that all security components are functioning
        let rate_limit_ok = self.rate_limiter.is_healthy().await;
        let audit_ok = self.audit_logger.is_healthy().await;
        let token_ok = self.token_manager.is_healthy().await;
        let rbac_ok = self.rbac_validator.is_healthy().await;

        Ok(rate_limit_ok && audit_ok && token_ok && rbac_ok)
    }

    /// Sanitize input for safe processing
    pub async fn sanitize_input(&self, input: &str) -> SecurityResult<String> {
        Ok(self.input_validator.sanitize_input(input).await?)
    }

    /// Check if user is authorized
    pub fn is_user_authorized(&self, username: &str) -> bool {
        self.authorized_users.contains(&username.to_string())
    }

    /// Add authorized user
    pub fn add_authorized_user(&mut self, username: String) {
        if !self.authorized_users.contains(&username) {
            self.authorized_users.push(username);
        }
    }

    /// Remove authorized user
    pub fn remove_authorized_user(&mut self, username: &str) {
        self.authorized_users.retain(|u| u != username);
    }

    /// Get authorized users list
    pub fn get_authorized_users(&self) -> &[String] {
        &self.authorized_users
    }
}

/// Security validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub sanitized_input: Option<String>,
}

/// Create a default security manager
pub fn create_default_security_manager() -> SecurityResult<SecurityManager> {
    SecurityManager::new()
}
