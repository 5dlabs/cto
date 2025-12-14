//! OpenAI API response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// Common response wrapper
// ============================================================================

/// Paginated response from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiPage<T> {
    /// Response object type (e.g., "page").
    pub object: String,
    /// Data items.
    pub data: Vec<T>,
    /// Whether there are more pages.
    pub has_more: bool,
    /// Cursor for the next page.
    pub next_page: Option<String>,
}

/// Time bucket from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiBucket<T> {
    /// Object type (e.g., "bucket").
    pub object: String,
    /// Start time (Unix timestamp).
    pub start_time: i64,
    /// End time (Unix timestamp).
    pub end_time: i64,
    /// Results within this bucket.
    pub results: Vec<T>,
}

// ============================================================================
// Completions usage
// ============================================================================

/// Completions usage result from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiCompletionsUsage {
    /// Object type.
    pub object: String,
    /// Input tokens used.
    pub input_tokens: i64,
    /// Output tokens used.
    pub output_tokens: i64,
    /// Cached input tokens.
    #[serde(default)]
    pub input_cached_tokens: i64,
    /// Audio input tokens.
    #[serde(default)]
    pub input_audio_tokens: i64,
    /// Audio output tokens.
    #[serde(default)]
    pub output_audio_tokens: i64,
    /// Number of model requests.
    pub num_model_requests: i64,
    /// Project ID (if grouped).
    pub project_id: Option<String>,
    /// User ID (if grouped).
    pub user_id: Option<String>,
    /// API key ID (if grouped).
    pub api_key_id: Option<String>,
    /// Model name (if grouped).
    pub model: Option<String>,
    /// Whether this is a batch request (if grouped).
    pub batch: Option<bool>,
    /// Service tier (if grouped).
    pub service_tier: Option<String>,
}

// ============================================================================
// Embeddings usage
// ============================================================================

/// Embeddings usage result from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiEmbeddingsUsage {
    /// Object type.
    pub object: String,
    /// Input tokens used.
    pub input_tokens: i64,
    /// Number of model requests.
    pub num_model_requests: i64,
    /// Project ID (if grouped).
    pub project_id: Option<String>,
    /// User ID (if grouped).
    pub user_id: Option<String>,
    /// API key ID (if grouped).
    pub api_key_id: Option<String>,
    /// Model name (if grouped).
    pub model: Option<String>,
}

// ============================================================================
// Costs
// ============================================================================

/// Cost amount from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiAmount {
    /// The value (in dollars, not cents).
    pub value: f64,
    /// Currency code.
    pub currency: String,
}

/// Cost result from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiCost {
    /// Object type.
    pub object: String,
    /// Cost amount.
    pub amount: OpenAiAmount,
    /// Line item description (if grouped).
    pub line_item: Option<String>,
    /// Project ID (if grouped).
    pub project_id: Option<String>,
}

// ============================================================================
// Error response
// ============================================================================

/// Error response from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiError {
    /// Error details.
    pub error: OpenAiErrorDetails,
}

/// Error details from OpenAI API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiErrorDetails {
    /// Error message.
    pub message: String,
    /// Error type.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error code.
    pub code: Option<String>,
}

// ============================================================================
// Request types (for internal use)
// ============================================================================

/// Query parameters for completions usage endpoint.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CompletionsUsageQuery {
    /// Start time (Unix seconds).
    pub start_time: i64,
    /// End time (Unix seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Bucket width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_width: Option<String>,
    /// Group by fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Model filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    /// Project ID filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_ids: Option<Vec<String>>,
    /// API key ID filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_ids: Option<Vec<String>>,
    /// User ID filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_ids: Option<Vec<String>>,
    /// Limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Page cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

/// Query parameters for costs endpoint.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CostsQuery {
    /// Start time (Unix seconds).
    pub start_time: i64,
    /// End time (Unix seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Bucket width (only "1d" supported).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_width: Option<String>,
    /// Group by fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Project ID filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_ids: Option<Vec<String>>,
    /// Limit (1-180).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Page cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}
