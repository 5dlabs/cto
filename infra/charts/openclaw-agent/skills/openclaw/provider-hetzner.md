# Provider Skill: Hetzner Robot

Hetzner dedicated servers via the Robot API. This is the **hardest** provider for Talos installation because it does NOT support custom iPXE — you must use rescue mode and dd the image to disk.

## API Reference

- **Base URL**: `https://robot-ws.your-server.de`
- **Auth**: HTTP Basic Auth (`username:password`)
- **Docs**: https://robot.hetzner.com/doc/webservice/en.html
- **Content-Type**: `application/x-www-form-urlencoded` (form data for POST)

### Authentication

```
Authorization: Basic <base64(username:password)>
```

Credentials come from `ProviderConfig.hetzner_user` and `ProviderConfig.hetzner_password`.

## Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/server` | List all servers |
| GET | `/server/{id}` | Get server details |
| POST | `/order/server/transaction` | Order a new server |
| GET | `/order/server/transaction/{id}` | Check order status |
| POST | `/server/{id}/cancellation` | Cancel (delete) server |
| POST | `/boot/{id}/rescue` | Activate rescue mode |
| POST | `/reset/{id}` | Reset server (hardware/software/power) |

### Order Server

```
POST /order/server/transaction
Content-Type: application/x-www-form-urlencoded

product_id=EX44&location=FSN1&authorized_key[]=<fingerprint>
```

Response includes a transaction ID (e.g., `B20150121-344958-251479`). Poll the transaction endpoint until `status: "ready"` and `server_number` is populated.

### Server Cancellation

```
POST /server/{id}/cancellation
cancellation_date=now&cancellation_reason=Automated+via+CTO
```

Use `cancellation_date=now` for immediate cancellation. Check current cancellation status with `GET /server/{id}/cancellation` first.

## Provisioning Flow

```
1. POST /order/server/transaction     → Get transaction_id
2. Poll GET /order/server/transaction/{id}  → Wait for status="ready"
3. GET /server/{server_number}        → Get server details + IP
4. Server is now running with default OS
```

Transaction IDs start with "B" and contain dashes (e.g., `B20150121-344958-251479`). Server numbers are plain integers (e.g., `12345`). The client detects which format to use in `wait_ready()`.

## Talos Boot Method: Rescue Mode + dd

Hetzner does NOT support custom iPXE boot. The only way to install Talos is:

```
1. POST /boot/{server_number}/rescue   → Activate rescue (os="linux")
   Response includes root password for rescue environment
2. POST /reset/{server_number}         → Hardware reset (type="hw")
   Server reboots into rescue mode
3. SSH into rescue environment (root@<server_ip> with rescue password)
4. Download and write Talos image:
   curl -fsSL "https://factory.talos.dev/image/{schematic}/{version}/metal-amd64-raw.xz" \
     | xz -d | dd of=/dev/sda bs=4M status=progress
   sync
5. reboot → Server boots Talos from disk in maintenance mode
6. talosctl apply-config → Install completes
```

### Rescue Mode Details

- Rescue OS options: `"linux"` (default), `"freebsd"`, `"vkvm"`
- Rescue mode response includes a one-time root password
- SSH host key changes between rescue boots — disable strict checking
- Rescue environment has `curl`, `wget`, `dd`, `xz` available

## Network Configuration

**Critical**: Hetzner uses /32 IP assignments. The gateway is NOT in the same subnet as the server IP.

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
            gateway: <gateway_ip>
    nameservers:
      - 185.12.64.1
      - 185.12.64.2
```

The gateway IP is typically the first IP in the subnet (e.g., for `136.243.1.42`, the gateway is often `136.243.1.1`). Check the Hetzner panel or Robot API for the actual gateway.

### IPv6

Hetzner provides a /64 IPv6 subnet via `server_ipv6_net`. Configure with:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        addresses:
          - <ipv6>::1/64
        routes:
          - network: ::/0
            gateway: fe80::1
```

### vSwitch (Private Networking)

Hetzner provides vSwitch for VLAN-based private networking. API models include `VSwitch` and `VSwitchListResponse` types. vSwitches use 802.1Q VLAN tagging.

## Disk Layout

- Dedicated servers use physical disks (HDD or SSD/NVMe)
- Hardware RAID controllers may be present — check with `lsblk` in rescue mode
- Common devices: `/dev/sda` (SATA/SAS), `/dev/nvme0n1` (NVMe)
- For NVMe servers, always target `/dev/nvme0n1`

## Console Parameters

Hetzner servers typically do NOT have a hardware serial console. Use VGA/KVM console:

```
console=tty0
```

Do NOT use `console=ttyS0` or `console=ttyS1` — there is no serial port on most Hetzner hardware.

## Provider-Specific Gotchas

1. **No iPXE support** — Must use rescue mode + dd. This is the most complex boot method.
2. **/32 networking** — Gateway is outside the server's subnet. Requires explicit route configuration.
3. **Transaction ID format** — Order responses return transaction IDs (e.g., `B20150121-...`), not server numbers. Must poll until `server_number` appears.
4. **Form-encoded POST** — Unlike other providers, Hetzner uses `application/x-www-form-urlencoded`, not JSON.
5. **Manual order processing** — If `comment` field is set on order, it triggers manual review (slower).
6. **SSH key format** — Uses SSH key fingerprints, not IDs.
7. **Cancellation** — `cancellation_date=now` for immediate; otherwise uses earliest allowed date.
8. **Server Market** — Cheaper servers available via `/order/server_market/transaction` with numeric product IDs.

## Source Code

| File | Lines | Description |
|------|-------|-------------|
| `crates/metal/src/providers/hetzner/client.rs` | 598 | API client with rescue mode, ordering, cancellation |
| `crates/metal/src/providers/hetzner/models.rs` | 392 | Request/response types including vSwitch, SSH keys |
