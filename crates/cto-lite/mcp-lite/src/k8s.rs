//! Kubernetes client for workflow management
//!
//! Communicates with the Kind cluster to create and query Argo Workflows.

use anyhow::{anyhow, Result};
use tracing::{debug, info};

/// Kubernetes client for CTO Lite
pub struct K8sClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
    namespace: String,
}

/// Workflow status
#[derive(Debug, Clone)]
pub struct WorkflowStatus {
    pub phase: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub message: Option<String>,
    pub nodes: Vec<NodeStatus>,
}

/// Node (step) status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NodeStatus {
    pub name: String,
    pub display_name: String,
    pub phase: String,
}

/// Job summary
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JobSummary {
    pub name: String,
    pub phase: String,
    pub created_at: String,
    pub repo: Option<String>,
}

impl K8sClient {
    /// Create a new client, auto-detecting cluster configuration
    pub async fn new() -> Result<Self> {
        // Try in-cluster config first
        if let Ok(client) = Self::in_cluster().await {
            return Ok(client);
        }

        // Fall back to kubeconfig
        Self::from_kubeconfig().await
    }

    async fn in_cluster() -> Result<Self> {
        let token = std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")?;
        let namespace =
            std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace")
                .unwrap_or_else(|_| "cto-lite".to_string());

        Ok(Self {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // In-cluster cert
                .build()?,
            base_url: "https://kubernetes.default.svc".to_string(),
            token: Some(token),
            namespace,
        })
    }

