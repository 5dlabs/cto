//! GitOps verification for ArgoCD application sync status.
//!
//! This module provides functionality to:
//! - Apply the app-of-apps manifest to bootstrap GitOps
//! - Wait for all ArgoCD applications to be synced and healthy
//! - Report on application status

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use kube::api::{Api, DynamicObject, ListParams};
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::discovery::ApiResource;
use kube::{Client, Config};
use tracing::{debug, info, warn};

use crate::ui;

/// ArgoCD Application API resource definition.
fn argocd_application_api() -> ApiResource {
    ApiResource {
        group: "argoproj.io".to_string(),
        version: "v1alpha1".to_string(),
        api_version: "argoproj.io/v1alpha1".to_string(),
        kind: "Application".to_string(),
        plural: "applications".to_string(),
    }
}

/// Application status from ArgoCD.
#[derive(Debug, Clone)]
pub struct AppStatus {
    /// Application name.
    pub name: String,
    /// Sync status (Synced, OutOfSync, Unknown).
    pub sync_status: String,
    /// Health status (Healthy, Degraded, Progressing, Missing, Unknown).
    pub health_status: String,
    /// Optional status message.
    pub message: Option<String>,
}

/// Sync report for all applications.
#[derive(Debug, Default)]
pub struct SyncReport {
    /// Total number of applications.
    pub total_count: usize,
    /// Number of synced applications.
    pub synced_count: usize,
    /// Number of healthy applications.
    pub healthy_count: usize,
    /// Applications that are not synced or healthy.
    pub degraded: Vec<AppStatus>,
}

impl SyncReport {
    /// Check if all applications are synced and healthy.
    #[must_use]
    pub fn all_green(&self) -> bool {
        self.synced_count == self.total_count && self.healthy_count == self.total_count
    }
}

/// GitOps verifier for ArgoCD application sync.
pub struct GitOpsVerifier {
    /// Kubernetes client.
    client: Client,
}

impl GitOpsVerifier {
    /// Create a verifier from a kubeconfig file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the kubeconfig cannot be read or the client cannot be created.
    pub async fn from_kubeconfig(path: &Path) -> Result<Self> {
        let kubeconfig = Kubeconfig::read_from(path)
            .with_context(|| format!("Failed to read kubeconfig from {}", path.display()))?;

        let config = Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
            .await
            .context("Failed to create Kubernetes config from kubeconfig")?;

        let client = Client::try_from(config).context("Failed to create Kubernetes client")?;

        Ok(Self { client })
    }

    /// Apply the app-of-apps manifest from GitHub using kubectl.
    ///
    /// This method first applies the platform project (required by app-of-apps),
    /// then applies the app-of-apps manifest itself.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching or applying the manifests fails.
    pub async fn apply_app_of_apps(&self, repo: &str, branch: &str) -> Result<()> {
        info!(repo = %repo, branch = %branch, "Applying app-of-apps manifest");

        // Construct raw GitHub URL base
        // raw.githubusercontent.com format: https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}
        let base_url = repo.replace("github.com", "raw.githubusercontent.com");

        // First apply the platform project (required by app-of-apps)
        let project_url =
            format!("{base_url}/{branch}/infra/gitops/projects/platform-project.yaml");
        debug!(url = %project_url, "Fetching platform project manifest");

        let project_manifest = reqwest::get(&project_url)
            .await
            .context("Failed to fetch platform project manifest")?
            .text()
            .await
            .context("Failed to read platform project manifest body")?;

        self.kubectl_apply(&project_manifest).await?;
        info!("Platform project applied successfully");

        // Now apply the app-of-apps
        let app_url = format!("{base_url}/{branch}/infra/gitops/app-of-apps.yaml");
        debug!(url = %app_url, "Fetching app-of-apps manifest");

        let manifest = reqwest::get(&app_url)
            .await
            .context("Failed to fetch app-of-apps manifest")?
            .text()
            .await
            .context("Failed to read app-of-apps manifest body")?;

        // Apply using kubectl since we have complex YAML
        self.kubectl_apply(&manifest).await?;

        info!("App-of-apps manifest applied successfully");
        Ok(())
    }

