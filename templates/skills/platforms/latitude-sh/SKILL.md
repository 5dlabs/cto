---
name: latitude-sh
description: Latitude.sh bare metal infrastructure management via API, MCP server, or CLI. Use when provisioning servers, managing VLANs, checking plans/regions, or automating bare metal deployments.
agents: [bolt, rex]
triggers: [latitude, bare metal, server provisioning, vlan, datacenter, mia2]
---

# Latitude.sh Infrastructure

Manage bare metal servers on [Latitude.sh](https://latitude.sh) via direct API, MCP server, or CLI.

## Access Methods

| Method | Use Case | Location |
|--------|----------|----------|
| **Rust API Client** | Programmatic access from Metal/GPU crates | `crates/metal/src/providers/latitude/` |
| **MCP Server** | AI agent workflows (natural language) | `npx latitudesh start` |
| **CLI (`lsh`)** | Interactive debugging | `brew install latitudesh/tools/lsh` |

## Rust API Client (Primary)

The platform's native Latitude client lives in `crates/metal/src/providers/latitude/`.

### Usage

```rust
use metal::providers::latitude::Latitude;

let client = Latitude::new(api_key, project_id)?;

// Create server
let server = client.create_server(CreateServerRequest {
    hostname: "node-1".to_string(),
    plan: "c2-small-x86".to_string(),
    region: "MIA2".to_string(),
    os: "ubuntu_22_04".to_string(),
    ssh_keys: vec!["key-id".to_string()],
}).await?;

// Wait for ready
let server = client.wait_ready(&server.id, 600).await?;

// Reinstall with iPXE (for Talos)
client.reinstall_ipxe(&server.id, ReinstallIpxeRequest {
    hostname: "node-1".to_string(),
    ipxe_url: "https://example.com/talos.ipxe".to_string(),
}).await?;
```

### Available Operations

| Operation | Method |
|-----------|--------|
| Create server | `create_server()` |
| Get server | `get_server(id)` |
| List servers | `list_servers()` |
| Wait for ready | `wait_ready(id, timeout)` |
| Reinstall iPXE | `reinstall_ipxe(id, req)` |
| Delete server | `delete_server(id)` |
| List plans | `list_plans()` |
| List regions | `list_regions()` |
| Create VLAN | `create_virtual_network(site, desc)` |
| Assign to VLAN | `assign_server_to_vlan(vlan_id, server_id)` |
| Delete VLAN | `delete_virtual_network(vlan_id)` |

### Lessons Learned (Baked Into Client)

- **Stuck server detection**: Servers can get stuck in "off"/"deploying" state. Client detects after 10 minutes.
- **Post-ready buffer**: 15-second delay after "on" status before operations (API lag).
- **Reinstall retries**: Automatic retry on "SERVER_BEING_PROVISIONED" errors (up to 6 attempts).

## MCP Server (AI Agents)

For natural language infrastructure management in Cursor, Claude, or VS Code.

### Installation

```bash
# Cursor: Settings → Tools and Integrations → New MCP Server
{
  "mcpServers": {
    "latitudesh": {
      "command": "npx",
      "args": ["latitudesh", "start", "--bearer", "YOUR_API_TOKEN"]
    }
  }
}

# Claude Code CLI
claude mcp add latitudesh npx latitudesh start -- --bearer YOUR_API_TOKEN
```

### Capabilities

- List and inspect servers, projects, SSH keys
- Create or delete bare-metal instances
- Manage networking and automation

### Example Prompts

```
"List all my Latitude servers"
"Create a c2-small-x86 server in MIA2 named test-node"
"What GPU plans are available in Dallas?"
"Delete server srv_xxx"
```

## CLI (`lsh`)

For interactive exploration and debugging.

### Installation

```bash
brew install latitudesh/tools/lsh
# or
curl -fsSL https://cli.latitude.sh/install.sh | sh
```

### Usage

```bash
lsh login API_KEY
lsh servers list
lsh servers create --hostname test --plan c2-small-x86 --site MIA2 --os ubuntu_22_04
lsh servers get srv_xxx
lsh servers delete srv_xxx
lsh plans list --json
```

## API Reference

Base URL: `https://api.latitude.sh`

### Authentication

```
Authorization: Bearer <API_KEY>
```

### Key Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/servers` | GET | List servers |
| `/servers` | POST | Create server |
| `/servers/{id}` | GET | Get server |
| `/servers/{id}` | DELETE | Delete server |
| `/servers/{id}/reinstall` | POST | Reinstall server |
| `/plans` | GET | List plans |
| `/regions` | GET | List regions |
| `/virtual_networks` | GET/POST | VLANs |

### JSON:API Format

All requests use [JSON:API](https://jsonapi.org/) specification:

```json
{
  "data": {
    "type": "servers",
    "attributes": {
      "hostname": "node-1",
      "plan": "c2-small-x86",
      "site": "MIA2"
    }
  }
}
```

## Common Patterns

### Provision Talos Cluster Node

```rust
// 1. Create server with any OS
let server = client.create_server(req).await?;

// 2. Wait for ready
let server = client.wait_ready(&server.id, 600).await?;

// 3. Reinstall with Talos iPXE
client.reinstall_ipxe(&server.id, ReinstallIpxeRequest {
    hostname: server.hostname.clone(),
    ipxe_url: talos_ipxe_url,
}).await?;

// 4. Wait for reinstall to complete
let server = client.wait_ready(&server.id, 600).await?;
```

### Check Plan Availability

```rust
let plans = client.list_plans().await?;
for plan in plans {
    if let Some(regions) = &plan.attributes.regions {
        for region in regions {
            if region.stock_level.as_deref() == Some("high") {
                println!("{}: available in {:?}", 
                    plan.attributes.slug.as_deref().unwrap_or("unknown"),
                    region.locations.as_ref().and_then(|l| l.in_stock.as_ref())
                );
            }
        }
    }
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `LATITUDE_API_KEY` | API key from dashboard |
| `LATITUDE_PROJECT_ID` | Project ID for operations |

## Resources

- [API Reference](https://www.latitude.sh/docs/api-reference/summary)
- [MCP Server](https://github.com/latitudesh/latitudesh-mcp)
- [CLI Source](https://github.com/latitudesh/cli)
- [Terraform Provider](https://registry.terraform.io/providers/latitudesh/latitudesh/latest)
