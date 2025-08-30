//! # Monitoring and Observability Module
//!
//! This module provides comprehensive monitoring and observability for the Agent Remediation Loop,
//! including Prometheus metrics, structured logging, distributed tracing, and health checks.

pub mod metrics;
pub mod logging;
pub mod tracing;
pub mod health;
pub mod alerts;

use crate::remediation::RemediationStateManager;
use crate::tasks::escalation::EscalationManager;
use crate::tasks::label::client::GitHubLabelClient;
use crate::tasks::cancel::CancellationCoordinator;
use metrics::MetricsCollector;
use logging::StructuredLogger;
use tracing::TraceManager;
use health::HealthChecker;
use alerts::AlertManager;
use std::sync::Arc;
use thiserror::Error;
use ::tracing::{debug, error, info, warn};

/// Central monitoring and observability manager
pub struct ObservabilityManager {
    metrics_collector: MetricsCollector,
    structured_logger: StructuredLogger,
    trace_manager: TraceManager,
    health_checker: HealthChecker,
    alert_manager: AlertManager,
}

/// Errors that can occur in monitoring operations
#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("Metrics collection error: {0}")]
    MetricsError(String),

    #[error("Logging error: {0}")]
    LoggingError(String),

    #[error("Tracing error: {0}")]
    TracingError(String),

    #[error("Health check error: {0}")]
    HealthError(String),

    #[error("Alerting error: {0}")]
    AlertError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl From<metrics::MetricsError> for ObservabilityError {
    fn from(err: metrics::MetricsError) -> Self {
        ObservabilityError::MetricsError(err.to_string())
    }
}

impl From<logging::LoggingError> for ObservabilityError {
    fn from(err: logging::LoggingError) -> Self {
        ObservabilityError::LoggingError(err.to_string())
    }
}

impl From<tracing::TracingError> for ObservabilityError {
    fn from(err: tracing::TracingError) -> Self {
        ObservabilityError::TracingError(err.to_string())
    }
}

impl From<health::HealthError> for ObservabilityError {
    fn from(err: health::HealthError) -> Self {
        ObservabilityError::HealthError(err.to_string())
    }
}

impl From<alerts::AlertError> for ObservabilityError {
    fn from(err: alerts::AlertError) -> Self {
        ObservabilityError::AlertError(err.to_string())
    }
}

/// Result type for observability operations
pub type ObservabilityResult<T> = Result<T, ObservabilityError>;

impl ObservabilityManager {
    /// Create a new observability manager
    pub fn new() -> ObservabilityResult<Self> {
        Ok(Self {
            metrics_collector: MetricsCollector::new()?,
            structured_logger: StructuredLogger::new()?,
            trace_manager: TraceManager::new()?,
            health_checker: HealthChecker::new()?,
            alert_manager: AlertManager::new()?,
        })
    }

    /// Initialize all monitoring components
    pub async fn initialize(&mut self) -> ObservabilityResult<()> {
        info!("Initializing observability system");

        // Initialize metrics collection
        self.metrics_collector.initialize().await?;

        // Initialize structured logging
        self.structured_logger.initialize().await?;

        // Initialize distributed tracing
        self.trace_manager.initialize().await?;

        // Initialize health checks
        self.health_checker.initialize().await?;

        // Initialize alerting
        self.alert_manager.initialize().await?;

        info!("Observability system initialized successfully");
        Ok(())
    }

    /// Get metrics collector reference
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    /// Get structured logger reference
    pub fn logger(&self) -> &StructuredLogger {
        &self.structured_logger
    }

    /// Get trace manager reference
    pub fn tracer(&self) -> &TraceManager {
        &self.trace_manager
    }

    /// Get health checker reference
    pub fn health(&self) -> &HealthChecker {
        &self.health_checker
    }

    /// Get alert manager reference
    pub fn alerts(&self) -> &AlertManager {
        &self.alert_manager
    }

    /// Record remediation cycle start
    pub async fn record_remediation_start(
        &self,
        task_id: &str,
        pr_number: i32,
        correlation_id: &str,
    ) -> ObservabilityResult<()> {
        // Record metrics
        self.metrics_collector.increment_remediation_cycles()?;

        // Start tracing span
        let span = self.trace_manager.start_remediation_span(task_id, pr_number, correlation_id)?;

        // Log event
        self.structured_logger.log_remediation_started(task_id, pr_number, correlation_id)?;

        Ok(())
    }

    /// Record remediation cycle completion
    pub async fn record_remediation_complete(
        &self,
        task_id: &str,
        pr_number: i32,
        correlation_id: &str,
        success: bool,
        duration_seconds: f64,
    ) -> ObservabilityResult<()> {
        // Record metrics
        if success {
            self.metrics_collector.increment_successful_remediations()?;
        } else {
            self.metrics_collector.increment_failed_remediations()?;
        }

        // Update duration histogram
        self.metrics_collector.record_remediation_duration(duration_seconds)?;

        // End tracing span
        self.trace_manager.end_remediation_span(task_id, success)?;

        // Log event
        self.structured_logger.log_remediation_completed(task_id, pr_number, success, duration_seconds)?;

        Ok(())
    }

    /// Record agent cancellation
    pub async fn record_agent_cancellation(
        &self,
        task_id: &str,
        pr_number: i32,
        reason: &str,
    ) -> ObservabilityResult<()> {
        // Record metrics
        self.metrics_collector.increment_agent_cancellations()?;

        // Log event
        self.structured_logger.log_agent_cancellation(task_id, pr_number, reason)?;

        Ok(())
    }

    /// Record escalation event
    pub async fn record_escalation(
        &self,
        task_id: &str,
        pr_number: i32,
        escalation_type: &str,
        severity: &str,
    ) -> ObservabilityResult<()> {
        // Record metrics
        self.metrics_collector.increment_escalations(escalation_type, severity)?;

        // Log event
        self.structured_logger.log_escalation(task_id, pr_number, escalation_type, severity)?;

        Ok(())
    }

    /// Perform comprehensive health check
    pub async fn perform_health_check(&self) -> ObservabilityResult<health::HealthStatus> {
        Ok(self.health_checker.perform_comprehensive_check().await?)
    }

    /// Export metrics for Prometheus scraping
    pub async fn export_metrics(&self) -> ObservabilityResult<String> {
        Ok(self.metrics_collector.export_prometheus_metrics().await?)
    }

    /// Get system health score (0.0 to 1.0)
    pub async fn get_health_score(&self) -> ObservabilityResult<f64> {
        let health_status = self.perform_health_check().await?;
        Ok(health_status.overall_score)
    }
}

/// Create a default observability manager with standard configuration
pub fn create_default_observability_manager() -> ObservabilityResult<ObservabilityManager> {
    ObservabilityManager::new()
}
