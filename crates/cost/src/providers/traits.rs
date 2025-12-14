//! Cost provider trait and common types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during cost provider operations.
#[derive(Error, Debug)]
pub enum CostProviderError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Authentication error (missing or invalid API key).
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid time range.
    #[error("Invalid time range: {0}")]
    InvalidTimeRange(String),
}

// ============================================================================
// Time bucket granularity
// ============================================================================

/// Time bucket width for aggregating usage/cost data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BucketWidth {
    /// 1 minute buckets (for real-time monitoring).
    #[serde(rename = "1m")]
    Minute,
    /// 1 hour buckets (for daily patterns).
    #[serde(rename = "1h")]
    Hour,
    /// 1 day buckets (for weekly/monthly reports).
    #[serde(rename = "1d")]
    #[default]
    Day,
}

impl std::fmt::Display for BucketWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minute => write!(f, "1m"),
            Self::Hour => write!(f, "1h"),
            Self::Day => write!(f, "1d"),
        }
    }
}

// ============================================================================
// Usage types
// ============================================================================

/// Token usage statistics for a time bucket.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Total input tokens used.
    pub input_tokens: i64,
    /// Total output tokens used.
    pub output_tokens: i64,
    /// Cached input tokens (from prompt caching).
    #[serde(default)]
    pub cached_input_tokens: i64,
    /// Cache creation tokens (tokens stored in cache).
    #[serde(default)]
    pub cache_creation_tokens: i64,
    /// Audio input tokens (for multimodal APIs).
    #[serde(default)]
    pub audio_input_tokens: i64,
    /// Audio output tokens (for multimodal APIs).
    #[serde(default)]
    pub audio_output_tokens: i64,
    /// Number of API requests made.
    pub num_requests: i64,
}

impl TokenUsage {
    /// Calculate total tokens (input + output).
    #[must_use]
    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }

    /// Calculate billable input tokens (excluding cached tokens).
    #[must_use]
    pub fn billable_input_tokens(&self) -> i64 {
        self.input_tokens - self.cached_input_tokens
    }
}

/// Usage data aggregated by a specific dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBucket {
    /// Start time of the bucket (inclusive).
    pub start_time: DateTime<Utc>,
    /// End time of the bucket (exclusive).
    pub end_time: DateTime<Utc>,
    /// Token usage for this bucket.
    pub usage: TokenUsage,
    /// Model name (if grouped by model).
    pub model: Option<String>,
    /// Project/workspace ID (if grouped by project).
    pub project_id: Option<String>,
    /// API key ID (if grouped by API key).
    pub api_key_id: Option<String>,
    /// User ID (if grouped by user).
    pub user_id: Option<String>,
    /// Service tier (e.g., "default", "batch", "priority").
    pub service_tier: Option<String>,
}

/// Request for fetching usage data.
#[derive(Debug, Clone, Default)]
pub struct UsageRequest {
    /// Start time (inclusive).
    pub start_time: DateTime<Utc>,
    /// End time (exclusive).
    pub end_time: DateTime<Utc>,
    /// Time bucket width.
    pub bucket_width: BucketWidth,
    /// Fields to group by.
    pub group_by: Vec<GroupByField>,
    /// Filter by specific models.
    pub models: Option<Vec<String>>,
    /// Filter by specific project/workspace IDs.
    pub project_ids: Option<Vec<String>>,
    /// Filter by specific API key IDs.
    pub api_key_ids: Option<Vec<String>>,
    /// Filter by specific user IDs.
    pub user_ids: Option<Vec<String>>,
    /// Maximum number of buckets to return.
    pub limit: Option<i32>,
    /// Pagination cursor.
    pub page: Option<String>,
}

/// Fields that can be used for grouping usage/cost data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupByField {
    /// Group by model name.
    Model,
    /// Group by project/workspace ID.
    ProjectId,
    /// Group by API key ID.
    ApiKeyId,
    /// Group by user ID.
    UserId,
    /// Group by service tier.
    ServiceTier,
    /// Group by line item (for costs).
    LineItem,
}

impl std::fmt::Display for GroupByField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Model => write!(f, "model"),
            Self::ProjectId => write!(f, "project_id"),
            Self::ApiKeyId => write!(f, "api_key_id"),
            Self::UserId => write!(f, "user_id"),
            Self::ServiceTier => write!(f, "service_tier"),
            Self::LineItem => write!(f, "line_item"),
        }
    }
}

/// Response containing usage data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    /// Usage data buckets.
    pub buckets: Vec<UsageBucket>,
    /// Whether there are more pages.
    pub has_more: bool,
    /// Cursor for the next page.
    pub next_page: Option<String>,
}

// ============================================================================
// Cost types
// ============================================================================

/// Currency code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    /// US Dollars.
    #[default]
    Usd,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usd => write!(f, "USD"),
        }
    }
}

/// Monetary amount with currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount {
    /// The value in the smallest unit (e.g., cents for USD).
    pub value: f64,
    /// The currency.
    pub currency: Currency,
}

impl Amount {
    /// Create a new amount in USD.
    #[must_use]
    pub fn usd(value: f64) -> Self {
        Self {
            value,
            currency: Currency::Usd,
        }
    }

