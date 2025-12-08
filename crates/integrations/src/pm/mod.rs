//! Project management integrations.
//!
//! This module provides integrations with various project management platforms:
//!
//! ## Fully Implemented
//!
//! - **Linear** - Issue tracking and project management with agent system
//!
//! ## Scaffolding (Coming Soon)
//!
//! - **Asana** - Work management platform
//! - **ClickUp** - Productivity and project management
//! - **Jira** - Atlassian issue and project tracking
//! - **Monday.com** - Work OS and project management
//! - **Notion** - All-in-one workspace
//! - **Trello** - Kanban-style project boards
//!
//! # Example
//!
//! ```no_run
//! use integrations::pm::linear::{LinearClient, ActivityContent};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = LinearClient::new("your-token")?;
//! let issue = client.get_issue("issue-id").await?;
//! println!("Issue: {}", issue.title);
//! # Ok(())
//! # }
//! ```

pub mod asana;
pub mod clickup;
pub mod jira;
pub mod linear;
pub mod monday;
pub mod notion;
pub mod trello;

// Re-export Linear as the primary implementation for convenience
pub use linear::{
    ActivityContent, ActivitySignal, Config, CtoConfig, LinearClient, WebhookPayload,
};