    /// Apply YAML using kubectl.
    async fn kubectl_apply(&self, yaml: &str) -> Result<()> {
        let mut child = Command::new("kubectl")
            .args(["apply", "-f", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(yaml.as_bytes())
                .context("Failed to write YAML to kubectl stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for kubectl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl apply failed: {}", stderr.trim());
        }

        Ok(())
    }

    /// Wait for all ArgoCD applications to be synced and healthy.
    ///
    /// # Errors
    ///
    /// Returns an error if the timeout is reached or listing apps fails.
    pub async fn wait_for_full_sync(&self, timeout: Duration) -> Result<SyncReport> {
        let start = Instant::now();
        let poll_interval = Duration::from_secs(15);

        info!(
            timeout_secs = timeout.as_secs(),
            "Waiting for all applications to sync"
        );

        loop {
            if start.elapsed() > timeout {
                let report = self.get_sync_report().await?;
                anyhow::bail!(
                    "Timeout waiting for GitOps sync. {} of {} apps synced, {} of {} healthy. \
                     Degraded apps: {}",
                    report.synced_count,
                    report.total_count,
                    report.healthy_count,
                    report.total_count,
                    report
                        .degraded
                        .iter()
                        .map(|a| format!("{}({})", a.name, a.health_status))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            let report = self.get_sync_report().await?;

            ui::print_gitops_progress(
                report.synced_count,
                report.healthy_count,
                report.total_count,
            );

            if report.all_green() {
                info!(
                    total = report.total_count,
                    "All applications synced and healthy"
                );
                return Ok(report);
            }

            // Log degraded apps at debug level
            for app in &report.degraded {
                debug!(
                    app = %app.name,
                    sync = %app.sync_status,
                    health = %app.health_status,
                    message = app.message.as_deref().unwrap_or(""),
                    "Degraded application"
                );
            }

            // Show warning for apps that are stuck
            let stuck_apps: Vec<_> = report
                .degraded
                .iter()
                .filter(|a| {
                    a.health_status == "Degraded"
                        || a.health_status == "Missing"
                        || a.sync_status == "OutOfSync"
                })
                .collect();

            if !stuck_apps.is_empty() && start.elapsed() > Duration::from_secs(120) {
                for app in stuck_apps.iter().take(5) {
                    warn!(
                        app = %app.name,
                        health = %app.health_status,
                        "Application not progressing"
                    );
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Get the current sync status of all applications.
    ///
    /// # Errors
    ///
    /// Returns an error if listing applications fails.
    pub async fn get_sync_report(&self) -> Result<SyncReport> {
        let api_resource = argocd_application_api();
        let apps: Api<DynamicObject> = Api::all_with(self.client.clone(), &api_resource);

        let list = apps
            .list(&ListParams::default())
            .await
            .context("Failed to list ArgoCD applications")?;

        let mut report = SyncReport {
            total_count: list.items.len(),
            ..Default::default()
        };

        for app in list.items {
            let name = app
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            // Extract sync status from app.data["status"]["sync"]["status"]
            let sync_status = app
                .data
                .get("status")
                .and_then(|s| s.get("sync"))
                .and_then(|s| s.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Extract health status from app.data["status"]["health"]["status"]
            let health_status = app
                .data
                .get("status")
                .and_then(|s| s.get("health"))
                .and_then(|h| h.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Extract message from app.data["status"]["health"]["message"]
            let message = app
                .data
                .get("status")
                .and_then(|s| s.get("health"))
                .and_then(|h| h.get("message"))
                .and_then(|m| m.as_str())
                .map(ToString::to_string);

            // Count synced
            if sync_status == "Synced" {
                report.synced_count += 1;
            }

            // Count healthy
            if health_status == "Healthy" {
                report.healthy_count += 1;
            } else {
                report.degraded.push(AppStatus {
                    name,
                    sync_status,
                    health_status,
                    message,
                });
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_report_all_green() {
        let report = SyncReport {
            total_count: 10,
            synced_count: 10,
            healthy_count: 10,
            degraded: vec![],
        };
        assert!(report.all_green());

        let report = SyncReport {
            total_count: 10,
            synced_count: 9,
            healthy_count: 10,
            degraded: vec![],
        };
        assert!(!report.all_green());

        let report = SyncReport {
            total_count: 10,
            synced_count: 10,
            healthy_count: 9,
            degraded: vec![AppStatus {
                name: "test".to_string(),
                sync_status: "Synced".to_string(),
                health_status: "Degraded".to_string(),
                message: None,
            }],
        };
        assert!(!report.all_green());
    }

    #[test]
    fn test_argocd_api_resource() {
        let api = argocd_application_api();
        assert_eq!(api.group, "argoproj.io");
        assert_eq!(api.version, "v1alpha1");
        assert_eq!(api.kind, "Application");
        assert_eq!(api.plural, "applications");
    }
}
