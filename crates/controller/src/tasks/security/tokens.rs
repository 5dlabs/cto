//! # GitHub Token Management
//!
//! This module handles GitHub token validation, permission checking,
//! and secure token management for the Agent Remediation Loop.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};

/// Token management errors
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token validation error: {0}")]
    ValidationError(String),

    #[error("Token expired: {0}")]
    TokenExpired(String),

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("API rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Result type for token operations
pub type TokenResult<T> = Result<T, TokenError>;

/// GitHub token permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPermissions {
    pub contents: String,      // read access for repository contents
    pub issues: String,        // write access for issue/PR interactions
    pub pull_requests: String, // write access for PR management
    pub metadata: String,      // read access for repository metadata
    pub statuses: String,      // read access for commit/PR status checks
    pub discussions: String,   // write access for discussion management
}

/// GitHub token information
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token_id: String,
    pub permissions: GitHubPermissions,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit_remaining: i32,
    pub rate_limit_reset: DateTime<Utc>,
    pub last_validated: DateTime<Utc>,
}

/// GitHub token manager
pub struct GitHubTokenManager {
    token_cache: std::sync::Mutex<HashMap<String, TokenInfo>>,
    required_permissions: GitHubPermissions,
    validation_interval_seconds: u64,
    rate_limit_threshold: i32,
}

impl GitHubTokenManager {
    /// Create a new GitHub token manager
    pub fn new() -> TokenResult<Self> {
        let required_permissions = GitHubPermissions {
            contents: "read".to_string(),
            issues: "write".to_string(),
            pull_requests: "write".to_string(),
            metadata: "read".to_string(),
            statuses: "read".to_string(),
            discussions: "write".to_string(),
        };

        Ok(Self {
            token_cache: std::sync::Mutex::new(HashMap::new()),
            required_permissions,
            validation_interval_seconds: 300, // 5 minutes
            rate_limit_threshold: 100, // Minimum remaining calls before warning
        })
    }

    /// Validate token permissions
    pub async fn validate_token_permissions(&self) -> TokenResult<()> {
        // Get token from environment (in production, this would be from secure storage)
        let token = self.get_github_token()?;

        // Check cache first
        if let Some(cached_info) = self.get_cached_token_info(&token) {
            if self.is_cache_valid(&cached_info) {
                return self.validate_cached_permissions(&cached_info);
            }
        }

        // Perform fresh validation
        let token_info = self.validate_token_fresh(&token).await?;
        self.cache_token_info(token, token_info);

        Ok(())
    }

    /// Get GitHub token from secure storage
    fn get_github_token(&self) -> TokenResult<String> {
        // In production, this would retrieve from:
        // 1. Kubernetes secrets
        // 2. OpenBao/secure key management
        // 3. Environment variables (development only)

        std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .map_err(|_| TokenError::ValidationError(
                "GitHub token not found in environment variables".to_string()
            ))
    }

    /// Validate token with fresh API call
    async fn validate_token_fresh(&self, token: &str) -> TokenResult<TokenInfo> {
        // In a real implementation, this would:
        // 1. Make API call to /user to verify token
        // 2. Check /rate_limit for current limits
        // 3. Validate specific permissions

        // For now, simulate validation
        info!("Validating GitHub token permissions");

        let permissions = GitHubPermissions {
            contents: "read".to_string(),
            issues: "write".to_string(),
            pull_requests: "write".to_string(),
            metadata: "read".to_string(),
            statuses: "read".to_string(),
            discussions: "write".to_string(),
        };

        let token_info = TokenInfo {
            token_id: format!("token-{}", &token[..8]), // Masked for security
            permissions,
            expires_at: None, // GitHub tokens don't expire by default
            rate_limit_remaining: 4990, // Simulate remaining calls
            rate_limit_reset: Utc::now() + chrono::Duration::hours(1),
            last_validated: Utc::now(),
        };

        Ok(token_info)
    }

    /// Validate cached token permissions
    fn validate_cached_permissions(&self, token_info: &TokenInfo) -> TokenResult<()> {
        // Check if token has required permissions
        self.check_permissions(&token_info.permissions)?;

        // Check rate limit status
        if token_info.rate_limit_remaining < self.rate_limit_threshold {
            warn!(
                "GitHub API rate limit low: {} remaining",
                token_info.rate_limit_remaining
            );
        }

        Ok(())
    }

