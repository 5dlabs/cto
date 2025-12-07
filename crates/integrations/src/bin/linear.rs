//! Linear service binary.
//!
//! Standalone HTTP service for Linear webhook handling.

use anyhow::{Context, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use integrations::{config::Config, ensure_github_webhooks, server, LinearClient};

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
    let linear_client = if let Some(token) = &config.oauth_token {
        match LinearClient::new(token) {
            Ok(client) => {
                info!("Linear API client configured");

                // Ensure CTO config labels exist (cli:*, model:*)
                match client.ensure_cto_config_labels().await {
                    Ok(labels) => {
                        info!(
                            count = labels.len(),
                            "CTO config labels initialized"
                        );
                    }
                    Err(e) => {
                        // Non-fatal - labels might already exist or we lack permissions
                        info!(
                            error = %e,
                            "Could not auto-create CTO config labels (may already exist)"
                        );
                    }
                }

                Some(client)
            }
            Err(e) => {
                error!(error = %e, "Failed to create Linear client");
                None
            }
        }
    } else {
        info!("No LINEAR_OAUTH_TOKEN configured - API calls will be disabled");
        None
    };

    // Ensure GitHub webhooks are configured
    if let (Some(token), Some(callback_url)) = (&config.github_token, &config.webhook_callback_url)
    {
        if config.github_webhook_repos.is_empty() {
            info!("No GITHUB_WEBHOOK_REPOS configured - skipping GitHub webhook setup");
        } else {
            match ensure_github_webhooks(token, callback_url, &config.github_webhook_repos).await {
                Ok(results) => {
                    let success = results.iter().filter(|(_, ok)| *ok).count();
                    let failed = results.len() - success;
                    if failed > 0 {
                        info!(
                            success = success,
                            failed = failed,
                            "GitHub webhooks initialization completed with some failures"
                        );
                    } else {
                        info!(
                            count = success,
                            "GitHub webhooks initialized successfully"
                        );
                    }
                }
                Err(e) => {
                    // Non-fatal - continue starting the service
                    info!(
                        error = %e,
                        "Could not initialize GitHub webhooks"
                    );
                }
            }
        }
    } else {
        if config.github_token.is_none() {
            info!("No GITHUB_TOKEN configured - skipping GitHub webhook setup");
        }
        if config.webhook_callback_url.is_none() {
            info!("No WEBHOOK_CALLBACK_URL configured - skipping GitHub webhook setup");
        }
    }

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
