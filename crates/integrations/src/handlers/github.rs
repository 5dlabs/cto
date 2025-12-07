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
}

/// Handle GitHub webhook
pub async fn handle_github_webhook(
    State(state): State<Arc<CallbackState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    // Get event type from header
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
        "Received GitHub webhook"
    );

    // Only process pull_request events
    if event_type != "pull_request" {
        debug!(event_type = %event_type, "Ignoring non-pull_request event");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not_pull_request_event"
        })));
    }

    // Parse payload
    let payload: PullRequestEvent = serde_json::from_slice(&body).map_err(|e| {
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
    let is_intake_pr = is_intake_pr(&payload);

    if !is_intake_pr {
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

    // Get Linear client
    let Some(client) = &state.linear_client else {
        error!("Linear client not configured");
        return Ok(Json(json!({
            "status": "error",
            "error": "Linear client not configured"
        })));
    };

    // Extract metadata from PR
    let Some(metadata) = extract_intake_metadata(&payload) else {
        warn!(
            pr_number = payload.pull_request.number,
            "Could not extract intake metadata from PR"
        );
        return Ok(Json(json!({
            "status": "error",
            "error": "Missing intake metadata in PR"
        })));
    };

    // Create project and issues
    match create_project_from_intake(client, &payload, &metadata).await {
        Ok(result) => {
            info!(
                project_id = %result.project_id,
                issue_count = result.issue_count,
                "Created Linear project from intake PR"
            );

            // Emit activity to Linear session
            if let Err(e) = client
                .emit_response(
                    &metadata.session_id,
                    format!(
                        "## Intake Complete 🎉\n\n\
                         **Project:** [{}]({})\n\
                         **Issues Created:** {}\n\
                         **PR Merged:** [#{}]({})\n\n\
                         Your tasks are ready for implementation!",
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

            Ok(Json(json!({
                "status": "success",
                "action": "project_created",
                "project_id": result.project_id,
                "project_name": result.project_name,
                "issue_count": result.issue_count,
                "pr_number": payload.pull_request.number,
                "repository": payload.repository.full_name
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to create project from intake PR");

            // Emit error to Linear session
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

    // Check for intake branch pattern (e.g., intake/TSK-1-*, cto-intake/*)
    let branch = &payload.pull_request.head.ref_name;
    let is_intake_branch = branch.starts_with("intake/")
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
            let json_str = &body[start + 21..start + end].trim();
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
async fn create_project_from_intake(
    client: &LinearClient,
    payload: &PullRequestEvent,
    metadata: &IntakeMetadata,
) -> anyhow::Result<IntakeProjectResult> {
    // Determine project name
    let project_name = metadata
        .project_name
        .clone()
        .unwrap_or_else(|| payload.repository.name.clone());

    // Create the project
    let project_input = ProjectCreateInput {
        name: project_name.clone(),
        description: Some(format!(
            "Generated from intake PR [#{}]({}) in repository `{}`.\n\n\
             **PRD:** {}\n\
             **Repository:** [{}]({})",
            payload.pull_request.number,
            payload.pull_request.html_url,
            payload.repository.full_name,
            metadata.prd_identifier,
            payload.repository.name,
            payload.repository.html_url
        )),
        team_ids: Some(vec![metadata.team_id.clone()]),
        lead_id: None,
        target_date: None,
    };

    let project = client.create_project(project_input).await?;

    info!(
        project_id = %project.id,
        project_name = %project.name,
        "Created Linear project"
    );

    // Now we need to get tasks.json from the merged PR
    // For now, we'll create a placeholder - in practice this would be fetched
    // from the repository or passed via the callback
    let issue_count = 0; // TODO: Parse tasks.json and create issues

    // In a full implementation, we would:
    // 1. Fetch tasks.json from the repository at the merge commit
    // 2. Parse the tasks
    // 3. Create Linear issues for each task
    // 4. Link them to the project

    Ok(IntakeProjectResult {
        project_id: project.id,
        project_name: project.name,
        project_url: project.url,
        issue_count,
    })
}

/// Task from tasks.json
#[derive(Debug, Clone, Deserialize)]
pub struct TaskFromJson {
    /// Task ID
    pub id: u32,
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Priority (1=highest, 4=lowest)
    #[serde(default)]
    pub priority: Option<i32>,
    /// Dependencies (task IDs)
    #[serde(default)]
    pub dependencies: Vec<u32>,
    /// Subtasks
    #[serde(default)]
    pub subtasks: Vec<SubtaskFromJson>,
}

/// Subtask from tasks.json
#[derive(Debug, Clone, Deserialize)]
pub struct SubtaskFromJson {
    /// Subtask ID
    pub id: u32,
    /// Subtask title
    pub title: String,
    /// Subtask description
    #[serde(default)]
    pub description: Option<String>,
}

/// Create Linear issues from tasks.json content
pub async fn create_issues_from_tasks(
    client: &LinearClient,
    team_id: &str,
    project_id: &str,
    parent_issue_id: &str,
    tasks: &[TaskFromJson],
) -> anyhow::Result<Vec<(u32, String)>> {
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

        let input = IssueCreateInput {
            team_id: team_id.to_string(),
            title: format!("[Task {}] {}", task.id, task.title),
            description: Some(description),
            parent_id: Some(parent_issue_id.to_string()),
            priority: task.priority,
            label_ids: None,
            project_id: Some(project_id.to_string()),
            state_id: None,
        };

        match client.create_issue(input).await {
            Ok(issue) => {
                info!(
                    task_id = task.id,
                    issue_id = %issue.id,
                    issue_identifier = %issue.identifier,
                    "Created issue for task"
                );
                created_issues.push((task.id, issue.id));
            }
            Err(e) => {
                warn!(
                    task_id = task.id,
                    error = %e,
                    "Failed to create issue for task"
                );
            }
        }
    }

    Ok(created_issues)
}

