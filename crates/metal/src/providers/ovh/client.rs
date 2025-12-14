//! `OVHcloud` API client implementation.
//!
//! Uses the `OVHcloud` API for dedicated server management.
//! API Documentation: <https://api.ovh.com/>

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    AddCartItemRequest, BaremetalProduct, Cart, CartItem, CheckoutRequest, ConfigurationResponse,
    CreateCartRequest, DedicatedServer, InstallationDetails, InstallationRequest,
    ItemConfigurationRequest, OrderResponse, RebootRequest,
};
use crate::providers::traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};

/// Base URL for `OVHcloud` API (Europe).
const API_BASE_URL: &str = "https://eu.api.ovh.com/1.0";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for server status.
const POLL_INTERVAL_SECS: u64 = 15;

/// `OVHcloud` bare metal provider.
#[derive(Clone)]
pub struct Ovh {
    /// HTTP client.
    client: Client,
    /// Application key.
    application_key: String,
    /// Application secret.
    application_secret: String,
    /// Consumer key (for authenticated requests).
    consumer_key: String,
    /// OVH subsidiary (US, EU, etc).
    subsidiary: String,
}

impl Ovh {
    /// Create a new OVH provider.
    ///
    /// # Arguments
    /// * `application_key` - OVH Application Key
    /// * `application_secret` - OVH Application Secret
    /// * `consumer_key` - OVH Consumer Key
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        application_key: impl Into<String>,
        application_secret: impl Into<String>,
        consumer_key: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        Self::with_subsidiary(application_key, application_secret, consumer_key, "US")
    }

    /// Create a new OVH provider with a specific subsidiary.
    ///
    /// # Arguments
    /// * `application_key` - OVH Application Key
    /// * `application_secret` - OVH Application Secret
    /// * `consumer_key` - OVH Consumer Key
    /// * `subsidiary` - OVH subsidiary code (US, EU, FR, etc.)
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn with_subsidiary(
        application_key: impl Into<String>,
        application_secret: impl Into<String>,
        consumer_key: impl Into<String>,
        subsidiary: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            application_key: application_key.into(),
            application_secret: application_secret.into(),
            consumer_key: consumer_key.into(),
            subsidiary: subsidiary.into(),
        })
    }

    /// Generate OVH API signature.
    fn generate_signature(&self, method: &str, url: &str, body: &str, timestamp: i64) -> String {
        use sha1::{Digest, Sha1};

        let to_sign = format!(
            "{}+{}+{}+{}+{}+{}",
            self.application_secret, self.consumer_key, method, url, body, timestamp
        );

        let mut hasher = Sha1::new();
        hasher.update(to_sign.as_bytes());
        let result = hasher.finalize();

        format!("$1${}", hex::encode(result))
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "GET request");

        let timestamp = chrono::Utc::now().timestamp();
        let signature = self.generate_signature("GET", &url, "", timestamp);

        let response = self
            .client
            .get(&url)
            .header("X-Ovh-Application", &self.application_key)
            .header("X-Ovh-Consumer", &self.consumer_key)
            .header("X-Ovh-Timestamp", timestamp.to_string())
            .header("X-Ovh-Signature", signature)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request.
    #[allow(dead_code)]
    async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ProviderError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request");

        let body_str = serde_json::to_string(body)?;
        let timestamp = chrono::Utc::now().timestamp();
        let signature = self.generate_signature("POST", &url, &body_str, timestamp);

        let response = self
            .client
            .post(&url)
            .header("X-Ovh-Application", &self.application_key)
            .header("X-Ovh-Consumer", &self.consumer_key)
            .header("X-Ovh-Timestamp", timestamp.to_string())
            .header("X-Ovh-Signature", signature)
            .header("Content-Type", "application/json")
            .body(body_str)
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

        let body_str = serde_json::to_string(body)?;
        let timestamp = chrono::Utc::now().timestamp();
        let signature = self.generate_signature("POST", &url, &body_str, timestamp);

        let response = self
            .client
            .post(&url)
            .header("X-Ovh-Application", &self.application_key)
            .header("X-Ovh-Consumer", &self.consumer_key)
            .header("X-Ovh-Timestamp", timestamp.to_string())
            .header("X-Ovh-Signature", signature)
            .header("Content-Type", "application/json")
            .body(body_str)
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

        let timestamp = chrono::Utc::now().timestamp();
        let signature = self.generate_signature("DELETE", &url, "", timestamp);

        let response = self
            .client
            .delete(&url)
            .header("X-Ovh-Application", &self.application_key)
            .header("X-Ovh-Consumer", &self.consumer_key)
            .header("X-Ovh-Timestamp", timestamp.to_string())
            .header("X-Ovh-Signature", signature)
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

    /// Convert OVH server to our Server type.
    fn to_server(server: &DedicatedServer) -> Server {
        let status = match server.state.as_str() {
            "ok" => ServerStatus::On,
            "hacked" | "hackedBlocked" => ServerStatus::Off,
            "installing" => ServerStatus::Reinstalling,
            _ => ServerStatus::Unknown,
        };

        Server {
            id: server.name.clone(),
            hostname: server
                .reverse
                .clone()
                .unwrap_or_else(|| server.name.clone()),
            status,
            ipv4: server.ip.clone(),
            ipv6: None, // OVH provides IPv6 via separate API
            plan: server.commercial_range.clone().unwrap_or_default(),
            region: server.datacenter.clone().unwrap_or_default(),
            created_at: None, // OVH doesn't provide creation date
        }
    }
}

