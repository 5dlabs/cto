//! GraphQL client for Linear Agent Activity API.
//!
//! This module provides a lightweight client focused on agent activity emission.
//! For full Linear API operations (issues, projects, etc.), use the PM crate.

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
use tracing::instrument;

use crate::activities::{
    ActivityContent, AgentActivityCreateInput, AgentActivityCreateResponse,
    AgentSessionUpdateInput, AgentSessionUpdateResponse, PlanStep, AGENT_ACTIVITY_CREATE_MUTATION,
    AGENT_SESSION_UPDATE_MUTATION,
};

/// GraphQL mutation to create an agent session on an issue
const AGENT_SESSION_CREATE_ON_ISSUE_MUTATION: &str = r"
    mutation AgentSessionCreateOnIssue($input: AgentSessionCreateOnIssueInput!) {
        agentSessionCreateOnIssue(input: $input) {
            success
            agentSession {
                id
            }
        }
    }
";

/// GraphQL query to get issue ID by identifier
const GET_ISSUE_BY_IDENTIFIER_QUERY: &str = r"
    query GetIssueByIdentifier($id: String!) {
        issue(id: $id) {
            id
        }
    }
";

/// Linear API endpoint
const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// Linear GraphQL client for agent activities
#[derive(Debug, Clone)]
pub struct LinearClient {
    client: reqwest::Client,
    api_url: String,
}

/// GraphQL request body
#[derive(Debug, Serialize)]
struct GraphQLRequest<V: Serialize> {
    query: &'static str,
    variables: V,
}

/// GraphQL response wrapper
#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

impl LinearClient {
    /// Create a new Linear client with access token.
    ///
    /// # Arguments
    /// * `access_token` - OAuth access token or Personal API key
    ///   - OAuth tokens: Use "Bearer" prefix (handled automatically)
    ///   - API keys (`lin_api_*`): Use token directly without prefix
    pub fn new(access_token: &str) -> Result<Self> {
        Self::with_url(access_token, LINEAR_API_URL)
    }

