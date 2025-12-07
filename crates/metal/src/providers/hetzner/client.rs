//! Hetzner Robot API client implementation.
//!
//! Uses the Hetzner Robot API for dedicated server management.
//! API Documentation: <https://robot.hetzner.com/doc/webservice/en.html>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    CancellationRequest, CancellationResponse, HetznerServer, OrderResponse, OrderServerRequest,
    RescueRequest, RescueResponse, ResetRequest, ServerListResponse,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for Hetzner Robot API.
const API_BASE_URL: &str = "https://robot-ws.your-server.de";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// Hetzner Robot bare metal provider.
#[derive(Clone)]
pub struct Hetzner {
    /// HTTP client.
    client: Client,
    /// Robot API username.
    username: String,
    /// Robot API password.
    password: String,
}

impl Hetzner {
    /// Create a new Hetzner provider.
    ///
    /// # Arguments
    /// * `username` - Hetzner Robot API username
    /// * `password` - Hetzner Robot API password
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            username: username.into(),
            password: password.into(),
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "GET request");

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request with form data.
    async fn post_form<T, B>(&self, path: &str, body: &B) -> Result<T, ProviderError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request (form)");

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .form(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request that returns empty body.
    async fn post_form_empty<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request (empty response)");

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .form(body)
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
    #[allow(dead_code)]
    async fn delete(&self, path: &str) -> Result<(), ProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
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

    /// Convert Hetzner server to our Server type.
    fn to_server(server: &HetznerServer) -> Server {
        let status = match server.server.status.as_str() {
            "ready" => ServerStatus::On,
            "in process" => ServerStatus::Deploying,
            "cancelled" => ServerStatus::Deleting,
            _ => ServerStatus::Unknown,
        };

        Server {
            id: server.server.server_number.to_string(),
            hostname: server.server.server_name.clone(),
            status,
            ipv4: Some(server.server.server_ip.clone()),
            ipv6: server.server.server_ipv6_net.clone(),
            plan: server.server.product.clone(),
            region: server.server.dc.clone(),
            created_at: None, // Hetzner doesn't provide creation date in server list
        }
    }

    /// Activate rescue mode for iPXE reinstall.
    async fn activate_rescue(&self, server_number: &str) -> Result<RescueResponse, ProviderError> {
        let req = RescueRequest {
            os: "linux".to_string(),
            authorized_key: vec![],
        };
        self.post_form(&format!("/boot/{server_number}/rescue"), &req)
            .await
    }

    /// Trigger hardware reset.
    async fn reset_server(&self, server_number: &str) -> Result<(), ProviderError> {
        let req = ResetRequest {
            reset_type: "hw".to_string(),
        };
        self.post_form_empty(&format!("/reset/{server_number}"), &req)
            .await
    }

    /// Check order transaction status.
    /// Used to poll for server provisioning completion.
    ///
    /// # Errors
    /// Returns error if the API request fails or transaction is not found.
    pub async fn get_transaction(&self, transaction_id: &str) -> Result<OrderResponse, ProviderError> {
        self.get(&format!("/order/server/transaction/{transaction_id}"))
            .await
    }

    /// Check if an ID looks like a transaction ID (starts with "B").
    fn is_transaction_id(id: &str) -> bool {
        id.starts_with('B') && id.contains('-')
    }
}

