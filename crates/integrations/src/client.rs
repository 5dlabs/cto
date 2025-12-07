//! GraphQL client for Linear API.

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};

use crate::activities::{
    AgentActivityCreateInput, AgentActivityCreateResponse, AGENT_ACTIVITY_CREATE_MUTATION,
};
use crate::models::{
    AgentStatus, Attachment, AttachmentCreateInput, Comment, CommentCreateInput, Document, Issue,
    IssueCreateInput, IssueRelationCreateInput, IssueUpdateInput, Label, Project,
    ProjectCreateInput, Team, WorkflowState,
};

/// Linear API endpoint
const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// Linear GraphQL client
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
    ///
    /// # Errors
    /// Returns error if headers cannot be constructed
    pub fn new(access_token: &str) -> Result<Self> {
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
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_url: LINEAR_API_URL.to_string(),
        })
    }

    /// Create a client with custom API URL (for testing)
    #[cfg(test)]
    pub fn with_url(access_token: &str, api_url: &str) -> Result<Self> {
        let mut client = Self::new(access_token)?;
        client.api_url = api_url.to_string();
        Ok(client)
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
    // Issue Operations
    // =========================================================================

    /// Get an issue by ID
    #[instrument(skip(self), fields(issue_id = %issue_id))]
    pub async fn get_issue(&self, issue_id: &str) -> Result<Issue> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            issue: Issue,
        }

        const QUERY: &str = r"
            query GetIssue($id: String!) {
                issue(id: $id) {
                    id
                    identifier
                    title
                    description
                    url
                    priority
                    state {
                        id
                        name
                        type
                        position
                    }
                    team {
                        id
                        name
                        key
                    }
                    labels {
                        nodes {
                            id
                            name
                            color
                        }
                    }
                    attachments {
                        nodes {
                            id
                            title
                            url
                            sourceType
                        }
                    }
                    parent {
                        id
                    }
                    delegate {
                        id
                        name
                    }
                    assignee {
                        id
                        name
                    }
                    createdAt
                    updatedAt
                }
            }
        ";

        let response: Response = self.execute(QUERY, Variables { id: issue_id }).await?;
        debug!("Retrieved issue: {}", response.issue.identifier);
        Ok(response.issue)
    }

    /// Get the active agent session ID for an issue (if any).
    ///
    /// This checks if the issue has been delegated to an agent and returns
    /// the session ID if there's an active (not completed) session.
    #[instrument(skip(self), fields(issue_id = %issue_id))]
    pub async fn get_active_session_for_issue(&self, issue_id: &str) -> Result<Option<String>> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            issue: Option<IssueWithSessions>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct IssueWithSessions {
            agent_sessions: AgentSessionsConnection,
        }

        #[derive(Deserialize)]
        struct AgentSessionsConnection {
            nodes: Vec<AgentSessionNode>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AgentSessionNode {
            id: String,
            status: String,
        }

        const QUERY: &str = r"
            query GetIssueAgentSessions($id: String!) {
                issue(id: $id) {
                    agentSessions {
                        nodes {
                            id
                            status
                        }
                    }
                }
            }
        ";

        let response: Response = self.execute(QUERY, Variables { id: issue_id }).await?;

        // Find an active session (status is not "completed" or "cancelled")
        let active_session = response.issue.and_then(|i| {
            i.agent_sessions
                .nodes
                .into_iter()
                .find(|s| s.status != "completed" && s.status != "cancelled")
                .map(|s| s.id)
        });

        if let Some(ref session_id) = active_session {
            debug!(session_id = %session_id, "Found active agent session");
        } else {
            debug!("No active agent session found");
        }

        Ok(active_session)
    }

    /// Create a new issue
    #[instrument(skip(self, input), fields(title = %input.title))]
    pub async fn create_issue(&self, input: IssueCreateInput) -> Result<Issue> {
        #[derive(Serialize)]
        struct Variables {
            input: IssueCreateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueCreate")]
            issue_create: IssueCreateResult,
        }

        #[derive(Deserialize)]
        struct IssueCreateResult {
            success: bool,
            issue: Option<Issue>,
        }

        const MUTATION: &str = r"
            mutation CreateIssue($input: IssueCreateInput!) {
                issueCreate(input: $input) {
                    success
                    issue {
                        id
                        identifier
                        title
                        description
                        url
                        priority
                        state {
                            id
                            name
                            type
                        }
                        team {
                            id
                            name
                            key
                        }
                    }
                }
            }
        ";

        let response: Response = self.execute(MUTATION, Variables { input }).await?;

        if !response.issue_create.success {
            return Err(anyhow!("Failed to create issue"));
        }

        response
            .issue_create
            .issue
            .ok_or_else(|| anyhow!("Issue not returned after creation"))
    }

    /// Update an issue
    #[instrument(skip(self, input), fields(issue_id = %issue_id))]
    pub async fn update_issue(&self, issue_id: &str, input: IssueUpdateInput) -> Result<Issue> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
            input: IssueUpdateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueUpdate")]
            issue_update: IssueUpdateResult,
        }

        #[derive(Deserialize)]
        struct IssueUpdateResult {
            success: bool,
            issue: Option<Issue>,
        }

        const MUTATION: &str = r"
            mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
                issueUpdate(id: $id, input: $input) {
                    success
                    issue {
                        id
                        identifier
                        title
                        state {
                            id
                            name
                            type
                        }
                    }
                }
            }
        ";

        let response: Response = self
            .execute(
                MUTATION,
                Variables {
                    id: issue_id,
                    input,
                },
            )
            .await?;

        if !response.issue_update.success {
            return Err(anyhow!("Failed to update issue"));
        }

        response
            .issue_update
            .issue
            .ok_or_else(|| anyhow!("Issue not returned after update"))
    }

    /// Create an issue relation (blocking/blocked)
    #[instrument(skip(self, input))]
    pub async fn create_issue_relation(&self, input: IssueRelationCreateInput) -> Result<()> {
        #[derive(Serialize)]
        struct Variables {
            input: IssueRelationCreateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueRelationCreate")]
            issue_relation_create: SuccessResult,
        }

        #[derive(Deserialize)]
        struct SuccessResult {
            success: bool,
        }

        const MUTATION: &str = r"
            mutation CreateIssueRelation($input: IssueRelationCreateInput!) {
                issueRelationCreate(input: $input) {
                    success
                }
            }
        ";

        let response: Response = self.execute(MUTATION, Variables { input }).await?;

        if !response.issue_relation_create.success {
            return Err(anyhow!("Failed to create issue relation"));
        }

        Ok(())
    }

    // =========================================================================
    // Team Operations
    // =========================================================================

    /// Get a team by ID
    #[instrument(skip(self), fields(team_id = %team_id))]
    pub async fn get_team(&self, team_id: &str) -> Result<Team> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            team: Team,
        }

        const QUERY: &str = r"
            query GetTeam($id: String!) {
                team(id: $id) {
                    id
                    name
                    key
                }
            }
        ";

        let response: Response = self.execute(QUERY, Variables { id: team_id }).await?;
        Ok(response.team)
    }

    /// Get workflow states for a team
    #[instrument(skip(self), fields(team_id = %team_id))]
    pub async fn get_team_workflow_states(&self, team_id: &str) -> Result<Vec<WorkflowState>> {
        #[derive(Serialize)]
        struct Variables<'a> {
            #[serde(rename = "teamId")]
            team_id: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "workflowStates")]
            workflow_states: WorkflowStatesConnection,
        }

        #[derive(Deserialize)]
        struct WorkflowStatesConnection {
            nodes: Vec<WorkflowState>,
        }

        const QUERY: &str = r"
            query GetTeamWorkflowStates($teamId: ID!) {
                workflowStates(filter: { team: { id: { eq: $teamId } } }) {
                    nodes {
                        id
                        name
                        type
                        position
                    }
                }
            }
        ";

        let response: Response = self.execute(QUERY, Variables { team_id }).await?;
        Ok(response.workflow_states.nodes)
    }

    /// Get the first "started" workflow state for a team
    #[instrument(skip(self), fields(team_id = %team_id))]
    pub async fn get_started_state(&self, team_id: &str) -> Result<WorkflowState> {
        let states = self.get_team_workflow_states(team_id).await?;

        states
            .into_iter()
            .filter(|s| s.state_type == "started")
            .min_by(|a, b| {
                a.position
                    .partial_cmp(&b.position)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No started state found for team"))
    }

    /// Get workflow state by type for a team
    #[instrument(skip(self), fields(team_id = %team_id, state_type = %state_type))]
    pub async fn get_state_by_type(
        &self,
        team_id: &str,
        state_type: &str,
    ) -> Result<WorkflowState> {
        let states = self.get_team_workflow_states(team_id).await?;

        states
            .into_iter()
            .filter(|s| s.state_type == state_type)
            .min_by(|a, b| {
                a.position
                    .partial_cmp(&b.position)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No {state_type} state found for team"))
    }

    /// Get workflow state by name for a team
    #[instrument(skip(self), fields(team_id = %team_id, name = %name))]
    pub async fn get_state_by_name(
        &self,
        team_id: &str,
        name: &str,
    ) -> Result<Option<WorkflowState>> {
        let states = self.get_team_workflow_states(team_id).await?;

        Ok(states
            .into_iter()
            .find(|s| s.name.eq_ignore_ascii_case(name)))
    }

    /// Create a workflow state for a team
    ///
    /// # Arguments
    /// * `team_id` - The team ID
    /// * `name` - The state name (e.g., "Ready for Intake")
    /// * `state_type` - One of: backlog, unstarted, started, completed, canceled
    /// * `color` - Hex color code (e.g., "#5e6ad2")
    #[instrument(skip(self), fields(team_id = %team_id, name = %name, state_type = %state_type))]
    pub async fn create_workflow_state(
        &self,
        team_id: &str,
        name: &str,
        state_type: &str,
        color: &str,
    ) -> Result<WorkflowState> {
        #[derive(Serialize)]
        struct Variables<'a> {
            #[serde(rename = "teamId")]
            team_id: &'a str,
            name: &'a str,
            #[serde(rename = "type")]
            state_type: &'a str,
            color: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "workflowStateCreate")]
            workflow_state_create: WorkflowStateCreateResult,
        }

        #[derive(Deserialize)]
        struct WorkflowStateCreateResult {
            success: bool,
            #[serde(rename = "workflowState")]
            workflow_state: Option<WorkflowState>,
        }

        const MUTATION: &str = r"
            mutation CreateWorkflowState($teamId: String!, $name: String!, $type: String!, $color: String!) {
                workflowStateCreate(input: { teamId: $teamId, name: $name, type: $type, color: $color }) {
                    success
                    workflowState {
                        id
                        name
                        type
                        position
                    }
                }
            }
        ";

        let response: Response = self
            .execute(
                MUTATION,
                Variables {
                    team_id,
                    name,
                    state_type,
                    color,
                },
            )
            .await?;

        if !response.workflow_state_create.success {
            return Err(anyhow!("Failed to create workflow state"));
        }

        response
            .workflow_state_create
            .workflow_state
            .ok_or_else(|| anyhow!("Workflow state not returned after creation"))
    }

    /// Get or create a workflow state by name
    ///
    /// If a state with the given name exists, returns it.
    /// Otherwise, creates a new state with the specified type and color.
    #[instrument(skip(self), fields(team_id = %team_id, name = %name))]
    pub async fn get_or_create_workflow_state(
        &self,
        team_id: &str,
        name: &str,
        state_type: &str,
        color: &str,
    ) -> Result<WorkflowState> {
        // Check if state already exists
        if let Some(state) = self.get_state_by_name(team_id, name).await? {
            debug!("Found existing workflow state: {}", state.name);
            return Ok(state);
        }

        // Create new state
        debug!("Creating new workflow state: {name}");
        self.create_workflow_state(team_id, name, state_type, color)
            .await
    }

    // =========================================================================
    // Label Operations
    // =========================================================================

    /// Get or create a label by name
    #[instrument(skip(self), fields(team_id = %team_id, name = %name))]
    pub async fn get_or_create_label(&self, team_id: &str, name: &str) -> Result<Label> {
        // First try to find existing label
        #[derive(Serialize)]
        struct FindVariables<'a> {
            #[serde(rename = "teamId")]
            team_id: &'a str,
            name: &'a str,
        }

        #[derive(Deserialize)]
        struct FindResponse {
            #[serde(rename = "issueLabels")]
            issue_labels: LabelsConnection,
        }

        #[derive(Deserialize)]
        struct LabelsConnection {
            nodes: Vec<Label>,
        }

        const FIND_QUERY: &str = r"
            query FindLabel($teamId: String!, $name: String!) {
                issueLabels(filter: { 
                    team: { id: { eq: $teamId } },
                    name: { eq: $name }
                }) {
                    nodes {
                        id
                        name
                        color
                    }
                }
            }
        ";

        let find_response: FindResponse = self
            .execute(FIND_QUERY, FindVariables { team_id, name })
            .await?;

        if let Some(label) = find_response.issue_labels.nodes.into_iter().next() {
            return Ok(label);
        }

        // Create new label
        #[derive(Serialize)]
        struct CreateVariables<'a> {
            #[serde(rename = "teamId")]
            team_id: &'a str,
            name: &'a str,
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            #[serde(rename = "issueLabelCreate")]
            issue_label_create: LabelCreateResult,
        }

        #[derive(Deserialize)]
        struct LabelCreateResult {
            #[serde(rename = "issueLabel")]
            issue_label: Option<Label>,
        }

        const CREATE_MUTATION: &str = r"
            mutation CreateLabel($teamId: String!, $name: String!) {
                issueLabelCreate(input: { teamId: $teamId, name: $name }) {
                    success
                    issueLabel {
                        id
                        name
                        color
                    }
                }
            }
        ";

        let create_response: CreateResponse = self
            .execute(CREATE_MUTATION, CreateVariables { team_id, name })
            .await?;

        create_response
            .issue_label_create
            .issue_label
            .ok_or_else(|| anyhow!("Failed to create label"))
    }

    /// Get or create a workspace-level label by name with optional color.
    ///
    /// Workspace labels are available to all teams.
    #[instrument(skip(self), fields(name = %name))]
    pub async fn get_or_create_workspace_label(
        &self,
        name: &str,
        color: Option<&str>,
        description: Option<&str>,
    ) -> Result<Label> {
        // First try to find existing workspace label (no team filter)
        #[derive(Serialize)]
        struct FindVariables<'a> {
            name: &'a str,
        }

        #[derive(Deserialize)]
        struct FindResponse {
            #[serde(rename = "issueLabels")]
            issue_labels: LabelsConnection,
        }

        #[derive(Deserialize)]
        struct LabelsConnection {
            nodes: Vec<Label>,
        }

        const FIND_QUERY: &str = r"
            query FindWorkspaceLabel($name: String!) {
                issueLabels(filter: { 
                    name: { eq: $name }
                }) {
                    nodes {
                        id
                        name
                        color
                    }
                }
            }
        ";

        let find_response: FindResponse = self.execute(FIND_QUERY, FindVariables { name }).await?;

        if let Some(label) = find_response.issue_labels.nodes.into_iter().next() {
            return Ok(label);
        }

        // Create new workspace label (no teamId = workspace-level)
        #[derive(Serialize)]
        struct CreateVariables<'a> {
            name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            color: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            #[serde(rename = "issueLabelCreate")]
            issue_label_create: LabelCreateResult,
        }

        #[derive(Deserialize)]
        struct LabelCreateResult {
            success: bool,
            #[serde(rename = "issueLabel")]
            issue_label: Option<Label>,
        }

        const CREATE_MUTATION: &str = r"
            mutation CreateWorkspaceLabel($name: String!, $color: String, $description: String) {
                issueLabelCreate(input: { name: $name, color: $color, description: $description }) {
                    success
                    issueLabel {
                        id
                        name
                        color
                    }
                }
            }
        ";

        let create_response: CreateResponse = self
            .execute(
                CREATE_MUTATION,
                CreateVariables {
                    name,
                    color,
                    description,
                },
            )
            .await?;

        if create_response.issue_label_create.success {
            info!(name = %name, "Created workspace label");
        }

        create_response
            .issue_label_create
            .issue_label
            .ok_or_else(|| anyhow!("Failed to create workspace label: {name}"))
    }

    /// Ensure all CTO configuration labels exist in the workspace.
    ///
    /// Creates labels for CLI and model selection if they don't already exist.
    /// Should be called on server initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if label creation fails (typically due to permissions).
    #[instrument(skip(self))]
    pub async fn ensure_cto_config_labels(&self) -> Result<Vec<Label>> {
        use crate::handlers::intake::CTO_CONFIG_LABELS;

        let mut created_labels = Vec::new();

        for (name, color, description) in CTO_CONFIG_LABELS {
            match self
                .get_or_create_workspace_label(name, Some(color), Some(description))
                .await
            {
                Ok(label) => {
                    created_labels.push(label);
                }
                Err(e) => {
                    // Log but don't fail - label might already exist with different case
                    // or we might not have permissions
                    warn!(
                        name = %name,
                        error = %e,
                        "Failed to ensure CTO config label exists"
                    );
                }
            }
        }

        info!(
            count = created_labels.len(),
            "Ensured CTO config labels exist"
        );

        Ok(created_labels)
    }

    /// Set labels on an issue (replaces existing labels)
    #[instrument(skip(self), fields(issue_id = %issue_id))]
    pub async fn set_issue_labels(&self, issue_id: &str, label_ids: &[String]) -> Result<Issue> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
            #[serde(rename = "labelIds")]
            label_ids: &'a [String],
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueUpdate")]
            issue_update: IssueUpdateResult,
        }

        #[derive(Deserialize)]
        struct IssueUpdateResult {
            success: bool,
            issue: Option<Issue>,
        }

        const MUTATION: &str = r"
            mutation SetIssueLabels($id: String!, $labelIds: [String!]!) {
                issueUpdate(id: $id, input: { labelIds: $labelIds }) {
                    success
                    issue {
                        id
                        identifier
                        title
                        labels {
                            nodes {
                                id
                                name
                                color
                            }
                        }
                    }
                }
            }
        ";

        let response: Response = self
            .execute(
                MUTATION,
                Variables {
                    id: issue_id,
                    label_ids,
                },
            )
            .await?;

        if !response.issue_update.success {
            return Err(anyhow!("Failed to set issue labels"));
        }

        response
            .issue_update
            .issue
            .ok_or_else(|| anyhow!("Issue not returned after updating labels"))
    }

    /// Add a label to an issue (preserves existing labels)
    #[instrument(skip(self), fields(issue_id = %issue_id, label_id = %label_id))]
    pub async fn add_issue_label(&self, issue_id: &str, label_id: &str) -> Result<Issue> {
        // First get existing labels
        let issue = self.get_issue(issue_id).await?;
        let mut label_ids: Vec<String> = issue.labels.iter().map(|l| l.id.clone()).collect();

        // Add new label if not already present
        if !label_ids.contains(&label_id.to_string()) {
            label_ids.push(label_id.to_string());
        }

        self.set_issue_labels(issue_id, &label_ids).await
    }

    /// Remove a label from an issue
    #[instrument(skip(self), fields(issue_id = %issue_id, label_id = %label_id))]
    pub async fn remove_issue_label(&self, issue_id: &str, label_id: &str) -> Result<Issue> {
        // First get existing labels
        let issue = self.get_issue(issue_id).await?;
        let label_ids: Vec<String> = issue
            .labels
            .iter()
            .filter(|l| l.id != label_id)
            .map(|l| l.id.clone())
            .collect();

        self.set_issue_labels(issue_id, &label_ids).await
    }

    /// Update agent status label on an issue
    ///
    /// This removes any existing agent:* labels and adds the new one.
    /// Status labels: agent:pending, agent:working, agent:blocked, agent:pr-created, agent:complete, agent:error
    #[instrument(skip(self), fields(issue_id = %issue_id, team_id = %team_id, status = %status))]
    pub async fn update_agent_status_label(
        &self,
        issue_id: &str,
        team_id: &str,
        status: AgentStatus,
    ) -> Result<Issue> {
        // Get the issue to check existing labels
        let issue = self.get_issue(issue_id).await?;

        // Filter out existing agent:* labels and collect IDs
        let mut label_ids: Vec<String> = issue
            .labels
            .iter()
            .filter(|l| !l.name.starts_with("agent:"))
            .map(|l| l.id.clone())
            .collect();

        // Get or create the new status label
        let label_name = status.to_label_name();
        let new_label = self.get_or_create_label(team_id, label_name).await?;
        label_ids.push(new_label.id);

        self.set_issue_labels(issue_id, &label_ids).await
    }

    // =========================================================================
    // Project Operations
    // =========================================================================

    /// Create a new project
    #[instrument(skip(self, input), fields(name = %input.name))]
    pub async fn create_project(&self, input: ProjectCreateInput) -> Result<Project> {
        #[derive(Serialize)]
        struct Variables {
            input: ProjectCreateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "projectCreate")]
            project_create: ProjectCreateResult,
        }

        #[derive(Deserialize)]
        struct ProjectCreateResult {
            success: bool,
            project: Option<Project>,
        }

        const MUTATION: &str = r"
            mutation CreateProject($input: ProjectCreateInput!) {
                projectCreate(input: $input) {
                    success
                    project {
                        id
                        name
                        description
                        url
                    }
                }
            }
        ";

        let response: Response = self.execute(MUTATION, Variables { input }).await?;

        if !response.project_create.success {
            return Err(anyhow!("Failed to create project"));
        }

        response
            .project_create
            .project
            .ok_or_else(|| anyhow!("Project not returned after creation"))
    }

    /// Get a project by ID
    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn get_project(&self, project_id: &str) -> Result<Project> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            project: Project,
        }

        const QUERY: &str = r"
            query GetProject($id: String!) {
                project(id: $id) {
                    id
                    name
                    description
                    url
                }
            }
        ";

        let response: Response = self.execute(QUERY, Variables { id: project_id }).await?;
        Ok(response.project)
    }

    // =========================================================================
    // Document Operations
    // =========================================================================

    /// Get a document by ID
    #[instrument(skip(self), fields(document_id = %document_id))]
    pub async fn get_document(&self, document_id: &str) -> Result<Document> {
        #[derive(Serialize)]
        struct Variables<'a> {
            id: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            document: Document,
        }

        const QUERY: &str = r"
            query GetDocument($id: String!) {
                document(id: $id) {
                    id
                    title
                    content
                    url
                }
            }
        ";

        let response: Response = self.execute(QUERY, Variables { id: document_id }).await?;
        Ok(response.document)
    }

    // =========================================================================
    // Attachment Operations
    // =========================================================================

    /// Create an attachment on an issue
    #[instrument(skip(self, input))]
    pub async fn create_attachment(&self, input: AttachmentCreateInput) -> Result<Attachment> {
        #[derive(Serialize)]
        struct Variables {
            input: AttachmentCreateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "attachmentCreate")]
            attachment_create: AttachmentCreateResult,
        }

        #[derive(Deserialize)]
        struct AttachmentCreateResult {
            attachment: Option<Attachment>,
        }

        const MUTATION: &str = r"
            mutation CreateAttachment($input: AttachmentCreateInput!) {
                attachmentCreate(input: $input) {
                    success
                    attachment {
                        id
                        title
                        url
                    }
                }
            }
        ";

        let response: Response = self.execute(MUTATION, Variables { input }).await?;

        response
            .attachment_create
            .attachment
            .ok_or_else(|| anyhow!("Failed to create attachment"))
    }

    // =========================================================================
    // Comment Operations
    // =========================================================================

    /// Create a comment on an issue
    #[instrument(skip(self, input))]
    pub async fn create_comment(&self, input: CommentCreateInput) -> Result<Comment> {
        #[derive(Serialize)]
        struct Variables {
            input: CommentCreateInput,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "commentCreate")]
            comment_create: CommentCreateResult,
        }

        #[derive(Deserialize)]
        struct CommentCreateResult {
            comment: Option<Comment>,
        }

        const MUTATION: &str = r"
            mutation CreateComment($input: CommentCreateInput!) {
                commentCreate(input: $input) {
                    success
                    comment {
                        id
                        body
                        createdAt
                    }
                }
            }
        ";

        let response: Response = self.execute(MUTATION, Variables { input }).await?;

        response
            .comment_create
            .comment
            .ok_or_else(|| anyhow!("Failed to create comment"))
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
        let input = AgentActivityCreateInput::new(
            session_id,
            crate::activities::ActivityContent::thought(body),
        );
        self.emit_activity(input).await
    }

    /// Emit an ephemeral thought activity
    pub async fn emit_ephemeral_thought(
        &self,
        session_id: &str,
        body: impl Into<String>,
    ) -> Result<String> {
        let input = AgentActivityCreateInput::new(
            session_id,
            crate::activities::ActivityContent::thought(body),
        )
        .ephemeral();
        self.emit_activity(input).await
    }

    /// Emit an action activity (in progress)
    pub async fn emit_action(
        &self,
        session_id: &str,
        action: impl Into<String>,
        parameter: impl Into<String>,
    ) -> Result<String> {
        let input = AgentActivityCreateInput::new(
            session_id,
            crate::activities::ActivityContent::action(action, parameter),
        );
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
            crate::activities::ActivityContent::action_with_result(action, parameter, result),
        );
        self.emit_activity(input).await
    }

    /// Emit a response activity (completion)
    pub async fn emit_response(&self, session_id: &str, body: impl Into<String>) -> Result<String> {
        let input = AgentActivityCreateInput::new(
            session_id,
            crate::activities::ActivityContent::response(body),
        );
        self.emit_activity(input).await
    }

    /// Emit an error activity
    pub async fn emit_error(&self, session_id: &str, body: impl Into<String>) -> Result<String> {
        let input = AgentActivityCreateInput::new(
            session_id,
            crate::activities::ActivityContent::error(body),
        );
        self.emit_activity(input).await
    }

    /// Emit an elicitation activity (request user input)
    pub async fn emit_elicitation(
        &self,
        session_id: &str,
        body: impl Into<String>,
    ) -> Result<String> {
        let input = AgentActivityCreateInput::new(
            session_id,
            crate::activities::ActivityContent::elicitation(body),
        );
        self.emit_activity(input).await
    }

    // =========================================================================
    // Agent Plan Operations
    // =========================================================================

    /// Update the agent session plan.
    ///
    /// Plans are visual checklists shown in Linear UI.
    /// The plan array replaces the existing plan entirely.
    ///
    /// # Errors
    /// Returns error if the API call fails.
    #[instrument(skip(self))]
    pub async fn update_plan(
        &self,
        session_id: &str,
        plan: Vec<crate::activities::PlanStep>,
    ) -> Result<bool> {
        use crate::activities::{
            AgentSessionUpdateInput, AgentSessionUpdateResponse, AGENT_SESSION_UPDATE_MUTATION,
        };

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

    /// Set an external URL for the session.
    ///
    /// This URL opens the session in your dashboard when clicked.
    /// Setting an external URL also prevents the session from being marked unresponsive.
    ///
    /// # Errors
    /// Returns error if the API call fails.
    #[instrument(skip(self, url))]
    pub async fn set_session_external_url(
        &self,
        session_id: &str,
        url: impl Into<String>,
    ) -> Result<bool> {
        use crate::activities::{
            AgentSessionUpdateInput, AgentSessionUpdateResponse, AGENT_SESSION_UPDATE_MUTATION,
        };

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
            input: AgentSessionUpdateInput::with_external_url(url),
        };

        let response: Response = self
            .execute(AGENT_SESSION_UPDATE_MUTATION, variables)
            .await?;
        Ok(response.agent_session_update.success)
    }

    // =========================================================================
    // Viewer Operations
    // =========================================================================

    /// Get the current user/app info
    #[instrument(skip(self))]
    pub async fn get_viewer(&self) -> Result<crate::models::User> {
        #[derive(Deserialize)]
        struct Response {
            viewer: crate::models::User,
        }

        const QUERY: &str = r"
            query GetViewer {
                viewer {
                    id
                    name
                    email
                }
            }
        ";

        #[derive(Serialize)]
        struct EmptyVariables {}

        let response: Response = self.execute(QUERY, EmptyVariables {}).await?;
        Ok(response.viewer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests would require a real Linear API token
    // These are placeholder tests for compilation verification

    #[test]
    fn test_client_creation() {
        let result = LinearClient::new("test-token");
        assert!(result.is_ok());
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
