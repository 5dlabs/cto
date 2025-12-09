//! Play workflow handler for Linear integration.
//!
//! Handles triggering play workflows when a task issue is delegated to the CTO agent.

use anyhow::{anyhow, Context, Result};
use tracing::{info, warn};

use crate::config::{CtoConfig, PlayConfig};
use crate::handlers::intake::extract_cto_config;
use crate::models::Issue;

/// Play request extracted from a Linear webhook.
#[derive(Debug, Clone)]
pub struct PlayRequest {
    /// Linear session ID for activity updates.
    pub session_id: String,
    /// Task issue ID.
    pub task_issue_id: String,
    /// Task issue identifier (e.g., "TSK-2").
    pub task_identifier: String,
    /// Team ID.
    pub team_id: String,
    /// Task ID extracted from issue.
    pub task_id: Option<u32>,
    /// Task title.
    pub title: String,
    /// Task description.
    pub description: Option<String>,
    /// GitHub repository URL (if known).
    pub repository_url: Option<String>,
    /// CTO configuration from labels/frontmatter.
    pub cto_config: CtoConfig,
}

/// Result of submitting a play workflow.
#[derive(Debug, Clone)]
pub struct PlayResult {
    /// Workflow name.
    pub workflow_name: String,
    /// Task ID.
    pub task_id: u32,
}

/// Extract play request from a Linear issue.
pub fn extract_play_request(session_id: &str, issue: &Issue) -> Result<PlayRequest> {
    let team_id = issue
        .team
        .as_ref()
        .ok_or_else(|| anyhow!("Issue has no team"))?
        .id
        .clone();

    // Try to extract task ID from issue title (e.g., "Task 1: ..." or "Task-1: ...")
    let task_id = extract_task_id_from_title(&issue.title);

    // Extract CTO config from labels and frontmatter
    let cto_config = extract_cto_config(issue);

    Ok(PlayRequest {
        session_id: session_id.to_string(),
        task_issue_id: issue.id.clone(),
        task_identifier: issue.identifier.clone(),
        team_id,
        task_id,
        title: issue.title.clone(),
        description: issue.description.clone(),
        repository_url: None, // Will be extracted from project settings or provided
        cto_config,
    })
}

/// Extract task ID from issue title.
///
/// Supported formats:
/// - "Task 1: Title" -> 1
/// - "Task-1: Title" -> 1
/// - "Task #1: Title" -> 1
/// - "[Task 1] Title" -> 1
fn extract_task_id_from_title(title: &str) -> Option<u32> {
    // Pattern: "Task N:" or "Task-N:" or "Task #N:" or "[Task N]"
    let patterns = [
        r"Task\s+(\d+)",
        r"Task-(\d+)",
        r"Task\s*#(\d+)",
        r"\[Task\s*(\d+)\]",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(title) {
                if let Some(id_match) = caps.get(1) {
                    if let Ok(id) = id_match.as_str().parse::<u32>() {
                        return Some(id);
                    }
                }
            }
        }
    }

    None
}

