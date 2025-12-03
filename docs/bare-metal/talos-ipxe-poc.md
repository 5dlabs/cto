# Talos iPXE Boot POC - Latitude.sh

> **Status**: Implementation Ready  
> **Scope**: Validate Talos Linux iPXE boot on Latitude.sh bare metal  
> **Estimated Time**: 2-3 days  
> **Estimated Cost**: ~$1-2 (hourly billing)

## 1. Objective

Validate the critical path for bare metal provisioning:

1. ✅ Provision a bare metal server via Latitude API
2. ✅ Reinstall with iPXE pointing to Talos Factory
3. ✅ Boot into Talos Linux maintenance mode
4. ✅ Verify `talosctl` connectivity

**This is the first potential showstopper** - if iPXE boot doesn't work from Latitude's network to Talos Factory, we need to know immediately.

---

## 2. Technical Details

### 2.1 Latitude API Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│ Step 1: Get API Key from Vault                                       │
│ secret/providers/latitude → api_key                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 2: List Plans (find cheapest in-stock)                          │
│ GET https://api.latitude.sh/plans?filter[in_stock]=true              │
│ → Select c2-small-x86 (~$0.15/hour)                                  │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 3: Deploy Server with Ubuntu (temporary)                        │
│ POST https://api.latitude.sh/servers                                 │
│ {                                                                    │
│   "data": {                                                          │
│     "type": "servers",                                               │
│     "attributes": {                                                  │
│       "project": "<project_id>",                                     │
│       "plan": "c2-small-x86",                                        │
│       "site": "ash",                                                 │
│       "operating_system": "ubuntu_22_04_x64_lts",                    │
│       "hostname": "cto-talos-poc"                                    │
│     }                                                                │
│   }                                                                  │
│ }                                                                    │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 4: Poll until status="on"                                       │
│ GET https://api.latitude.sh/servers/{id}                             │
│ → Wait for primary_ipv4 to be assigned                               │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 5: Reinstall with Talos iPXE                                    │
│ POST https://api.latitude.sh/servers/{id}/reinstall                  │
│ {                                                                    │
│   "data": {                                                          │
│     "type": "reinstalls",                                            │
│     "attributes": {                                                  │
│       "operating_system": "ipxe",                                    │
│       "hostname": "cto-talos-poc",                                   │
│       "ipxe": "https://pxe.factory.talos.dev/pxe/376567988.../v1.8.0/metal-amd64"
│     }                                                                │
│   }                                                                  │
│ }                                                                    │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 6: Wait for Talos boot, verify with talosctl                    │
│ talosctl --nodes <IP> version                                        │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Talos Factory iPXE URL

The "vanilla" schematic ID is: `376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba`

**iPXE URL format:**
```
https://pxe.factory.talos.dev/pxe/{schematic_id}/{version}/{platform}
```

**For our POC:**
```
https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.8.0/metal-amd64
```

This URL returns an iPXE script that:
1. Downloads the Talos kernel and initramfs
2. Sets required kernel parameters (`talos.platform=metal`, `slab_nomerge`, `pti=on`)
3. Boots into Talos maintenance mode

### 2.3 Talos Maintenance Mode

When booted without a machine configuration, Talos enters **maintenance mode**:
- API listens on port 50000
- Accepts machine configuration via `talosctl apply-config`
- No Kubernetes components running yet

**Verification command:**
```bash
talosctl --nodes <SERVER_IP> --endpoints <SERVER_IP> version --insecure
```

---

## 3. Implementation Plan

### 3.1 Module Structure

```
crates/installer/src/
├── providers/
│   ├── mod.rs              # Provider trait + re-exports
│   └── latitude/
│       ├── mod.rs          # Module exports
│       ├── client.rs       # HTTP client for Latitude API
│       ├── models.rs       # Serde types for JSON:API
│       └── error.rs        # Provider-specific errors
└── commands/
    └── bare_metal.rs       # CLI subcommand (future)
```

### 3.2 Core Types

