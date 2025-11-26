use anyhow::Result;
use clap::Args;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::config::{ClusterType, InstallConfig, InstallProfile};
use crate::installer::Installer;
use crate::ui;
use crate::validator::PrerequisitesValidator;

/// Install the CTO platform
#[derive(Args)]
pub struct InstallCommand {
    /// Installation profile (minimal, standard, production)
    #[arg(short, long, value_name = "PROFILE")]
    profile: Option<String>,

    /// Use local kind cluster
    #[arg(long)]
    local: bool,

    /// Use remote Kubernetes cluster
    #[arg(long)]
    remote: bool,

    /// Skip interactive prompts (use defaults)
    #[arg(long)]
    non_interactive: bool,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

impl InstallCommand {
    pub async fn run(&self) -> Result<()> {
        ui::print_section("🚀 CTO Platform Installation");

        // Step 1: Validate prerequisites
        ui::print_step(1, 7, "Checking prerequisites");
        let validator = PrerequisitesValidator::new();
        validator.validate()?;

        // Step 2: Gather configuration
        ui::print_step(2, 7, "Configuring installation");
        let config = if let Some(config_file) = &self.config {
            Self::load_config_from_file(config_file)?
        } else if self.non_interactive {
            self.create_default_config()
        } else {
            self.create_interactive_config()?
        };

        println!();
        ui::print_config_summary(&config);
        println!();

        // Confirm installation
        if !self.non_interactive {
            let proceed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Proceed with installation?")
                .default(true)
                .interact()?;

            if !proceed {
                println!("{}", "Installation cancelled.".yellow());
                return Ok(());
            }
        }

        // Step 3: Run installation
        let mut installer = Installer::new(config);
        installer.install().await?;

        ui::print_success("Installation complete! 🎉");
        Ok(())
    }

    fn load_config_from_file(_path: &str) -> Result<InstallConfig> {
        // TODO: Implement config file loading
        Err(anyhow::anyhow!("Config file loading not yet implemented"))
    }

    fn create_default_config(&self) -> InstallConfig {
        let cluster_type = if self.local {
            ClusterType::Kind
        } else if self.remote {
            ClusterType::Remote
        } else {
            ClusterType::Kind // Default to local
        };

        InstallConfig {
            profile: InstallProfile::Minimal,
            cluster_type,
            namespace: "cto".to_string(),
            github_org: None,
            github_repo: None,
            registry: "ghcr.io".to_string(),
            registry_namespace: None,
            domain: None,
            install_monitoring: false,
            install_databases: false,
            auto_generate_config: true,
        }
    }

    #[allow(clippy::too_many_lines, clippy::unused_self)]
    fn create_interactive_config(&self) -> Result<InstallConfig> {
        let theme = ColorfulTheme::default();

        println!();
        println!("{}", "Let's configure your CTO installation.".cyan().bold());
        println!();

        // Step 1: Choose installation profile
        let profile_options = vec![
            "Minimal - Local development (8GB RAM, core components only)",
            "Standard - Team development (16GB RAM, full features)",
            "Production - Enterprise deployment (32GB RAM, high availability)",
        ];

        let profile_idx = Select::with_theme(&theme)
            .with_prompt("Select installation profile")
            .default(0)
            .items(&profile_options)
            .interact()?;

        let profile = match profile_idx {
            1 => InstallProfile::Standard,
            2 => InstallProfile::Production,
            _ => InstallProfile::Minimal,
        };

        println!();

        // Step 2: Choose cluster type
        let cluster_options = vec![
            "Local kind cluster (recommended for development)",
            "Remote Kubernetes cluster (existing cluster)",
        ];

        let cluster_idx = Select::with_theme(&theme)
            .with_prompt("Where should CTO be installed?")
            .default(0)
            .items(&cluster_options)
            .interact()?;

        let cluster_type = match cluster_idx {
            1 => ClusterType::Remote,
            _ => ClusterType::Kind,
        };

        println!();

        // Step 3: Namespace
        let namespace: String = Input::with_theme(&theme)
            .with_prompt("Kubernetes namespace")
            .default("cto".to_string())
            .interact_text()?;

        println!();

        // Step 4: GitHub configuration
        println!(
            "{}",
            "GitHub Integration (can be configured later)".bright_black()
        );

        let configure_github = Confirm::with_theme(&theme)
            .with_prompt("Configure GitHub integration now?")
            .default(false)
            .interact()?;

        let (github_org, github_repo) = if configure_github {
            let org: String = Input::with_theme(&theme)
                .with_prompt("GitHub organization or username")
                .interact_text()?;

            let repo: String = Input::with_theme(&theme)
                .with_prompt("GitHub repository")
                .interact_text()?;

            (Some(org), Some(repo))
        } else {
            (None, None)
        };

        println!();

        // Step 5: Container registry
        println!("{}", "Container Registry Configuration".bright_black());

        let registry: String = Input::with_theme(&theme)
            .with_prompt("Container registry URL")
            .default("ghcr.io".to_string())
            .interact_text()?;

        let registry_namespace = if registry.contains("ghcr.io") {
            let ns: String = Input::with_theme(&theme)
                .with_prompt("Registry namespace (e.g., your-org)")
                .interact_text()?;
            Some(ns)
        } else {
            None
        };

        println!();

        // Step 6: Optional features (based on profile)
        let (install_monitoring, install_databases) = match profile {
            InstallProfile::Minimal => (false, false),
            InstallProfile::Standard | InstallProfile::Production => {
                let monitoring = Confirm::with_theme(&theme)
                    .with_prompt("Install monitoring stack? (Grafana, VictoriaMetrics)")
                    .default(true)
                    .interact()?;

                let databases = Confirm::with_theme(&theme)
                    .with_prompt("Install database operators? (PostgreSQL, Redis, QuestDB)")
                    .default(true)
                    .interact()?;

                (monitoring, databases)
            }
        };

        println!();

        // Step 7: Domain (for production)
        let domain = if matches!(profile, InstallProfile::Production) {
            let use_domain = Confirm::with_theme(&theme)
                .with_prompt("Configure custom domain?")
                .default(false)
                .interact()?;

            if use_domain {
                let d: String = Input::with_theme(&theme)
                    .with_prompt("Domain name (e.g., agents.company.com)")
                    .interact_text()?;
                Some(d)
            } else {
                None
            }
        } else {
            None
        };

        Ok(InstallConfig {
            profile,
            cluster_type,
            namespace,
            github_org,
            github_repo,
            registry,
            registry_namespace,
            domain,
            install_monitoring,
            install_databases,
            auto_generate_config: true,
        })
    }
}
