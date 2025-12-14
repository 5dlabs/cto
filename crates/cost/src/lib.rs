#![allow(clippy::doc_markdown)] // Allow brand names like OpenAI, Anthropic without backticks

//! Cost analysis and optimization for AI/LLM providers.
//!
//! This crate provides a unified interface for tracking API usage and costs
//! across multiple AI/LLM providers:
//!
//! - **OpenAI** - Usage API and Costs API
//! - **Anthropic** - Usage and Cost Admin API
//!
//! ## Features
//!
//! - Track token usage (input, output, cached, audio)
//! - Get cost breakdowns by project, model, and line item
//! - Aggregate usage summaries over time periods
//! - Support for pagination and filtering
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cto_cost::providers::{OpenAiCostProvider, CostProvider, UsageRequest, BucketWidth};
//! use chrono::{Utc, Duration};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create provider from environment variable
//!     let provider = OpenAiCostProvider::from_env()?;
//!
//!     // Get usage for the last 7 days
//!     let usage = provider.get_usage(UsageRequest {
//!         start_time: Utc::now() - Duration::days(7),
//!         end_time: Utc::now(),
//!         bucket_width: BucketWidth::Day,
//!         ..Default::default()
//!     }).await?;
//!
//!     for bucket in usage.buckets {
//!         println!(
//!             "{}: {} input tokens, {} output tokens",
//!             bucket.start_time,
//!             bucket.usage.input_tokens,
//!             bucket.usage.output_tokens
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Providers
//!
//! ### OpenAI Provider
//!
//! Requires an Admin API key from OpenAI organization settings.
//! Set the `OPENAI_ADMIN_KEY` environment variable or pass directly.
//!
//! ```rust,ignore
//! use cto_cost::providers::OpenAiCostProvider;
//!
//! // From environment
//! let provider = OpenAiCostProvider::from_env()?;
//!
//! // Or directly
//! let provider = OpenAiCostProvider::new("sk-admin-xxx")?;
//! ```
//!
//! ### Anthropic Provider
//!
//! Requires an Admin API key from Claude Console settings.
//! Set the `ANTHROPIC_ADMIN_KEY` environment variable or pass directly.
//!
//! ```rust,ignore
//! use cto_cost::providers::AnthropicCostProvider;
//!
//! // From environment
//! let provider = AnthropicCostProvider::from_env()?;
//!
//! // Or directly
//! let provider = AnthropicCostProvider::new("sk-ant-admin-xxx")?;
//! ```
//!
//! ## Usage Patterns
//!
//! ### Daily Usage by Model
//!
//! ```rust,ignore
//! use cto_cost::providers::{CostProvider, UsageRequest, BucketWidth, GroupByField};
//!
//! let usage = provider.get_usage(UsageRequest {
//!     start_time: Utc::now() - Duration::days(7),
//!     end_time: Utc::now(),
//!     bucket_width: BucketWidth::Day,
//!     group_by: vec![GroupByField::Model],
//!     ..Default::default()
//! }).await?;
//! ```
//!
//! ### Cost Breakdown by Project
//!
//! ```rust,ignore
//! use cto_cost::providers::{CostProvider, CostRequest, GroupByField};
//!
//! let costs = provider.get_costs(CostRequest {
//!     start_time: Utc::now() - Duration::days(30),
//!     end_time: Utc::now(),
//!     group_by: vec![GroupByField::ProjectId, GroupByField::LineItem],
//!     ..Default::default()
//! }).await?;
//!
//! for bucket in costs.buckets {
//!     println!(
//!         "{}: {} - {}",
//!         bucket.start_time,
//!         bucket.project_id.unwrap_or_default(),
//!         bucket.amount.format_dollars()
//!     );
//! }
//! ```
//!
//! ### Usage Summary
//!
//! ```rust,ignore
//! let summary = provider.get_summary(
//!     Utc::now() - Duration::days(7),
//!     Utc::now()
//! ).await?;
//!
//! println!("Total cost: {}", summary.total_cost.format_dollars());
//! println!("Total requests: {}", summary.total_usage.num_requests);
//! println!("Total tokens: {}", summary.total_usage.total_tokens());
//! ```

pub mod providers;

pub use providers::{
    Amount, AnthropicCostProvider, BucketWidth, CostBucket, CostProvider, CostProviderError,
    CostRequest, CostResponse, Currency, GroupByField, ModelUsageSummary, OpenAiCostProvider,
    TokenUsage, UsageBucket, UsageRequest, UsageResponse, UsageSummary,
};
