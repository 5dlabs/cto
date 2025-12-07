# Bare-Metal Provider Implementation Plan

## Executive Summary

This document outlines the implementation status of all bare-metal providers in the `cto-metal` crate, using the Latitude implementation as the reference.

**Last Updated:** December 2024

## Current Implementation Status

| Provider | create_server | get_server | wait_ready | reinstall_ipxe | delete_server | list_servers | Tests |
|----------|--------------|------------|------------|----------------|--------------|--------------|-------|
| **Latitude** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Vultr** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Cherry Servers** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Scaleway** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **Hetzner** | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ✅ |
| **OVH** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **DigitalOcean** | ✅ | ✅ | ✅ | ❌ N/A | ✅ | ✅ | ✅ |

Legend:
- ✅ Fully implemented
- ⚠️ Partially implemented or needs review
- ❌ Not implemented/Not applicable

---

## Provider Details

### 1. Latitude (Reference Implementation)

**Status:** ✅ Complete

**API Documentation:** https://docs.latitude.sh/reference

**Key Implementation Features:**
- JSON:API specification format
- Bearer token authentication
- Full CRUD operations for servers
- iPXE reinstall via `/servers/{id}/actions/reinstall`
- Comprehensive models in `models.rs`

**Files:**
- `crates/metal/src/providers/latitude/client.rs`
- `crates/metal/src/providers/latitude/models.rs`

---

### 2. Hetzner

**Status:** ✅ Complete (as of December 2024)

**API Documentation:** https://robot.hetzner.com/doc/webservice/en.html

**Implementation Details:**

#### Server Ordering (`create_server`)
Uses `POST /order/server/transaction` endpoint:
```rust
// Parameters:
// - product_id: Server product ID (e.g., "EX44")
// - location: Data center (e.g., "FSN1", "NBG1", "HEL1")
// - authorized_key[]: SSH key fingerprints
// - dist: Distribution for preinstallation (optional)
// - lang: Language (optional)
```

Returns a transaction ID which can be polled via `wait_ready()`.

#### Server Cancellation (`delete_server`)
Uses `POST /server/{server-number}/cancellation` endpoint:
```rust
// Parameters:
// - cancellation_date: "now" for immediate cancellation
// - cancellation_reason: Optional reason string
// - reserve_location: "true" or "false"
```

#### Transaction Handling
- `create_server` returns transaction ID (format: `B20150121-xxx`) for pending orders
- `wait_ready` handles both transaction IDs and server numbers
- Polls transaction status until `server_number` is assigned

#### iPXE Reinstall
Uses rescue mode + hardware reset:
1. `POST /boot/{server}/rescue` - Activate rescue mode
2. `POST /reset/{server}` - Hardware reset to boot into rescue
3. Note: After rescue boot, SSH in to chainload iPXE URL

**Files:**
- `crates/metal/src/providers/hetzner/client.rs`
- `crates/metal/src/providers/hetzner/models.rs`

---

### 3. OVH

**Status:** ✅ Complete (as of December 2024)

**API Documentation:** https://eu.api.ovh.com/console/?section=%2Fdedicated%2Fserver

**Implementation Details:**

#### Server Ordering (`create_server`)
Uses OVH Cart API - multi-step process:

1. **Create Cart:** `POST /order/cart`
   ```json
   {"ovhSubsidiary": "US", "description": "cto-metal-{hostname}"}
   ```

2. **Assign Cart:** `POST /order/cart/{cartId}/assign`

3. **Get Products:** `GET /order/cart/{cartId}/baremetalServers`
   - Returns available plans with `planCode`, `duration`, `pricingMode`

4. **Add to Cart:** `POST /order/cart/{cartId}/baremetalServers`
   ```json
   {"planCode": "24rise01-us", "duration": "P1M", "pricingMode": "default", "quantity": 1}
   ```

5. **Configure Server:** `POST /order/cart/{cartId}/item/{itemId}/configuration`
   - `dedicated_datacenter`: Data center code
   - `dedicated_os`: OS template (or `none_64.en` for no OS)
   - `region`: Region code

6. **Checkout:** `POST /order/cart/{cartId}/checkout`
   ```json
   {"autoPayWithPreferredPaymentMethod": true, "waiveRetractationPeriod": false}
   ```

**Note:** Requires pre-funded OVH account or default payment method.

#### Server Termination (`delete_server`)
Uses `POST /dedicated/server/{serviceName}/terminate`

#### iPXE Reinstall
Uses installation API with post-install script:
```rust
// POST /dedicated/server/{id}/install/start
// template_name: "none_64"
// post_installation_script_link: iPXE URL
```

#### API Authentication
OVH uses signature-based authentication:
```
Signature = SHA1($ApplicationSecret+$ConsumerKey+$Method+$URL+$Body+$Timestamp)
Headers: X-Ovh-Application, X-Ovh-Consumer, X-Ovh-Timestamp, X-Ovh-Signature
```

