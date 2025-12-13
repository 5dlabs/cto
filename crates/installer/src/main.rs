//! CTO Platform Installer CLI.
//!
//! This CLI provisions bare metal Kubernetes clusters on Latitude.sh,
//! bootstraps Talos Linux, and deploys the full CTO platform via GitOps.

// Allow product names without backticks in doc comments
#![allow(clippy::doc_markdown)]
// Allow async functions that don't use await (may need await in future)
#![allow(clippy::unused_async)]
// Allow imports after statements in functions
#![allow(clippy::items_after_statements)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod bare_metal;
mod bootstrap;
mod commands;
mod config;
mod gitops;
mod kubeconfig;
mod orchestrator;
mod state;
mod ui;
mod validation;
mod validator;

use commands::install::InstallCommand;
use commands::validate::ValidateCommand;

/// CTO Platform - Bare Metal Kubernetes Platform Installer.
#[derive(Parser)]
#[command(
    name = "cto",
    version,
    about = "CTO Platform bare metal installer",
    long_about = "Install the CTO Platform on bare metal servers.\n\n\
                  This CLI provisions servers on Latitude.sh, installs Talos Linux,\n\
                  bootstraps Kubernetes, and deploys the full platform via GitOps.\n\n\
                  All operations are idempotent - re-running the same command will\n\
                  resume from where it left off."
)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose logging.
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // Install is the primary command, boxing adds indirection
enum Commands {
    /// Install the CTO platform on bare metal.
    ///
    /// Provisions servers, installs Talos Linux, bootstraps Kubernetes,
    /// and deploys the full platform stack via GitOps.
    Install(InstallCommand),

    /// Validate a cluster with AI-powered health checks.
    ///
    /// Runs Claude with full kubectl access to thoroughly test the cluster,
    /// deploy test workloads, and optionally remediate issues.
    Validate(ValidateCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("info,metal=debug,cto_cli=debug")
    } else {
        EnvFilter::new("warn,metal=info,cto_cli=info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Install(cmd) => cmd.run().await,
        Commands::Validate(cmd) => cmd.run().await,
    }
}
