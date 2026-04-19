//! Token health manager for Linear OAuth apps.
//!
//! Proactively refreshes agent OAuth tokens before expiration to avoid
//! manual intervention during intake or play workflows.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::handlers::oauth::{
    mint_client_credentials_token, refresh_access_token, store_access_token_public,
};

const DEFAULT_REFRESH_INTERVAL_SECS: u64 = 300;
const REFRESH_BUFFER_SECS: i64 = 3600;
const DEFAULT_REFRESH_CONCURRENCY: usize = 3;

/// Background task that keeps agent OAuth tokens fresh.
pub struct TokenHealthManager {
    config: Config,
    kube_client: kube::Client,
    refresh_interval: Duration,
    refresh_semaphore: Arc<Semaphore>,
}

impl TokenHealthManager {
    /// Create a new token health manager with default settings.
    #[must_use]
    pub fn new(config: Config, kube_client: kube::Client) -> Self {
        Self {
            config,
            kube_client,
            refresh_interval: Duration::from_secs(DEFAULT_REFRESH_INTERVAL_SECS),
            refresh_semaphore: Arc::new(Semaphore::new(DEFAULT_REFRESH_CONCURRENCY)),
        }
    }

    /// Run the background refresh loop.
    pub async fn run(self) {
        let mut interval = tokio::time::interval(self.refresh_interval);
        loop {
            interval.tick().await;
            if let Err(e) = self.refresh_expiring_tokens().await {
                warn!(error = %e, "Token health sweep failed");
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn refresh_expiring_tokens(&self) -> Result<(), String> {
        let now = Utc::now().timestamp();
        let candidates = {
            let linear_config = self
                .config
                .linear
                .read()
                .map_err(|e| format!("Failed to read Linear config: {e}"))?;

            linear_config
                .apps
                .iter()
                .filter_map(|(agent_name, app)| {
                    if app.can_refresh() {
                        app.access_token.as_ref()?;

                        let needs_refresh = match app.expires_at {
                            Some(expires_at) => expires_at - now < REFRESH_BUFFER_SECS,
                            None => true,
                        };

                        if !needs_refresh {
                            return None;
                        }

                        let refresh_token = app.refresh_token.clone()?;

                        return Some(TokenRotationCandidate::Refresh {
                            agent_name: agent_name.clone(),
                            refresh_token,
                            client_id: app.client_id.clone(),
                            client_secret: app.client_secret.clone(),
                            expires_at: app.expires_at,
                        });
                    }

                    if app.should_proactively_mint_client_credentials() {
                        return Some(TokenRotationCandidate::Mint {
                            agent_name: agent_name.clone(),
                            client_id: app.client_id.clone(),
                            client_secret: app.client_secret.clone(),
                            expires_at: app.expires_at,
                        });
                    }

                    None
                })
                .collect::<Vec<_>>()
        };

        if candidates.is_empty() {
            debug!("No Linear tokens need refresh");
            return Ok(());
        }

        info!(
            count = candidates.len(),
            "Refreshing expiring Linear tokens"
        );

        let mut join_set = JoinSet::new();
        for candidate in candidates {
            let config = self.config.clone();
            let kube_client = self.kube_client.clone();
            let semaphore = Arc::clone(&self.refresh_semaphore);

            join_set.spawn(async move {
                let _permit = semaphore.acquire().await.map_err(|_| "Semaphore closed")?;
                match candidate {
                    TokenRotationCandidate::Refresh {
                        agent_name,
                        refresh_token,
                        client_id,
                        client_secret,
                        expires_at,
                    } => {
                        refresh_agent_token(
                            &config,
                            &kube_client,
                            &agent_name,
                            &refresh_token,
                            &client_id,
                            &client_secret,
                            expires_at,
                        )
                        .await
                    }
                    TokenRotationCandidate::Mint {
                        agent_name,
                        client_id,
                        client_secret,
                        expires_at,
                    } => {
                        mint_agent_token(
                            &config,
                            &kube_client,
                            &agent_name,
                            &client_id,
                            &client_secret,
                            expires_at,
                        )
                        .await
                    }
                }
            });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => warn!(error = %e, "Token refresh task failed"),
                Err(e) => warn!(error = %e, "Token refresh task join error"),
            }
        }

        Ok(())
    }
}

enum TokenRotationCandidate {
    Refresh {
        agent_name: String,
        refresh_token: String,
        client_id: String,
        client_secret: String,
        expires_at: Option<i64>,
    },
    Mint {
        agent_name: String,
        client_id: String,
        client_secret: String,
        expires_at: Option<i64>,
    },
}

async fn refresh_agent_token(
    config: &Config,
    kube_client: &kube::Client,
    agent_name: &str,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
    expires_at: Option<i64>,
) -> Result<(), String> {
    let ttl = expires_at.map(|exp| exp - Utc::now().timestamp());
    info!(
        agent = %agent_name,
        ttl_seconds = ?ttl,
        "Refreshing Linear OAuth token"
    );

    let token_response = refresh_access_token(refresh_token, client_id, client_secret).await?;

    store_access_token_public(
        kube_client,
        &config.namespace,
        agent_name,
        &token_response.access_token,
        token_response.refresh_token.as_deref(),
        token_response.expires_in,
    )
    .await
    .map_err(|e| format!("Failed to persist refreshed token: {e}"))?;

    if let Ok(mut linear_config) = config.linear.write() {
        linear_config.update_tokens(
            agent_name,
            &token_response.access_token,
            token_response.refresh_token.as_deref(),
            token_response.expires_in,
        );
        info!(agent = %agent_name, "Updated in-memory token config");
    } else {
        warn!(
            agent = %agent_name,
            "Failed to acquire write lock for in-memory token update"
        );
    }

    Ok(())
}

async fn mint_agent_token(
    config: &Config,
    kube_client: &kube::Client,
    agent_name: &str,
    client_id: &str,
    client_secret: &str,
    expires_at: Option<i64>,
) -> Result<(), String> {
    let ttl = expires_at.map(|exp| exp - Utc::now().timestamp());
    info!(
        agent = %agent_name,
        ttl_seconds = ?ttl,
        "Minting Linear access token via client_credentials"
    );

    let token_response = mint_client_credentials_token(client_id, client_secret).await?;

    store_access_token_public(
        kube_client,
        &config.namespace,
        agent_name,
        &token_response.access_token,
        None,
        token_response.expires_in,
    )
    .await
    .map_err(|e| format!("Failed to persist minted token: {e}"))?;

    if let Ok(mut linear_config) = config.linear.write() {
        linear_config.update_tokens(
            agent_name,
            &token_response.access_token,
            None,
            token_response.expires_in,
        );
        info!(
            agent = %agent_name,
            "Updated in-memory token config from client_credentials"
        );
    } else {
        warn!(
            agent = %agent_name,
            "Failed to acquire write lock for in-memory token update"
        );
    }

    Ok(())
}