/// Submit a play workflow to Kubernetes.
#[allow(clippy::too_many_lines)]
pub async fn submit_play_workflow(
    namespace: &str,
    request: &PlayRequest,
    config: &PlayConfig,
) -> Result<PlayResult> {
    let task_id = request.task_id.ok_or_else(|| {
        anyhow!(
            "Could not determine task ID from issue. \
             Please include task ID in the title (e.g., 'Task 1: ...')"
        )
    })?;

    let repository = request
        .repository_url
        .clone()
        .or_else(|| config.repository.clone())
        .ok_or_else(|| {
            anyhow!(
                "No repository configured. \
                 Please set DEFAULT_REPOSITORY or provide repository URL"
            )
        })?;

    let timestamp = chrono::Utc::now().timestamp();
    let workflow_name = format!("play-linear-{task_id}-{timestamp}");

    // Apply CTO config overrides from labels/frontmatter
    let model = request.cto_config.model.as_deref().unwrap_or(&config.model);

    if !request.cto_config.is_empty() {
        info!(
            model = %model,
            cli = ?request.cto_config.cli,
            "Using CTO config overrides from issue"
        );
    }

    // Build workflow parameters
    let workflow_template = if config.parallel_execution {
        "workflowtemplate/play-project-workflow-template"
    } else {
        "workflowtemplate/play-workflow-template"
    };

    let mut args = vec![
        "submit".to_string(),
        "--from".to_string(),
        workflow_template.to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "--name".to_string(),
        workflow_name.clone(),
        "-p".to_string(),
        format!("repository={repository}"),
        "-p".to_string(),
        format!("task-id={task_id}"),
        "-p".to_string(),
        format!("github-app={}", config.github_app),
        "-p".to_string(),
        format!("model={model}"),
        "-p".to_string(),
        format!("implementation-agent={}", config.implementation_agent),
        "-p".to_string(),
        format!("testing-agent={}", config.testing_agent),
        "-p".to_string(),
        format!("quality-agent={}", config.quality_agent),
        "-p".to_string(),
        format!("frontend-agent={}", config.frontend_agent),
        "-p".to_string(),
        format!("parallel-execution={}", config.parallel_execution),
        "-p".to_string(),
        format!("auto-merge={}", config.auto_merge),
        // Linear metadata for callbacks
        "-p".to_string(),
        format!("linear-session-id={}", request.session_id),
        "-p".to_string(),
        format!("linear-issue-id={}", request.task_issue_id),
        "-p".to_string(),
        format!("linear-team-id={}", request.team_id),
        "-l".to_string(),
        format!("linear-session={}", request.session_id),
        "-l".to_string(),
        "source=linear".to_string(),
        "--wait=false".to_string(),
    ];

    // Add docs project directory if configured
    if let Some(docs_dir) = &config.docs_project_directory {
        args.push("-p".to_string());
        args.push(format!("docs-project-directory={docs_dir}"));
    }

    info!(
        workflow_name = %workflow_name,
        task_id = task_id,
        repository = %repository,
        "Submitting play workflow"
    );

    let output = tokio::process::Command::new("argo")
        .args(&args)
        .output()
        .await
        .context("Failed to execute argo submit command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to submit play workflow: {stderr}"));
    }

    info!(
        workflow_name = %workflow_name,
        task_id = task_id,
        "Play workflow submitted successfully"
    );

    Ok(PlayResult {
        workflow_name,
        task_id,
    })
}

/// Cancel a running play workflow.
pub async fn cancel_play_workflow(namespace: &str, session_id: &str) -> Result<()> {
    // Find workflow by Linear session label
    let list_output = tokio::process::Command::new("argo")
        .args([
            "list",
            "-n",
            namespace,
            "-l",
            &format!("linear-session={session_id}"),
            "-o",
            "name",
        ])
        .output()
        .await
        .context("Failed to list workflows")?;

    if !list_output.status.success() {
        let stderr = String::from_utf8_lossy(&list_output.stderr);
        return Err(anyhow!("Failed to list workflows: {stderr}"));
    }

    let workflow_names = String::from_utf8_lossy(&list_output.stdout);
    let workflows: Vec<&str> = workflow_names.lines().filter(|l| !l.is_empty()).collect();

    if workflows.is_empty() {
        warn!(session_id = %session_id, "No active workflows found for session");
        return Ok(());
    }

    // Stop each workflow
    for workflow_name in workflows {
        info!(workflow_name = %workflow_name, "Stopping workflow");

        let stop_output = tokio::process::Command::new("argo")
            .args(["stop", "-n", namespace, workflow_name])
            .output()
            .await
            .context("Failed to stop workflow")?;

        if !stop_output.status.success() {
            let stderr = String::from_utf8_lossy(&stop_output.stderr);
            warn!(
                workflow_name = %workflow_name,
                error = %stderr,
                "Failed to stop workflow"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_task_id_from_title() {
        assert_eq!(extract_task_id_from_title("Task 1: Setup project"), Some(1));
        assert_eq!(extract_task_id_from_title("Task-42: Add feature"), Some(42));
        assert_eq!(extract_task_id_from_title("Task #123: Fix bug"), Some(123));
        assert_eq!(
            extract_task_id_from_title("[Task 5] Implement API"),
            Some(5)
        );
        assert_eq!(extract_task_id_from_title("Random title"), None);
        assert_eq!(extract_task_id_from_title("Task: No number"), None);
    }
}
