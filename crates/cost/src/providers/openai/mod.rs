//! OpenAI Usage and Costs API client.
//!
//! This module provides integration with OpenAI's Usage and Costs API:
//!
//! - **Usage API**: Detailed insights into token usage across completions, embeddings, etc.
//! - **Costs API**: Daily spend breakdown by project and line item.
//!
//! ## Authentication
//!
//! Requires an Admin API key (obtained from OpenAI organization settings).
//! The key should start with `sk-admin-` or be an organization admin key.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cto_cost::providers::{OpenAiCostProvider, UsageRequest, BucketWidth};
//! use chrono::{Utc, Duration};
//!
//! let provider = OpenAiCostProvider::new("sk-admin-xxx")?;
//! let usage = provider.get_usage(UsageRequest {
//!     start_time: Utc::now() - Duration::days(7),
//!     end_time: Utc::now(),
//!     bucket_width: BucketWidth::Day,
//!     ..Default::default()
//! }).await?;
//! ```

mod client;
mod models;

pub use client::OpenAiCostProvider;
pub use models::*;
