//! # Prometheus Metrics Collection
//!
//! This module provides comprehensive Prometheus metrics collection for the Agent Remediation Loop,
//! including counters, gauges, and histograms for monitoring system health and performance.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Metrics collection errors
#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Metrics initialization error: {0}")]
    InitializationError(String),

    #[error("Metrics collection error: {0}")]
    CollectionError(String),

    #[error("Metrics export error: {0}")]
    ExportError(String),
}

/// Result type for metrics operations
pub type MetricsResult<T> = Result<T, MetricsError>;

/// Central metrics collector
pub struct MetricsCollector {
    /// Remediation cycle counters
    remediation_cycles_total: Arc<RwLock<u64>>,
    successful_remediations_total: Arc<RwLock<u64>>,
    failed_remediations_total: Arc<RwLock<u64>>,

    /// Agent operation counters
    agent_cancellations_total: Arc<RwLock<u64>>,
    active_agents_count: Arc<RwLock<HashMap<String, u32>>>,

    /// Escalation metrics
    escalations_total: Arc<RwLock<HashMap<String, u64>>>, // key: "type:severity"

    /// State management metrics
    state_operations_total: Arc<RwLock<HashMap<String, u64>>>, // key: operation_type
    state_operation_duration_seconds: Arc<RwLock<Vec<f64>>>,
    configmap_size_bytes: Arc<RwLock<u64>>,

    /// GitHub API metrics
    github_api_requests_total: Arc<RwLock<HashMap<String, u64>>>, // key: endpoint
    github_api_rate_limit_remaining: Arc<RwLock<i32>>,

    /// System health metrics
    system_health_score: Arc<RwLock<f64>>,
    error_rate_percent: Arc<RwLock<HashMap<String, f64>>>, // key: component

    /// Label management metrics
    label_operations_total: Arc<RwLock<HashMap<String, u64>>>, // key: operation_type
    label_operation_duration_seconds: Arc<RwLock<Vec<f64>>>,

    /// Performance tracking
    start_time: DateTime<Utc>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> MetricsResult<Self> {
        Ok(Self {
            remediation_cycles_total: Arc::new(RwLock::new(0)),
            successful_remediations_total: Arc::new(RwLock::new(0)),
            failed_remediations_total: Arc::new(RwLock::new(0)),
            agent_cancellations_total: Arc::new(RwLock::new(0)),
            active_agents_count: Arc::new(RwLock::new(HashMap::new())),
            escalations_total: Arc::new(RwLock::new(HashMap::new())),
            state_operations_total: Arc::new(RwLock::new(HashMap::new())),
            state_operation_duration_seconds: Arc::new(RwLock::new(Vec::new())),
            configmap_size_bytes: Arc::new(RwLock::new(0)),
            github_api_requests_total: Arc::new(RwLock::new(HashMap::new())),
            github_api_rate_limit_remaining: Arc::new(RwLock::new(5000)), // GitHub's default
            system_health_score: Arc::new(RwLock::new(1.0)), // Start healthy
            error_rate_percent: Arc::new(RwLock::new(HashMap::new())),
            label_operations_total: Arc::new(RwLock::new(HashMap::new())),
            label_operation_duration_seconds: Arc::new(RwLock::new(Vec::new())),
            start_time: Utc::now(),
        })
    }

    /// Initialize the metrics collector
    pub async fn initialize(&self) -> MetricsResult<()> {
        info!("Initializing Prometheus metrics collector");

        // Set initial values
        *self.system_health_score.write().await = 1.0;

        info!("Metrics collector initialized successfully");
        Ok(())
    }

    /// Increment remediation cycles counter
    pub fn increment_remediation_cycles(&self) -> MetricsResult<()> {
        let counter = Arc::clone(&self.remediation_cycles_total);
        tokio::spawn(async move {
            let mut count = counter.write().await;
            *count += 1;
        });
        Ok(())
    }

    /// Increment successful remediations counter
    pub fn increment_successful_remediations(&self) -> MetricsResult<()> {
        let counter = Arc::clone(&self.successful_remediations_total);
        tokio::spawn(async move {
            let mut count = counter.write().await;
            *count += 1;
        });
        Ok(())
    }

