# Provider Skill: Sidero Metal

Sidero Metal is a **bare-metal provisioning system with native Talos Linux support** built on Kubernetes Cluster API. It provides API-driven lifecycle management for bare-metal servers with automated PXE booting, IPMI/Redfish integration, and declarative machine configuration.

## Overview

- **Project Status**: Community-maintained (Sidero Labs no longer actively developing, but project remains functional)
- **License**: Apache 2.0
- **Integration**: Kubernetes Cluster API (CAPI) provider
- **GitHub**: https://github.com/siderolabs/sidero
- **Docs**: https://www.sidero.dev/docs/

## Why Sidero Metal?

- **Native Talos Integration**: Built specifically for Talos Linux — no custom boot scripts needed
- **Declarative Configuration**: Define servers, server classes, and clusters as Kubernetes CRDs
- **Cluster API Standard**: Follows CAPI patterns for multi-cluster management
- **IPMI/Redfish Automation**: Out-of-band management for server power control and provisioning
- **PXE Boot Orchestration**: Automatic iPXE chain-loading from Talos Image Factory

## Architecture

### Core Components

1. **Management Plane (Kubernetes cluster)**
   - Sidero Metal controller
   - Cluster API controllers (bootstrap, control-plane, infrastructure)
   - SideroLink for secure node communication

2. **Bare-Metal Servers**
   - Boot via PXE (iPXE) into Talos maintenance mode
   - Register with management plane via SideroLink
   - Provisioned declaratively via CAPI resources

3. **Workload Clusters**
   - Created via CAPI Machine/MachineDeployment resources
   - Managed lifecycle (create, scale, upgrade, destroy)

## Installation

### Prerequisites

- Existing Kubernetes cluster (management cluster)
- `clusterctl` CLI tool
- Bare-metal servers with IPMI/Redfish support (optional but recommended)

### Install Sidero Metal

```bash
# Initialize Cluster API with Sidero Metal provider
clusterctl init \
  --bootstrap talos \
  --control-plane talos \
  --infrastructure sidero
```

This installs:
- Sidero Metal controller
- Talos Bootstrap Provider
- Talos Control Plane Provider

### Verify Installation

```bash
kubectl get pods -n sidero-system
kubectl get pods -n capi-system
kubectl get pods -n cabpt-system
kubectl get pods -n cacppt-system
```

## Key Resources (CRDs)

### 1. **Environment** (Grouping & Configuration)

Organizes infrastructure resources and defines environment-level config:

```yaml
apiVersion: metal.sidero.dev/v1alpha2
kind: Environment
metadata:
  name: production
spec:
  kernel:
    url: "https://pxe.factory.talos.dev/pxe/{schematic}/{version}/kernel-amd64"
  initrd:
    url: "https://pxe.factory.talos.dev/pxe/{schematic}/{version}/initramfs-amd64.xz"
```

### 2. **Server** (Physical Server Inventory)

Represents a discovered bare-metal server:

```yaml
apiVersion: metal.sidero.dev/v1alpha2
kind: Server
metadata:
  name: server-001
spec:
  accepted: true
  environmentRef:
    name: production
  bmcRef:
    name: server-001-bmc
  configPatches:
    - op: add
      path: /machine/network/interfaces
      value:
        - deviceSelector:
            physical: true
          dhcp: true
```

Key fields:
- `accepted`: Must be `true` to allow provisioning
- `environmentRef`: Links to Environment resource
- `bmcRef`: Links to ServerBMC for IPMI/Redfish control
- `configPatches`: Talos machine config patches (JSON Patch format)

### 3. **ServerClass** (Server Grouping by Specs)

Groups servers by hardware characteristics (auto-discovery):

```yaml
apiVersion: metal.sidero.dev/v1alpha2
kind: ServerClass
metadata:
  name: control-plane
spec:
  qualifiers:
    cpu:
      - manufacturer: Intel
        version: "Xeon"
    memory:
      - size: ">= 16GB"
  configPatches:
    - op: add
      path: /machine/kubelet/extraArgs
      value:
        node-labels: "node-role.kubernetes.io/control-plane="
```

### 4. **ServerBMC** (Out-of-Band Management)

Defines IPMI/Redfish credentials for server power control:

```yaml
apiVersion: metal.sidero.dev/v1alpha2
kind: ServerBMC
metadata:
  name: server-001-bmc
spec:
  host: "10.0.1.10"
  port: 623
  user: "ADMIN"
  pass:
    secretRef:
      name: server-001-bmc-secret
      key: password
```

