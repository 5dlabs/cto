# Infrastructure Deployment Options

This document outlines the architectural options for where CTO platform components run relative to customer infrastructure.

---

## Component Overview

The CTO platform has three distinct component categories:

| Component | Description | Resource Profile |
|-----------|-------------|------------------|
| **Control Plane** | API Gateway, Web Portal, PM Server, Tenant Operator, MCP Tools, Orchestration | Lightweight, always-on |
| **Agent Execution** | Claude/Cursor/Codex pods that run code tasks | CPU/memory intensive, ephemeral |
| **Customer Applications** | The apps our agents help build/deploy | Customer's concern |

The key architectural question: **Where does each component run?**

---

## Option A: Fully Centralized

Everything runs on 5D Labs managed infrastructure.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     5D LABS INFRASTRUCTURE (Latitude.sh)                         │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  CONTROL PLANE (shared)                                                  │    │
│  │  API Gateway │ Web Portal │ PM Server │ Tenant Operator │ MCP Tools     │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  AGENT EXECUTION POOL (shared, multi-tenant)                            │    │
│  │                                                                          │    │
│  │  tenant-acme/           tenant-bigcorp/          tenant-startup/        │    │
│  │  ┌─────────────┐       ┌─────────────┐          ┌─────────────┐        │    │
│  │  │ Claude Pod  │       │ Claude Pod  │          │ Claude Pod  │        │    │
│  │  └─────────────┘       └─────────────┘          └─────────────┘        │    │
│  │                                                                          │    │
│  │  (isolated by namespace + network policy + resource quotas)             │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘

CUSTOMER SIDE: Nothing (browser access only)
```

### Pros
- Simplest to operate (one cluster)
- Easy to update all tenants simultaneously
- Efficient resource utilization (shared pool)
- Lowest operational cost for 5D Labs
- Fastest time to market

### Cons
- Customer code passes through our infrastructure
- Noisy neighbor potential (resource contention)
- Some enterprises won't allow this model
- Single region (latency for global customers)
- Single point of failure

### Bare Metal Provider Role
- Customer has no choice - we run everything on our Latitude.sh pool
- Provider selection removed from onboarding

---

## Option B: Dedicated Execution Clusters

Control plane shared, but each customer gets their own execution cluster.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│  5D LABS CONTROL PLANE (Latitude.sh - shared)                                   │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  API Gateway │ Web Portal │ PM Server │ Tenant Operator │ Orchestration │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ Manages
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│  CUSTOMER DEDICATED CLUSTERS (Bare Metal - Customer's Choice)                   │
│                                                                                  │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐           │
│  │  Acme Cluster     │  │  BigCorp Cluster  │  │  Startup Cluster  │           │
│  │  (Latitude US)    │  │  (Hetzner EU)     │  │  (Vultr Asia)     │           │
│  │                   │  │                   │  │                   │           │
│  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │  ┌─────────────┐  │           │
│  │  │ Agent Pods  │  │  │  │ Agent Pods  │  │  │  │ Agent Pods  │  │           │
│  │  │ MCP Sidecar │  │  │  │ MCP Sidecar │  │  │  │ MCP Sidecar │  │           │
│  │  │ Secrets     │  │  │  │ Secrets     │  │  │  │ Secrets     │  │           │
│  │  └─────────────┘  │  │  └─────────────┘  │  │  └─────────────┘  │           │
│  │                   │  │                   │  │                   │           │
│  │  Nodes: 2-4       │  │  Nodes: 8-12      │  │  Nodes: 1-2       │           │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘           │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Pros
- True isolation (dedicated cluster per customer)
- Customer chooses region and provider
- Dedicated resources (no noisy neighbor)
- Data stays in customer's chosen region
- Enterprise-friendly for compliance

### Cons
- Much more complex to manage operationally
- Higher cost (dedicated nodes even when idle)
- Cluster sprawl (100 customers = 100 clusters)
- Slower to onboard new customers (cluster provisioning)
- Updates must roll out to many clusters

### Bare Metal Provider Role
- Customer selects provider and region during onboarding
- Tenant Operator provisions dedicated cluster via `crates/metal`
- We manage the cluster, they just pay for dedicated resources

---

## Option C: Hybrid by Tier (Recommended)

Shared pool for most customers, dedicated options for Enterprise.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│  5D LABS CONTROL PLANE (always centralized)                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  API │ Portal │ PM Server │ Tenant Operator │ Orchestrator │ Billing    │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  SHARED EXECUTION POOL (Free / Team / Growth tiers)                             │
│  ───────────────────────────────────────────────────                            │
│  • All non-Enterprise customers share this pool                                 │
│  • Namespace isolation, network policies, resource quotas                       │
│  • 5D Labs chooses infrastructure (Latitude.sh)                                 │
│  • Customer has NO infrastructure choices                                       │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  tenant-free-1/     tenant-team-2/     tenant-growth-3/     ...         │    │
│  │  (namespaced, resource-limited per tier)                                │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  DEDICATED EXECUTION (Enterprise tier only - optional)                          │
│  ─────────────────────────────────────────────────────                          │
│  Enterprise customers CAN opt for dedicated infrastructure:                     │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  SUB-OPTION C1: 5D Labs Managed Dedicated                               │    │
│  │  • We provision dedicated cluster on customer's chosen provider         │    │
│  │  • Customer picks: Latitude, Hetzner, Vultr, OVH, etc.                  │    │
│  │  • We manage everything (they just pay premium)                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  SUB-OPTION C2: Customer Managed (BYOC)                                 │    │
│  │  • Customer provides their own K8s cluster                              │    │
│  │  • They install CTO Gateway (Helm chart)                                │    │
│  │  • Gateway connects back to our control plane                           │    │
│  │  • Agent execution happens entirely in THEIR infrastructure             │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Pros
- Simple for 95% of customers (shared pool)
- Dedicated option available for enterprise compliance
- Clear upgrade path (shared → dedicated)
- Efficient resource utilization for lower tiers
- Enterprise gets data sovereignty options
- Matches tier-based pricing model

### Cons
- Two operational modes to support
- Gateway pattern needs additional development
- More complex Tenant Operator logic

### Bare Metal Provider Role
- **Free/Team/Growth**: No choice (we use Latitude.sh shared pool)
- **Enterprise C1**: Customer picks provider/region, we provision dedicated
- **Enterprise C2**: Customer brings their own cluster, installs our gateway

---

## Tenant Operator Behavior by Option

| Option | Tenant Operator Creates |
|--------|------------------------|
| **A: Centralized** | Namespace + RBAC + ExternalSecret + ArgoCD App in shared cluster |
| **B: Dedicated** | New cluster via metal crate, then namespace + full agent stack |
| **C: Hybrid** | Shared: namespace in shared cluster / Dedicated: cluster OR register external gateway |

---

## Recommendation

**Implement Option C (Hybrid) in phases:**

### Phase 1: MVP (Centralized Shared Pool)

Target: Initial launch

- All customers run on shared Latitude.sh cluster
- No bare metal provider selection in onboarding
- Tenant Operator creates: namespace + RBAC + secrets + ArgoCD app
- Infrastructure fields in Tenant CRD ignored or hardcoded
- Simple, fast to ship

```yaml
# Phase 1 Tenant CRD (simplified)
spec:
  owner:
    email: user@example.com
  tier: starter
  # infrastructure field not used