    /// Increment failed remediations counter
    pub fn increment_failed_remediations(&self) -> MetricsResult<()> {
        let counter = Arc::clone(&self.failed_remediations_total);
        tokio::spawn(async move {
            let mut count = counter.write().await;
            *count += 1;
        });
        Ok(())
    }

    /// Increment agent cancellations counter
    pub fn increment_agent_cancellations(&self) -> MetricsResult<()> {
        let counter = Arc::clone(&self.agent_cancellations_total);
        tokio::spawn(async move {
            let mut count = counter.write().await;
            *count += 1;
        });
        Ok(())
    }

    /// Update active agents count
    pub fn update_active_agents(&self, agent_type: &str, count: u32) -> MetricsResult<()> {
        let agents = Arc::clone(&self.active_agents_count);
        let agent_type = agent_type.to_string();
        tokio::spawn(async move {
            let mut agents_map = agents.write().await;
            agents_map.insert(agent_type, count);
        });
        Ok(())
    }

    /// Increment escalations counter
    pub fn increment_escalations(&self, escalation_type: &str, severity: &str) -> MetricsResult<()> {
        let escalations = Arc::clone(&self.escalations_total);
        let key = format!("{}:{}", escalation_type, severity);
        tokio::spawn(async move {
            let mut escalation_map = escalations.write().await;
            *escalation_map.entry(key).or_insert(0) += 1;
        });
        Ok(())
    }

    /// Record remediation duration
    pub fn record_remediation_duration(&self, duration_seconds: f64) -> MetricsResult<()> {
        let durations = Arc::clone(&self.state_operation_duration_seconds);
        tokio::spawn(async move {
            let mut duration_vec = durations.write().await;
            duration_vec.push(duration_seconds);

            // Keep only last 1000 samples to prevent unbounded growth
            if duration_vec.len() > 1000 {
                duration_vec.remove(0);
            }
        });
        Ok(())
    }

    /// Update GitHub rate limit remaining
    pub fn update_github_rate_limit(&self, remaining: i32) -> MetricsResult<()> {
        let rate_limit = Arc::clone(&self.github_api_rate_limit_remaining);
        tokio::spawn(async move {
            let mut limit = rate_limit.write().await;
            *limit = remaining;
        });
        Ok(())
    }

    /// Update system health score
    pub fn update_health_score(&self, score: f64) -> MetricsResult<()> {
        let health_score = Arc::clone(&self.system_health_score);
        tokio::spawn(async move {
            let mut score_ref = health_score.write().await;
            *score_ref = score.clamp(0.0, 1.0);
        });
        Ok(())
    }

