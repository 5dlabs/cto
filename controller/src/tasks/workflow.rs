//! Argo Workflows integration for resuming suspended workflows

use anyhow::{Context, Result};
use kube::Client;
use tracing::{info, warn};

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
    Ok(format!("play-task-{}-workflow", task_id))
}

/// Extract PR number from GitHub PR URL
pub fn extract_pr_number(pr_url: &str) -> Result<u32> {
    // Parse URLs like: https://github.com/owner/repo/pull/123
    let parts: Vec<&str> = pr_url.split('/').collect();
    if let Some(number_str) = parts.last() {
        number_str
            .parse::<u32>()
            .with_context(|| format!("Failed to parse PR number from URL: {}", pr_url))
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
    resume_workflow_via_http(client, namespace, workflow_name, Some(pr_url), Some(pr_number), None).await
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
    resume_workflow_via_http(client, namespace, workflow_name, None, None, Some(error_message)).await
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
    warn!("Would resume workflow {} with no-PR status: {} (namespace: {})", workflow_name, coderun_status, namespace);
    Ok(())
}

/// Resume workflow via direct API call
async fn resume_workflow_via_http(
    _client: &Client,
    _namespace: &str,
    workflow_name: &str,
    pr_url: Option<&str>,
    pr_number: Option<u32>,
    error_message: Option<&str>,
) -> Result<()> {
    // For now, just log the action
    // TODO: Implement actual Argo Workflows API calls
    if let Some(pr_url) = pr_url {
        info!("Would resume workflow {} with PR: {} (#{:?})", 
              workflow_name, pr_url, pr_number);
    } else if let Some(error) = error_message {
        info!("Would resume workflow {} with error: {}", 
              workflow_name, error);
    } else {
        info!("Would resume workflow {} with no-PR status", 
              workflow_name);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pr_number() {
        assert_eq!(extract_pr_number("https://github.com/owner/repo/pull/123").unwrap(), 123);
        assert_eq!(extract_pr_number("https://github.com/5dlabs/cto/pull/42").unwrap(), 42);
        assert!(extract_pr_number("invalid-url").is_err());
        assert!(extract_pr_number("https://github.com/owner/repo/pull/abc").is_err());
    }

    #[test]
    fn test_extract_workflow_name_from_labels() {
        use std::collections::BTreeMap;
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

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

        assert_eq!(extract_workflow_name(&code_run).unwrap(), "play-task-5-workflow");
    }
}
