//! Anthropic Usage and Cost Admin API client.
//!
//! This module provides integration with Anthropic's Admin API:
//!
//! - **Usage API**: Token consumption with breakdowns by model, workspace, and service tier.
//! - **Cost API**: Daily spend breakdown by workspace and description.
//!
//! ## Authentication
//!
//! Requires an Admin API key (obtained from Claude Console settings).
//! The key should start with `sk-ant-admin`.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cto_cost::providers::{AnthropicCostProvider, UsageRequest, BucketWidth};
//! use chrono::{Utc, Duration};
//!
//! let provider = AnthropicCostProvider::new("sk-ant-admin-xxx")?;
//! let usage = provider.get_usage(UsageRequest {
//!     start_time: Utc::now() - Duration::days(7),
//!     end_time: Utc::now(),
//!     bucket_width: BucketWidth::Day,
//!     ..Default::default()
//! }).await?;
//! ```

mod client;
mod models;

pub use client::AnthropicCostProvider;
pub use models::*;