    /// Check if token has required permissions
    fn check_permissions(&self, permissions: &GitHubPermissions) -> TokenResult<()> {
        let mut missing_permissions = Vec::new();

        if permissions.contents != "read" {
            missing_permissions.push("contents:read");
        }
        if permissions.issues != "write" {
            missing_permissions.push("issues:write");
        }
        if permissions.pull_requests != "write" {
            missing_permissions.push("pull_requests:write");
        }
        if permissions.metadata != "read" {
            missing_permissions.push("metadata:read");
        }
        if permissions.statuses != "read" {
            missing_permissions.push("statuses:read");
        }
        if permissions.discussions != "write" {
            missing_permissions.push("discussions:write");
        }

        if missing_permissions.is_empty() {
            Ok(())
        } else {
            Err(TokenError::InsufficientPermissions(
                format!("Missing permissions: {}", missing_permissions.join(", "))
            ))
        }
    }

    /// Check if token is expired
    fn is_token_expired(&self, token_info: &TokenInfo) -> bool {
        if let Some(expires_at) = token_info.expires_at {
            Utc::now() > expires_at
        } else {
            false // GitHub tokens don't expire by default
        }
    }

    /// Check if cache entry is still valid
    fn is_cache_valid(&self, token_info: &TokenInfo) -> bool {
        let age = Utc::now() - token_info.last_validated;
        let max_age = std::time::Duration::from_secs(self.validation_interval_seconds);

        !self.is_token_expired(token_info) && age.to_std().unwrap_or(max_age) < max_age
    }

    /// Get cached token information
    fn get_cached_token_info(&self, token: &str) -> Option<TokenInfo> {
        let cache = self.token_cache.lock().unwrap();
        let token_key = format!("token-{}", &token[..8]);
        cache.get(&token_key).cloned()
    }

    /// Cache token information
    fn cache_token_info(&self, token: String, info: TokenInfo) {
        let mut cache = self.token_cache.lock().unwrap();
        let token_key = format!("token-{}", &token[..8]);
        cache.insert(token_key, info);
    }

    /// Get token statistics
    pub async fn get_statistics(&self) -> TokenResult<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        let cache = self.token_cache.lock().unwrap();

        stats.insert("cached_tokens".to_string(), cache.len() as u64);

        let expired_count = cache.values()
            .filter(|info| self.is_token_expired(info))
            .count() as u64;
        stats.insert("expired_tokens".to_string(), expired_count);

        let low_rate_limit_count = cache.values()
            .filter(|info| info.rate_limit_remaining < self.rate_limit_threshold)
            .count() as u64;
        stats.insert("tokens_low_rate_limit".to_string(), low_rate_limit_count);

        Ok(stats)
    }

    /// Check if token manager is healthy
    pub async fn is_healthy(&self) -> bool {
        // Check if we can access token
        self.get_github_token().is_ok()
    }

    /// Rotate token (invalidate cache)
    pub fn rotate_token(&self) {
        let mut cache = self.token_cache.lock().unwrap();
        cache.clear();
        info!("Token cache cleared for rotation");
    }

    /// Get current rate limit status
    pub async fn get_rate_limit_status(&self) -> TokenResult<(i32, DateTime<Utc>)> {
        let token = self.get_github_token()?;

        if let Some(cached_info) = self.get_cached_token_info(&token) {
            if self.is_cache_valid(&cached_info) {
                return Ok((cached_info.rate_limit_remaining, cached_info.rate_limit_reset));
            }
        }

        // Fallback to default values if not cached
        Ok((5000, Utc::now() + chrono::Duration::hours(1)))
    }

    /// Test token permissions (non-destructive)
    pub async fn test_token_permissions(&self) -> TokenResult<Vec<String>> {
        let token = self.get_github_token()?;

        // In a real implementation, this would make a test API call
        // to verify permissions without side effects

        info!("Testing GitHub token permissions for token: {}", &token[..8]);

        // Simulate permission test results
        let permissions = vec![
            "contents:read".to_string(),
            "issues:write".to_string(),
            "pull_requests:write".to_string(),
            "metadata:read".to_string(),
            "statuses:read".to_string(),
            "discussions:write".to_string(),
        ];

        Ok(permissions)
    }
}
