use acp_runtime::AcpSessionMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Normalized monitor status emitted by Stakpak schedules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonitorEventStatus {
    Success,
    Failure,
    Timeout,
    Paused,
}

/// Severity attached to a monitor event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonitorEventSeverity {
    Info,
    Warn,
    Critical,
}

/// Monitor event payload ingested from always-on Stakpak schedules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StakpakMonitorEvent {
    /// Event source. Defaults to `stakpak`.
    #[serde(default = "default_monitor_source")]
    pub source: String,
    /// Schedule name in the monitor runtime.
    pub schedule_name: String,
    /// Runtime-specific run identifier.
    pub run_id: String,
    /// Normalized run outcome.
    pub status: MonitorEventStatus,
    /// Actionability severity.
    pub severity: MonitorEventSeverity,
    /// Short summary for operators.
    pub summary: String,
    /// Detailed event body or trace snippet.
    pub details: String,
    /// Idempotency key for retries.
    pub fingerprint: String,
    /// Run start time.
    pub started_at: DateTime<Utc>,
    /// Run finish time.
    pub finished_at: DateTime<Utc>,
    /// Optional deep links for the event.
    #[serde(default)]
    pub links: Vec<String>,
}

fn default_monitor_source() -> String {
    "stakpak".to_string()
}

/// Shared in-memory monitor event store.
#[derive(Debug, Clone, Default)]
pub struct MonitorEventStore {
    events: Arc<RwLock<HashMap<String, StakpakMonitorEvent>>>,
}

impl MonitorEventStore {
    /// Insert or update an event keyed by fingerprint.
    ///
    /// Returns true when the fingerprint already existed.
    pub async fn upsert(&self, event: StakpakMonitorEvent) -> bool {
        let mut events = self.events.write().await;
        let duplicate = events.contains_key(&event.fingerprint);
        events.insert(event.fingerprint.clone(), event);
        duplicate
    }

    /// List all known events.
    pub async fn list(&self) -> Vec<StakpakMonitorEvent> {
        self.events.read().await.values().cloned().collect()
    }

    /// Number of stored events.
    pub async fn len(&self) -> usize {
        self.events.read().await.len()
    }
}

/// Healer's tracked ACP investigation session.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct HealerAcpSessionRecord {
    /// Healer-local key, for example an incident or issue ID.
    pub key: String,
    /// Optional issue identifier associated with the session.
    #[serde(skip_serializing_if = "Option::is_none", rename = "issueId")]
    pub issue_id: Option<String>,
    /// ACP runtime session metadata.
    #[serde(default)]
    pub session: AcpSessionMetadata,
}
