//! Latitude.sh API client implementation.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    ApiResponse, CreateServerAttributes, CreateServerBody, CreateServerData,
    ReinstallServerAttributes, ReinstallServerBody, ReinstallServerData, ServerResource,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for Latitude.sh API.
const API_BASE_URL: &str = "https://api.latitude.sh";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 10;

/// Latitude.sh bare metal provider.
#[derive(Clone)]
pub struct Latitude {
    /// HTTP client.
    client: Client,
    /// API key for authentication.
    api_key: String,
    /// Project ID for server operations.
    project_id: String,
}

impl Latitude {
    /// Create a new Latitude provider.
    ///
    /// # Arguments
    /// * `api_key` - Latitude.sh API key
    /// * `project_id` - Project ID for server operations
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        api_key: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            project_id: project_id.into(),
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

    /// Convert API server resource to our Server type.
    fn to_server(resource: &ServerResource) -> Server {
        let status = match resource.attributes.status.as_str() {
            "deploying" => ServerStatus::Deploying,
            "on" => ServerStatus::On,
            "off" => ServerStatus::Off,
            "disk_erasing" => ServerStatus::DiskErasing,
            "reinstalling" => ServerStatus::Reinstalling,
            "deleting" => ServerStatus::Deleting,
            _ => ServerStatus::Unknown,
        };

        let plan = resource
            .attributes
            .plan
            .as_ref()
            .and_then(|p| p.slug.clone())
            .unwrap_or_default();

        Server {
            id: resource.id.clone(),
            hostname: resource.attributes.hostname.clone(),
            status,
            ipv4: resource.attributes.primary_ipv4.clone(),
            ipv6: resource.attributes.primary_ipv6.clone(),
            plan,
            region: String::new(), // Not directly in API response
            created_at: resource
                .attributes
                .created_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}

#[async_trait]
impl Provider for Latitude {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating server"
        );

        let body = CreateServerBody {
            data: CreateServerData {
                resource_type: "servers".to_string(),
                attributes: CreateServerAttributes {
                    project: self.project_id.clone(),
                    plan: req.plan,
                    site: req.region,
                    operating_system: req.os,
                    hostname: req.hostname,
                    ssh_keys: req.ssh_keys,
                },
            },
        };

        let response: ApiResponse<ServerResource> = self.post("/servers", &body).await?;
        let server = Self::to_server(&response.data);

        info!(
            server_id = %server.id,
            ipv4 = ?server.ipv4,
            "Server created"
        );

        Ok(server)
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let response: ApiResponse<ServerResource> = self.get(&format!("/servers/{id}")).await?;
        Ok(Self::to_server(&response.data))
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
            "Triggering iPXE reinstall"
        );

        let body = ReinstallServerBody {
            data: ReinstallServerData {
                resource_type: "reinstalls".to_string(),
                attributes: ReinstallServerAttributes {
                    operating_system: "ipxe".to_string(),
                    hostname: req.hostname,
                    ipxe: Some(req.ipxe_url),
                },
            },
        };

        self.post_empty(&format!("/servers/{id}/reinstall"), &body)
            .await?;

        info!(server_id = %id, "iPXE reinstall triggered");
        Ok(())
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Deleting server");
        self.delete(&format!("/servers/{id}")).await?;
        info!(server_id = %id, "Server deleted");
        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let response: ApiResponse<Vec<ServerResource>> = self.get("/servers").await?;
        Ok(response.data.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_display() {
        assert_eq!(ServerStatus::On.to_string(), "on");
        assert_eq!(ServerStatus::Deploying.to_string(), "deploying");
        assert_eq!(ServerStatus::DiskErasing.to_string(), "disk_erasing");
    }
}