#[async_trait]
impl Provider for Ovh {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        info!(
            hostname = %req.hostname,
            plan = %req.plan,
            region = %req.region,
            "Creating server via OVH Cart API"
        );

        // Step 1: Create a cart
        let cart_req = CreateCartRequest {
            ovh_subsidiary: self.subsidiary.clone(),
            description: Some(format!("cto-metal-{}", req.hostname)),
        };
        let cart: Cart = self.post("/order/cart", &cart_req).await?;
        info!(cart_id = %cart.cart_id, "Created cart");

        // Step 2: Assign cart to account
        self.post_empty(&format!("/order/cart/{}/assign", cart.cart_id), &serde_json::json!({}))
            .await?;
        debug!(cart_id = %cart.cart_id, "Assigned cart to account");

        // Step 3: Get available plans and find matching one
        let products: Vec<BaremetalProduct> = self
            .get(&format!("/order/cart/{}/baremetalServers", cart.cart_id))
            .await?;

        let product = products
            .iter()
            .find(|p| p.plan_code == req.plan)
            .ok_or_else(|| {
                ProviderError::Config(format!(
                    "Plan '{}' not found. Available plans: {}",
                    req.plan,
                    products
                        .iter()
                        .map(|p| p.plan_code.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })?;

        let duration = product.duration.first().cloned().unwrap_or_else(|| "P1M".to_string());

        // Step 4: Add server to cart
        let add_item_req = AddCartItemRequest {
            plan_code: req.plan.clone(),
            duration: duration.clone(),
            pricing_mode: product.pricing_mode.clone(),
            quantity: 1,
        };
        let item: CartItem = self
            .post(&format!("/order/cart/{}/baremetalServers", cart.cart_id), &add_item_req)
            .await?;
        info!(item_id = item.item_id, "Added server to cart");

        // Step 5: Configure server - datacenter
        let dc_config = ItemConfigurationRequest {
            label: "dedicated_datacenter".to_string(),
            value: req.region.clone(),
        };
        let _: ConfigurationResponse = self
            .post(
                &format!("/order/cart/{}/item/{}/configuration", cart.cart_id, item.item_id),
                &dc_config,
            )
            .await?;

        // Step 5b: Configure server - OS
        let os_config = ItemConfigurationRequest {
            label: "dedicated_os".to_string(),
            value: if req.os.is_empty() {
                "none_64.en".to_string()
            } else {
                req.os.clone()
            },
        };
        let _: ConfigurationResponse = self
            .post(
                &format!("/order/cart/{}/item/{}/configuration", cart.cart_id, item.item_id),
                &os_config,
            )
            .await?;

        // Step 5c: Configure server - region
        let region_config = ItemConfigurationRequest {
            label: "region".to_string(),
            value: self.subsidiary.to_lowercase(),
        };
        let _: ConfigurationResponse = self
            .post(
                &format!("/order/cart/{}/item/{}/configuration", cart.cart_id, item.item_id),
                &region_config,
            )
            .await?;

        debug!(cart_id = %cart.cart_id, "Configured server options");

        // Step 6: Checkout with auto-pay
        let checkout_req = CheckoutRequest {
            auto_pay_with_preferred_payment_method: true,
            waive_retractation_period: false,
        };
        let order: OrderResponse = self
            .post(&format!("/order/cart/{}/checkout", cart.cart_id), &checkout_req)
            .await?;

        info!(
            order_id = order.order_id,
            order_url = %order.url,
            "Order placed successfully"
        );

        // Return a placeholder server - actual server details come after provisioning
        // The order ID can be used to track the order status
        Ok(Server {
            id: format!("order-{}", order.order_id),
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
        let server: DedicatedServer = self.get(&format!("/dedicated/server/{id}")).await?;
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
            "Triggering iPXE reinstall via OVH API"
        );

        // Install with custom iPXE script
        let install_req = InstallationRequest {
            template_name: "none_64".to_string(), // Bypass OS installation
            partition_scheme_name: None,
            details: Some(InstallationDetails {
                custom_hostname: Some(req.hostname),
                ssh_key_name: None,
                post_installation_script_link: Some(req.ipxe_url),
                post_installation_script_return: None,
            }),
        };

        self.post_empty(
            &format!("/dedicated/server/{id}/install/start"),
            &install_req,
        )
        .await?;

        // Reboot the server
        let reboot_req = RebootRequest {
            reboot_type: Some("hardreset".to_string()),
        };
        self.post_empty(&format!("/dedicated/server/{id}/reboot"), &reboot_req)
            .await?;

        info!(server_id = %id, "iPXE reinstall triggered");
        Ok(())
    }

    async fn delete_server(&self, id: &str) -> Result<(), ProviderError> {
        info!(server_id = %id, "Terminating server via OVH API");

        // OVH servers are terminated through the terminate endpoint
        // POST /dedicated/server/{serviceName}/terminate
        self.post_empty(
            &format!("/dedicated/server/{id}/terminate"),
            &serde_json::json!({}),
        )
        .await?;

        info!(server_id = %id, "Server termination request submitted");
        Ok(())
    }

    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError> {
        // First get list of server names
        let server_names: Vec<String> = self.get("/dedicated/server").await?;

        // Then get details for each server
        let mut servers = Vec::new();
        for name in server_names {
            match self.get_server(&name).await {
                Ok(server) => servers.push(server),
                Err(e) => warn!(server = %name, error = %e, "Failed to get server details"),
            }
        }

        Ok(servers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::models::*;

    #[test]
    fn test_server_status_mapping() {
        let server = DedicatedServer {
            name: "ns1234567.ip-1-2-3.eu".to_string(),
            ip: Some("1.2.3.4".to_string()),
            datacenter: Some("gra1".to_string()),
            professional_use: false,
            commercial_range: Some("Rise".to_string()),
            os: Some("ubuntu2204-server_64".to_string()),
            state: "ok".to_string(),
            reverse: Some("my-server.example.com".to_string()),
            monitoring: true,
            root_device: None,
            rack: None,
            boot_id: None,
            link_speed: Some(1000),
        };

        let converted = Ovh::to_server(&server);
        assert_eq!(converted.status, ServerStatus::On);
        assert_eq!(converted.id, "ns1234567.ip-1-2-3.eu");
        assert_eq!(converted.hostname, "my-server.example.com");
    }

    #[test]
    fn test_create_cart_request_serialization() {
        let req = CreateCartRequest {
            ovh_subsidiary: "US".to_string(),
            description: Some("test-cart".to_string()),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("ovhSubsidiary"));
        assert!(json_str.contains("US"));
    }

    #[test]
    fn test_cart_response_deserialization() {
        let json = r#"{
            "cartId": "abc123",
            "description": "test-cart",
            "expire": "2024-12-31T23:59:59Z",
            "readOnly": false
        }"#;

        let cart: Result<Cart, _> = serde_json::from_str(json);
        assert!(cart.is_ok());
        let cart = cart.unwrap();
        assert_eq!(cart.cart_id, "abc123");
        assert!(!cart.read_only);
    }

    #[test]
    fn test_baremetal_product_deserialization() {
        let json = r#"{
            "planCode": "24rise01-us",
            "productId": "baremetal",
            "productName": "Rise Server",
            "duration": ["P1M", "P12M"],
            "pricingMode": "default",
            "orderable": true
        }"#;

        let product: Result<BaremetalProduct, _> = serde_json::from_str(json);
        assert!(product.is_ok());
        let product = product.unwrap();
        assert_eq!(product.plan_code, "24rise01-us");
        assert_eq!(product.duration.len(), 2);
    }

    #[test]
    fn test_add_cart_item_request_serialization() {
        let req = AddCartItemRequest {
            plan_code: "24rise01-us".to_string(),
            duration: "P1M".to_string(),
            pricing_mode: "default".to_string(),
            quantity: 1,
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("planCode"));
        assert!(json_str.contains("24rise01-us"));
    }

    #[test]
    fn test_cart_item_deserialization() {
        let json = r#"{
            "itemId": 12345678,
            "duration": "P1M",
            "planCode": "24rise01-us"
        }"#;

        let item: Result<CartItem, _> = serde_json::from_str(json);
        assert!(item.is_ok());
        let item = item.unwrap();
        assert_eq!(item.item_id, 12345678);
        assert_eq!(item.duration, "P1M");
    }

    #[test]
    fn test_item_configuration_request_serialization() {
        let req = ItemConfigurationRequest {
            label: "dedicated_datacenter".to_string(),
            value: "hil".to_string(),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("dedicated_datacenter"));
    }

    #[test]
    fn test_checkout_request_serialization() {
        let req = CheckoutRequest {
            auto_pay_with_preferred_payment_method: true,
            waive_retractation_period: false,
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("autoPayWithPreferredPaymentMethod"));
        assert!(json_str.contains("true"));
    }

    #[test]
    fn test_order_response_deserialization() {
        let json = r#"{
            "orderId": 123456789,
            "url": "https://www.ovh.com/cgi-bin/order/displayOrder.cgi?orderId=123456789"
        }"#;

        let order: Result<OrderResponse, _> = serde_json::from_str(json);
        assert!(order.is_ok());
        let order = order.unwrap();
        assert_eq!(order.order_id, 123456789);
        assert!(order.url.contains("123456789"));
    }

    #[test]
    fn test_signature_generation() {
        // Create a test OVH client
        let ovh = Ovh::with_subsidiary(
            "test_app_key",
            "test_app_secret",
            "test_consumer_key",
            "US",
        )
        .unwrap();

        // Test signature format
        let signature = ovh.generate_signature(
            "GET",
            "https://eu.api.ovh.com/1.0/dedicated/server",
            "",
            1234567890,
        );

        // OVH signatures start with $1$
        assert!(signature.starts_with("$1$"));
        // And are hex-encoded SHA1 (40 chars after $1$)
        assert_eq!(signature.len(), 43); // "$1$" + 40 hex chars
    }
}
