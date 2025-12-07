//! Vultr API client implementation.
//!
//! API Documentation: <https://www.vultr.com/api/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    BareMetalInstance, BareMetalListResponse, BareMetalResponse,
    CreateBareMetalRequest as VultrCreateRequest, IpxeChainRequest, RebootRequest,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for Vultr API.
const API_BASE_URL: &str = "https://api.vultr.com/v2";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// Vultr bare metal provider.
#[derive(Clone)]
pub struct Vultr {
    /// HTTP client.
    client: Client,
    /// API key for authentication.
    api_key: String,
}

impl Vultr {
    /// Create a new Vultr provider.
    ///
    /// # Arguments
    /// * `api_key` - Vultr API key
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.into(),
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
            .header("Authorization", format!("Bearer {}", self.api_key))
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

    /// Convert Vultr instance to our Server type.
    fn to_server(instance: &BareMetalInstance) -> Server {
        let status = match (instance.status.as_str(), instance.power_status.as_str()) {
            ("active", "running") => ServerStatus::On,
            ("active", "stopped") => ServerStatus::Off,
            ("pending", _) => ServerStatus::Deploying,
            ("resizing", _) => ServerStatus::Reinstalling,
            _ => ServerStatus::Unknown,
        };

        Server {
            id: instance.id.clone(),
            hostname: instance.label.clone(),
            status,
            ipv4: Some(instance.main_ip.clone()),
            ipv6: instance.v6_network.clone(),
            plan: instance.plan.clone(),
            region: instance.region.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&instance.date_created)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }

    /// Map OS slug to Vultr OS ID.
    fn os_slug_to_id(slug: &str) -> Option<i32> {
        // Common OS mappings - in production, query /os endpoint
        match slug {
            "ubuntu_24_04" | "ubuntu_24_04_x64_lts" => Some(2284),
            "ubuntu_22_04" | "ubuntu_22_04_x64_lts" => Some(1743),
            "debian_12" | "debian_12_x64" => Some(2136),
            "rocky_9" | "rocky_9_x64" => Some(1869),
            // "ipxe" and other slugs use None for iPXE chain
            _ => None,
        }
    }
}

#[async_trait]
impl Provider for Vultr {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating server"
        );

        let body = VultrCreateRequest {
            region: req.region,
            plan: req.plan,
            os_id: Self::os_slug_to_id(&req.os),
            image_id: None,
            label: Some(req.hostname),
            sshkey_id: req.ssh_keys,
            enable_ipv6: Some(true),
            user_data: None,
            tags: vec![],
            script_id: None,
            app_id: None,
            hostname: None,
        };

        let response: BareMetalResponse = self.post("/bare-metals", &body).await?;

        info!(
            server_id = %response.bare_metal.id,
            ipv4 = %response.bare_metal.main_ip,
            "Server created"
        );

        Ok(Self::to_server(&response.bare_metal))
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let response: BareMetalResponse = self.get(&format!("/bare-metals/{id}")).await?;
        Ok(Self::to_server(&response.bare_metal))
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
            "Triggering iPXE reinstall"
        );

        // Set iPXE chain URL
        let ipxe_req = IpxeChainRequest {
            chain_url: req.ipxe_url,
        };
        self.post_empty(&format!("/bare-metals/{id}/ipxe"), &ipxe_req)
            .await?;

        // Reboot the server to boot via iPXE
        let reboot_req = RebootRequest {};
        self.post_empty(&format!("/bare-metals/{id}/reboot"), &reboot_req)
            .await?;

        info!(server_id = %id, "iPXE reinstall triggered");
        Ok(())
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Deleting server");
        self.delete(&format!("/bare-metals/{id}")).await?;
        info!(server_id = %id, "Server deleted");
        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let response: BareMetalListResponse = self.get("/bare-metals").await?;
        Ok(response.bare_metals.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_mapping() {
        let instance = BareMetalInstance {
            id: "12345".to_string(),
            label: "test-server".to_string(),
            main_ip: "1.2.3.4".to_string(),
            v6_network: Some("2001:db8::/64".to_string()),
            cpu_count: 8,
            ram: "32768".to_string(),
            disk: "480 GB SSD".to_string(),
            region: "ewr".to_string(),
            plan: "vbm-4c-32gb".to_string(),
            status: "active".to_string(),
            power_status: "running".to_string(),
            server_state: "ok".to_string(),
            os: "Ubuntu 22.04 x64".to_string(),
            date_created: "2024-01-01T00:00:00+00:00".to_string(),
            netmask_v4: Some("255.255.255.0".to_string()),
            gateway_v4: Some("1.2.3.1".to_string()),
            mac_address: Some("00:00:00:00:00:00".to_string()),
            os_id: 1743,
            app_id: 0,
            image_id: None,
            tags: vec![],
        };

        let converted = Vultr::to_server(&instance);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "12345");
        assert_eq!(converted.ipv4, Some("1.2.3.4".to_string()));
    }
}
