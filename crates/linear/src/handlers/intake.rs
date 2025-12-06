//! Intake workflow handler for Linear integration.
//!
//! Handles the intake workflow triggered by delegating a PRD issue to the CTO agent.

use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{Api, ObjectMeta, PostParams},
    Client as KubeClient,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use tracing::{error, info, warn};

use crate::config::IntakeConfig;
use crate::models::{Issue, IssueCreateInput, IssueRelationCreateInput, IssueRelationType};
use crate::LinearClient;

/// Intake request extracted from a Linear webhook.
#[derive(Debug, Clone)]
pub struct IntakeRequest {
    /// Linear session ID for activity updates.
    pub session_id: String,
    /// PRD issue ID.
    pub prd_issue_id: String,
    /// PRD issue identifier (e.g., "TSK-1").
    pub prd_identifier: String,
    /// Team ID for creating task issues.
    pub team_id: String,
    /// PRD title.
    pub title: String,
    /// PRD content (issue description).
    pub prd_content: String,
    /// Architecture content (from linked documents).
    pub architecture_content: Option<String>,
    /// GitHub repository URL.
    pub repository_url: Option<String>,
    /// Source branch.
    pub source_branch: Option<String>,
}

/// Task from intake workflow output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeTask {
    /// Task ID.
    pub id: i32,
    /// Task title.
    pub title: String,
    /// Task description.
    pub description: String,
    /// Detailed implementation notes.
    #[serde(default)]
    pub details: String,
    /// Dependencies (list of task IDs).
    #[serde(default)]
    pub dependencies: Vec<i32>,
    /// Priority (1=highest, 5=lowest).
    #[serde(default)]
    pub priority: i32,
    /// Test strategy.
    #[serde(default, rename = "testStrategy")]
    pub test_strategy: String,
    /// Agent hint for assignment.
    #[serde(default, rename = "agentHint")]
    pub agent_hint: String,
}

/// `tasks.json` structure from intake output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksJson {
    /// List of generated tasks.
    pub tasks: Vec<IntakeTask>,
}

/// Result of the intake workflow.
#[derive(Debug, Clone)]
pub struct IntakeResult {
    /// Workflow name.
    pub workflow_name: String,
    /// `ConfigMap` name containing intake data.
    pub configmap_name: String,
}

/// Extract intake request from a Linear issue.
///
/// This function extracts the PRD content and any linked documents from the issue.
pub fn extract_intake_request(session_id: &str, issue: &Issue) -> Result<IntakeRequest> {
    let team_id = issue
        .team
        .as_ref()
        .ok_or_else(|| anyhow!("Issue has no team"))?
        .id
        .clone();

    let prd_content = issue
        .description
        .clone()
        .ok_or_else(|| anyhow!("PRD issue has no description"))?;

    // For now, we don't have a way to link documents directly.
    // TODO: Parse document links from description (e.g., Linear document URLs).
    let architecture_content = None;

    Ok(IntakeRequest {
        session_id: session_id.to_string(),
        prd_issue_id: issue.id.clone(),
        prd_identifier: issue.identifier.clone(),
        team_id,
        title: issue.title.clone(),
        prd_content,
        architecture_content,
        repository_url: None, // Will be extracted from Linear project settings or provided
        source_branch: None,  // Default to main
    })
}

