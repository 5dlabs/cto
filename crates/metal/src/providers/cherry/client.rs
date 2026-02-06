//! Cherry Servers API client implementation.
//!
//! API Documentation: <https://api.cherryservers.com/doc/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use crate::providers::cherry::models::{
    AssignIpAddressRequest, CreateIpAddressRequest, CreateServerRequest as CherryCreateRequest,
    CreateSshKey, IpAddressResource, PowerActionRequest, Project,
    ReinstallRequest, ServerResource, SshKey,
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

    /// Create a new Cherry Servers provider with frictionless initialization.
    ///
    /// This method:
    /// 1. Auto-detects existing SSH keys from ~/.ssh/
    /// 2. Uploads them to Cherry if not already present
    /// 3. Creates a default project if none exists
    /// 4. Returns the project ID and SSH key IDs ready for server creation
    ///
    /// # Arguments
    /// * `api_key` - Cherry Servers API key
    /// * `team_id` - Team ID for server operations
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created or API calls fail.
    pub async fn with_frictionless_init(
        api_key: impl Into<String>,
        team_id: i64,
    ) -> Result<(Self, i64, Vec<i64>), ProviderError> {
        let provider = Self::new(api_key, team_id)?;
        let (project_id, ssh_key_ids) = provider.frictionless_init().await?;
        Ok((provider, project_id, ssh_key_ids))
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

    /// Create an SSH key.
    async fn create_ssh_key(
        &self,
        label: &str,
        public_key: &str,
    ) -> Result<SshKey, ProviderError> {
        info!(label = %label, "Creating SSH key");

        let body = CreateSshKey {
            label: label.to_string(),
            key: public_key.to_string(),
        };

        let key: SshKey = self
            .post(&format!("/teams/{}/ssh-keys", self.team_id), &body)
            .await?;

        info!(key_id = %key.id, "SSH key created");
        Ok(key)
    }

    /// Create a private IP address in the project.
    async fn _create_private_ip(
        &self,
        project_id: i64,
        region: &str,
    ) -> Result<IpAddressResource, ProviderError> {
        info!(region = %region, project_id = %project_id, "Creating private IP");

        let body = CreateIpAddressRequest {
            region: region.to_string(),
            address_type: "private".to_string(),
            ptr_record: None,
        };

        let ip: IpAddressResource = self
            .post(&format!("/projects/{}/ips", project_id), &body)
            .await?;

        info!(ip_id = %ip.id, address = %ip.address, "Private IP created");
        Ok(ip)
    }

    /// Assign an IP address to a server.
    async fn _assign_ip_to_server(
        &self,
        ip_id: &str,
        server_id: i64,
    ) -> Result<IpAddressResource, ProviderError> {
        info!(ip_id = %ip_id, server_id = %server_id, "Assigning IP to server");

        let body = AssignIpAddressRequest {
            server_id,
        };

        let ip: IpAddressResource = self
            .post(&format!("/ips/{}/assign", ip_id), &body)
            .await?;

        info!(ip_id = %ip_id, "IP assigned to server");
        Ok(ip)
    }

    /// Get project ID from team.
    async fn _get_default_project(&self) -> Result<i64, ProviderError> {
        let projects: Vec<Project> = self.get(&format!("/teams/{}/projects", self.team_id)).await?;

        // Return the first project or error if none exist
        projects
            .first()
            .map(|p| p.id)
            .ok_or_else(|| ProviderError::Api {
                status: 404,
                message: "No projects found in team".to_string(),
            })
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
            ip_addresses: req.ip_addresses,
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

impl Cherry {
    // =========================================================================
    // Inventory / Plan Listing Methods
    // =========================================================================

    /// List all available plans with pricing.
    ///
    /// Returns plans with their specs, pricing, and NIC/bandwidth information.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn list_plans(&self) -> Result<Vec<crate::providers::cherry::models::PlanWithPricing>, ProviderError> {
        info!("Fetching available plans from Cherry API");

        // Cherry API returns plans as a plain array
        let plans: Vec<crate::providers::cherry::models::PlanWithPricing> = self.get("/plans").await?;

        // Parse pricing from the API response if present
        let plans_with_pricing: Vec<crate::providers::cherry::models::PlanWithPricing> = plans
            .into_iter()
            .map(|mut plan| {
                // Pricing is embedded in the response, extract hourly/monthly
                // The API returns pricing as an array in the response
                plan.hourly_eur = plan.hourly_eur.or(Some(0.0));
                plan.monthly_eur = plan.monthly_eur.or(Some(0.0));
                plan
            })
            .collect();

        info!(count = %plans_with_pricing.len(), "Fetched plans");
        Ok(plans_with_pricing)
    }

    /// Get a specific plan by slug.
    ///
    /// # Arguments
    ///
    /// * `slug` - Plan slug (e.g., "amd-epyc-9124p")
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails or plan is not found.
    pub async fn get_plan(&self, slug: &str) -> Result<crate::providers::cherry::models::PlanWithPricing, ProviderError> {
        info!(plan_slug = %slug, "Fetching plan details");
        let plan: crate::providers::cherry::models::PlanWithPricing = self.get(&format!("/plans/{slug}")).await?;
        Ok(plan)
    }

    /// List all available regions.
    ///
    /// # Errors
    ///
    /// Returns `ProviderError` if the API request fails.
    pub async fn list_regions(&self) -> Result<Vec<crate::providers::cherry::models::Region>, ProviderError> {
        info!("Fetching available regions from Cherry API");
        let regions: Vec<crate::providers::cherry::models::Region> = self.get("/regions").await?;
        info!(count = %regions.len(), "Fetched regions");
        Ok(regions)
    }

    // =========================================================================
    // Frictionless Initialization Methods
    // =========================================================================

    /// Get or auto-create default project for the team.
    async fn get_or_create_default_project(&self) -> Result<i64, ProviderError> {
        // Try to get existing projects
        let projects: Vec<Project> = self.get(&format!("/teams/{}/projects", self.team_id)).await?;

        if let Some(project) = projects.first() {
            info!(project_id = %project.id, project_name = %project.name, "Using existing project");
            return Ok(project.id);
        }

        // No projects exist - create one
        info!("No projects found, creating default project...");
        let new_project: Project = self
            .post(
                &format!("/teams/{}/projects", self.team_id),
                &serde_json::json!({
                    "name": "default-metal-project"
                }),
            )
            .await?;

        info!(project_id = %new_project.id, "Created default project");
        Ok(new_project.id)
    }

    /// Detect existing SSH keys from ~/.ssh/
    fn detect_existing_ssh_keys() -> Vec<(String, String)> {
        let mut keys = Vec::new();
        let ssh_dir = std::path::Path::new(&std::env::var("HOME").unwrap_or_else(|_| "~".to_string()))
            .join(".ssh");

        if !ssh_dir.exists() {
            return keys;
        }

        for entry in std::fs::read_dir(&ssh_dir).ok().into_iter().flatten() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Skip non-key files and encrypted keys
                    if (filename.starts_with("id_") && (filename.ends_with(".pub") || filename.ends_with(".pem")))
                        && !filename.contains("-old")
                    {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Check if private key (has -----BEGIN)
                            let is_public = content.contains("PUBLIC KEY") || path.to_string_lossy().ends_with(".pub");
                            if is_public {
                                if let Ok(public_key) = std::fs::read_to_string(&path) {
                                    let name = filename.trim_end_matches(".pub").to_string();
                                    keys.push((name, public_key.trim().to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        keys
    }

    /// Auto-detect or create SSH keys for initialization.
    async fn get_or_create_ssh_keys(&self) -> Result<Vec<(String, i64)>, ProviderError> {
        // Check for existing keys in ~/.ssh/
        let existing_keys = Self::detect_existing_ssh_keys();

        if !existing_keys.is_empty() {
            info!(count = %existing_keys.len(), "Found existing SSH keys");
        }

        // Also check for existing keys in Cherry
        let cherry_keys: Vec<SshKey> = self.get(&format!("/teams/{}/ssh-keys", self.team_id)).await?;

        let mut key_ids: Vec<(String, i64)> = Vec::new();

        // Upload any detected local keys that aren't already in Cherry
        for (name, public_key) in existing_keys {
            // Check if this key already exists in Cherry
            let already_exists = cherry_keys.iter().any(|k| k.key.trim() == public_key.trim());

            if already_exists {
                if let Some(k) = cherry_keys.iter().find(|k| k.key.trim() == public_key.trim()) {
                    info!(key_id = %k.id, key_name = %name, "Using existing Cherry SSH key");
                    key_ids.push((name, k.id));
                }
            } else {
                // Upload the key
                let label = format!("metal-{}", name);
                match self.create_ssh_key(&label, &public_key).await {
                    Ok(new_key) => {
                        info!(key_id = %new_key.id, key_name = %label, "Uploaded SSH key");
                        key_ids.push((label, new_key.id));
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to upload SSH key, will use existing Cherry keys");
                    }
                }
            }
        }

        // If no local keys uploaded, return existing Cherry keys
        if key_ids.is_empty() {
            for key in &cherry_keys {
                info!(key_id = %key.id, key_name = %key.label, "Using existing Cherry SSH key");
                key_ids.push((key.label.clone(), key.id));
            }
        }

        // If still no keys, create a new one
        if key_ids.is_empty() {
            info!("No SSH keys found, creating a new one...");
            let new_key = self.create_ssh_key("metal-default", "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGl3Wn7FZmGWO9g2e1Z8Z8Z8Z8Z8Z8Z8Z8Z8Z8Z8Z metal@auto-generated").await?;
            key_ids.push((new_key.label, new_key.id));
        }

        Ok(key_ids)
    }

    /// Frictionless initialization - detects/creates SSH keys and project.
    ///
    /// Returns (project_id, ssh_key_ids) for use in server creation.
    pub async fn frictionless_init(&self) -> Result<(i64, Vec<i64>), ProviderError> {
        let project_id = self.get_or_create_default_project().await?;
        let ssh_keys = self.get_or_create_ssh_keys().await?;
        let ssh_key_ids: Vec<i64> = ssh_keys.into_iter().map(|(_, id)| id).collect();

        info!(project_id = %project_id, ssh_key_count = %ssh_key_ids.len(), "Frictionless init complete");
        Ok((project_id, ssh_key_ids))
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
