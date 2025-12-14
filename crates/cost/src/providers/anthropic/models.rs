//! Anthropic API response models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Usage report response
// ============================================================================

/// Paginated response from Anthropic Usage API.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicUsageResponse {
    /// Usage data buckets.
    pub data: Vec<AnthropicUsageBucket>,
    /// Whether there are more pages.
    pub has_more: bool,
    /// Cursor for the next page.
    pub next_page: Option<String>,
}

/// Usage bucket from Anthropic API.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicUsageBucket {
    /// Snapshot timestamp for this bucket.
    pub snapshot_at: DateTime<Utc>,
    /// Bucket start time.
    pub bucket_start_time: DateTime<Utc>,
    /// Bucket end time.
    pub bucket_end_time: DateTime<Utc>,
    /// Request count.
    pub request_count: i64,
    /// Input tokens (excluding cached).
    pub input_tokens: i64,
    /// Output tokens.
    pub output_tokens: i64,
    /// Cached read input tokens.
    #[serde(default)]
    pub cache_read_input_tokens: i64,
    /// Cache creation input tokens.
    #[serde(default)]
    pub cache_creation_input_tokens: i64,
    /// Model name (if grouped by model).
    pub model: Option<String>,
    /// Workspace ID (if grouped by workspace).
    pub workspace_id: Option<String>,
    /// API key ID (if grouped by `api_key`).
    pub api_key_id: Option<String>,
    /// Service tier (if grouped by `service_tier`).
    pub service_tier: Option<String>,
    /// Context window (if grouped).
    pub context_window: Option<String>,
}

// ============================================================================
// Cost report response
// ============================================================================

/// Paginated response from Anthropic Cost API.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicCostResponse {
    /// Cost data buckets.
    pub data: Vec<AnthropicCostBucket>,
    /// Whether there are more pages.
    pub has_more: bool,
    /// Cursor for the next page.
    pub next_page: Option<String>,
}

/// Cost bucket from Anthropic API.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicCostBucket {
    /// Snapshot timestamp.
    pub snapshot_at: DateTime<Utc>,
    /// Bucket start time.
    pub bucket_start_time: DateTime<Utc>,
    /// Bucket end time.
    pub bucket_end_time: DateTime<Utc>,
    /// Cost in USD (as string in smallest unit - cents).
    pub cost_usd: String,
    /// Workspace ID (if grouped).
    pub workspace_id: Option<String>,
    /// Description/line item (if grouped).
    pub description: Option<String>,
}

// ============================================================================
// Error response
// ============================================================================

/// Error response from Anthropic API.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicError {
    /// Error type.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error message.
    pub message: String,
}

// ============================================================================
// Request parameter types
// ============================================================================

/// Group by options for usage report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnthropicUsageGroupBy {
    /// Group by model.
    Model,
    /// Group by workspace.
    WorkspaceId,
    /// Group by API key.
    ApiKeyId,
    /// Group by service tier.
    ServiceTier,
    /// Group by context window.
    ContextWindow,
}

impl std::fmt::Display for AnthropicUsageGroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Model => write!(f, "model"),
            Self::WorkspaceId => write!(f, "workspace_id"),
            Self::ApiKeyId => write!(f, "api_key_id"),
            Self::ServiceTier => write!(f, "service_tier"),
            Self::ContextWindow => write!(f, "context_window"),
        }
    }
}

/// Group by options for cost report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnthropicCostGroupBy {
    /// Group by workspace.
    WorkspaceId,
    /// Group by description.
    Description,
}

impl std::fmt::Display for AnthropicCostGroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceId => write!(f, "workspace_id"),
            Self::Description => write!(f, "description"),
        }
    }
}
