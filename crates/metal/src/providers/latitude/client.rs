//! Latitude.sh API client implementation.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    ApiResponse, AssignVirtualNetworkAttributes, AssignVirtualNetworkBody,
    AssignVirtualNetworkData, CreateServerAttributes, CreateServerBody, CreateServerData,
    CreateVirtualNetworkAttributes, CreateVirtualNetworkBody, CreateVirtualNetworkData,
    PlanResource, RegionResource, ReinstallServerAttributes, ReinstallServerBody,
    ReinstallServerData, ServerResource, VirtualNetworkAssignmentResource, VirtualNetworkResource,
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

/// Buffer delay after server reaches "on" status before attempting operations.
/// This gives the Latitude API time to fully register the server as ready.
const POST_READY_BUFFER_SECS: u64 = 15;

/// Maximum retries for reinstall when server reports "still provisioning".
const REINSTALL_MAX_RETRIES: u32 = 6;

/// Delay between reinstall retries.
const REINSTALL_RETRY_DELAY_SECS: u64 = 30;

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
                "Polling server status via API"
            );

            if server.status == ServerStatus::On {
                info!(
                    server_id = %id,
                    buffer_secs = POST_READY_BUFFER_SECS,
                    "Server status is 'on', waiting buffer period before proceeding"
                );
                // Wait a buffer period after status becomes "on" to ensure
                // the Latitude API has fully registered the server as ready
                // for subsequent operations like reinstall.
                tokio::time::sleep(Duration::from_secs(POST_READY_BUFFER_SECS)).await;
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
                    hostname: req.hostname.clone(),
                    ipxe: Some(req.ipxe_url.clone()),
                },
            },
        };

        // Retry loop for "SERVER_BEING_PROVISIONED" errors
        // The Latitude API may still report provisioning even after status is "on"
        for attempt in 1..=REINSTALL_MAX_RETRIES {
            match self
                .post_empty(&format!("/servers/{id}/reinstall"), &body)
                .await
            {
                Ok(()) => {
                    info!(server_id = %id, "iPXE reinstall triggered");
                    return Ok(());
                }
                Err(ProviderError::Api { status, message })
                    if message.contains("SERVER_BEING_PROVISIONED")
                        || message.contains("being provisioned") =>
                {
                    if attempt < REINSTALL_MAX_RETRIES {
                        warn!(
                            server_id = %id,
                            attempt,
                            max_retries = REINSTALL_MAX_RETRIES,
                            retry_delay_secs = REINSTALL_RETRY_DELAY_SECS,
                            "Server still provisioning, retrying reinstall"
                        );
                        tokio::time::sleep(Duration::from_secs(REINSTALL_RETRY_DELAY_SECS)).await;
                    } else {
                        return Err(ProviderError::Api { status, message });
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Should not reach here, but handle it gracefully
        Err(ProviderError::Api {
            status: 422,
            message: "Reinstall failed after max retries".to_string(),
        })
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

impl Latitude {
    /// List all available plans.
    ///
    /// Returns plans with their specs, pricing, and stock availability by region.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn list_plans(&self) -> Result<Vec<PlanResource>, ProviderError> {
        let response: ApiResponse<Vec<PlanResource>> = self.get("/plans").await?;
        Ok(response.data)
    }

    /// List all available regions.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn list_regions(&self) -> Result<Vec<RegionResource>, ProviderError> {
        let response: ApiResponse<Vec<RegionResource>> = self.get("/regions").await?;
        Ok(response.data)
    }

    // ========================================================================
    // Virtual Network (VLAN) operations
    // ========================================================================

    /// Create a Virtual Network (VLAN) in a specific site.
    ///
    /// VLANs provide private networking between servers in the same location.
    /// This is essential for secure cluster-internal communication.
    ///
    /// # Arguments
    ///
    /// * `site` - Site slug (e.g., "MIA2")
    /// * `description` - Description of the VLAN
    ///
    /// # Returns
    ///
    /// The created Virtual Network resource with its VLAN ID.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn create_virtual_network(
        &self,
        site: &str,
        description: &str,
    ) -> Result<VirtualNetworkResource, ProviderError> {
        info!(site = %site, description = %description, "Creating Virtual Network");

        let body = CreateVirtualNetworkBody {
            data: CreateVirtualNetworkData {
                resource_type: "virtual_networks".to_string(),
                attributes: CreateVirtualNetworkAttributes {
                    description: description.to_string(),
                    project: self.project_id.clone(),
                    site: site.to_string(),
                },
            },
        };

        let response: ApiResponse<VirtualNetworkResource> =
            self.post("/virtual_networks", &body).await?;

        info!(
            vlan_id = %response.data.id,
            vid = %response.data.attributes.vid,
            "Virtual Network created"
        );

        Ok(response.data)
    }

    /// Get a Virtual Network by ID.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn get_virtual_network(
        &self,
        vlan_id: &str,
    ) -> Result<VirtualNetworkResource, ProviderError> {
        let response: ApiResponse<VirtualNetworkResource> =
            self.get(&format!("/virtual_networks/{vlan_id}")).await?;
        Ok(response.data)
    }

    /// List all Virtual Networks in the project.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn list_virtual_networks(&self) -> Result<Vec<VirtualNetworkResource>, ProviderError> {
        let response: ApiResponse<Vec<VirtualNetworkResource>> =
            self.get("/virtual_networks").await?;
        Ok(response.data)
    }

    /// Assign a server to a Virtual Network.
    ///
    /// This attaches the server's internal NIC to the VLAN and assigns a
    /// private IP address. The server will then be able to communicate with
    /// other servers on the same VLAN using their private IPs.
    ///
    /// # Arguments
    ///
    /// * `vlan_id` - Virtual Network ID (e.g., "vlan_xxx")
    /// * `server_id` - Server ID to assign
    ///
    /// # Returns
    ///
    /// The assignment resource with the private IP.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn assign_server_to_vlan(
        &self,
        vlan_id: &str,
        server_id: &str,
    ) -> Result<VirtualNetworkAssignmentResource, ProviderError> {
        info!(vlan_id = %vlan_id, server_id = %server_id, "Assigning server to VLAN");

        let body = AssignVirtualNetworkBody {
            data: AssignVirtualNetworkData {
                resource_type: "virtual_network_assignment".to_string(),
                attributes: AssignVirtualNetworkAttributes {
                    server_id: server_id.to_string(),
                },
            },
        };

        let response: ApiResponse<VirtualNetworkAssignmentResource> = self
            .post(&format!("/virtual_networks/{vlan_id}/assignments"), &body)
            .await?;

        info!(
            assignment_id = %response.data.id,
            private_ip = ?response.data.attributes.ip,
            "Server assigned to VLAN"
        );

        Ok(response.data)
    }

    /// List all server assignments for a Virtual Network.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn list_vlan_assignments(
        &self,
        vlan_id: &str,
    ) -> Result<Vec<VirtualNetworkAssignmentResource>, ProviderError> {
        let response: ApiResponse<Vec<VirtualNetworkAssignmentResource>> = self
            .get(&format!("/virtual_networks/{vlan_id}/assignments"))
            .await?;
        Ok(response.data)
    }

    /// Delete a Virtual Network.
    ///
    /// Note: All server assignments must be removed first.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn delete_virtual_network(&self, vlan_id: &str) -> Result<(), ProviderError> {
        info!(vlan_id = %vlan_id, "Deleting Virtual Network");
        self.delete(&format!("/virtual_networks/{vlan_id}")).await?;
        info!(vlan_id = %vlan_id, "Virtual Network deleted");
        Ok(())
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
