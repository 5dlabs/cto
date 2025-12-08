//! GitHub API client for webhook management.
//!
//! Re-exports the GitHub client from the integrations crate root for backwards compatibility.

pub use crate::github_client::{ensure_github_webhooks, GitHubClient, Webhook, WebhookConfig};

