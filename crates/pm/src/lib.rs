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
//! - Onboarding state machine for tenant setup
//!
//! # Note on Activity Types
//!
//! For new code requiring CLI-agnostic Linear activity emission, prefer using
//! the `linear-sink` crate directly. This crate maintains its own activity types
//! for backwards compatibility with existing binaries.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // Many async API methods can fail
#![allow(clippy::must_use_candidate)] // Detection functions are called for side effects
#![allow(clippy::cast_precision_loss)] // Acceptable for scoring calculations
#![allow(clippy::match_same_arms)] // Intentional for clarity in agent mapping
// Detection module - WIP, some functions not yet wired up
#![allow(dead_code)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::needless_raw_string_hashes)]

pub mod activities;
pub mod client;
pub mod config;
pub mod detection;
pub mod emitter;
pub mod handlers;
pub mod models;
pub mod onboarding;
pub mod server;
pub mod state;
pub mod webhooks;

// Re-export types from local modules (backwards compatible)
pub use activities::{ActivityContent, ActivitySignal, PlanStep, PlanStepStatus};
pub use client::LinearClient;
pub use config::Config;
pub use emitter::{AgentActivityEmitter, LinearAgentEmitter};
pub use models::*;
pub use webhooks::{
    identify_agent_from_signature, identify_agent_or_legacy, verify_webhook_signature,
    AgentIdentification, WebhookPayload,
};

// Also expose linear-sink types for gradual migration
pub use linear_sink;
