use agent_client_protocol::{Implementation, SessionNotification, StopReason};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Permission policy to apply when an ACP runtime requests approval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcpPermissionPolicy {
    /// Select the first allow-like option automatically.
    AllowAll,
    /// Cancel the tool request and let the runtime fail gracefully.
    #[default]
    DenyAll,
}

/// Generic ACP session state tracked by CTO services.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcpRunState {
    /// Runtime selected but no ACP session has been established yet.
    #[default]
    Pending,
    /// ACP initialization completed.
    Initialized,
    /// Session is actively executing work.
    Running,
    /// Session is waiting on a permission decision.
    WaitingForPermission,
    /// Session finished successfully.
    Completed,
    /// Session failed.
    Failed,
    /// Session was cancelled.
    Cancelled,
}

/// Shared ACP session metadata persisted by services.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AcpSessionMetadata {
    /// Selected runtime identifier.
    #[serde(skip_serializing_if = "Option::is_none", rename = "runtimeId")]
    pub runtime_id: Option<String>,

    /// ACP session identifier allocated by the runtime.
    #[serde(skip_serializing_if = "Option::is_none", rename = "sessionId")]
    pub session_id: Option<String>,

    /// Current run state.
    #[serde(default, rename = "runState")]
    pub run_state: AcpRunState,

    /// Last observed event cursor or sequence identifier.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastEventCursor")]
    pub last_event_cursor: Option<String>,
}

/// Human-readable runtime implementation metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpImplementationInfo {
    /// Stable implementation name.
    pub name: String,
    /// Optional UI title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Version string.
    pub version: String,
}

impl From<&Implementation> for AcpImplementationInfo {
    fn from(value: &Implementation) -> Self {
        Self {
            name: value.name.clone(),
            title: value.title.clone(),
            version: value.version.clone(),
        }
    }
}

/// Prompt request issued through an ACP runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpPromptRequest {
    /// Runtime to target.
    #[serde(rename = "runtimeId")]
    pub runtime_id: String,

    /// Working directory for the ACP session.
    pub cwd: PathBuf,

    /// Prompt text sent to the runtime.
    pub prompt: String,

    /// Existing ACP session to resume, if any.
    #[serde(skip_serializing_if = "Option::is_none", rename = "sessionId")]
    pub session_id: Option<String>,
}

/// Result of a one-shot ACP prompt invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcpPromptResult {
    /// Runtime that handled the prompt.
    #[serde(rename = "runtimeId")]
    pub runtime_id: String,

    /// ACP session identifier returned by the runtime.
    #[serde(rename = "sessionId")]
    pub session_id: String,

    /// Negotiated runtime implementation info.
    #[serde(skip_serializing_if = "Option::is_none", rename = "agentInfo")]
    pub agent_info: Option<AcpImplementationInfo>,

    /// Stop reason reported by the runtime.
    #[serde(rename = "stopReason")]
    pub stop_reason: StopReason,

    /// Session notifications collected while the prompt was running.
    #[serde(default)]
    pub notifications: Vec<SessionNotification>,
}
