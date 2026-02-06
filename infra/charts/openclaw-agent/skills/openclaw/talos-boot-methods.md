# Talos Linux Boot Methods — Cross-Provider Reference

This skill covers how to install Talos Linux on bare metal servers across all supported providers. Each provider has a different boot method depending on its API capabilities.

## Provider Boot Method Matrix

| Provider | Boot Method | Difficulty | Notes |
|----------|-------------|-----------|-------|
| Latitude | Native iPXE (`os: "ipxe"`) | Easy | Full iPXE URL support in reinstall API |
| Vultr | iPXE chain URL | Easy | **BIOS only, no EFI** — critical limitation |
| Cherry | user_data `#!ipxe` | Easy | iPXE script in user_data field |
| OVH | post_installation_script_link | Medium | API-only, use template `none_64` |
| Scaleway | os_id: `"ipxe"` | Medium | Zone-based API, BMC access available |
| Hetzner | Rescue mode + dd | Hard | No custom iPXE — rescue boot, write image to disk |

## Talos Image Factory PXE

All iPXE-based installs use the Talos Image Factory:

```
https://pxe.factory.talos.dev/pxe/{schematic-id}/{version}/metal-{arch}
```

- `schematic-id`: A hex string identifying the system extensions and kernel args baked into the image.
- `version`: Talos version (e.g., `v1.9.2`).
- `arch`: `amd64` or `arm64`.

Example iPXE URL:
```
https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.9.2/metal-amd64
```

## Required Kernel Parameters

All Talos metal installs require:
- `talos.platform=metal` — tells Talos it's running on bare metal

### Console Parameters (provider-specific)

| Provider | Console Args | Reason |
|----------|-------------|--------|
| Hetzner | `console=tty0` | No hardware serial console; VGA/KVM only |
| OVH | `console=ttyS1,115200n8` | Serial console via IPMI |
| Latitude | `console=ttyS1,115200n8` | Standard serial console |
| Vultr | `console=ttyS0,115200n8` | BIOS-only serial on ttyS0 |
| Scaleway | `console=ttyS1,115200n8` | Serial console via BMC |
| Cherry | `console=ttyS1,115200n8` | Standard serial console |

Kernel params are embedded in the schematic ID via Image Factory. Generate a schematic at `https://factory.talos.dev/` with the appropriate `kernel.arguments`.

## Talos Boot Lifecycle

1. **PXE/iPXE boot** — Server boots the Talos kernel + initramfs from Image Factory URL.
2. **Maintenance mode** — Talos boots into maintenance mode, listening on port 50000 for machine config.
3. **Apply machine config** — Use `talosctl apply-config` to push the machine configuration.
4. **Install to disk** — Talos writes itself to the install disk and reboots from disk.
5. **Running** — Talos boots from disk, joins the Kubernetes cluster.

Key point: Talos does NOT install to disk during PXE boot. It waits in maintenance mode until machine config is applied.

## Boot Method Details

### Easy: Native iPXE (Latitude, Vultr, Cherry)

These providers support iPXE URLs directly in their API:

```
# Generic flow:
1. Create server with standard OS
2. Wait for server to reach "on" status
3. Reinstall with iPXE URL pointing to Image Factory
4. Server PXE boots → Talos maintenance mode
5. Apply machine config → Talos installs to disk
```

The reinstall API call differs per provider but the concept is the same — set the iPXE URL and reboot.

### Medium: API iPXE (OVH, Scaleway)

These providers support iPXE but through indirect API mechanisms:

**OVH**: Uses `post_installation_script_link` in the install API with template `none_64`. The script URL is fetched and executed after the minimal OS boots.

**Scaleway**: Uses `os_id: "ipxe"` in the install API. Scaleway natively supports an iPXE OS option that chains to your URL.

### Hard: Rescue Mode (Hetzner)

Hetzner does NOT support custom iPXE. The install flow is:

```
1. Create server (or use existing)
2. Activate rescue mode (Linux rescue)
3. Hardware reset server → boots into rescue
4. SSH into rescue environment
5. Download Talos raw image from Image Factory
6. dd image to install disk (e.g., /dev/sda or /dev/nvme0n1)
7. Reboot → Talos boots from disk in maintenance mode
8. Apply machine config
```

The rescue mode approach requires SSH automation and is significantly more complex.

Rescue mode dd command:
```bash
curl -fsSL "https://factory.talos.dev/image/{schematic}/{version}/metal-amd64-raw.xz" | xz -d | dd of=/dev/sda bs=4M status=progress
sync
reboot
```

## Universal Fallback: boot-to-talos (kexec)

For any provider where iPXE isn't available or working, use kexec to boot directly into Talos from an existing Linux installation:

**Tool**: `https://github.com/cozystack/boot-to-talos`

```bash
# From any running Linux:
curl -fsSL https://github.com/cozystack/boot-to-talos/releases/latest/download/boot-to-talos-amd64 -o boot-to-talos
chmod +x boot-to-talos
./boot-to-talos --talos-version v1.9.2 --schematic-id <schematic>
```

This downloads the Talos kernel and initramfs, then uses kexec to switch to Talos without a traditional reboot. Works on any provider with SSH access.

## Disk Configuration

Talos installs to a single disk. Use `install.disk` in machine config:

```yaml
machine:
  install:
    disk: /dev/sda          # or /dev/nvme0n1
    wipe: true               # Required for first install
```

### Disk Selectors (preferred over device paths)

```yaml
machine:
  install:
    diskSelector:
      size: ">= 100GB"
      type: ssd
```

### Provider Disk Notes

| Provider | Typical Disk Device | Notes |
|----------|-------------------|-------|
| Hetzner | `/dev/sda` or `/dev/nvme0n1` | Check hardware; may have RAID controller |
| OVH | `/dev/sda` | Software RAID by default; may need to break RAID first |
| Latitude | `/dev/sda` or `/dev/nvme0n1` | Varies by plan |
| Vultr | `/dev/vda` or `/dev/sda` | Depends on plan type |
| Scaleway | `/dev/sda` | Usually single SSD |
| Cherry | `/dev/sda` or `/dev/nvme0n1` | Varies by plan |

## Network Configuration

Talos networking is configured via machine config. Key provider-specific notes:

### Hetzner /32 Routes
Hetzner uses /32 IP assignments with a gateway that's NOT in the same subnet. Talos config must include explicit routes:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        addresses:
          - <server_ip>/32
        routes:
          - network: 0.0.0.0/0
            gateway: <gateway_ip>  # Not in same /32 subnet
    nameservers:
      - 185.12.64.1  # Hetzner DNS
```

### Standard Providers (Latitude, OVH, Vultr, Cherry, Scaleway)
Most providers assign IPs with normal subnet masks. DHCP usually works, but static config is recommended for production:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        dhcp: true     # or static config
```

## Source Code Reference

| File | Purpose |
|------|---------|
| `crates/metal/src/providers/traits.rs` | Provider trait (create, get, reinstall_ipxe, delete, list) |
| `crates/metal/src/providers/factory.rs` | Provider factory (ProviderKind enum, create_provider) |
| `crates/metal/src/providers/<name>/client.rs` | Per-provider API client implementation |
| `crates/metal/src/providers/<name>/models.rs` | Per-provider request/response types |
