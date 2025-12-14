use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{Api, DeleteParams, Patch, PatchParams, PostParams},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::info;

const NAMESPACE: &str = "cto";

/// Status of a play workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PlayStatus {
    /// Workflow is currently running
    InProgress,
    /// Workflow is suspended waiting for external event (PR merge, approval, etc.)
    Suspended,
    /// Workflow failed (PR validation failed, agent error, etc.)
    Failed,
    /// Workflow completed successfully
    Completed,
}

impl std::fmt::Display for PlayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InProgress => write!(f, "in-progress"),
            Self::Suspended => write!(f, "suspended"),
            Self::Failed => write!(f, "failed"),
            Self::Completed => write!(f, "completed"),
        }
    }
}

/// Progress tracking for a play workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayProgress {
    /// GitHub repository (e.g., "5dlabs/cto")
    pub repository: String,
    /// Branch in the repository
    pub branch: String,
    /// Current task ID being worked on
    pub current_task_id: Option<u32>,
    /// Name of the Argo workflow
    pub workflow_name: Option<String>,
    /// Current status
    pub status: PlayStatus,
    /// Current stage (implementation, code-quality, qa, etc.)
    pub stage: Option<String>,
    /// When the workflow started
    pub started_at: DateTime<Utc>,
    /// Last update time
    pub last_updated: DateTime<Utc>,
}

impl PlayProgress {
    /// Create a new progress entry
    #[must_use]
    pub fn new(repository: String, branch: String, task_id: u32, workflow_name: String) -> Self {
        let now = Utc::now();
        Self {
            repository,
            branch,
            current_task_id: Some(task_id),
            workflow_name: Some(workflow_name),
            status: PlayStatus::InProgress,
            stage: Some("implementation".to_string()),
            started_at: now,
            last_updated: now,
        }
    }

    /// Convert to `ConfigMap` data format
    fn to_config_map_data(&self) -> BTreeMap<String, String> {
        let mut data = BTreeMap::new();
        data.insert("repository".to_string(), self.repository.clone());
        data.insert("branch".to_string(), self.branch.clone());

        if let Some(task_id) = self.current_task_id {
            data.insert("current-task-id".to_string(), task_id.to_string());
        }

        if let Some(ref workflow_name) = self.workflow_name {
            data.insert("workflow-name".to_string(), workflow_name.clone());
        }

        data.insert("status".to_string(), self.status.to_string());

        if let Some(ref stage) = self.stage {
            data.insert("stage".to_string(), stage.clone());
        }

        data.insert("started-at".to_string(), self.started_at.to_rfc3339());
        data.insert("last-updated".to_string(), self.last_updated.to_rfc3339());

        data
    }

    /// Parse from `ConfigMap` data
    fn from_config_map_data(data: &BTreeMap<String, String>) -> Result<Self> {
        let repository = data
            .get("repository")
            .ok_or_else(|| anyhow!("Missing repository in ConfigMap"))?
            .clone();

        let branch = data
            .get("branch")
            .ok_or_else(|| anyhow!("Missing branch in ConfigMap"))?
            .clone();

        let current_task_id = data
            .get("current-task-id")
            .and_then(|s| s.parse::<u32>().ok());

        let workflow_name = data.get("workflow-name").cloned();

        let status = data
            .get("status")
            .and_then(|s| match s.as_str() {
                "in-progress" => Some(PlayStatus::InProgress),
                "suspended" => Some(PlayStatus::Suspended),
                "failed" => Some(PlayStatus::Failed),
                "completed" => Some(PlayStatus::Completed),
                _ => None,
            })
            .unwrap_or(PlayStatus::InProgress);

        let stage = data.get("stage").cloned();

        let started_at = data
            .get("started-at")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

        let last_updated = data
            .get("last-updated")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

        Ok(Self {
            repository,
            branch,
            current_task_id,
            workflow_name,
            status,
            stage,
            started_at,
            last_updated,
        })
    }
}

/// Generate `ConfigMap` name from repository
/// e.g., "5dlabs/cto" -> "play-progress-5dlabs-cto"
fn configmap_name(repo: &str) -> String {
    format!("play-progress-{}", repo.replace('/', "-"))
}

/// Read progress from `ConfigMap`
pub async fn read_progress(client: &Client, repo: &str) -> Result<Option<PlayProgress>> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), NAMESPACE);
    let name = configmap_name(repo);

    match configmaps.get(&name).await {
        Ok(cm) => {
            if let Some(data) = cm.data {
                let progress = PlayProgress::from_config_map_data(&data)?;
                Ok(Some(progress))
            } else {
                Ok(None)
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => Ok(None),
        Err(e) => Err(anyhow!("Failed to read progress ConfigMap: {e}")),
    }
}

/// Write or update progress to `ConfigMap`
pub async fn write_progress(client: &Client, progress: &PlayProgress) -> Result<()> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), NAMESPACE);
    let name = configmap_name(&progress.repository);

    let mut cm = ConfigMap {
        metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            name: Some(name.clone()),
            namespace: Some(NAMESPACE.to_string()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("play-tracking".to_string(), "true".to_string());
                labels
            }),
            ..Default::default()
        },
        data: Some(progress.to_config_map_data()),
        ..Default::default()
    };

    // Try to get existing ConfigMap
    match configmaps.get(&name).await {
        Ok(existing) => {
            // Update existing
            cm.metadata.resource_version = existing.metadata.resource_version;
            let patch = Patch::Merge(&cm);
            configmaps
                .patch(&name, &PatchParams::default(), &patch)
                .await
                .context("Failed to update progress ConfigMap")?;
            info!("Updated progress ConfigMap for {}", progress.repository);
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            // Create new
            configmaps
                .create(&PostParams::default(), &cm)
                .await
                .context("Failed to create progress ConfigMap")?;
            info!("Created progress ConfigMap for {}", progress.repository);
        }
        Err(e) => {
            return Err(anyhow!("Failed to check for existing ConfigMap: {e}"));
        }
    }

    Ok(())
}

/// Clear progress for a repository
pub async fn clear_progress(client: &Client, repo: &str) -> Result<()> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), NAMESPACE);
    let name = configmap_name(repo);

    match configmaps.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Cleared progress ConfigMap for {}", repo);
            Ok(())
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            // Already deleted, that's fine
            Ok(())
        }
        Err(e) => Err(anyhow!("Failed to delete progress ConfigMap: {e}")),
    }
}

// Note: Workflow reconciliation is handled by the MCP server via argo CLI commands
// to avoid complexity with Kubernetes dynamic clients and workflow CRDs
