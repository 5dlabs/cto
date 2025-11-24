use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;
use sysinfo::System;

use crate::ui;

/// Validates prerequisites for CTO installation
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
    pub fn new() -> Self {
        let mut requirements = Vec::new();

        // Docker
        requirements.push(Requirement {
            name: "Docker".to_string(),
            check: Box::new(|| {
                Command::new("docker")
                    .arg("--version")
                    .output()
                    .map(|output| output.status.success())
                    .context("Docker not found")
            }),
            install_instructions: "Install Docker from https://docker.com".to_string(),
            critical: true,
        });

        // kubectl
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

        // helm
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

        // kind (optional, only if using local cluster)
        requirements.push(Requirement {
            name: "kind".to_string(),
            check: Box::new(|| {
                Command::new("kind")
                    .arg("version")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
                    .then_some(true)
                    .ok_or_else(|| anyhow::anyhow!("kind not found"))
            }),
            install_instructions:
                "Install kind from https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
                    .to_string(),
            critical: false,
        });

        // System resources
        requirements.push(Requirement {
            name: "System Memory".to_string(),
            check: Box::new(|| {
                let mut sys = System::new_all();
                sys.refresh_memory();
                let total_memory_gb = sys.total_memory() / 1024 / 1024 / 1024;

                if total_memory_gb < 8 {
                    return Ok(false);
                }
                Ok(true)
            }),
            install_instructions: "At least 8GB of RAM required for minimal installation"
                .to_string(),
            critical: true,
        });

        Self { requirements }
    }

    pub fn validate(&self) -> Result<()> {
        println!();
        let mut failures = Vec::new();

        for requirement in &self.requirements {
            if let Ok(true) = (requirement.check)() {
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
            ui::print_warning("Some prerequisites are not met:");
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

            if failures.iter().any(|f| f.critical) {
                return Err(anyhow::anyhow!(
                    "Critical prerequisites not met. Please install the required tools and try again."
                ));
            }
        }

        Ok(())
    }
}

impl Default for PrerequisitesValidator {
    fn default() -> Self {
        Self::new()
    }
}
