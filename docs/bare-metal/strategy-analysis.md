# CTO Platform: Bare Metal Strategy Analysis

> Connecting Latitude.sh capabilities with CTO Platform business model and installer

## Executive Summary

This analysis connects the Latitude.sh API/CLI research with CTO Platform's existing business case documents and installer codebase. The goal is to identify how Latitude.sh can enhance the **Platform-in-a-Box** offering.

---

## Current State Assessment

### Business Model (from `platform-packaging/`)

| Tier | Target | Delivery | Price |
|------|--------|----------|-------|
| **Appliance** | Small teams, startups | Bootable ISO | $990/yr |
| **Enterprise** | Growing orgs | Multi-node | $7,990+/yr |
| **Platform** | DevOps teams | Helm charts | $2,990/yr |

**Key Value Prop**: 70-80% cost savings vs cloud, with data sovereignty.

### Current Installer (`crates/installer/`)

```
installer/
├── commands/install.rs    # Main install command
├── config.rs              # Profile configs (Minimal/Standard/Production)
├── installer/
│   ├── cluster.rs         # Kind cluster provisioning
│   ├── components.rs      # ArgoCD, Workflows, etc.
│   └── config_generator.rs # cto-config.json generation
└── validator.rs           # Prerequisites check
```

**Current Limitations:**
- Only supports **Kind** (local) or **Remote** (existing k8s)
- No bare metal provisioning automation
- No cloud provider integration
- Manual hardware setup required

---

## Latitude.sh Integration Opportunity

### Why Latitude.sh?

| Factor | Benefit for CTO Platform |
|--------|--------------------------|
| **API-first** | Easy automation integration |
| **GPU Support** | H100/L40S for AI workloads |
| **Go SDK** | Reference for Rust implementation |
| **Modern CLI** | Pattern for our installer |
| **Ex-Packet team** | Enterprise reliability |
| **Hourly billing** | POC-friendly for customers |

### Latitude.sh vs Current Bare Metal Recommendations

From `bare-metal-providers.md`:

| Provider | 3-Node Cost | GPU Support | API Quality |
|----------|-------------|-------------|-------------|
| Hetzner | $195/mo | Limited | Good |
| OVH | $210/mo | Limited | Moderate |
| Vultr | $360/mo | ✅ H100/A100 | Good |
| **Latitude.sh** | $540/mo | ✅ H100/L40S | **Excellent** |

**Latitude.sh Trade-off**: Higher price, but:
- Best API for automation
- Superior GPU availability  
- US-based with global reach
- Best for Kubernetes workloads

---

## Proposed Architecture Enhancement

### Current Flow (Manual)

```
Customer → Download ISO → Boot on Hardware → Setup Wizard → Done
                              ↑
                    (customer provisions hardware manually)
```

### Proposed Flow (Latitude-Automated)

```
Customer → CTO CLI → Latitude API → Provision Servers → Install Talos → Deploy Platform
              │
              └── "cto deploy --provider latitude --plan c2.medium.x86 --nodes 3"
```

### Integration Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CTO CLI (Enhanced)                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  cto deploy                                                                 │
│  ├── --provider <latitude|hetzner|vultr|local>                             │
│  ├── --plan <plan-slug>                                                     │
│  ├── --nodes <count>                                                        │
│  ├── --region <site-code>                                                   │
│  ├── --profile <minimal|standard|production>                                │
│  └── --license <key>                                                        │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Provider Abstraction Layer                        │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │ Latitude    │  │ Hetzner     │  │ Vultr       │  │ Local     │  │   │
│  │  │ Provider    │  │ Provider    │  │ Provider    │  │ (Kind)    │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Provisioning Pipeline                             │   │
│  │  1. Validate credentials & license                                   │   │
│  │  2. List available plans (filter by GPU, RAM, location)             │   │
│  │  3. Provision servers via API                                        │   │
│  │  4. Wait for servers to be ready                                     │   │
│  │  5. Configure networking (VLANs, IPs)                                │   │
│  │  6. Install Talos Linux (via IPMI/rescue mode)                      │   │
│  │  7. Bootstrap Kubernetes cluster                                     │   │
│  │  8. Deploy CTO Platform components                                   │   │
│  │  9. Configure DNS (optional)                                         │   │
│  │  10. Output access credentials                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Provider Abstraction (2-3 weeks)

Add provider trait to `crates/installer/`:

```rust
// src/providers/mod.rs
pub trait CloudProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<()>;
    async fn list_plans(&self, filter: PlanFilter) -> Result<Vec<Plan>>;
    async fn list_regions(&self) -> Result<Vec<Region>>;
    async fn provision_server(&self, config: ServerConfig) -> Result<Server>;
    async fn get_server_status(&self, id: &str) -> Result<ServerStatus>;
    async fn destroy_server(&self, id: &str) -> Result<()>;
    async fn create_network(&self, config: NetworkConfig) -> Result<Network>;
    async fn assign_server_to_network(&self, server_id: &str, network_id: &str) -> Result<()>;
}

// Provider implementations
pub mod latitude;  // New - based on our research
pub mod hetzner;   // Future
pub mod vultr;     // Future
```

### Phase 2: Latitude.sh Provider (2-3 weeks)

Implement Latitude provider based on API research:

```rust
// src/providers/latitude.rs
pub struct LatitudeProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl CloudProvider for LatitudeProvider {
    async fn list_plans(&self, filter: PlanFilter) -> Result<Vec<Plan>> {
        // GET /plans with filters
        // - filter[gpu] = true
        // - filter[location] = "NY1"
        // - filter[in_stock] = true
    }
    
    async fn provision_server(&self, config: ServerConfig) -> Result<Server> {
        // POST /servers
        // {
        //   "data": {
        //     "type": "servers",
        //     "attributes": {
        //       "project": "proj_xxx",
        //       "plan": "c2.medium.x86",
        //       "site": "NY1",
        //       "operating_system": "talos_v1.8", // or custom
        //       "hostname": "cto-node-1"
        //     }
        //   }
        // }
    }
}
```