/// Submit an intake workflow to Kubernetes.
///
/// Creates a `ConfigMap` with PRD content and submits an Argo workflow.
#[allow(clippy::too_many_lines)]
pub async fn submit_intake_workflow(
    kube_client: &KubeClient,
    namespace: &str,
    request: &IntakeRequest,
    config: &IntakeConfig,
) -> Result<IntakeResult> {
    let timestamp = chrono::Utc::now().timestamp();
    let project_name = sanitize_project_name(&request.prd_identifier);
    let configmap_name = format!("intake-linear-{project_name}-{timestamp}");
    let workflow_name = format!("intake-linear-{project_name}-{timestamp}");

    // Prepare config JSON for the workflow.
    let config_json = serde_json::json!({
        "project_name": project_name,
        "repository_url": request.repository_url.clone().unwrap_or_default(),
        "github_app": config.github_app,
        "primary_model": config.primary_model,
        "research_model": config.research_model,
        "fallback_model": config.fallback_model,
        "primary_provider": config.primary_provider,
        "research_provider": config.research_provider,
        "fallback_provider": config.fallback_provider,
        "model": config.primary_model,
        "num_tasks": config.num_tasks,
        "expand_tasks": config.expand_tasks,
        "analyze_complexity": config.analyze_complexity,
        "docs_model": config.docs_model,
        "enrich_context": config.enrich_context,
        "include_codebase": config.include_codebase,
        "cli": config.cli,
        // Linear-specific metadata for callbacks.
        "linear_session_id": request.session_id,
        "linear_issue_id": request.prd_issue_id,
        "linear_team_id": request.team_id,
    });

    // Create ConfigMap with PRD content.
    let mut data = BTreeMap::new();
    data.insert("prd.txt".to_string(), request.prd_content.clone());
    data.insert(
        "architecture.md".to_string(),
        request.architecture_content.clone().unwrap_or_default(),
    );
    data.insert("config.json".to_string(), config_json.to_string());

    let configmap = ConfigMap {
        metadata: ObjectMeta {
            name: Some(configmap_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                ("app.kubernetes.io/name".to_string(), "cto-intake".to_string()),
                (
                    "app.kubernetes.io/component".to_string(),
                    "intake".to_string(),
                ),
                ("cto.5dlabs.io/source".to_string(), "linear".to_string()),
                (
                    "cto.5dlabs.io/linear-issue".to_string(),
                    request.prd_identifier.clone(),
                ),
            ])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    };

    let cm_api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), namespace);
    cm_api
        .create(&PostParams::default(), &configmap)
        .await
        .context("Failed to create intake ConfigMap")?;

    info!(configmap_name = %configmap_name, "Created intake ConfigMap");

    // Submit Argo workflow.
    let repository_url = request
        .repository_url
        .clone()
        .unwrap_or_else(|| "https://github.com/5dlabs/cto".to_string());
    let source_branch = request
        .source_branch
        .clone()
        .unwrap_or_else(|| "main".to_string());

    let output = tokio::process::Command::new("argo")
        .args([
            "submit",
            "--from",
            "workflowtemplate/project-intake",
            "-n",
            namespace,
            "--name",
            &workflow_name,
            "-p",
            &format!("configmap-name={configmap_name}"),
            "-p",
            &format!("project-name={project_name}"),
            "-p",
            &format!("repository-url={repository_url}"),
            "-p",
            &format!("source-branch={source_branch}"),
            "-p",
            &format!("github-app={}", config.github_app),
            "-p",
            &format!("primary-model={}", config.primary_model),
            "-p",
            &format!("research-model={}", config.research_model),
            "-p",
            &format!("fallback-model={}", config.fallback_model),
            "-p",
            &format!("primary-provider={}", config.primary_provider),
            "-p",
            &format!("research-provider={}", config.research_provider),
            "-p",
            &format!("fallback-provider={}", config.fallback_provider),
            "-p",
            &format!("num-tasks={}", config.num_tasks),
            "-p",
            &format!("expand-tasks={}", config.expand_tasks),
            "-p",
            &format!("analyze-complexity={}", config.analyze_complexity),
            "-p",
            &format!("docs-model={}", config.docs_model),
            "-p",
            &format!("enrich-context={}", config.enrich_context),
            "-p",
            &format!("include-codebase={}", config.include_codebase),
            "-p",
            &format!("cli={}", config.cli),
            "--wait=false",
        ])
        .output()
        .await
        .context("Failed to execute argo submit command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to submit Argo workflow: {stderr}"));
    }

    info!(workflow_name = %workflow_name, "Submitted intake workflow");

    Ok(IntakeResult {
        workflow_name,
        configmap_name,
    })
}

/// Create Linear issues from intake tasks.
///
/// This function creates sub-issues under the PRD issue for each generated task.
pub async fn create_task_issues(
    client: &LinearClient,
    request: &IntakeRequest,
    tasks: &[IntakeTask],
) -> Result<HashMap<i32, String>> {
    let mut task_issue_map = HashMap::new();

    // Get workflow states for the team.
    let states = client.get_team_workflow_states(&request.team_id).await?;
    let initial_state = states
        .iter()
        .find(|s| s.state_type == "unstarted")
        .ok_or_else(|| anyhow!("No unstarted state found for team"))?;

    // Get or create labels.
    let cto_task_label = client
        .get_or_create_label(&request.team_id, "cto-task")
        .await?;

    info!(
        task_count = tasks.len(),
        parent_issue = %request.prd_identifier,
        "Creating task issues"
    );

    // Create issues for each task.
    for task in tasks {
        let priority_label_name = match task.priority {
            1 => "priority:urgent",
            2 => "priority:high",
            4 | 5 => "priority:low",
            _ => "priority:normal",
        };

        let priority_label = client
            .get_or_create_label(&request.team_id, priority_label_name)
            .await
            .ok();

        // Format task description with details.
        let description = format_task_description(task);

        let mut label_ids = vec![cto_task_label.id.clone()];
        if let Some(label) = priority_label {
            label_ids.push(label.id);
        }

        let input = IssueCreateInput {
            team_id: request.team_id.clone(),
            title: format!("Task {}: {}", task.id, task.title),
            description: Some(description),
            parent_id: Some(request.prd_issue_id.clone()),
            priority: Some(task.priority),
            label_ids: Some(label_ids),
            project_id: None,
            state_id: Some(initial_state.id.clone()),
        };

        match client.create_issue(input).await {
            Ok(issue) => {
                info!(
                    task_id = task.id,
                    issue_identifier = %issue.identifier,
                    "Created task issue"
                );
                task_issue_map.insert(task.id, issue.id);
            }
            Err(e) => {
                error!(task_id = task.id, error = %e, "Failed to create task issue");
            }
        }
    }

    // Create dependency relationships.
    for task in tasks {
        if task.dependencies.is_empty() {
            continue;
        }

        let Some(issue_id) = task_issue_map.get(&task.id) else {
            continue;
        };

        for dep_id in &task.dependencies {
            let Some(dep_issue_id) = task_issue_map.get(dep_id) else {
                warn!(
                    task_id = task.id,
                    dep_id = dep_id,
                    "Dependency task issue not found"
                );
                continue;
            };

            let input = IssueRelationCreateInput {
                issue_id: issue_id.clone(),
                related_issue_id: dep_issue_id.clone(),
                relation_type: IssueRelationType::BlockedBy,
            };

            if let Err(e) = client.create_issue_relation(input).await {
                warn!(
                    task_id = task.id,
                    dep_id = dep_id,
                    error = %e,
                    "Failed to create dependency relation"
                );
            }
        }
    }

    Ok(task_issue_map)
}

