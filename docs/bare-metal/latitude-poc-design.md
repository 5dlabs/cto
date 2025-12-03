# Latitude.sh Bare Metal POC Design

> **Status**: Draft  
> **Author**: CTO Platform Team  
> **Created**: December 2024  
> **Scope**: Proof of Concept - Single Node

## 1. Overview

### 1.1 Goal

Validate that the CTO Platform can be automatically provisioned on Latitude.sh bare metal infrastructure using their API, with Talos Linux as the operating system and full Kubernetes/CTO stack deployment.

### 1.2 Success Criteria

| Criteria | Description |
|----------|-------------|
| ✅ Provision | Successfully provision a bare metal server via Latitude API |
| ✅ Talos Boot | Boot Talos Linux via iPXE on the provisioned server |
| ✅ Kubernetes | Bootstrap single-node Kubernetes cluster |
| ✅ CTO Stack | Install core CTO components (ArgoCD, Vault, workflows) |
| ✅ Accessible | Platform accessible via public IP or tunnel |

### 1.3 Non-Goals (MVP Exclusions)

- Multi-node clustering
- Multi-provider support
- Automated teardown/cleanup via API
- Cost estimation/display
- GPU or specialized hardware
- Production hardening
- Custom domain/TLS setup

## 2. Architecture

### 2.1 High-Level Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CTO Installer CLI                                │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 1: Authentication                                                 │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Prompt user for Latitude API key                              │   │
│  │ 2. Validate key against Latitude API                             │   │
│  │ 3. Store key locally (or Vault if available)                     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 2: Server Selection                                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Query available plans: GET /plans?filter[in_stock]=true       │   │
│  │ 2. Filter to smallest/cheapest available                         │   │
│  │ 3. Query regions: GET /regions                                   │   │
│  │ 4. Present options to user with hourly pricing                   │   │
│  │ 5. User confirms selection                                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 3: Server Provisioning                                            │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Create SSH key: POST /ssh_keys                                │   │
│  │ 2. Create project: POST /projects (if needed)                    │   │
│  │ 3. Deploy server: POST /servers                                  │   │
│  │    - Plan: selected plan                                         │   │
│  │    - Site: selected region                                       │   │
│  │    - OS: ubuntu_22_04_x64_lts (temporary)                       │   │
│  │    - SSH Key: created key                                        │   │
│  │ 4. Poll for ready: GET /servers/{id} until status=on             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 4: Talos Installation                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Generate Talos machine config (secrets, certs)                │   │
│  │ 2. Host config at temporary URL (or embed in iPXE)               │   │
│  │ 3. Reinstall with Talos: POST /servers/{id}/reinstall            │   │
│  │    - OperatingSystem: ipxe                                       │   │
│  │    - Ipxe: talos factory URL with config                         │   │
│  │ 4. Poll for boot completion                                      │   │
│  │ 5. Verify Talos API responds                                     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 5: Kubernetes Bootstrap                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Apply machine config via talosctl                             │   │
│  │ 2. Bootstrap etcd: talosctl bootstrap                            │   │
│  │ 3. Wait for Kubernetes API                                       │   │
│  │ 4. Generate kubeconfig                                           │   │
│  │ 5. Verify cluster health                                         │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 6: CTO Platform Installation                                      │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Install ArgoCD                                                │   │
│  │ 2. Install Argo Workflows                                        │   │
│  │ 3. Install Vault (dev mode for POC)                              │   │
│  │ 4. Install CTO Controller                                        │   │
│  │ 5. Install MCP Server                                            │   │
│  │ 6. Configure basic secrets                                       │   │
│  │ 7. Verify all pods running                                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 7: Output & Access                                                │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. Display server public IP                                      │   │
│  │ 2. Output kubeconfig location                                    │   │
│  │ 3. Output talosconfig location                                   │   │
│  │ 4. Display ArgoCD URL and credentials                            │   │
│  │ 5. Display Vault URL                                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Latitude.sh Bare Metal Server                     │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                         Talos Linux                                 │ │
│  │  ┌──────────────────────────────────────────────────────────────┐  │ │
│  │  │                    Kubernetes (single node)                   │  │ │
│  │  │                                                               │  │ │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │  │ │
│  │  │  │  ArgoCD     │ │ Argo        │ │ CTO Platform            │ │  │ │
│  │  │  │  Namespace  │ │ Workflows   │ │ ┌─────────┬───────────┐ │ │  │ │
│  │  │  │             │ │ Namespace   │ │ │Controller│ MCP Server│ │ │  │ │
│  │  │  └─────────────┘ └─────────────┘ │ └─────────┴───────────┘ │ │  │ │
│  │  │                                   └─────────────────────────┘ │  │ │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │  │ │
│  │  │  │  Vault      │ │ Monitoring  │ │ Storage                 │ │  │ │
│  │  │  │  (dev mode) │ │ (optional)  │ │ (local-path)            │ │  │ │
│  │  │  └─────────────┘ └─────────────┘ └─────────────────────────┘ │  │ │
│  │  │                                                               │  │ │
│  │  └──────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Public IP: xxx.xxx.xxx.xxx                                              │
│  IPMI: Available via VPN                                                 │
└─────────────────────────────────────────────────────────────────────────┘
```

## 3. Latitude.sh API Integration

### 3.1 Required Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/plans` | GET | List available server plans with pricing |
| `/regions` | GET | List available regions/sites |
| `/projects` | POST | Create project to contain server |
| `/ssh_keys` | POST | Create SSH key for initial access |
| `/servers` | POST | Provision new server |
| `/servers/{id}` | GET | Check server status |
| `/servers/{id}/reinstall` | POST | Reinstall with Talos via iPXE |

