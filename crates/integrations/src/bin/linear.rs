//! Linear service binary.
//!
//! Standalone HTTP service for Linear webhook handling.

use anyhow::{Context, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use integrations::{config::Config, server, LinearClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("linear=info".parse()?))
        .init();

    info!("Starting Linear service...");

    // Load configuration
    let config = Config::default();

    if !config.enabled {
        error!("LINEAR_ENABLED is not set to true. Service will not process webhooks.");
    }

    // Initialize Kubernetes client
    let kube_client = kube::Client::try_default()
        .await
        .context("Failed to create Kubernetes client")?;

    info!(namespace = %config.namespace, "Connected to Kubernetes");

    // Initialize Linear client
    let linear_client = match &config.oauth_token {
        Some(token) => match LinearClient::new(token) {
            Ok(client) => {
                info!("Linear API client configured");
                Some(client)
            }
            Err(e) => {
                error!(error = %e, "Failed to create Linear client");
                None
            }
        },
        None => {
            info!("No LINEAR_OAUTH_TOKEN configured - API calls will be disabled");
            None
        }
    };

    // Build application state
    let state = server::AppState {
        config: config.clone(),
        kube_client,
        linear_client,
    };

    // Build router
    let app = server::build_router(state);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    info!(port = config.port, "Linear service listening");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
