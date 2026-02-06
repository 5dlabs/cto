# Provider Skill: Scaleway Elastic Metal

Scaleway Elastic Metal servers with **iPXE boot via os_id**. Medium difficulty — use `os_id: "ipxe"` in the install API.

## API Reference

- **Base URL**: `https://api.scaleway.com/baremetal/v1/zones/{zone}`
- **Auth**: Secret key header (`X-Auth-Token: <secret_key>`)
- **Docs**: https://www.scaleway.com/en/developers/api/elastic-metal/
- **Content-Type**: `application/json`

### Authentication

```
X-Auth-Token: <secret_key>
```

Credentials: `ProviderConfig.scaleway_secret_key`, `scaleway_org_id`, `scaleway_project_id`, `scaleway_zone`.

### Zone-Based API

All endpoints are zone-scoped. The zone is embedded in the URL path:

```
https://api.scaleway.com/baremetal/v1/zones/fr-par-1/servers
https://api.scaleway.com/baremetal/v1/zones/nl-ams-1/servers
```

Common zones: `fr-par-1`, `fr-par-2`, `nl-ams-1`, `nl-ams-2`, `pl-waw-1`, `pl-waw-2`.

## Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/servers` | List servers in zone |
| GET | `/servers/{id}` | Get server details |
| POST | `/servers` | Create server |
| DELETE | `/servers/{id}` | Delete server |
| POST | `/servers/{id}/install` | Install OS (supports iPXE) |
| POST | `/servers/{id}/actions` | Server actions (reboot, poweron, poweroff) |

### BMC (IPMI) Access

Scaleway provides BMC access for out-of-band management:

| Method | Path | Description |
|--------|------|-------------|
| POST | `/servers/{id}/bmc-access` | Start BMC access (requires source IP) |
| GET | `/servers/{id}/bmc-access` | Get BMC credentials |

BMC access returns a URL, login, and password with an expiration time.

## Provisioning Flow

```
1. POST /servers → Create server (offer_id, name, project_id, install config)
2. Poll GET /servers/{id} → Wait for status="ready"
3. POST /servers/{id}/install → Reinstall with os_id="ipxe"
4. POST /servers/{id}/actions → Reboot (action="reboot")
5. Server boots via iPXE → Talos maintenance mode
6. talosctl apply-config → Talos installs to disk
```

### Create Request

```json
{
  "offer_id": "<offer_uuid>",
  "name": "talos-cp-1",
  "project_id": "<project_id>",
  "install": {
    "os_id": "<os_uuid>",
    "hostname": "talos-cp-1",
    "ssh_key_ids": ["<key_id>"]
  }
}
```

## Talos Boot Method: os_id iPXE

Scaleway supports iPXE as a native OS option:

```
POST /servers/{id}/install
{
  "os_id": "ipxe",
  "hostname": "talos-cp-1",
  "ssh_key_ids": []
}
```

Then reboot:
```
POST /servers/{id}/actions
{
  "action": "reboot"
}
```

### How It Works

1. Set `os_id` to `"ipxe"` — Scaleway treats iPXE as an OS option
2. The server boots into an iPXE environment
3. You need to configure the iPXE URL either via:
   - BMC console access to manually enter the chain URL
   - Or pre-configure the server's iPXE boot script via Scaleway's boot configuration
4. Server chain-loads the Talos Image Factory PXE URL
5. Talos enters maintenance mode

### Using BMC for iPXE Configuration

If direct iPXE URL isn't supported via API, use BMC access:

```
POST /servers/{id}/bmc-access
{ "ip": "<your_ip>" }
```

Response provides KVM console URL where you can interact with iPXE prompt.

## Network Configuration

Scaleway provides IPs in the server response via the `ips` array:

```json
{
  "ips": [
    { "id": "...", "address": "1.2.3.4", "version": "IPv4" },
    { "id": "...", "address": "2001:db8::1", "version": "IPv6" }
  ]
}
```

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        dhcp: true
```

## Disk Layout

- Disk specs available in offer details (`disk` field with type and capacity in bytes)
- Common device: `/dev/sda`
- Disk types include SSD and NVMe depending on offer

## Console Parameters

```
console=ttyS1,115200n8
```

Scaleway servers support serial console via BMC.

## Server States

| Scaleway Status | Mapped Status |
|----------------|--------------|
| `ready` | On |
| `delivering` / `ordered` | Deploying |
| `stopped` | Off |
| `resetting` / `installing` | Reinstalling |
| `deleting` | Deleting |

## Server Actions

The actions endpoint supports:
- `poweron` — Power on the server
- `poweroff` — Power off the server
- `reboot` — Reboot the server

## Provider-Specific Gotchas

1. **Zone-scoped API** — All endpoints include the zone in the URL path. You must specify the correct zone.
2. **Offer IDs** — Plans are identified by UUID offer IDs, not human-readable slugs. Query available offers first.
3. **BMC access** — Requires specifying your source IP. Access expires (check `expires_at` field).
4. **SSH key scope** — SSH keys are per-project. Create with `project_id`.
5. **204 No Content** — Some operations return 204 with no body.
6. **OS listing** — Query `/os` endpoint to discover available OS IDs including iPXE.
7. **Org vs Project** — Both `organization_id` and `project_id` are required for provider creation.

## Source Code

| File | Lines | Description |
|------|-------|-------------|
| `crates/metal/src/providers/scaleway/client.rs` | 394 | Zone-based API, iPXE install, BMC access |
| `crates/metal/src/providers/scaleway/models.rs` | 272 | Server, offer, IP, BMC, OS types |
