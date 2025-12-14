//! Scaleway Elastic Metal API client implementation.
//!
//! API Documentation: <https://www.scaleway.com/en/developers/api/elastic-metal/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    ActionRequest, CreateServerRequest as ScalewayCreateRequest, InstallRequest,
    ReinstallRequest as ScalewayReinstallRequest, Server as ScalewayServer, ServerListResponse,
    ServerResponse,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for Scaleway API.
const API_BASE_URL: &str = "https://api.scaleway.com/baremetal/v1";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// Scaleway Elastic Metal provider.
#[derive(Clone)]
pub struct Scaleway {
    /// HTTP client.
    client: Client,
    /// Secret key for authentication.
    secret_key: String,
    /// Organization ID.
    #[allow(dead_code)]
    organization_id: String,
    /// Project ID.
    project_id: String,
    /// Zone (e.g., "fr-par-1", "nl-ams-1").
    zone: String,
}

impl Scaleway {
    /// Create a new Scaleway provider.
    ///
    /// # Arguments
    /// * `secret_key` - Scaleway Secret Key
    /// * `organization_id` - Organization ID
    /// * `project_id` - Project ID
    /// * `zone` - Zone (e.g., "fr-par-1")
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        secret_key: impl Into<String>,
        organization_id: impl Into<String>,
        project_id: impl Into<String>,
        zone: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            secret_key: secret_key.into(),
            organization_id: organization_id.into(),
            project_id: project_id.into(),
            zone: zone.into(),
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        let url = format!("{API_BASE_URL}/zones/{}{path}", self.zone);
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
        let url = format!("{API_BASE_URL}/zones/{}{path}", self.zone);
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

    /// Make an authenticated POST request that returns empty body.
    async fn post_empty<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ProviderError> {
        let url = format!("{API_BASE_URL}/zones/{}{path}", self.zone);
        debug!(url = %url, "POST request (empty response)");

        let response = self
            .client
            .post(&url)
            .header("X-Auth-Token", &self.secret_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(ProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Make an authenticated DELETE request.
    async fn delete(&self, path: &str) -> Result<(), ProviderError> {
        let url = format!("{API_BASE_URL}/zones/{}{path}", self.zone);
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_key)
            .send()
            .await?;

        let status = response.status();
        if status.is_success()
            || status == StatusCode::NO_CONTENT
            || status == StatusCode::NOT_FOUND
        {
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
                warn!(error = %e, body = %text, "Failed to parse response");
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

    /// Convert Scaleway server to our Server type.
    fn to_server(server: &ScalewayServer) -> Server {
        let status = match server.status.as_str() {
            "ready" => ServerStatus::On,
            "delivering" | "ordered" => ServerStatus::Deploying,
            "stopped" => ServerStatus::Off,
            "resetting" | "installing" => ServerStatus::Reinstalling,
            "deleting" => ServerStatus::Deleting,
            _ => ServerStatus::Unknown,
        };

        let ipv4 = server
            .ips
            .iter()
            .find(|ip| ip.version == "IPv4")
            .map(|ip| ip.address.clone());

        let ipv6 = server
            .ips
            .iter()
            .find(|ip| ip.version == "IPv6")
            .map(|ip| ip.address.clone());

        let plan = server
            .offer
            .as_ref()
            .map(|o| o.name.clone())
            .unwrap_or_default();

        Server {
            id: server.id.clone(),
            hostname: server.name.clone(),
            status,
            ipv4,
            ipv6,
            plan,
            region: server.zone.clone(),
            created_at: server
                .created_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}

#[async_trait]
impl Provider for Scaleway {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating server"
        );

        let body = ScalewayCreateRequest {
            offer_id: req.plan,
            name: req.hostname.clone(),
            project_id: self.project_id.clone(),
            description: None,
            tags: vec![],
            install: Some(InstallRequest {
                os_id: req.os,
                hostname: req.hostname,
                ssh_key_ids: req.ssh_keys,
                user: None,
                password: None,
                service_user: None,
                service_password: None,
            }),
        };

        let response: ServerResponse = self.post("/servers", &body).await?;

        info!(
            server_id = %response.server.id,
            "Server created"
        );

        Ok(Self::to_server(&response.server))
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let response: ServerResponse = self.get(&format!("/servers/{id}")).await?;
        Ok(Self::to_server(&response.server))
    }

    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError> {
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

    async fn reinstall_ipxe(
        &self,
        id: &str,
        req: ReinstallIpxeRequest,
    ) -> Result<(), ProviderError> {
        info!(
            server_id = %id,
            ipxe_url = %req.ipxe_url,
            hostname = %req.hostname,
            "Triggering reinstall"
        );

        // Scaleway supports custom installation via iPXE
        // First, install with a minimal OS, then configure iPXE boot
        let reinstall_req = ScalewayReinstallRequest {
            os_id: "ipxe".to_string(), // Use iPXE boot
            hostname: req.hostname,
            ssh_key_ids: vec![],
        };

        self.post_empty(&format!("/servers/{id}/install"), &reinstall_req)
            .await?;

        // Reboot the server
        let action_req = ActionRequest {
            action: "reboot".to_string(),
        };
        self.post_empty(&format!("/servers/{id}/actions"), &action_req)
            .await?;

        info!(server_id = %id, "Reinstall triggered");
        Ok(())
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Deleting server");
        self.delete(&format!("/servers/{id}")).await?;
        info!(server_id = %id, "Server deleted");
        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let response: ServerListResponse = self.get("/servers").await?;
        Ok(response.servers.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_mapping() {
        let server = ScalewayServer {
            id: "12345".to_string(),
            name: "test-server".to_string(),
            organization_id: "org-123".to_string(),
            project_id: "proj-123".to_string(),
            zone: "fr-par-1".to_string(),
            status: "ready".to_string(),
            offer: Some(super::super::models::Offer {
                id: "offer-1".to_string(),
                name: "PRO-6-S-SSD".to_string(),
                cpu: None,
                memory: None,
                disk: None,
                bandwidth: None,
            }),
            ips: vec![super::super::models::Ip {
                id: "ip-1".to_string(),
                address: "1.2.3.4".to_string(),
                reverse: None,
                version: "IPv4".to_string(),
            }],
            install: None,
            tags: vec![],
            created_at: None,
            updated_at: None,
        };

        let converted = Scaleway::to_server(&server);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "12345");
        assert_eq!(converted.ipv4, Some("1.2.3.4".to_string()));
    }
}
