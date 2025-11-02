use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::InstallConfig;
use crate::ui;

pub struct ComponentInstaller<'a> {
    config: &'a InstallConfig,
}

impl<'a> ComponentInstaller<'a> {
    pub fn new(config: &'a InstallConfig) -> Self {
        Self { config }
    }

    pub async fn install_component(&self, component: &str) -> Result<()> {
        match component {
            "argocd" => self.install_argocd().await,
            "argo-workflows" => self.install_argo_workflows().await,
            "argo-events" => self.install_argo_events().await,
            "controller" => self.install_controller().await,
            "victoria-metrics" => self.install_victoria_metrics().await,
            "victoria-logs" => self.install_victoria_logs().await,
            "grafana" => self.install_grafana().await,
            "postgres-operator" => self.install_postgres_operator().await,
            "redis-operator" => self.install_redis_operator().await,
            "questdb-operator" => self.install_questdb_operator().await,
            _ => Err(anyhow::anyhow!("Unknown component: {}", component)),
        }
    }

    async fn install_argocd(&self) -> Result<()> {
        ui::print_progress("Creating argocd namespace...");
        self.create_namespace_if_not_exists("argocd")?;

        ui::print_progress("Installing ArgoCD...");
        let output = Command::new("kubectl")
            .args([
                "apply",
                "-n",
                "argocd",
                "-f",
                "https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml",
            ])
            .output()
            .context("Failed to install ArgoCD")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("ArgoCD installation failed: {}", stderr));
        }

        ui::print_progress("Waiting for ArgoCD to be ready...");
        self.wait_for_deployment("argocd", "argocd-server").await?;

        Ok(())
    }

    async fn install_argo_workflows(&self) -> Result<()> {
        ui::print_progress("Creating argo namespace...");
        self.create_namespace_if_not_exists("argo")?;

        ui::print_progress("Adding Argo Helm repository...");
        self.add_helm_repo("argo", "https://argoproj.github.io/argo-helm")?;

        ui::print_progress("Installing Argo Workflows...");
        let output = Command::new("helm")
            .args([
                "install",
                "argo-workflows",
                "argo/argo-workflows",
                "-n",
                "argo",
                "--create-namespace",
                "--wait",
            ])
            .output()
            .context("Failed to install Argo Workflows")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Argo Workflows installation failed: {}",
                stderr
            ));
        }

        Ok(())
    }

    async fn install_argo_events(&self) -> Result<()> {
        ui::print_progress("Installing Argo Events...");
        let output = Command::new("helm")
            .args([
                "install",
                "argo-events",
                "argo/argo-events",
                "-n",
                "argo",
                "--wait",
            ])
            .output()
            .context("Failed to install Argo Events")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Argo Events installation failed: {}",
                stderr
            ));
        }

        Ok(())
    }

    async fn install_controller(&self) -> Result<()> {
        ui::print_progress("Installing CTO Controller...");

        // Build controller image locally for kind
        ui::print_progress("Building controller image...");
        let build_output = Command::new("docker")
            .args([
                "build",
                "-t",
                "ghcr.io/5dlabs/controller:latest",
                "-f",
                "infra/images/controller/Dockerfile",
                ".",
            ])
            .current_dir("/Users/jonathonfritz/code/work-projects/5dlabs/cto")
            .output()
            .context("Failed to build controller image")?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            return Err(anyhow::anyhow!("Controller image build failed: {}", stderr));
        }

        // Load image into kind cluster
        ui::print_progress("Loading controller image into kind...");
        let load_output = Command::new("kind")
            .args([
                "load",
                "docker-image",
                "ghcr.io/5dlabs/controller:latest",
                "--name",
                "cto-platform",
            ])
            .output()
            .context("Failed to load controller image")?;

        if !load_output.status.success() {
            let stderr = String::from_utf8_lossy(&load_output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to load controller image: {}",
                stderr
            ));
        }

        // Install controller Helm chart
        ui::print_progress("Installing controller chart...");
        let helm_output = Command::new("helm")
            .args([
                "install",
                "controller",
                "./infra/charts/controller",
                "-n",
                &self.config.namespace,
                "--create-namespace",
                "--wait",
            ])
            .current_dir("/Users/jonathonfritz/code/work-projects/5dlabs/cto")
            .output()
            .context("Failed to install controller")?;

        if !helm_output.status.success() {
            let stderr = String::from_utf8_lossy(&helm_output.stderr);
            return Err(anyhow::anyhow!("Controller installation failed: {}", stderr));
        }

        Ok(())
    }

    async fn install_victoria_metrics(&self) -> Result<()> {
        ui::print_progress("Adding VictoriaMetrics Helm repository...");
        self.add_helm_repo(
            "victoriametrics",
            "https://victoriametrics.github.io/helm-charts/",
        )?;

        ui::print_progress("Installing VictoriaMetrics...");
        let output = Command::new("helm")
            .args([
                "install",
                "victoria-metrics",
                "victoriametrics/victoria-metrics-single",
                "-n",
                "monitoring",
                "--create-namespace",
                "--wait",
            ])
            .output()
            .context("Failed to install VictoriaMetrics")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "VictoriaMetrics installation failed: {}",
                stderr
            ));
        }

        Ok(())
    }

    async fn install_victoria_logs(&self) -> Result<()> {
        ui::print_progress("Installing VictoriaLogs...");
        let output = Command::new("helm")
            .args([
                "install",
                "victoria-logs",
                "victoriametrics/victoria-logs-single",
                "-n",
                "monitoring",
                "--wait",
            ])
            .output()
            .context("Failed to install VictoriaLogs")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "VictoriaLogs installation failed: {}",
                stderr
            ));
        }

        Ok(())
    }

    async fn install_grafana(&self) -> Result<()> {
        ui::print_progress("Adding Grafana Helm repository...");
        self.add_helm_repo("grafana", "https://grafana.github.io/helm-charts")?;

        ui::print_progress("Installing Grafana...");
        let output = Command::new("helm")
            .args([
                "install",
                "grafana",
                "grafana/grafana",
                "-n",
                "monitoring",
                "--wait",
            ])
            .output()
            .context("Failed to install Grafana")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Grafana installation failed: {}", stderr));
        }

        Ok(())
    }

    async fn install_postgres_operator(&self) -> Result<()> {
        ui::print_progress("Installing PostgreSQL Operator...");
        self.create_namespace_if_not_exists("postgres-operator")?;

        let output = Command::new("helm")
            .args([
                "install",
                "postgres-operator",
                "https://opensource.zalando.com/postgres-operator/charts/postgres-operator",
                "-n",
                "postgres-operator",
                "--wait",
            ])
            .output()
            .context("Failed to install PostgreSQL Operator")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "PostgreSQL Operator installation failed: {}",
                stderr
            ));
        }

        Ok(())
    }

    async fn install_redis_operator(&self) -> Result<()> {
        ui::print_progress("Installing Redis Operator...");
        self.create_namespace_if_not_exists("redis-operator")?;

        // For now, just acknowledge it would be installed
        ui::print_info("Redis Operator installation would go here");

        Ok(())
    }

    async fn install_questdb_operator(&self) -> Result<()> {
        ui::print_progress("Installing QuestDB Operator...");
        self.create_namespace_if_not_exists("questdb-operator")?;

        // For now, just acknowledge it would be installed
        ui::print_info("QuestDB Operator installation would go here");

        Ok(())
    }

    fn create_namespace_if_not_exists(&self, namespace: &str) -> Result<()> {
        let output = Command::new("kubectl")
            .args(["get", "namespace", namespace])
            .output()
            .context("Failed to check namespace")?;

        if !output.status.success() {
            // Namespace doesn't exist, create it
            let create_output = Command::new("kubectl")
                .args(["create", "namespace", namespace])
                .output()
                .context("Failed to create namespace")?;

            if !create_output.status.success() {
                let stderr = String::from_utf8_lossy(&create_output.stderr);
                return Err(anyhow::anyhow!("Failed to create namespace: {}", stderr));
            }
        }

        Ok(())
    }

    fn add_helm_repo(&self, name: &str, url: &str) -> Result<()> {
        let output = Command::new("helm")
            .args(["repo", "add", name, url])
            .output()
            .context("Failed to add Helm repository")?;

        if !output.status.success() {
            // Might already exist, check if it's just an update needed
            let update_output = Command::new("helm")
                .args(["repo", "update"])
                .output()
                .context("Failed to update Helm repositories")?;

            if !update_output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to add Helm repository: {}", stderr));
            }
        }

        Ok(())
    }

    async fn wait_for_deployment(&self, namespace: &str, deployment: &str) -> Result<()> {
        let max_attempts = 60; // 5 minutes (5 seconds * 60)
        let mut attempts = 0;

        loop {
            let output = Command::new("kubectl")
                .args([
                    "get",
                    "deployment",
                    deployment,
                    "-n",
                    namespace,
                    "-o",
                    "jsonpath={.status.availableReplicas}",
                ])
                .output()
                .context("Failed to check deployment status")?;

            if output.status.success() {
                let available = String::from_utf8_lossy(&output.stdout);
                if let Ok(count) = available.parse::<i32>() {
                    if count > 0 {
                        return Ok(());
                    }
                }
            }

            attempts += 1;
            if attempts >= max_attempts {
                return Err(anyhow::anyhow!(
                    "Deployment {} in namespace {} did not become ready within timeout",
                    deployment,
                    namespace
                ));
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}

