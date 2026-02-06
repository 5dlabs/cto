# Provider Skill: OVH (OVHcloud)

OVH dedicated servers via the OVHcloud API. Talos installation uses the **post_installation_script_link** mechanism with the `none_64` template.

## API Reference

- **Base URL**: `https://eu.api.ovh.com/1.0`
- **Auth**: OVH API signature (Application Key + Application Secret + Consumer Key)
- **Docs**: https://api.ovh.com/
- **Content-Type**: `application/json`

### Authentication

OVH uses a custom signature scheme. Every request requires four headers:

```
X-Ovh-Application: <application_key>
X-Ovh-Consumer: <consumer_key>
X-Ovh-Timestamp: <unix_timestamp>
X-Ovh-Signature: $1$<sha1_hex>
```

Signature is computed as:
```
SHA1(application_secret + "+" + consumer_key + "+" + method + "+" + url + "+" + body + "+" + timestamp)
```

Prefixed with `$1$`. Credentials come from `ProviderConfig.ovh_app_key`, `ovh_app_secret`, `ovh_consumer_key`.

### Subsidiary

OVH operates regional subsidiaries (US, EU, FR, CA, etc.). Set via `ProviderConfig.ovh_subsidiary` (defaults to "US").

## Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/dedicated/server` | List server names |
| GET | `/dedicated/server/{name}` | Get server details |
| POST | `/dedicated/server/{name}/install/start` | Start OS installation |
| POST | `/dedicated/server/{name}/reboot` | Reboot server |
| POST | `/dedicated/server/{name}/terminate` | Terminate server |
| POST | `/order/cart` | Create shopping cart |
| POST | `/order/cart/{id}/assign` | Assign cart to account |
| GET | `/order/cart/{id}/baremetalServers` | List available plans |
| POST | `/order/cart/{id}/baremetalServers` | Add server to cart |
| POST | `/order/cart/{id}/item/{itemId}/configuration` | Configure server |
| POST | `/order/cart/{id}/checkout` | Checkout cart |

## Provisioning Flow

OVH uses a cart-based ordering system:

```
1. POST /order/cart                           → Create cart (ovhSubsidiary)
2. POST /order/cart/{id}/assign               → Assign to account
3. GET  /order/cart/{id}/baremetalServers      → Find plan
4. POST /order/cart/{id}/baremetalServers      → Add plan to cart
5. POST /order/cart/{id}/item/{id}/configuration  (x3):
   - label: "dedicated_datacenter", value: "<dc>"    (e.g., "gra1", "hil")
   - label: "dedicated_os", value: "none_64.en"      (no OS for Talos)
   - label: "region", value: "<subsidiary>"
6. POST /order/cart/{id}/checkout             → Place order (auto-pay)
7. Wait for server to appear in /dedicated/server
```

Server IDs in OVH look like `ns1234567.ip-1-2-3.eu`.

### Cart Configuration Details

- `dedicated_datacenter`: Datacenter code (e.g., `gra1`, `hil`, `bhs`, `rbx`)
- `dedicated_os`: Use `none_64.en` for Talos (no OS installation)
- `region`: Matches the subsidiary in lowercase
- Duration: Usually `P1M` (monthly) or `P12M` (annual)
- `pricingMode`: Usually `"default"`

## Talos Boot Method: post_installation_script_link

OVH supports custom post-installation scripts via the install API:

```
POST /dedicated/server/{name}/install/start
{
  "templateName": "none_64",
  "details": {
    "customHostname": "<hostname>",
    "postInstallationScriptLink": "<ipxe_url>"
  }
}
```

Then reboot:
```
POST /dedicated/server/{name}/reboot
{ "type": "hardreset" }
```

### How It Works

1. Set `templateName: "none_64"` to bypass standard OS installation
2. The `postInstallationScriptLink` is fetched and executed by the OVH installer
3. Point it at the Talos Image Factory PXE URL
4. Server boots into Talos maintenance mode
5. Apply machine config with `talosctl apply-config`

### iPXE URL Format

```
https://pxe.factory.talos.dev/pxe/{schematic}/{version}/metal-amd64
```

## Network Configuration

OVH provides standard subnet-based IP assignments:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          physical: true
        dhcp: true
```

For static configuration, check the server's IP details via:
```
GET /dedicated/server/{name}
```

The response includes `ip`, `datacenter`, and `reverse` fields.

### IPv6

OVH provides IPv6 via a separate API. Not included in the main server response.

### vRack (Private Networking)

OVH provides vRack for private networking between dedicated servers. Models include `VRack` and `VRackAllowedServices` types. Assign servers to a vRack for internal cluster communication.

## Disk Layout

- OVH servers often have software RAID configured by default
- May need to break RAID before Talos install
- Disk groups available via server hardware specs endpoint
- Common device: `/dev/sda`
- Root device available via `root_device` field in server response

## Console Parameters

```
console=ttyS1,115200n8
```

OVH servers support IPMI serial console on ttyS1.

## Server States

| OVH State | Mapped Status |
|-----------|--------------|
| `ok` | On |
| `installing` | Reinstalling |
| `hacked` / `hackedBlocked` | Off |

## Provider-Specific Gotchas

1. **Cart-based ordering** — Complex multi-step ordering flow (create cart → configure → checkout). Not instant provisioning.
2. **Signature authentication** — Custom SHA1-based signature scheme, not simple API keys. Must compute per-request.
3. **Server naming** — Server IDs are DNS-style names (e.g., `ns1234567.ip-1-2-3.eu`), not numeric IDs.
4. **template none_64** — Must use this template name for iPXE/custom boot, not an empty string.
5. **Sequential server list** — `GET /dedicated/server` returns a list of names; must fetch each individually.
6. **Termination** — Uses `/terminate` endpoint, not DELETE. Termination may have a notice period.
7. **Auto-pay** — Checkout supports `autoPayWithPreferredPaymentMethod` for hands-free ordering.
8. **Link speed** — Available via `link_speed` field (in Mbps).

## Source Code

| File | Lines | Description |
|------|-------|-------------|
| `crates/metal/src/providers/ovh/client.rs` | 701 | Cart-based ordering, signature auth, install API |
| `crates/metal/src/providers/ovh/models.rs` | 363 | Cart, installation, boot, vRack types |