## Provisioning Workflow

### Phase 1: Server Discovery

1. **PXE Boot**: Server boots via network PXE
2. **Talos Maintenance Mode**: Sidero Metal provides iPXE script pointing to Talos Image Factory
3. **Server Registration**: Talos agent registers server with management plane via SideroLink
4. **Server Resource Created**: Sidero controller creates Server CRD with discovered hardware specs

### Phase 2: Server Acceptance

```bash
# List discovered servers
kubectl get servers

# Accept a server for provisioning
kubectl patch server server-001 -p '{"spec":{"accepted":true}}' --type=merge
```

### Phase 3: Workload Cluster Creation (via Cluster API)

```yaml
apiVersion: cluster.x-k8s.io/v1beta1
kind: Cluster
metadata:
  name: workload-cluster-1
spec:
  clusterNetwork:
    pods:
      cidrBlocks:
        - 10.244.0.0/16
    services:
      cidrBlocks:
        - 10.96.0.0/12
  infrastructureRef:
    apiVersion: infrastructure.cluster.x-k8s.io/v1alpha3
    kind: MetalCluster
    name: workload-cluster-1
  controlPlaneRef:
    apiVersion: controlplane.cluster.x-k8s.io/v1alpha3
    kind: TalosControlPlane
    name: workload-cluster-1-cp
---
apiVersion: infrastructure.cluster.x-k8s.io/v1alpha3
kind: MetalCluster
metadata:
  name: workload-cluster-1
spec:
  controlPlaneEndpoint:
    host: "10.0.1.100"
    port: 6443
---
apiVersion: controlplane.cluster.x-k8s.io/v1alpha3
kind: TalosControlPlane
metadata:
  name: workload-cluster-1-cp
spec:
  version: v1.8.4
  replicas: 3
  infrastructureTemplate:
    apiVersion: infrastructure.cluster.x-k8s.io/v1alpha3
    kind: MetalMachineTemplate
    name: workload-cluster-1-cp
  controlPlaneConfig:
    controlplane:
      generateType: controlplane
---
apiVersion: infrastructure.cluster.x-k8s.io/v1alpha3
kind: MetalMachineTemplate
metadata:
  name: workload-cluster-1-cp
spec:
  template:
    spec:
      serverClassRef:
        name: control-plane
```

## Integration with CTO Platform

### Option 1: Direct Sidero Metal Integration

Use Sidero Metal CRDs directly from CTO's `cto-metal` CLI:

```rust
// Pseudo-code for crates/metal/src/providers/sidero/client.rs
pub struct SideroProvider {
    kube_client: kube::Client,
}

impl SideroProvider {
    pub async fn create_server(&self, config: ServerConfig) -> Result<Server> {
        // 1. Create ServerBMC resource with IPMI/Redfish credentials
        // 2. Wait for Server resource to auto-discover
        // 3. Accept server (set spec.accepted = true)
        // 4. Apply configPatches for Talos machine config
    }

    pub async fn create_cluster(&self, config: ClusterConfig) -> Result<Cluster> {
        // 1. Create CAPI Cluster + MetalCluster resources
        // 2. Create TalosControlPlane with desired replicas
        // 3. Create MetalMachineTemplate referencing ServerClass
        // 4. Wait for cluster to become ready
    }
}
```

### Option 2: Omni Integration (Recommended)

Use **Omni** (see `provider-omni.md`) for higher-level cluster management. Omni provides:
- Simplified API (REST/gRPC instead of raw CRDs)
- SaaS or on-prem options
- Built-in UI for cluster monitoring
- Automatic Talos + Kubernetes upgrades

## API Access

Sidero Metal uses **Kubernetes API** (no separate REST API):

```bash
# Set KUBECONFIG to management cluster
export KUBECONFIG=~/.kube/config-management

# List servers
kubectl get servers -o wide

# Get server details
kubectl get server server-001 -o yaml

# List server classes
kubectl get serverclasses

# List CAPI clusters
kubectl get clusters -A
```

## Server Lifecycle Management

### Create/Update Server

```bash
# Apply Server resource
kubectl apply -f server-001.yaml

# Patch server to accept it
kubectl patch server server-001 -p '{"spec":{"accepted":true}}' --type=merge

# Add config patches
kubectl patch server server-001 --type=merge -p '
spec:
  configPatches:
    - op: add
      path: /machine/network/hostname
      value: "node-001"
'
```

