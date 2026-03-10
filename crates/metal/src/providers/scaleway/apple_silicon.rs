//! Scaleway Apple Silicon API client implementation.
//!
//! API Documentation: <https://www.scaleway.com/en/developers/api/apple-silicon/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info};

use super::models::{
    AppleSiliconOffer, AppleSiliconServer, AppleSiliconServerListResponse,
    CreateAppleSiliconRequest, RemoteDesktop,
};
use crate::providers::traits::{Provider, ProviderError, Server, ServerStatus};

/// Base URL for Apple Silicon API.
const API_BASE_URL: &str = "https://api.scaleway.com/apple-silicon/v1alpha1";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// Scaleway Apple Silicon provider (Mac mini M2/M2 Pro).
#[derive(Clone)]
pub struct AppleSilicon {
    /// HTTP client.
    client: Client,
    /// Secret key for authentication.
    secret_key: String,
    /// Project ID.
    project_id: String,
    /// Region (e.g., "fr-par-1", "nl-ams-1").
    region: String,
}

impl AppleSilicon {
    /// Create a new Apple Silicon provider.
    ///
    /// # Arguments
    /// * `secret_key` - Scaleway Secret Key
    /// * `project_id` - Project ID
    /// * `region` - Region (e.g., "fr-par-1")
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        secret_key: impl Into<String>,
        project_id: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            secret_key: secret_key.into(),
            project_id: project_id.into(),
            region: region.into(),
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        let url = format!("{API_BASE_URL}/regions/{}{path}", self.region);
        debug!(url = %url, "GET request");

        let response = self
            .client
            .get(&url)
            .header("X-Auth-Token", &self.secret_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request.
    async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ProviderError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{API_BASE_URL}/regions/{}{path}", self.region);
        debug!(url = %url, "POST request");

        let response = self
            .client
            .post(&url)
            .header("X-Auth-Token", &self.secret_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated DELETE request.
    async fn delete(&self, path: &str) -> Result<(), ProviderError> {
        let url = format!("{API_BASE_URL}/regions/{}{path}", self.region);
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_key)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(ProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Handle API response, parsing JSON or error.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ProviderError> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&text).map_err(|e| {
                tracing::warn!(error = %e, body = %text, "Failed to parse response");
                ProviderError::Serialization(e)
            })
        } else if status == StatusCode::NOT_FOUND {
            Err(ProviderError::NotFound(text))
        } else {
            Err(ProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Convert Apple Silicon server to our Server type.
    fn to_server(server: &AppleSiliconServer) -> Server {
        let status = match server.status.as_str() {
            "ready" => ServerStatus::On,
            "delivering" | "queued" => ServerStatus::Deploying,
            "stopped" => ServerStatus::Off,
            "rebooting" | "reinstalling" => ServerStatus::Reinstalling,
            "deleting" => ServerStatus::Deleting,
            _ => ServerStatus::Unknown,
        };

        let plan = server
            .offer
            .as_ref()
            .map(|o| o.name.clone())
            .unwrap_or_default();

        Server {
            id: server.id.clone(),
            hostname: server.name.clone(),
            status,
            ipv4: server.ip.clone(),
            ipv6: None,
            plan,
            region: server.region.clone(),
            created_at: server
                .created_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }

    /// Get remote desktop URL for VNC access.
    pub async fn get_remote_desktop(&self, id: &str) -> Result<RemoteDesktop, ProviderError> {
        self.get(&format!("/servers/{id}/remote-desktop")).await
    }

    /// Create an Apple Silicon server.
    pub async fn create_server(&self, offer_id: &str, name: &str) -> Result<Server, ProviderError> {
        info!(offer_id, name, region = %self.region, "Creating Apple Silicon server");

        let body = CreateAppleSiliconRequest {
            offer_id: offer_id.to_string(),
            name: name.to_string(),
            project_id: self.project_id.clone(),
            region: self.region.clone(),
        };

        let server: AppleSiliconServer = self.post("/servers", &body).await?;

        info!(server_id = %server.id, "Apple Silicon server created");

        Ok(Self::to_server(&server))
    }

    /// Get an Apple Silicon server.
    pub async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let server: AppleSiliconServer = self.get(&format!("/servers/{id}")).await?;
        Ok(Self::to_server(&server))
    }

    /// List all Apple Silicon servers.
    pub async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let response: AppleSiliconServerListResponse = self.get("/servers").await?;
        Ok(response.servers.iter().map(Self::to_server).collect())
    }

    /// Delete an Apple Silicon server.
    pub async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Deleting Apple Silicon server");
        self.delete(&format!("/servers/{id}")).await?;
        info!(server_id = %id, "Server deleted");
        Ok(())
    }

    /// Wait for server to be ready.
    pub async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError> {
        info!(server_id = %id, timeout_secs, "Waiting for server to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let server = self.get_server(id).await?;

            debug!(
                server_id = %id,
                status = %server.status,
                elapsed_secs = start.elapsed().as_secs(),
                "Polling server status"
            );

            if server.status == ServerStatus::On {
                info!(server_id = %id, "Server is ready");
                return Ok(server);
            }

            if start.elapsed() > timeout {
                return Err(ProviderError::Timeout(timeout_secs));
            }

            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_silicon_status_mapping() {
        let server = AppleSiliconServer {
            id: "mac-12345".to_string(),
            name: "mac-mini-test".to_string(),
            organization_id: "org-123".to_string(),
            project_id: "proj-123".to_string(),
            region: "fr-par-1".to_string(),
            status: "ready".to_string(),
            offer: Some(AppleSiliconOffer {
                id: "apple-m2-mini".to_string(),
                name: "Mac mini M2".to_string(),
            }),
            ip: Some("10.0.0.1".to_string()),
            remote_desktop: None,
            created_at: None,
            updated_at: None,
        };

        let converted = AppleSilicon::to_server(&server);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "mac-12345");
        assert_eq!(converted.ipv4, Some("10.0.0.1".to_string()));
        assert_eq!(converted.hostname, "mac-mini-test");
    }
}