```rust
// providers/latitude/models.rs

use serde::{Deserialize, Serialize};

/// JSON:API wrapper for requests
#[derive(Debug, Serialize)]
pub struct JsonApiRequest<T> {
    pub data: T,
}

/// JSON:API wrapper for responses
#[derive(Debug, Deserialize)]
pub struct JsonApiResponse<T> {
    pub data: T,
    #[serde(default)]
    pub meta: serde_json::Value,
}

/// Server creation request
#[derive(Debug, Serialize)]
pub struct CreateServerData {
    #[serde(rename = "type")]
    pub data_type: String, // "servers"
    pub attributes: CreateServerAttributes,
}

#[derive(Debug, Serialize)]
pub struct CreateServerAttributes {
    pub project: String,
    pub plan: String,
    pub site: String,
    pub operating_system: String,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_keys: Option<Vec<String>>,
}

/// Server reinstall request
#[derive(Debug, Serialize)]
pub struct ReinstallServerData {
    #[serde(rename = "type")]
    pub data_type: String, // "reinstalls"
    pub attributes: ReinstallServerAttributes,
}

#[derive(Debug, Serialize)]
pub struct ReinstallServerAttributes {
    pub operating_system: String, // "ipxe"
    pub hostname: String,
    pub ipxe: String, // Talos Factory PXE URL
}

/// Server response
#[derive(Debug, Deserialize)]
pub struct Server {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub attributes: ServerAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ServerAttributes {
    pub hostname: String,
    pub status: String,
    pub primary_ipv4: Option<String>,
    pub plan: PlanInfo,
}

#[derive(Debug, Deserialize)]
pub struct PlanInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
}

/// Plan response
#[derive(Debug, Deserialize)]
pub struct Plan {
    pub id: String,
    pub attributes: PlanAttributes,
}

#[derive(Debug, Deserialize)]
pub struct PlanAttributes {
    pub slug: String,
    pub name: String,
    pub specs: PlanSpecs,
    pub regions: Vec<PlanRegion>,
}

#[derive(Debug, Deserialize)]
pub struct PlanSpecs {
    pub cpu: CpuSpec,
    pub memory: MemorySpec,
}

#[derive(Debug, Deserialize)]
pub struct CpuSpec {
    pub cores: u32,
    pub clock: f32,
}

#[derive(Debug, Deserialize)]
pub struct MemorySpec {
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlanRegion {
    pub name: String,
    pub stock_level: String,
    pub locations: PlanLocations,
    pub pricing: PlanPricing,
}

#[derive(Debug, Deserialize)]
pub struct PlanLocations {
    pub available: Vec<String>,
    pub in_stock: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlanPricing {
    #[serde(rename = "USD")]
    pub usd: PriceTier,
}

#[derive(Debug, Deserialize)]
pub struct PriceTier {
    pub hour: f32,
    pub month: f32,
}
```

### 3.3 HTTP Client

```rust
// providers/latitude/client.rs

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::time::Duration;

use super::models::*;

const API_BASE: &str = "https://api.latitude.sh";
const TALOS_VERSION: &str = "v1.8.0";
const TALOS_SCHEMATIC: &str = "376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba";

pub struct LatitudeClient {
    client: reqwest::Client,
    api_key: String,
}

impl LatitudeClient {
    pub fn new(api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self { client, api_key })
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.api+json"),
        );
        headers
    }

    /// List available plans with stock
    pub async fn list_plans(&self, in_stock: bool) -> Result<Vec<Plan>> {
        let url = format!(
            "{}/plans?filter[in_stock]={}",
            API_BASE,
            in_stock
        );
        
        let resp: JsonApiResponse<Vec<Plan>> = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(resp.data)
    }

    /// Create a new server
    pub async fn create_server(
        &self,
        project: &str,
        plan: &str,
        site: &str,
        hostname: &str,
    ) -> Result<Server> {
        let body = JsonApiRequest {
            data: CreateServerData {
                data_type: "servers".to_string(),
                attributes: CreateServerAttributes {
                    project: project.to_string(),
                    plan: plan.to_string(),
                    site: site.to_string(),
                    operating_system: "ubuntu_22_04_x64_lts".to_string(),
                    hostname: hostname.to_string(),
                    ssh_keys: None,
                },
            },
        };

        let resp: JsonApiResponse<Server> = self.client
            .post(&format!("{}/servers", API_BASE))
            .headers(self.headers())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(resp.data)
    }

    /// Get server status
    pub async fn get_server(&self, server_id: &str) -> Result<Server> {
        let resp: JsonApiResponse<Server> = self.client
            .get(&format!("{}/servers/{}", API_BASE, server_id))
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(resp.data)
    }

    /// Reinstall server with Talos iPXE
    pub async fn reinstall_with_talos(&self, server_id: &str, hostname: &str) -> Result<()> {
        let ipxe_url = format!(
            "https://pxe.factory.talos.dev/pxe/{}/{}/metal-amd64",
            TALOS_SCHEMATIC,
            TALOS_VERSION
        );

        let body = JsonApiRequest {
            data: ReinstallServerData {
                data_type: "reinstalls".to_string(),
                attributes: ReinstallServerAttributes {
                    operating_system: "ipxe".to_string(),
                    hostname: hostname.to_string(),
                    ipxe: ipxe_url,
                },
            },
        };

        self.client
            .post(&format!("{}/servers/{}/reinstall", API_BASE, server_id))
            .headers(self.headers())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Delete server
    pub async fn delete_server(&self, server_id: &str) -> Result<()> {
        self.client
            .delete(&format!("{}/servers/{}", API_BASE, server_id))
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Wait for server to be ready
    pub async fn wait_for_server_ready(
        &self,
        server_id: &str,
        timeout: Duration,
    ) -> Result<Server> {
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for server to be ready");
            }

            let server = self.get_server(server_id).await?;
            
            if server.attributes.status == "on" && server.attributes.primary_ipv4.is_some() {
                return Ok(server);
            }

            tracing::info!(
                "Server status: {}, waiting...",
                server.attributes.status
            );
            
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    }
}
```

### 3.4 POC Binary