    /// Create a client with custom API URL (for testing)
    pub fn with_url(access_token: &str, api_url: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();

        // Linear API keys (lin_api_*) should NOT use Bearer prefix
        // OAuth tokens should use Bearer prefix
        let auth_value = if access_token.starts_with("lin_api_") {
            access_token.to_string()
        } else {
            format!("Bearer {access_token}")
        };

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).context("Invalid access token")?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_url: api_url.to_string(),
        })
    }

    /// Execute a GraphQL query/mutation
    async fn execute<V: Serialize, R: DeserializeOwned>(
        &self,
        query: &'static str,
        variables: V,
    ) -> Result<R> {
        let request = GraphQLRequest { query, variables };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Linear API")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Linear API returned error status {status}: {body}"));
        }

        let gql_response: GraphQLResponse<R> = response
            .json()
            .await
            .context("Failed to parse Linear API response")?;

        if let Some(errors) = gql_response.errors {
            let error_messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            return Err(anyhow!("GraphQL errors: {}", error_messages.join(", ")));
        }

        gql_response
            .data
            .ok_or_else(|| anyhow!("No data in GraphQL response"))
    }

    // =========================================================================
    // Agent Activity Operations
    // =========================================================================

    /// Emit an agent activity
    #[instrument(skip(self, input), fields(session_id = %input.agent_session_id))]
    pub async fn emit_activity(&self, input: AgentActivityCreateInput) -> Result<String> {
        #[derive(Serialize)]
        struct Variables {
            input: AgentActivityCreateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "agentActivityCreate")]
            agent_activity_create: AgentActivityCreateResponse,
        }

        let response: Response = self
            .execute(AGENT_ACTIVITY_CREATE_MUTATION, Variables { input })
            .await?;

        if !response.agent_activity_create.success {
            return Err(anyhow!("Failed to emit agent activity"));
        }

        response
            .agent_activity_create
            .agent_activity
            .map(|a| a.id)
            .ok_or_else(|| anyhow!("Activity ID not returned"))
    }

    /// Emit a thought activity
    pub async fn emit_thought(&self, session_id: &str, body: impl Into<String>) -> Result<String> {
        let input = AgentActivityCreateInput::new(session_id, ActivityContent::thought(body));
        self.emit_activity(input).await
    }

    /// Emit an ephemeral thought activity
    pub async fn emit_ephemeral_thought(
        &self,
        session_id: &str,
        body: impl Into<String>,
    ) -> Result<String> {
        let input =
            AgentActivityCreateInput::new(session_id, ActivityContent::thought(body)).ephemeral();
        self.emit_activity(input).await
    }

    /// Emit an action activity (in progress)
    pub async fn emit_action(
        &self,
        session_id: &str,
        action: impl Into<String>,
        parameter: impl Into<String>,
    ) -> Result<String> {
        let input =
            AgentActivityCreateInput::new(session_id, ActivityContent::action(action, parameter));
        self.emit_activity(input).await
    }

    /// Emit an action activity with result
    pub async fn emit_action_with_result(
        &self,
        session_id: &str,
        action: impl Into<String>,
        parameter: impl Into<String>,
        result: impl Into<String>,
    ) -> Result<String> {
        let input = AgentActivityCreateInput::new(
            session_id,
            ActivityContent::action_with_result(action, parameter, result),
        );
        self.emit_activity(input).await
    }

    /// Emit a response activity (completion)
    pub async fn emit_response(&self, session_id: &str, body: impl Into<String>) -> Result<String> {
        let input = AgentActivityCreateInput::new(session_id, ActivityContent::response(body));
        self.emit_activity(input).await
    }

    /// Emit an error activity
    pub async fn emit_error(&self, session_id: &str, body: impl Into<String>) -> Result<String> {
        let input = AgentActivityCreateInput::new(session_id, ActivityContent::error(body));
        self.emit_activity(input).await
    }

    /// Emit an elicitation activity (request user input)
    pub async fn emit_elicitation(
        &self,
        session_id: &str,
        body: impl Into<String>,
    ) -> Result<String> {
        let input = AgentActivityCreateInput::new(session_id, ActivityContent::elicitation(body));
        self.emit_activity(input).await
    }

    // =========================================================================
    // Agent Plan Operations
    // =========================================================================

    /// Update the agent session plan.
    ///
    /// Plans are visual checklists shown in Linear UI.
    /// The plan array replaces the existing plan entirely.
    #[instrument(skip(self))]
    pub async fn update_plan(&self, session_id: &str, plan: Vec<PlanStep>) -> Result<bool> {
        #[derive(Serialize)]
        struct Variables {
            id: String,
            input: AgentSessionUpdateInput,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            agent_session_update: AgentSessionUpdateResponse,
        }

        let variables = Variables {
            id: session_id.to_string(),
            input: AgentSessionUpdateInput::with_plan(plan),
        };

        let response: Response = self
            .execute(AGENT_SESSION_UPDATE_MUTATION, variables)
            .await?;
        Ok(response.agent_session_update.success)
    }

    /// Set an external link for the session.
    ///
    /// This URL opens the session in your dashboard when clicked.
    /// Setting an external link also prevents the session from being marked unresponsive.
    #[instrument(skip(self, url))]
    pub async fn set_session_external_link(
        &self,
        session_id: &str,
        url: impl Into<String>,
    ) -> Result<bool> {
        #[derive(Serialize)]
        struct Variables {
            id: String,
            input: AgentSessionUpdateInput,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            agent_session_update: AgentSessionUpdateResponse,
        }

        let variables = Variables {
            id: session_id.to_string(),
            input: AgentSessionUpdateInput::with_external_link(url),
        };

        let response: Response = self
            .execute(AGENT_SESSION_UPDATE_MUTATION, variables)
            .await?;
        Ok(response.agent_session_update.success)
    }

    // =========================================================================
    // Session Creation Operations
    // =========================================================================

    /// Get issue ID by its identifier (e.g., "CTOPA-123")
    #[instrument(skip(self))]
    pub async fn get_issue_id_by_identifier(&self, identifier: &str) -> Result<String> {
        #[derive(Serialize)]
        struct Variables {
            id: String,
        }

        #[derive(Deserialize)]
        struct Issue {
            id: String,
        }

        #[derive(Deserialize)]
        struct Response {
            issue: Issue,
        }

        let variables = Variables {
            id: identifier.to_string(),
        };

        let response: Response = self
            .execute(GET_ISSUE_BY_IDENTIFIER_QUERY, variables)
            .await
            .context("Failed to get issue by identifier")?;

        Ok(response.issue.id)
    }

    /// Create an agent session on an issue.
    ///
    /// # Arguments
    /// * `issue_id` - The Linear issue UUID (not the identifier like "CTOPA-123")
    ///
    /// # Returns
    /// The created session ID
    #[instrument(skip(self))]
    pub async fn create_session_on_issue(&self, issue_id: &str) -> Result<String> {
        #[derive(Serialize)]
        struct Input {
            #[serde(rename = "issueId")]
            issue_id: String,
        }

        #[derive(Serialize)]
        struct Variables {
            input: Input,
        }

        #[derive(Deserialize)]
        struct AgentSession {
            id: String,
        }

        #[derive(Deserialize)]
        struct SessionCreateResponse {
            success: bool,
            #[serde(rename = "agentSession")]
            agent_session: Option<AgentSession>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            agent_session_create_on_issue: SessionCreateResponse,
        }

        let variables = Variables {
            input: Input {
                issue_id: issue_id.to_string(),
            },
        };

        let response: Response = self
            .execute(AGENT_SESSION_CREATE_ON_ISSUE_MUTATION, variables)
            .await
            .context("Failed to create session on issue")?;

        if !response.agent_session_create_on_issue.success {
            return Err(anyhow!("Failed to create agent session"));
        }

        response
            .agent_session_create_on_issue
            .agent_session
            .map(|s| s.id)
            .ok_or_else(|| anyhow!("Session ID not returned"))
    }

    /// Create a session on an issue by identifier (e.g., "CTOPA-123").
    ///
    /// Convenience method that looks up the issue ID first.
    #[instrument(skip(self))]
    pub async fn create_session_on_issue_by_identifier(&self, identifier: &str) -> Result<String> {
        let issue_id = self.get_issue_id_by_identifier(identifier).await?;
        self.create_session_on_issue(&issue_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let result = LinearClient::new("test-token");
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_key_auth_format() {
        // API keys should not have Bearer prefix
        let client = LinearClient::new("lin_api_test123").unwrap();
        assert!(format!("{client:?}").contains("lin_api"));

        // OAuth tokens should have Bearer prefix
        let client = LinearClient::new("oauth_token_test").unwrap();
        assert!(format!("{client:?}").contains("oauth_token"));
    }

    #[test]
    fn test_graphql_request_serialization() {
        #[derive(Serialize)]
        struct TestVars {
            id: String,
        }

        let request = GraphQLRequest {
            query: "query { test }",
            variables: TestVars {
                id: "test-id".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("query"));
        assert!(json.contains("test-id"));
    }
}
