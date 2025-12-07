//! Cherry Servers API client implementation.
//!
//! API Documentation: <https://api.cherryservers.com/doc/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    CreateServerRequest as CherryCreateRequest, PowerActionRequest, ReinstallRequest,
    ServerResource,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for Cherry Servers API.
const API_BASE_URL: &str = "https://api.cherryservers.com/v1";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// Cherry Servers bare metal provider.
#[derive(Clone)]
pub struct Cherry {
    /// HTTP client.
    client: Client,
    /// API key for authentication.
    api_key: String,
    /// Team ID for server operations.
    team_id: i64,
}

impl Cherry {
    /// Create a new Cherry Servers provider.
    ///
    /// # Arguments
    /// * `api_key` - Cherry Servers API key
    /// * `team_id` - Team ID for server operations
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(api_key: impl Into<String>, team_id: i64) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            team_id,
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "GET request");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request (empty response)");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
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
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

    /// Convert Cherry server to our Server type.
    fn to_server(server: &ServerResource) -> Server {
        let status = match server.status.as_str() {
            "active" | "deployed" => ServerStatus::On,
            "pending" | "deploying" => ServerStatus::Deploying,
            "powering_off" | "powered_off" | "off" => ServerStatus::Off,
            "reinstalling" => ServerStatus::Reinstalling,
            "terminating" => ServerStatus::Deleting,
            _ => ServerStatus::Unknown,
        };

        let ipv4 = server
            .ip_addresses
            .iter()
            .find(|ip| ip.address_family == 4)
            .map(|ip| ip.address.clone());

        let ipv6 = server
            .ip_addresses
            .iter()
            .find(|ip| ip.address_family == 6)
            .map(|ip| ip.address.clone());

        let plan = server
            .plan
            .as_ref()
            .map(|p| p.slug.clone())
            .unwrap_or_default();

        let region = server
            .region
            .as_ref()
            .map(|r| r.slug.clone())
            .unwrap_or_default();

        Server {
            id: server.id.to_string(),
            hostname: server.hostname.clone(),
            status,
            ipv4,
            ipv6,
            plan,
            region,
            created_at: server
                .created_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}

#[async_trait]
impl Provider for Cherry {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating server"
        );

        let body = CherryCreateRequest {
            region: req.region,
            plan: req.plan,
            hostname: req.hostname,
            image: req.os,
            ssh_keys: vec![], // Would need to convert string IDs to i64
            user_data: None,
            tags: None,
        };

        let server: ServerResource = self
            .post(&format!("/teams/{}/servers", self.team_id), &body)
            .await?;

        info!(
            server_id = %server.id,
            "Server created"
        );

        Ok(Self::to_server(&server))
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let server: ServerResource = self.get(&format!("/servers/{id}")).await?;
        Ok(Self::to_server(&server))
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

        // Cherry supports user_data for custom boot scripts
        let reinstall_req = ReinstallRequest {
            image: "custom_ipxe".to_string(),
            hostname: req.hostname,
            ssh_keys: vec![],
            user_data: Some(format!("#!ipxe\nchain {}", req.ipxe_url)),
        };

        self.post_empty(&format!("/servers/{id}/reinstall"), &reinstall_req)
            .await?;

        // Reboot the server
        let power_req = PowerActionRequest {
            action_type: "reboot".to_string(),
        };
        self.post_empty(&format!("/servers/{id}/power"), &power_req)
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
        let servers: Vec<ServerResource> = self
            .get(&format!("/teams/{}/servers", self.team_id))
            .await?;
        Ok(servers.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_mapping() {
        let server = ServerResource {
            id: 12345,
            name: Some("test".to_string()),
            hostname: "test-server".to_string(),
            status: "active".to_string(),
            ip_addresses: vec![super::super::models::IpAddress {
                address: "1.2.3.4".to_string(),
                address_type: Some("primary".to_string()),
                address_family: 4,
            }],
            region: Some(super::super::models::Region {
                id: 1,
                name: "EU-Nord-1".to_string(),
                slug: "eu_nord_1".to_string(),
            }),
            plan: Some(super::super::models::Plan {
                id: 1,
                name: "E3-1240v3".to_string(),
                slug: "e3_1240v3".to_string(),
                specs: None,
            }),
            project_id: Some(1),
            created_at: None,
        };

        let converted = Cherry::to_server(&server);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "12345");
        assert_eq!(converted.ipv4, Some("1.2.3.4".to_string()));
    }
}
