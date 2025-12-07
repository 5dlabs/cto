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

use crate::config::{CtoConfig, IntakeConfig};
use crate::models::{
    Issue, IssueCreateInput, IssueRelationCreateInput, IssueRelationType, Label, Project,
    ProjectCreateInput,
};
use crate::LinearClient;

/// Agent assignments for different task types.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Backend agent.
    #[serde(default)]
    pub backend: Option<String>,
    /// Frontend agent.
    #[serde(default)]
    pub frontend: Option<String>,
    /// Testing agent.
    #[serde(default)]
    pub testing: Option<String>,
    /// Quality agent.
    #[serde(default)]
    pub quality: Option<String>,
}

/// Tech stack configuration extracted from issue labels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechStack {
    /// Backend framework/language.
    #[serde(default)]
    pub backend: Option<String>,
    /// Frontend framework.
    #[serde(default)]
    pub frontend: Option<String>,
    /// Programming languages.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Agent assignments.
    #[serde(default)]
    pub agents: AgentConfig,
}

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
    /// Tech stack from labels.
    pub tech_stack: TechStack,
    /// CTO configuration from labels/frontmatter.
    pub cto_config: CtoConfig,
}

// =========================================================================
// CTO Configuration Extraction
// =========================================================================

/// Known CLI options from labels.
const KNOWN_CLIS: &[&str] = &["claude", "cursor", "codex", "dexter", "opencode"];

/// Known model shortcut labels mapped to full model names.
const KNOWN_MODEL_LABELS: &[(&str, &str)] = &[
    ("sonnet", "claude-sonnet-4-20250514"),
    ("opus", "claude-opus-4-20250514"),
    ("gpt-4.1", "gpt-4.1"),
    ("o3", "o3"),
];

/// Wrapper for frontmatter parsing that contains the cto config.
#[derive(Debug, Deserialize)]
struct FrontmatterWrapper {
    cto: Option<CtoConfig>,
}

/// Parse YAML frontmatter from issue description.
///
/// Supports format:
/// ```text
/// ---
/// cto:
///   cli: cursor
///   model: claude-opus-4-20250514
/// ---
/// Actual PRD content...
/// ```
///
/// Returns `None` if no valid frontmatter is found or parsing fails.
#[must_use]
pub fn parse_cto_frontmatter(description: &str) -> Option<CtoConfig> {
    // Check for frontmatter delimiter
    let trimmed = description.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing delimiter
    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;

    // Extract the YAML content between delimiters
    let yaml_content = &after_first[..end_pos].trim();

    // Parse the YAML
    match serde_yaml::from_str::<FrontmatterWrapper>(yaml_content) {
        Ok(wrapper) => {
            if let Some(config) = wrapper.cto {
                if !config.is_empty() {
                    info!(
                        cli = ?config.cli,
                        model = ?config.model,
                        "Extracted CTO config from frontmatter"
                    );
                    return Some(config);
                }
            }
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to parse frontmatter YAML");
            None
        }
    }
}

/// Extract CTO config from issue labels.
///
/// Looks for labels in the format:
/// - `CTO CLI/claude`, `CTO CLI/cursor`, etc. (grouped labels)
/// - `claude`, `cursor`, etc. (flat labels - for CLI)
/// - `CTO Model/sonnet`, `CTO Model/opus`, etc. (grouped labels)
/// - `sonnet`, `opus`, etc. (flat labels - for model shortcuts)
#[must_use]
pub fn extract_config_from_labels(labels: &[Label]) -> CtoConfig {
    let mut config = CtoConfig::default();

    for label in labels {
        let name = label.name.to_lowercase();

        // Check for grouped label format: "CTO CLI/xxx" or "CTO Model/xxx"
        if let Some(cli) = name.strip_prefix("cto cli/") {
            let cli = cli.trim();
            if KNOWN_CLIS.contains(&cli) {
                config.cli = Some(cli.to_string());
                continue;
            }
        }

        if let Some(model_label) = name.strip_prefix("cto model/") {
            let model_label = model_label.trim();
            // Check if it's a shortcut or a full model name
            if let Some((_, full_model)) = KNOWN_MODEL_LABELS
                .iter()
                .find(|(short, _)| *short == model_label)
            {
                config.model = Some((*full_model).to_string());
            } else {
                // Assume it's a full model name
                config.model = Some(model_label.to_string());
            }
            continue;
        }

        // Check for flat CLI labels
        if config.cli.is_none() && KNOWN_CLIS.contains(&name.as_str()) {
            config.cli = Some(name.clone());
            continue;
        }

        // Check for flat model shortcut labels
        if config.model.is_none() {
            if let Some((_, full_model)) = KNOWN_MODEL_LABELS
                .iter()
                .find(|(short, _)| *short == name.as_str())
            {
                config.model = Some((*full_model).to_string());
            }
        }
    }

    if !config.is_empty() {
        info!(
            cli = ?config.cli,
            model = ?config.model,
            "Extracted CTO config from labels"
        );
    }

    config
}

