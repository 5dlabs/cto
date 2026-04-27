# Provider Skill: Omni

Omni is a **SaaS-first Kubernetes management platform** built on Talos Linux that simplifies cluster creation, management, and lifecycle operations. It provides a unified API for managing hardware, operating system, and Kubernetes across bare-metal, cloud, edge, and hybrid environments.

## Overview

- **Vendor**: Sidero Labs (makers of Talos Linux)
- **License**: Commercial (production), Free (non-production/home labs)
- **Deployment**: SaaS (primary) or self-hosted (commercial license required)
- **Website**: https://www.siderolabs.com/omni/
- **Docs**: https://docs.siderolabs.com/omni/
- **Trial**: 2-week free trial at https://signup.siderolabs.io

## Why Omni?

- **Single API**: Unified REST/gRPC API for hardware, OS, and Kubernetes management
- **Automatic Upgrades**: OS and Kubernetes upgrades with zero-downtime rolling updates
- **Built-in HA**: Automatic highly available Kubernetes API endpoints
- **Secure by Default**: WireGuard-encrypted communications, identity provider integration
- **Multi-Environment**: Works on bare-metal, cloud, edge, on-premises, and hybrid setups
- **Native Talos**: Built specifically for Talos Linux — no SSH, no shell access, immutable OS

## Architecture

### Components

