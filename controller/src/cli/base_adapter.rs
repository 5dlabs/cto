//! Base Adapter Implementation
//!
//! Provides shared functionality that all CLI adapters can leverage, including
//! logging, metrics, template rendering, and common utilities.

use crate::cli::trait_adapter::*;
use crate::cli::types::CLIType;
use handlebars::{Handlebars, Template};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn, debug, instrument, Span};
use uuid::Uuid;

/// Base adapter with shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// Metrics collector
    pub metrics: Arc<AdapterMetrics>,
    /// Template engine
    pub handlebars: Arc<RwLock<Handlebars<'static>>>,
    /// Telemetry tracer
    pub tracer: Arc<dyn Tracer>,
}

/// Configuration for the base adapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// Correlation ID for this adapter instance
    pub correlation_id: String,
    /// Log level for this adapter
    pub log_level: String,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable distributed tracing
    pub enable_tracing: bool,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            log_level: "info".to_string(),
            enable_metrics: true,
            enable_tracing: true,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(30),
        }
    }
}

/// Metrics collector for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Operation counters
    pub counters: RwLock<HashMap<String, u64>>,
    /// Operation durations
    pub histograms: RwLock<HashMap<String, Vec<Duration>>>,
    /// Error counts by type
    pub error_counts: RwLock<HashMap<String, u64>>,
    /// Health check results
    pub health_checks: RwLock<Vec<HealthCheckResult>>,
}

impl AdapterMetrics {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            error_counts: RwLock::new(HashMap::new()),
            health_checks: RwLock::new(Vec::new()),
        }
    }

    /// Record an operation with duration and success status
    #[instrument(skip(self))]
    pub async fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let metric_key = format!("{}_{}", cli_type, operation);

        // Update counter
        {
            let mut counters = self.counters.write().await;
            *counters.entry(metric_key.clone()).or_insert(0) += 1;
        }

        // Update histogram
        {
            let mut histograms = self.histograms.write().await;
            histograms
                .entry(metric_key.clone())
                .or_insert_with(Vec::new)
                .push(duration);
        }

        // Update error count if failed
        if !success {
            let error_key = format!("{}_error", metric_key);
            let mut error_counts = self.error_counts.write().await;
            *error_counts.entry(error_key).or_insert(0) += 1;
        }

        debug!(
            cli_type = ?cli_type,
            operation = operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter operation metrics"
        );
    }

    /// Record health check result
    pub async fn record_health_check(&self, result: HealthCheckResult) {
        let mut health_checks = self.health_checks.write().await;
        health_checks.push(result);

        // Keep only last 100 health checks
        if health_checks.len() > 100 {
            let excess = health_checks.len() - 100;
            health_checks.drain(0..excess);
        }
    }

    /// Get operation statistics
    pub async fn get_stats(&self, operation: &str) -> Option<OperationStats> {
        let counters = self.counters.read().await;
        let histograms = self.histograms.read().await;
        let error_counts = self.error_counts.read().await;

        let count = *counters.get(operation)?;
        let durations = histograms.get(operation)?;
        let errors = error_counts.get(&format!("{}_error", operation)).unwrap_or(&0);

        if durations.is_empty() {
            return None;
        }

        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / durations.len() as u32;
        let min_duration = *durations.iter().min()?;
        let max_duration = *durations.iter().max()?;

        Some(OperationStats {
            operation_name: operation.to_string(),
            total_count: count,
            error_count: *errors,
            success_rate: (count - errors) as f64 / count as f64,
            avg_duration,
            min_duration,
            max_duration,
        })
    }
}

impl Default for AdapterMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Timestamp of the check
    pub timestamp: std::time::SystemTime,
    /// CLI type checked
    pub cli_type: CLIType,
    /// Health status
    pub status: HealthStatus,
    /// Duration of the check
    pub duration: Duration,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStats {
    /// Operation name
    pub operation_name: String,
    /// Total operation count
    pub total_count: u64,
    /// Error count
    pub error_count: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average duration
    pub avg_duration: Duration,
    /// Minimum duration
    pub min_duration: Duration,
    /// Maximum duration
    pub max_duration: Duration,
}

/// Telemetry tracer interface
pub trait Tracer: Send + Sync + std::fmt::Debug {
    /// Start a new span
    fn start_span(&self, operation: &str) -> Box<dyn TracerSpan>;
    /// Record an event
    fn record_event(&self, event: &str, attributes: HashMap<String, String>);
}

/// Tracer span interface
pub trait TracerSpan: Send + Sync + std::fmt::Debug {
    /// Add attribute to span
    fn set_attribute(&mut self, key: &str, value: &str);
    /// Set span status
    fn set_status(&mut self, status: SpanStatus);
    /// End the span
    fn end(self: Box<Self>);
}

