//! Base Adapter Implementation
//!
//! Provides shared functionality for all CLI adapters including logging, metrics collection,
//! template rendering, and common utilities.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, ContainerContext, HealthState, HealthStatus,
};
use crate::cli::types::CLIType;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output,
    RenderContext,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Operation context for tracing
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub cli_type: CLIType,
    pub correlation_id: String,
    pub start_time: Instant,
}

/// Configuration for `BaseAdapter`
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Root directory for CLI templates
    pub template_root: PathBuf,
    /// Template cache size
    pub template_cache_size: usize,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Metrics prefix
    pub metrics_prefix: String,
    /// Enable detailed logging
    pub verbose_logging: bool,
}

impl AdapterConfig {
    pub fn new(cli_type: CLIType) -> Self {
        let default_template_root = if let Ok(path) = std::env::var("CLI_TEMPLATES_ROOT") { PathBuf::from(path) } else {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok();
            let repo_relative = manifest_dir
                .map(PathBuf::from)
                .map(|dir| {
                    dir.join("..")
                        .join("infra/charts/controller/agent-templates")
                })
                .filter(|path| path.exists());

            if let Some(path) = repo_relative {
                path
            } else {
                PathBuf::from("/agent-templates")
            }
        };
        Self {
            cli_type,
            correlation_id: Uuid::new_v4().to_string(),
            template_root: default_template_root,
            template_cache_size: 100,
            health_check_timeout: Duration::from_secs(10),
            metrics_prefix: format!("cli_adapter_{cli_type}"),
            verbose_logging: false,
        }
    }

    #[must_use] pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    pub fn with_template_root<P: Into<PathBuf>>(mut self, template_root: P) -> Self {
        self.template_root = template_root.into();
        self
    }

    #[must_use] pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }
}

/// Base adapter providing shared functionality
#[derive(Debug)]
pub struct BaseAdapter {
    /// CLI type this adapter handles
    pub cli_type: CLIType,
    /// Adapter configuration
    pub config: AdapterConfig,
    /// OpenTelemetry metrics
    pub metrics: Arc<AdapterMetrics>,
    /// Handlebars template engine
    pub templates: Arc<Handlebars<'static>>,
    /// OpenTelemetry tracer (simplified)
    pub tracer: Option<String>,
}

impl BaseAdapter {
    /// Create new base adapter
    pub async fn new(config: AdapterConfig) -> AdapterResult<Self> {
        let metrics = Arc::new(AdapterMetrics::new(&config.metrics_prefix)?);
        let mut templates = Handlebars::new();

        // Register custom helpers
        Self::register_template_helpers(&mut templates);

        let tracer = Some("cli_adapter".to_string());

        Ok(Self {
            cli_type: config.cli_type,
            config,
            metrics,
            templates: Arc::new(templates),
            tracer,
        })
    }

    /// Log structured operation with correlation ID and context
    #[instrument(skip(self, context))]
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        let mut log_context = context.clone();
        log_context.insert("cli_type".to_string(), self.cli_type.to_string());
        log_context.insert(
            "correlation_id".to_string(),
            self.config.correlation_id.clone(),
        );

