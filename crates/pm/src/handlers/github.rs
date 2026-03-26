//! GitHub webhook handlers.
//!
//! Handles GitHub webhook events, particularly PR merge events
//! for creating Linear projects from merged intake PRs.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use kube::{api::PostParams, Api};
use scm::ScmClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::callbacks::CallbackState;
use crate::models::{IssueCreateInput, ProjectCreateInput};
use crate::LinearClient;

/// GitHub PR event payload (simplified)
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestEvent {
    /// Action type (opened, closed, merged, etc.)
    pub action: String,
    /// Pull request details
    pub pull_request: PullRequest,
    /// Repository info
    pub repository: Repository,
    /// Sender (user who triggered the event)
    #[serde(default)]
    pub sender: Option<GitHubUser>,
}

/// GitHub Pull Request
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    /// PR number
    pub number: u64,
    /// PR title
    pub title: String,
    /// PR body/description
    #[serde(default)]
    pub body: Option<String>,
    /// Source branch
    pub head: GitRef,
    /// Target branch
    pub base: GitRef,
    /// Whether PR was merged
    #[serde(default)]
    pub merged: bool,
    /// Merge commit SHA
    #[serde(default)]
    pub merge_commit_sha: Option<String>,
    /// PR HTML URL
    pub html_url: String,
    /// PR state (open, closed)
    pub state: String,
    /// Labels on the PR
    #[serde(default)]
    pub labels: Vec<GitHubLabel>,
}

/// Git reference (branch)
#[derive(Debug, Clone, Deserialize)]
pub struct GitRef {
    /// Branch name
    #[serde(rename = "ref")]
    pub ref_name: String,
    /// SHA
    pub sha: String,
}

/// GitHub Repository
#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: u64,
    /// Repository name
    pub name: String,
    /// Full name (org/repo)
    pub full_name: String,
    /// Clone URL
    pub clone_url: String,
    /// HTML URL
    pub html_url: String,
    /// Default branch
    #[serde(default)]
    pub default_branch: Option<String>,
}

/// GitHub User
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    /// User login
    pub login: String,
    /// User ID
    pub id: u64,
}

/// GitHub Label
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubLabel {
    /// Label name
    pub name: String,
    /// Label color
    #[serde(default)]
    pub color: Option<String>,
}

/// Intake metadata stored in PR body or branch name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeMetadata {
    /// Linear session ID
    pub session_id: String,
    /// Linear issue ID (PRD)
    pub prd_issue_id: String,
    /// Linear issue identifier (e.g., "TSK-1")
    pub prd_identifier: String,
    /// Linear team ID
    pub team_id: String,
    /// Project name (defaults to repo name)
    #[serde(default)]
    pub project_name: Option<String>,
    /// Project directory name within repo (normalized project name)
    #[serde(default)]
    pub project_dir: Option<String>,
}

/// Handle GitHub webhook (Axum handler).
///
/// This is the legacy endpoint at `/webhooks/github` used by Argo Events.
/// New code should use the unified `/webhooks/github/events` endpoint which
/// calls [`handle_github_webhook_inner`] for intake PR processing.
#[allow(clippy::too_many_lines)]
pub async fn handle_github_webhook(
    State(state): State<Arc<CallbackState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!(
        event_type = %event_type,
        delivery_id = %delivery_id,
        "Received GitHub webhook (legacy endpoint)"
    );

    if event_type != "pull_request" {
        debug!(event_type = %event_type, "Ignoring non-pull_request event");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not_pull_request_event"
        })));
    }

    handle_github_webhook_inner(&state, &body).await
}

/// Inner handler for intake PR merges, callable from both the legacy Axum
/// endpoint and the unified GitHub Events handler.
///
/// Processes merged intake PRs: creates Linear project + issues, then
/// triggers the play CodeRun.
#[allow(clippy::too_many_lines)]
pub async fn handle_github_webhook_inner(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let payload: PullRequestEvent = serde_json::from_slice(body).map_err(|e| {
        error!(error = %e, "Failed to parse GitHub webhook payload");
        StatusCode::BAD_REQUEST
    })?;

    // Only process closed PRs that were merged
    if payload.action != "closed" || !payload.pull_request.merged {
        debug!(
            action = %payload.action,
            merged = payload.pull_request.merged,
            "Ignoring non-merged PR event"
        );
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not_merged_pr"
        })));
    }

    // Check if this is an intake PR (has cto-intake label or branch pattern)
    if !is_intake_pr(&payload) {
        debug!(
            pr_number = payload.pull_request.number,
            branch = %payload.pull_request.head.ref_name,
            "PR is not an intake PR"
        );
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not_intake_pr"
        })));
    }

    info!(
        pr_number = payload.pull_request.number,
        repo = %payload.repository.full_name,
        branch = %payload.pull_request.head.ref_name,
        "Processing merged intake PR"
    );

    handle_intake_pr_impl(state, &payload).await
}