/// Span status
#[derive(Debug, Clone)]
pub enum SpanStatus {
    Ok,
    Error(String),
}

/// Default no-op tracer
#[derive(Debug)]
pub struct NoOpTracer;

impl Tracer for NoOpTracer {
    fn start_span(&self, _operation: &str) -> Box<dyn TracerSpan> {
        Box::new(NoOpSpan)
    }

    fn record_event(&self, _event: &str, _attributes: HashMap<String, String>) {}
}

#[derive(Debug)]
pub struct NoOpSpan;

impl TracerSpan for NoOpSpan {
    fn set_attribute(&mut self, _key: &str, _value: &str) {}
    fn set_status(&mut self, _status: SpanStatus) {}
    fn end(self: Box<Self>) {}
}

impl BaseAdapter {
    /// Create a new base adapter
    pub fn new(cli_type: CLIType) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register common helpers
        handlebars.register_helper("json", Box::new(json_helper));
        handlebars.register_helper("upper", Box::new(upper_helper));
        handlebars.register_helper("lower", Box::new(lower_helper));

        Self {
            cli_type,
            config: AdapterConfig::default(),
            metrics: Arc::new(AdapterMetrics::new()),
            handlebars: Arc::new(RwLock::new(handlebars)),
            tracer: Arc::new(NoOpTracer),
        }
    }

    /// Create base adapter with custom configuration
    pub fn with_config(cli_type: CLIType, config: AdapterConfig) -> Self {
        let mut base = Self::new(cli_type);
        base.config = config;
        base
    }

    /// Create base adapter with custom tracer
    pub fn with_tracer(mut self, tracer: Arc<dyn Tracer>) -> Self {
        self.tracer = tracer;
        self
    }

    /// Log an operation with structured data
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        info!(
            cli_type = ?self.cli_type,
            operation = operation,
            correlation_id = self.config.correlation_id,
            context = ?context,
            "CLI adapter operation"
        );
    }

    /// Log an error with context
    #[instrument(skip(self, context))]
    pub fn log_error(&self, operation: &str, error: &AdapterError, context: &HashMap<String, String>) {
        error!(
            cli_type = ?self.cli_type,
            operation = operation,
            error = %error,
            correlation_id = self.config.correlation_id,
            context = ?context,
            "CLI adapter error"
        );
    }

    /// Log a warning with context
    #[instrument(skip(self, context))]
    pub fn log_warning(&self, operation: &str, message: &str, context: &HashMap<String, String>) {
        warn!(
            cli_type = ?self.cli_type,
            operation = operation,
            message = message,
            correlation_id = self.config.correlation_id,
            context = ?context,
            "CLI adapter warning"
        );
    }

    /// Record metrics for an operation
    pub async fn record_metrics(&self, operation: &str, duration: Duration, success: bool) {
        if self.config.enable_metrics {
            self.metrics
                .record_operation(self.cli_type, operation, duration, success)
                .await;
        }
    }

    /// Validate base configuration
    pub fn validate_base_config(&self, config: &AgentConfig) -> Result<(), AdapterError> {
        if config.model.trim().is_empty() {
            return Err(AdapterError::ConfigGeneration(
                "Model name cannot be empty".to_string(),
            ));
        }

        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 {
                return Err(AdapterError::ConfigGeneration(
                    "Max tokens must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ConfigGeneration(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Render a template with the given context
    pub async fn render_template(
        &self,
        template: &str,
        context: &Value,
    ) -> Result<String, AdapterError> {
        let handlebars = self.handlebars.read().await;

        handlebars
            .render_template(template, context)
            .map_err(|e| AdapterError::TemplateRendering(format!("Template rendering failed: {}", e)))
    }

    /// Register a template with a name
    pub async fn register_template(
        &self,
        name: &str,
        template: &str,
    ) -> Result<(), AdapterError> {
        let mut handlebars = self.handlebars.write().await;

        handlebars
            .register_template_string(name, template)
            .map_err(|e| AdapterError::TemplateRendering(format!("Template registration failed: {}", e)))
    }

    /// Render a named template
    pub async fn render_named_template(
        &self,
        name: &str,
        context: &Value,
    ) -> Result<String, AdapterError> {
        let handlebars = self.handlebars.read().await;

        handlebars
            .render(name, context)
            .map_err(|e| AdapterError::TemplateRendering(format!("Named template rendering failed: {}", e)))
    }

    /// Time an operation and record metrics
    pub async fn time_operation<F, T>(&self, operation: &str, f: F) -> Result<T, AdapterError>
    where
        F: std::future::Future<Output = Result<T, AdapterError>>,
    {
        let start = Instant::now();
        let span = if self.config.enable_tracing {
            Some(self.tracer.start_span(operation))
        } else {
            None
        };

        let result = f.await;
        let duration = start.elapsed();

        // Record metrics
        self.record_metrics(operation, duration, result.is_ok()).await;

        // Update span
        if let Some(mut span) = span {
            match &result {
                Ok(_) => span.set_status(SpanStatus::Ok),
                Err(err) => span.set_status(SpanStatus::Error(err.to_string())),
            }
            span.end();
        }

        result
    }

    /// Create operation context
    pub fn create_context(&self, operation: &str) -> HashMap<String, String> {
        let mut context = HashMap::new();
        context.insert("cli_type".to_string(), self.cli_type.to_string());
        context.insert("operation".to_string(), operation.to_string());
        context.insert("correlation_id".to_string(), self.config.correlation_id.clone());
        context.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        context
    }
}

// Handlebars helpers

fn json_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| {
        handlebars::RenderErrorReason::Other("json helper requires a parameter".to_string())
    })?;

    let json_string = serde_json::to_string(param.value())
        .map_err(|e| handlebars::RenderErrorReason::Other(format!("JSON serialization failed: {}", e)))?;

    out.write(&json_string)?;
    Ok(())
}

fn upper_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| {
        handlebars::RenderErrorReason::Other("upper helper requires a parameter".to_string())
    })?;

    let value = param.value().as_str().ok_or_else(|| {
        handlebars::RenderErrorReason::Other("upper helper requires a string parameter".to_string())
    })?;

    out.write(&value.to_uppercase())?;
    Ok(())
}