    async fn from_kubeconfig() -> Result<Self> {
        // For local development, use kubectl proxy or direct API
        let namespace = std::env::var("CTO_NAMESPACE").unwrap_or_else(|_| "cto-lite".to_string());

        // Try kubectl proxy first (localhost:8001)
        let proxy_url = std::env::var("KUBERNETES_PROXY_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            base_url: proxy_url,
            token: None,
            namespace,
        })
    }

    fn workflow_url(&self, name: &str) -> String {
        format!(
            "{}/apis/argoproj.io/v1alpha1/namespaces/{}/workflows/{}",
            self.base_url, self.namespace, name
        )
    }

    fn workflows_url(&self) -> String {
        format!(
            "{}/apis/argoproj.io/v1alpha1/namespaces/{}/workflows",
            self.base_url, self.namespace
        )
    }

    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.token {
            builder.bearer_auth(token)
        } else {
            builder
        }
    }

    /// Create a new workflow
    pub async fn create_workflow(
        &self,
        repo: &str,
        prompt: &str,
        issue_number: Option<i64>,
        stack: &str,
    ) -> Result<String> {
        let workflow_name = format!(
            "cto-{}-{}",
            repo.replace('/', "-").chars().take(30).collect::<String>(),
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );

        let workflow = serde_json::json!({
            "apiVersion": "argoproj.io/v1alpha1",
            "kind": "Workflow",
            "metadata": {
                "name": workflow_name,
                "namespace": self.namespace,
                "labels": {
                    "app.kubernetes.io/part-of": "cto-lite",
                    "cto.dev/repo": repo.replace('/', "-"),
                }
            },
            "spec": {
                "workflowTemplateRef": {
                    "name": "play-workflow-lite"
                },
                "arguments": {
                    "parameters": [
                        {"name": "repo", "value": repo},
                        {"name": "branch", "value": "main"},
                        {"name": "issue-number", "value": issue_number.map(|n| n.to_string()).unwrap_or_default()},
                        {"name": "pr-number", "value": ""},
                        {"name": "prompt", "value": prompt},
                        {"name": "stack", "value": stack},
                    ]
                }
            }
        });

        info!("Creating workflow: {}", workflow_name);
        debug!("Workflow spec: {:?}", workflow);

        let response = self
            .add_auth(self.client.post(self.workflows_url()).json(&workflow))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create workflow: {} - {}", status, body));
        }

        Ok(workflow_name)
    }

    /// Get workflow status
    pub async fn get_workflow_status(&self, name: &str) -> Result<WorkflowStatus> {
        let response = self
            .add_auth(self.client.get(self.workflow_url(name)))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(anyhow!("Workflow not found: {}", name));
            }
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get workflow: {} - {}", status, body));
        }

        let workflow: serde_json::Value = response.json().await?;
        let status = workflow.get("status").cloned().unwrap_or_default();

        let mut nodes = Vec::new();
        if let Some(nodes_map) = status.get("nodes").and_then(|n| n.as_object()) {
            for (_, node) in nodes_map {
                let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if node_type == "Pod" || node_type == "Steps" || node_type == "DAG" {
                    nodes.push(NodeStatus {
                        name: node
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        display_name: node
                            .get("displayName")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        phase: node
                            .get("phase")
                            .and_then(|p| p.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                    });
                }
            }
        }

        Ok(WorkflowStatus {
            phase: status
                .get("phase")
                .and_then(|p| p.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            started_at: status
                .get("startedAt")
                .and_then(|s| s.as_str())
                .map(String::from),
            finished_at: status
                .get("finishedAt")
                .and_then(|s| s.as_str())
                .map(String::from),
            message: status
                .get("message")
                .and_then(|m| m.as_str())
                .map(String::from),
            nodes,
        })
    }

    /// Get workflow logs
    pub async fn get_workflow_logs(&self, name: &str, tail: i64) -> Result<String> {
        // Get pods for this workflow
        let pods_url = format!(
            "{}/api/v1/namespaces/{}/pods?labelSelector=workflows.argoproj.io/workflow={}",
            self.base_url, self.namespace, name
        );

        let response = self.add_auth(self.client.get(&pods_url)).send().await?;

        if !response.status().is_success() {
            return Ok("No logs available yet.".to_string());
        }

        let pods: serde_json::Value = response.json().await?;
        let items = pods.get("items").and_then(|i| i.as_array());

        let mut all_logs = String::new();

        if let Some(items) = items {
            for pod in items.iter().take(5) {
                // Limit to 5 pods
                let pod_name = pod
                    .get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                let logs_url = format!(
                    "{}/api/v1/namespaces/{}/pods/{}/log?tailLines={}&container=main",
                    self.base_url, self.namespace, pod_name, tail
                );

                if let Ok(response) = self.add_auth(self.client.get(&logs_url)).send().await {
                    if response.status().is_success() {
                        if let Ok(log_text) = response.text().await {
                            if !log_text.is_empty() {
                                all_logs
                                    .push_str(&format!("=== {} ===\n{}\n\n", pod_name, log_text));
                            }
                        }
                    }
                }
            }
        }

        if all_logs.is_empty() {
            Ok("No logs available yet.".to_string())
        } else {
            Ok(all_logs)
        }
    }

    /// List workflows
    pub async fn list_workflows(
        &self,
        limit: i64,
        repo_filter: Option<&str>,
    ) -> Result<Vec<JobSummary>> {
        let mut url = format!("{}?limit={}", self.workflows_url(), limit);

        if let Some(repo) = repo_filter {
            url.push_str(&format!(
                "&labelSelector=cto.dev/repo={}",
                repo.replace('/', "-")
            ));
        }

        let response = self.add_auth(self.client.get(&url)).send().await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let list: serde_json::Value = response.json().await?;
        let items = list.get("items").and_then(|i| i.as_array());

        let mut jobs = Vec::new();
        if let Some(items) = items {
            for item in items {
                let name = item
                    .get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let phase = item
                    .get("status")
                    .and_then(|s| s.get("phase"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let created_at = item
                    .get("metadata")
                    .and_then(|m| m.get("creationTimestamp"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let repo = item
                    .get("metadata")
                    .and_then(|m| m.get("labels"))
                    .and_then(|l| l.get("cto.dev/repo"))
                    .and_then(|r| r.as_str())
                    .map(|r| r.replace('-', "/"));

                jobs.push(JobSummary {
                    name,
                    phase,
                    created_at,
                    repo,
                });
            }
        }

        Ok(jobs)
    }
}