        info!(
            operation = %operation,
            cli_type = %self.cli_type,
            correlation_id = %self.config.correlation_id,
            context = ?log_context,
            "CLI adapter operation"
        );
    }

    /// Record metrics with OpenTelemetry
    #[instrument(skip(self))]
    pub async fn record_metrics(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
    ) -> AdapterResult<()> {
        let labels = [
            KeyValue::new("cli_type", self.cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
            KeyValue::new("correlation_id", self.config.correlation_id.clone()),
        ];

        // Record operation count
        self.metrics.operations_total.add(1, &labels);

        // Record operation duration
        self.metrics
            .operation_duration
            .record(duration.as_millis() as f64, &labels);

        // Record failures
        if !success {
            self.metrics.operation_failures.add(1, &labels);
        }

        debug!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded adapter metrics"
        );

        Ok(())
    }

    /// Validate base configuration common to all adapters
    #[instrument(skip(self, config))]
    pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
        // Validate required fields
        if config.github_app.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "GitHub app cannot be empty".to_string(),
            ));
        }

        if config.cli.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "CLI type cannot be empty".to_string(),
            ));
        }

        if config.model.trim().is_empty() {
            return Err(AdapterError::ValidationError(
                "Model cannot be empty".to_string(),
            ));
        }

        // Validate CLI type matches adapter
        let expected_cli = self.cli_type.to_string();
        if config.cli != expected_cli {
            return Err(AdapterError::ValidationError(format!(
                "CLI type mismatch: expected '{}', got '{}'",
                expected_cli, config.cli
            )));
        }

        // Validate optional fields
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 || max_tokens > 1_000_000 {
                return Err(AdapterError::ValidationError(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                ));
            }
        }

        if let Some(temperature) = config.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(AdapterError::ValidationError(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        info!(
            github_app = %config.github_app,
            cli = %config.cli,
            model = %config.model,
            "Base configuration validation passed"
        );

        Ok(())
    }

    /// Render template with Handlebars and context
    #[instrument(skip(self, template_content, context))]
    pub fn render_template(
        &self,
        template_content: &str,
        context: &Value,
    ) -> AdapterResult<String> {
        let start_time = Instant::now();

        // Add base context
        let mut full_context = context.clone();
        if let Value::Object(ref mut map) = full_context {
            map.insert(
                "cli_type".to_string(),
                Value::String(self.cli_type.to_string()),
            );
            map.insert(
                "correlation_id".to_string(),
                Value::String(self.config.correlation_id.clone()),
            );
            map.insert(
                "timestamp".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let result = self
            .templates
            .render_template(template_content, &full_context)
            .map_err(|e| {
                error!(error = %e, "Template rendering failed");
                AdapterError::TemplateError(format!("Template rendering failed: {e}"))
            })?;

        let duration = start_time.elapsed();
        debug!(
            template_length = template_content.len(),
            result_length = result.len(),
            duration_ms = duration.as_millis(),
            "Template rendered successfully"
        );

        Ok(result)
    }

    /// Render template from file relative to template root
    #[instrument(skip(self, template_path, context))]
    pub fn render_template_file(
        &self,
        template_path: &str,
        context: &Value,
    ) -> AdapterResult<String> {
        let template_content = self.load_template(template_path)?;
        self.render_template(&template_content, context)
    }

    /// Load a template from the configured template root
    pub fn load_template(&self, relative_path: &str) -> AdapterResult<String> {
        let path = self.config.template_root.join(relative_path);
        let content = fs::read_to_string(&path).map_err(|e| {
            AdapterError::TemplateError(format!(
                "Failed to load template {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(content)
    }

    /// Register custom template helpers
    fn register_template_helpers(hb: &mut Handlebars<'static>) {
        // Helper for JSON serialization
        handlebars_helper!(json: |obj: Value| {
            serde_json::to_string(&obj).unwrap_or_else(|_| "null".to_string())
        });

        // Helper for CLI-specific formatting
        handlebars_helper!(cli_format: |cli_type: str, content: str| {
            match cli_type {
                "claude" => format!("# Claude Configuration\n\n{content}"),
                "codex" => format!("# Codex Configuration\n{content}"),
                _ => content.to_string(),
            }
        });

        // Helper for timestamp formatting
        fn timestamp_helper(
            _: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let timestamp = chrono::Utc::now().to_rfc3339();
            out.write(&timestamp)?;
            Ok(())
        }

        // Helper for environment variable substitution
        fn env_helper(
            h: &Helper,
            _: &Handlebars,
            _: &HandlebarsContext,
            _: &mut RenderContext,
            out: &mut dyn Output,
        ) -> HelperResult {
            let var_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

            let value = std::env::var(var_name).unwrap_or_else(|_| format!("${{{var_name}}}"));

            out.write(&value)?;
            Ok(())
        }

        // Helper for conditional CLI features
        handlebars_helper!(if_cli_supports: |cli_type: str, feature: str, then_val: Value, else_val: Value| {
            let supports = match (cli_type, feature) {
                ("claude", "streaming") => true,
                ("claude", "multimodal") => false,
                ("codex", "streaming") => false,
                ("codex", "toml_config") => true,
                ("gemini", "multimodal") => true,
                _ => false,
            };

            if supports { then_val } else { else_val }
        });

        hb.register_helper("json", Box::new(json));
        hb.register_helper("cli_format", Box::new(cli_format));
        hb.register_helper("timestamp", Box::new(timestamp_helper));
        hb.register_helper("env", Box::new(env_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports));
    }

    /// Create operation context for tracing
    #[instrument(skip(self))]
    pub fn create_operation_context(&self, operation: &str) -> OperationContext {
        OperationContext {
            operation: operation.to_string(),
            cli_type: self.cli_type,
            correlation_id: self.config.correlation_id.clone(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Perform common health checks
    #[instrument(skip(self, container))]
    pub async fn base_health_check(
        &self,
        container: &ContainerContext,
    ) -> AdapterResult<HealthStatus> {
        let _ctx = self.create_operation_context("health_check");
        let start_time = Instant::now();
        let mut health_details = HashMap::new();

        // Check adapter configuration
        health_details.insert(
            "config_valid".to_string(),
            json!(self.config.cli_type == self.cli_type),
        );

        // Check template engine
        let template_test = self.render_template("test: {{cli_type}}", &json!({}));
        health_details.insert(
            "templates_working".to_string(),
            json!(template_test.is_ok()),
        );

        // Check metrics collection
        health_details.insert(
            "metrics_available".to_string(),
            json!(true), // Always true since we initialize metrics
        );

        // Check container context if provided
        if let Some(pod) = &container.pod {
            health_details.insert(
                "pod_available".to_string(),
                json!(pod.metadata.name.is_some()),
            );
        }

        let duration = start_time.elapsed();
        health_details.insert("check_duration_ms".to_string(), json!(duration.as_millis()));

        // Determine overall health
        let is_healthy = template_test.is_ok() && duration < self.config.health_check_timeout;

        let status = if is_healthy {
            HealthState::Healthy
        } else {
            HealthState::Warning
        };

        let health = HealthStatus {
            status,
            message: if is_healthy {
                None
            } else {
                Some("Some health checks failed".to_string())
            },
            checked_at: chrono::Utc::now(),
            details: health_details,
        };

        // Operation completed

        info!(
            status = ?health.status,
            duration_ms = duration.as_millis(),
            "Base health check completed"
        );

        Ok(health)
    }

    /// Common initialization tasks
    #[instrument(skip(self, container))]
    pub async fn base_initialize(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("initialize");

        info!(
            container_name = %container.container_name,
            working_dir = %container.working_dir,
            namespace = %container.namespace,
            env_vars_count = container.env_vars.len(),
            "Initializing base adapter"
        );

        // Validate container context
        if container.container_name.is_empty() {
            // Container name validation failed
            return Err(AdapterError::InitializationError(
                "Container name cannot be empty".to_string(),
            ));
        }

        if container.working_dir.is_empty() {
            // Working directory validation failed
            return Err(AdapterError::InitializationError(
                "Working directory cannot be empty".to_string(),
            ));
        }

        // Log environment variables for debugging (without values for security)
        if self.config.verbose_logging {
            let env_keys: Vec<&String> = container.env_vars.keys().collect();
            debug!(env_keys = ?env_keys, "Container environment variables");
        }

        // Operation completed

        info!("Base adapter initialization completed successfully");
        Ok(())
    }

    /// Common cleanup tasks
    #[instrument(skip(self, container))]
    pub async fn base_cleanup(&self, container: &ContainerContext) -> AdapterResult<()> {
        let _ctx = self.create_operation_context("cleanup");

        info!(
            container_name = %container.container_name,
            "Starting base adapter cleanup"
        );

        // Flush any pending metrics
        // Note: OpenTelemetry metrics are typically handled by the SDK
        debug!("Flushed pending metrics");

        // Log cleanup completion
        info!("Base adapter cleanup completed");

        // Operation completed

        Ok(())
    }

    /// Get adapter configuration summary for diagnostics
    #[must_use] pub fn get_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert("cli_type".to_string(), json!(self.cli_type.to_string()));
        summary.insert(
            "correlation_id".to_string(),
            json!(self.config.correlation_id),
        );
        summary.insert(
            "template_cache_size".to_string(),
            json!(self.config.template_cache_size),
        );
        summary.insert(
            "health_check_timeout_ms".to_string(),
            json!(self.config.health_check_timeout.as_millis()),
        );
        summary.insert(
            "metrics_prefix".to_string(),
            json!(self.config.metrics_prefix),
        );
        summary.insert(
            "verbose_logging".to_string(),
            json!(self.config.verbose_logging),
        );

        summary
    }
}

/// OpenTelemetry metrics for adapters
#[derive(Debug)]
pub struct AdapterMetrics {
    /// Counter for total operations
    pub operations_total: Counter<u64>,
    /// Counter for operation failures
    pub operation_failures: Counter<u64>,
    /// Histogram for operation duration
    pub operation_duration: Histogram<f64>,
}

impl AdapterMetrics {
    /// Create new metrics instance
    pub fn new(prefix: &str) -> AdapterResult<Self> {
        let meter = global::meter("cli_adapter");

        let operations_total = meter
            .u64_counter(format!("{prefix}_operations_total"))
            .with_description("Total number of adapter operations")
            .build();

        let operation_failures = meter
            .u64_counter(format!("{prefix}_operation_failures"))
            .with_description("Number of failed adapter operations")
            .build();

        let operation_duration = meter
            .f64_histogram(format!("{prefix}_operation_duration_ms"))
            .with_description("Duration of adapter operations in milliseconds")
            .build();

        Ok(Self {
            operations_total,
            operation_failures,
            operation_duration,
        })
    }

    /// Record operation metrics
    pub fn record_operation(
        &self,
        cli_type: CLIType,
        operation: &str,
        duration: Duration,
        success: bool,
    ) {
        let labels = [
            KeyValue::new("cli_type", cli_type.to_string()),
            KeyValue::new("operation", operation.to_string()),
            KeyValue::new("success", success.to_string()),
        ];

        self.operations_total.add(1, &labels);
        self.operation_duration
            .record(duration.as_millis() as f64, &labels);

        if !success {
            self.operation_failures.add(1, &labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[tokio::test]
    async fn test_base_adapter_creation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        assert_eq!(adapter.cli_type, CLIType::Claude);
        assert!(!adapter.config.correlation_id.is_empty());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new(CLIType::Codex)
            .with_correlation_id("test-123".to_string())
            .with_verbose_logging(true);

        assert_eq!(config.cli_type, CLIType::Codex);
        assert_eq!(config.correlation_id, "test-123");
        assert!(config.verbose_logging);
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let template = "Hello {{name}}, CLI: {{cli_type}}";
        let context = json!({"name": "World"});

        let result = adapter.render_template(template, &context).unwrap();
        assert!(result.contains("Hello World"));
        assert!(result.contains("CLI: claude"));
    }

    #[tokio::test]
    async fn test_base_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid config
        let valid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: "claude-3-opus".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&valid_config).is_ok());

        // Invalid config - empty model
        let invalid_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "claude".to_string(),
            model: String::new(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        assert!(adapter.validate_base_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        let container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let health = adapter.base_health_check(&container).await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[test]
    fn test_template_helpers() {
        let mut hb = Handlebars::new();
        BaseAdapter::register_template_helpers(&mut hb);

        // Test JSON helper
        let template = "{{json data}}";
        let context = json!({"data": {"key": "value"}});
        let result = hb.render_template(template, &context).unwrap();
        // JSON helper should output valid JSON (formatting may vary)
        assert!(result.contains("key") && result.contains("value"));

        // Test timestamp helper
        let template = "Generated at: {{timestamp}}";
        let result = hb.render_template(template, &json!({})).unwrap();
        assert!(result.contains("Generated at:"));
    }

    #[tokio::test]
    async fn test_config_summary() {
        let config =
            AdapterConfig::new(CLIType::Gemini).with_correlation_id("test-456".to_string());

        let adapter = BaseAdapter::new(config).await.unwrap();
        let summary = adapter.get_config_summary();

        assert_eq!(summary["cli_type"], json!("gemini"));
        assert_eq!(summary["correlation_id"], json!("test-456"));
        assert!(summary.contains_key("health_check_timeout_ms"));
    }

    #[tokio::test]
    async fn test_base_initialization_validation() {
        let config = AdapterConfig::new(CLIType::Claude);
        let adapter = BaseAdapter::new(config).await.unwrap();

        // Valid container context
        let valid_container = ContainerContext {
            pod: None,
            container_name: "test-container".to_string(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&valid_container).await.is_ok());

        // Invalid container context - empty name
        let invalid_container = ContainerContext {
            pod: None,
            container_name: String::new(),
            working_dir: "/workspace".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        assert!(adapter.base_initialize(&invalid_container).await.is_err());
    }
}
