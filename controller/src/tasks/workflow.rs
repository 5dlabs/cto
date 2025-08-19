//! Argo Workflows integration for resuming suspended workflows

use anyhow::{Context, Result};
use kube::Client;
use serde_json::json;
use tracing::{info, warn, error};

use crate::crds::coderun::CodeRun;

/// Extract workflow name from CodeRun labels
pub fn extract_workflow_name(code_run: &CodeRun) -> Result<String> {
    let labels = code_run
        .metadata
        .labels
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("CodeRun has no labels"))?;

    // Try to get workflow name from label
    if let Some(workflow_name) = labels.get("workflow-name") {
        return Ok(workflow_name.clone());
    }

    // Fallback: construct from task ID
    let task_id = code_run.spec.task_id;
    Ok(format!("play-task-{task_id}-workflow"))
}

/// Extract PR number from GitHub PR URL
pub fn extract_pr_number(pr_url: &str) -> Result<u32> {
    // Parse URLs like: https://github.com/owner/repo/pull/123
    let parts: Vec<&str> = pr_url.split('/').collect();
    if let Some(number_str) = parts.last() {
        number_str
            .parse::<u32>()
            .with_context(|| format!("Failed to parse PR number from URL: {pr_url}"))
    } else {
        Err(anyhow::anyhow!("Invalid PR URL format: {}", pr_url))
    }
}

/// Resume an Argo Workflow with PR details
pub async fn resume_workflow_for_pr(
    client: &Client,
    namespace: &str,
    workflow_name: &str,
    pr_url: &str,
    pr_number: u32,
) -> Result<()> {
    info!(
        "Resuming workflow {} with PR: {} (#{}) in namespace: {}",
        workflow_name, pr_url, pr_number, namespace
    );

    // Use direct HTTP calls to Argo Workflows API
    resume_workflow_via_http(
        client,
        namespace,
        workflow_name,
        Some(pr_url),
        Some(pr_number),
        None,
    )
    .await
}

/// Resume workflow when CodeRun failed
pub async fn resume_workflow_for_failure(
    client: &Client,
    namespace: &str,
    workflow_name: &str,
    error_message: &str,
) -> Result<()> {
    info!(
        "Resuming workflow {} with failure status in namespace: {}",
        workflow_name, namespace
    );

    // Use direct HTTP calls to Argo Workflows API
    resume_workflow_via_http(
        client,
        namespace,
        workflow_name,
        None,
        None,
        Some(error_message),
    )
    .await
}

/// Resume workflow when no PR was created
pub async fn resume_workflow_for_no_pr(
    _client: &Client,
    namespace: &str,
    workflow_name: &str,
    coderun_status: &str,
) -> Result<()> {
    info!(
        "Resuming workflow {} with no-PR status in namespace: {}",
        workflow_name, namespace
    );

    // For now, log the action but don't actually resume
    // TODO: Implement proper workflow resumption
    warn!(
        "Would resume workflow {} with no-PR status: {} (namespace: {})",
        workflow_name, coderun_status, namespace
    );
    Ok(())
}

