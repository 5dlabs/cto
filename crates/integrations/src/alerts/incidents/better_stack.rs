//! Better Stack integration for all-in-one monitoring and incident management.
//!
//! This module will provide integration with Better Stack (formerly Better Uptime):
//!
//! - Incident creation and management
//! - Status page integration
//! - On-call scheduling
//! - Heartbeat monitoring
//!
//! Better Stack provides an all-in-one solution eliminating the need for
//! separate monitoring and incident management vendors, with a meaningful
//! free tier and extremely fast setup (under 5 minutes).
//!
//! # Status
//!
//! This integration is currently scaffolding only. Implementation coming soon.
//!
//! # Example (Future API)
//!
//! ```ignore
//! use integrations::alerts::incidents::better_stack::BetterStackClient;
//!
//! let client = BetterStackClient::from_env()?;
//!
//! // Create an incident
//! client.create_incident("API latency spike", "warning").await?;
//!
//! // Send heartbeat
//! client.heartbeat("cto-platform-scheduler").await?;
//! ```
//!
//! # Configuration
//!
//! - `BETTER_STACK_API_TOKEN`: Better Stack API token
//! - `BETTER_STACK_HEARTBEAT_URL`: Heartbeat endpoint URL (optional)

// TODO: Implement Better Stack Incidents API
// TODO: Implement heartbeat monitoring
// TODO: Implement status page updates
// TODO: Implement on-call schedule queries
