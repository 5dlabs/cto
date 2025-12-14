//! Anthropic Usage and Cost Admin API client.

use async_trait::async_trait;
use reqwest::Client;
use tracing::{debug, instrument};

use super::models::{AnthropicCostResponse, AnthropicError, AnthropicUsageResponse};
use crate::providers::{
    Amount, BucketWidth, CostBucket, CostProvider, CostProviderError, CostRequest, CostResponse,
    Currency, GroupByField, TokenUsage, UsageBucket, UsageRequest, UsageResponse,
};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1/organizations";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Usage and Cost Admin API provider.
///
/// Provides access to Anthropic's Admin API for usage and cost data.
#[derive(Debug, Clone)]
pub struct AnthropicCostProvider {
    client: Client,
    api_key: String,
}

impl AnthropicCostProvider {
    /// Create a new Anthropic cost provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic Admin API key (starts with `sk-ant-admin`)
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is empty or invalid format.
    pub fn new(api_key: impl Into<String>) -> Result<Self, CostProviderError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(CostProviderError::Auth(
                "Anthropic Admin API key is required".to_string(),
            ));
        }

        // Validate key format (should start with sk-ant-admin)
        if !api_key.starts_with("sk-ant-admin") {
            return Err(CostProviderError::Auth(
                "Anthropic Admin API key should start with 'sk-ant-admin'".to_string(),
            ));
        }

        let client = Client::builder()
            .user_agent("cto-cost/0.1.0")
            .build()
            .map_err(CostProviderError::Http)?;

        Ok(Self { client, api_key })
    }

    /// Create a new Anthropic cost provider from environment variable.
    ///
    /// Reads the API key from `ANTHROPIC_ADMIN_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set.
    pub fn from_env() -> Result<Self, CostProviderError> {
        let api_key = std::env::var("ANTHROPIC_ADMIN_KEY").map_err(|_| {
            CostProviderError::Auth("ANTHROPIC_ADMIN_KEY environment variable not set".to_string())
        })?;
        Self::new(api_key)
    }

    /// Make a GET request to the Anthropic API.
    async fn get<T>(&self, endpoint: &str, query: &str) -> Result<T, CostProviderError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{ANTHROPIC_API_BASE}{endpoint}?{query}");
        debug!(url = %url, "Making Anthropic API request");

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(error) = serde_json::from_str::<AnthropicError>(&error_text) {
                return Err(CostProviderError::Api {
                    status: status.as_u16(),
                    message: error.message,
                });
            }
            return Err(CostProviderError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let body = response.text().await?;
        serde_json::from_str(&body).map_err(CostProviderError::Serialization)
    }

    /// Convert bucket width to Anthropic format.
    fn format_bucket_width(width: BucketWidth) -> &'static str {
        match width {
            BucketWidth::Minute => "1m",
            BucketWidth::Hour => "1h",
            BucketWidth::Day => "1d",
        }
    }

    /// Convert our group by fields to Anthropic usage group by.
    fn convert_usage_group_by(field: GroupByField) -> Option<&'static str> {
        match field {
            GroupByField::Model => Some("model"),
            GroupByField::ProjectId => Some("workspace_id"),
            GroupByField::ApiKeyId => Some("api_key_id"),
            GroupByField::ServiceTier => Some("service_tier"),
            GroupByField::UserId | GroupByField::LineItem => None,
        }
    }

    /// Convert our group by fields to Anthropic cost group by.
    fn convert_cost_group_by(field: GroupByField) -> Option<&'static str> {
        match field {
            GroupByField::ProjectId => Some("workspace_id"),
            GroupByField::LineItem => Some("description"),
            _ => None,
        }
    }

    /// Build query string for usage endpoint.
    fn build_usage_query(request: &UsageRequest) -> String {
        let mut params = vec![
            format!(
                "starting_at={}",
                request.start_time.format("%Y-%m-%dT%H:%M:%SZ")
            ),
            format!(
                "ending_at={}",
                request.end_time.format("%Y-%m-%dT%H:%M:%SZ")
            ),
            format!(
                "bucket_width={}",
                Self::format_bucket_width(request.bucket_width)
            ),
        ];

        for field in &request.group_by {
            if let Some(group) = Self::convert_usage_group_by(*field) {
                params.push(format!("group_by[]={group}"));
            }
        }

        if let Some(models) = &request.models {
            for model in models {
                params.push(format!("models[]={model}"));
            }
        }

        // Anthropic uses workspace_ids instead of project_ids
        if let Some(project_ids) = &request.project_ids {
            for id in project_ids {
                params.push(format!("workspace_ids[]={id}"));
            }
        }

        if let Some(api_key_ids) = &request.api_key_ids {
            for id in api_key_ids {
                params.push(format!("api_key_ids[]={id}"));
            }
        }

        if let Some(limit) = request.limit {
            params.push(format!("limit={limit}"));
        }

        if let Some(page) = &request.page {
            params.push(format!("page={page}"));
        }

        params.join("&")
    }

    /// Build query string for cost endpoint.
    fn build_cost_query(request: &CostRequest) -> String {
        let mut params = vec![
            format!(
                "starting_at={}",
                request.start_time.format("%Y-%m-%dT%H:%M:%SZ")
            ),
            format!(
                "ending_at={}",
                request.end_time.format("%Y-%m-%dT%H:%M:%SZ")
            ),
        ];

        for field in &request.group_by {
            if let Some(group) = Self::convert_cost_group_by(*field) {
                params.push(format!("group_by[]={group}"));
            }
        }

        // Anthropic uses workspace_ids instead of project_ids
        if let Some(project_ids) = &request.project_ids {
            for id in project_ids {
                params.push(format!("workspace_ids[]={id}"));
            }
        }

        if let Some(limit) = request.limit {
            params.push(format!("limit={limit}"));
        }

        if let Some(page) = &request.page {
            params.push(format!("page={page}"));
        }

        params.join("&")
    }

    /// Parse cost string (in cents) to f64 dollars.
    fn parse_cost(cost_str: &str) -> f64 {
        cost_str.parse::<f64>().unwrap_or(0.0) / 100.0
    }
}

