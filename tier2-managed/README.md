# Tier 2 Managed Dedicated Infrastructure

A dual-agent self-healing system for provisioning and managing dedicated customer clusters on bare metal providers. This is the "Managed Dedicated" tier where 5D Labs provisions and manages infrastructure on the customer's chosen provider.

## Overview

This system uses two AI agents working together:

| Agent | CLI | Role |
|-------|-----|------|
| **Installer (Bolt)** | Claude | Provisions infrastructure, sets up connectivity, deploys platform |
| **Monitor** | Droid | Watches progress, detects failures, reports diagnostics |

Both agents have access to:
- **Provider MCP** - Latitude.sh, Hetzner, Vultr, etc. APIs for server management
- **Talos MCP** - Talos Linux operations (config apply, health checks)
- **Cloudflare MCP** - WARP Connector and Zero Trust configuration

## Current Scope

**Ready to implement (no cluster required):**
| Phase | Stories | Description |
|-------|---------|-------------|
| Onboarding | ONB-001 to ONB-004 | Web UI for provider/region/size selection |
| Credentials | SEC-001 to SEC-003 | API key validation and OpenBao storage |
| BoltRun | BOLT-001 to BOLT-004 | CRD schema and controller in tenant-operator |

**Blocked (requires cluster):**
| Phase | Stories | Blocker |
|-------|---------|---------|
| Infrastructure | INF-001 to INF-006 | Installer binary (separate agent) |
| Connectivity | CONN-001 to CONN-005 | Requires running cluster |
| Platform | PLAT-001 to PLAT-004 | Requires connectivity |
| Verification | VERIFY-001 to VERIFY-004 | Requires platform |

**External dependencies:**
- `crates/installer/` is managed by a separate agent - DO NOT MODIFY

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                         TIER 2 MANAGED DEDICATED ARCHITECTURE                        │
│                                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │  5D LABS CONTROL PLANE                                                       │    │
│  │                                                                              │    │
│  │  Web UI ──► Tenant Operator ──► BoltRun Controller ──► Bolt Agent Pod       │    │
│  │     │              │                    │                    │              │    │
│  │     └─► OpenBao ◄──┴────────────────────┘                    │              │    │
│  │         (credentials)                                        │              │    │
│  │                                                              │              │    │
│  │  WARP Connector ◄────────── L3 Tunnel ─────────────┐        │              │    │
│  │  Cilium ClusterMesh ◄───── Service Discovery ──────┤        │              │    │
│  └──────────────────────────────────────────────────────────────│──────────────┘    │
│                                                                 │                    │
│                                                                 │ Provisions         │
│                                                                 ▼                    │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │  CUSTOMER DEDICATED CLUSTER (Latitude/Hetzner/Vultr/etc.)                   │    │
│  │                                                                              │    │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                   │    │
│  │  │ Control Plane │  │ Worker Node 1 │  │ Worker Node N │                   │    │
│  │  │ (Talos)       │  │ (Talos)       │  │ (Talos)       │                   │    │
│  │  └───────────────┘  └───────────────┘  └───────────────┘                   │    │
│  │                                                                              │    │
│  │  Components deployed:                                                        │    │
│  │  • WARP Connector (L3 tunnel to control plane)                              │    │
│  │  • Cilium CNI + ClusterMesh                                                 │    │
│  │  • Agent Controller (watches CodeRun CRDs)                                  │    │
│  │  • ArgoCD (GitOps)                                                          │    │
│  │  • OpenBao (secrets)                                                        │    │
│  │  • Mayastor (storage)                                                       │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Option 1: tmux (Recommended)

Launch all agents in a single tmux session with split panes:

```bash
# Start tmux session with installer, monitor, and state viewer
./run-tmux.sh --tenant acme

# Attach to existing session
./run-tmux.sh --attach
```

tmux layout:
```
┌─────────────────┬─────────────────┐
│                 │   Monitor       │
│   Installer     │   Agent         │
│   Agent         ├─────────────────┤
│                 │   State Viewer  │
│                 │   (json watch)  │
└─────────────────┴─────────────────┘
```

### Option 2: Separate Terminals

```bash
# Terminal 1: Start the installer agent (unattended mode)
./run-installer.sh --tenant acme

# Terminal 2: Start the monitor agent (waits for installer to be running)
./run-monitor.sh
```

### Unattended Mode (Default)

Both agents run in fully unattended mode by default:

- **Claude** uses `--dangerously-skip-permissions` to auto-approve all operations
- **Droid** uses `droid exec --skip-permissions-unsafe --auto high` for non-interactive execution

### Interactive Mode

For debugging or manual oversight:

```bash
# Interactive mode (prompts for approval)
./run-installer.sh --interactive
```

## Files

| File | Description |
|------|-------------|
| `prd.json` | User stories and acceptance criteria (30 stories) |
| `installer-prompt.md` | Instructions for Installer agent |
| `monitor-prompt.md` | Instructions for Monitor agent |
| `ralph-coordination.json` | Shared state between agents |
| `progress.txt` | Human-readable progress log |
| `run-tmux.sh` | **Launch tmux session with all panes** |
| `run-installer.sh` | Launch Installer agent (any CLI) |
| `run-monitor.sh` | Launch Monitor agent (any CLI) |
| `cleanup.sh` | Reset state and optionally delete servers |