```rust
// src/bin/latitude_poc.rs

use anyhow::Result;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

mod providers;
use providers::latitude::LatitudeClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Get API key from environment (in production, from Vault)
    let api_key = std::env::var("LATITUDE_API_KEY")
        .expect("LATITUDE_API_KEY environment variable required");
    
    let project_id = std::env::var("LATITUDE_PROJECT_ID")
        .expect("LATITUDE_PROJECT_ID environment variable required");

    let client = LatitudeClient::new(api_key)?;

    // Step 1: List available plans
    tracing::info!("📋 Listing available plans...");
    let plans = client.list_plans(true).await?;
    
    // Find cheapest plan
    let cheapest = plans
        .iter()
        .filter(|p| !p.attributes.regions.is_empty())
        .min_by(|a, b| {
            let a_price = a.attributes.regions[0].pricing.usd.hour;
            let b_price = b.attributes.regions[0].pricing.usd.hour;
            a_price.partial_cmp(&b_price).unwrap()
        })
        .expect("No plans available");

    let site = &cheapest.attributes.regions[0].locations.in_stock[0];
    
    tracing::info!(
        "✅ Selected plan: {} (${}/hour) in {}",
        cheapest.attributes.name,
        cheapest.attributes.regions[0].pricing.usd.hour,
        site
    );

    // Step 2: Create server
    tracing::info!("🚀 Creating server...");
    let server = client
        .create_server(&project_id, &cheapest.attributes.slug, site, "cto-talos-poc")
        .await?;
    
    tracing::info!("✅ Server created: {}", server.id);

    // Step 3: Wait for server ready
    tracing::info!("⏳ Waiting for server to be ready (this may take 5-10 minutes)...");
    let server = client
        .wait_for_server_ready(&server.id, Duration::from_secs(900))
        .await?;
    
    let server_ip = server.attributes.primary_ipv4.as_ref().unwrap();
    tracing::info!("✅ Server ready at IP: {}", server_ip);

    // Step 4: Reinstall with Talos
    tracing::info!("🔄 Reinstalling with Talos Linux via iPXE...");
    client
        .reinstall_with_talos(&server.id, "cto-talos-poc")
        .await?;
    
    tracing::info!("✅ Reinstall triggered, waiting for Talos boot...");

    // Step 5: Wait for Talos (this is the key validation)
    tracing::info!("⏳ Waiting for Talos to boot (this may take 5-10 minutes)...");
    tokio::time::sleep(Duration::from_secs(300)).await;

    // Step 6: Verify Talos
    tracing::info!("🔍 Verifying Talos connectivity...");
    tracing::info!("Run: talosctl --nodes {} --endpoints {} version --insecure", server_ip, server_ip);

    // Cleanup prompt
    tracing::info!("");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("POC COMPLETE - Server is running at {}", server_ip);
    tracing::info!("Server ID: {}", server.id);
    tracing::info!("");
    tracing::info!("⚠️  Remember to delete the server when done:");
    tracing::info!("   LATITUDE_API_KEY=... cargo run --bin latitude_poc -- delete {}", server.id);
    tracing::info!("═══════════════════════════════════════════════════════════════");

    Ok(())
}
```

---

## 4. Success Criteria

| Test | Command | Expected Result |
|------|---------|-----------------|
| API Auth | `GET /user/profile` | 200 OK with user info |
| List Plans | `GET /plans?filter[in_stock]=true` | List of available plans |
| Create Server | `POST /servers` | Server created, ID returned |
| Server Ready | Poll `GET /servers/{id}` | status="on", primary_ipv4 assigned |
| iPXE Reinstall | `POST /servers/{id}/reinstall` | 201 Created |
| Talos Boot | `talosctl version --insecure` | Talos version response |

---

## 5. Potential Failure Modes

| Failure | Detection | Mitigation |
|---------|-----------|------------|
| iPXE URL unreachable from Latitude | Server stays in "deploying" | Check IPMI console, try alternative PXE server |
| Talos Factory down | iPXE script fails to load | Host our own iPXE endpoint |
| Network config mismatch | Talos boots but no network | Use IPMI to debug, check DHCP |
| Server never reaches "on" | Timeout in polling | Check Latitude dashboard, contact support |

---

## 6. Timeline

| Day | Tasks |
|-----|-------|
| Day 1 | Implement `LatitudeClient` with auth, list_plans, create_server |
| Day 1 | Test API connectivity with real credentials |
| Day 2 | Implement reinstall_with_talos, wait logic |
| Day 2 | Run full POC, validate Talos boot |
| Day 3 | Document results, cleanup, plan next phase |

---

## 7. References

- [Latitude API - Deploy Server](https://www.latitude.sh/docs/api-reference/create-server)
- [Latitude API - Reinstall Server](https://www.latitude.sh/docs/api-reference/create-server-reinstall)
- [Talos Image Factory](https://docs.siderolabs.com/talos/v1.8/learn-more/image-factory)
- [Talos PXE Boot](https://docs.siderolabs.com/talos/v1.8/platform-specific-installations/bare-metal-platforms/pxe)
- [Talos Factory PXE URL](https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.8.0/metal-amd64)

