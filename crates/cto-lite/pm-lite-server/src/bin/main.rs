//! PM Lite binary entry point

use anyhow::Result;
use pm_lite::{Config, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,pm_lite=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load dotenv for local development
    let _ = dotenvy::dotenv();

    // Load configuration - try file first, then environment
    let config = Config::load();

    // Run server
    Server::new(config).run().await
}
