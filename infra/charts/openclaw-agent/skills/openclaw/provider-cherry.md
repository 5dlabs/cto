# Provider Skill: Cherry Servers

Cherry Servers bare metal with **iPXE via user_data**. Easy boot method — set `image: "custom_ipxe"` and provide an iPXE script in `user_data`.

## API Reference

- **Base URL**: `https://api.cherryservers.com/v1`
- **Auth**: Bearer token (`Authorization: Bearer <api_key>`)
- **Docs**: https://api.cherryservers.com/doc/
- **Content-Type**: `application/json`

### Authentication

```
Authorization: Bearer <api_key>
```

Credentials: `ProviderConfig.cherry_api_key` and `ProviderConfig.cherry_team_id` (numeric).

## Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/teams/{team_id}/servers` | List servers in team |
| GET | `/servers/{id}` | Get server details |
| POST | `/teams/{team_id}/servers` | Create server |
| DELETE | `/servers/{id}` | Delete server |
| POST | `/servers/{id}/reinstall` | Reinstall server |
| POST | `/servers/{id}/power` | Power actions (reboot, power_on, power_off) |
| GET | `/plans` | List available plans |
| GET | `/plans/{slug}` | Get plan details |
| GET | `/regions` | List regions |

## Provisioning Flow

```
1. POST /teams/{team_id}/servers → Create server (region, plan, hostname, image)
2. Poll GET /servers/{id} → Wait for status="active" or "deployed"
3. POST /servers/{id}/reinstall → Reinstall with custom_ipxe + user_data
4. POST /servers/{id}/power → Reboot (type="reboot")
5. Server boots via iPXE → Talos maintenance mode
6. talosctl apply-config → Talos installs to disk
```

### Create Request

```json
{
  "region": "eu_nord_1",
  "plan": "e3_1240v3",
  "hostname": "talos-cp-1",
  "image": "ubuntu_24_04",
  "ssh_keys": [],
  "user_data": null
}
```

Note: SSH keys use numeric IDs (`i64`), not string UUIDs.

## Talos Boot Method: user_data iPXE

Cherry Servers supports custom iPXE boot via the `user_data` field:

```
POST /servers/{id}/reinstall
{
  "image": "custom_ipxe",
  "hostname": "talos-cp-1",
  "ssh_keys": [],
  "user_data": "#!ipxe\nchain https://pxe.factory.talos.dev/pxe/{schematic}/{version}/metal-amd64"
}
```

Then reboot:
```
POST /servers/{id}/power
{ "type": "reboot" }
```

### How It Works

1. Set `image` to `"custom_ipxe"`
2. Put the iPXE script in `user_data`, starting with `#!ipxe`
3. The `chain` directive loads the Talos Image Factory PXE URL
4. Server boots the Talos kernel and initramfs via iPXE
5. Talos enters maintenance mode
6. Apply machine config with `talosctl apply-config`

### iPXE Script Format

The user_data must be a valid iPXE script:

```
#!ipxe
chain https://pxe.factory.talos.dev/pxe/{schematic}/{version}/metal-amd64
```

You can add kernel arguments too:

```
#!ipxe
set base https://pxe.factory.talos.dev/pxe/{schematic}/{version}
kernel ${base}/vmlinuz talos.platform=metal console=ttyS1,115200n8
initrd ${base}/initramfs.xz
boot
```

## Network Configuration

Cherry provides IP addresses in the server response:

```json
{
  "ip_addresses": [
    { "address": "1.2.3.4", "address_family": 4, "address_type": "primary" },
    { "address": "2001:db8::1", "address_family": 6, "address_type": "primary" }
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

### BGP Support

Cherry Servers supports BGP via project settings. The `Project` model includes a `bgp` field with `enabled` and `local_asn` settings.

## Disk Layout

- Storage specs available in plan details (count, size, type)
- Common devices: `/dev/sda` (SATA), `/dev/nvme0n1` (NVMe)
- Varies by plan

## Console Parameters

```
console=ttyS1,115200n8
```

Standard serial console.

## Server States

| Cherry Status | Mapped Status |
|--------------|--------------|
| `active` / `deployed` | On |
| `pending` / `deploying` | Deploying |
| `powering_off` / `powered_off` / `off` | Off |
| `reinstalling` | Reinstalling |
| `terminating` | Deleting |

## Plans and Pricing

Cherry provides detailed plan information with pricing:

```json
{
  "id": 1,
  "name": "E3-1240v3",
  "slug": "e3_1240v3",
  "category": "baremetal",
  "specs": {
    "cpus": { "cores": 4, "frequency": 3.4, "name": "Intel Xeon E3-1240v3" },
    "memory": { "total": 16 },
    "storage": [{ "count": 2, "size": 250, "type": "SSD" }],
    "nics": { "name": "1 Gbps" },
    "bandwidth": { "name": "1 Gbps unmetered" }
  },
  "pricing": [
    { "unit": "Hourly", "price": 0.11, "currency": "EUR" },
    { "unit": "Monthly", "price": 59.0, "currency": "EUR" }
  ]
}
```

Plans are available in all regions (no per-region stock tracking like Latitude).

## Provider-Specific Gotchas

1. **Team-scoped servers** — Server creation and listing use `/teams/{team_id}/servers`. Individual server operations use `/servers/{id}`.
2. **Numeric IDs** — Server IDs and SSH key IDs are numeric (`i64`), not string UUIDs.
3. **user_data iPXE** — The `#!ipxe` shebang is required at the start of user_data for iPXE scripts.
4. **Pricing in EUR** — All pricing is in EUR by default.
5. **No per-region stock** — All plans are available in all regions (unlike Latitude's stock tracking).
6. **Power actions** — Use `POST /servers/{id}/power` with `type` field (`reboot`, `power_on`, `power_off`).
7. **BGP support** — Available at the project level, useful for anycast networking.

## Source Code

| File | Lines | Description |
|------|-------|-------------|
| `crates/metal/src/providers/cherry/client.rs` | 554 | Team-scoped CRUD, plan listing with pricing |
| `crates/metal/src/providers/cherry/models.rs` | 305 | Server, plan, pricing, region, BGP types |
