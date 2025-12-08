//! Grafana `OnCall` integration for open-source incident management.
//!
//! This module will provide integration with Grafana `OnCall`:
//!
//! - Alert routing and escalation
//! - On-call schedule management
//! - Integration with Grafana alerting
//! - Slack/Teams/SMS/Phone notifications
//!
//! Grafana `OnCall` is free and open-source, integrating naturally
//! if you're already using Grafana for observability. Great for
//! teams that want to own their stack.
//!
//! # Status
//!
//! This integration is currently scaffolding only. Implementation coming soon.
//!
//! # Example (Future API)
//!
//! ```ignore
//! use integrations::alerts::incidents::grafana_oncall::GrafanaOnCallClient;
//!
//! let client = GrafanaOnCallClient::from_env()?;
//!
//! // Send alert via integration
//! client.send_alert(
//!     "integration-url",
//!     "High CPU usage on worker nodes",
//!     "warning",
//! ).await?;
//!
//! // Query on-call schedule
//! let oncall = client.get_current_oncall("schedule-id").await?;
//! ```
//!
//! # Configuration
//!
//! - `GRAFANA_ONCALL_URL`: Grafana `OnCall` instance URL
//! - `GRAFANA_ONCALL_API_TOKEN`: API token for authentication
//! - `GRAFANA_ONCALL_INTEGRATION_URL`: Default integration webhook URL

// TODO: Implement Grafana OnCall HTTP API
// TODO: Implement alert sending via integrations
// TODO: Implement on-call schedule queries
// TODO: Implement escalation chain management