/// Core intake PR processing: Linear project creation + play CodeRun trigger.
#[allow(clippy::too_many_lines)]
async fn handle_intake_pr_impl(
    state: &CallbackState,
    payload: &PullRequestEvent,
) -> Result<Json<Value>, StatusCode> {
    let Some(client) = &state.linear_client else {
        error!("Linear client not configured");
        return Ok(Json(json!({
            "status": "error",
            "error": "Linear client not configured"
        })));
    };

    let Some(metadata) = extract_intake_metadata(payload) else {
        warn!(
            pr_number = payload.pull_request.number,
            "Could not extract intake metadata from PR"
        );
        return Ok(Json(json!({
            "status": "error",
            "error": "Missing intake metadata in PR"
        })));
    };

    match create_project_from_intake(state, client, payload, &metadata).await {
        Ok(result) => {
            info!(
                project_id = %result.project_id,
                issue_count = result.issue_count,
                "Created Linear project from intake PR"
            );

            if let Err(e) = client
                .emit_response(
                    &metadata.session_id,
                    format!(
                        "## Intake Complete 🎉\n\n\
                         **Project:** [{}]({})\n\
                         **Issues Created:** {}\n\
                         **PR Merged:** [#{}]({})\n\n\
                         Starting play workflow...",
                        result.project_name,
                        result.project_url.as_deref().unwrap_or("#"),
                        result.issue_count,
                        payload.pull_request.number,
                        payload.pull_request.html_url
                    ),
                )
                .await
            {
                warn!(error = %e, "Failed to emit completion activity");
            }

            let play_result =
                trigger_play_workflow(state, client, payload, &metadata, &result).await;

            let workflow_info = match &play_result {
                Ok(wf) => {
                    info!(
                        workflow_name = %wf.workflow_name,
                        "Play workflow submitted successfully"
                    );
                    Some(json!({
                        "workflow_name": wf.workflow_name,
                        "status": "submitted"
                    }))
                }
                Err(e) => {
                    warn!(error = %e, "Failed to submit play workflow");
                    let _ = client
                        .emit_error(
                            &metadata.session_id,
                            format!("⚠️ Failed to start play workflow: {e}"),
                        )
                        .await;
                    None
                }
            };

            Ok(Json(json!({
                "status": "success",
                "action": "project_created",
                "project_id": result.project_id,
                "project_name": result.project_name,
                "issue_count": result.issue_count,
                "pr_number": payload.pull_request.number,
                "repository": payload.repository.full_name,
                "play_workflow": workflow_info
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to create project from intake PR");

            let _ = client
                .emit_error(
                    &metadata.session_id,
                    format!("Failed to create project from merged PR: {e}"),
                )
                .await;

            Ok(Json(json!({
                "status": "error",
                "error": format!("{}", e)
            })))
        }
    }
}

/// Check if a PR is an intake PR
fn is_intake_pr(payload: &PullRequestEvent) -> bool {
    // Check for cto-intake label
    let has_intake_label = payload
        .pull_request
        .labels
        .iter()
        .any(|l| l.name == "cto-intake" || l.name == "intake");

    // Check for intake branch pattern (e.g., intake/TSK-1-*, intake-project-*, cto-intake/*)
    let branch = &payload.pull_request.head.ref_name;
    let is_intake_branch = branch.starts_with("intake/")
        || branch.starts_with("intake-")
        || branch.starts_with("cto-intake/")
        || branch.contains("-intake-");

    // Check PR title for intake pattern
    let title = payload.pull_request.title.to_lowercase();
    let has_intake_title = title.contains("[intake]") || title.starts_with("intake:");

    has_intake_label || is_intake_branch || has_intake_title
}

/// Extract intake metadata from PR body
fn extract_intake_metadata(payload: &PullRequestEvent) -> Option<IntakeMetadata> {
    let body = payload.pull_request.body.as_deref().unwrap_or("");

    // Try to parse JSON metadata block from PR body
    // Look for <!-- intake-metadata: {...} --> or ```json intake-metadata\n...\n```
    if let Some(start) = body.find("<!-- intake-metadata:") {
        if let Some(end) = body[start..].find("-->") {
            let json_str = body[start + 21..start + end].trim();
            if let Ok(metadata) = serde_json::from_str::<IntakeMetadata>(json_str) {
                return Some(metadata);
            }
        }
    }

    // Try code block format
    if let Some(start) = body.find("```json intake-metadata") {
        if let Some(end) = body[start + 23..].find("```") {
            let json_str = body[start + 23..start + 23 + end].trim();
            if let Ok(metadata) = serde_json::from_str::<IntakeMetadata>(json_str) {
                return Some(metadata);
            }
        }
    }

    // Try to extract from branch name pattern: intake/{identifier}-{session_id_prefix}
    let branch = &payload.pull_request.head.ref_name;
    if let Some(branch_suffix) = branch.strip_prefix("intake/") {
        let parts: Vec<&str> = branch_suffix.split('-').collect();
        if parts.len() >= 2 {
            // This is a fallback - ideally metadata should be in PR body
            warn!(
                branch = %branch,
                "Using branch name for metadata extraction (incomplete)"
            );
            // Can't extract full metadata from branch alone
        }
    }

    None
}

/// Result of creating a project from intake
struct IntakeProjectResult {
    project_id: String,
    project_name: String,
    project_url: Option<String>,
    issue_count: usize,
}

/// Create a Linear project from a merged intake PR
#[allow(clippy::too_many_lines)] // Complex function not easily split
async fn create_project_from_intake(
    state: &CallbackState,
    client: &LinearClient,
    payload: &PullRequestEvent,
    metadata: &IntakeMetadata,
) -> anyhow::Result<IntakeProjectResult> {
    use anyhow::Context;

    // Determine project name and directory
    let project_name = metadata
        .project_name
        .clone()
        .unwrap_or_else(|| payload.repository.name.clone());

    // Derive project directory from metadata or project name
    let project_dir = metadata.project_dir.clone().unwrap_or_else(|| {
        // Normalize project name to directory format (lowercase, alphanumeric + hyphens)
        project_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    });

    info!(
        project_name = %project_name,
        project_dir = %project_dir,
        repo = %payload.repository.full_name,
        "Starting Linear project creation from intake PR"
    );

    // Step 1: Fetch tasks.json from the repository
    let tasks_json = fetch_tasks_json_from_repo(
        &state.http_client,
        state.github_token.as_deref(),
        &payload.repository.full_name,
        payload.pull_request.merge_commit_sha.as_deref(),
        &project_dir,
    )
    .await
    .context("Failed to fetch tasks.json from repository")?;

    let task_count = tasks_json.tasks.len();
    info!(
        task_count = task_count,
        "Fetched {task_count} tasks from tasks.json"
    );

    // Step 2: Create the Linear project
    // Try to find the "Play Workflow" template for consistent project structure
    let template_id = match client.find_project_template_by_name("Play Workflow").await {
        Ok(Some(template)) => {
            info!(template_id = %template.id, "Using Play Workflow template");
            Some(template.id)
        }
        Ok(None) => {
            debug!("Play Workflow template not found, creating project without template");
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to look up project template, continuing without");
            None
        }
    };

    // Try to find "Planned" project status for initial project state
    let status_id = match client.find_project_status_by_type("planned").await {
        Ok(Some(status)) => {
            info!(status_id = %status.id, status_name = %status.name, "Using 'Planned' project status");
            Some(status.id)
        }
        Ok(None) => {
            debug!("No 'planned' type project status found, project will use default status");
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to look up project status, continuing without");
            None
        }
    };

    let project_input = ProjectCreateInput {
        name: project_name.clone(),
        description: Some(format!(
            "Generated from intake PR [#{}]({}) in repository `{}`.\n\n\
             **PRD:** {}\n\
             **Repository:** [{}]({})\n\
             **Tasks:** {}",
            payload.pull_request.number,
            payload.pull_request.html_url,
            payload.repository.full_name,
            metadata.prd_identifier,
            payload.repository.name,
            payload.repository.html_url,
            task_count
        )),
        team_ids: Some(vec![metadata.team_id.clone()]),
        lead_id: None,
        target_date: None,
        template_id,
        status_id,
    };

    let project = client
        .create_project(project_input)
        .await
        .context("Failed to create Linear project")?;

    info!(
        project_id = %project.id,
        project_name = %project.name,
        "Created Linear project"
    );

    // Step 3: Create Linear issues for each task (linked to project, not as sub-issues)
    let created_issues =
        create_issues_from_tasks(client, &metadata.team_id, &project.id, &tasks_json.tasks)
            .await
            .context("Failed to create issues from tasks")?;

    let issue_count = created_issues.len();
    info!(
        issue_count = issue_count,
        project_id = %project.id,
        "Created {issue_count} Linear issues from tasks"
    );

    Ok(IntakeProjectResult {
        project_id: project.id,
        project_name: project.name,
        project_url: project.url,
        issue_count,
    })
}

/// Task from tasks.json
///
/// Note: `id` uses a custom deserializer to accept both string ("1") and numeric (1) IDs,
/// since different intake implementations may produce either format.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskFromJson {
    /// Task ID (accepts both string and numeric)
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Priority (1=urgent, 2=high, 3=normal, 4=low).
    /// Accepts both string ("medium", "high") and integer (1-4) formats.
    #[serde(default, deserialize_with = "deserialize_priority_flexible_option")]
    pub priority: Option<i32>,
    /// Dependencies (task IDs)
    #[serde(default, deserialize_with = "deserialize_string_or_number_vec")]
    pub dependencies: Vec<String>,
    /// Subtasks
    #[serde(default)]
    pub subtasks: Vec<SubtaskFromJson>,
    /// Agent hint for delegate assignment (e.g., "rex", "blaze", "bolt")
    #[serde(default, rename = "agentHint")]
    pub agent_hint: Option<String>,
}

/// Subtask from tasks.json
#[derive(Debug, Clone, Deserialize)]
pub struct SubtaskFromJson {
    /// Subtask ID (accepts both string and numeric)
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Subtask title
    pub title: String,
    /// Subtask description
    #[serde(default)]
    pub description: Option<String>,
}

/// Deserialize a value that could be either a string or a number into a String.
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrNumberVisitor;

    impl Visitor<'_> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a number")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

/// Deserialize a Vec where each element could be either a string or a number.
fn deserialize_string_or_number_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    // Deserialize as a Vec of JSON values first, then convert each
    let values: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
    values
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => Ok(s),
            serde_json::Value::Number(n) => Ok(n.to_string()),
            _ => Err(D::Error::custom("expected string or number")),
        })
        .collect()
}

