# Provider Skill: Vultr

Vultr bare metal servers with **iPXE chain URL support**. Easy iPXE boot, but **BIOS only — no EFI support** for iPXE chain.

## API Reference

- **Base URL**: `https://api.vultr.com/v2`
- **Auth**: Bearer token (`Authorization: Bearer <api_key>`)
- **Docs**: https://www.vultr.com/api/
- **Content-Type**: `application/json`

### Authentication

```
Authorization: Bearer <api_key>
```

Credentials: `ProviderConfig.vultr_api_key`.

## Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/bare-metals` | List all bare metal instances |
| GET | `/bare-metals/{id}` | Get instance details |
| POST | `/bare-metals` | Create bare metal instance |
| DELETE | `/bare-metals/{id}` | Delete instance |
| POST | `/bare-metals/{id}/ipxe` | Set iPXE chain URL |
| POST | `/bare-metals/{id}/reboot` | Reboot instance |
| POST | `/bare-metals/{id}/halt` | Halt instance |
| POST | `/bare-metals/{id}/start` | Start instance |

## Provisioning Flow

```
1. POST /bare-metals → Create instance (region, plan, os_id, label, ssh keys)
2. Poll GET /bare-metals/{id} → Wait for status="active" + power_status="running"
3. POST /bare-metals/{id}/ipxe → Set iPXE chain URL
4. POST /bare-metals/{id}/reboot → Reboot to iPXE
5. Server boots via iPXE → Talos maintenance mode
6. talosctl apply-config → Talos installs to disk
```

### Create Request

```json
{
  "region": "ewr",
  "plan": "vbm-4c-32gb",
  "os_id": 2284,
  "label": "talos-cp-1",
  "sshkey_id": ["<key_id>"],
  "enable_ipv6": true
}
```

### OS ID Mapping

The client maps OS slugs to Vultr numeric OS IDs:

| Slug | OS ID |
|------|-------|
| `ubuntu_24_04` / `ubuntu_24_04_x64_lts` | 2284 |
| `ubuntu_22_04` / `ubuntu_22_04_x64_lts` | 1743 |
| `debian_12` / `debian_12_x64` | 2136 |
| `rocky_9` / `rocky_9_x64` | 1869 |

For Talos iPXE boot, the initial OS doesn't matter — it will be replaced.

## Talos Boot Method: iPXE Chain URL

Vultr supports setting an iPXE chain URL via a dedicated endpoint:

```
POST /bare-metals/{id}/ipxe
{
  "chain_url": "https://pxe.factory.talos.dev/pxe/{schematic}/{version}/metal-amd64"
}
```

Then reboot:
```
POST /bare-metals/{id}/reboot
{}
```

### CRITICAL LIMITATION: BIOS Only

Vultr iPXE chain boot is **BIOS-only**. There is **no EFI iPXE support** for bare metal instances. This means:
- The Talos image must support BIOS boot
- Use `metal-amd64` (not `metal-amd64-efi`) from Image Factory
- Some newer hardware may not support BIOS boot — verify before deploying

## Network Configuration

Vultr provides standard networking with subnet masks:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        dhcp: true
```

Server response includes `main_ip`, `netmask_v4`, `gateway_v4`, `v6_network`, and `mac_address`.

For static config:
```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        addresses:
          - <main_ip>/<prefix>
        routes:
          - network: 0.0.0.0/0
            gateway: <gateway_v4>
```

## Disk Layout

- Disk info available in instance response (`disk` field, e.g., "480 GB SSD")
- Common devices: `/dev/vda` (virtio) or `/dev/sda` (SATA)
- Depends on plan type — check with `lsblk` if unsure

## Console Parameters

```
console=ttyS0,115200n8
```

Vultr uses ttyS0 for serial console (BIOS mode).

## Server States

Vultr uses a two-field status system:

| status | power_status | Mapped Status |
|--------|-------------|--------------|
| `active` | `running` | On |
| `active` | `stopped` | Off |
| `pending` | * | Deploying |
| `resizing` | * | Reinstalling |

Additional field `server_state` tracks internal state (`ok`, `locked`, `installingbooting`).

## Provider-Specific Gotchas

1. **BIOS only** — iPXE chain boot does NOT work with EFI. This is the most critical limitation. Ensure the Talos schematic is configured for BIOS boot.
2. **Numeric OS IDs** — OS selection uses numeric IDs, not slugs. The client maps common slugs to IDs.
3. **Two-field status** — Server status requires checking both `status` and `power_status` fields together.
4. **204 No Content** — DELETE and some POST operations return 204 with no body (handled by client).
5. **Region codes** — Short codes like `ewr` (New Jersey), `lax` (LA), `ord` (Chicago).
6. **Tags support** — Instances support tags for organization.
7. **Pagination** — List endpoints return `meta.total` and `links.next`/`links.prev` for pagination.

## Source Code

| File | Lines | Description |
|------|-------|-------------|
| `crates/metal/src/providers/vultr/client.rs` | 357 | iPXE chain, create, reboot, list |
| `crates/metal/src/providers/vultr/models.rs` | 268 | Instance, plan, region, OS types |
