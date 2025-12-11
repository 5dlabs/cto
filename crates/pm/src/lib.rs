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
pub mod emitter;
pub mod handlers;
pub mod models;
pub mod server;
pub mod webhooks;

pub use activities::{ActivityContent, ActivitySignal, PlanStep, PlanStepStatus};
pub use client::LinearClient;
pub use config::Config;
pub use emitter::{AgentActivityEmitter, LinearAgentEmitter};
pub use models::*;
pub use webhooks::{verify_webhook_signature, WebhookPayload};
