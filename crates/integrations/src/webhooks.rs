//! Webhook payload parsing and signature verification.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::models::{AgentSession, Comment, Issue};

type HmacSha256 = Hmac<Sha256>;

/// Verify Linear webhook signature using HMAC-SHA256.
///
/// # Arguments
/// * `body` - Raw webhook body bytes
/// * `signature` - Hex-encoded signature from `Linear-Signature` header
/// * `secret` - Webhook signing secret
///
/// # Returns
/// `true` if signature is valid, `false` otherwise
#[must_use]
pub fn verify_webhook_signature(body: &[u8], signature: &str, secret: &str) -> bool {
    // Decode the hex signature
    let Ok(signature_bytes) = hex::decode(signature) else {
        return false;
    };

    // Compute HMAC-SHA256
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let computed = mac.finalize().into_bytes();

    // Constant-time comparison to prevent timing attacks
    computed.as_slice().ct_eq(&signature_bytes).into()
}

/// Validate webhook timestamp is within acceptable range.
///
/// # Arguments
/// * `timestamp_ms` - Webhook timestamp in milliseconds
/// * `max_age_ms` - Maximum age in milliseconds (default 60000 = 60 seconds)
///
/// # Returns
/// `true` if timestamp is valid, `false` if stale
#[must_use]
pub fn validate_webhook_timestamp(timestamp_ms: i64, max_age_ms: i64) -> bool {
    let now_ms = chrono::Utc::now().timestamp_millis();
    (now_ms - timestamp_ms).abs() <= max_age_ms
}

/// Webhook action type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookAction {
    /// New entity created (Linear sends "create" for Issue events)
    Create,
    /// New agent session created (mention or delegation)
    Created,
    /// User sent follow-up prompt to existing session
    Prompted,
    /// Generic update action
    Update,
    /// Generic remove action
    Remove,
    /// Unknown action (catch-all to avoid parse failures)
    #[serde(other)]
    Unknown,
}

/// Webhook event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookType {
    /// Agent session event
    AgentSessionEvent,
    /// Issue event
    Issue,
    /// Issue attachment event (link added/removed)
    IssueAttachment,
    /// Comment event
    Comment,
    /// App user notification
    AppUserNotification,
    /// Permission change
    PermissionChange,
    /// OAuth app revoked
    OAuthApp,
    /// Unknown type
    #[serde(other)]
    Unknown,
}

/// Agent activity in webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookAgentActivity {
    /// Activity ID
    pub id: String,
    /// Activity content
    pub content: AgentActivityContent,
    /// Signals attached to the activity
    #[serde(default)]
    pub signals: Vec<String>,
}

/// Agent activity content from webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum AgentActivityContent {
    /// Thought content
    AgentActivityThoughtContent {
        /// Thought body
        body: String,
    },
    /// Action content
    AgentActivityActionContent {
        /// Action name
        action: String,
        /// Action parameter
        parameter: String,
        /// Action result
        #[serde(default)]
        result: Option<String>,
    },
    /// Elicitation content
    AgentActivityElicitationContent {
        /// Elicitation body
        body: String,
    },
    /// Response content
    AgentActivityResponseContent {
        /// Response body
        body: String,
    },
    /// Error content
    AgentActivityErrorContent {
        /// Error body
        body: String,
    },
    /// Prompt content (user message)
    AgentActivityPromptContent {
        /// Prompt body
        body: String,
    },
}

/// Main webhook payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    /// Action type
    pub action: WebhookAction,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: WebhookType,
    /// Timestamp when webhook was sent (milliseconds)
    pub webhook_timestamp: i64,
    /// Unique webhook ID
    #[serde(default)]
    pub webhook_id: Option<String>,
    /// Organization ID
    #[serde(default)]
    pub organization_id: Option<String>,
    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<String>,
    /// URL of the subject entity
    #[serde(default)]
    pub url: Option<String>,
    /// Agent session (for `AgentSessionEvent`)
    #[serde(default)]
    pub agent_session: Option<AgentSession>,
    /// Agent activity (for prompted events)
    #[serde(default)]
    pub agent_activity: Option<WebhookAgentActivity>,
    /// Generic data payload (varies by event type - Issue, Comment, etc.)
    /// We use Value to avoid parse failures for different payload structures
    #[serde(default)]
    pub data: Option<Value>,
    /// Actor who triggered the event
    #[serde(default)]
    pub actor: Option<WebhookActor>,
}

/// Actor who triggered the webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookActor {
    /// Actor ID
    pub id: String,
    /// Actor type (user, `OauthClient`, Integration)
    #[serde(rename = "type")]
    pub actor_type: String,
    /// Actor name
    #[serde(default)]
    pub name: Option<String>,
    /// Actor email
    #[serde(default)]
    pub email: Option<String>,
}

impl WebhookPayload {
    /// Check if this is an agent session event
    #[must_use]
    pub const fn is_agent_session_event(&self) -> bool {
        matches!(self.event_type, WebhookType::AgentSessionEvent)
    }

