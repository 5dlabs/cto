use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;

use crate::config::InstallConfig;
use crate::ui;

pub struct ClusterProvisioner<'a> {
    config: &'a InstallConfig,
}

impl<'a> ClusterProvisioner<'a> {
    pub const fn new_kind(config: &'a InstallConfig) -> Self {
        Self { config }
    }

    pub fn provision(&self) -> Result<()> {
        // Check if cluster already exists
        if Self::cluster_exists()? {
            ui::print_info("kind cluster 'cto-platform' already exists");
            return Ok(());
        }

        ui::print_progress("Creating kind cluster...");

        let kind_config = Self::generate_kind_config();

        // Write config to temp file
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(temp_file.path(), kind_config)?;

        // Create cluster
        let output = Command::new("kind")
            .args([
                "create",
                "cluster",
                "--name",
                "cto-platform",
                "--config",
                temp_file.path().to_str().unwrap(),
            ])
            .output()
            .context("Failed to create kind cluster")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("kind cluster creation failed: {stderr}"));
        }

        ui::print_success("kind cluster created");

        // Wait for cluster to be ready
        ui::print_progress("Waiting for cluster to be ready...");
        Self::wait_for_cluster_ready()?;

        // Create namespace
        self.create_namespace()?;

        Ok(())
    }

    fn cluster_exists() -> Result<bool> {
        let output = Command::new("kind")
            .args(["get", "clusters"])
            .output()
            .context("Failed to list kind clusters")?;

        if !output.status.success() {
            return Ok(false);
        }

        let clusters = String::from_utf8_lossy(&output.stdout);
        Ok(clusters.contains("cto-platform"))
    }

    fn generate_kind_config() -> String {
        "kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: cto-platform
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: \"cto-platform=true\"
  extraPortMappings:
  # ArgoCD
  - containerPort: 8080
    hostPort: 8080
    protocol: TCP
  # Argo Workflows
  - containerPort: 2746
    hostPort: 2746
    protocol: TCP
  # HTTP
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  # HTTPS
  - containerPort: 443
    hostPort: 443
    protocol: TCP
"
        .to_string()
    }

    fn wait_for_cluster_ready() -> Result<()> {
        let output = Command::new("kubectl")
            .args([
                "wait",
                "--for=condition=Ready",
                "nodes",
                "--all",
                "--timeout=300s",
            ])
            .output()
            .context("Failed to wait for nodes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Cluster not ready: {stderr}"));
        }

        ui::print_success("Cluster is ready");
        Ok(())
    }

    fn create_namespace(&self) -> Result<()> {
        ui::print_progress(&format!("Creating namespace '{}'...", self.config.namespace));

        let output = Command::new("kubectl")
            .args([
                "create",
                "namespace",
                &self.config.namespace,
                "--dry-run=client",
                "-o",
                "yaml",
            ])
            .output()
            .context("Failed to generate namespace manifest")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to create namespace manifest: {stderr}"
            ));
        }

        let manifest = String::from_utf8_lossy(&output.stdout);

        let mut apply_cmd = Command::new("kubectl")
            .args(["apply", "-f", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        apply_cmd
            .stdin
            .as_mut()
            .unwrap()
            .write_all(manifest.as_bytes())
            .context("Failed to write to kubectl stdin")?;

        let _ = apply_cmd.wait().context("Failed to wait for kubectl")?;

        ui::print_success(&format!("Namespace '{}' created", self.config.namespace));

        Ok(())
    }
}
