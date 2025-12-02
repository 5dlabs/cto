use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod agent_defs;
mod commands;
mod config;
mod installer;
mod tui;
mod ui;
mod validator;

use commands::install::InstallCommand;

/// CTO Platform - Multi-Agent Development Orchestration
#[derive(Parser)]
#[command(
    name = "cto",
    version,
    about = "CTO Platform installer and management CLI",
    long_about = "Install and manage the CTO multi-agent development platform.\n\n\
                  The CTO platform orchestrates AI agents (Rex, Cleo, Tess, Blaze, Cipher, Morgan)\n\
                  to automate software development workflows with production-grade quality assurance."
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run in demo mode (no actual installation)
    #[arg(long, global = true)]
    demo: bool,

    /// Use legacy CLI instead of TUI
    #[arg(long, global = true)]
    no_tui: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install the CTO platform (launches TUI by default)
    Install(InstallCommand),

    /// Show platform status
    Status,

    /// Upgrade platform components
    Upgrade,

    /// Uninstall the platform
    Uninstall,

    /// Validate installation
    Validate,

    /// Check system requirements
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // If no command specified, launch TUI installer
    if cli.command.is_none() {
        return tui::run(cli.demo).await;
    }

    match cli.command.unwrap() {
        Commands::Install(cmd) => {
            if cli.no_tui {
                // Legacy CLI mode
                print_banner();
                cmd.run().await
            } else {
                // TUI mode
                tui::run(cli.demo).await
            }
        }
        Commands::Status => {
            print_banner();
            println!("{}", "Status command not yet implemented".yellow());
            Ok(())
        }
        Commands::Upgrade => {
            print_banner();
            println!("{}", "Upgrade command not yet implemented".yellow());
            Ok(())
        }
        Commands::Uninstall => {
            print_banner();
            println!("{}", "Uninstall command not yet implemented".yellow());
            Ok(())
        }
        Commands::Validate => {
            print_banner();
            println!("{}", "Validate command not yet implemented".yellow());
            Ok(())
        }
        Commands::Doctor => {
            print_banner();
            run_doctor().await
        }
    }
}

/// Run the doctor command to check system requirements
async fn run_doctor() -> Result<()> {
    use crate::validator::PrerequisitesValidator;

    println!();
    println!("{}", "Checking system requirements...".cyan().bold());
    println!();

    let validator = PrerequisitesValidator::new();
    validator.validate()?;

    Ok(())
}

fn print_banner() {
    let banner = r"
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║   ██████╗████████╗ ██████╗     ██████╗ ██╗      █████╗  ║
    ║  ██╔════╝╚══██╔══╝██╔═══██╗    ██╔══██╗██║     ██╔══██╗ ║
    ║  ██║        ██║   ██║   ██║    ██████╔╝██║     ███████║ ║
    ║  ██║        ██║   ██║   ██║    ██╔═══╝ ██║     ██╔══██║ ║
    ║  ╚██████╗   ██║   ╚██████╔╝    ██║     ███████╗██║  ██║ ║
    ║   ╚═════╝   ╚═╝    ╚═════╝     ╚═╝     ╚══════╝╚═╝  ╚═╝ ║
    ║                                                           ║
    ║         Multi-Agent Development Orchestration            ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝
";

    println!("{}", banner.cyan().bold());
}