/// Format task description for Linear issue.
fn format_task_description(task: &IntakeTask) -> String {
    use std::fmt::Write;

    let mut description = format!("## Description\n\n{}\n\n", task.description);

    if !task.details.is_empty() {
        let _ = write!(description, "## Details\n\n{}\n\n", task.details);
    }

    if !task.test_strategy.is_empty() {
        let _ = write!(description, "## Test Strategy\n\n{}\n\n", task.test_strategy);
    }

    if !task.dependencies.is_empty() {
        let deps: Vec<String> = task
            .dependencies
            .iter()
            .map(|d| format!("Task {d}"))
            .collect();
        let _ = write!(
            description,
            "## Dependencies\n\nThis task depends on: {}\n\n",
            deps.join(", ")
        );
    }

    if !task.agent_hint.is_empty() {
        let _ = write!(description, "---\n\n*Agent hint: {}*\n", task.agent_hint);
    }

    description
}

/// Generate intake completion summary.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn generate_completion_summary(
    request: &IntakeRequest,
    tasks: &[IntakeTask],
    task_issue_map: &HashMap<i32, String>,
) -> String {
    let task_count = tasks.len();
    let high_priority = tasks.iter().filter(|t| t.priority <= 2).count();
    let with_deps = tasks.iter().filter(|t| !t.dependencies.is_empty()).count();
    let issues_created = task_issue_map.len();

    format!(
        r"## ✅ Intake Complete

### Summary
- **PRD**: {}
- **Tasks Generated**: {}
- **High Priority**: {}
- **Tasks with Dependencies**: {}
- **Linear Issues Created**: {}

### Next Steps
1. Review the generated task issues below
2. Adjust priorities or details as needed
3. When ready, delegate a task to `@CTO-Agent` to begin implementation

### Generated Tasks
{}
",
        request.prd_identifier,
        task_count,
        high_priority,
        with_deps,
        issues_created,
        format_task_list(tasks)
    )
}

/// Format task list for summary.
fn format_task_list(tasks: &[IntakeTask]) -> String {
    tasks
        .iter()
        .map(|t| {
            let deps = if t.dependencies.is_empty() {
                String::new()
            } else {
                format!(" (depends on: {:?})", t.dependencies)
            };
            format!(
                "- **Task {}** [P{}]: {}{}\n  {}",
                t.id, t.priority, t.title, deps, t.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sanitize project name for Kubernetes resource names.
fn sanitize_project_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_task_description() {
        let task = IntakeTask {
            id: 1,
            title: "Test Task".to_string(),
            description: "Test description".to_string(),
            details: "Implementation details".to_string(),
            dependencies: vec![2, 3],
            priority: 2,
            test_strategy: "Unit tests".to_string(),
            agent_hint: "rex".to_string(),
        };

        let description = format_task_description(&task);
        assert!(description.contains("Test description"));
        assert!(description.contains("Implementation details"));
        assert!(description.contains("Unit tests"));
        assert!(description.contains("Task 2"));
        assert!(description.contains("rex"));
    }

    #[test]
    fn test_deserialize_tasks_json() {
        let json = r#"{
            "tasks": [
                {
                    "id": 1,
                    "title": "Setup project",
                    "description": "Initialize the project",
                    "details": "",
                    "dependencies": [],
                    "priority": 1,
                    "testStrategy": "Integration test",
                    "agentHint": "rex"
                }
            ]
        }"#;

        let tasks: TasksJson = serde_json::from_str(json).unwrap();
        assert_eq!(tasks.tasks.len(), 1);
        assert_eq!(tasks.tasks[0].id, 1);
        assert_eq!(tasks.tasks[0].title, "Setup project");
    }

    #[test]
    fn test_sanitize_project_name() {
        assert_eq!(sanitize_project_name("TSK-123"), "tsk-123");
        assert_eq!(sanitize_project_name("My Project!"), "my-project");
        assert_eq!(sanitize_project_name("test_name"), "test-name");
    }
}

