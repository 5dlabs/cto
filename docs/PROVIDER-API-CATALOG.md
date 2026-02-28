# CTO Provider API Catalog

This document catalogs all API calls across supported infrastructure providers in the CTO platform.

## Table of Contents

1. [Provider Trait (Common Interface)](#provider-trait-common-interface)
2. [Latitude.sh](#latitudesh)
3. [Hetzner](#hetzner)
4. [OVH](#ovh)
5. [Vultr](#vultr)
6. [Scaleway](#scaleway)
7. [Cherry Servers](#cherry-servers)
8. [DigitalOcean](#digitalocean)
9. [Servers.com](#serverscom)
10. [PhoenixNAP](#phoenixnap)
11. [i3D.net (FlexMetal)](#i3dnet-flexmetal)
12. [On-Premises](#on-premises)
13. [Latitude GPU VMs](#latitude-gpu-vms)

---

## Provider Trait (Common Interface)

All providers implement a common `Provider` trait with these operations:

| Method | Description |
|--------|-------------|
| `create_server(req)` | Provision a new bare-metal server |
| `get_server(id)` | Get server details by ID |
| `wait_ready(id, timeout)` | Poll until server reaches "on" status |
| `reinstall_ipxe(id, req)` | Reinstall server via iPXE boot |
| `delete_server(id)` | Delete/terminate a server |
| `list_servers()` | List all servers |

### Common Data Types

```rust
struct Server {
    id: String,           // Unique server identifier
    hostname: String,     // Server hostname
    status: ServerStatus, // Deploying, On, Off, Reinstalling, Deleting, Unknown
    ipv4: Option<String>, // Primary IPv4 address
    ipv6: Option<String>, // Primary IPv6 address
    plan: String,         // Server plan/type
    region: String,       // Region/site location
    created_at: Option<DateTime<Utc>>,
}

struct CreateServerRequest {
    hostname: String,     // Hostname for the server
    plan: String,         // Plan/instance type (e.g., "c2-small-x86")
    region: String,       // Region/site to deploy in
    os: String,           // Operating system slug
    ssh_keys: Vec<String>, // SSH key IDs
}

struct ReinstallIpxeRequest {
    hostname: String,     // Hostname for the server
    ipxe_url: String,     // URL to the iPXE script
}
```

---

## Latitude.sh

**Base URL:** `https://api.latitude.sh`
**Auth:** Bearer token (`Authorization: Bearer {api_key}`)
**API Style:** JSON:API

### Bare Metal Server APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/servers` | POST | Create a new server |
| `/servers/{id}` | GET | Get server details |
| `/servers` | GET | List all servers |
| `/servers/{id}/reinstall` | POST | Reinstall with iPXE |
| `/servers/{id}` | DELETE | Delete server |

### Create Server Request

```json
POST /servers
{
  "data": {
    "type": "servers",
    "attributes": {
      "project": "{project_id}",
      "plan": "c2-small-x86",
      "site": "MIA2",
      "operating_system": "ubuntu_24_04_x64_lts",
      "hostname": "my-server",
      "ssh_keys": ["key-id-1", "key-id-2"]
    }
  }
}
```

### Reinstall Server Request (iPXE)

```json
POST /servers/{id}/reinstall
{
  "data": {
    "type": "reinstalls",
    "attributes": {
      "operating_system": "ipxe",
      "hostname": "my-server",
      "ipxe": "https://example.com/boot.ipxe"
    }
  }
}
```

### Plan & Region APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/plans` | GET | List available plans with specs and pricing |
| `/regions` | GET | List available regions |

### Virtual Network (VLAN) APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/virtual_networks` | POST | Create VLAN |
| `/virtual_networks/{id}` | GET | Get VLAN details |
| `/virtual_networks` | GET | List VLANs |
| `/virtual_networks/{id}` | DELETE | Delete VLAN |
| `/virtual_networks/{vlan_id}/assignments` | POST | Assign server to VLAN |
| `/virtual_networks/{vlan_id}/assignments` | GET | List VLAN assignments |

**Fallback endpoints (some accounts):**
- `/virtual_network_assignments` (POST/GET)
- `/private_networks/{vlan_id}/assignments` (POST/GET)

### Create VLAN Request

```json
POST /virtual_networks
{
  "data": {
    "type": "virtual_networks",
    "attributes": {
      "description": "cluster-internal",
      "project": "{project_id}",
      "site": "MIA2"
    }
  }
}
```

### Assign Server to VLAN

```json
POST /virtual_networks/{vlan_id}/assignments
{
  "data": {
    "type": "virtual_network_assignment",
    "attributes": {
      "server_id": "{server_id}"
    }
  }
}
```

---

## Hetzner

**Base URL:** `https://robot-ws.your-server.de`
**Auth:** HTTP Basic Auth (username/password)
**Docs:** https://robot.hetzner.com/doc/webservice/en.html

### Server APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/server` | GET | List all servers |
| `/server/{server_number}` | GET | Get server details |
| `/order/server/transaction` | POST | Order new server |
| `/order/server/transaction/{id}` | GET | Get order status |
| `/boot/{server_number}/rescue` | POST | Activate rescue mode |
| `/reset/{server_number}` | POST | Hardware reset |
| `/server/{server_number}/cancellation` | GET | Get cancellation status |
| `/server/{server_number}/cancellation` | POST | Submit cancellation |

### Order Server Request (Form-encoded)

```
POST /order/server/transaction
product_id=EX44
location=FSN1
authorized_key[]=abc123
dist=ubuntu-2204
lang=en
```

### Rescue Mode Request

```
POST /boot/{server_number}/rescue
os=linux
authorized_key[]=abc123
```

### Reset Request

```
POST /reset/{server_number}
type=hw
```

### Cancellation Request

```
POST /server/{server_number}/cancellation
cancellation_date=now
cancellation_reason=Automated cancellation via CTO Metal API
reserve_location=false
```

---

## OVH

**Base URL:** `https://eu.api.ovh.com/1.0` (or `us.api.ovh.com` for US)
**Auth:** OVH signature (X-Ovh-Application, X-Ovh-Consumer, X-Ovh-Timestamp, X-Ovh-Signature)
**Docs:** https://api.ovh.com/

### Authentication

OVH uses signature-based authentication:

```
$1$sha1_hex(application_secret + consumer_key + method + url + body + timestamp)
```

Headers required:
- `X-Ovh-Application`: Application Key
- `X-Ovh-Consumer`: Consumer Key
- `X-Ovh-Timestamp`: Unix timestamp
- `X-Ovh-Signature`: Generated signature

### Server APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/dedicated/server` | GET | List server names |
| `/dedicated/server/{serviceName}` | GET | Get server details |
| `/dedicated/server/{serviceName}/install/start` | POST | Start installation |
| `/dedicated/server/{serviceName}/reboot` | POST | Reboot server |
| `/dedicated/server/{serviceName}/terminate` | POST | Terminate server |

### Cart/Ordering APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/order/cart` | POST | Create cart |
| `/order/cart/{cartId}/assign` | POST | Assign cart to account |
| `/order/cart/{cartId}/baremetalServers` | GET | List available plans |
| `/order/cart/{cartId}/baremetalServers` | POST | Add server to cart |
| `/order/cart/{cartId}/item/{itemId}/configuration` | POST | Configure item |
| `/order/cart/{cartId}/checkout` | POST | Checkout cart |

### Create Cart Request

```json
POST /order/cart
{
  "ovhSubsidiary": "US",
  "description": "cto-metal-myserver"
}
```

### Add Server to Cart

```json
POST /order/cart/{cartId}/baremetalServers
{
  "planCode": "24rise01-us",
  "duration": "P1M",
  "pricingMode": "default",
  "quantity": 1
}
```

### Configure Server

```json
POST /order/cart/{cartId}/item/{itemId}/configuration
{
  "label": "dedicated_datacenter",
  "value": "hil"
}
```

### Checkout

```json
POST /order/cart/{cartId}/checkout
{
  "autoPayWithPreferredPaymentMethod": true,
  "waiveRetractationPeriod": false
}
```

### Installation Request

```json
POST /dedicated/server/{serviceName}/install/start
{
  "templateName": "none_64",
  "details": {
    "customHostname": "my-server",
    "postInstallationScriptLink": "https://example.com/boot.ipxe"
  }
}
```

---

## Vultr

**Base URL:** `https://api.vultr.com/v2`
**Auth:** Bearer token (`Authorization: Bearer {api_key}`)
**Docs:** https://www.vultr.com/api/

### Bare Metal APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/bare-metals` | POST | Create bare-metal instance |
| `/bare-metals/{id}` | GET | Get instance details |
| `/bare-metals` | GET | List all instances |
| `/bare-metals/{id}` | DELETE | Delete instance |
| `/bare-metals/{id}/ipxe` | POST | Set iPXE chain URL |
| `/bare-metals/{id}/reboot` | POST | Reboot instance |

### Create Bare Metal Request

```json
POST /bare-metals
{
  "region": "ewr",
  "plan": "vbm-4c-32gb",
  "os_id": 1743,
  "label": "my-server",
  "sshkey_id": ["key-1", "key-2"],
  "enable_ipv6": true
}
```

### Set iPXE Chain URL

```json
POST /bare-metals/{id}/ipxe
{
  "chain_url": "https://example.com/boot.ipxe"
}
```

### Reboot Request

```json
POST /bare-metals/{id}/reboot
{}
```

### OS ID Mappings

| OS Slug | OS ID |
|---------|-------|
| `ubuntu_24_04` | 2284 |
| `ubuntu_22_04` | 1743 |
| `debian_12` | 2136 |
| `rocky_9` | 1869 |

---

## Scaleway

**Base URL:** `https://api.scaleway.com/baremetal/v1/zones/{zone}`
**Auth:** X-Auth-Token header
**Docs:** https://www.scaleway.com/en/developers/api/elastic-metal/

### Server APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/servers` | POST | Create server |
| `/servers/{id}` | GET | Get server details |
| `/servers` | GET | List servers |
| `/servers/{id}` | DELETE | Delete server |
| `/servers/{id}/install` | POST | Reinstall server |
| `/servers/{id}/actions` | POST | Perform action (reboot, etc.) |

### Create Server Request

```json
POST /servers
{
  "offer_id": "PRO-6-S-SSD",
  "name": "my-server",
  "project_id": "{project_id}",
  "tags": [],
  "install": {
    "os_id": "ubuntu_22.04",
    "hostname": "my-server",
    "ssh_key_ids": ["key-1", "key-2"]
  }
}
```

### Reinstall Request

```json
POST /servers/{id}/install
{
  "os_id": "ipxe",
  "hostname": "my-server",
  "ssh_key_ids": []
}
```

### Action Request

```json
POST /servers/{id}/actions
{
  "action": "reboot"
}
```

---

## Cherry Servers

**Base URL:** `https://api.cherryservers.com/v1`
**Auth:** Bearer token (`Authorization: Bearer {api_key}`)
**Docs:** https://api.cherryservers.com/doc/

### Server APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/teams/{team_id}/servers` | POST | Create server |
| `/servers/{id}` | GET | Get server details |
| `/teams/{team_id}/servers` | GET | List servers |
| `/servers/{id}` | DELETE | Delete server |
| `/servers/{id}/reinstall` | POST | Reinstall server |
| `/servers/{id}/power` | POST | Power action |

### Create Server Request

```json
POST /teams/{team_id}/servers
{
  "region": "eu_nord_1",
  "plan": "e3_1240v3",
  "hostname": "my-server",
  "image": "ubuntu_22_04"
}
```

### Reinstall Request (iPXE)

```json
POST /servers/{id}/reinstall
{
  "image": "custom_ipxe",
  "hostname": "my-server",
  "ssh_keys": [],
  "user_data": "#!ipxe\nchain https://example.com/boot.ipxe"
}
```

### Power Action Request

```json
POST /servers/{id}/power
{
  "type": "reboot"
}
```

---

## DigitalOcean

**Base URL:** `https://api.digitalocean.com/v2`
**Auth:** Bearer token (`Authorization: Bearer {api_token}`)
**Docs:** https://docs.digitalocean.com/reference/api/

**Note:** DigitalOcean Droplets do NOT support direct iPXE boot. Use custom images or cloud-init for bootstrapping.

### Droplet APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/droplets` | POST | Create droplet |
| `/droplets/{id}` | GET | Get droplet details |
| `/droplets` | GET | List droplets |
| `/droplets/{id}` | DELETE | Delete droplet |

### Create Droplet Request

```json
POST /droplets
{
  "name": "my-server",
  "region": "nyc1",
  "size": "c-4",
  "image": "ubuntu-22-04-x64",
  "ssh_keys": ["key-1", "key-2"],
  "ipv6": true,
  "monitoring": true
}
```

---

## Servers.com

**Partnership** — Bare-metal and cloud provider. API details and `Provider` trait integration TBD.

- **Regions:** Global
- **Docs:** https://www.servers.com/

---

## PhoenixNAP

**Partnership** — Bare-metal and hybrid cloud with global data centers. API details and `Provider` trait integration TBD.

- **Regions:** Americas, Europe, Asia
- **Docs:** https://www.phoenixnap.com/
- **Note:** Referenced in trading cluster architecture (ArgoCD external destination).

---

## i3D.net (FlexMetal)

**API:** Yes — FlexMetal API (BETA). Base URL: `https://api.i3d.net`, auth: `PRIVATE-TOKEN` header.

**Compatibility with CTO:** Yes. Create, list, get, delete servers via API; power_on / power_off / reboot; locations and instance types per location; **Talos Omni** supported as OS slug (`talos-omni-1110`). Initial provision can use Talos image directly (no custom iPXE required for first install). Reinstall-with-custom-iPXE flow not documented; may require delete + create or contact i3D for reinstall API.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v3/flexMetal/servers` | POST | Create server (name, location, instanceType, os.slug, sshKey, postInstallScript, tags, contractId, overflow) |
| `/v3/flexMetal/servers` | GET | List servers (filter by status, tag; pagination via RANGED-DATA header) |
| `/v3/flexMetal/servers/{uuid}` | GET | Get server details |
| `/v3/flexMetal/servers/{uuid}` | DELETE | Release server |
| `/v3/flexMetal/location/` | GET | List locations |
| `/v3/flexMetal/plans` | GET | Instance types per location |
| `/v3/operatingsystem` | GET | Supported OS (includes Talos versions) |
| `/server/{uuid}/commands` | POST | power_on, power_off, reboot |

- **Regions:** Americas, Europe, Asia (e.g. EU: Rotterdam; full list via location API)
- **Docs:** [FlexMetal API](https://docs.i3d.net/compute/flexmetal/api), [Product docs](https://www.i3d.net/resources/documentation/)
- **Dedicated Bare Metal** (non-FlexMetal): monthly, sales-led only — not API-provisioned.

---

## On-Premises

**No API** - Uses local YAML inventory file at `~/.cto/onprem-inventory.yaml`
**Control:** IPMI/BMC via `ipmitool`

### IPMI Commands

| Command | Description |
|---------|-------------|
| `power on` | Power on server |
| `power off` | Power off server |
| `power reset` | Hardware reset |
| `power cycle` | Power cycle |
| `power status` | Get power status |
| `chassis bootdev pxe options=efiboot` | Set PXE boot |

### Example IPMI Invocation

```bash
ipmitool -I lanplus -H {bmc_address} -p 623 -U {username} -P {password} power reset
```

### Inventory File Structure

```yaml
servers:
  - id: "server-001"
    hostname: "node1.local"
    status: "ready"  # ready, provisioning, powered_off, maintenance, decommissioning
    ipv4: "192.168.1.100"
    plan: "Dell PowerEdge R640"
    location: "Rack 42, DC1"
    bmc:
      address: "192.168.1.200"
      port: 623
      username: "admin"
      password: "secret"
      type: "ipmi"
    tags:
      - kubernetes
      - production
updated_at: "2024-01-15T00:00:00Z"
```

---

## Latitude GPU VMs

**Base URL:** `https://api.latitude.sh`
**Auth:** Bearer token
**Note:** Separate API for GPU virtual machines (different from bare-metal)

### GPU VM APIs

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/plans/virtual_machines` | GET | List GPU VM plans |
| `/virtual_machines` | POST | Create GPU VM |
| `/virtual_machines/{id}` | GET | Get VM details |
| `/virtual_machines` | GET | List VMs (add `?extra_fields[virtual_machines]=credentials` for SSH info) |
| `/virtual_machines/{id}` | DELETE | Delete VM |
| `/virtual_machines/{id}/actions` | POST | VM action (start, stop, reboot) |

### Create GPU VM Request

```json
POST /virtual_machines
{
  "data": {
    "type": "virtual_machines",
    "attributes": {
      "name": "my-gpu-vm",
      "plan": "vm.h100.small",
      "ssh_keys": ["key-id-1"],
      "project": "{project_id}"
    }
  }
}
```

### VM Action Request

```json
POST /virtual_machines/{id}/actions
{
  "id": "{vm_id}",
  "type": "virtual_machines",
  "attributes": {
    "action": "reboot"
  }
}
```

### GPU Plan Examples

| Plan ID | GPU | vCPUs | RAM | Storage |
|---------|-----|-------|-----|---------|
| `vm.h100.small` | H100 | 12 | 96GB | 512GB NVMe |
| `vm.l40s.small` | L40S | 16 | 128GB | 1TB NVMe |
| `vm.rtx6000.small` | RTX 6000 Pro | 8 | 64GB | 512GB NVMe |

### VM Status Values

| Status | Description |
|--------|-------------|
| `Scheduling` | VM is being scheduled |
| `Scheduled` | VM is scheduled |
| `Starting` | VM is starting |
| `Configuring network` | Network being configured |
| `Running` | VM is running and ready |
| `Stopped` | VM is stopped |

---

## Error Handling

All providers return consistent error types:

| Error Type | Description |
|------------|-------------|
| `Http` | HTTP request failed |
| `Api { status, message }` | API returned error response |
| `NotFound` | Server not found |
| `Timeout` | Operation timed out |
| `ServerStuck` | Server stuck in non-responsive state (Latitude-specific) |
| `Config` | Invalid configuration |
| `Serialization` | JSON parsing error |

### Retry Strategies

- **Latitude:** 6 retries with 30s delay for "SERVER_BEING_PROVISIONED" errors
- **All providers:** Configurable timeout for `wait_ready()` polling

---

## Configuration Reference

### Environment Variables

| Provider | Required Variables |
|----------|-------------------|
| Latitude | `LATITUDE_API_KEY`, `LATITUDE_PROJECT_ID` |
| Hetzner | `HETZNER_ROBOT_USER`, `HETZNER_ROBOT_PASSWORD` |
| OVH | `OVH_APPLICATION_KEY`, `OVH_APPLICATION_SECRET`, `OVH_CONSUMER_KEY` |
| Vultr | `VULTR_API_KEY` |
| Scaleway | `SCALEWAY_SECRET_KEY`, `SCALEWAY_ORGANIZATION_ID`, `SCALEWAY_PROJECT_ID` |
| Cherry | `CHERRY_API_KEY`, `CHERRY_TEAM_ID` |
| DigitalOcean | `DIGITALOCEAN_TOKEN` |
| On-Prem | Local inventory file |

---

*Generated: 2026-01-31*
*Source: crates/metal/src/providers/, crates/gpu/src/providers/*