### 3.2 Authentication

```rust
// API key stored in header
Authorization: Bearer <API_KEY>
```

**Vault Storage:**
- Path: `secret/providers/latitude`
- Key: `api_key`
- Kubernetes Secret: `provider-latitude` in `cto` namespace

API key workflow:
1. User creates key at https://www.latitude.sh/dashboard/api
2. CLI prompts for key input
3. Key validated via `GET /user/profile`
4. Key stored in Vault at `secret/providers/latitude`
5. VSO syncs to Kubernetes as `provider-latitude` secret

### 3.3 Server Provisioning Request

```json
{
  "data": {
    "type": "servers",
    "attributes": {
      "project": "proj_xxx",
      "plan": "c2-small-x86",
      "site": "ash",
      "operating_system": "ubuntu_22_04_x64_lts",
      "hostname": "cto-platform-001",
      "ssh_keys": ["ssh_xxx"]
    }
  }
}
```

### 3.4 Talos Reinstall Request

```json
{
  "data": {
    "type": "reinstalls",
    "attributes": {
      "operating_system": "ipxe",
      "hostname": "cto-platform-001",
      "ipxe": "https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.8.0/metal-amd64"
    }
  }
}
```

## 4. Talos Configuration

### 4.1 iPXE Boot Strategy

We'll use Talos Factory's PXE service which provides:
- Signed, verified Talos images
- Custom configuration embedding
- Schematic-based customization

**iPXE URL Format:**
```
https://pxe.factory.talos.dev/pxe/{schematic_id}/{version}/{platform}
```

### 4.2 Talos Factory Schematic

Create a minimal schematic for bare metal:

```yaml
# talos-schematic.yaml
customization:
  systemExtensions:
    officialExtensions:
      - siderolabs/iscsi-tools      # For potential storage
      - siderolabs/util-linux-tools  # Useful utilities
```

Generate schematic ID:
```bash
curl -X POST --data-binary @talos-schematic.yaml \
  https://factory.talos.dev/schematics
# Returns: {"id": "376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba"}
```

### 4.3 Machine Configuration

Generate Talos machine config with:

```bash
talosctl gen config cto-cluster https://<SERVER_IP>:6443 \
  --output-dir ./talos-config \
  --with-docs=false \
  --with-examples=false
```

Key configuration items:
- Single control-plane node (acts as both control plane and worker)
- Local path provisioner for storage
- Disable default CNI (install Cilium via Helm)

### 4.4 Config Delivery Options

**Option A: Talos Factory with Embedded Config (Recommended for POC)**

1. Base64 encode machine config
2. Include in factory URL parameters
3. Talos boots with config pre-applied

**Option B: Config URL**

1. Host config at publicly accessible URL
2. Pass URL to Talos via kernel param: `talos.config=https://...`

**Option C: Post-Boot Apply**

1. Boot Talos in maintenance mode
2. Apply config via `talosctl apply-config`

For POC, we'll use **Option C** as it's simplest to implement and debug.

## 5. Implementation Plan

### 5.1 New Installer Module Structure