### Decommission Server

```bash
# Delete CAPI Machine first (if in use)
kubectl delete machine workload-cluster-1-cp-0

# Set server to not accepted
kubectl patch server server-001 -p '{"spec":{"accepted":false}}' --type=merge

# Wipe server (if ServerBMC configured)
kubectl patch server server-001 -p '{"spec":{"pxeBootAlways":true}}' --type=merge
```

## Network Configuration

### DHCP (Recommended)

```yaml
spec:
  configPatches:
    - op: add
      path: /machine/network/interfaces
      value:
        - deviceSelector:
            physical: true
          dhcp: true
```

### Static IP

```yaml
spec:
  configPatches:
    - op: add
      path: /machine/network/interfaces
      value:
        - deviceSelector:
            physical: true
          addresses:
            - 10.0.1.50/24
          routes:
            - network: 0.0.0.0/0
              gateway: 10.0.1.1
          nameservers:
            - 8.8.8.8
            - 1.1.1.1
```

## Monitoring & Debugging

### Check Server Status

```bash
# View server resources
kubectl get servers -o wide

# Check server events
kubectl describe server server-001

# View SideroLink status
kubectl get siderolink -A
```

### Check Cluster Provisioning

```bash
# View CAPI clusters
kubectl get clusters -A

# Check machine status
kubectl get machines -A

# View control plane status
kubectl get taloscontrolplanes -A

# Check events
kubectl get events -A --sort-by='.lastTimestamp'
```

### Common Issues

1. **Server not discovered**
   - Check PXE boot configuration (DHCP option 66/67)
   - Verify SideroLink connectivity
   - Check management cluster logs: `kubectl logs -n sidero-system -l app=sidero-controller-manager`

2. **Server stuck in "Pending"**
   - Ensure server is accepted: `kubectl patch server <name> -p '{"spec":{"accepted":true}}'`
   - Check BMC connectivity if using IPMI/Redfish
   - Verify ServerClass qualifiers match server hardware

3. **Cluster creation fails**
   - Check Cluster API controller logs
   - Verify Talos version compatibility
   - Ensure sufficient accepted servers available

## Integration with cto-metal CLI

### Proposed Commands

```bash
# Initialize Sidero Metal on management cluster
cto-metal provider add sidero --kubeconfig ~/.kube/config-mgmt

# Discover servers (wait for PXE registration)
cto-metal sidero servers list

# Accept server for provisioning
cto-metal sidero server accept server-001

# Create workload cluster
cto-metal sidero cluster create \
  --name workload-1 \
  --control-plane-nodes 3 \
  --worker-nodes 5 \
  --server-class control-plane \
  --talos-version v1.8.4

# Scale cluster
cto-metal sidero cluster scale workload-1 --workers 10

# Delete cluster
cto-metal sidero cluster delete workload-1
```

## References

- **Documentation**: https://www.sidero.dev/docs/
- **GitHub**: https://github.com/siderolabs/sidero
- **CAPI Docs**: https://cluster-api.sigs.k8s.io/
- **Talos Bootstrap Provider**: https://github.com/siderolabs/cluster-api-bootstrap-provider-talos
- **Community**: Slack (#sidero in Talos Community)

## Comparison: Sidero Metal vs Omni

| Feature | Sidero Metal | Omni |
|---------|-------------|------|
| **Interface** | Kubernetes CRDs (kubectl) | REST/gRPC API + UI |
| **Deployment** | Self-hosted only | SaaS or self-hosted |
| **Ease of Use** | Requires CAPI knowledge | Simplified, opinionated |
| **Flexibility** | Full CAPI customization | Opinionated workflows |
| **Cost** | Free (Apache 2.0) | Free (non-production), paid (production) |
| **Best For** | Advanced users, CAPI workflows | Quick setup, managed experience |

**Recommendation**: Use **Omni** for CTO platform integration — simpler API, better UX, and supports SaaS model. Use Sidero Metal if you need full CAPI control or prefer 100% self-hosted.

## Source Code (Future Implementation)

| File | Purpose |
|------|---------|
| `crates/metal/src/providers/sidero/client.rs` | Kubernetes API client for Sidero CRDs |
| `crates/metal/src/providers/sidero/models.rs` | Server, ServerClass, Environment types |
| `crates/metal/src/providers/sidero/cluster.rs` | CAPI cluster creation/management |
