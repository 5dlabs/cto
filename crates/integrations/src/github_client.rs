//! GitHub API client for webhook management.

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

const GITHUB_API_URL: &str = "https://api.github.com";

/// GitHub API client for managing webhooks.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
}

/// GitHub webhook configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub content_type: String,
    #[serde(default)]
    pub insecure_ssl: String,
}

/// GitHub webhook response.
#[derive(Debug, Clone, Deserialize)]
pub struct Webhook {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
    pub config: WebhookConfig,
}

/// Request to create a webhook.
#[derive(Debug, Serialize)]
struct CreateWebhookRequest {
    name: String,
    active: bool,
    events: Vec<String>,
    config: WebhookConfig,
}

impl GitHubClient {
    /// Create a new GitHub client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("cto-integrations/1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            token: token.to_string(),
        })
    }

    /// List webhooks for a repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails.
    pub async fn list_webhooks(&self, owner: &str, repo: &str) -> Result<Vec<Webhook>> {
        let url = format!("{GITHUB_API_URL}/repos/{owner}/{repo}/hooks");

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error: {status} - {body}"));
        }

        response
            .json()
            .await
            .context("Failed to parse webhook list response")
    }

    /// Create a webhook for a repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails.
    pub async fn create_webhook(
        &self,
        owner: &str,
        repo: &str,
        webhook_url: &str,
        events: Vec<String>,
    ) -> Result<Webhook> {
        let url = format!("{GITHUB_API_URL}/repos/{owner}/{repo}/hooks");

        let request = CreateWebhookRequest {
            name: "web".to_string(),
            active: true,
            events,
            config: WebhookConfig {
                url: webhook_url.to_string(),
                content_type: "json".to_string(),
                insecure_ssl: "0".to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .json(&request)
            .send()
            .await
            .context("Failed to send create webhook request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error creating webhook: {status} - {body}"));
        }

        response
            .json()
            .await
            .context("Failed to parse create webhook response")
    }

    /// Update a webhook's configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails.
    pub async fn update_webhook(
        &self,
        owner: &str,
        repo: &str,
        hook_id: u64,
        webhook_url: &str,
        events: Vec<String>,
    ) -> Result<Webhook> {
        let url = format!("{GITHUB_API_URL}/repos/{owner}/{repo}/hooks/{hook_id}");

        let request = CreateWebhookRequest {
            name: "web".to_string(),
            active: true,
            events,
            config: WebhookConfig {
                url: webhook_url.to_string(),
                content_type: "json".to_string(),
                insecure_ssl: "0".to_string(),
            },
        };

        let response = self
            .client
            .patch(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .json(&request)
            .send()
            .await
            .context("Failed to send update webhook request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error updating webhook: {status} - {body}"));
        }

        response
            .json()
            .await
            .context("Failed to parse update webhook response")
    }

    /// Ensure a webhook exists for the repository pointing to the given URL.
    ///
    /// If a webhook with the same URL already exists, it will be updated to ensure
    /// it has the correct events. If no webhook exists, a new one will be created.
    ///
    /// # Errors
    ///
    /// Returns an error if the API calls fail.
    pub async fn ensure_webhook(
        &self,
        owner: &str,
        repo: &str,
        webhook_url: &str,
        events: Vec<String>,
    ) -> Result<Webhook> {
        debug!(
            owner = %owner,
            repo = %repo,
            webhook_url = %webhook_url,
            "Ensuring GitHub webhook exists"
        );

        // List existing webhooks
        let existing = self.list_webhooks(owner, repo).await?;

        // Check if a webhook with this URL already exists
        if let Some(hook) = existing.iter().find(|h| h.config.url == webhook_url) {
            // Check if events match
            let events_match = events.iter().all(|e| hook.events.contains(e));

            if events_match && hook.active {
                info!(
                    owner = %owner,
                    repo = %repo,
                    hook_id = hook.id,
                    "GitHub webhook already exists and is configured correctly"
                );
                return Ok(hook.clone());
            }

            // Update the webhook
            info!(
                owner = %owner,
                repo = %repo,
                hook_id = hook.id,
                "Updating existing GitHub webhook"
            );
            return self
                .update_webhook(owner, repo, hook.id, webhook_url, events)
                .await;
        }

        // Create new webhook
        info!(
            owner = %owner,
            repo = %repo,
            "Creating new GitHub webhook"
        );
        self.create_webhook(owner, repo, webhook_url, events).await
    }
}

/// Ensure GitHub webhooks are configured for the given repositories.
///
/// This should be called on service startup to ensure all configured
/// repositories have the correct webhook pointing to this service.
///
/// # Errors
///
/// Individual repository failures are logged but don't fail the entire operation.
pub async fn ensure_github_webhooks(
    token: &str,
    callback_url: &str,
    repos: &[String],
) -> Result<Vec<(String, bool)>> {
    if repos.is_empty() {
        debug!("No GitHub repos configured for webhook setup");
        return Ok(vec![]);
    }

    let client = GitHubClient::new(token)?;
    let webhook_url = format!("{callback_url}/webhooks/github");
    let events = vec!["pull_request".to_string()];

    let mut results = Vec::new();

    for repo in repos {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            warn!(repo = %repo, "Invalid repository format (expected owner/repo)");
            results.push((repo.clone(), false));
            continue;
        }

        let (owner, repo_name) = (parts[0], parts[1]);

        match client
            .ensure_webhook(owner, repo_name, &webhook_url, events.clone())
            .await
        {
            Ok(hook) => {
                info!(
                    repo = %repo,
                    hook_id = hook.id,
                    "GitHub webhook configured successfully"
                );
                results.push((repo.clone(), true));
            }
            Err(e) => {
                warn!(
                    repo = %repo,
                    error = %e,
                    "Failed to configure GitHub webhook"
                );
                results.push((repo.clone(), false));
            }
        }
    }

    let success_count = results.iter().filter(|(_, ok)| *ok).count();
    info!(
        total = repos.len(),
        success = success_count,
        "GitHub webhook initialization complete"
    );

    Ok(results)
}

