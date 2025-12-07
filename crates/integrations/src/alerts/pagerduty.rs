//! `PagerDuty` integration for incident management.
//!
//! This module provides integration with `PagerDuty` Events API v2 for triggering
//! and resolving incidents.
//!
//! # Usage
//!
//! ```no_run
//! use integrations::alerts::pagerduty::{PagerDutyClient, PagerDutyEvent, EventSeverity};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = PagerDutyClient::from_env()?;
//!
//! // Trigger an incident
//! let event = PagerDutyEvent::trigger("Database connection failed", "cto-platform")
//!     .with_dedup_key("db-connection-prod-001")
//!     .with_severity(EventSeverity::Critical);
//!
//! let dedup_key = client.send_event(&event).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! - `PAGERDUTY_ROUTING_KEY`: Integration key from your `PagerDuty` service

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Environment variable for `PagerDuty` routing key.
const ENV_PAGERDUTY_ROUTING_KEY: &str = "PAGERDUTY_ROUTING_KEY";

/// `PagerDuty` Events API v2 endpoint.
const EVENTS_API_URL: &str = "https://events.pagerduty.com/v2/enqueue";

/// `PagerDuty` client for Events API v2.
#[derive(Debug, Clone)]
pub struct PagerDutyClient {
    routing_key: String,
    client: reqwest::Client,
}

impl PagerDutyClient {
    /// Create a new `PagerDuty` client from environment variables.
    ///
    /// # Errors
    /// Returns error if `PAGERDUTY_ROUTING_KEY` is not set.
    pub fn from_env() -> anyhow::Result<Self> {
        let routing_key = std::env::var(ENV_PAGERDUTY_ROUTING_KEY)
            .map_err(|_| anyhow::anyhow!("PAGERDUTY_ROUTING_KEY not set"))?;

        debug!("PagerDuty client initialized");

        Ok(Self {
            routing_key,
            client: reqwest::Client::new(),
        })
    }

    /// Create a new `PagerDuty` client with a specific routing key.
    #[must_use]
    pub fn new(routing_key: String) -> Self {
        Self {
            routing_key,
            client: reqwest::Client::new(),
        }
    }

    /// Send an event to `PagerDuty`.
    ///
    /// Returns the `dedup_key` for the event (useful for acknowledging/resolving).
    ///
    /// # Errors
    /// Returns error if the API request fails.
    pub async fn send_event(&self, event: &PagerDutyEvent) -> anyhow::Result<String> {
        let payload = ApiPayload {
            routing_key: &self.routing_key,
            event_action: event.event_action,
            dedup_key: event.dedup_key.as_deref(),
            payload: &event.payload,
        };

        debug!(
            action = ?event.event_action,
            dedup_key = ?event.dedup_key,
            "Sending PagerDuty event"
        );

        let response = self.client.post(EVENTS_API_URL).json(&payload).send().await?;

        if response.status().is_success() {
            let result: ApiResponse = response.json().await?;
            debug!(dedup_key = %result.dedup_key, "PagerDuty event sent successfully");
            Ok(result.dedup_key)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            warn!(
                status = %status,
                body = %body,
                "PagerDuty API request failed"
            );

            Err(anyhow::anyhow!("PagerDuty returned {status}: {body}"))
        }
    }

    /// Acknowledge an existing incident by `dedup_key`.
    ///
    /// # Errors
    /// Returns error if the API request fails.
    pub async fn acknowledge(&self, dedup_key: &str) -> anyhow::Result<()> {
        let event = PagerDutyEvent {
            event_action: EventAction::Acknowledge,
            dedup_key: Some(dedup_key.to_string()),
            payload: EventPayload {
                summary: "Acknowledged".to_string(),
                source: "cto-platform".to_string(),
                severity: EventSeverity::Info,
                timestamp: None,
                component: None,
                group: None,
                class: None,
                custom_details: None,
            },
        };

        self.send_event(&event).await?;
        Ok(())
    }

