//! Argo Workflows integration for resuming suspended workflows

use anyhow::{Context, Result};
use kube::Client;
use serde_json::json;
use tracing::{info, warn};

use crate::crds::coderun::CodeRun;

/// Extract workflow name from `CodeRun` labels
pub fn extract_workflow_name(code_run: &CodeRun) -> Result<String> {
    // Try to get workflow name from label if labels exist
    if let Some(labels) = code_run.metadata.labels.as_ref() {
        if let Some(workflow_name) = labels.get("workflow-name") {
            return Ok(workflow_name.clone());
        }
    }

    // Fallback: construct from task ID (use 0 for docs/intake runs without task ID)
    let task_id = code_run.spec.task_id.unwrap_or(0);
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
        Err(anyhow::anyhow!("Invalid PR URL format: {pr_url}"))
    }
}

/// Resume an Argo Workflow with PR details
pub async fn resume_workflow_for_pr(
    client: &Client,
    namespace: &str,
    workflow_name: &str,
    pr_url: &str,
    pr_number: u32,
    remediation_status: Option<&str>,
    qa_status: Option<&str>,
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
        remediation_status,
        qa_status,
    )
    .await
}

/// Resume workflow when `CodeRun` failed
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
        None,
        None,
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
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
async fn resume_workflow_via_http(
    _client: &Client,
    namespace: &str,
    workflow_name: &str,
    pr_url: Option<&str>,
    pr_number: Option<u32>,
    error_message: Option<&str>,
    remediation_status: Option<&str>,
    qa_status: Option<&str>,
) -> Result<()> {
    info!(
        "🚀 Attempting to force workflow {} to re-evaluate stuck nodes in namespace {}",
        workflow_name, namespace
    );

    // Use raw HTTP calls since we need to work with Argo Workflows CRDs
    // and the kube dynamic API is complex for this use case
    let token = std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
        .context("Failed to read service account token")?;

    let ca_cert = std::fs::read("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")
        .context("Failed to read CA certificate")?;

    let cert =
        reqwest::Certificate::from_pem(&ca_cert).context("Failed to parse CA certificate")?;

    let http_client = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()
        .context("Failed to create HTTP client")?;

    // Get the current workflow via HTTP API
    let api_server = "https://kubernetes.default.svc";
    let get_url = format!(
        "{api_server}/apis/argoproj.io/v1alpha1/namespaces/{namespace}/workflows/{workflow_name}"
    );

    let workflow_response = http_client
        .get(&get_url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .context("Failed to get workflow")?;

    if !workflow_response.status().is_success() {
        let status = workflow_response.status();
        let error_text = workflow_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!(
            "Failed to get workflow {workflow_name}: HTTP {status} - {error_text}"
        ));
    }

    let workflow: serde_json::Value = workflow_response
        .json()
        .await
        .context("Failed to parse workflow JSON")?;

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
                node_data.get("phase").and_then(|p| p.as_str()),
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
        info!(
            "ℹ️ No stuck wait-coderun-completion nodes found in workflow {}",
            workflow_name
        );
        return Ok(());
    }

    // Force workflow controller to re-evaluate by adding a retry annotation
    // This is similar to what `argo retry` does, but we also propagate context annotations
    let mut annotations = serde_json::Map::new();
    annotations.insert(
        "workflows.argoproj.io/force-retry".to_string(),
        json!(chrono::Utc::now().to_rfc3339()),
    );

    if let Some(pr_url) = pr_url {
        annotations.insert("agents.platform/pr-url".to_string(), json!(pr_url));
    }

    if let Some(pr_number) = pr_number {
        annotations.insert("agents.platform/pr-number".to_string(), json!(pr_number));
    }

    if let Some(remediation_status) = remediation_status {
        annotations.insert(
            "agents.platform/remediation-status".to_string(),
            json!(remediation_status),
        );
    }

    if let Some(qa_status) = qa_status {
        annotations.insert("agents.platform/qa-status".to_string(), json!(qa_status));
    }

    if let Some(error_message) = error_message {
        annotations.insert(
            "agents.platform/error-message".to_string(),
            json!(error_message),
        );
    }

    let retry_patch = json!({
        "metadata": {
            "annotations": serde_json::Value::Object(annotations)
        }
    });

    info!(
        "🔄 Forcing workflow controller to re-evaluate workflow: {}",
        workflow_name
    );

    // Patch the workflow via HTTP API
    let patch_url = format!(
        "{api_server}/apis/argoproj.io/v1alpha1/namespaces/{namespace}/workflows/{workflow_name}"
    );

    let patch_response = http_client
        .patch(&patch_url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/merge-patch+json")
        .json(&retry_patch)
        .send()
        .await
        .context("Failed to patch workflow with retry annotation")?;

    if !patch_response.status().is_success() {
        let status = patch_response.status();
        let error_text = patch_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!(
            "Failed to patch workflow {workflow_name}: HTTP {status} - {error_text}"
        ));
    }

    info!(
        "✅ Successfully triggered workflow re-evaluation: {}",
        workflow_name
    );

    // Log the context for debugging
    if let Some(pr_url) = pr_url {
        info!(
            "📝 Workflow triggered with PR context: {} (#{:?})",
            pr_url, pr_number
        );
    } else if let Some(error) = error_message {
        info!("❌ Workflow triggered with error context: {}", error);
    } else {
        info!("⚠️ Workflow triggered with no-PR context");
    }

    if let Some(remediation_status) = remediation_status {
        info!(
            "🔄 Forwarded remediation status to workflow annotations: {}",
            remediation_status
        );
    }

    if let Some(qa_status) = qa_status {
        info!(
            "🧪 Forwarded QA status to workflow annotations: {}",
            qa_status
        );
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
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(5),
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
                enable_docker: true,
                task_requirements: None,
                service_account_name: None,
                docs_project_directory: None,
                working_directory: None,
                github_user: None,
                github_app: None,
                linear_integration: None,
                prompt_modification: None,
                acceptance_criteria: None,
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
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(5),
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
                enable_docker: true,
                task_requirements: None,
                service_account_name: None,
                docs_project_directory: None,
                working_directory: None,
                github_user: None,
                github_app: None,
                linear_integration: None,
                prompt_modification: None,
                acceptance_criteria: None,
            },
            status: None,
        };

        assert_eq!(
            extract_workflow_name(&code_run).unwrap(),
            "play-task-5-workflow"
        );
    }
}
