//! # GitHub Label API Client
//!
//! This module provides a comprehensive GitHub API client for label operations,
//! including rate limiting, retry logic, and atomic operations using `ETags`.

use crate::tasks::label::schema::{LabelOperation, LabelOperationType};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client};
use reqwest::{header, Client as HttpClient, Response};
use serde::{Deserialize, Serialize};

use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, info, instrument, warn};

/// GitHub API client for label operations
#[derive(Clone)]
pub struct GitHubLabelClient {
    http_client: HttpClient,
    base_url: String,
    token: String,
    owner: String,
    repo: String,
    rate_limit_remaining: i32,
    rate_limit_reset: Option<Instant>,
}

#[derive(Debug, Error)]
pub enum GitHubLabelError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("GitHub API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Rate limit exceeded, reset in {reset_in:?}")]
    RateLimitExceeded { reset_in: Duration },

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Label operation failed: {0}")]
    OperationFailed(String),

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubError {
    message: String,
    documentation_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRateLimit {
    limit: i32,
    remaining: i32,
    reset: i64,
}

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPR {
    labels: Vec<GitHubLabel>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct UpdateLabelsRequest {
    labels: Vec<String>,
}

impl GitHubLabelClient {
    /// Create a new GitHub label client
    pub async fn new(
        client: Client,
        namespace: &str,
        secret_name: &str,
        owner: String,
        repo: String,
    ) -> Result<Self, GitHubLabelError> {
        let secrets: Api<Secret> = Api::namespaced(client, namespace);

        let secret = secrets.get(secret_name).await.map_err(|e| {
            GitHubLabelError::OperationFailed(format!("Failed to get GitHub token secret: {e}"))
        })?;

        let token_data = secret
            .data
            .as_ref()
            .and_then(|data| data.get("token"))
            .and_then(|token_bytes| String::from_utf8(token_bytes.0.clone()).ok())
            .ok_or_else(|| GitHubLabelError::AuthenticationFailed)?;

        let http_client = HttpClient::builder()
            .user_agent("cto-agent-remediation-loop/1.0")
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            http_client,
            base_url: "https://api.github.com".to_string(),
            token: token_data,
            owner,
            repo,
            rate_limit_remaining: 5000, // GitHub's default rate limit
            rate_limit_reset: None,
        })
    }

    /// Create a client with a direct token (for testing)
    #[must_use]
    pub fn with_token(token: String, owner: String, repo: String) -> Self {
        let http_client = HttpClient::builder()
            .user_agent("cto-agent-remediation-loop/1.0")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            base_url: "https://api.github.com".to_string(),
            token,
            owner,
            repo,
            rate_limit_remaining: 5000,
            rate_limit_reset: None,
        }
    }

    /// Get all labels for a PR
    #[instrument(skip(self), fields(pr_number = %pr_number))]
    pub async fn get_labels(&mut self, pr_number: i32) -> Result<Vec<String>, GitHubLabelError> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.base_url, self.owner, self.repo, pr_number
        );

        let response = self.make_request(reqwest::Method::GET, &url, None).await?;
        let pr: GitHubPR = response.json().await?;

        let labels: Vec<String> = pr.labels.into_iter().map(|label| label.name).collect();

        debug!("Retrieved {} labels for PR #{}", labels.len(), pr_number);
        Ok(labels)
    }

    /// Add labels to a PR
    #[instrument(skip(self), fields(pr_number = %pr_number, labels = ?labels))]
    pub async fn add_labels(
        &mut self,
        pr_number: i32,
        labels: &[String],
    ) -> Result<(), GitHubLabelError> {
        if labels.is_empty() {
            return Ok(());
        }

        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels",
            self.base_url, self.owner, self.repo, pr_number
        );

        let body = serde_json::json!({ "labels": labels });
        let response = self
            .make_request(reqwest::Method::POST, &url, Some(body))
            .await?;
        let status = response.status();

        if status.is_success() {
            info!("Added {} labels to PR #{}", labels.len(), pr_number);
            Ok(())
        } else {
            let error: GitHubError = response.json().await?;
            Err(GitHubLabelError::ApiError {
                status: status.as_u16(),
                message: error.message,
            })
        }
    }

    /// Remove a label from a PR
    #[instrument(skip(self), fields(pr_number = %pr_number, label = %label))]
    pub async fn remove_label(
        &mut self,
        pr_number: i32,
        label: &str,
    ) -> Result<(), GitHubLabelError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels/{}",
            self.base_url, self.owner, self.repo, pr_number, label
        );

        let response = self
            .make_request(reqwest::Method::DELETE, &url, None)
            .await?;

        match response.status().as_u16() {
            204 => {
                debug!("Removed label '{}' from PR #{}", label, pr_number);
                Ok(())
            }
            404 => {
                // Label doesn't exist, which is fine for removal
                debug!(
                    "Label '{}' not found on PR #{} (already removed)",
                    label, pr_number
                );
                Ok(())
            }
            status => {
                let error: GitHubError = response.json().await?;
                Err(GitHubLabelError::ApiError {
                    status,
                    message: error.message,
                })
            }
        }
    }

    /// Replace all labels on a PR
    #[instrument(skip(self), fields(pr_number = %pr_number, labels = ?labels))]
    pub async fn replace_labels(
        &mut self,
        pr_number: i32,
        labels: &[String],
    ) -> Result<(), GitHubLabelError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels",
            self.base_url, self.owner, self.repo, pr_number
        );

        let body = serde_json::json!({ "labels": labels });
        let response = self
            .make_request(reqwest::Method::PUT, &url, Some(body))
            .await?;
        let status = response.status();

        if status.is_success() {
            info!(
                "Replaced all labels on PR #{} with {} labels",
                pr_number,
                labels.len()
            );
            Ok(())
        } else {
            let error: GitHubError = response.json().await?;
            Err(GitHubLabelError::ApiError {
                status: status.as_u16(),
                message: error.message,
            })
        }
    }

    /// Perform atomic label operations with retry logic
    #[instrument(skip(self, operations), fields(pr_number = %pr_number, operations_count = %operations.len()))]
    pub async fn update_labels_atomic(
        &mut self,
        pr_number: i32,
        operations: &[LabelOperation],
    ) -> Result<(), GitHubLabelError> {
        let max_retries = 5;
        let mut last_error: Option<GitHubLabelError> = None;

        for attempt in 1..=max_retries {
            match self.try_atomic_update(pr_number, operations).await {
                Ok(()) => {
                    if attempt > 1 {
                        info!(
                            "Atomic label update succeeded on attempt {} for PR #{}",
                            attempt, pr_number
                        );
                    }
                    return Ok(());
                }
                Err(GitHubLabelError::ConcurrentModification) => {
                    if attempt < max_retries {
                        let backoff_ms = (1000 * 2_i32.pow(attempt - 1)).min(5000);
                        warn!(
                            "Concurrent modification detected, retrying in {}ms (attempt {}/{})",
                            backoff_ms, attempt, max_retries
                        );
                        #[allow(clippy::cast_sign_loss)]
                        sleep(Duration::from_millis(backoff_ms as u64)).await;
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    break;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            GitHubLabelError::OperationFailed("Atomic update failed after all retries".to_string())
        }))
    }

    /// Attempt a single atomic update
    async fn try_atomic_update(
        &mut self,
        pr_number: i32,
        operations: &[LabelOperation],
    ) -> Result<(), GitHubLabelError> {
        // Get current state with ETag
        let (current_labels, etag) = self.get_labels_with_etag(pr_number).await?;

        // Calculate new labels based on operations
        let new_labels = Self::calculate_new_labels(&current_labels, operations);

        // Attempt atomic update
        let url = format!(
            "{}/repos/{}/{}/issues/{}/labels",
            self.base_url, self.owner, self.repo, pr_number
        );

        let body = serde_json::json!({ "labels": new_labels });

        let response = self
            .http_client
            .put(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::IF_MATCH, etag)
            .json(&body)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                debug!("Atomic label update succeeded for PR #{}", pr_number);
                Ok(())
            }
            412 => {
                // Precondition failed - concurrent modification
                Err(GitHubLabelError::ConcurrentModification)
            }
            422 => {
                // Validation failed - likely due to label name constraints
                let error: GitHubError = response.json().await?;
                Err(GitHubLabelError::ApiError {
                    status: 422,
                    message: format!("Label validation failed: {}", error.message),
                })
            }
            status => {
                let error: GitHubError = response.json().await?;
                Err(GitHubLabelError::ApiError {
                    status,
                    message: error.message,
                })
            }
        }
    }

    /// Get labels with `ETag` for conditional requests
    async fn get_labels_with_etag(
        &mut self,
        pr_number: i32,
    ) -> Result<(Vec<String>, String), GitHubLabelError> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.base_url, self.owner, self.repo, pr_number
        );

        let response = self
            .http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error: GitHubError = response.json().await?;
            return Err(GitHubLabelError::ApiError {
                status: status.as_u16(),
                message: error.message,
            });
        }

        let etag = response
            .headers()
            .get(header::ETAG)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        let pr: GitHubPR = response.json().await?;
        let labels: Vec<String> = pr.labels.into_iter().map(|label| label.name).collect();

        Ok((labels, etag))
    }

    /// Calculate new labels after applying operations
    fn calculate_new_labels(current: &[String], operations: &[LabelOperation]) -> Vec<String> {
        let mut labels: std::collections::HashSet<String> = current.iter().cloned().collect();

        for operation in operations {
            match operation.operation_type {
                LabelOperationType::Add => {
                    for label in &operation.labels {
                        labels.insert(label.clone());
                    }
                }
                LabelOperationType::Remove => {
                    for label in &operation.labels {
                        labels.remove(label);
                    }
                }
                LabelOperationType::Replace => {
                    if let Some(from_label) = &operation.from_label {
                        labels.remove(from_label);
                    }
                    for label in &operation.labels {
                        labels.insert(label.clone());
                    }
                }
            }
        }

        let mut result: Vec<String> = labels.into_iter().collect();
        result.sort();
        result
    }

    /// Make an HTTP request with rate limiting and retry logic
    async fn make_request(
        &mut self,
        method: reqwest::Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response, GitHubLabelError> {
        // Check rate limit
        self.check_rate_limit()?;

        let mut request = self
            .http_client
            .request(method, url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::CONTENT_TYPE, "application/json");

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        self.update_rate_limit(&response);

        if response.status().as_u16() == 403 {
            // Check if it's a rate limit error
            if let Some(reset_time) = Self::get_rate_limit_reset(&response) {
                return Err(GitHubLabelError::RateLimitExceeded {
                    reset_in: reset_time,
                });
            }
        }

        Ok(response)
    }

    /// Check if we're within rate limits
    fn check_rate_limit(&mut self) -> Result<(), GitHubLabelError> {
        if let Some(reset_time) = self.rate_limit_reset {
            if Instant::now() < reset_time {
                let remaining = reset_time - Instant::now();
                return Err(GitHubLabelError::RateLimitExceeded {
                    reset_in: remaining,
                });
            }
        }

        if self.rate_limit_remaining <= 0 {
            return Err(GitHubLabelError::RateLimitExceeded {
                reset_in: Duration::from_secs(60), // Conservative fallback
            });
        }

        Ok(())
    }

    /// Update rate limit tracking from response headers
    fn update_rate_limit(&mut self, response: &Response) {
        if let Some(remaining) = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<i32>().ok())
        {
            self.rate_limit_remaining = remaining;
        }

        if let Some(reset) = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
        {
            let now = chrono::Utc::now().timestamp();
            #[allow(clippy::cast_sign_loss)]
            let seconds_until_reset = (reset - now).max(0) as u64;
            self.rate_limit_reset = Some(Instant::now() + Duration::from_secs(seconds_until_reset));
        }
    }

    /// Extract rate limit reset time from response
    fn get_rate_limit_reset(response: &Response) -> Option<Duration> {
        response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .map(|reset_timestamp| {
                let now = chrono::Utc::now().timestamp();
                #[allow(clippy::cast_sign_loss)]
                let seconds_until_reset = (reset_timestamp - now).max(0) as u64;
                Duration::from_secs(seconds_until_reset)
            })
    }
}