/// Extract CTO config from issue (labels + frontmatter).
///
/// Resolution order: Description frontmatter > Labels
/// Frontmatter values override label values.
#[must_use]
pub fn extract_cto_config(issue: &Issue) -> CtoConfig {
    // 1. Extract from labels
    let mut config = extract_config_from_labels(&issue.labels);

    // 2. Parse frontmatter (overrides labels)
    if let Some(desc) = &issue.description {
        if let Some(fm_config) = parse_cto_frontmatter(desc) {
            config.merge(&fm_config);
        }
    }

    config
}

/// Extract tech stack from issue labels.
fn extract_tech_stack(labels: &[Label]) -> TechStack {
    let label_names: Vec<String> = labels.iter().map(|l| l.name.to_lowercase()).collect();

    let mut tech_stack = TechStack::default();

    // Backend detection
    if label_names.iter().any(|l| l == "rust") {
        tech_stack.backend = Some("rust".to_string());
        tech_stack.languages.push("rust".to_string());
    }
    if label_names.iter().any(|l| l == "python") {
        tech_stack.backend = Some("python".to_string());
        tech_stack.languages.push("python".to_string());
    }
    if label_names.iter().any(|l| l == "go" || l == "golang") {
        tech_stack.backend = Some("go".to_string());
        tech_stack.languages.push("go".to_string());
    }
    if label_names.iter().any(|l| l == "node" || l == "nodejs") {
        tech_stack.backend = Some("node".to_string());
    }

    // Frontend detection
    if label_names.iter().any(|l| l == "react") {
        tech_stack.frontend = Some("react".to_string());
    }
    if label_names.iter().any(|l| l == "vue") {
        tech_stack.frontend = Some("vue".to_string());
    }
    if label_names.iter().any(|l| l == "svelte") {
        tech_stack.frontend = Some("svelte".to_string());
    }
    if label_names.iter().any(|l| l == "typescript" || l == "ts") {
        tech_stack.languages.push("typescript".to_string());
    }

    tech_stack
}

