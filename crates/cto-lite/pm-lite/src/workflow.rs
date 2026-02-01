//! Argo Workflow triggering

use anyhow::{anyhow, Result};
use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, PostParams},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

/// Parameters for triggering a workflow
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowParams {
    /// GitHub repository (owner/repo)
    pub repo: String,
    /// Target branch for PR
    pub branch: String,
    /// Issue number (optional)
    pub issue_number: Option<i64>,
    /// PR number (optional)
    pub pr_number: Option<i64>,
    /// User prompt
    pub prompt: String,
    /// Backend stack (grizz or nova)
    pub stack: String,
}

/// Workflow status
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowStatus {
    pub name: String,
    pub phase: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

/// Sanitize a string for use in Kubernetes resource names (DNS-1123 compliant)
fn sanitize_k8s_name(s: &str, max_len: usize) -> String {
    s.to_lowercase()
        .replace(['/', '_'], "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(max_len)
        .collect::<String>()
        .trim_end_matches('-')
        .to_string()
}

/// Validate repository format (owner/repo)
fn validate_repo(repo: &str) -> Result<()> {
    if repo.is_empty() {
        return Err(anyhow!("Repository name cannot be empty"));
    }
    if !repo.contains('/') || repo.split('/').count() != 2 {
        return Err(anyhow!(
            "Invalid repository format. Expected 'owner/repo', got '{repo}'"
        ));
    }
    if repo.len() > 200 {
        return Err(anyhow!("Repository name too long (max 200 chars)"));
    }
    Ok(())
}

/// Validate stack parameter
fn validate_stack(stack: &str) -> Result<()> {
    match stack {
        "nova" | "grizz" => Ok(()),
        _ => Err(anyhow!(
            "Invalid stack '{stack}'. Must be 'nova' or 'grizz'"
        )),
    }
}

/// Validate prompt length
fn validate_prompt(prompt: &str) -> Result<()> {
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.is_empty() {
        return Err(anyhow!("Prompt cannot be empty"));
    }
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(anyhow!(
            "Prompt too long ({} chars, max {} chars)",
            prompt.len(),
            MAX_PROMPT_LEN
        ));
    }
    Ok(())
}

/// Trigger an Argo Workflow using the play-workflow-lite template
///
/// # Errors
/// Returns error if workflow submission fails or validation fails
pub async fn trigger_workflow(namespace: &str, params: WorkflowParams) -> Result<String> {
    // Validate inputs
    validate_repo(&params.repo)?;
    validate_prompt(&params.prompt)?;
    validate_stack(&params.stack)?;

    let client = Client::try_default().await?;

    // Generate DNS-1123 compliant workflow name with longer UUID to reduce collision risk
    let workflow_name = format!(
        "cto-{}-{}",
        sanitize_k8s_name(&params.repo, 30),
        uuid::Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(12)
            .collect::<String>()
    );

    let workflow = json!({
        "apiVersion": "argoproj.io/v1alpha1",
        "kind": "Workflow",
        "metadata": {
            "name": workflow_name,
            "namespace": namespace,
            "labels": {
                "app.kubernetes.io/part-of": "cto-lite",
                "cto.dev/repo": sanitize_k8s_name(&params.repo, 63),
            }
        },
        "spec": {
            "workflowTemplateRef": {
                "name": "play-workflow-lite"
            },
            "arguments": {
                "parameters": [
                    {"name": "repo", "value": params.repo},
                    {"name": "branch", "value": params.branch},
                    {"name": "issue-number", "value": params.issue_number.map(|n| n.to_string()).unwrap_or_default()},
                    {"name": "pr-number", "value": params.pr_number.map(|n| n.to_string()).unwrap_or_default()},
                    {"name": "prompt", "value": params.prompt},
                    {"name": "stack", "value": params.stack},
                ]
            }
        }
    });

    // Use dynamic API since Workflow is a CRD
    let gvk = GroupVersionKind::gvk("argoproj.io", "v1alpha1", "Workflow");
    let api_resource = ApiResource::from_gvk(&gvk);
    let workflows: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let workflow_obj: DynamicObject = serde_json::from_value(workflow)?;
    workflows
        .create(&PostParams::default(), &workflow_obj)
        .await?;

    info!("Created workflow: {workflow_name}");
    Ok(workflow_name)
}

/// Get the default backend stack from user preferences
/// For now, defaults to "nova" (TypeScript)
#[must_use]
pub fn get_default_stack() -> String {
    std::env::var("CTO_DEFAULT_STACK").unwrap_or_else(|_| "nova".to_string())
}