1. **Omni Backend (Control Plane)**
   - SaaS: Hosted by Sidero Labs (https://omni.siderolabs.com)
   - Self-Hosted: Run on your own infrastructure (commercial license required)

2. **Omni Agent (talos-metal-agent)**
   - Runs on bare-metal servers as Talos system extension
   - Auto-registers servers with Omni backend
   - Handles BMC auto-configuration (IPMI credentials)

3. **omnictl CLI**
   - Command-line interface for Omni API
   - Cluster creation, machine management, kubeconfig download

4. **Omni UI**
   - Web-based dashboard for cluster visualization
   - Single sign-on (SSO) with OIDC providers
   - Real-time cluster health and events

## Getting Started

### 1. Sign Up for Omni SaaS

Visit https://signup.siderolabs.io for a **2-week free trial**.

### 2. Install omnictl CLI

```bash
# macOS
brew install siderolabs/tap/omnictl

# Linux (x86_64)
curl -Lo omnictl https://github.com/siderolabs/omni/releases/latest/download/omnictl-linux-amd64
chmod +x omnictl
sudo mv omnictl /usr/local/bin/

# Verify installation
omnictl version
```

### 3. Authenticate with Omni

```bash
# Login to Omni SaaS
omnictl login

# Or authenticate with API key
export OMNI_ENDPOINT="https://omni.siderolabs.com:443"
export OMNI_API_KEY="your-api-key"
```

## API Access

Omni provides **gRPC** and **REST** APIs for programmatic access.

### Authentication

- **Web UI**: OIDC/SSO via identity providers (Google, GitHub, Okta, etc.)
- **API**: Service account tokens or API keys
- **omnictl**: Interactive login or `OMNI_API_KEY` environment variable

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/omni.management.Omni/CreateCluster` | Create new cluster |
| `GET` | `/omni.management.Omni/ListClusters` | List all clusters |
| `GET` | `/omni.management.Omni/GetCluster` | Get cluster details |
| `DELETE` | `/omni.management.Omni/DestroyCluster` | Delete cluster |
| `POST` | `/omni.management.Omni/CreateMachine` | Add machine to cluster |
| `GET` | `/omni.management.Omni/ListMachines` | List all machines |
| `GET` | `/omni.management.Omni/GetKubeconfig` | Download kubeconfig |

**API Documentation**: https://docs.siderolabs.com/omni/reference/api

## Bare-Metal Infrastructure Provider

Omni includes a **Bare-Metal Infrastructure Provider** for automated out-of-band provisioning.

### Features

- **IPMI/Redfish Automation**: Auto-configures BMC settings (IP, username, password)
- **talos-metal-agent Extension**: Runs on Talos machines to register with Omni
- **Automatic Discovery**: Servers PXE boot, install Talos, and register with Omni
- **Multi-Architecture**: Supports x86_64 and ARM64 (including Apple Silicon via Scaleway)

### Setup Steps

1. **Boot servers via PXE with talos-metal-agent extension**
2. **Servers auto-register with Omni backend**
3. **Accept machines in Omni UI or via `omnictl`**
4. **Configure BMC settings** (IP, credentials) — agent auto-applies via IPMI
5. **Create clusters** by assigning machines to control-plane or worker roles

**Documentation**: https://docs.siderolabs.com/omni/omni-cluster-setup/setting-up-the-bare-metal-infrastructure-provider

## Cluster Management

### Create Cluster

```bash
# Create cluster via omnictl
omnictl cluster create my-cluster \
  --talos-version v1.8.4 \
  --kubernetes-version v1.31.0 \
  --machines machine-001,machine-002,machine-003 \
  --control-plane-machines 3
```

### Scale Cluster

```bash
# Add worker nodes
omnictl machine create worker-004 --cluster my-cluster --role worker

# Remove worker nodes
omnictl machine delete worker-004
```

### Upgrade Cluster

```bash
# Upgrade Talos version (rolling update, zero-downtime)
omnictl cluster update my-cluster --talos-version v1.9.0

# Upgrade Kubernetes version
omnictl cluster update my-cluster --kubernetes-version v1.32.0
```

### Get Kubeconfig

```bash
# Download kubeconfig for cluster access
omnictl kubeconfig my-cluster > ~/.kube/config-my-cluster

# Use with kubectl
export KUBECONFIG=~/.kube/config-my-cluster
kubectl get nodes
```

### Delete Cluster

```bash
# Destroy cluster (releases machines back to pool)
omnictl cluster destroy my-cluster
```

## Machine Management

### List Machines

```bash
# List all machines
omnictl machine list

# List machines in specific cluster
omnictl machine list --cluster my-cluster
```

### Accept Machine

```bash
# Accept discovered machine (make it available for provisioning)
omnictl machine accept machine-001
```

### Configure Machine

```bash
# Apply Talos machine config patches
omnictl machine patch machine-001 \
  --patch '[{"op":"add","path":"/machine/network/hostname","value":"node-001"}]'
```

## Networking

### WireGuard Mesh

Omni uses **WireGuard** for encrypted node-to-Omni communication:
- All traffic encrypted between machines and Omni backend
- Optional inter-node encryption for spanning insecure networks
- Automatic tunnel setup — no manual VPN configuration

### Cluster Endpoints

Omni provides **automatic highly available API endpoints**:
- No need for external load balancers
- Built-in failover between control-plane nodes
- Secure access via kubeconfig with embedded certificates

## Integration with CTO Platform

### Option 1: omnictl CLI Wrapper

Wrap `omnictl` commands in `cto-metal` CLI:

```bash
# Initialize Omni provider
cto-metal provider add omni --endpoint https://omni.siderolabs.com:443

# Authenticate
cto-metal omni login

# Create cluster
cto-metal omni cluster create my-cluster \
  --control-plane-nodes 3 \
  --worker-nodes 5 \
  --talos-version v1.8.4

# Get kubeconfig
cto-metal omni kubeconfig my-cluster > ~/.kube/config-my-cluster
```

### Option 2: Native Rust Client (Future)

Implement native Rust client using Omni's gRPC API:

```rust
// Pseudo-code for crates/metal/src/providers/omni/client.rs
pub struct OmniProvider {
    grpc_client: omni::ManagementClient,
    api_key: String,
}

impl OmniProvider {
    pub async fn create_cluster(&self, config: ClusterConfig) -> Result<Cluster> {
        let request = CreateClusterRequest {
            name: config.name,
            talos_version: config.talos_version,
            kubernetes_version: config.kubernetes_version,
            control_plane_count: config.control_plane_nodes,
        };

        let response = self.grpc_client.create_cluster(request).await?;
        Ok(response.into_inner())
    }

    pub async fn scale_cluster(&self, cluster_id: &str, workers: usize) -> Result<()> {
        // Scale worker nodes via Omni API
    }

    pub async fn get_kubeconfig(&self, cluster_id: &str) -> Result<String> {
        let request = GetKubeconfigRequest { cluster_id };
        let response = self.grpc_client.get_kubeconfig(request).await?;
        Ok(response.into_inner().kubeconfig)
    }
}
```

## Pricing

| Tier | Cost | Use Case |
|------|------|----------|
| **Free** | $0 | Non-production (home labs, testing) |
| **Production** | Contact sales | Commercial deployments |
| **Self-Hosted** | Commercial license required | On-premises, air-gapped environments |

**Trial**: 2-week free trial for evaluation (https://signup.siderolabs.io)

**Contact**: sales@siderolabs.com for commercial pricing and credits

## Features Comparison: Omni SaaS vs Self-Hosted

| Feature | SaaS | Self-Hosted |
|---------|------|-------------|
| **Hosting** | Managed by Sidero Labs | Your infrastructure |
| **Updates** | Automatic | Manual |
| **Cost** | Free (non-prod) + paid (prod) | Commercial license |
| **SSO/OIDC** | Included | Included |
| **API Access** | Included | Included |
| **Support** | Enterprise support available | Enterprise support available |
| **Air-Gapped** | No | Yes |
| **Data Sovereignty** | US/EU regions | Full control |

## Monitoring & Observability

### Omni UI Dashboard

- Real-time cluster health
- Machine status and events
- Kubernetes version tracking
- Talos upgrade status
- Audit logs for all operations

### Integration with Prometheus/Grafana

Omni clusters include:
- Built-in metrics via Talos metrics endpoint
- Integration with Prometheus Operator
- Pre-configured Grafana dashboards

## Advanced Features

### Cluster Templates

Define reusable cluster configurations:

```yaml
apiVersion: omni.sidero.dev/v1alpha1
kind: ClusterTemplate
metadata:
  name: production-cluster
spec:
  talosVersion: v1.8.4
  kubernetesVersion: v1.31.0
  controlPlane:
    replicas: 3
    machineClass: high-memory
  workers:
    replicas: 5
    machineClass: standard
  patches:
    - path: /machine/network/nameservers
      value: ["1.1.1.1", "8.8.8.8"]
```

### Kernel Arguments Management

Manage kernel arguments directly via Omni UI or API (added Feb 2026):

```bash
omnictl machine patch machine-001 \
  --patch '[{"op":"add","path":"/machine/kernel/args","value":["console=ttyS0,115200"]}]'
```

### gRPC Tunnel Mode Switching

Switch tunnel modes for connected machines (added Feb 2026):

```bash
# Switch to direct WireGuard tunnel
omnictl machine tunnel machine-001 --mode wireguard

# Switch to gRPC tunnel (for restrictive networks)
omnictl machine tunnel machine-001 --mode grpc
```

## Debugging & Troubleshooting

### Check Machine Status

```bash
# View machine details
omnictl machine get machine-001

# Check machine logs
omnictl machine logs machine-001

# View machine events
omnictl machine events machine-001
```

### Check Cluster Health

```bash
# View cluster status
omnictl cluster get my-cluster

# Check cluster events
omnictl cluster events my-cluster

# Validate cluster health
omnictl cluster validate my-cluster
```

### Common Issues

1. **Machine not registering**
   - Verify talos-metal-agent extension is installed
   - Check network connectivity to Omni backend
   - Ensure PXE boot is configured correctly

2. **BMC auto-configuration failed**
   - Verify IPMI/Redfish is accessible from Talos machine
   - Check BMC credentials in Omni UI
   - Manually configure BMC settings if auto-detection fails

3. **Cluster creation stuck**
   - Check machine availability (`omnictl machine list`)
   - Verify sufficient accepted machines
   - Review cluster events for errors

## Integration with cto-metal CLI

### Proposed Commands

```bash
# Add Omni provider
cto-metal provider add omni --endpoint https://omni.siderolabs.com:443

# Authenticate
cto-metal omni login

# List available machines
cto-metal omni machines list

# Accept machine
cto-metal omni machine accept machine-001

# Create cluster
cto-metal omni cluster create workload-1 \
  --control-plane 3 \
  --workers 5 \
  --talos-version v1.8.4 \
  --kubernetes-version v1.31.0

# Get kubeconfig
cto-metal omni kubeconfig workload-1 > ~/.kube/config-workload-1

# Scale cluster
cto-metal omni cluster scale workload-1 --workers 10

# Upgrade cluster
cto-metal omni cluster upgrade workload-1 --talos v1.9.0

# Delete cluster
cto-metal omni cluster delete workload-1
```

## References

- **Website**: https://www.siderolabs.com/omni/
- **Documentation**: https://docs.siderolabs.com/omni/
- **GitHub**: https://github.com/siderolabs/omni
- **API Docs**: https://docs.siderolabs.com/omni/reference/api
- **Trial Signup**: https://signup.siderolabs.io
- **Community**: Slack (#omni in Talos Community)
- **Sales**: sales@siderolabs.com

## Comparison: Omni vs Sidero Metal

| Feature | Omni | Sidero Metal |
|---------|------|--------------|
| **Interface** | REST/gRPC API + UI | Kubernetes CRDs (kubectl) |
| **Deployment** | SaaS or self-hosted | Self-hosted only |
| **Ease of Use** | Simple, opinionated | Requires CAPI knowledge |
| **Cluster Creation** | Single command | Multiple CAPI resources |
| **Upgrades** | Automatic, zero-downtime | Manual via CAPI |
| **SSO/OIDC** | Built-in | Not included |
| **Cost** | Free (non-prod), paid (prod) | Free (Apache 2.0) |
| **Best For** | Quick setup, managed experience | Advanced users, CAPI workflows |

**Recommendation**: Use **Omni** for CTO platform integration — simpler API, better UX, automatic upgrades, and supports both SaaS and self-hosted deployments.

## Partnership Opportunity

**CTO Platform + Omni Integration Benefits:**
- CTO provides autonomous AI development teams
- Omni provides seamless bare-metal Kubernetes management
- **Together**: Complete end-to-end platform from bare-metal provisioning to production deployment

**Potential Partnership:**
- Cross-promotion (CTO recommends Omni, Omni highlights CTO use case)
- Joint blog posts and case studies
- Trial credits for CTO users on Omni platform
- Technical collaboration on API integration

**Contact**: sales@siderolabs.com for partnership inquiries

## Source Code (Future Implementation)

| File | Purpose |
|------|---------|
| `crates/metal/src/providers/omni/client.rs` | gRPC client for Omni API |
| `crates/metal/src/providers/omni/models.rs` | Cluster, Machine, Kubeconfig types |
| `crates/metal/src/providers/omni/auth.rs` | API key authentication |
