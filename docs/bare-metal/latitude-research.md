# Latitude.sh Bare Metal Research

> Research and analysis for CTO Platform bare metal strategy

## Overview

[Latitude.sh](https://www.latitude.sh/) is a bare metal cloud provider offering dedicated servers, virtual machines, storage, and networking capabilities globally. They provide both a comprehensive REST API and a Go-based CLI (`lsh`).

## Key Resources

- **Documentation**: https://www.latitude.sh/docs
- **API Reference**: https://www.latitude.sh/docs/api-reference/summary
- **CLI Repository**: https://github.com/latitudesh/cli (cloned to `./latitude-cli/`)
- **Go SDK**: `github.com/latitudesh/latitudesh-go-sdk`

---

## API Capabilities

### Authentication

- Bearer token authentication via API keys
- Create keys at: Settings & Billing → API Keys
- Header format: `Authorization: Bearer YOUR_API_KEY_HERE`

```bash
curl -s https://api.latitude.sh/user/profile \
  -H "Authorization: Bearer YOUR_API_KEY_HERE"
```

### Core Resources

#### 1. Servers (Bare Metal)

**Endpoints:**
- `GET /servers` - List all servers
- `POST /servers` - Deploy server
- `GET /servers/{id}` - Get server details
- `DELETE /servers/{id}` - Remove server
- `PATCH /servers/{id}` - Update server
- `POST /servers/{id}/actions` - Run server action (reboot, power_on, power_off)
- `POST /servers/{id}/reinstall` - Reinstall server OS
- `POST /servers/{id}/rescue` - Start/exit rescue mode
- `POST /servers/{id}/ipmi` - Generate IPMI credentials

**Server Creation Example (Go SDK):**
```go
res, err := s.Servers.Create(ctx, operations.CreateServerServersRequestBody{
    Data: &operations.CreateServerServersData{
        Type: operations.CreateServerServersTypeServers,
        Attributes: &operations.CreateServerServersAttributes{
            Project:         latitudeshgosdk.Pointer("proj_A05EdQ50dvKYQ"),
            Plan:            operations.CreateServerPlanC2SmallX86.ToPointer(),
            Site:            operations.CreateServerSiteAsh.ToPointer(),
            OperatingSystem: operations.CreateServerOperatingSystemUbuntu2204X64Lts.ToPointer(),
            Hostname:        latitudeshgosdk.Pointer("BRC1"),
        },
    },
})
```

**Server Response Structure:**
```json
{
  "data": {
    "type": "servers",
    "id": "sv_KXgRdRyoOv9k5",
    "attributes": {
      "hostname": "BRC1",
      "label": "868155NODEVO",
      "role": "Bare Metal",
      "status": "off",
      "primary_ipv4": "29.227.250.123",
      "primary_ipv6": "f5:1442:770b:d762:7b55:1583:8420:630b",
      "specs": {
        "cpu": "Xeon E-2186G CPU @ 3.80GHz (6 cores)",
        "disk": "500 GB SSD",
        "ram": "32 GB",
        "nic": ""
      },
      "plan": {
        "id": "plan_VE1Wd3aXDXnZJ",
        "name": "c2.small.x86",
        "slug": "c2-small-x86",
        "billing": "hourly"
      },
      "interfaces": [
        {"role": "ipmi", "name": "IPMI", "mac_address": "00:11:22:33:44:55"},
        {"role": "internal", "name": "PXE", "mac_address": "66:77:88:99:aa:bb"}
      ]
    }
  }
}
```

#### 2. Plans

**Endpoints:**
- `GET /plans` - List all plans
- `GET /plans/{id}` - Get plan details
- `GET /plans/bandwidth` - List bandwidth plans
- `GET /plans/storage` - List storage plans
- `GET /plans/virtual_machines` - List VM plans

**Filter Parameters:**
- `filter[name]` - Plan name
- `filter[slug]` - Plan slug
- `filter[location]` - Site location
- `filter[stock_level]` - Stock level (unavailable, low, medium, high, unique)
- `filter[in_stock]` - Boolean stock availability
- `filter[gpu]` - Filter by GPU availability
- `filter[ram]` - RAM size in GB (supports [eql], [gte], [lte])
- `filter[disk]` - Disk size in GB (supports [eql], [gte], [lte])

**Plan Response (includes GPU specs!):**
```json
{
  "data": [{
    "id": "plan_1Qkm7dXzD8nZV",
    "type": "plans",
    "attributes": {
      "slug": "g3-l40s-medium-31",
      "name": "g3.l40s.medium-31",
      "specs": {
        "cpu": {"type": "E-2276G", "clock": 3.8, "cores": 6, "count": 1},
        "memory": {"total": 32},
        "drives": [{"count": 1, "size": "3.8TB", "type": "SSD"}],
        "nics": [{"count": 1, "type": "10 Gbps"}],
        "gpu": {"count": 4, "type": "NVIDIA H100"}
      },
      "regions": [{
        "name": "Brazil",
        "locations": {"available": ["SAO"], "in_stock": ["SAO"]},
        "stock_level": "medium",
        "pricing": {
          "USD": {"hour": 10, "month": 50, "year": 100},
          "BRL": {"hour": 53, "month": 108.5, "year": 205}
        }
      }]
    }
  }]
}
```

#### 3. Virtual Machines

**Endpoints:**
- `GET /virtual_machines` - List VMs
- `POST /virtual_machines` - Create VM
- `GET /virtual_machines/{id}` - Get VM details
- `DELETE /virtual_machines/{id}` - Destroy VM
- `POST /virtual_machines/{id}/actions` - Run VM action

**VM Response (includes GPU!):**
```json
{
  "data": {
    "id": "vm_mw49QDB5qagKb",
    "type": "virtual_machines",
    "attributes": {
      "name": "my-new-vm",
      "status": "Starting",
      "specs": {
        "vcpu": 16,
        "ram": "128 GB",
        "storage": "100 GB",
        "nic": "1 x 1 Gbps",
        "gpu": "1 x NVIDIA H100 Tensor Core GPU"
      }
    }
  }
}
```

#### 4. Regions

**Endpoints:**
- `GET /regions` - List all regions
- `GET /regions/{id}` - Get region details

**Region Response:**
```json
{
  "data": [{
    "id": "loc_k0RyqvNvqW36X",
    "type": "regions",
    "attributes": {
      "name": "São Paulo",
      "slug": "sao-paulo",
      "facility": "SAO",
      "country": {"name": "Brazil", "slug": "brazil"},
      "type": "core"
    }
  }]
}
```

#### 5. Storage (Block Storage)

**Volume Endpoints:**
- `GET /storage/volumes` - List volumes
- `POST /storage/volumes` - Create volume
- `GET /storage/volumes/{id}` - Get volume
- `DELETE /storage/volumes/{id}` - Delete volume
- `POST /storage/volumes/{id}/mount` - Mount volume

**Filesystem Endpoints:**
- `GET /storage/filesystems` - List filesystems
- `POST /storage/filesystems` - Create filesystem
- `DELETE /storage/filesystems/{id}` - Delete filesystem
- `PATCH /storage/filesystems/{id}` - Update filesystem

**Volume Features:**
- NVMe-oF based storage
- Uses NQN (NVMe Qualified Name) identifiers
- Requires `nvme-cli` for mounting
- Connected via `nvme_tcp` kernel module

#### 6. Private Networks (VLANs)

**Endpoints:**
- `GET /virtual_networks` - List VLANs
- `POST /virtual_networks` - Create VLAN
- `GET /virtual_networks/{id}` - Get VLAN details
- `DELETE /virtual_networks/{id}` - Delete VLAN
- `PATCH /virtual_networks/{id}` - Update VLAN
- `GET /virtual_networks/assignments` - List server assignments
- `POST /virtual_networks/assignments` - Assign server to VLAN
- `DELETE /virtual_networks/assignments/{id}` - Remove assignment

**VLAN Response:**
```json
{
  "data": {
    "id": "vlan_3YjJOLQjdvZ87",
    "type": "virtual_networks",
    "attributes": {
      "vid": 2000,
      "name": "MIA-2000",
      "description": "Miami VLAN",
      "site": "MIA"
    }
  }
}
```

#### 7. Additional Resources

- **Projects**: Logical grouping for resources
- **SSH Keys**: Managed at project or account level
- **Firewalls**: Network security rules
- **IP Addresses**: IPv4/IPv6 management
- **Teams**: Multi-user access management
- **Tags**: Resource labeling
- **Events**: Activity logging
- **Traffic/Bandwidth**: Usage monitoring

---

## CLI (`lsh`) Architecture

### Installation

```bash
# Homebrew
brew install latitudesh/tools/lsh

# Script
curl -fsSL https://cli.latitude.sh/install.sh | sh
```

### Authentication

```bash
lsh login <API_KEY>
```

### Key Commands

```bash
# Servers
lsh servers list
lsh servers list --hostname <HOSTNAME>
lsh servers create --operating_system ubuntu_24_04_x64_lts \
  --project <PROJECT_ID> --site <LOCATION> --hostname <HOSTNAME> --plan <PLAN>

# Plans
lsh plans list
lsh plans list --gpu true

# Volumes
lsh volume list --project <PROJECT_ID>
sudo lsh volume mount --id <VOLUME_ID>

# Projects
lsh projects list
lsh projects create --name "My Project"

# Virtual Networks
lsh virtual_networks list
```

### CLI Code Structure

```
latitude-cli/
├── main.go              # Entry point
├── cmd/
│   ├── root.go          # Cobra root command setup
│   └── tags/            # Tag-related commands
├── cli/
│   ├── cli.go           # Main CLI construction
│   ├── login.go         # Authentication
│   ├── *_operation.go   # API operation wrappers
│   └── *_model.go       # Data models
├── client/              # HTTP client implementation
├── internal/
│   ├── api/             # Error handling, resources
│   ├── cmdflag/         # Flag parsing utilities
│   ├── generator/       # Code generation tools
│   ├── output/          # Output formatting
│   ├── prompt/          # Interactive prompts
│   ├── renderer/        # Table/JSON rendering
│   ├── tui/             # Terminal UI components
│   └── utils/           # Shared utilities
└── models/              # API response models
```

### Technology Stack

- **Language**: Go
- **CLI Framework**: [Cobra](https://github.com/spf13/cobra)
- **TUI**: [Bubble Tea](https://github.com/charmbracelet/bubbletea) (likely)
- **Output**: Table rendering, JSON output
- **Config**: User home directory (`~/.lsh/`)

---

## Integration Opportunities for CTO Platform

### 1. Infrastructure Provisioning

- Automate bare metal server deployment for agent workloads
- GPU server provisioning for AI/ML tasks
- On-demand scaling with hourly billing

### 2. Kubernetes Cluster Expansion

- Deploy bare metal nodes as Kubernetes workers
- Use private networks for cluster networking
- NVMe storage for persistent volumes

### 3. Agent Deployment

- Direct deployment of agents to bare metal
- IPMI access for remote management
- Rescue mode for recovery operations

### 4. Cost Optimization

- Use plans API to find optimal pricing
- Mix hourly/monthly billing based on workload
- Multi-region deployment support

### 5. Potential Rust SDK Development

Based on the Go SDK patterns, a Rust SDK could be developed:
- Async/await with Tokio
- Serde for JSON serialization
- Reqwest for HTTP client
- Similar resource-based structure

---

## Next Steps

1. [ ] Evaluate GPU plan availability and pricing
2. [ ] Test API authentication and basic operations
3. [ ] Prototype bare metal provisioning workflow
4. [ ] Design Kubernetes integration strategy
5. [ ] Consider developing Rust SDK or using API directly

---

## References

- [Latitude.sh Documentation](https://www.latitude.sh/docs)
- [API Reference](https://www.latitude.sh/docs/api-reference/summary)
- [CLI Documentation](https://www.latitude.sh/docs/cli)
- [CLI GitHub Repository](https://github.com/latitudesh/cli)
- [Go SDK](https://github.com/latitudesh/latitudesh-go-sdk)

