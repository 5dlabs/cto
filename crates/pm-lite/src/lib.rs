//! pm-lite: GitHub App integration for CTO Desktop
//!
//! Provides webhook handling and Tauri commands for GitHub App integration:
//! - Webhook handlers for pull_request, workflow_run, check_run events
//! - Tauri commands for app installation and event management
//! - In-memory event storage for desktop context

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod github_app;

pub use github_app::{
    install_github_app, list_webhook_events, redeliver_webhook, verify_webhook_signature,
    CheckRunEvent, GitHubAppConfig, PullRequestEvent, WebhookEvent, WebhookEventType,
    WorkflowRunEvent,
};
