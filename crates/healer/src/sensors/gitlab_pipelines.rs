//! GitLab pipeline failure sensor.
//!
//! Polls GitLab CI pipelines for failures and triggers remediation.
//! Parallel to github_actions.rs for dual SCM support.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Configuration for the GitLab pipeline sensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabSensorConfig {
    /// GitLab host (e.g., "gitlab.5dlabs.ai")
    #[serde(default = "default_host")]
    pub host: String,
    /// GitLab API base URL
    #[serde(default = "default_api_base")]
    pub api_base: String,
    /// Projects to monitor (e.g., "5dlabs/cto")
    pub projects: Vec<String>,
    /// Poll interval in seconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    /// Kubernetes namespace for CodeRuns
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

fn default_host() -> String {
    "gitlab.5dlabs.ai".to_string()
}
fn default_api_base() -> String {
    "https://gitlab.5dlabs.ai/api/v4".to_string()
}
fn default_poll_interval() -> u64 {
    300
}
fn default_namespace() -> String {
    "cto".to_string()
}

/// GitLab pipeline failure sensor.
pub struct GitLabPipelineSensor {
    config: GitLabSensorConfig,
    token: Option<String>,
}

impl GitLabPipelineSensor {
    pub fn new(config: GitLabSensorConfig) -> Self {
        let token = std::env::var("GITLAB_TOKEN").ok();
        Self { config, token }
    }

    /// Poll GitLab pipelines for failures.
    pub async fn poll(&self) -> Result<Vec<PipelineFailure>> {
        let Some(ref token) = self.token else {
            warn!("No GITLAB_TOKEN set — skipping GitLab pipeline polling");
            return Ok(vec![]);
        };

        let client = reqwest::Client::new();
        let mut failures = Vec::new();

        for project in &self.config.projects {
            let pid = urlencoding::encode(project);
            let url = format!(
                "{}/projects/{pid}/pipelines?status=failed&per_page=10",
                self.config.api_base
            );

            let resp = client
                .get(&url)
                .header("PRIVATE-TOKEN", token)
                .send()
                .await?;

            if !resp.status().is_success() {
                warn!(project = %project, "Failed to fetch pipelines");
                continue;
            }

            let pipelines: Vec<GitLabPipeline> = resp.json().await?;
            for pipeline in pipelines {
                info!(
                    project = %project,
                    pipeline_id = pipeline.id,
                    ref_name = %pipeline.ref_name,
                    "Found failed pipeline"
                );
                failures.push(PipelineFailure {
                    project: project.clone(),
                    pipeline_id: pipeline.id,
                    ref_name: pipeline.ref_name,
                    web_url: pipeline.web_url,
                    status: pipeline.status,
                });
            }
        }

        Ok(failures)
    }
}

#[derive(Debug, Deserialize)]
struct GitLabPipeline {
    id: u64,
    #[serde(rename = "ref")]
    ref_name: String,
    status: String,
    web_url: String,
}

/// A failed GitLab pipeline.
#[derive(Debug, Clone)]
pub struct PipelineFailure {
    pub project: String,
    pub pipeline_id: u64,
    pub ref_name: String,
    pub web_url: String,
    pub status: String,
}