/// Deserialize optional priority from either string ("medium") or integer (3).
/// Maps string priorities to Linear's integer format:
/// - "critical"/"urgent" → 1 (Urgent)
/// - "high" → 2 (High)
/// - "medium"/"normal" → 3 (Normal)
/// - "low" → 4 (Low)
fn deserialize_priority_flexible_option<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct PriorityVisitor;

    impl Visitor<'_> for PriorityVisitor {
        type Value = Option<i32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a priority string, integer, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(match value.to_lowercase().as_str() {
                "critical" | "urgent" => 1,
                "high" => 2,
                "low" => 4,
                // "medium", "normal", "med", or any unknown string defaults to normal priority
                _ => 3,
            }))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.clamp(0, 4) as i32))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.min(4) as i32))
        }
    }

    deserializer.deserialize_any(PriorityVisitor)
}

/// Create Linear issues from tasks.json content.
///
/// Issues are created as standalone issues linked to the project (not as sub-issues).
/// This enables proper use of Linear's project board view with workflow state columns.
pub async fn create_issues_from_tasks(
    client: &LinearClient,
    team_id: &str,
    project_id: &str,
    tasks: &[TaskFromJson],
) -> anyhow::Result<Vec<(String, String)>> {
    use std::fmt::Write;

    let mut created_issues = Vec::new();

    for task in tasks {
        // Build description with subtasks
        let mut description = task.description.clone();
        if !task.subtasks.is_empty() {
            description.push_str("\n\n## Subtasks\n");
            for subtask in &task.subtasks {
                let _ = writeln!(
                    description,
                    "- [ ] **{}**: {}",
                    subtask.title,
                    subtask.description.as_deref().unwrap_or("")
                );
            }
        }

        // Add dependencies info
        if !task.dependencies.is_empty() {
            description.push_str("\n\n## Dependencies\n");
            let _ = writeln!(
                description,
                "This task depends on tasks: {}",
                task.dependencies
                    .iter()
                    .map(|d| format!("#{d}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        // Include agent hint in description if available
        if let Some(ref agent) = task.agent_hint {
            use std::fmt::Write;
            let _ = writeln!(description, "\n\n## Agent\n**Assigned to:** {agent}");
        }

        // Create as standalone issue linked to project (not as sub-issue).
        // This enables proper board view in Linear with workflow state columns.
        //
        // Note: Delegate assignment happens when the Play workflow starts each task.
        // The PM service looks up the agent's Linear user ID and updates the issue.
        let input = IssueCreateInput {
            team_id: team_id.to_string(),
            title: format!("[Task {}] {}", task.id, task.title),
            description: Some(description),
            parent_id: None, // Not a sub-issue - linked via project instead
            priority: task.priority,
            label_ids: None,
            project_id: Some(project_id.to_string()),
            state_id: None,
            delegate_id: None, // Set by PM service when task starts
        };

        match client.create_issue(input).await {
            Ok(issue) => {
                info!(
                    task_id = %task.id,
                    issue_id = %issue.id,
                    issue_identifier = %issue.identifier,
                    "Created issue for task"
                );
                created_issues.push((task.id.clone(), issue.id));
            }
            Err(e) => {
                warn!(
                    task_id = %task.id,
                    error = %e,
                    "Failed to create issue for task"
                );
            }
        }
    }

    Ok(created_issues)
}

/// Root structure of tasks.json
#[derive(Debug, Clone, Deserialize)]
pub struct TasksJson {
    /// Tasks array
    pub tasks: Vec<TaskFromJson>,
    /// Metadata
    #[serde(default)]
    pub metadata: Option<TasksJsonMetadata>,
}

/// Metadata from tasks.json
#[derive(Debug, Clone, Deserialize)]
pub struct TasksJsonMetadata {
    /// Total task count
    #[serde(default)]
    pub task_count: Option<u32>,
    /// Version
    #[serde(default)]
    pub version: Option<String>,
}

/// GitHub content response for file fetching
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubContentResponse {
    /// Base64-encoded content
    content: Option<String>,
    /// Encoding type (usually "base64")
    encoding: Option<String>,
    /// File path
    path: Option<String>,
}

/// Fetch tasks.json from a GitHub repository at a specific commit
///
/// Uses the GitHub Contents API to fetch the file at the merge commit SHA.
pub async fn fetch_tasks_json_from_repo(
    http_client: &reqwest::Client,
    github_token: Option<&str>,
    repo_full_name: &str,
    commit_sha: Option<&str>,
    project_dir: &str,
) -> anyhow::Result<TasksJson> {
    use anyhow::Context;
    use base64::Engine;

    // Build the path to tasks.json
    let file_path = format!("{project_dir}/.tasks/tasks/tasks.json");

    // Build URL with optional ref (commit SHA)
    let url = if let Some(sha) = commit_sha {
        format!("https://api.github.com/repos/{repo_full_name}/contents/{file_path}?ref={sha}")
    } else {
        format!("https://api.github.com/repos/{repo_full_name}/contents/{file_path}")
    };

    info!(
        url = %url,
        repo = %repo_full_name,
        project_dir = %project_dir,
        "Fetching tasks.json from GitHub"
    );

    // Build request
    let mut request = http_client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "cto-pm-server/1.0");

    // Add auth token if available
    if let Some(token) = github_token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    // Execute request
    let response = request
        .send()
        .await
        .context("Failed to send GitHub API request")?;

    // Check status
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API returned {status}: {error_body}");
    }

    // Parse response
    let content_response: GitHubContentResponse = response
        .json()
        .await
        .context("Failed to parse GitHub content response")?;

    // Decode base64 content
    let content = content_response
        .content
        .ok_or_else(|| anyhow::anyhow!("No content in GitHub response"))?;

    // GitHub returns base64 with newlines, so we need to strip them
    let content_clean: String = content.chars().filter(|c| !c.is_whitespace()).collect();

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&content_clean)
        .context("Failed to decode base64 content")?;

    let json_str = String::from_utf8(decoded).context("tasks.json is not valid UTF-8")?;

    // Parse tasks.json
    let tasks_json: TasksJson =
        serde_json::from_str(&json_str).context("Failed to parse tasks.json")?;

    info!(
        task_count = tasks_json.tasks.len(),
        "Successfully fetched tasks.json"
    );

    Ok(tasks_json)
}

/// Fetch tasks.json using the unified SCM client (supports GitHub and GitLab).
pub async fn fetch_tasks_json_from_scm(
    scm_client: &dyn ScmClient,
    repo_full_name: &str,
    commit_sha: Option<&str>,
    project_dir: &str,
) -> anyhow::Result<TasksJson> {
    use anyhow::Context;

    let (owner, repo) = scm_client
        .parse_repo_from_url(&scm_client.repo_url(
            repo_full_name.split('/').next().unwrap_or(""),
            repo_full_name.split('/').nth(1).unwrap_or(""),
        ))
        .unwrap_or_else(|_| {
            let parts: Vec<&str> = repo_full_name.splitn(2, '/').collect();
            (
                parts.first().unwrap_or(&"").to_string(),
                parts.get(1).unwrap_or(&"").to_string(),
            )
        });

    let file_path = format!("{project_dir}/.tasks/tasks/tasks.json");
    let ref_ = commit_sha.unwrap_or("main");

    info!(
        owner = %owner,
        repo = %repo,
        file_path = %file_path,
        ref_ = %ref_,
        "Fetching tasks.json via SCM client"
    );

    let content = scm_client
        .get_file_contents(&owner, &repo, &file_path, ref_)
        .await
        .context("Failed to fetch tasks.json via SCM")?;

    let json_str = String::from_utf8(content).context("tasks.json is not valid UTF-8")?;
    let tasks_json: TasksJson =
        serde_json::from_str(&json_str).context("Failed to parse tasks.json")?;

    info!(
        task_count = tasks_json.tasks.len(),
        "Fetched tasks.json via SCM"
    );
    Ok(tasks_json)
}

/// Result of triggering a play workflow.
#[derive(Debug)]
pub struct PlayTriggerResult {
    /// `CodeRun` name (replaces `workflow_name` for backward compatibility).
    pub workflow_name: String,
}

/// Trigger the play workflow after intake PR is merged.
///
/// Creates a `CodeRun` CRD with Morgan as the agent to start the play workflow.
/// This ensures the project-specific `ConfigMap` (`cto-config-project-{project_id}`)
/// is mounted, providing the correct agent assignments and settings.
#[allow(clippy::too_many_lines)] // Complex function not easily split
async fn trigger_play_workflow(
    state: &CallbackState,
    client: &LinearClient,
    payload: &PullRequestEvent,
    metadata: &IntakeMetadata,
    project_result: &IntakeProjectResult,
) -> anyhow::Result<PlayTriggerResult> {
    use anyhow::Context;

    let namespace = &state.namespace;

    let repository = format!("https://github.com/{}", payload.repository.full_name);

    // Determine project directory (normalize project name)
    let project_dir = metadata.project_dir.clone().unwrap_or_else(|| {
        metadata
            .project_name
            .clone()
            .unwrap_or_else(|| payload.repository.name.clone())
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    });

    // Determine docs directory - use "." for new repos where tasks are at root
    // New repos have the same name as the normalized project (repo was created for this project)
    let docs_project_dir = if project_dir == payload.repository.name {
        info!(
            project_dir = %project_dir,
            repo_name = %payload.repository.name,
            "New repo detected - using '.' for docs directory"
        );
        ".".to_string()
    } else {
        info!(
            project_dir = %project_dir,
            repo_name = %payload.repository.name,
            "Existing repo - using project subdirectory for docs"
        );
        project_dir.clone()
    };

    // Generate deterministic CodeRun name using PR number + merge SHA so that
    // duplicate webhook deliveries are idempotent (Kubernetes rejects duplicates).
    let merge_sha7 = payload
        .pull_request
        .merge_commit_sha
        .as_deref()
        .map_or("unknown", |s| &s[..7.min(s.len())]);
    let name_suffix = project_dir
        .chars()
        .take(20)
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let coderun_name = format!(
        "play-{name_suffix}-pr{}-{merge_sha7}",
        payload.pull_request.number
    );

    info!(
        coderun_name = %coderun_name,
        repository = %repository,
        project_dir = %project_dir,
        project_id = %project_result.project_id,
        "Creating play CodeRun (project ConfigMap will be mounted)"
    );

    // Emit activity before submission
    let _ = client
        .emit_response(
            &metadata.session_id,
            format!(
                "## 🚀 Starting Play Workflow\n\n\
                 **CodeRun:** `{coderun_name}`\n\
                 **Repository:** {repository}\n\
                 **Project:** {project_dir}\n\n\
                 Morgan will start the play workflow with project-specific settings..."
            ),
        )
        .await;

    // Build CodeRun CRD manifest
    // Using Morgan as the agent with runType: "play" to trigger the play.md.hbs template
    // LINEAR_PROJECT_ID is set to trigger ConfigMap mounting in the controller
    let coderun_json = serde_json::json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": coderun_name,
            "namespace": namespace,
            "labels": {
                "workflow-type": "play",
                "project-name": name_suffix,
                "github-app": "5DLabs-Morgan",
                "cto.5dlabs.io/linear-issue": metadata.prd_identifier,
                "cto.5dlabs.io/source": "github-pr-merge"
            }
        },
        "spec": {
            "runType": "play",
            "service": project_dir,
            "repositoryUrl": repository,
            "docsRepositoryUrl": repository,
            "docsProjectDirectory": docs_project_dir,
            "workingDirectory": ".",
            "githubApp": "5DLabs-Morgan",
            "model": state.play_config.model.as_deref().unwrap_or("claude-sonnet-4-20250514"),
            "enableDocker": false,
            "env": {
                "PROJECT_NAME": metadata.project_name.clone().unwrap_or_else(|| payload.repository.name.clone()),
                "PROJECT_DIR": project_dir,
                "REPOSITORY_URL": repository,
                // This triggers ConfigMap mounting in the controller
                "LINEAR_PROJECT_ID": project_result.project_id
            },
            "linearIntegration": {
                "enabled": true,
                "sessionId": metadata.session_id,
                "issueId": metadata.prd_issue_id,
                "teamId": metadata.team_id,
                "projectId": project_result.project_id
            }
        }
    });

    // Create CodeRun via kube client (works both in-cluster and locally with kubeconfig)
    let coderun_gvk = kube::api::GroupVersionKind::gvk("agents.platform", "v1", "CodeRun");
    let coderun_api_resource = kube::api::ApiResource::from_gvk(&coderun_gvk);
    let coderuns: Api<kube::api::DynamicObject> =
        Api::namespaced_with(state.kube_client.clone(), namespace, &coderun_api_resource);

    let coderun_obj: kube::api::DynamicObject =
        serde_json::from_value(coderun_json).context("Failed to parse CodeRun JSON")?;

    match coderuns.create(&PostParams::default(), &coderun_obj).await {
        Ok(_) => {
            info!(
                coderun_name = %coderun_name,
                project_id = %project_result.project_id,
                "Play CodeRun created successfully (ConfigMap will be mounted)"
            );
        }
        Err(kube::Error::Api(ref api_err)) if api_err.reason == "AlreadyExists" => {
            // Duplicate webhook delivery — CodeRun already created from a prior
            // delivery of the same event. This is expected and safe to ignore.
            info!(
                coderun_name = %coderun_name,
                "Play CodeRun already exists (duplicate webhook delivery — idempotent)"
            );
        }
        Err(e) => return Err(e).context("Failed to create play CodeRun"),
    }

    Ok(PlayTriggerResult {
        workflow_name: coderun_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_task_priority_string_medium() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "medium"
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, Some(3)); // medium = normal = 3
    }

    #[test]
    fn test_deserialize_task_priority_string_high() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "high"
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, Some(2)); // high = 2
    }

    #[test]
    fn test_deserialize_task_priority_string_critical() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "critical"
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, Some(1)); // critical = urgent = 1
    }

    #[test]
    fn test_deserialize_task_priority_string_low() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "low"
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, Some(4)); // low = 4
    }

    #[test]
    fn test_deserialize_task_priority_integer() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": 2
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, Some(2)); // Integer passed through
    }

    #[test]
    fn test_deserialize_task_priority_missing() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test"
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, None); // No default, returns None
    }

    #[test]
    fn test_deserialize_task_priority_case_insensitive() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "HIGH"
        }"#;

        let task: TaskFromJson = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, Some(2)); // HIGH = high = 2
    }
}