#[async_trait]
impl Provider for Hetzner {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating server via Hetzner Robot API"
        );

        // Build order request
        // Note: plan maps to product_id, region maps to location
        let order_req = OrderServerRequest {
            product_id: req.plan.clone(),
            location: Some(req.region.clone()),
            authorized_key: req.ssh_keys.clone(),
            dist: if req.os.is_empty() {
                None
            } else {
                Some(req.os.clone())
            },
            lang: Some("en".to_string()),
            comment: None,
            test: None,
        };

        // Submit the order
        let response: OrderResponse = self
            .post_form("/order/server/transaction", &order_req)
            .await?;

        info!(
            transaction_id = %response.transaction.id,
            status = %response.transaction.status,
            "Server order submitted"
        );

        // Check if server is already ready (instant provisioning)
        if let (Some(server_number), Some(server_ip)) = (
            response.transaction.server_number,
            &response.transaction.server_ip,
        ) {
            info!(
                server_number = server_number,
                server_ip = %server_ip,
                "Server provisioned immediately"
            );

            return Ok(Server {
                id: server_number.to_string(),
                hostname: req.hostname,
                status: ServerStatus::On,
                ipv4: Some(server_ip.clone()),
                ipv6: None,
                plan: req.plan,
                region: req.region,
                created_at: None,
            });
        }

        // Server is being provisioned - return with deploying status
        // The caller should use wait_ready() to poll for completion
        // For now, use transaction ID as temporary server ID
        // Caller will need to query the transaction to get actual server_number
        Ok(Server {
            id: response.transaction.id.clone(),
            hostname: req.hostname,
            status: ServerStatus::Deploying,
            ipv4: None,
            ipv6: None,
            plan: req.plan,
            region: req.region,
            created_at: None,
        })
    }

    async fn get_server(&self, id: &str) -> Result<Server, ProviderError> {
        let response: HetznerServer = self.get(&format!("/server/{id}")).await?;
        Ok(Self::to_server(&response))
    }

    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError> {
        info!(server_id = %id, timeout_secs, "Waiting for server to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        // Check if this is a transaction ID (from create_server)
        let is_transaction = Self::is_transaction_id(id);

        loop {
            if is_transaction {
                // Poll transaction status until we get a server_number
                match self.get_transaction(id).await {
                    Ok(tx) => {
                        debug!(
                            transaction_id = %id,
                            status = %tx.transaction.status,
                            server_number = ?tx.transaction.server_number,
                            elapsed_secs = start.elapsed().as_secs(),
                            "Polling transaction status"
                        );

                        match tx.transaction.status.as_str() {
                            "ready" => {
                                if let Some(server_number) = tx.transaction.server_number {
                                    info!(
                                        transaction_id = %id,
                                        server_number = server_number,
                                        "Transaction complete, fetching server details"
                                    );
                                    // Server is ready, fetch full details
                                    return self.get_server(&server_number.to_string()).await;
                                }
                            }
                            "cancelled" => {
                                return Err(ProviderError::Api {
                                    status: 400,
                                    message: format!("Order transaction {id} was cancelled"),
                                });
                            }
                            _ => {
                                // Still in process, continue polling
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            transaction_id = %id,
                            error = %e,
                            "Failed to get transaction status"
                        );
                    }
                }
            } else {
                // Regular server ID - poll server status
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
            "Triggering iPXE reinstall via rescue mode"
        );

        // 1. Activate rescue mode
        let rescue = self.activate_rescue(id).await?;
        info!(
            server_id = %id,
            password = ?rescue.rescue.password,
            "Rescue mode activated"
        );

        // 2. Reset the server to boot into rescue
        self.reset_server(id).await?;
        info!(server_id = %id, "Server reset triggered");

        // Note: After rescue boot, you'd SSH in and chainload the iPXE URL
        // This is provider-specific and may require additional automation

        Ok(())
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Cancelling server via Hetzner Robot API");

        // First, check current cancellation status
        let cancel_status_result: Result<CancellationResponse, _> =
            self.get(&format!("/server/{id}/cancellation")).await;

        // Check if already cancelled
        if let Ok(status) = &cancel_status_result {
            if status.cancellation.cancelled {
                info!(
                    server_id = %id,
                    cancellation_date = ?status.cancellation.cancellation_date,
                    "Server is already cancelled"
                );
                return Ok(());
            }
        }

        // Submit cancellation request with immediate cancellation
        let cancel_req = CancellationRequest {
            cancellation_date: "now".to_string(),
            cancellation_reason: Some("Automated cancellation via CTO Metal API".to_string()),
            reserve_location: Some("false".to_string()),
        };

        let response: CancellationResponse = self
            .post_form(&format!("/server/{id}/cancellation"), &cancel_req)
            .await?;

        if response.cancellation.cancelled {
            info!(
                server_id = %id,
                cancellation_date = ?response.cancellation.cancellation_date,
                "Server cancellation submitted successfully"
            );
            Ok(())
        } else {
            warn!(
                server_id = %id,
                "Cancellation request accepted but server not marked as cancelled"
            );
            Ok(())
        }
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        let response: ServerListResponse = self.get("/server").await?;
        Ok(response.servers.iter().map(Self::to_server).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_mapping() {
        let server = HetznerServer {
            server: super::super::models::ServerDetails {
                server_number: 12345,
                server_name: "test-server".to_string(),
                server_ip: "1.2.3.4".to_string(),
                server_ipv6_net: Some("2001:db8::/64".to_string()),
                product: "AX52".to_string(),
                dc: "FSN1".to_string(),
                status: "ready".to_string(),
                paid_until: None,
                cancelled: false,
            },
        };

        let converted = Hetzner::to_server(&server);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "12345");
        assert_eq!(converted.plan, "AX52");
    }

    #[test]
    fn test_transaction_id_detection() {
        // Valid transaction IDs
        assert!(Hetzner::is_transaction_id("B20150121-344958-251479"));
        assert!(Hetzner::is_transaction_id("B20240101-123456-789012"));

        // Server numbers (not transaction IDs)
        assert!(!Hetzner::is_transaction_id("12345"));
        assert!(!Hetzner::is_transaction_id("1234567"));

        // Other invalid formats
        assert!(!Hetzner::is_transaction_id("A20150121-344958-251479"));
        assert!(!Hetzner::is_transaction_id("B20150121"));
    }

    #[test]
    fn test_order_request_serialization() {
        let req = OrderServerRequest {
            product_id: "EX44".to_string(),
            location: Some("FSN1".to_string()),
            authorized_key: vec!["abc123".to_string()],
            dist: Some("ubuntu-2204".to_string()),
            lang: Some("en".to_string()),
            comment: None,
            test: None,
        };

        // Verify it can serialize
        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("EX44"));
        assert!(json_str.contains("FSN1"));
    }

    #[test]
    fn test_cancellation_request_serialization() {
        let req = super::super::models::CancellationRequest {
            cancellation_date: "now".to_string(),
            cancellation_reason: Some("Test cancellation".to_string()),
            reserve_location: Some("false".to_string()),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("cancellation_date"));
        assert!(json_str.contains("now"));
    }

    #[test]
    fn test_order_response_deserialization() {
        let json = r#"{
            "transaction": {
                "id": "B20150121-344958-251479",
                "date": "2015-01-21T10:55:42+01:00",
                "status": "ready",
                "server_number": 321,
                "server_ip": "123.123.123.123",
                "product": {
                    "id": "EX40",
                    "name": "EX40",
                    "location": "FSN1"
                }
            }
        }"#;

        let response: Result<OrderResponse, _> = serde_json::from_str(json);
        assert!(response.is_ok());
        let resp = response.unwrap();
        assert_eq!(resp.transaction.id, "B20150121-344958-251479");
        assert_eq!(resp.transaction.status, "ready");
        assert_eq!(resp.transaction.server_number, Some(321));
    }

    #[test]
    fn test_cancellation_response_deserialization() {
        let json = r#"{
            "cancellation": {
                "server_ip": "123.123.123.123",
                "server_number": 321,
                "server_name": "my-server",
                "earliest_cancellation_date": "2024-02-01",
                "cancelled": true,
                "cancellation_date": "2024-01-15"
            }
        }"#;

        let response: Result<super::super::models::CancellationResponse, _> =
            serde_json::from_str(json);
        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.cancellation.cancelled);
        assert_eq!(resp.cancellation.server_number, 321);
    }
}
