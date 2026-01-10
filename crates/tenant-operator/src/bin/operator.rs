//! Tenant Operator Binary
//!
//! Entry point for the tenant-operator Kubernetes controller.

use tenant_operator::run_controller;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting CTO Tenant Operator");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Run the controller
    run_controller().await?;

    Ok(())
}
