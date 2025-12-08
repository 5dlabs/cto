//! Linear API client and webhook service for CTO platform integration.
//!
//! This crate provides integration with Linear for task management and workflow coordination.
//!
//! Note: This crate is under active development. Some modules may be incomplete.

pub mod config;
pub mod handlers;
// pub mod server; // Commented out: has dependencies on missing modules

pub use config::Config;
