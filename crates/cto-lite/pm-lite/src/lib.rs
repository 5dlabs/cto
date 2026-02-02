//! PM Lite - Lightweight Project Management server for CTO Lite
//!
//! This crate provides:
//! - GitHub webhook handling (issues, PRs, comments)
//! - Workflow triggering via Argo Workflows
//! - Simple state management
//!
//! Unlike the full PM server, this excludes:
//! - Linear integration
//! - Multi-tenant support
//! - Complex onboarding flows

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub mod config;
pub mod github;
pub mod server;
pub mod workflow;

pub use config::Config;
pub use server::Server;