```

### Phase 2: Enterprise Dedicated

Target: First enterprise customers

- Add `mode: Managed` support to Tenant Operator
- Enterprise customers can request dedicated infrastructure
- Tenant Operator provisions cluster via `crates/metal`
- Customer selects provider/region in onboarding (Enterprise only)

```yaml
# Phase 2 Tenant CRD (Enterprise)
spec:
  owner:
    email: enterprise@bigcorp.com
  tier: enterprise
  infrastructure:
    mode: Managed
    provider: hetzner
    region: eu-central
```

### Phase 3: BYOC Gateway

Target: Regulated enterprise customers

- Build CTO Gateway Helm chart
- Gateway watches for CodeRun CRDs locally
- Forwards to control plane, syncs status back
- Agent execution happens in customer's cluster

```yaml
# Phase 3 Tenant CRD (BYOC)
spec:
  owner:
    email: compliance@bank.com
  tier: enterprise
  infrastructure:
    mode: Byoc
    clusterRef: gateway-token-xxxxx
```

### Phase 4: Regional Pools

Target: Global scale

- Add US-West, EU, Asia shared pools
- Non-enterprise customers can select region (not provider)
- Reduces latency for global customers
- Still shared pool, just multi-region

---

## Current Implementation Status

The Tenant CRD already includes infrastructure fields:

```rust
pub struct InfrastructureConfig {
    pub mode: InfraMode,                    // CtoCloud | Managed | Byoc
    pub provider: Option<BareMetalProvider>, // Latitude, Hetzner, etc.
    pub region: Option<String>,
    pub cluster_ref: Option<String>,
}
```

**For Phase 1**, these fields should be:
- Ignored by the controller (or defaulted)
- Hidden from onboarding UI
- Documented as "reserved for future use"

The Tenant Operator currently implements Phase 1 behavior (creates namespace in existing cluster).

---

## Related Documents

- [SaaS Architecture](saas-architecture.md) - Overall platform architecture
- [SaaS Monetization](saas-monetization.md) - Tier definitions and pricing
- [Implementation Roadmap](implementation-roadmap.md) - Development priorities
