//! Argo Workflow triggering

use anyhow::Result;
use cto_lite_common::{sanitize_k8s_name, validate_prompt, validate_repo, validate_stack};
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
