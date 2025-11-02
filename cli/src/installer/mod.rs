use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::config::{ClusterType, CtoConfig, InstallConfig};
use crate::ui;

mod cluster;
mod components;
mod config_generator;

use cluster::ClusterProvisioner;
use components::ComponentInstaller;
use config_generator::ConfigGenerator;

pub struct Installer {
    config: InstallConfig,
}

impl Installer {
    pub fn new(config: InstallConfig) -> Self {
        Self { config }
    }

    pub async fn install(&mut self) -> Result<()> {
        ui::print_section("🔧 Installing CTO Platform");

        // Step 3: Setup/validate cluster
        ui::print_step(3, 7, "Setting up Kubernetes cluster");
        self.setup_cluster()?;

        // Step 4: Install core components
        ui::print_step(4, 7, "Installing core components");
        self.install_core_components().await?;

        // Step 5: Install optional components
        if self.config.install_monitoring || self.config.install_databases {
            ui::print_step(5, 7, "Installing optional components");
            self.install_optional_components().await?;
        } else {
            ui::print_step(5, 7, "Skipping optional components");
        }

        // Step 6: Install CTO binary
        ui::print_step(6, 7, "Installing CTO binary");
        self.install_cto_binary()?;

        // Step 7: Generate configuration
        if self.config.auto_generate_config {
            ui::print_step(7, 7, "Generating cto-config.json");
            self.generate_config()?;
        } else {
            ui::print_step(7, 7, "Skipping config generation");
        }

        ui::print_section("✨ Installation Summary");
        self.print_access_info();

        Ok(())
    }

    fn setup_cluster(&self) -> Result<()> {
        match self.config.cluster_type {
            ClusterType::Kind => {
                ui::print_component("Local kind cluster");
                let provisioner = ClusterProvisioner::new_kind(&self.config);
                provisioner.provision()?;
            }
            ClusterType::Remote => {
                ui::print_component("Remote Kubernetes cluster");
                ui::print_info("Using existing cluster");

                // Validate cluster access
                let output = Command::new("kubectl")
                    .arg("cluster-info")
                    .output()
                    .context("Failed to check cluster connection")?;

                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "Cannot connect to Kubernetes cluster. Please check your kubeconfig."
                    ));
                }

                ui::print_success("Connected to cluster");
            }
        }

        Ok(())
    }

    async fn install_core_components(&self) -> Result<()> {
        let installer = ComponentInstaller::new(&self.config);

        // Core components in order
        let core_components = vec![
            "argocd",
            "argo-workflows",
            "argo-events",
            "controller",
        ];

        for component in core_components {
            ui::print_component(component);
            installer.install_component(component).await?;
            ui::print_success(&format!("{component} installed"));
        }

        Ok(())
    }

    async fn install_optional_components(&self) -> Result<()> {
        let installer = ComponentInstaller::new(&self.config);

        if self.config.install_monitoring {
            ui::print_component("Monitoring Stack");
            installer.install_component("victoria-metrics").await?;
            installer.install_component("victoria-logs").await?;
            installer.install_component("grafana").await?;
            ui::print_success("Monitoring stack installed");
        }

        if self.config.install_databases {
            ui::print_component("Database Operators");
            installer.install_component("postgres-operator").await?;
            installer.install_component("redis-operator").await?;
            installer.install_component("questdb-operator").await?;
            ui::print_success("Database operators installed");
        }

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn install_cto_binary(&self) -> Result<()> {
        ui::print_component("CTO Binary");

        // For now, we'll build from source
        // In production, this would download a pre-built binary

        ui::print_progress("Building from source...");

        let output = Command::new("cargo")
            .args(["build", "--release", "-p", "cto-mcp"])
            .output()
            .context("Failed to build CTO binary")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Build failed: {stderr}"));
        }

        ui::print_success("CTO binary built successfully");

        Ok(())
    }

    fn generate_config(&self) -> Result<()> {
        let _generator = ConfigGenerator::new(&self.config);
        let config = CtoConfig::default_config(&self.config);

        let config_path = PathBuf::from("cto-config.json");
        ConfigGenerator::write_config(&config, &config_path)?;

        ui::print_success(&format!(
            "Generated configuration at {}",
            config_path.display()
        ));

        Ok(())
    }

    fn print_access_info(&self) {
        println!("{}", "Next Steps".cyan().bold());
        println!("{}", "─".repeat(70).bright_black());
        println!();

        println!("  1️⃣  {} Check cluster status:", "Verify Installation:".bold());
        println!("     {}", "kubectl get pods -n agent-platform".bright_black());
        println!();

        println!("  2️⃣  {} Access ArgoCD:", "ArgoCD:".bold());
        println!(
            "     {}",
            "kubectl port-forward svc/argocd-server -n argocd 8080:443".bright_black()
        );
        println!("     {}", "https://localhost:8080".cyan());
        println!("     Username: {}", "admin".yellow());
        println!("     Password: Get with:");
        println!(
            "     {}",
            "kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath=\"{.data.password}\" | base64 -d".bright_black()
        );
        println!();

        println!("  3️⃣  {} Access Argo Workflows:", "Argo Workflows:".bold());
        println!(
            "     {}",
            "kubectl port-forward svc/argo-workflows-server -n argo 2746:2746".bright_black()
        );
        println!("     {}", "http://localhost:2746".cyan());
        println!();

        if self.config.github_org.is_none() {
            println!("  4️⃣  {} Configure GitHub integration:", "GitHub:".bold());
            println!("     - Create GitHub Apps for your agents");
            println!("     - Update cto-config.json with app IDs");
            println!("     - Add secrets to Kubernetes");
            println!();
        }

        println!("{}", "Documentation".cyan().bold());
        println!("{}", "─".repeat(70).bright_black());
        println!();
        println!("  📖 Full documentation: {}", "https://github.com/5dlabs/cto".cyan());
        println!("  💬 Support: {}", "https://github.com/5dlabs/cto/discussions".cyan());
        println!();
    }
}

