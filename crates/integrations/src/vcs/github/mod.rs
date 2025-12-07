//! GitHub integration for webhooks and API interactions.
//!
//! This module provides:
//! - GitHub API client for repository operations
//! - Webhook handling for PR and push events
//! - Webhook management (creating/updating webhooks on repos)
//!
//! # Example
//!
//! ```no_run
//! use integrations::vcs::github::{GitHubClient, ensure_github_webhooks};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a client for webhook management
//! let client = GitHubClient::new("your-token")?;
//!
//! // Ensure webhooks are configured on startup
//! let results = ensure_github_webhooks(
//!     "your-token",
//!     "https://your-service.example.com",
//!     &["org/repo".to_string()],
//! ).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod webhooks;

pub use client::{ensure_github_webhooks, GitHubClient, Webhook, WebhookConfig};
pub use webhooks::{
    GitHubLabel, GitHubUser, GitRef, IntakeMetadata, PullRequest, PullRequestEvent, Repository,
    SubtaskFromJson, TaskFromJson, TasksJsonFile, TasksMetadata,
};
