//! `DigitalOcean` API client implementation.
//!
//! API Documentation: <https://docs.digitalocean.com/reference/api/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    CreateDropletRequest, Droplet, DropletListResponse, DropletResponse, ImageIdentifier,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for `DigitalOcean` API.
const API_BASE_URL: &str = "https://api.digitalocean.com/v2";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 10;

/// `DigitalOcean` provider.
#[derive(Clone)]
pub struct DigitalOcean {
    /// HTTP client.
    client: Client,
    /// API token for authentication.
    api_token: String,
}

impl DigitalOcean {
    /// Create a new `DigitalOcean` provider.
    ///
    /// # Arguments
    /// * `api_token` - `DigitalOcean` API token
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(api_token: impl Into<String>) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            api_token: api_token.into(),
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "GET request");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
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
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request that returns empty body.
    #[allow(dead_code)]
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
            .header("Authorization", format!("Bearer {}", self.api_token))
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
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
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

    /// Convert `DigitalOcean` droplet to our Server type.
    fn to_server(droplet: &Droplet) -> Server {
        let status = match droplet.status.as_str() {
            "active" => ServerStatus::On,
            "off" => ServerStatus::Off,
            "new" => ServerStatus::Deploying,
            "archive" => ServerStatus::Deleting,
            _ => ServerStatus::Unknown,
        };

        let ipv4 = droplet
            .networks
            .v4
            .iter()
            .find(|ip| ip.address_type == "public")
            .map(|ip| ip.ip_address.clone());

        let ipv6 = droplet
            .networks
            .v6
            .iter()
            .find(|ip| ip.address_type == "public")
            .map(|ip| ip.ip_address.clone());

        Server {
            id: droplet.id.to_string(),
            hostname: droplet.name.clone(),
            status,
            ipv4,
            ipv6,
            plan: droplet.size_slug.clone(),
            region: droplet.region.slug.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&droplet.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}

#[async_trait]
impl Provider for DigitalOcean {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating droplet"
        );

        let body = CreateDropletRequest {
            name: req.hostname,
            region: req.region,
            size: req.plan,
            image: ImageIdentifier::Slug(req.os),
            ssh_keys: req.ssh_keys,
            ipv6: Some(true),
            monitoring: Some(true),
            user_data: None,
            tags: vec![],
            vpc_uuid: None,
        };

        let response: DropletResponse = self.post("/droplets", &body).await?;

        info!(
            droplet_id = %response.droplet.id,
            "Droplet created"
        );

        Ok(Self::to_server(&response.droplet))
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let response: DropletResponse = self.get(&format!("/droplets/{id}")).await?;
        Ok(Self::to_server(&response.droplet))
    }

    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError> {
        info!(server_id = %id, timeout_secs, "Waiting for droplet to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let server = self.get_server(id).await?;

            debug!(
                server_id = %id,
                status = %server.status,
                elapsed_secs = start.elapsed().as_secs(),
                "Polling droplet status"
            );

            if server.status == ServerStatus::On {
                info!(server_id = %id, "Droplet is ready");
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
            "DigitalOcean does not support iPXE boot directly"
        );

        // DigitalOcean droplets don't support iPXE boot directly
        // You'd need to use a custom image or user-data for bootstrapping
        Err(ProviderError::Config(
            "DigitalOcean droplets do not support direct iPXE boot. \
            Use a custom image or cloud-init user-data for bootstrapping."
                .to_string(),
        ))
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Deleting droplet");
        self.delete(&format!("/droplets/{id}")).await?;
        info!(server_id = %id, "Droplet deleted");
        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let response: DropletListResponse = self.get("/droplets").await?;
        Ok(response.droplets.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_mapping() {
        let droplet = Droplet {
            id: 12345,
            name: "test-server".to_string(),
            memory: 8192,
            vcpus: 4,
            disk: 160,
            status: "active".to_string(),
            region: super::super::models::Region {
                slug: "nyc1".to_string(),
                name: "New York 1".to_string(),
                features: vec![],
                available: true,
            },
            size_slug: "c-4".to_string(),
            networks: super::super::models::Networks {
                v4: vec![super::super::models::NetworkAddress {
                    ip_address: "1.2.3.4".to_string(),
                    netmask: Some("255.255.255.0".to_string()),
                    gateway: Some("1.2.3.1".to_string()),
                    address_type: "public".to_string(),
                }],
                v6: vec![],
            },
            image: super::super::models::Image {
                id: 123,
                name: "Ubuntu 22.04".to_string(),
                slug: Some("ubuntu-22-04-x64".to_string()),
                distribution: "Ubuntu".to_string(),
                public: true,
            },
            tags: vec![],
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let converted = DigitalOcean::to_server(&droplet);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "12345");
        assert_eq!(converted.ipv4, Some("1.2.3.4".to_string()));
    }
}
