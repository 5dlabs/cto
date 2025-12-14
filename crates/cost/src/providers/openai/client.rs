//! OpenAI Usage and Costs API client.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use reqwest::Client;
use tracing::{debug, instrument};

use super::models::{OpenAiBucket, OpenAiCompletionsUsage, OpenAiCost, OpenAiError, OpenAiPage};
use crate::providers::{
    Amount, CostBucket, CostProvider, CostProviderError, CostRequest, CostResponse, Currency,
    TokenUsage, UsageBucket, UsageRequest, UsageResponse,
};

const OPENAI_API_BASE: &str = "https://api.openai.com/v1/organization";

/// OpenAI Usage and Costs API provider.
///
/// Provides access to OpenAI's Usage API (token consumption) and Costs API (spend breakdown).
#[derive(Debug, Clone)]
pub struct OpenAiCostProvider {
    client: Client,
    api_key: String,
}

impl OpenAiCostProvider {
    /// Create a new OpenAI cost provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - OpenAI Admin API key (starts with `sk-admin-` or organization admin key)
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is empty.
    pub fn new(api_key: impl Into<String>) -> Result<Self, CostProviderError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(CostProviderError::Auth(
                "OpenAI API key is required".to_string(),
            ));
        }

        let client = Client::builder()
            .user_agent("cto-cost/0.1.0")
            .build()
            .map_err(CostProviderError::Http)?;

        Ok(Self { client, api_key })
    }

    /// Create a new OpenAI cost provider from environment variable.
    ///
    /// Reads the API key from `OPENAI_ADMIN_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set.
    pub fn from_env() -> Result<Self, CostProviderError> {
        let api_key = std::env::var("OPENAI_ADMIN_KEY").map_err(|_| {
            CostProviderError::Auth("OPENAI_ADMIN_KEY environment variable not set".to_string())
        })?;
        Self::new(api_key)
    }

    /// Make a GET request to the OpenAI API.
    async fn get<T>(&self, endpoint: &str, query: &str) -> Result<T, CostProviderError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{OPENAI_API_BASE}{endpoint}?{query}");
        debug!(url = %url, "Making OpenAI API request");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(error) = serde_json::from_str::<OpenAiError>(&error_text) {
                return Err(CostProviderError::Api {
                    status: status.as_u16(),
                    message: error.error.message,
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

    /// Build query string for completions usage endpoint.
    fn build_usage_query(request: &UsageRequest) -> String {
        let mut params = vec![format!("start_time={}", request.start_time.timestamp())];

        params.push(format!("end_time={}", request.end_time.timestamp()));
        params.push(format!("bucket_width={}", request.bucket_width));

        for field in &request.group_by {
            params.push(format!("group_by[]={field}"));
        }

        if let Some(models) = &request.models {
            for model in models {
                params.push(format!("models[]={model}"));
            }
        }

        if let Some(project_ids) = &request.project_ids {
            for id in project_ids {
                params.push(format!("project_ids[]={id}"));
            }
        }

        if let Some(api_key_ids) = &request.api_key_ids {
            for id in api_key_ids {
                params.push(format!("api_key_ids[]={id}"));
            }
        }

        if let Some(user_ids) = &request.user_ids {
            for id in user_ids {
                params.push(format!("user_ids[]={id}"));
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

    /// Build query string for costs endpoint.
    fn build_cost_query(request: &CostRequest) -> String {
        let mut params = vec![format!("start_time={}", request.start_time.timestamp())];

        params.push(format!("end_time={}", request.end_time.timestamp()));

        for field in &request.group_by {
            params.push(format!("group_by[]={field}"));
        }

        if let Some(project_ids) = &request.project_ids {
            for id in project_ids {
                params.push(format!("project_ids[]={id}"));
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

    /// Convert Unix timestamp to `DateTime<Utc>`.
    fn timestamp_to_datetime(timestamp: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(timestamp, 0).unwrap()
    }

    /// Convert OpenAI completions usage to our common format.
    fn convert_completions_usage(
        bucket: &OpenAiBucket<OpenAiCompletionsUsage>,
    ) -> Vec<UsageBucket> {
        bucket
            .results
            .iter()
            .map(|result| UsageBucket {
                start_time: Self::timestamp_to_datetime(bucket.start_time),
                end_time: Self::timestamp_to_datetime(bucket.end_time),
                usage: TokenUsage {
                    input_tokens: result.input_tokens,
                    output_tokens: result.output_tokens,
                    cached_input_tokens: result.input_cached_tokens,
                    cache_creation_tokens: 0,
                    audio_input_tokens: result.input_audio_tokens,
                    audio_output_tokens: result.output_audio_tokens,
                    num_requests: result.num_model_requests,
                },
                model: result.model.clone(),
                project_id: result.project_id.clone(),
                api_key_id: result.api_key_id.clone(),
                user_id: result.user_id.clone(),
                service_tier: result.service_tier.clone(),
            })
            .collect()
    }

    /// Convert OpenAI cost to our common format.
    fn convert_cost(bucket: &OpenAiBucket<OpenAiCost>) -> Vec<CostBucket> {
        bucket
            .results
            .iter()
            .map(|result| CostBucket {
                start_time: Self::timestamp_to_datetime(bucket.start_time),
                end_time: Self::timestamp_to_datetime(bucket.end_time),
                amount: Amount {
                    value: result.amount.value,
                    currency: Currency::Usd,
                },
                project_id: result.project_id.clone(),
                line_item: result.line_item.clone(),
                model: None,
            })
            .collect()
    }
}

#[async_trait]
impl CostProvider for OpenAiCostProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    #[instrument(skip(self), fields(provider = "openai"))]
    async fn get_usage(&self, request: UsageRequest) -> Result<UsageResponse, CostProviderError> {
        // Validate time range
        if request.start_time >= request.end_time {
            return Err(CostProviderError::InvalidTimeRange(
                "start_time must be before end_time".to_string(),
            ));
        }

        let query = Self::build_usage_query(&request);
        let response: OpenAiPage<OpenAiBucket<OpenAiCompletionsUsage>> =
            self.get("/usage/completions", &query).await?;

        let buckets: Vec<UsageBucket> = response
            .data
            .iter()
            .flat_map(Self::convert_completions_usage)
            .collect();

        Ok(UsageResponse {
            buckets,
            has_more: response.has_more,
            next_page: response.next_page,
        })
    }

    #[instrument(skip(self), fields(provider = "openai"))]
    async fn get_costs(&self, request: CostRequest) -> Result<CostResponse, CostProviderError> {
        // Validate time range
        if request.start_time >= request.end_time {
            return Err(CostProviderError::InvalidTimeRange(
                "start_time must be before end_time".to_string(),
            ));
        }

        let query = Self::build_cost_query(&request);
        let response: OpenAiPage<OpenAiBucket<OpenAiCost>> = self.get("/costs", &query).await?;

        let buckets: Vec<CostBucket> = response.data.iter().flat_map(Self::convert_cost).collect();

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
    use crate::providers::{BucketWidth, GroupByField};

    #[test]
    fn test_new_provider_requires_api_key() {
        let result = OpenAiCostProvider::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_provider_with_api_key() {
        let result = OpenAiCostProvider::new("sk-admin-test");
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

        let query = OpenAiCostProvider::build_usage_query(&request);
        assert!(query.contains("start_time="));
        assert!(query.contains("end_time="));
        assert!(query.contains("bucket_width=1d"));
        assert!(query.contains("group_by[]=model"));
    }

    #[test]
    fn test_build_cost_query() {
        let request = CostRequest {
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end_time: Utc.with_ymd_and_hms(2024, 1, 8, 0, 0, 0).unwrap(),
            group_by: vec![GroupByField::ProjectId, GroupByField::LineItem],
            limit: Some(30),
            ..Default::default()
        };

        let query = OpenAiCostProvider::build_cost_query(&request);
        assert!(query.contains("start_time="));
        assert!(query.contains("end_time="));
        assert!(query.contains("group_by[]=project_id"));
        assert!(query.contains("group_by[]=line_item"));
        assert!(query.contains("limit=30"));
    }
}