/// Resume workflow by forcing re-evaluation of stuck resource nodes
async fn resume_workflow_via_http(
    client: &Client,
    namespace: &str,
    workflow_name: &str,
    pr_url: Option<&str>,
    pr_number: Option<u32>,
    error_message: Option<&str>,
) -> Result<()> {
    info!(
        "🚀 Attempting to force workflow {} to re-evaluate stuck nodes in namespace {}",
        workflow_name, namespace
    );

    // Use the Kubernetes client to get the workflow
    use kube::Api;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    
    // Create a dynamic API for workflows
    let workflows: Api<serde_json::Value> = Api::namespaced_with(
        client.clone(),
        namespace,
        &kube::api::ApiResource {
            group: "argoproj.io".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "argoproj.io/v1alpha1".to_string(),
            kind: "Workflow".to_string(),
            plural: "workflows".to_string(),
        },
    );

    // Get the current workflow
    let workflow = workflows
        .get(workflow_name)
        .await
        .context("Failed to get workflow")?;

    // Find nodes that are waiting for CodeRun completion
    let nodes = workflow
        .get("status")
        .and_then(|s| s.get("nodes"))
        .ok_or_else(|| anyhow::anyhow!("No nodes found in workflow"))?;

    let mut stuck_nodes = Vec::new();
    
    if let Some(nodes_obj) = nodes.as_object() {
        for (node_id, node_data) in nodes_obj {
            if let (Some(template_name), Some(phase)) = (
                node_data.get("templateName").and_then(|t| t.as_str()),
                node_data.get("phase").and_then(|p| p.as_str())
            ) {
                // Look for wait-coderun-completion nodes that are running
                if template_name == "wait-coderun-completion" && phase == "Running" {
                    info!("🔍 Found stuck wait-coderun-completion node: {}", node_id);
                    stuck_nodes.push(node_id.clone());
                }
            }
        }
    }

    if stuck_nodes.is_empty() {
        info!("ℹ️ No stuck wait-coderun-completion nodes found in workflow {}", workflow_name);
        return Ok(());
    }

    // Force workflow controller to re-evaluate by adding a retry annotation
    // This is similar to what `argo retry` does
    let retry_patch = json!({
        "metadata": {
            "annotations": {
                "workflows.argoproj.io/force-retry": chrono::Utc::now().to_rfc3339()
            }
        }
    });

    info!("🔄 Forcing workflow controller to re-evaluate workflow: {}", workflow_name);
    
    let patch_params = kube::api::PatchParams::default();
    workflows
        .patch(workflow_name, &patch_params, &kube::api::Patch::Merge(retry_patch))
        .await
        .context("Failed to patch workflow with retry annotation")?;

    info!("✅ Successfully triggered workflow re-evaluation: {}", workflow_name);

    // Log the context for debugging
    if let Some(pr_url) = pr_url {
        info!("📝 Workflow triggered with PR context: {} (#{:?})", pr_url, pr_number);
    } else if let Some(error) = error_message {
        info!("❌ Workflow triggered with error context: {}", error);
    } else {
        info!("⚠️ Workflow triggered with no-PR context");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pr_number() {
        assert_eq!(
            extract_pr_number("https://github.com/owner/repo/pull/123").unwrap(),
            123
        );
        assert_eq!(
            extract_pr_number("https://github.com/5dlabs/cto/pull/42").unwrap(),
            42
        );
        assert!(extract_pr_number("invalid-url").is_err());
        assert!(extract_pr_number("https://github.com/owner/repo/pull/abc").is_err());
    }

    #[test]
    fn test_extract_workflow_name_from_labels() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        use std::collections::BTreeMap;

        let mut labels = BTreeMap::new();
        labels.insert("workflow-name".to_string(), "test-workflow".to_string());

        let code_run = CodeRun {
            metadata: ObjectMeta {
                labels: Some(labels),
                ..Default::default()
            },
            spec: crate::crds::coderun::CodeRunSpec {
                task_id: 5,
                service: "test".to_string(),
                repository_url: "test".to_string(),
                docs_repository_url: "test".to_string(),
                model: "test".to_string(),
                context_version: 1,
                docs_branch: "main".to_string(),
                continue_session: false,
                overwrite_memory: false,
                env: std::collections::HashMap::new(),
                env_from_secrets: vec![],
                enable_docker: None,
                task_requirements: None,
                service_account_name: None,
                docs_project_directory: None,
                working_directory: None,
                github_user: None,
                github_app: None,
            },
            status: None,
        };

        assert_eq!(extract_workflow_name(&code_run).unwrap(), "test-workflow");
    }

    #[test]
    fn test_extract_workflow_name_fallback() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

        let code_run = CodeRun {
            metadata: ObjectMeta {
                labels: None,
                ..Default::default()
            },
            spec: crate::crds::coderun::CodeRunSpec {
                task_id: 5,
                service: "test".to_string(),
                repository_url: "test".to_string(),
                docs_repository_url: "test".to_string(),
                model: "test".to_string(),
                context_version: 1,
                docs_branch: "main".to_string(),
                continue_session: false,
                overwrite_memory: false,
                env: std::collections::HashMap::new(),
                env_from_secrets: vec![],
                enable_docker: None,
                task_requirements: None,
                service_account_name: None,
                docs_project_directory: None,
                working_directory: None,
                github_user: None,
                github_app: None,
            },
            status: None,
        };

        assert_eq!(
            extract_workflow_name(&code_run).unwrap(),
            "play-task-5-workflow"
        );
    }
}
