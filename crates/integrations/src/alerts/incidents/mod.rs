//! Incident management and alerting integrations.
//!
//! This module provides integrations with various incident management and
//! on-call alerting platforms for handling critical alerts and incidents.
//!
//! ## Fully Implemented
//!
//! - **`PagerDuty`** - Industry-standard incident management with Events API v2
//!
//! ## Scaffolding (Coming Soon)
//!
//! - **Opsgenie** - Atlassian incident management (note: EOL April 2027)
//! - **incident.io** - Chat-native incident management for Slack/Teams
//! - **Better Stack** - All-in-one monitoring + incident management
//! - **Zenduty** - Fast-growing `PagerDuty` alternative
//! - **Grafana `OnCall`** - Free, open-source, integrates with Grafana
//! - **Rootly** - Comprehensive incident response platform
//! - **Spike** - Simple incident response, 5-minute setup
//!
//! # Example
//!
//! ```no_run
//! use integrations::alerts::incidents::pagerduty::{PagerDutyClient, PagerDutyEvent, EventSeverity};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = PagerDutyClient::from_env()?;
//!
//! // Trigger an incident
//! let event = PagerDutyEvent::trigger("Database connection failed", "cto-platform")
//!     .with_dedup_key("db-prod-001")
//!     .with_severity(EventSeverity::Critical);
//!
//! let dedup_key = client.send_event(&event).await?;
//!
//! // Later, resolve it
//! client.resolve(&dedup_key).await?;
//! # Ok(())
//! # }
//! ```

pub mod better_stack;
pub mod grafana_oncall;
pub mod incident_io;
pub mod opsgenie;
pub mod pagerduty;
pub mod rootly;
pub mod spike;
pub mod zenduty;

// Re-export PagerDuty as the primary implementation
pub use pagerduty::{EventAction, EventPayload, EventSeverity, PagerDutyClient, PagerDutyEvent};
