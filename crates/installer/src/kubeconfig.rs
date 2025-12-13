//! Kubeconfig management and Lens integration.
//!
//! This module handles:
//! - Merging generated kubeconfig into ~/.kube/config
//! - Checking for Lens/OpenLens installation
//! - Providing setup instructions

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use tracing::info;

use crate::ui;

/// Get the default kubeconfig path (~/.kube/config).
#[must_use]
pub fn default_kubeconfig_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".kube").join("config"))
}

/// Merge a kubeconfig file into the user's default kubeconfig.
///
/// This uses `kubectl config view --merge` to safely combine configs.
///
/// # Errors
///
/// Returns an error if the merge fails.
pub fn merge_kubeconfig(source: &Path, cluster_name: &str) -> Result<PathBuf> {
    let default_path = default_kubeconfig_path().context("Could not determine home directory")?;

    // Ensure ~/.kube directory exists
    if let Some(parent) = default_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create ~/.kube directory")?;
    }

    info!(
        source = %source.display(),
        target = %default_path.display(),
        "Merging kubeconfig"
    );

    // Read the source kubeconfig
    let source_content =
        std::fs::read_to_string(source).context("Failed to read generated kubeconfig")?;

    // If default config doesn't exist, just copy the source
    if !default_path.exists() {
        std::fs::write(&default_path, &source_content)
            .context("Failed to write kubeconfig to ~/.kube/config")?;
        info!("Created new kubeconfig at ~/.kube/config");
        return Ok(default_path);
    }

    // Use KUBECONFIG env var trick to merge configs
    // KUBECONFIG=~/.kube/config:/path/to/new/config kubectl config view --flatten
    let kubeconfig_env = format!("{}:{}", default_path.display(), source.display());

    let output = Command::new("kubectl")
        .env("KUBECONFIG", &kubeconfig_env)
        .args(["config", "view", "--flatten"])
        .output()
        .context("Failed to run kubectl config view --flatten")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to merge kubeconfig: {}", stderr.trim());
    }

    // Write the merged config
    std::fs::write(&default_path, &output.stdout)
        .context("Failed to write merged kubeconfig to ~/.kube/config")?;

    // Set the current context to the new cluster
    let set_context = Command::new("kubectl")
        .args(["config", "use-context", cluster_name])
        .output()
        .context("Failed to set kubectl context")?;

    if !set_context.status.success() {
        // Try with admin@ prefix which is common for Talos
        let _ = Command::new("kubectl")
            .args(["config", "use-context", &format!("admin@{cluster_name}")])
            .output();
    }

    info!(
        cluster = cluster_name,
        "Merged kubeconfig and set current context"
    );

    Ok(default_path)
}

/// Check if Lens or OpenLens is installed.
#[derive(Debug, Clone)]
pub enum LensInstallation {
    /// Lens Desktop (commercial) is installed.
    LensDesktop,
    /// OpenLens (open source) is installed.
    OpenLens,
    /// Neither is installed.
    NotInstalled,
}

impl LensInstallation {
    /// Check the system for Lens installations.
    #[must_use]
    pub fn detect() -> Self {
        // Check for Lens Desktop (macOS)
        if Path::new("/Applications/Lens.app").exists() {
            return Self::LensDesktop;
        }

        // Check for OpenLens (macOS)
        if Path::new("/Applications/OpenLens.app").exists() {
            return Self::OpenLens;
        }

        // Check if lens command is available
        if Command::new("lens").arg("--version").output().is_ok() {
            return Self::LensDesktop;
        }

        // Check if openlens command is available
        if Command::new("openlens").arg("--version").output().is_ok() {
            return Self::OpenLens;
        }

        Self::NotInstalled
    }

    /// Get the application name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::LensDesktop => "Lens Desktop",
            Self::OpenLens => "OpenLens",
            Self::NotInstalled => "Lens/OpenLens",
        }
    }

    /// Check if any Lens variant is installed.
    #[must_use]
    pub fn is_installed(&self) -> bool {
        !matches!(self, Self::NotInstalled)
    }
}

/// Print Lens setup instructions.
pub fn print_lens_instructions(cluster_name: &str) {
    let lens = LensInstallation::detect();

    ui::print_section("Lens Kubernetes IDE");

    if lens.is_installed() {
        ui::print_success(&format!("{} is installed!", lens.name()));
        ui::print_info(
            "Your cluster has been added to ~/.kube/config and will appear automatically in Lens.",
        );
        ui::print_info("");
        ui::print_info("To open Lens:");

        match lens {
            LensInstallation::LensDesktop => {
                ui::print_info("  open -a Lens");
            }
            LensInstallation::OpenLens => {
                ui::print_info("  open -a OpenLens");
            }
            LensInstallation::NotInstalled => {}
        }

        ui::print_info("");
        ui::print_info(&format!(
            "Look for the cluster '{cluster_name}' in the catalog."
        ));
    } else {
        ui::print_warning("Lens/OpenLens is not installed.");
        ui::print_info("");
        ui::print_info("Lens is a powerful Kubernetes IDE that makes cluster management easy.");
        ui::print_info(
            "Your kubeconfig has been set up - just install Lens and it will auto-discover your cluster.",
        );
        ui::print_info("");
        ui::print_info("Install options:");
        ui::print_info("  - Lens Desktop: https://k8slens.dev/");
        ui::print_info("  - OpenLens (OSS): brew install --cask openlens");
        ui::print_info("");
        ui::print_info(
            "After installation, open Lens and your cluster will appear in the catalog.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_kubeconfig_path() {
        let path = default_kubeconfig_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(".kube"));
        assert!(path.to_string_lossy().ends_with("config"));
    }

    #[test]
    fn test_lens_detection() {
        // Just verify the function runs without panicking
        let _ = LensInstallation::detect();
    }

    #[test]
    fn test_lens_name() {
        assert_eq!(LensInstallation::LensDesktop.name(), "Lens Desktop");
        assert_eq!(LensInstallation::OpenLens.name(), "OpenLens");
        assert_eq!(LensInstallation::NotInstalled.name(), "Lens/OpenLens");
    }
}
