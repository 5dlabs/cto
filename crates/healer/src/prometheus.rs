//! Prometheus client for querying platform metrics.
//!
//! Provides functions to query Prometheus for:
//! - Pod status and health
//! - Resource usage
//! - Workflow execution metrics
//! - Alert states

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Default Prometheus service URL (internal Kubernetes DNS)
const DEFAULT_PROMETHEUS_URL: &str = "http://prometheus-server.observability.svc.cluster.local:80";

/// Configuration for the Prometheus client
#[derive(Debug, Clone)]
pub struct PrometheusConfig {
    /// Base URL for the Prometheus API
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("PROMETHEUS_URL")
                .unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string()),
            timeout_secs: 30,
        }
    }
}

/// Prometheus query response
#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    status: String,
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    #[serde(rename = "resultType")]
    #[allow(dead_code)]
    result_type: String,
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    metric: std::collections::HashMap<String, String>,
    value: Option<(f64, String)>,          // For instant queries
    values: Option<Vec<(f64, String)>>,    // For range queries
}

/// A metric sample from Prometheus
#[derive(Debug, Clone, Serialize)]
pub struct MetricSample {
    /// Labels associated with this metric
    pub labels: std::collections::HashMap<String, String>,
    /// The metric value
    pub value: f64,
    /// Timestamp of the sample
    pub timestamp: DateTime<Utc>,
}

/// Pod status information
#[derive(Debug, Clone, Serialize)]
pub struct PodStatus {
    /// Pod name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Current phase (Running, Pending, Failed, etc.)
    pub phase: String,
    /// Whether pod is ready
    pub ready: bool,
    /// Number of restarts
    pub restart_count: u32,
    /// Age in seconds
    pub age_seconds: f64,
}

/// Prometheus client for querying metrics
#[derive(Debug, Clone)]
pub struct PrometheusClient {
    config: PrometheusConfig,
    client: reqwest::Client,
}

impl PrometheusClient {
    /// Create a new Prometheus client with the given configuration.
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be created.
    #[must_use]
    pub fn new(config: PrometheusConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Create a new client with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(PrometheusConfig::default())
    }

    /// Execute an instant query.
    ///
    /// # Errors
    /// Returns an error if the query fails or response cannot be parsed.
    pub async fn query(&self, query: &str) -> Result<Vec<MetricSample>> {
        let url = format!(
            "{}/api/v1/query",
            self.config.base_url.trim_end_matches('/')
        );

        debug!(query = %query, "Executing Prometheus query");

        let response = self
            .client
            .get(&url)
            .query(&[("query", query)])
            .send()
            .await
            .context("Failed to send request to Prometheus")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Prometheus query failed with status {status}: {body}");
        }

        let prom_response: PrometheusResponse = response
            .json()
            .await
            .context("Failed to parse Prometheus response")?;

        if prom_response.status != "success" {
            anyhow::bail!("Prometheus query returned status: {}", prom_response.status);
        }