### Phase 3: Talos Integration (1-2 weeks)

Extend provisioning to install Talos:

```rust
// Options:
// 1. Use Latitude's custom image support (upload Talos image)
// 2. Use rescue mode + talosctl to install
// 3. PXE boot with Talos ISO

async fn install_talos(&self, server: &Server, config: &TalosConfig) -> Result<()> {
    // 1. Enable rescue mode
    self.client.post(&format!("/servers/{}/rescue", server.id)).await?;
    
    // 2. Wait for rescue mode
    self.wait_for_rescue(server).await?;
    
    // 3. SSH and install Talos
    let ssh = self.connect_via_ipmi(server).await?;
    ssh.execute("curl -LO https://...talos.iso && dd if=...").await?;
    
    // 4. Apply Talos machine config
    talosctl::apply_config(&server.primary_ip, &config).await?;
}
```

### Phase 4: Enhanced CLI (1 week)

Update installer CLI:

```rust
// src/commands/deploy.rs
#[derive(clap::Args)]
pub struct DeployCommand {
    /// Cloud provider to use
    #[arg(long, value_enum)]
    provider: Provider,
    
    /// Server plan (e.g., "c2.medium.x86")
    #[arg(long)]
    plan: String,
    
    /// Number of nodes
    #[arg(long, default_value = "1")]
    nodes: u32,
    
    /// Region/site code (e.g., "NY1", "LAX")
    #[arg(long)]
    region: String,
    
    /// Installation profile
    #[arg(long, value_enum, default_value = "standard")]
    profile: InstallProfile,
    
    /// Project name for organization
    #[arg(long)]
    project: Option<String>,
    
    /// Enable GPU nodes
    #[arg(long)]
    gpu: bool,
}

impl DeployCommand {
    pub async fn run(&self) -> Result<()> {
        let provider = match self.provider {
            Provider::Latitude => Box::new(LatitudeProvider::from_env()?),
            Provider::Hetzner => Box::new(HetznerProvider::from_env()?),
            Provider::Local => Box::new(KindProvider::new()),
        };
        
        // Interactive plan selection if not specified
        let plan = if self.plan.is_empty() {
            self.select_plan(&provider).await?
        } else {
            self.plan.clone()
        };
        
        // Provision infrastructure
        let servers = provider.provision_cluster(
            &plan,
            self.nodes,
            &self.region,
            self.gpu,
        ).await?;
        
        // Install Talos
        let cluster = self.install_talos(&servers).await?;
        
        // Deploy CTO Platform
        let installer = Installer::new(self.profile.into());
        installer.install_on_cluster(&cluster).await?;
        
        Ok(())
    }
}
```

---

## Cost-Benefit Analysis

### For CTO Platform Business

| Benefit | Impact |
|---------|--------|
| **Faster time-to-value** | 15 min setup vs days |
| **Lower support burden** | Automated = fewer tickets |
| **Expanded addressable market** | Teams without hardware expertise |
| **Premium tier opportunity** | "Managed bare metal" offering |
| **Competitive advantage** | First AI platform with cloud-agnostic bare metal |

### For Customers

| Scenario | Manual Setup | With Latitude Integration |
|----------|--------------|---------------------------|
| Time to running platform | 2-3 days | 30 minutes |
| Infrastructure expertise needed | High | None |
| Risk of misconfiguration | High | Eliminated |
| GPU availability | Research required | One-click |

### Pricing Impact

Could enable new tier:

| Tier | Current | With Bare Metal Automation |
|------|---------|---------------------------|
| Starter | $990/yr | $990/yr (no change) |
| Team | $2,990/yr | $2,990/yr (no change) |
| **Business + Infra** | N/A | $4,990/yr + pass-through |
| Enterprise | $20K+/yr | $25K+/yr (premium automation) |

---

## Open Questions

1. **Latitude Partnership?**
   - Could we become a Latitude partner for better rates?
   - Co-marketing opportunity?

2. **Multi-Provider Priority?**
   - Latitude first (best API) → then Hetzner (best price)?
   - Or abstract from start?

3. **Talos Image Strategy?**
   - Custom Latitude image (simpler)?
   - Rescue mode installation (more flexible)?
   - PXE boot (most control)?

4. **License Tie-In?**
   - Should bare metal provisioning require Business+ tier?
   - Or offer as separate add-on?

5. **GPU-First Positioning?**
   - Market as "AI infrastructure platform"?
   - H100 availability as differentiator?

---

## Next Steps

1. [ ] **Prototype** Latitude provider in Rust (1 week)
2. [ ] **Test** Talos installation via rescue mode
3. [ ] **Design** provider abstraction trait
4. [ ] **Evaluate** Latitude partnership opportunity
5. [ ] **Update** pricing model with managed option
6. [ ] **Document** GPU workflow for AI teams

---

## References

- [Latitude Research](./latitude-research.md) - API capabilities
- [Latitude CLI](./latitude-cli/) - Go SDK patterns
- [Bare Metal Providers](../platform-packaging/bare-metal-providers.md) - Provider comparison
- [Business Model](../platform-packaging/business-model.md) - Pricing strategy
- [PRD](../platform-packaging/prd.txt) - Product requirements
- [Architecture](../platform-packaging/architecture.md) - Technical design