```
crates/installer/src/
├── providers/
│   ├── mod.rs
│   ├── latitude/
│   │   ├── mod.rs
│   │   ├── client.rs      # HTTP client for Latitude API
│   │   ├── models.rs      # API request/response types
│   │   └── provisioner.rs # Server provisioning logic
│   └── traits.rs          # Provider trait for future providers
├── talos/
│   ├── mod.rs
│   ├── config.rs          # Machine config generation
│   └── bootstrap.rs       # Cluster bootstrap logic
└── ...
```

### 5.2 Provider Trait

```rust
/// Trait for bare metal providers
#[async_trait]
pub trait BareMetalProvider {
    /// Authenticate with the provider
    async fn authenticate(&self, credentials: &Credentials) -> Result<()>;
    
    /// List available server plans
    async fn list_plans(&self) -> Result<Vec<Plan>>;
    
    /// List available regions
    async fn list_regions(&self) -> Result<Vec<Region>>;
    
    /// Provision a new server
    async fn provision(&self, config: &ServerConfig) -> Result<Server>;
    
    /// Get server status
    async fn get_server(&self, id: &str) -> Result<Server>;
    
    /// Reinstall server with custom OS/iPXE
    async fn reinstall(&self, id: &str, config: &ReinstallConfig) -> Result<()>;
    
    /// Get server's public IP
    async fn get_public_ip(&self, id: &str) -> Result<IpAddr>;
}
```

### 5.3 CLI Flow

```rust
// New subcommand for bare metal
#[derive(Subcommand)]
enum Commands {
    Install(InstallArgs),
    // ... existing commands ...
    
    /// Provision on bare metal cloud
    BareMetal {
        #[command(subcommand)]
        provider: BareMetalProvider,
    },
}

#[derive(Subcommand)]
enum BareMetalProvider {
    /// Deploy to Latitude.sh
    Latitude {
        /// API key (or prompted if not provided)
        #[arg(long, env = "LATITUDE_API_KEY")]
        api_key: Option<String>,
        
        /// Server plan (or interactively selected)
        #[arg(long)]
        plan: Option<String>,
        
        /// Region/site (or interactively selected)
        #[arg(long)]
        region: Option<String>,
        
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
    },
}
```

### 5.4 User Interaction Flow

```
$ cto-cli bare-metal latitude

╭──────────────────────────────────────────────────────╮
│           CTO Platform - Latitude.sh Setup           │
╰──────────────────────────────────────────────────────╯

→ Checking for Latitude API key...

  No API key found. Please create one at:
  https://www.latitude.sh/dashboard/api

  Enter your Latitude API key: ████████████████████████

→ Validating API key... ✓

→ Fetching available servers...

  Available Plans (sorted by price):

  │ # │ Plan          │ CPU        │ RAM  │ Storage │ $/hour │ Region    │
  ├───┼───────────────┼────────────┼──────┼─────────┼────────┼───────────┤
  │ 1 │ c2.small.x86  │ 6c/3.8GHz  │ 32GB │ 500GB   │ $0.15  │ Ashburn   │
  │ 2 │ c2.small.x86  │ 6c/3.8GHz  │ 32GB │ 500GB   │ $0.15  │ São Paulo │
  │ 3 │ c2.medium.x86 │ 8c/3.6GHz  │ 64GB │ 1TB     │ $0.25  │ Ashburn   │

  Select a plan [1]: 1

→ Provisioning server...
  ├─ Creating SSH key... ✓
  ├─ Creating project... ✓
  ├─ Deploying server... ✓
  └─ Waiting for server ready... ████████░░ 80%

→ Installing Talos Linux...
  ├─ Generating machine config... ✓
  ├─ Triggering iPXE reinstall... ✓
  └─ Waiting for Talos boot... ████████████ 100%

→ Bootstrapping Kubernetes...
  ├─ Applying machine config... ✓
  ├─ Bootstrapping etcd... ✓
  ├─ Waiting for API server... ✓
  └─ Generating kubeconfig... ✓

→ Installing CTO Platform...
  ├─ Installing ArgoCD... ✓
  ├─ Installing Argo Workflows... ✓
  ├─ Installing Vault... ✓
  ├─ Installing CTO Controller... ✓
  └─ Installing MCP Server... ✓

╭──────────────────────────────────────────────────────╮
│              🎉 Installation Complete!               │
╰──────────────────────────────────────────────────────╯

  Server IP:     203.0.113.42
  Kubeconfig:    ~/.cto/kubeconfig
  Talosconfig:   ~/.cto/talosconfig
  
  ArgoCD:        https://203.0.113.42:30080
  ArgoCD User:   admin
  ArgoCD Pass:   (run: kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d)

  ⚠️  Remember: This server bills hourly. Delete when done testing!
```