/// Strip frontmatter from description content.
///
/// Returns the description with the YAML frontmatter block removed.
#[must_use]
pub fn strip_frontmatter(description: &str) -> String {
    let trimmed = description.trim_start();
    if !trimmed.starts_with("---") {
        return description.to_string();
    }

    let after_first = &trimmed[3..];
    if let Some(end_pos) = after_first.find("\n---") {
        // Skip past the closing --- and any following newline
        let rest = &after_first[end_pos + 4..];
        rest.trim_start_matches('\n').to_string()
    } else {
        description.to_string()
    }
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
/// This function extracts the PRD content, CTO config from labels/frontmatter,
/// and any linked documents from the issue.
pub fn extract_intake_request(session_id: &str, issue: &Issue) -> Result<IntakeRequest> {
    let team_id = issue
        .team
        .as_ref()
        .ok_or_else(|| anyhow!("Issue has no team"))?
        .id
        .clone();

    let raw_description = issue
        .description
        .clone()
        .ok_or_else(|| anyhow!("PRD issue has no description"))?;

    // Extract CTO config from labels and frontmatter
    let cto_config = extract_cto_config(issue);

    // Extract tech stack from labels
    let tech_stack = extract_tech_stack(&issue.labels);

    // Strip frontmatter from PRD content (if present)
    let prd_content = strip_frontmatter(&raw_description);

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
        repository_url: None, // Not extracted from attachments in this version
        source_branch: None,  // Default to main
        tech_stack,
        cto_config,
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

    // Default repository URL if not provided in the request.
    let repository_url = request
        .repository_url
        .clone()
        .unwrap_or_else(|| "https://github.com/5dlabs/cto".to_string());
    let source_branch = request
        .source_branch
        .clone()
        .unwrap_or_else(|| "main".to_string());

    // Apply CTO config overrides from labels/frontmatter
    let cli = request
        .cto_config
        .cli
        .as_deref()
        .unwrap_or(&config.cli);
    let primary_model = request
        .cto_config
        .model
        .as_deref()
        .unwrap_or(&config.primary_model);

    if !request.cto_config.is_empty() {
        info!(
            cli = %cli,
            model = %primary_model,
            "Using CTO config overrides from issue"
        );
    }

    // Prepare config JSON for the workflow.
    let config_json = serde_json::json!({
        "project_name": project_name,
        "repository_url": repository_url,
        "github_app": config.github_app,
        "primary_model": primary_model,
        "research_model": config.research_model,
        "fallback_model": config.fallback_model,
        "primary_provider": config.primary_provider,
        "research_provider": config.research_provider,
        "fallback_provider": config.fallback_provider,
        "model": primary_model,
        "num_tasks": config.num_tasks,
        "expand_tasks": config.expand_tasks,
        "analyze_complexity": config.analyze_complexity,
        "docs_model": config.docs_model,
        "enrich_context": config.enrich_context,
        "include_codebase": config.include_codebase,
        "cli": cli,
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
                (
                    "app.kubernetes.io/name".to_string(),
                    "cto-intake".to_string(),
                ),
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
            &format!("primary-model={primary_model}"),
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
            &format!("cli={cli}"),
            // Linear callback parameters
            "-p",
            &format!("linear-session-id={}", request.session_id),
            "-p",
            &format!("linear-issue-id={}", request.prd_issue_id),
            "-p",
            &format!("linear-issue-identifier={}", request.prd_identifier),
            "-p",
            &format!("linear-team-id={}", request.team_id),
            // Labels for pod discovery (used for two-way communication)
            "-l",
            &format!("linear-session={}", request.session_id),
            "-l",
            &format!("cto.5dlabs.io/linear-issue={}", request.prd_identifier),
            "-l",
            "cto.5dlabs.io/agent-type=intake",
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
    create_task_issues_with_project(client, request, tasks, None).await
}

/// Create Linear issues for generated tasks, optionally linked to a project.
pub async fn create_task_issues_with_project(
    client: &LinearClient,
    request: &IntakeRequest,
    tasks: &[IntakeTask],
    project_id: Option<&str>,
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
    
    // Get or create agent:pending label for new tasks
    let agent_pending_label = client
        .get_or_create_label(&request.team_id, "agent:pending")
        .await?;

    info!(
        task_count = tasks.len(),
        parent_issue = %request.prd_identifier,
        project_id = ?project_id,
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

        let mut label_ids = vec![cto_task_label.id.clone(), agent_pending_label.id.clone()];
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
            project_id: project_id.map(String::from),
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

/// Create a Linear project for an intake request.
///
/// Creates a project linked to the PRD issue's team, with appropriate
/// description and metadata.
pub async fn create_intake_project(
    client: &LinearClient,
    request: &IntakeRequest,
    task_count: usize,
) -> Result<Project> {
    // Determine project name from PRD title
    let project_name = derive_project_name(&request.title);

    let description = format!(
        "## Project Overview\n\n\
         Generated from PRD: **{}** ({})\n\n\
         This project contains {} tasks for implementation.\n\n\
         ---\n\n\
         *Created by CTO Agent intake workflow*",
        request.title, request.prd_identifier, task_count
    );

    let input = ProjectCreateInput {
        name: project_name,
        description: Some(description),
        team_ids: Some(vec![request.team_id.clone()]),
        lead_id: None,
        target_date: None,
    };

    let project = client.create_project(input).await?;

    info!(
        project_id = %project.id,
        project_name = %project.name,
        prd = %request.prd_identifier,
        "Created Linear project for intake"
    );

    Ok(project)
}

/// Derive a clean project name from PRD title.
fn derive_project_name(title: &str) -> String {
    // Remove common prefixes
    let cleaned = title
        .trim()
        .strip_prefix("[PRD]")
        .or_else(|| title.strip_prefix("PRD:"))
        .or_else(|| title.strip_prefix("PRD -"))
        .unwrap_or(title)
        .trim();

    // Limit length
    if cleaned.len() > 100 {
        format!("{}...", &cleaned[..97])
    } else {
        cleaned.to_string()
    }
}

/// Format task description for Linear issue.
fn format_task_description(task: &IntakeTask) -> String {
    use std::fmt::Write;

    let mut description = format!("## Description\n\n{}\n\n", task.description);

    if !task.details.is_empty() {
        let _ = write!(description, "## Details\n\n{}\n\n", task.details);
    }

    if !task.test_strategy.is_empty() {
        let _ = write!(
            description,
            "## Test Strategy\n\n{}\n\n",
            task.test_strategy
        );
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

    // =========================================================================
    // CTO Config Extraction Tests
    // =========================================================================

    #[test]
    fn test_parse_cto_frontmatter_valid() {
        let description = r"---
cto:
  cli: cursor
  model: claude-opus-4-20250514
---
## PRD: My Feature

This is the actual content.";

        let config = parse_cto_frontmatter(description).unwrap();
        assert_eq!(config.cli, Some("cursor".to_string()));
        assert_eq!(config.model, Some("claude-opus-4-20250514".to_string()));
    }

    #[test]
    fn test_parse_cto_frontmatter_cli_only() {
        let description = r"---
cto:
  cli: codex
---
Content here";

        let config = parse_cto_frontmatter(description).unwrap();
        assert_eq!(config.cli, Some("codex".to_string()));
        assert_eq!(config.model, None);
    }

    #[test]
    fn test_parse_cto_frontmatter_model_only() {
        let description = r"---
cto:
  model: gpt-4.1
---
Content here";

        let config = parse_cto_frontmatter(description).unwrap();
        assert_eq!(config.cli, None);
        assert_eq!(config.model, Some("gpt-4.1".to_string()));
    }

    #[test]
    fn test_parse_cto_frontmatter_no_frontmatter() {
        let description = "Just regular content without frontmatter";
        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_parse_cto_frontmatter_no_cto_section() {
        let description = r"---
title: My PRD
author: Someone
---
Content here";

        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_parse_cto_frontmatter_empty_cto() {
        let description = r"---
cto:
---
Content here";

        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_parse_cto_frontmatter_invalid_yaml() {
        let description = r"---
cto: [invalid yaml here
---
Content";

        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_strip_frontmatter() {
        let description = r"---
cto:
  cli: cursor
---
## My PRD

Content here";

        let stripped = strip_frontmatter(description);
        assert_eq!(stripped, "## My PRD\n\nContent here");
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let description = "Just regular content";
        let stripped = strip_frontmatter(description);
        assert_eq!(stripped, "Just regular content");
    }

    #[test]
    fn test_extract_config_from_labels_grouped() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "CTO CLI/cursor".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "CTO Model/opus".to_string(),
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.cli, Some("cursor".to_string()));
        assert_eq!(config.model, Some("claude-opus-4-20250514".to_string()));
    }

    #[test]
    fn test_extract_config_from_labels_flat() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "claude".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "sonnet".to_string(),
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.cli, Some("claude".to_string()));
        assert_eq!(config.model, Some("claude-sonnet-4-20250514".to_string()));
    }

    #[test]
    fn test_extract_config_from_labels_mixed() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "CTO CLI/codex".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "gpt-4.1".to_string(), // flat model label
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.cli, Some("codex".to_string()));
        assert_eq!(config.model, Some("gpt-4.1".to_string()));
    }

    #[test]
    fn test_extract_config_from_labels_empty() {
        let labels: Vec<Label> = vec![];
        let config = extract_config_from_labels(&labels);
        assert!(config.is_empty());
    }

    #[test]
    fn test_extract_config_from_labels_unrelated() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "bug".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "high-priority".to_string(),
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert!(config.is_empty());
    }

    #[test]
    fn test_extract_config_from_labels_full_model_in_group() {
        // If someone uses a full model name in the group label
        let labels = vec![Label {
            id: "1".to_string(),
            name: "CTO Model/claude-sonnet-4-20250514".to_string(),
            color: None,
        }];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.model, Some("claude-sonnet-4-20250514".to_string()));
    }
}
