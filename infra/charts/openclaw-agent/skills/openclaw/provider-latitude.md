# Provider Skill: Latitude.sh

Latitude.sh dedicated servers with **native iPXE support** — the easiest provider for Talos installation. Set `operating_system: "ipxe"` and provide the iPXE URL directly.

## API Reference

- **Base URL**: `https://api.latitude.sh`
- **Auth**: Bearer token (`Authorization: Bearer <api_key>`)
- **Docs**: https://docs.latitude.sh/reference
- **Content-Type**: `application/json`
- **Format**: JSON:API specification (resources wrapped in `{ "data": { "type": "...", "attributes": {...} } }`)

### Authentication

```
Authorization: Bearer <api_key>
```

Credentials: `ProviderConfig.latitude_api_key` and `ProviderConfig.latitude_project_id`.

## Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/servers` | List all servers |
| GET | `/servers/{id}` | Get server details |
| POST | `/servers` | Create server |
| POST | `/servers/{id}/reinstall` | Reinstall server (iPXE supported) |
| DELETE | `/servers/{id}` | Delete server |
| GET | `/plans` | List available plans |
| GET | `/regions` | List available regions |
| GET | `/virtual_networks` | List VLANs |
| POST | `/virtual_networks` | Create VLAN |
| POST | `/virtual_networks/{id}/assignments` | Assign server to VLAN |

## Provisioning Flow

```
1. POST /servers → Create server (project, plan, site, os, hostname, ssh_keys)
2. Poll GET /servers/{id} → Wait for status="on"
   - 15s buffer after "on" before proceeding (API may not be fully ready)
3. POST /servers/{id}/reinstall → Reinstall with iPXE
4. Server PXE boots → Talos maintenance mode
5. talosctl apply-config → Talos installs to disk
```

### Create Server Request

```json
{
  "data": {
    "type": "servers",
    "attributes": {
      "project": "<project_id>",
      "plan": "c2-small-x86",
      "site": "MIA2",
      "operating_system": "ubuntu_24_04_x64_lts",
      "hostname": "talos-cp-1",
      "ssh_keys": ["<key_id>"]
    }
  }
}
```

## Talos Boot Method: Native iPXE

Latitude natively supports iPXE as an operating system option:

```json
POST /servers/{id}/reinstall
{
  "data": {
    "type": "reinstalls",
    "attributes": {
      "operating_system": "ipxe",
      "hostname": "talos-cp-1",
      "ipxe": "https://pxe.factory.talos.dev/pxe/{schematic}/{version}/metal-amd64"
    }
  }
}
```

Key fields:
- `operating_system`: Must be `"ipxe"` (literal string)
- `ipxe`: The iPXE URL to chain-load (Talos Image Factory PXE URL)
- `hostname`: Server hostname

### Reinstall Retry Logic

The Latitude API may return `SERVER_BEING_PROVISIONED` errors even after the server shows status "on". The client retries up to 6 times with 30s delays between attempts.

## Network Configuration

Latitude provides standard IP assignments:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        dhcp: true
```

Server response includes `primary_ipv4` and `primary_ipv6` fields.

### Virtual Networks (VLANs)

Latitude supports private networking via VLANs. The client has full VLAN support:

1. **Create VLAN**: `POST /virtual_networks` with `site` and `description`
2. **Assign server**: `POST /virtual_networks/{id}/assignments` with `server_id`
3. **List assignments**: `GET /virtual_networks/{id}/assignments`

VLAN assignment returns a `vid` (802.1Q tag) and private `ip`. Configure in Talos:

```yaml
machine:
  network:
    interfaces:
      - interface: eth0.{vid}
        addresses:
          - <private_ip>/24
```

**Fallback endpoints**: Some accounts don't expose nested VLAN assignment endpoints. The client automatically falls back through:
1. `/virtual_networks/{id}/assignments`
2. `/virtual_network_assignments` (top-level)
3. `/private_networks/{id}/assignments` (legacy)

## Disk Layout

- Varies by plan — check `specs.disk` in server response
- Common devices: `/dev/sda` (SATA), `/dev/nvme0n1` (NVMe)
- Use disk selectors in machine config for portability

## Console Parameters

```
console=ttyS1,115200n8
```

Standard serial console.

## Server States

| Latitude Status | Mapped Status |
|----------------|--------------|
| `deploying` | Deploying |
| `on` | On |
| `off` | Off |
| `disk_erasing` | DiskErasing |
| `reinstalling` | Reinstalling |
| `deleting` | Deleting |

## Provider-Specific Gotchas

1. **Stuck servers** — Servers can get permanently stuck in "off" or "deploying" state (learned Dec 2024). After 10 minutes in the same state, the server should be deleted and recreated. The client returns a `ServerStuck` error.
2. **Post-ready buffer** — 15-second wait after status becomes "on" before operations. The API needs time to fully register the server.
3. **Reinstall retries** — `SERVER_BEING_PROVISIONED` errors are retried up to 6 times with 30s delays.
4. **JSON:API format** — All requests/responses use JSON:API wrapping (`{ "data": { "type": "...", "attributes": {...} } }`).
5. **VLAN endpoint instability** — Some accounts lack certain VLAN assignment endpoints. Client has 3-level fallback.
6. **Plan slug format** — Plans use slugs like `c2-small-x86`, not numeric IDs.
7. **Site codes** — Regions use short codes like `MIA2`, `DAL`, `LAX`, `NYC`.

## Source Code

| File | Lines | Description |
|------|-------|-------------|
| `crates/metal/src/providers/latitude/client.rs` | 695 | Full CRUD + VLAN management + stuck detection |
| `crates/metal/src/providers/latitude/models.rs` | 496 | JSON:API types, plans, regions, VLANs |
