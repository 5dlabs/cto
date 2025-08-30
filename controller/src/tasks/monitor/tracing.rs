//! # Distributed Tracing System
//!
//! This module provides distributed tracing capabilities using OpenTelemetry,
//! enabling end-to-end request tracing across the Agent Remediation Loop.

use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Tracing errors
#[derive(Debug, Error)]
pub enum TracingError {
    #[error("Tracing initialization error: {0}")]
    InitializationError(String),

    #[error("Span creation error: {0}")]
    SpanError(String),

    #[error("Trace export error: {0}")]
    ExportError(String),
}

/// Result type for tracing operations
pub type TracingResult<T> = Result<T, TracingError>;

/// Active trace span
#[derive(Debug, Clone)]
pub struct ActiveSpan {
    span_id: String,
    trace_id: String,
    operation: String,
    start_time: std::time::Instant,
    attributes: HashMap<String, String>,
}

impl ActiveSpan {
    /// Create a new active span
    pub fn new(span_id: String, trace_id: String, operation: String) -> Self {
        Self {
            span_id,
            trace_id,
            operation,
            start_time: std::time::Instant::now(),
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the span
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    /// Get span ID
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// Get trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

/// Distributed trace manager
pub struct TraceManager {
    service_name: String,
    service_version: String,
    active_spans: std::sync::Mutex<HashMap<String, ActiveSpan>>,
}

impl TraceManager {
    /// Create a new trace manager
    pub fn new() -> TracingResult<Self> {
        Ok(Self {
            service_name: "agent-remediation-loop".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            active_spans: std::sync::Mutex::new(HashMap::new()),
        })
    }

    /// Initialize OpenTelemetry tracing
    pub async fn initialize(&self) -> TracingResult<()> {
        info!("Initializing OpenTelemetry tracing for service: {}", self.service_name);

        // In a real implementation, this would:
        // 1. Initialize OpenTelemetry SDK
        // 2. Configure Jaeger exporter
        // 3. Set up trace and span processors
        // 4. Configure sampling and resource attributes

        debug!("Distributed tracing initialized successfully");
        Ok(())
    }

    /// Start a remediation cycle span
    pub fn start_remediation_span(
        &self,
        task_id: &str,
        pr_number: i32,
        correlation_id: &str,
    ) -> TracingResult<ActiveSpan> {
        let span_id = format!("span-{}", Uuid::new_v4().simple());
        let trace_id = format!("trace-{}", correlation_id);

        let mut span = ActiveSpan::new(
            span_id.clone(),
            trace_id.clone(),
            "remediation_cycle".to_string(),
        );

        // Add standard attributes
        span.add_attribute("service.name", &self.service_name);
        span.add_attribute("service.version", &self.service_version);
        span.add_attribute("task.id", task_id);
        span.add_attribute("pr.number", &pr_number.to_string());
        span.add_attribute("correlation.id", correlation_id);
        span.add_attribute("operation", "remediation");

        // Store the active span
        {
            let mut spans = self.active_spans.lock().unwrap();
            spans.insert(span_id.clone(), span.clone());
        }

        debug!("Started remediation span: {} for task {}", span_id, task_id);
        Ok(span)
    }

    /// Start an agent operation span
    pub fn start_agent_operation_span(
        &self,
        task_id: &str,
        agent_type: &str,
        operation: &str,
        parent_span_id: Option<&str>,
    ) -> TracingResult<ActiveSpan> {
        let span_id = format!("span-{}", Uuid::new_v4().simple());
        let trace_id = if let Some(parent_id) = parent_span_id {
            // Inherit trace from parent span
            if let Some(parent_span) = self.active_spans.lock().unwrap().get(parent_id) {
                parent_span.trace_id().to_string()
            } else {
                format!("trace-{}", Uuid::new_v4().simple())
            }
        } else {
            format!("trace-{}", Uuid::new_v4().simple())
        };

        let mut span = ActiveSpan::new(
            span_id.clone(),
            trace_id.clone(),
            format!("agent_{}", operation),
        );

        // Add attributes
        span.add_attribute("service.name", &self.service_name);
        span.add_attribute("task.id", task_id);
        span.add_attribute("agent.type", agent_type);
        span.add_attribute("operation", operation);

        if let Some(parent_id) = parent_span_id {
            span.add_attribute("parent.span.id", parent_id);
        }

        // Store the active span
        {
            let mut spans = self.active_spans.lock().unwrap();
            spans.insert(span_id.clone(), span.clone());
        }

        debug!("Started agent operation span: {} for task {}", span_id, task_id);
        Ok(span)
    }

    /// Start a state operation span
    pub fn start_state_operation_span(
        &self,
        operation: &str,
        task_id: Option<&str>,
        pr_number: Option<i32>,
    ) -> TracingResult<ActiveSpan> {
        let span_id = format!("span-{}", Uuid::new_v4().simple());
        let trace_id = format!("trace-{}", uuid::Uuid::new_v4().simple());

        let mut span = ActiveSpan::new(
            span_id.clone(),
            trace_id.clone(),
            format!("state_{}", operation),
        );

        // Add attributes
        span.add_attribute("service.name", &self.service_name);
        span.add_attribute("operation", operation);

        if let Some(task_id) = task_id {
            span.add_attribute("task.id", task_id);
        }

        if let Some(pr_number) = pr_number {
            span.add_attribute("pr.number", &pr_number.to_string());
        }

        // Store the active span
        {
            let mut spans = self.active_spans.lock().unwrap();
            spans.insert(span_id.clone(), span.clone());
        }

        debug!("Started state operation span: {}", span_id);
        Ok(span)
    }

    /// End a remediation span
    pub fn end_remediation_span(&self, task_id: &str, success: bool) -> TracingResult<()> {
        let span_key = self.find_span_by_task_id(task_id)?;

        let mut spans = self.active_spans.lock().unwrap();
        if let Some(span) = spans.remove(&span_key) {
            let duration_ms = span.elapsed_ms();

            // In a real implementation, this would export the span to Jaeger
            debug!(
                "Ended remediation span: {} (duration: {}ms, success: {})",
                span.span_id(),
                duration_ms,
                success
            );
        }

        Ok(())
    }

    /// End an agent operation span
    pub fn end_agent_operation_span(&self, span_id: &str, success: bool) -> TracingResult<()> {
        let mut spans = self.active_spans.lock().unwrap();
        if let Some(span) = spans.remove(span_id) {
            let duration_ms = span.elapsed_ms();

            // In a real implementation, this would export the span
            debug!(
                "Ended agent operation span: {} (duration: {}ms, success: {})",
                span_id,
                duration_ms,
                success
            );
        }

        Ok(())
    }

    /// Add an event to a span
    pub fn add_span_event(&self, span_id: &str, event_name: &str, attributes: HashMap<String, String>) -> TracingResult<()> {
        let spans = self.active_spans.lock().unwrap();
        if let Some(_span) = spans.get(span_id) {
            debug!(
                "Span event: {} on span {} with attributes: {:?}",
                event_name,
                span_id,
                attributes
            );

            // In a real implementation, this would add the event to the span
        }

        Ok(())
    }

    /// Set error status on a span
    pub fn set_span_error(&self, span_id: &str, error_message: &str) -> TracingResult<()> {
        let spans = self.active_spans.lock().unwrap();
        if let Some(span) = spans.get(span_id) {
            warn!(
                "Span error: {} on span {} ({})",
                error_message,
                span_id,
                span.operation
            );

            // In a real implementation, this would set the error status
        }

        Ok(())
    }

    /// Get active spans count
    pub fn active_spans_count(&self) -> usize {
        self.active_spans.lock().unwrap().len()
    }

    /// Clean up completed spans (for memory management)
    pub fn cleanup_completed_spans(&self) -> TracingResult<usize> {
        // In a real implementation, this would remove spans that have been
        // exported and are no longer needed
        let count = self.active_spans.lock().unwrap().len();
        debug!("Active spans count: {}", count);
        Ok(count)
    }

    /// Find span by task ID (helper method)
    fn find_span_by_task_id(&self, task_id: &str) -> TracingResult<String> {
        let spans = self.active_spans.lock().unwrap();
        for (span_id, span) in spans.iter() {
            if span.attributes.get("task.id").map(|s| s.as_str()) == Some(task_id) {
                return Ok(span_id.clone());
            }
        }
        Err(TracingError::SpanError(format!("No active span found for task {}", task_id)))
    }

    /// Create a child span from a parent span
    pub fn create_child_span(
        &self,
        parent_span_id: &str,
        operation: &str,
        attributes: HashMap<String, String>,
    ) -> TracingResult<ActiveSpan> {
        let spans = self.active_spans.lock().unwrap();
        let parent_span = spans.get(parent_span_id)
            .ok_or_else(|| TracingError::SpanError(format!("Parent span {} not found", parent_span_id)))?;

        let span_id = format!("span-{}", Uuid::new_v4().simple());
        let trace_id = parent_span.trace_id().to_string();

        let mut span = ActiveSpan::new(
            span_id.clone(),
            trace_id.clone(),
            operation.to_string(),
        );

        // Add parent relationship
        span.add_attribute("parent.span.id", parent_span_id);

        // Add provided attributes
        for (key, value) in attributes {
            span.add_attribute(&key, &value);
        }

        // Store the span
        drop(spans);
        {
            let mut spans = self.active_spans.lock().unwrap();
            spans.insert(span_id.clone(), span.clone());
        }

        debug!("Created child span: {} from parent {}", span_id, parent_span_id);
        Ok(span)
    }
}

/// Convenience functions for common tracing patterns

/// Trace a remediation cycle with automatic span management
pub async fn trace_remediation_cycle<F, Fut, T>(
    trace_manager: &TraceManager,
    task_id: &str,
    pr_number: i32,
    correlation_id: &str,
    operation: F,
) -> TracingResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = TracingResult<T>>,
{
    let span = trace_manager.start_remediation_span(task_id, pr_number, correlation_id)?;

    let result = operation().await;

    let success = result.is_ok();
    trace_manager.end_remediation_span(task_id, success)?;

    result
}

/// Trace an agent operation with automatic span management
pub async fn trace_agent_operation<F, Fut, T>(
    trace_manager: &TraceManager,
    task_id: &str,
    agent_type: &str,
    operation_name: &str,
    parent_span_id: Option<&str>,
    operation: F,
) -> TracingResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = TracingResult<T>>,
{
    let span = trace_manager.start_agent_operation_span(
        task_id,
        agent_type,
        operation_name,
        parent_span_id,
    )?;

    let result = operation().await;

    let success = result.is_ok();
    trace_manager.end_agent_operation_span(span.span_id(), success)?;

    result
}
