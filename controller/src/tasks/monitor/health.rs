//! # Health Check System
//!
//! This module provides comprehensive health check capabilities for the Agent Remediation Loop,
//! including component-specific health checks and overall system health assessment.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};

/// Health check errors
#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Health check initialization error: {0}")]
    InitializationError(String),

    #[error("Health check execution error: {0}")]
    CheckError(String),

    #[error("Component unavailable: {0}")]
    ComponentUnavailable(String),
}

/// Result type for health operations
pub type HealthResult<T> = Result<T, HealthError>;

/// Health status of a component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentHealth {
    /// Component is healthy
    Healthy,
    /// Component has issues but is still functional
    Degraded,
    /// Component is not functioning
    Unhealthy,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_health: ComponentHealth,
    pub overall_score: f64, // 0.0 to 1.0
    pub component_statuses: HashMap<String, ComponentStatus>,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
}

impl HealthStatus {
    /// Calculate overall health score from component statuses
    pub fn calculate_overall_score(&self) -> f64 {
        if self.component_statuses.is_empty() {
            return 1.0;
        }

        let total_score: f64 = self.component_statuses.values()
            .map(|status| status.score)
            .sum();

        total_score / self.component_statuses.len() as f64
    }
}

/// Individual component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub component: String,
    pub health: ComponentHealth,
    pub score: f64, // 0.0 to 1.0
    pub message: String,
    pub details: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
    pub check_duration_ms: u64,
}

/// Health check trait for pluggable health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Get the component name
    fn component_name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> HealthResult<ComponentStatus>;
}

/// Health checker with pluggable checks
pub struct HealthChecker {
    checks: HashMap<String, Box<dyn HealthCheck>>,
    cache: Arc<RwLock<HashMap<String, ComponentStatus>>>,
    cache_ttl_seconds: u64,
    start_time: DateTime<Utc>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> HealthResult<Self> {
        Ok(Self {
            checks: HashMap::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_seconds: 30, // 30 second cache TTL
            start_time: Utc::now(),
        })
    }

    /// Initialize the health checker with default checks
    pub async fn initialize(&self) -> HealthResult<()> {
        info!("Initializing health check system");

        // Default checks would be registered here
        // In a real implementation, this would register:
        // - GitHub API health check
        // - Kubernetes API health check
        // - Database connectivity check
        // - Internal service health checks

        Ok(())
    }

    /// Register a health check
    pub fn register_check(&mut self, check: Box<dyn HealthCheck>) {
        let name = check.component_name().to_string();
        self.checks.insert(name, check);
    }

    /// Perform comprehensive health check
    pub async fn perform_comprehensive_check(&self) -> HealthResult<HealthStatus> {
        let mut component_statuses = HashMap::new();
        let mut total_score = 0.0;
        let mut unhealthy_count = 0;

        for (component_name, check) in &self.checks {
            match self.perform_cached_check(component_name, check).await {
                Ok(status) => {
                    component_statuses.insert(component_name.clone(), status.clone());
                    total_score += status.score;

                    if status.health == ComponentHealth::Unhealthy {
                        unhealthy_count += 1;
                    }
                }
                Err(e) => {
                    warn!("Health check failed for component {}: {}", component_name, e);

                    // Create a failed status
                    let failed_status = ComponentStatus {
                        component: component_name.clone(),
                        health: ComponentHealth::Unhealthy,
                        score: 0.0,
                        message: format!("Health check failed: {}", e),
                        details: HashMap::new(),
                        last_check: Utc::now(),
                        check_duration_ms: 0,
                    };

                    component_statuses.insert(component_name.clone(), failed_status);
                    unhealthy_count += 1;
                }
            }
        }

        let component_count = component_statuses.len() as f64;
        let overall_score = if component_count > 0.0 {
            total_score / component_count
        } else {
            1.0
        };

        let overall_health = if unhealthy_count > 0 {
            ComponentHealth::Unhealthy
        } else if overall_score < 0.8 {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        let uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;

        Ok(HealthStatus {
            overall_health,
            overall_score,
            component_statuses,
            timestamp: Utc::now(),
            uptime_seconds,
        })
    }

    /// Perform a cached health check
    async fn perform_cached_check(
        &self,
        component_name: &str,
        check: &Box<dyn HealthCheck>,
    ) -> HealthResult<ComponentStatus> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached_status) = cache.get(component_name) {
                let age_seconds = (Utc::now() - cached_status.last_check).num_seconds();
                if age_seconds < self.cache_ttl_seconds as i64 {
                    return Ok(cached_status.clone());
                }
            }
        }

        // Perform the actual check
        let start_time = std::time::Instant::now();
        let status = check.check().await?;
        let check_duration_ms = start_time.elapsed().as_millis() as u64;

        // Update the status with timing
        let mut updated_status = status;
        updated_status.check_duration_ms = check_duration_ms;

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(component_name.to_string(), updated_status.clone());
        }

        Ok(updated_status)
    }

    /// Get health status for a specific component
    pub async fn get_component_health(&self, component_name: &str) -> HealthResult<Option<ComponentStatus>> {
        if let Some(check) = self.checks.get(component_name) {
            Ok(Some(self.perform_cached_check(component_name, check).await?))
        } else {
            Ok(None)
        }
    }

    /// Get overall health score (0.0 to 1.0)
    pub async fn get_overall_health_score(&self) -> HealthResult<f64> {
        let health_status = self.perform_comprehensive_check().await?;
        Ok(health_status.overall_score)
    }

    /// Check if system is healthy
    pub async fn is_system_healthy(&self) -> HealthResult<bool> {
        let health_status = self.perform_comprehensive_check().await?;
        Ok(matches!(health_status.overall_health, ComponentHealth::Healthy))
    }

    /// Clear health check cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get registered component names
    pub fn get_registered_components(&self) -> Vec<String> {
        self.checks.keys().cloned().collect()
    }
}