## Provisioning Steps

The installer runs through multiple phases:

### Phase 1: Pre-Flight (Steps 1-3)
1. ValidatingPrerequisites - Check tools and MCP access
2. FetchingCredentials - Get provider API key from OpenBao
3. ValidatingCredentials - Test API key with provider

### Phase 2: Infrastructure (Steps 4-9)
4. CreatingServers - Provision bare metal via provider API
5. CreatingVLAN - Create private network for node communication
6. WaitingServersReady - Poll until servers are "on"
7. BootingTalos - Trigger iPXE reinstall with Talos image
8. WaitingTalosMaintenance - Wait for Talos maintenance mode
9. GeneratingConfigs - Generate Talos secrets and machine configs

### Phase 3: Kubernetes Bootstrap (Steps 10-15)
10. ApplyingCPConfig - Apply config to control plane
11. WaitingCPInstall - Wait for Talos installation
12. Bootstrapping - Bootstrap etcd and Kubernetes
13. DeployingCilium - Deploy CNI (required for node Ready)
14. WaitingKubernetes - Wait for K8s API ready
15. ApplyingWorkerConfig - Apply configs to workers

### Phase 4: Connectivity (Steps 16-19)
16. DeployingWarpConnector - Install WARP Connector DaemonSet
17. RegisteringTunnel - Register with Cloudflare Zero Trust
18. ConfiguringClusterMesh - Set up Cilium ClusterMesh peering
19. VerifyingConnectivity - Test L3 and service connectivity

### Phase 5: Platform Stack (Steps 20-25)
20. DeployingBootstrapResources - Namespaces, RBAC
21. DeployingLocalPathProvisioner - Initial storage
22. DeployingArgoCD - GitOps controller
23. WaitingArgoCDReady - ArgoCD healthy
24. ApplyingAppOfApps - Deploy app-of-apps manifest
25. WaitingGitOpsSync - All apps synced

### Phase 6: Post-GitOps (Steps 26-28)
26. ConfiguringStorage - Mayastor DiskPools
27. BootstrappingOpenBao - Secrets management
28. RegisteringWithControlPlane - Fleet manager registration

### Phase 7: Verification (Steps 29-31)
29. TestingCodeRunDispatch - E2E CodeRun from control plane
30. VerifyingStatusSync - Status flows back via ClusterMesh
31. ConfiguringDashboard - Tenant dashboard shows cluster

## Cluster Size Options

| Size | Nodes | Control Plane | Workers | Plan | Use Case |
|------|-------|---------------|---------|------|----------|
| Small | 2 | 1 | 1 | c2-small-x86 | Development/Testing |
| Medium | 4 | 1 | 3 | c2-medium-x86 | Small teams |
| Large | 8 | 3 (HA) | 5 | c2-large-x86 | Production workloads |

## Supported Providers

| Provider | Status | Regions |
|----------|--------|---------|
| Latitude.sh | Supported | DAL, NYC, LAX, MIA, CHI, SEA, SJC, ASH, AMS, FRA, LON, SYD, TYO |
| Hetzner | Planned | FSN, NBG, HEL, ASH |
| Vultr | Planned | Global |
| OVH | Planned | EU, NA, APAC |
| Scaleway | Planned | PAR, AMS |

## Coordination System

The agents coordinate via `ralph-coordination.json`:

```json
{
  "installer": {
    "status": "running|waiting|failed|complete",
    "currentStep": "CreatingServers",
    "stepNumber": 4,
    "totalSteps": 31,
    "lastUpdate": "2026-01-20T12:00:00Z"
  },
  "monitor": {
    "status": "running|idle",
    "lastCheck": "2026-01-20T12:00:00Z"
  },
  "tenant": {
    "id": "acme",
    "provider": "latitude",
    "region": "DAL",
    "size": "medium"
  },
  "credentials": {
    "secretPath": "tenants/acme/provider-creds",
    "validated": true
  },
  "cluster": {
    "name": "acme-prod",
    "kubeconfig": null,
    "endpoint": null
  },
  "connectivity": {
    "warpConnector": "pending|deployed|verified",
    "clusterMesh": "pending|configured|verified",
    "tunnelId": null
  },
  "issueQueue": [],
  "circuitBreaker": {
    "state": "closed|open",
    "failureCount": 0,
    "threshold": 3
  }
}
```

## Billing Model

This implements Tier 2 "Managed Dedicated":

- **Customer pays**: Provider directly (Latitude/Hetzner/etc.) for bare metal
- **Customer pays**: 5D Labs subscription fee for managed service
- **5D Labs manages**: Provisioning, updates, monitoring, support

The customer provides their provider API key through the onboarding UI. We store it securely in OpenBao and use it to provision their dedicated cluster.

## Cleanup

```bash
# Clean local state only (installer state, coordination file, progress log)
./cleanup.sh --local

# Full cleanup (guides through server deletion + local state)
./cleanup.sh --full
```

## Related Documentation

- [Latitude Installer](../latitude-install/) - Similar Ralph loop for testing
- [Installer source](../crates/installer/) - Rust installer binary
- [Metal crate](../crates/metal/) - Provider API clients
- [Tenant Operator](../crates/tenant-operator/) - Tenant CRD and controller
- [Infrastructure Deployment Options](../docs/business/infrastructure-deployment-options.md) - Architecture decisions