    /// Check if this is a new session (created action)
    #[must_use]
    pub const fn is_new_session(&self) -> bool {
        matches!(self.action, WebhookAction::Created) && self.is_agent_session_event()
    }

    /// Check if this is a prompted session (follow-up message)
    #[must_use]
    pub const fn is_prompted_session(&self) -> bool {
        matches!(self.action, WebhookAction::Prompted) && self.is_agent_session_event()
    }

    /// Check if the prompt contains a stop signal
    #[must_use]
    pub fn has_stop_signal(&self) -> bool {
        self.agent_activity
            .as_ref()
            .is_some_and(|activity| activity.signals.iter().any(|s| s == "stop"))
    }

    /// Get the issue from the agent session
    #[must_use]
    pub fn get_issue(&self) -> Option<&Issue> {
        self.agent_session.as_ref().and_then(|s| s.issue.as_ref())
    }

    /// Get the session ID
    #[must_use]
    pub fn get_session_id(&self) -> Option<&str> {
        self.agent_session.as_ref().map(|s| s.id.as_str())
    }

    /// Get the prompt body from a prompted event
    #[must_use]
    pub fn get_prompt_body(&self) -> Option<&str> {
        self.agent_activity.as_ref().and_then(|activity| {
            if let AgentActivityContent::AgentActivityPromptContent { body } = &activity.content {
                Some(body.as_str())
            } else {
                None
            }
        })
    }

    /// Try to parse the data field as an Issue (for Issue events)
    #[must_use]
    pub fn get_issue_data(&self) -> Option<Issue> {
        self.data
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Try to parse the data field as a Comment (for Comment events)
    #[must_use]
    pub fn get_comment_data(&self) -> Option<Comment> {
        self.data
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Try to get the attachment URL from `IssueAttachment` events
    #[must_use]
    pub fn get_attachment_url(&self) -> Option<String> {
        self.data.as_ref().and_then(|v| {
            v.get("url").and_then(|u| u.as_str()).map(String::from)
        })
    }

    /// Try to get the issue ID from `IssueAttachment` events
    #[must_use]
    pub fn get_attachment_issue_id(&self) -> Option<String> {
        self.data.as_ref().and_then(|v| {
            // Try nested issue.id first, then issueId
            v.get("issue")
                .and_then(|i| i.get("id"))
                .and_then(|id| id.as_str())
                .map(String::from)
                .or_else(|| {
                    v.get("issueId")
                        .and_then(|id| id.as_str())
                        .map(String::from)
                })
        })
    }
}

/// Parsed webhook headers
#[derive(Debug, Clone)]
pub struct WebhookHeaders {
    /// Unique delivery ID
    pub delivery_id: Option<String>,
    /// Event type
    pub event_type: Option<String>,
    /// HMAC signature
    pub signature: Option<String>,
}

impl WebhookHeaders {
    /// Parse headers from a request
    #[must_use]
    pub fn from_header_map(get_header: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            delivery_id: get_header("linear-delivery"),
            event_type: get_header("linear-event"),
            signature: get_header("linear-signature"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_webhook_signature_valid() {
        let body = b"test payload";
        let secret = "test-secret";

        // Compute expected signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(verify_webhook_signature(body, &signature, secret));
    }

    #[test]
    fn test_verify_webhook_signature_invalid() {
        let body = b"test payload";
        let secret = "test-secret";
        let wrong_signature = "0000000000000000000000000000000000000000000000000000000000000000";

        assert!(!verify_webhook_signature(body, wrong_signature, secret));
    }

    #[test]
    fn test_verify_webhook_signature_malformed() {
        let body = b"test payload";
        let secret = "test-secret";

        // Not valid hex
        assert!(!verify_webhook_signature(body, "not-hex", secret));
    }

    #[test]
    fn test_validate_timestamp_valid() {
        let now_ms = chrono::Utc::now().timestamp_millis();
        assert!(validate_webhook_timestamp(now_ms, 60_000));
        assert!(validate_webhook_timestamp(now_ms - 30_000, 60_000));
    }

    #[test]
    fn test_validate_timestamp_stale() {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let stale_ms = now_ms - 120_000; // 2 minutes ago
        assert!(!validate_webhook_timestamp(stale_ms, 60_000));
    }

    #[test]
    fn test_parse_webhook_payload() {
        let json = r#"{
            "action": "created",
            "type": "AgentSessionEvent",
            "webhookTimestamp": 1733482800000,
            "webhookId": "webhook-123",
            "agentSession": {
                "id": "session-456",
                "issue": {
                    "id": "issue-789",
                    "identifier": "TSK-1",
                    "title": "Test Issue"
                }
            }
        }"#;

        let payload: WebhookPayload = serde_json::from_str(json).unwrap();

        assert!(payload.is_new_session());
        assert_eq!(payload.get_session_id(), Some("session-456"));
        assert_eq!(
            payload.get_issue().map(|i| i.identifier.as_str()),
            Some("TSK-1")
        );
    }
}