#[async_trait]
impl CostProvider for AnthropicCostProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    #[instrument(skip(self), fields(provider = "anthropic"))]
    async fn get_usage(&self, request: UsageRequest) -> Result<UsageResponse, CostProviderError> {
        // Validate time range
        if request.start_time >= request.end_time {
            return Err(CostProviderError::InvalidTimeRange(
                "start_time must be before end_time".to_string(),
            ));
        }

        let query = Self::build_usage_query(&request);
        let response: AnthropicUsageResponse = self.get("/usage_report/messages", &query).await?;

        let buckets: Vec<UsageBucket> = response
            .data
            .into_iter()
            .map(|bucket| UsageBucket {
                start_time: bucket.bucket_start_time,
                end_time: bucket.bucket_end_time,
                usage: TokenUsage {
                    input_tokens: bucket.input_tokens,
                    output_tokens: bucket.output_tokens,
                    cached_input_tokens: bucket.cache_read_input_tokens,
                    cache_creation_tokens: bucket.cache_creation_input_tokens,
                    audio_input_tokens: 0,
                    audio_output_tokens: 0,
                    num_requests: bucket.request_count,
                },
                model: bucket.model,
                project_id: bucket.workspace_id,
                api_key_id: bucket.api_key_id,
                user_id: None,
                service_tier: bucket.service_tier,
            })
            .collect();

        Ok(UsageResponse {
            buckets,
            has_more: response.has_more,
            next_page: response.next_page,
        })
    }

    #[instrument(skip(self), fields(provider = "anthropic"))]
    async fn get_costs(&self, request: CostRequest) -> Result<CostResponse, CostProviderError> {
        // Validate time range
        if request.start_time >= request.end_time {
            return Err(CostProviderError::InvalidTimeRange(
                "start_time must be before end_time".to_string(),
            ));
        }

        let query = Self::build_cost_query(&request);
        let response: AnthropicCostResponse = self.get("/cost_report", &query).await?;

        let buckets: Vec<CostBucket> = response
            .data
            .into_iter()
            .map(|bucket| CostBucket {
                start_time: bucket.bucket_start_time,
                end_time: bucket.bucket_end_time,
                amount: Amount {
                    value: Self::parse_cost(&bucket.cost_usd),
                    currency: Currency::Usd,
                },
                project_id: bucket.workspace_id,
                line_item: bucket.description,
                model: None,
            })
            .collect();

        Ok(CostResponse {
            buckets,
            has_more: response.has_more,
            next_page: response.next_page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_new_provider_requires_api_key() {
        let result = AnthropicCostProvider::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_provider_validates_key_format() {
        let result = AnthropicCostProvider::new("invalid-key");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("should start with"));
    }

    #[test]
    fn test_new_provider_with_valid_key() {
        let result = AnthropicCostProvider::new("sk-ant-admin-test-key");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_usage_query() {
        let request = UsageRequest {
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end_time: Utc.with_ymd_and_hms(2024, 1, 8, 0, 0, 0).unwrap(),
            bucket_width: BucketWidth::Day,
            group_by: vec![GroupByField::Model],
            ..Default::default()
        };

        let query = AnthropicCostProvider::build_usage_query(&request);
        assert!(query.contains("starting_at=2024-01-01"));
        assert!(query.contains("ending_at=2024-01-08"));
        assert!(query.contains("bucket_width=1d"));
        assert!(query.contains("group_by[]=model"));
    }

    #[test]
    fn test_build_cost_query() {
        let request = CostRequest {
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end_time: Utc.with_ymd_and_hms(2024, 1, 31, 0, 0, 0).unwrap(),
            group_by: vec![GroupByField::ProjectId, GroupByField::LineItem],
            limit: Some(30),
            ..Default::default()
        };

        let query = AnthropicCostProvider::build_cost_query(&request);
        assert!(query.contains("starting_at=2024-01-01"));
        assert!(query.contains("ending_at=2024-01-31"));
        assert!(query.contains("group_by[]=workspace_id"));
        assert!(query.contains("group_by[]=description"));
        assert!(query.contains("limit=30"));
    }

    #[test]
    fn test_parse_cost() {
        assert!((AnthropicCostProvider::parse_cost("100") - 1.0).abs() < f64::EPSILON);
        assert!((AnthropicCostProvider::parse_cost("50") - 0.5).abs() < f64::EPSILON);
        assert!((AnthropicCostProvider::parse_cost("0") - 0.0).abs() < f64::EPSILON);
    }
}