    /// Format as a dollar string (e.g., "$1.23").
    #[must_use]
    pub fn format_dollars(&self) -> String {
        format!("${:.2}", self.value)
    }
}

/// Cost data for a time bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBucket {
    /// Start time of the bucket (inclusive).
    pub start_time: DateTime<Utc>,
    /// End time of the bucket (exclusive).
    pub end_time: DateTime<Utc>,
    /// Total cost amount.
    pub amount: Amount,
    /// Project/workspace ID (if grouped by project).
    pub project_id: Option<String>,
    /// Line item description (if grouped by line item).
    pub line_item: Option<String>,
    /// Model name (if available).
    pub model: Option<String>,
}

/// Request for fetching cost data.
#[derive(Debug, Clone, Default)]
pub struct CostRequest {
    /// Start time (inclusive).
    pub start_time: DateTime<Utc>,
    /// End time (exclusive).
    pub end_time: DateTime<Utc>,
    /// Fields to group by.
    pub group_by: Vec<GroupByField>,
    /// Filter by specific project/workspace IDs.
    pub project_ids: Option<Vec<String>>,
    /// Maximum number of buckets to return.
    pub limit: Option<i32>,
    /// Pagination cursor.
    pub page: Option<String>,
}

/// Response containing cost data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostResponse {
    /// Cost data buckets.
    pub buckets: Vec<CostBucket>,
    /// Whether there are more pages.
    pub has_more: bool,
    /// Cursor for the next page.
    pub next_page: Option<String>,
}

// ============================================================================
// Summary types (for convenience)
// ============================================================================

/// Summary of usage and costs over a time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Start time of the summary period.
    pub start_time: DateTime<Utc>,
    /// End time of the summary period.
    pub end_time: DateTime<Utc>,
    /// Total token usage.
    pub total_usage: TokenUsage,
    /// Total cost.
    pub total_cost: Amount,
    /// Usage breakdown by model.
    pub by_model: Vec<ModelUsageSummary>,
}

/// Usage summary for a specific model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageSummary {
    /// Model name.
    pub model: String,
    /// Token usage for this model.
    pub usage: TokenUsage,
    /// Estimated cost for this model.
    pub estimated_cost: Amount,
}

// ============================================================================
// Provider trait
// ============================================================================

/// Trait for AI/LLM cost providers.
///
/// Implementations should handle authentication and API communication
/// for their respective providers.
#[async_trait]
pub trait CostProvider: Send + Sync {
    /// Get the provider name (e.g., "openai", "anthropic").
    fn name(&self) -> &'static str;

    /// Fetch usage data for a time range.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or authentication fails.
    async fn get_usage(&self, request: UsageRequest) -> Result<UsageResponse, CostProviderError>;

    /// Fetch cost data for a time range.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or authentication fails.
    async fn get_costs(&self, request: CostRequest) -> Result<CostResponse, CostProviderError>;

    /// Get a summary of usage and costs for a time range.
    ///
    /// This is a convenience method that combines usage and cost data.
    /// The default implementation makes separate calls to `get_usage` and `get_costs`.
    ///
    /// # Errors
    ///
    /// Returns an error if the API requests fail.
    async fn get_summary(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<UsageSummary, CostProviderError> {
        // Default implementation: get usage grouped by model
        let usage_request = UsageRequest {
            start_time,
            end_time,
            bucket_width: BucketWidth::Day,
            group_by: vec![GroupByField::Model],
            ..Default::default()
        };
        let usage_response = self.get_usage(usage_request).await?;

        // Get total costs
        let cost_request = CostRequest {
            start_time,
            end_time,
            ..Default::default()
        };
        let cost_response = self.get_costs(cost_request).await?;

        // Aggregate usage
        let mut total_usage = TokenUsage::default();
        let mut by_model: std::collections::HashMap<String, TokenUsage> =
            std::collections::HashMap::new();

        for bucket in &usage_response.buckets {
            total_usage.input_tokens += bucket.usage.input_tokens;
            total_usage.output_tokens += bucket.usage.output_tokens;
            total_usage.cached_input_tokens += bucket.usage.cached_input_tokens;
            total_usage.num_requests += bucket.usage.num_requests;

            if let Some(model) = &bucket.model {
                let model_usage = by_model.entry(model.clone()).or_default();
                model_usage.input_tokens += bucket.usage.input_tokens;
                model_usage.output_tokens += bucket.usage.output_tokens;
                model_usage.cached_input_tokens += bucket.usage.cached_input_tokens;
                model_usage.num_requests += bucket.usage.num_requests;
            }
        }

        // Aggregate costs
        let total_cost = cost_response
            .buckets
            .iter()
            .fold(0.0, |acc, bucket| acc + bucket.amount.value);

        // Convert by_model to ModelUsageSummary (cost estimation would require pricing data)
        let by_model_summary: Vec<ModelUsageSummary> = by_model
            .into_iter()
            .map(|(model, usage)| ModelUsageSummary {
                model,
                usage,
                estimated_cost: Amount::usd(0.0), // Cost breakdown by model requires pricing data
            })
            .collect();

        Ok(UsageSummary {
            start_time,
            end_time,
            total_usage,
            total_cost: Amount::usd(total_cost),
            by_model: by_model_summary,
        })
    }
}