    /// Resolve an existing incident by `dedup_key`.
    ///
    /// # Errors
    /// Returns error if the API request fails.
    pub async fn resolve(&self, dedup_key: &str) -> anyhow::Result<()> {
        let event = PagerDutyEvent {
            event_action: EventAction::Resolve,
            dedup_key: Some(dedup_key.to_string()),
            payload: EventPayload {
                summary: "Resolved".to_string(),
                source: "cto-platform".to_string(),
                severity: EventSeverity::Info,
                timestamp: None,
                component: None,
                group: None,
                class: None,
                custom_details: None,
            },
        };

        self.send_event(&event).await?;
        Ok(())
    }
}

/// `PagerDuty` event for Events API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyEvent {
    /// Event action (trigger, acknowledge, resolve)
    pub event_action: EventAction,
    /// Dedup key for grouping related events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup_key: Option<String>,
    /// Event payload
    pub payload: EventPayload,
}

/// `PagerDuty` event action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventAction {
    /// Trigger a new incident or add to existing
    Trigger,
    /// Acknowledge an incident
    Acknowledge,
    /// Resolve an incident
    Resolve,
}

/// `PagerDuty` event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// Brief summary of the event
    pub summary: String,
    /// Source of the event
    pub source: String,
    /// Severity level
    pub severity: EventSeverity,
    /// Timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Component affected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// Group for categorization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Class/type of event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// Custom details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_details: Option<serde_json::Value>,
}

/// `PagerDuty` event severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    /// Critical severity
    Critical,
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Info severity
    Info,
}

impl PagerDutyEvent {
    /// Create a trigger event.
    #[must_use]
    pub fn trigger(summary: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            event_action: EventAction::Trigger,
            dedup_key: None,
            payload: EventPayload {
                summary: summary.into(),
                source: source.into(),
                severity: EventSeverity::Error,
                timestamp: None,
                component: None,
                group: None,
                class: None,
                custom_details: None,
            },
        }
    }

    /// Set the dedup key.
    #[must_use]
    pub fn with_dedup_key(mut self, key: impl Into<String>) -> Self {
        self.dedup_key = Some(key.into());
        self
    }

    /// Set the severity.
    #[must_use]
    pub const fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.payload.severity = severity;
        self
    }

    /// Set the component.
    #[must_use]
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.payload.component = Some(component.into());
        self
    }

    /// Set the group.
    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.payload.group = Some(group.into());
        self
    }

    /// Set custom details.
    #[must_use]
    pub fn with_custom_details(mut self, details: serde_json::Value) -> Self {
        self.payload.custom_details = Some(details);
        self
    }
}

// =============================================================================
// API types (internal)
// =============================================================================

#[derive(Debug, Serialize)]
struct ApiPayload<'a> {
    routing_key: &'a str,
    event_action: EventAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    dedup_key: Option<&'a str>,
    payload: &'a EventPayload,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiResponse {
    status: String,
    message: String,
    dedup_key: String,
}

/// `PagerDuty` webhook payload placeholder.
#[derive(Debug, Clone, Deserialize)]
pub struct PagerDutyWebhookPayload {
    /// Event type
    pub event: PagerDutyWebhookEvent,
}

/// `PagerDuty` webhook event.
#[derive(Debug, Clone, Deserialize)]
pub struct PagerDutyWebhookEvent {
    /// Event type (incident.triggered, incident.resolved, etc.)
    pub event_type: String,
    /// Resource type
    pub resource_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = PagerDutyEvent::trigger("Test alert", "test-source")
            .with_dedup_key("test-123")
            .with_severity(EventSeverity::Critical)
            .with_component("database");

        assert_eq!(event.event_action, EventAction::Trigger);
        assert_eq!(event.dedup_key, Some("test-123".to_string()));
        assert_eq!(event.payload.severity, EventSeverity::Critical);
        assert_eq!(event.payload.component, Some("database".to_string()));
    }

    #[test]
    fn test_event_serialization() {
        let event = PagerDutyEvent::trigger("Test", "source");
        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains("\"event_action\":\"trigger\""));
        assert!(json.contains("\"severity\":\"error\""));
    }
}

