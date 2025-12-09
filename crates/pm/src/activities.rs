//! Agent Activity emission for Linear's agent system.

use serde::{Deserialize, Serialize};

/// Agent activity content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ActivityContent {
    /// A thought or internal note
    Thought {
        /// Thought body (markdown supported)
        body: String,
    },
    /// A tool invocation or action
    Action {
        /// Action name (e.g., "Searching", "Running tests")
        action: String,
        /// Action parameter (e.g., search query, file path)
        parameter: String,
        /// Action result (optional, set when action completes)
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    /// Request for user input or clarification
    Elicitation {
        /// Elicitation body (markdown supported)
        body: String,
    },
    /// Final response or completion message
    Response {
        /// Response body (markdown supported)
        body: String,
    },
    /// Error report
    Error {
        /// Error body (markdown supported)
        body: String,
    },
}

impl ActivityContent {
    /// Create a thought activity
    #[must_use]
    pub fn thought(body: impl Into<String>) -> Self {
        Self::Thought { body: body.into() }
    }

    /// Create an action activity (in progress)
    #[must_use]
    pub fn action(action: impl Into<String>, parameter: impl Into<String>) -> Self {
        Self::Action {
            action: action.into(),
            parameter: parameter.into(),
            result: None,
        }
    }

    /// Create an action activity with result
    #[must_use]
    pub fn action_with_result(
        action: impl Into<String>,
        parameter: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self::Action {
            action: action.into(),
            parameter: parameter.into(),
            result: Some(result.into()),
        }
    }

    /// Create an elicitation activity
    #[must_use]
    pub fn elicitation(body: impl Into<String>) -> Self {
        Self::Elicitation { body: body.into() }
    }

    /// Create a response activity
    #[must_use]
    pub fn response(body: impl Into<String>) -> Self {
        Self::Response { body: body.into() }
    }

    /// Create an error activity
    #[must_use]
    pub fn error(body: impl Into<String>) -> Self {
        Self::Error { body: body.into() }
    }
}

/// Agent-to-human signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivitySignal {
    /// Request account linking
    Auth,
    /// Present selection options
    Select,
}

/// Signal metadata for auth signal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSignalMetadata {
    /// URL for account linking
    pub url: String,
    /// Optional: Restrict to specific user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Optional: Provider name for display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
}

/// Selection option for select signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    /// Option value
    pub value: String,
    /// Optional label (if different from value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Signal metadata for select signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectSignalMetadata {
    /// Available options
    pub options: Vec<SelectOption>,
}

/// Signal metadata union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalMetadata {
    /// Auth signal metadata
    Auth(AuthSignalMetadata),
    /// Select signal metadata
    Select(SelectSignalMetadata),
}

/// Input for creating an agent activity
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentActivityCreateInput {
    /// Agent session ID
    pub agent_session_id: String,
    /// Activity content
    pub content: ActivityContent,
    /// Optional signal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<ActivitySignal>,
    /// Optional signal metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_metadata: Option<SignalMetadata>,
    /// Whether the activity is ephemeral (replaced by next activity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
}

impl AgentActivityCreateInput {
    /// Create a new activity input
    #[must_use]
    pub fn new(session_id: impl Into<String>, content: ActivityContent) -> Self {
        Self {
            agent_session_id: session_id.into(),
            content,
            signal: None,
            signal_metadata: None,
            ephemeral: None,
        }
    }

    /// Mark the activity as ephemeral
    #[must_use]
    pub fn ephemeral(mut self) -> Self {
        self.ephemeral = Some(true);
        self
    }

    /// Add auth signal
    #[must_use]
    pub fn with_auth_signal(mut self, url: impl Into<String>) -> Self {
        self.signal = Some(ActivitySignal::Auth);
        self.signal_metadata = Some(SignalMetadata::Auth(AuthSignalMetadata {
            url: url.into(),
            user_id: None,
            provider_name: None,
        }));
        self
    }

