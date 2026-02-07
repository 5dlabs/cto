use reqwest::{Client, ClientBuilder};
use std::time::Duration;

use crate::config::ScalewayConfig;
use crate::error::{Error, Result};
use crate::scaleway::{RescueModeResponse, Server};

/// Try to get credentials from environment variables or scw CLI config
fn load_credentials_from_env() -> Option<(String, String, String)> {
    // Check environment variables first
    if let (Ok(access_key), Ok(secret_key), Ok(project_id)) = (
        std::env::var("SCW_ACCESS_KEY"),
        std::env::var("SCW_SECRET_KEY"),
        std::env::var("SCW_DEFAULT_PROJECT_ID"),
    ) {
        return Some((access_key, secret_key, project_id));
    }

    // Try to load from scw CLI config
    if let Ok(profile) = std::env::var("SCW_PROFILE") {
        if let Ok(output) = std::process::Command::new("scw")
            .args(&["config", "info", "--profile", &profile])
            .output()
        {
            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout);
                // Parse output for access_key, secret_key, project_id
                for line in info.lines() {
                    if line.starts_with("access-key=") {
                        let ak = line.trim_start_matches("access-key=").to_string();
                        // Try to get other values from env with scw profile prefix
                        if let Ok(sk) = std::env::var(&format!("SCW_SECRET_KEY_{}", profile.to_uppercase())) {
                            if let Ok(pid) = std::env::var(&format!("SCW_PROJECT_ID_{}", profile.to_uppercase())) {
                                return Some((ak, sk, pid));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub struct ScalewayClient {
    client: Client,
    base_url: String,
    project_id: String,
    access_key: String,
    secret_key: String,
}

impl ScalewayClient {
    pub fn new(config: &ScalewayConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::ScalewayApi(e.into()))?;

        // Use config values, fall back to environment if not set
        let (access_key, secret_key, project_id) = if config.access_key.is_empty() {
            load_credentials_from_env()
                .ok_or_else(|| Error::Config("No Scaleway credentials found. Set SCW_ACCESS_KEY/SCW_SECRET_KEY or configure scw CLI".to_string()))?
        } else {
            (config.access_key.clone(), config.secret_key.clone(), config.project_id.clone())
        };

        Ok(Self {
            client,
            base_url: "https://api.scaleway.com/baremetal/v1".to_string(),
            project_id,
            access_key,
            secret_key,
        })
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth = format!("Bearer {}", self.secret_key);
        headers.insert(reqwest::header::AUTHORIZATION, auth.parse().unwrap());
        headers.insert("X-Auth-Token", self.access_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// Enable rescue mode on a server
    pub async fn enable_rescue_mode(
        &self,
        server_id: &str,
        timeout: Duration,
    ) -> Result<(String, String)> {
        let url = format!("{}/servers/{}/enable-rescue-mode", self.base_url, server_id);

        // Poll for rescue mode to become active
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err(Error::RescueModeTimeout(timeout));
            }

            let response = self
                .client
                .post(&url)
                .headers(self.auth_headers())
                .json(&serde_json::json!({
                    "type": "ubuntu"
                }))
                .send()
                .await
                .map_err(Error::ScalewayApi)?;

            if response.status() == 200 {
                let rescue: RescueModeResponse = response.json().await?;
                return Ok((rescue.rescue_ip, rescue.rescue_password));
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    /// Disable rescue mode
    pub async fn disable_rescue_mode(&self, server_id: &str) -> Result<()> {
        let url = format!(
            "{}/servers/{}/disable-rescue-mode",
            self.base_url, server_id
        );

        self.client
            .post(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(Error::ScalewayApi)?;

        Ok(())
    }

    /// Power on a server
    pub async fn power_on(&self, server_id: &str) -> Result<()> {
        let url = format!("{}/servers/{}/action", self.base_url, server_id);

        self.client
            .post(&url)
            .headers(self.auth_headers())
            .json(&serde_json::json!({
                "action": "power_on"
            }))
            .send()
            .await
            .map_err(Error::ScalewayApi)?;

        Ok(())
    }

    /// Get server details
    pub async fn get_server(&self, server_id: &str) -> Result<Server> {
        let url = format!("{}/servers/{}", self.base_url, server_id);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(Error::ScalewayApi)?;

        if response.status() == 404 {
            return Err(Error::ServerNotFound(server_id.to_string()));
        }

        let server = response.json().await?;
        Ok(server)
    }

    /// Reboot server
    pub async fn reboot(&self, server_id: &str) -> Result<()> {
        let url = format!("{}/servers/{}/action", self.base_url, server_id);

        self.client
            .post(&url)
            .headers(self.auth_headers())
            .json(&serde_json::json!({
                "action": "reboot"
            }))
            .send()
            .await
            .map_err(Error::ScalewayApi)?;

        Ok(())
    }
}
