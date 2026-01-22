//! Bootstrap resources for pre-ArgoCD cluster setup.
//!
//! These resources must be applied before ArgoCD can deploy the full platform,
//! including namespaces with Pod Security labels and storage configuration.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use tracing::info;

/// Bootstrap resources embedded from infra/cluster-config/.
pub struct BootstrapResources;

impl BootstrapResources {
    /// Apply all pre-ArgoCD bootstrap resources.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl apply fails.
    pub fn apply(kubeconfig: &Path) -> Result<()> {
        info!("Applying bootstrap resources");

        // GPU operator namespace with Pod Security labels
        Self::apply_yaml(
            kubeconfig,
            "gpu-operator namespace",
            include_str!("../../../infra/cluster-config/gpu-operator-namespace.yaml"),
        )?;

        // Mayastor namespace with Pod Security labels
        Self::apply_yaml(
            kubeconfig,
            "mayastor namespace",
            include_str!("../../../infra/cluster-config/mayastor-namespace.yaml"),
        )?;

        // Observability namespace with Pod Security labels
        Self::apply_yaml(
            kubeconfig,
            "observability namespace",
            include_str!("../../../infra/cluster-config/observability-namespace.yaml"),
        )?;

        // NOTE: local-path-config is now applied directly in deploy_local_path_provisioner()
        // to ensure it's applied AFTER the provisioner is installed, not before.
        // See: crates/metal/src/stack.rs

        info!("Bootstrap resources applied successfully");
        Ok(())
    }

    /// Apply a single YAML resource.
    fn apply_yaml(kubeconfig: &Path, name: &str, yaml: &str) -> Result<()> {
        info!(resource = name, "Applying resource");

        let mut child = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "apply",
                "-f",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        // Write YAML to stdin
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
            anyhow::bail!("Failed to apply {name}: {}", stderr.trim());
        }

        Ok(())
    }

    /// Create a namespace if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the kubectl command fails.
    #[allow(dead_code)]
    pub fn create_namespace(kubeconfig: &Path, name: &str) -> Result<()> {
        info!(namespace = name, "Ensuring namespace exists");

        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "create",
                "namespace",
                name,
                "--dry-run=client",
                "-o",
                "yaml",
            ])
            .output()
            .context("Failed to run kubectl create namespace")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to generate namespace YAML: {}", stderr.trim());
        }

        // Apply the generated YAML
        let mut child = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "apply",
                "-f",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl apply")?;

        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(&output.stdout)
                .context("Failed to write namespace YAML to kubectl stdin")?;
        }

        let apply_output = child
            .wait_with_output()
            .context("Failed to wait for kubectl apply")?;

        if !apply_output.status.success() {
            let stderr = String::from_utf8_lossy(&apply_output.stderr);
            anyhow::bail!("Failed to apply namespace {name}: {}", stderr.trim());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bootstrap_resources_compile() {
        // Just verify that the include_str! macros work
        // (i.e., the files exist at compile time)
        let _ = include_str!("../../../infra/cluster-config/gpu-operator-namespace.yaml");
        let _ = include_str!("../../../infra/cluster-config/mayastor-namespace.yaml");
        let _ = include_str!("../../../infra/cluster-config/observability-namespace.yaml");
    }
}