    /// Update error rate for a component
    pub fn update_error_rate(&self, component: &str, error_rate: f64) -> MetricsResult<()> {
        let error_rates = Arc::clone(&self.error_rate_percent);
        let component = component.to_string();
        tokio::spawn(async move {
            let mut rates = error_rates.write().await;
            rates.insert(component, error_rate);
        });
        Ok(())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus_metrics(&self) -> MetricsResult<String> {
        let mut output = String::new();

        // Add header
        output.push_str("# HELP remediation_cycles_total Total number of remediation cycles started\n");
        output.push_str("# TYPE remediation_cycles_total counter\n");
        output.push_str(&format!("remediation_cycles_total {}\n", *self.remediation_cycles_total.read().await));

        output.push_str("# HELP successful_remediations_total Total number of successful remediations\n");
        output.push_str("# TYPE successful_remediations_total counter\n");
        output.push_str(&format!("successful_remediations_total {}\n", *self.successful_remediations_total.read().await));

        output.push_str("# HELP failed_remediations_total Total number of failed remediations\n");
        output.push_str("# TYPE failed_remediations_total counter\n");
        output.push_str(&format!("failed_remediations_total {}\n", *self.failed_remediations_total.read().await));

        output.push_str("# HELP agent_cancellations_total Total number of agent cancellations\n");
        output.push_str("# TYPE agent_cancellations_total counter\n");
        output.push_str(&format!("agent_cancellations_total {}\n", *self.agent_cancellations_total.read().await));

        // Active agents gauge
        output.push_str("# HELP active_agents_count Number of currently active agents by type\n");
        output.push_str("# TYPE active_agents_count gauge\n");
        for (agent_type, count) in &*self.active_agents_count.read().await {
            output.push_str(&format!("active_agents_count{{agent_type=\"{}\"}} {}\n", agent_type, count));
        }

        // Escalations counter
        output.push_str("# HELP escalations_total Total number of escalations by type and severity\n");
        output.push_str("# TYPE escalations_total counter\n");
        for (key, count) in &*self.escalations_total.read().await {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                output.push_str(&format!("escalations_total{{type=\"{}\",severity=\"{}\"}} {}\n", parts[0], parts[1], count));
            }
        }

        // System health gauge
        output.push_str("# HELP system_health_score Overall system health score (0.0-1.0)\n");
        output.push_str("# TYPE system_health_score gauge\n");
        output.push_str(&format!("system_health_score {}\n", *self.system_health_score.read().await));

        // GitHub rate limit gauge
        output.push_str("# HELP github_api_rate_limit_remaining Remaining GitHub API calls\n");
        output.push_str("# TYPE github_api_rate_limit_remaining gauge\n");
        output.push_str(&format!("github_api_rate_limit_remaining {}\n", *self.github_api_rate_limit_remaining.read().await));

        // Error rates
        output.push_str("# HELP error_rate_percent Error rate percentage by component\n");
        output.push_str("# TYPE error_rate_percent gauge\n");
        for (component, rate) in &*self.error_rate_percent.read().await {
            output.push_str(&format!("error_rate_percent{{component=\"{}\"}} {:.2}\n", component, rate));
        }

        // Remediation duration histogram (simplified)
        let durations = self.state_operation_duration_seconds.read().await;
        if !durations.is_empty() {
            let sum: f64 = durations.iter().sum();
            let count = durations.len();
            let avg = if count > 0 { sum / count as f64 } else { 0.0 };

            output.push_str("# HELP remediation_duration_seconds Duration of remediation cycles\n");
            output.push_str("# TYPE remediation_duration_seconds histogram\n");
            output.push_str(&format!("remediation_duration_seconds_sum {}\n", sum));
            output.push_str(&format!("remediation_duration_seconds_count {}\n", count));
            output.push_str("# HELP remediation_duration_seconds_avg Average remediation duration\n");
            output.push_str("# TYPE remediation_duration_seconds_avg gauge\n");
            output.push_str(&format!("remediation_duration_seconds_avg {}\n", avg));
        }

        Ok(output)
    }

    /// Get current metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> HashMap<String, serde_json::Value> {
        let mut snapshot = HashMap::new();

        snapshot.insert("remediation_cycles_total".to_string(), (*self.remediation_cycles_total.read().await).into());
        snapshot.insert("successful_remediations_total".to_string(), (*self.successful_remediations_total.read().await).into());
        snapshot.insert("failed_remediations_total".to_string(), (*self.failed_remediations_total.read().await).into());
        snapshot.insert("agent_cancellations_total".to_string(), (*self.agent_cancellations_total.read().await).into());
        snapshot.insert("system_health_score".to_string(), (*self.system_health_score.read().await).into());
        snapshot.insert("github_api_rate_limit_remaining".to_string(), (*self.github_api_rate_limit_remaining.read().await).into());

        snapshot
    }

    /// Reset all counters (useful for testing)
    pub async fn reset(&self) -> MetricsResult<()> {
        *self.remediation_cycles_total.write().await = 0;
        *self.successful_remediations_total.write().await = 0;
        *self.failed_remediations_total.write().await = 0;
        *self.agent_cancellations_total.write().await = 0;
        *self.escalations_total.write().await = HashMap::new();
        *self.state_operations_total.write().await = HashMap::new();
        *self.state_operation_duration_seconds.write().await = Vec::new();
        *self.github_api_requests_total.write().await = HashMap::new();
        *self.label_operations_total.write().await = HashMap::new();
        *self.label_operation_duration_seconds.write().await = Vec::new();

        Ok(())
    }
}