fn lower_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| {
        handlebars::RenderErrorReason::Other("lower helper requires a parameter".to_string())
    })?;

    let value = param.value().as_str().ok_or_else(|| {
        handlebars::RenderErrorReason::Other("lower helper requires a string parameter".to_string())
    })?;

    out.write(&value.to_lowercase())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let adapter = BaseAdapter::new(CLIType::Claude);
        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(adapter.config.enable_metrics);
        assert!(adapter.config.enable_tracing);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let adapter = BaseAdapter::new(CLIType::Claude);
        let template = "Hello {{name}}!";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).await.unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[tokio::test]
    async fn test_named_template() {
        let adapter = BaseAdapter::new(CLIType::Claude);
        let template = "{{upper name}} says {{lower message}}";
        let context = json!({"name": "Alice", "message": "HELLO"});

        adapter.register_template("greeting", template).await.unwrap();
        let result = adapter.render_named_template("greeting", &context).await.unwrap();
        assert_eq!(result, "ALICE says hello");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let adapter = BaseAdapter::new(CLIType::Claude);

        // Valid config
        let valid_config = AgentConfig {
            model: "claude-3-5-sonnet".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            ..Default::default()
        };
        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid model
        let invalid_model_config = AgentConfig {
            model: "".to_string(),
            ..Default::default()
        };
        assert!(adapter.validate_base_config(&invalid_model_config).is_err());

        // Invalid temperature
        let invalid_temp_config = AgentConfig {
            temperature: Some(3.0),
            ..Default::default()
        };
        assert!(adapter.validate_base_config(&invalid_temp_config).is_err());
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let adapter = BaseAdapter::new(CLIType::Claude);

        // Record some operations
        adapter.record_metrics("validate_model", Duration::from_millis(100), true).await;
        adapter.record_metrics("validate_model", Duration::from_millis(150), false).await;
        adapter.record_metrics("generate_config", Duration::from_millis(200), true).await;

        // Check stats
        let stats = adapter.metrics.get_stats("claude_validate_model").await.unwrap();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.success_rate, 0.5);
    }

    #[test]
    fn test_operation_context() {
        let adapter = BaseAdapter::new(CLIType::Claude);
        let context = adapter.create_context("test_operation");

        assert_eq!(context.get("cli_type").unwrap(), "claude");
        assert_eq!(context.get("operation").unwrap(), "test_operation");
        assert!(context.contains_key("correlation_id"));
        assert!(context.contains_key("timestamp"));
    }

    #[tokio::test]
    async fn test_time_operation() {
        let adapter = BaseAdapter::new(CLIType::Claude);

        let result = adapter
            .time_operation("test_op", async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<String, AdapterError>("success".to_string())
            })
            .await
            .unwrap();

        assert_eq!(result, "success");

        // Check that metrics were recorded
        let stats = adapter.metrics.get_stats("claude_test_op").await.unwrap();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.error_count, 0);
        assert!(stats.avg_duration >= Duration::from_millis(10));
    }
}