**Files:**
- `crates/metal/src/providers/ovh/client.rs`
- `crates/metal/src/providers/ovh/models.rs`

---

### 4. DigitalOcean

**Status:** ✅ Complete (with documented limitations)

**Important Note:** DigitalOcean provides **cloud VMs (Droplets)**, NOT bare-metal servers.

**API Documentation:** https://docs.digitalocean.com/reference/api/

**Implementation Details:**

All standard methods work:
- `create_server` - Creates Droplet via `POST /droplets`
- `get_server` - Gets Droplet via `GET /droplets/{id}`
- `wait_ready` - Polls until status is "active"
- `delete_server` - Deletes Droplet via `DELETE /droplets/{id}`
- `list_servers` - Lists Droplets via `GET /droplets`

**Limitation:** `reinstall_ipxe` returns `ProviderError::Config`

DigitalOcean Droplets:
- Cannot boot custom iPXE
- Cannot install custom OS via netboot
- Limited to DO-provided images or custom images uploaded beforehand

**Recommendation:** Use DigitalOcean for cloud workloads, not Talos bare-metal clusters.

---

### 5. Vultr

**Status:** ✅ Complete

**API Documentation:** https://www.vultr.com/api/

**Notes:**
- Fully implemented including iPXE chain URL
- Uses `ipxe_chain_url` parameter on instance create/reinstall
- Directly supports custom iPXE boot

---

### 6. Cherry Servers

**Status:** ✅ Complete

**API Documentation:** https://api.cherryservers.com/doc/

**Notes:**
- Fully implemented
- iPXE via `user_data` field with chainload script:
  ```
  #!ipxe
  chain {ipxe_url}
  ```

---

### 7. Scaleway

**Status:** ✅ Complete

**API Documentation:** https://www.scaleway.com/en/developers/api/elastic-metal/

**Notes:**
- All methods implemented
- Uses special `ipxe` OS ID for iPXE reinstall
- Multi-zone support via `SCALEWAY_ZONE` configuration

---

## Environment Variables Required

| Provider | Variables |
|----------|-----------|
| Latitude | `LATITUDE_API_KEY` |
| Hetzner | `HETZNER_ROBOT_USER`, `HETZNER_ROBOT_PASSWORD` |
| OVH | `OVH_APP_KEY`, `OVH_APP_SECRET`, `OVH_CONSUMER_KEY`, `OVH_SUBSIDIARY` (default: US) |
| Vultr | `VULTR_API_KEY` |
| Cherry Servers | `CHERRY_API_KEY` |
| Scaleway | `SCALEWAY_ACCESS_KEY`, `SCALEWAY_SECRET_KEY`, `SCALEWAY_PROJECT_ID`, `SCALEWAY_ZONE` |
| DigitalOcean | `DIGITALOCEAN_TOKEN` |

---

## Provider Comparison

### True Bare-Metal Providers (Full iPXE Support)

| Provider | iPXE Method | Notes |
|----------|-------------|-------|
| **Latitude** | Native iPXE reinstall API | Best developer experience |
| **Vultr** | `ipxe_chain_url` parameter | Simple, direct support |
| **Cherry Servers** | `user_data` chainload | Works well |
| **Scaleway** | `ipxe` OS ID | Uses special OS identifier |
| **Hetzner** | Rescue mode + chainload | Requires SSH after rescue |
| **OVH** | Post-install script | Uses installation API |

### Cloud VM Providers (Limited)

| Provider | Limitation |
|----------|------------|
| **DigitalOcean** | No iPXE boot support - use custom images instead |

---

## Testing Status

| Provider | Unit Tests | Integration Tests |
|----------|------------|-------------------|
| Latitude | ✅ | ✅ |
| Vultr | ✅ | ⚠️ |
| Cherry Servers | ✅ | ⚠️ |
| Scaleway | ✅ | ⚠️ |
| Hetzner | ✅ | ⚠️ |
| OVH | ✅ | ⚠️ |
| DigitalOcean | ✅ | ⚠️ |

Note: Integration tests with wiremock are recommended for all providers.

---

## References

- [Latitude API Docs](https://docs.latitude.sh/reference)
- [Hetzner Robot API](https://robot.hetzner.com/doc/webservice/en.html)
- [OVH API Console](https://eu.api.ovh.com/console/?section=%2Fdedicated%2Fserver)
- [OVH Order Cart Examples](https://github.com/ovh/order-cart-examples)
- [Vultr API](https://www.vultr.com/api/)
- [Cherry Servers API](https://api.cherryservers.com/doc/)
- [Scaleway Elastic Metal API](https://www.scaleway.com/en/developers/api/elastic-metal/)
- [DigitalOcean API](https://docs.digitalocean.com/reference/api/)
