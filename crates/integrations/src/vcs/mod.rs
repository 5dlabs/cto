//! Version control system integrations.
//!
//! This module provides integrations with various version control platforms:
//!
//! - **GitHub** - Git hosting and collaboration platform
//!
//! # Example
//!
//! ```no_run
//! use integrations::vcs::github::{GitHubClient, ensure_github_webhooks};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = GitHubClient::new("your-token")?;
//!
//! // List webhooks for a repository
//! let webhooks = client.list_webhooks("owner", "repo").await?;
//! # Ok(())
//! # }
//! ```

pub mod github;

// Re-export GitHub as the primary implementation
pub use github::{ensure_github_webhooks, GitHubClient};