## 6. Dependencies

### 6.1 Rust Crates

```toml
[dependencies]
# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI and TUI
clap = { version = "4", features = ["derive", "env"] }
dialoguer = "0.11"
indicatif = "0.17"
colored = "2"

# Talos/Kubernetes
kube = { version = "0.96", features = ["runtime", "derive"] }

# Error handling
anyhow = "1"
thiserror = "2"

# Secrets
secrecy = "0.10"
```

### 6.2 External Tools

| Tool | Purpose | Required |
|------|---------|----------|
| `talosctl` | Talos cluster management | Yes |
| `kubectl` | Kubernetes operations | Yes |
| `helm` | Chart installation | Yes |

These can be bundled with the installer or downloaded on demand.

## 7. Testing Strategy

### 7.1 Manual POC Testing

1. **Phase 1: API Validation**
   - Manually test Latitude API with curl
   - Confirm plan listing, server creation
   - Test reinstall with iPXE URL

2. **Phase 2: Talos Boot**
   - Confirm Talos factory URL works
   - Verify server boots into Talos
   - Test talosctl connectivity

3. **Phase 3: Full Flow**
   - Run installer end-to-end
   - Verify all components running
   - Access ArgoCD UI

### 7.2 Test Commands

```bash
# Test Latitude API
curl -H "Authorization: Bearer $LATITUDE_API_KEY" \
  https://api.latitude.sh/plans

# Test Talos factory
curl https://factory.talos.dev/schematics

# Verify Talos boot
talosctl --nodes <IP> health

# Verify Kubernetes
kubectl get nodes
kubectl get pods -A
```

## 8. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Talos factory unreachable from Latitude network | Low | High | Fall back to hosting our own iPXE endpoint |
| Server provisioning takes too long | Medium | Medium | Add timeout and retry logic |
| iPXE boot fails silently | Medium | High | Use IPMI console to debug |
| Kubernetes bootstrap fails | Low | Medium | Implement retry with exponential backoff |
| API rate limiting | Low | Low | Implement request throttling |

## 9. Cost Estimate

| Item | Cost |
|------|------|
| Smallest server (c2.small.x86) | ~$0.15/hour |
| Estimated testing time | 4-8 hours |
| **Total POC cost** | **$0.60 - $1.20** |

## 10. Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| Week 1 | 3 days | Latitude client implementation |
| Week 1 | 2 days | Talos config generation |
| Week 2 | 3 days | CLI integration |
| Week 2 | 2 days | CTO component installation |
| Week 3 | 2 days | Testing and iteration |
| Week 3 | 1 day | Documentation |

**Total: ~2-3 weeks**

## 11. Future Enhancements (Post-POC)

- [ ] Multi-node cluster support
- [ ] Automated server teardown
- [ ] Cost estimation before provisioning
- [ ] Multi-provider support (Hetzner, Vultr, OVH)
- [ ] Custom domain and TLS setup
- [ ] GPU server support
- [ ] Storage provisioning (Latitude volumes)
- [ ] Private networking configuration
- [ ] Backup and disaster recovery

## 12. Appendix

### A. Latitude API Reference

Full API documentation: https://www.latitude.sh/docs/api-reference/summary

### B. Talos Factory Reference

- Factory service: https://factory.talos.dev
- PXE boot docs: https://www.talos.dev/v1.8/talos-guides/install/bare-metal-platforms/network-booting/
- Machine config: https://www.talos.dev/v1.8/reference/configuration/

### C. Sample Talos Machine Config

```yaml
# controlplane.yaml (simplified for single-node)
version: v1alpha1
machine:
  type: controlplane
  token: <generated>
  ca:
    crt: <generated>
    key: <generated>
  certSANs:
    - <SERVER_IP>
  kubelet:
    image: ghcr.io/siderolabs/kubelet:v1.31.0
  network:
    hostname: cto-platform
  install:
    disk: /dev/sda
    image: ghcr.io/siderolabs/installer:v1.8.0
    wipe: true
cluster:
  controlPlane:
    endpoint: https://<SERVER_IP>:6443
  clusterName: cto-cluster
  network:
    cni:
      name: none  # We'll install Cilium
  allowSchedulingOnControlPlanes: true  # Single node needs this
```