    /// Add select signal
    #[must_use]
    pub fn with_select_signal(mut self, options: Vec<SelectOption>) -> Self {
        self.signal = Some(ActivitySignal::Select);
        self.signal_metadata = Some(SignalMetadata::Select(SelectSignalMetadata { options }));
        self
    }
}

/// GraphQL mutation for creating an agent activity
pub const AGENT_ACTIVITY_CREATE_MUTATION: &str = r"
mutation AgentActivityCreate($input: AgentActivityCreateInput!) {
    agentActivityCreate(input: $input) {
        success
        agentActivity {
            id
        }
    }
}
";

/// Response from agent activity creation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentActivityCreateResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Created activity
    #[serde(default)]
    pub agent_activity: Option<CreatedAgentActivity>,
}

/// Created agent activity
#[derive(Debug, Clone, Deserialize)]
pub struct CreatedAgentActivity {
    /// Activity ID
    pub id: String,
}

// ============================================================================
// Agent Plan API
// ============================================================================

/// Agent plan step status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanStepStatus {
    /// Not yet started
    Pending,
    /// Currently in progress
    InProgress,
    /// Successfully completed
    Completed,
    /// Cancelled or skipped
    Canceled,
}

/// A single step in an agent plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step description
    pub content: String,
    /// Step status
    pub status: PlanStepStatus,
}

impl PlanStep {
    /// Create a new pending step
    #[must_use]
    pub fn pending(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: PlanStepStatus::Pending,
        }
    }

    /// Create a new in-progress step
    #[must_use]
    pub fn in_progress(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: PlanStepStatus::InProgress,
        }
    }

    /// Create a new completed step
    #[must_use]
    pub fn completed(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: PlanStepStatus::Completed,
        }
    }

    /// Create a new cancelled step
    #[must_use]
    pub fn canceled(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: PlanStepStatus::Canceled,
        }
    }
}

/// Input for updating an agent session (including plan)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionUpdateInput {
    /// Updated plan (replaces existing plan entirely)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<Vec<PlanStep>>,
    /// External URL for the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
}

impl AgentSessionUpdateInput {
    /// Create input with just a plan update
    #[must_use]
    pub fn with_plan(plan: Vec<PlanStep>) -> Self {
        Self {
            plan: Some(plan),
            external_url: None,
        }
    }

    /// Create input with just an external URL
    #[must_use]
    pub fn with_external_url(url: impl Into<String>) -> Self {
        Self {
            plan: None,
            external_url: Some(url.into()),
        }
    }
}

/// GraphQL mutation for updating an agent session
pub const AGENT_SESSION_UPDATE_MUTATION: &str = r"
mutation AgentSessionUpdate($id: String!, $input: AgentSessionUpdateInput!) {
    agentSessionUpdate(id: $id, input: $input) {
        success
    }
}
";

/// Response from agent session update
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionUpdateResponse {
    /// Whether the operation succeeded
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_content_serialization() {
        let thought = ActivityContent::thought("Analyzing the PRD...");
        let json = serde_json::to_string(&thought).unwrap();
        assert!(json.contains("\"type\":\"thought\""));
        assert!(json.contains("Analyzing the PRD"));

        let action =
            ActivityContent::action_with_result("Running tests", "src/lib.rs", "24/24 passed");
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"type\":\"action\""));
        assert!(json.contains("\"result\":\"24/24 passed\""));
    }

    #[test]
    fn test_activity_input_builder() {
        let input =
            AgentActivityCreateInput::new("session-123", ActivityContent::thought("Processing..."))
                .ephemeral();

        assert_eq!(input.agent_session_id, "session-123");
        assert_eq!(input.ephemeral, Some(true));
    }

    #[test]
    fn test_select_signal() {
        let input = AgentActivityCreateInput::new(
            "session-123",
            ActivityContent::elicitation("Which repository?"),
        )
        .with_select_signal(vec![
            SelectOption {
                value: "repo-a".to_string(),
                label: None,
            },
            SelectOption {
                value: "repo-b".to_string(),
                label: Some("Repository B".to_string()),
            },
        ]);

        assert_eq!(input.signal, Some(ActivitySignal::Select));
        assert!(input.signal_metadata.is_some());
    }
}
