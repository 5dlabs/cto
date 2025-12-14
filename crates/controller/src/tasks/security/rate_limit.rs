//! # Rate Limiting Implementation
//!
//! This module provides distributed rate limiting for the Agent Remediation Loop
//! to prevent abuse and ensure fair resource usage.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, info};

/// Rate limiting errors
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("Rate limiter error: {0}")]
    RateLimiterError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Result type for rate limiting operations
pub type RateLimitResult<T> = Result<T, RateLimitError>;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_capacity: u32,
    pub cleanup_interval_seconds: u64,
}

/// Rate limit entry for tracking usage
#[derive(Debug, Clone)]
struct RateLimitEntry {
    request_count: u32,
    window_start: DateTime<Utc>,
    last_request: DateTime<Utc>,
}

/// Rate limiter for controlling request frequency
pub struct RateLimiter {
    config: RateLimitConfig,
    limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    last_cleanup: Arc<RwLock<DateTime<Utc>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> RateLimitResult<Self> {
        Ok(Self {
            config: RateLimitConfig {
                requests_per_minute: 100,
                burst_capacity: 20,
                cleanup_interval_seconds: 300, // 5 minutes
            },
            limits: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Utc::now())),
        })
    }

    /// Check if a request is allowed
    pub async fn check_limit(&self, key: &str) -> RateLimitResult<()> {
        let mut limits = self.limits.write().await;
        let now = Utc::now();

        // Perform cleanup if needed
        self.perform_cleanup(&mut limits, now).await;

        // Get or create entry
        let entry = limits.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            request_count: 0,
            window_start: now,
            last_request: now - Duration::minutes(1), // Ensure first request is allowed
        });

        // Check if we need to reset the window
        if (now - entry.window_start).num_minutes() >= 1 {
            entry.request_count = 0;
            entry.window_start = now;
        }

        // Check rate limit
        if entry.request_count >= self.config.requests_per_minute {
            let reset_time = entry.window_start + Duration::minutes(1);
            let wait_seconds = (reset_time - now).num_seconds();

            return Err(RateLimitError::LimitExceeded(
                format!("Rate limit exceeded for key '{}'. Try again in {} seconds", key, wait_seconds)
            ));
        }

        // Check burst capacity (requests within short time window)
        let time_since_last_request = (now - entry.last_request).num_seconds();
        if time_since_last_request < 60 && entry.request_count >= self.config.requests_per_minute - self.config.burst_capacity {
            return Err(RateLimitError::LimitExceeded(
                format!("Burst rate limit exceeded for key '{}'", key)
            ));
        }

        // Allow request
        entry.request_count += 1;
        entry.last_request = now;

        debug!("Rate limit check passed for key: {} (count: {})", key, entry.request_count);
        Ok(())
    }

    /// Perform cleanup of old entries
    async fn perform_cleanup(&self, limits: &mut HashMap<String, RateLimitEntry>, now: DateTime<Utc>) {
        let mut last_cleanup = self.last_cleanup.write().await;

        if (*last_cleanup - now).num_seconds() < -(self.config.cleanup_interval_seconds as i64) {
            let before_cleanup = limits.len();

            // Remove entries older than 1 hour
            limits.retain(|_, entry| {
                (now - entry.last_request).num_hours() < 1
            });

            let after_cleanup = limits.len();
            let removed = before_cleanup - after_cleanup;

            if removed > 0 {
                debug!("Cleaned up {} old rate limit entries", removed);
            }

            *last_cleanup = now;
        }
    }

    /// Get current rate limit status for a key
    pub async fn get_limit_status(&self, key: &str) -> RateLimitResult<(u32, u32, i64)> {
        let limits = self.limits.read().await;

        if let Some(entry) = limits.get(key) {
            let remaining = if entry.request_count >= self.config.requests_per_minute {
                0
            } else {
                self.config.requests_per_minute - entry.request_count
            };

            let reset_seconds = if entry.request_count >= self.config.requests_per_minute {
                let reset_time = entry.window_start + Duration::minutes(1);
                (reset_time - Utc::now()).num_seconds()
            } else {
                60 - (Utc::now() - entry.window_start).num_seconds()
            };

            Ok((entry.request_count, remaining, reset_seconds))
        } else {
            Ok((0, self.config.requests_per_minute, 60))
        }
    }

    /// Reset rate limit for a specific key
    pub async fn reset_limit(&self, key: &str) -> RateLimitResult<()> {
        let mut limits = self.limits.write().await;
        limits.remove(key);
        debug!("Reset rate limit for key: {}", key);
        Ok(())
    }

    /// Update rate limit configuration
    pub fn update_config(&mut self, config: RateLimitConfig) -> RateLimitResult<()> {
        if config.requests_per_minute == 0 {
            return Err(RateLimitError::ConfigurationError(
                "Requests per minute cannot be zero".to_string()
            ));
        }

        if config.burst_capacity > config.requests_per_minute {
            return Err(RateLimitError::ConfigurationError(
                "Burst capacity cannot exceed requests per minute".to_string()
            ));
        }

        let requests_per_minute = config.requests_per_minute;
        let burst_capacity = config.burst_capacity;
        self.config = config;
        info!("Updated rate limit configuration: {} req/min, {} burst", requests_per_minute, burst_capacity);
        Ok(())
    }

    /// Get rate limiting statistics
    pub async fn get_statistics(&self) -> RateLimitResult<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        let limits = self.limits.read().await;

        stats.insert("active_keys".to_string(), limits.len() as u64);

        let total_requests: u64 = limits.values().map(|entry| entry.request_count as u64).sum();
        stats.insert("total_requests".to_string(), total_requests);

        let rate_limited_keys = limits.values()
            .filter(|entry| entry.request_count >= self.config.requests_per_minute)
            .count() as u64;
        stats.insert("rate_limited_keys".to_string(), rate_limited_keys);

        let avg_requests_per_key = if limits.is_empty() {
            0
        } else {
            total_requests / limits.len() as u64
        };
        stats.insert("avg_requests_per_key".to_string(), avg_requests_per_key);

        Ok(stats)
    }

    /// Check if rate limiter is healthy
    pub async fn is_healthy(&self) -> bool {
        // Check if we can acquire locks (basic health check)
        let limits_result = self.limits.try_read();
        let cleanup_result = self.last_cleanup.try_read();

        limits_result.is_ok() && cleanup_result.is_ok()
    }

    /// Get current configuration
    pub fn get_config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Force cleanup of all entries
    pub async fn force_cleanup(&self) {
        let mut limits = self.limits.write().await;
        let before_cleanup = limits.len();

        limits.clear();

        let mut last_cleanup = self.last_cleanup.write().await;
        *last_cleanup = Utc::now();

        info!("Force cleaned up {} rate limit entries", before_cleanup);
    }

    /// Create a custom rate limiter for specific use cases
    pub fn create_custom_limiter(config: RateLimitConfig) -> RateLimitResult<Self> {
        if config.requests_per_minute == 0 {
            return Err(RateLimitError::ConfigurationError(
                "Requests per minute cannot be zero".to_string()
            ));
        }

        Ok(Self {
            config,
            limits: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Utc::now())),
        })
    }

    /// Check multiple keys at once (batch operation)
    pub async fn check_limits_batch(&self, keys: &[String]) -> RateLimitResult<Vec<Result<(), RateLimitError>>> {
        let mut results = Vec::new();

        for key in keys {
            let result = self.check_limit(key).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Get rate limit status for multiple keys
    pub async fn get_limits_status_batch(&self, keys: &[String]) -> RateLimitResult<HashMap<String, (u32, u32, i64)>> {
        let mut results = HashMap::new();

        for key in keys {
            if let Ok(status) = self.get_limit_status(key).await {
                results.insert(key.clone(), status);
            }
        }

        Ok(results)
    }
}

/// Distributed rate limiter using Kubernetes leases (for future implementation)
pub struct DistributedRateLimiter {
    base_limiter: RateLimiter,
    lease_name: String,
    lease_namespace: String,
}

impl DistributedRateLimiter {
    /// Create a new distributed rate limiter
    pub fn new(lease_name: String, lease_namespace: String) -> RateLimitResult<Self> {
        Ok(Self {
            base_limiter: RateLimiter::new()?,
            lease_name,
            lease_namespace,
        })
    }

    /// Check limit with distributed coordination
    pub async fn check_distributed_limit(&self, key: &str) -> RateLimitResult<()> {
        // In a real implementation, this would:
        // 1. Acquire a distributed lock using Kubernetes leases
        // 2. Check/update shared rate limit state
        // 3. Release the lock

        // For now, delegate to the base limiter
        self.base_limiter.check_limit(key).await
    }

    /// Get distributed rate limit status
    pub async fn get_distributed_status(&self, key: &str) -> RateLimitResult<(u32, u32, i64)> {
        // In a real implementation, this would check distributed state
        self.base_limiter.get_limit_status(key).await
    }
}
