// Example: Install Talos on a Scaleway bare metal server
//
// Usage:
//   cargo run --example install-server -- \
//     --config config.yaml \
//     --server-id "server-uuid-here" \
//     --disk /dev/sda
//
// Environment variables can also be used:
//   export SCALEWAY_PROJECT_ID="..."
//   export SCALEWAY_ACCESS_KEY="..."
//   export SCALEWAY_SECRET_KEY="..."

use talos_orchestrator::{Orchestrator, Config, ScalewayClient, TalosInstaller};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::load("config.yaml")?;

    println!("🔧 Talos Bare Metal Installer");
    println!("============================");
    println!("Server: {}", config.server.id);
    println!("Disk:   {}", config.server.disk);
    println!("Talos:  {} ({})", config.talos.version, config.talos.architecture);
    println!();

    // Initialize components
    let scaleway = ScalewayClient::new(&config.scaleway)?;
    let installer = TalosInstaller::new(
        PathBuf::from("talosctl"),
        PathBuf::from("/tmp/talos.raw.img"),
    );

    // Run installation
    let orchestrator = Orchestrator::new(scaleway, installer, config);

    println!("🚀 Starting installation...");
    let result = orchestrator.install("server-uuid-here").await;

    match result {
        Ok(state) => {
            println!();
            println!("✅ Installation complete!");
            println!("📊 Final state: {}", state);
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ Installation failed: {:#}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
