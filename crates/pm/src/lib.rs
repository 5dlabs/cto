//! Project Management server for CTO platform.
//!
//! This crate provides:
//! - PM integrations (Linear, Asana, Jira, etc.)
//! - GraphQL client for Linear API
//! - Webhook payload parsing and signature verification
//! - Agent Activity emission for Linear's agent system
//! - Type definitions for Linear entities
//! - HTTP server for webhook handling (standalone service)
//! - Handlers for intake and play workflows

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // Many async API methods can fail

pub mod activities;
pub mod client;
pub mod config;
pub mod handlers;
pub mod integrations;
pub mod models;
pub mod server;
pub mod webhooks;

// VCS module disabled - GitHub client implementation pending
// pub mod vcs;

pub use activities::{ActivityContent, ActivitySignal};
pub use client::LinearClient;
pub use config::Config;
pub use models::*;
pub use webhooks::{verify_webhook_signature, WebhookPayload};