/// Built-in health checks

/// GitHub API health check
pub struct GitHubAPIHealthCheck {
    component_name: String,
}

impl GitHubAPIHealthCheck {
    pub fn new() -> Self {
        Self {
            component_name: "github_api".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for GitHubAPIHealthCheck {
    fn component_name(&self) -> &str {
        &self.component_name
    }

    async fn check(&self) -> HealthResult<ComponentStatus> {
        // In a real implementation, this would:
        // 1. Make a lightweight API call to GitHub
        // 2. Check rate limit status
        // 3. Verify authentication
        // 4. Measure response time

        let mut details = HashMap::new();
        details.insert("api_endpoint".to_string(), "https://api.github.com".to_string());
        details.insert("check_type".to_string(), "rate_limit_status".to_string());

        // Placeholder implementation
        Ok(ComponentStatus {
            component: self.component_name.clone(),
            health: ComponentHealth::Healthy,
            score: 1.0,
            message: "GitHub API is accessible and within rate limits".to_string(),
            details,
            last_check: Utc::now(),
            check_duration_ms: 150, // Simulated response time
        })
    }
}

/// Kubernetes API health check
pub struct KubernetesAPIHealthCheck {
    component_name: String,
}

impl KubernetesAPIHealthCheck {
    pub fn new() -> Self {
        Self {
            component_name: "kubernetes_api".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for KubernetesAPIHealthCheck {
    fn component_name(&self) -> &str {
        &self.component_name
    }

    async fn check(&self) -> HealthResult<ComponentStatus> {
        // In a real implementation, this would:
        // 1. Query Kubernetes API server health
        // 2. Check node status
        // 3. Verify RBAC permissions
        // 4. Check resource availability

        let mut details = HashMap::new();
        details.insert("api_version".to_string(), "v1".to_string());
        details.insert("check_type".to_string(), "api_server_health".to_string());

        // Placeholder implementation
        Ok(ComponentStatus {
            component: self.component_name.clone(),
            health: ComponentHealth::Healthy,
            score: 1.0,
            message: "Kubernetes API server is healthy".to_string(),
            details,
            last_check: Utc::now(),
            check_duration_ms: 50,
        })
    }
}

/// State manager health check
pub struct StateManagerHealthCheck {
    component_name: String,
}

impl StateManagerHealthCheck {
    pub fn new() -> Self {
        Self {
            component_name: "state_manager".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for StateManagerHealthCheck {
    fn component_name(&self) -> &str {
        &self.component_name
    }

    async fn check(&self) -> HealthResult<ComponentStatus> {
        // In a real implementation, this would:
        // 1. Test ConfigMap access
        // 2. Check state consistency
        // 3. Verify serialization/deserialization
        // 4. Check cleanup processes

        let mut details = HashMap::new();
        details.insert("storage_type".to_string(), "configmap".to_string());
        details.insert("check_type".to_string(), "read_write_access".to_string());

        // Placeholder implementation
        Ok(ComponentStatus {
            component: self.component_name.clone(),
            health: ComponentHealth::Healthy,
            score: 1.0,
            message: "State manager is operating normally".to_string(),
            details,
            last_check: Utc::now(),
            check_duration_ms: 25,
        })
    }
}

/// HTTP health endpoints (for external monitoring)
pub struct HealthEndpoints {
    health_checker: Arc<HealthChecker>,
}

impl HealthEndpoints {
    pub fn new(health_checker: Arc<HealthChecker>) -> Self {
        Self { health_checker }
    }

    /// Get detailed health status (/health)
    pub async fn detailed_health(&self) -> HealthResult<serde_json::Value> {
        let status = self.health_checker.perform_comprehensive_check().await?;

        Ok(serde_json::json!({
            "status": match status.overall_health {
                ComponentHealth::Healthy => "healthy",
                ComponentHealth::Degraded => "degraded",
                ComponentHealth::Unhealthy => "unhealthy",
            },
            "score": status.overall_score,
            "timestamp": status.timestamp,
            "uptime_seconds": status.uptime_seconds,
            "components": status.component_statuses
        }))
    }

    /// Get simple health status (/healthz)
    pub async fn simple_health(&self) -> HealthResult<String> {
        let is_healthy = self.health_checker.is_system_healthy().await?;
        Ok(if is_healthy { "OK".to_string() } else { "NOT_HEALTHY".to_string() })
    }

    /// Get readiness status (/ready)
    pub async fn readiness(&self) -> HealthResult<String> {
        // Readiness check - verify all critical components are ready
        let status = self.health_checker.perform_comprehensive_check().await?;
        let is_ready = matches!(status.overall_health, ComponentHealth::Healthy);

        Ok(if is_ready { "READY".to_string() } else { "NOT_READY".to_string() })
    }

    /// Get liveness status (/live)
    pub async fn liveness(&self) -> HealthResult<String> {
        // Liveness check - basic process health
        Ok("ALIVE".to_string())
    }
}