        Ok(self.parse_results(&prom_response.data.result))
    }

    /// Execute a range query.
    ///
    /// # Errors
    /// Returns an error if the query fails or response cannot be parsed.
    pub async fn query_range(
        &self,
        query: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        step: &str,
    ) -> Result<Vec<MetricSample>> {
        let url = format!(
            "{}/api/v1/query_range",
            self.config.base_url.trim_end_matches('/')
        );

        debug!(
            query = %query,
            start = %start,
            end = %end,
            step = %step,
            "Executing Prometheus range query"
        );

        let response = self
            .client
            .get(&url)
            .query(&[
                ("query", query),
                ("start", &start.timestamp().to_string()),
                ("end", &end.timestamp().to_string()),
                ("step", step),
            ])
            .send()
            .await
            .context("Failed to send range request to Prometheus")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Prometheus range query failed with status {status}: {body}");
        }

        let prom_response: PrometheusResponse = response
            .json()
            .await
            .context("Failed to parse Prometheus range response")?;

        if prom_response.status != "success" {
            anyhow::bail!(
                "Prometheus range query returned status: {}",
                prom_response.status
            );
        }

        Ok(self.parse_range_results(&prom_response.data.result))
    }

    /// Get pod statuses for a namespace.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_pod_statuses(&self, namespace: &str) -> Result<Vec<PodStatus>> {
        let phase_query = format!(
            r#"kube_pod_status_phase{{namespace="{namespace}"}} == 1"#
        );
        let ready_query = format!(
            r#"kube_pod_status_ready{{namespace="{namespace}", condition="true"}} == 1"#
        );
        let restart_query = format!(
            r#"kube_pod_container_status_restarts_total{{namespace="{namespace}"}}"#
        );
        let created_query = format!(
            r#"kube_pod_created{{namespace="{namespace}"}}"#
        );

        // Execute queries in parallel
        let (phases, readies, restarts, created) = tokio::try_join!(
            self.query(&phase_query),
            self.query(&ready_query),
            self.query(&restart_query),
            self.query(&created_query),
        )?;

        // Build pod status map
        let mut pods: std::collections::HashMap<String, PodStatus> = std::collections::HashMap::new();

        // Process phases
        for sample in phases {
            let name = sample.labels.get("pod").cloned().unwrap_or_default();
            let phase = sample.labels.get("phase").cloned().unwrap_or_default();
            
            pods.entry(name.clone()).or_insert_with(|| PodStatus {
                name: name.clone(),
                namespace: namespace.to_string(),
                phase: String::new(),
                ready: false,
                restart_count: 0,
                age_seconds: 0.0,
            }).phase = phase;
        }

        // Process readiness
        for sample in readies {
            let name = sample.labels.get("pod").cloned().unwrap_or_default();
            if let Some(pod) = pods.get_mut(&name) {
                pod.ready = sample.value > 0.0;
            }
        }

        // Process restarts (sum across containers)
        for sample in restarts {
            let name = sample.labels.get("pod").cloned().unwrap_or_default();
            if let Some(pod) = pods.get_mut(&name) {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    pod.restart_count = pod.restart_count.saturating_add(sample.value as u32);
                }
            }
        }

        // Process creation time (calculate age)
        #[allow(clippy::cast_precision_loss)]
        let now = Utc::now().timestamp() as f64;
        for sample in created {
            let name = sample.labels.get("pod").cloned().unwrap_or_default();
            if let Some(pod) = pods.get_mut(&name) {
                pod.age_seconds = now - sample.value;
            }
        }

        Ok(pods.into_values().collect())
    }

    /// Get CTO platform pod statuses.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_cto_pod_statuses(&self) -> Result<Vec<PodStatus>> {
        self.get_pod_statuses("cto").await
    }

    /// Get workflow pod statuses (cto and automation namespaces).
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_workflow_pod_statuses(&self) -> Result<Vec<PodStatus>> {
        let (cto_pods, automation_pods) = tokio::try_join!(
            self.get_pod_statuses("cto"),
            self.get_pod_statuses("automation"),
        )?;

        let mut all_pods = cto_pods;
        all_pods.extend(automation_pods);

        // Filter to workflow pods only
        Ok(all_pods
            .into_iter()
            .filter(|p| {
                p.name.starts_with("play-")
                    || p.name.starts_with("intake-")
                    || p.name.starts_with("project-intake-")
            })
            .collect())
    }

    /// Check if a component is healthy.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn is_component_healthy(&self, namespace: &str, pod_pattern: &str) -> Result<bool> {
        let query = format!(
            r#"kube_pod_status_phase{{namespace="{namespace}", pod=~"{pod_pattern}", phase="Running"}} == 1"#
        );

        let results = self.query(&query).await?;
        Ok(!results.is_empty())
    }

    /// Get current firing alerts.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_firing_alerts(&self) -> Result<Vec<FiringAlert>> {
        let query = "ALERTS{alertstate=\"firing\"}";
        let results = self.query(query).await?;

        Ok(results
            .into_iter()
            .map(|sample| FiringAlert {
                name: sample.labels.get("alertname").cloned().unwrap_or_default(),
                severity: sample.labels.get("severity").cloned().unwrap_or_default(),
                labels: sample.labels,
            })
            .collect())
    }

    /// Check Prometheus health.
    ///
    /// # Errors
    /// Returns an error if there's an issue building the request.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/-/healthy", self.config.base_url.trim_end_matches('/'));

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!(error = %e, "Prometheus health check failed");
                Ok(false)
            }
        }
    }

    /// Parse instant query results.
    #[allow(clippy::unused_self)]
    fn parse_results(&self, results: &[PrometheusResult]) -> Vec<MetricSample> {
        let mut samples = Vec::new();

        for result in results {
            if let Some((timestamp, value_str)) = &result.value {
                let value: f64 = value_str.parse().unwrap_or(0.0);
                #[allow(clippy::cast_possible_truncation)]
                let ts = DateTime::from_timestamp(*timestamp as i64, 0)
                    .unwrap_or_else(Utc::now);

                samples.push(MetricSample {
                    labels: result.metric.clone(),
                    value,
                    timestamp: ts,
                });
            }
        }

        samples
    }

    /// Parse range query results (flattens all time series).
    #[allow(clippy::unused_self)]
    fn parse_range_results(&self, results: &[PrometheusResult]) -> Vec<MetricSample> {
        let mut samples = Vec::new();

        for result in results {
            if let Some(values) = &result.values {
                for (timestamp, value_str) in values {
                    let value: f64 = value_str.parse().unwrap_or(0.0);
                    #[allow(clippy::cast_possible_truncation)]
                    let ts = DateTime::from_timestamp(*timestamp as i64, 0)
                        .unwrap_or_else(Utc::now);

                    samples.push(MetricSample {
                        labels: result.metric.clone(),
                        value,
                        timestamp: ts,
                    });
                }
            }
        }

        samples
    }
}

/// A currently firing alert
#[derive(Debug, Clone, Serialize)]
pub struct FiringAlert {
    /// Alert name
    pub name: String,
    /// Alert severity
    pub severity: String,
    /// All labels
    pub labels: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = PrometheusConfig::default();
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_client_creation() {
        let client = PrometheusClient::with_defaults();
        assert!(!client.config.base_url.is_empty());
    }
}
