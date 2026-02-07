use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "talos-orchestrator")]
#[command(author = "5D Labs")]
#[command(version = "0.1.0")]
#[command(about = "Automated Talos Linux installation on Scaleway bare metal", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    #[arg(short, long)]
    server_id: Option<String>,

    #[arg(short, long)]
    disk: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Install,
    Status,
    GetConfig,
    Reboot,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Load configuration
    let mut config = talos_orchestrator::config::Config::load(&args.config)?;

    // Override with CLI args if provided
    if let Some(id) = args.server_id {
        config.set_from_args(&id, args.disk.as_deref().unwrap_or("/dev/sda"));
    }

    match args.command {
        Commands::Install => {
            let scaleway = talos_orchestrator::scaleway::ScalewayClient::new(&config.scaleway)?;
            let installer = talos_orchestrator::talosctl::TalosInstaller::new(
                PathBuf::from("talosctl"),
                PathBuf::from("/tmp/talos.raw.img"),
            );
            let orchestrator = talos_orchestrator::Orchestrator::new(scaleway, installer, config.clone());
            let server_id = config.server.id.clone();
            let result = orchestrator.install(&server_id).await;
            match result {
                Ok(state) => println!("✅ Success! Server state: {}", state),
                Err(e) => {
                    eprintln!("❌ Installation failed: {:#}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Status => {
            println!("Status command not yet implemented");
        }
        Commands::GetConfig => {
            println!("GetConfig command not yet implemented");
        }
        Commands::Reboot => {
            println!("Reboot command not yet implemented");
        }
    }

    Ok(())
}
