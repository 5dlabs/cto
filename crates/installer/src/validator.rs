//! Prerequisites validation for CTO installation.
//!
//! Validates that required tools are installed before proceeding with installation.

use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::ui;

/// Validates prerequisites for CTO installation.
pub struct PrerequisitesValidator {
    requirements: Vec<Requirement>,
}

struct Requirement {
    name: String,
    check: Box<dyn Fn() -> Result<bool>>,
    install_instructions: String,
    critical: bool,
}

impl PrerequisitesValidator {
    /// Create a new prerequisites validator for bare metal installation.
    #[must_use]
    pub fn new() -> Self {
        let mut requirements = Vec::new();

        // kubectl (required)
        requirements.push(Requirement {
            name: "kubectl".to_string(),
            check: Box::new(|| {
                Command::new("kubectl")
                    .arg("version")
                    .arg("--client")
                    .output()
                    .map(|output| output.status.success())
                    .context("kubectl not found")
            }),
            install_instructions: "Install kubectl from https://kubernetes.io/docs/tasks/tools/"
                .to_string(),
            critical: true,
        });

        // helm (required)
        requirements.push(Requirement {
            name: "Helm".to_string(),
            check: Box::new(|| {
                Command::new("helm")
                    .arg("version")
                    .output()
                    .map(|output| output.status.success())
                    .context("Helm not found")
            }),
            install_instructions: "Install Helm from https://helm.sh/docs/intro/install/"
                .to_string(),
            critical: true,
        });

        // talosctl (required for bare metal)
        requirements.push(Requirement {
            name: "talosctl".to_string(),
            check: Box::new(|| {
                Command::new("talosctl")
                    .arg("version")
                    .arg("--client")
                    .output()
                    .map(|output| output.status.success())
                    .context("talosctl not found")
            }),
            install_instructions:
                "Install talosctl from https://www.talos.dev/latest/talos-guides/install/talosctl/"
                    .to_string(),
            critical: true,
        });

        // cilium CLI (required for CNI deployment and health checks)
        requirements.push(Requirement {
            name: "cilium CLI".to_string(),
            check: Box::new(|| {
                Command::new("cilium")
                    .arg("version")
                    .output()
                    .map(|output| output.status.success())
                    .context("cilium CLI not found")
            }),
            install_instructions:
                "Install Cilium CLI from https://docs.cilium.io/en/stable/gettingstarted/k8s-install-default/#install-the-cilium-cli"
                    .to_string(),
            critical: true,
        });

        // 1Password CLI (optional, for credential management)
        requirements.push(Requirement {
            name: "1Password CLI (op)".to_string(),
            check: Box::new(|| {
                Command::new("op")
                    .arg("--version")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
                    .then_some(true)
                    .ok_or_else(|| anyhow::anyhow!("op not found"))
            }),
            install_instructions:
                "Install 1Password CLI from https://1password.com/downloads/command-line/ (optional, for credential management)"
                    .to_string(),
            critical: false,
        });

        // Lens/OpenLens (optional, for Kubernetes IDE)
        requirements.push(Requirement {
            name: "Lens/OpenLens".to_string(),
            check: Box::new(|| {
                // Check for Lens Desktop (macOS app)
                if std::path::Path::new("/Applications/Lens.app").exists() {
                    return Ok(true);
                }
                // Check for OpenLens (macOS app)
                if std::path::Path::new("/Applications/OpenLens.app").exists() {
                    return Ok(true);
                }
                // Check for lens command
                if Command::new("lens")
                    .arg("--version")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    return Ok(true);
                }
                Err(anyhow::anyhow!("Lens not found"))
            }),
            install_instructions:
                "Install Lens from https://k8slens.dev/ or OpenLens: brew install --cask openlens (optional, Kubernetes IDE)"
                    .to_string(),
            critical: false,
        });

        Self { requirements }
    }

    /// Validate all prerequisites.
    ///
    /// # Errors
    ///
    /// Returns an error if any critical prerequisites are not met.
    pub fn validate(&self) -> Result<()> {
        println!();
        let mut failures = Vec::new();

        for requirement in &self.requirements {
            let passed = (requirement.check)().unwrap_or(false);
            if passed {
                ui::print_check_result(&requirement.name, true, None);
            } else {
                ui::print_check_result(&requirement.name, false, None);
                failures.push(requirement);
            }
        }

        println!();

        if failures.is_empty() {
            ui::print_success("All prerequisites met!");
        } else {
            let has_critical_failure = failures.iter().any(|f| f.critical);

            if has_critical_failure {
                ui::print_error("Some required tools are missing:");
            } else {
                ui::print_warning("Some optional tools are missing:");
            }

            println!();
            for failure in &failures {
                if failure.critical {
                    println!(
                        "  {} {} - {}",
                        "✗".red(),
                        failure.name.red(),
                        failure.install_instructions.bright_black()
                    );
                } else {
                    println!(
                        "  {} {} - {}",
                        "⚠".yellow(),
                        failure.name.yellow(),
                        failure.install_instructions.bright_black()
                    );
                }
            }
            println!();

            if has_critical_failure {
                return Err(anyhow::anyhow!(
                    "Required prerequisites not met. Please install the required tools and try again."
                ));
            }
        }

        Ok(())
    }

    /// Check if a specific tool is available.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_tool(name: &str) -> bool {
        Command::new(name)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if credentials are available (either env vars or 1Password).
    #[must_use]
    #[allow(dead_code)]
    pub fn has_credentials() -> bool {
        // Check environment variables
        if std::env::var("LATITUDE_API_KEY").is_ok() && std::env::var("LATITUDE_PROJECT_ID").is_ok()
        {
            return true;
        }

        // Check 1Password
        Self::has_tool("op")
    }
}

impl Default for PrerequisitesValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = PrerequisitesValidator::new();
        assert!(!validator.requirements.is_empty());
    }

    #[test]
    fn test_has_tool_nonexistent() {
        assert!(!PrerequisitesValidator::has_tool("nonexistent-tool-xyz"));
    }
}
