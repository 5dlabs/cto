# CTO Mirror Deployment Plan

## Objective

Create a complete E2E deployment tool that provisions a **3-node bare metal cluster** with the full CTO platform stack, mirroring our production "Simple Cluster" environment.

---

## Phase 1: Discovery - Current Simple Cluster Components

### Infrastructure Layer (Talos + Kubernetes)

| Component | Current State | Notes |
|-----------|---------------|-------|
| Talos Linux | v1.10.4 | Immutable Linux for K8s |
| Kubernetes | v1.33.1 | Via Talos bootstrap |
| Nodes | 3+ (CP + workers) | Production requires 3 for Mayastor quorum |
| Storage | Local Path Provisioner | Need to add Mayastor for HA |

### GitOps Applications (from `infra/gitops/applications/`)

| Category | Applications |
|----------|-------------|
| **Core Platform** | ArgoCD, Cert-Manager, Vault, Vault Secrets Operator |
| **Ingress/Networking** | Ingress-NGINX, Cloudflare Tunnel, Cloudflare Operator, External-DNS, Gateway API, Kilo (WireGuard) |
| **CI/CD** | Argo Workflows, Argo Events, ARC Controller (GitHub runners), Platform Runners |
| **Databases** | CloudNative-PG Operator, Redis Operator, MinIO Operator + Tenant |
| **Observability** | Victoria Metrics, Victoria Logs, Grafana, Fluent-bit, OTEL Collector, Kube State Metrics, Metrics Server |
| **CTO Platform** | Controller, Tools (MCP servers), Heal, OpenMemory |
| **Webhooks/Sensors** | GitHub Webhooks, Bolt Sensor, Stitch Sensor, CI Remediation Sensor |
| **Misc** | ArgoCD Image Updater, TweakCN, Workspace Maintenance |

### Helm Charts (from `infra/charts/`)

| Chart | Purpose |
|-------|---------|
| `controller` | CTO agent controller + workflow templates |
| `tools` | MCP proxy server + tool management |
| `openmemory` | AI memory/context storage |
| `universal-app` | Generic app deployment template |
| `vault` | Vault configuration values |
| `workflow-templates` | Argo Workflow templates |

### Storage Requirements

| Component | Storage Need | Solution |
|-----------|--------------|----------|
| Vault | Persistent HA storage | Mayastor (3-way repl) |
| PostgreSQL | High-perf database storage | Mayastor |
| MinIO | Object storage | Mayastor |
| Redis | Cache/queue storage | Mayastor or ephemeral |
| OpenMemory | AI context storage | Local Path or Mayastor |

---

## Phase 2: Target Architecture

### 3-Node Cluster Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     CTO Mirror Cluster (3 nodes)                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐       │
│  │   Node 1 (CP)   │   │   Node 2 (WK)   │   │   Node 3 (WK)   │       │
│  │  c2-small-x86   │   │  c2-small-x86   │   │  c2-small-x86   │       │
│  │                 │   │                 │   │                 │       │
│  │  • Control Plane│   │  • Worker       │   │  • Worker       │       │
│  │  • etcd         │   │  • Mayastor IO  │   │  • Mayastor IO  │       │
│  │  • Mayastor IO  │   │  • Workloads    │   │  • Workloads    │       │
│  │                 │   │                 │   │                 │       │
│  │  NVMe: /dev/sda │   │  NVMe: /dev/sda │   │  NVMe: /dev/sda │       │
│  └─────────────────┘   └─────────────────┘   └─────────────────┘       │
│           │                     │                     │                 │
│           └─────────────────────┼─────────────────────┘                 │
│                                 │                                       │
│                    ┌────────────┴────────────┐                          │
│                    │   Mayastor Replication  │                          │
│                    │      (3-way sync)       │                          │
│                    └─────────────────────────┘                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Cost Estimate

| Resource | Hourly | Daily | Monthly |
|----------|--------|-------|---------|
| 3x c2-small-x86 | $0.54 | $12.96 | ~$389 |

---

## Phase 3: Implementation Tasks

### 3.1 Rust CLI Updates (`crates/metal`)

| Task | Description | Priority |
|------|-------------|----------|
| **3-node cluster support** | Update `cluster` command for N workers | P0 |
| **Mayastor deployment** | Add `stack::deploy_mayastor()` | P0 |
| **Talos config patches** | HugePages, NVMe-oF modules | P0 |
| **App-of-Apps deployment** | Apply ArgoCD Application | P1 |
| **OpenLens bundling** | Download/install OpenLens | P2 |

### 3.2 Stack Deployment Order

```
1. local-path-provisioner  (immediate storage for bootstrap)
2. cert-manager            (TLS prerequisites)
3. mayastor                (HA storage - requires 3 nodes)
4. vault                   (secrets - uses Mayastor PVs)
5. argocd                  (GitOps controller)
6. ingress-nginx           (external access)
7. argo-workflows          (CI/CD engine)
8. argo-events             (webhook triggers)
9. app-of-apps             (deploys remaining apps via GitOps)
```

### 3.3 Talos Config Patches Needed

```yaml
# Worker nodes for Mayastor
machine:
  sysctls:
    vm.nr_hugepages: "512"  # 1GB HugePages for SPDK
  kernel:
    modules:
      - name: nvme_tcp
      - name: nvmet_tcp
  nodeLabels:
    openebs.io/engine: mayastor
```

### 3.4 OpenLens Integration

**Source:** https://github.com/MuhammedKalkan/OpenLens
**License:** MIT (open source, no login required)
**Installation:**
- macOS: `brew install --cask openlens`
- Linux: Download `.AppImage` or `.deb`
- Windows: `winget install openlens`

**CLI Integration:**
```bash
# After cluster provisioning
metal cluster --name cto-mirror ... --install-lens

# Outputs:
# 1. Kubeconfig saved to /tmp/cto-cluster/kubeconfig
# 2. OpenLens installed (if not present)
# 3. Kubeconfig added to OpenLens
```

---

## Phase 4: CLI Command Design

### New `metal cluster` Flags

```bash
metal cluster \
  --name cto-mirror \
  --region MIA2 \
  --cp-plan c2-small-x86 \
  --worker-plan c2-small-x86 \
  --worker-count 2 \           # NEW: Multiple workers
  --ssh-keys ssh_xxx \
  --output-dir /tmp/cto-cluster \
  --deploy-stack \             # Deploy full stack
  --init-vault \               # Init/unseal Vault
  --mayastor \                 # Enable Mayastor storage
  --install-lens               # NEW: Install OpenLens
```

### New `metal stack` Options

```bash
metal stack \
  --kubeconfig /tmp/cto-cluster/kubeconfig \
  --mayastor \                 # Deploy Mayastor
  --app-of-apps \              # Deploy ArgoCD app-of-apps
  --gitops-repo https://github.com/5dlabs/cto
```

---

## Phase 5: Deliverables

### Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/metal/src/bin/metal.rs` | Modify | Add `--worker-count`, `--mayastor`, `--install-lens` |
| `crates/metal/src/stack/mayastor.rs` | Create | Mayastor Helm deployment |
| `crates/metal/src/stack/app_of_apps.rs` | Create | ArgoCD app-of-apps deployment |
| `crates/metal/src/tools/openlens.rs` | Create | OpenLens installation helper |
| `infra/gitops/applications/mayastor.yaml` | Create | ArgoCD Application for Mayastor |

### Test Plan

1. **Unit tests** - State machine, config generation
2. **Integration test** - 3-node cluster on Latitude
3. **E2E test** - Full stack deployment + workload test

---

## Phase 6: Execution Order

| Step | Task | Estimated Time |
|------|------|----------------|
| 1 | Add `--worker-count` support | 30 min |
| 2 | Add Mayastor deployment module | 2 hr |
| 3 | Add Talos config patches for Mayastor | 1 hr |
| 4 | Add app-of-apps deployment | 1 hr |
| 5 | Add OpenLens installation helper | 30 min |
| 6 | Test on 3-node cluster | 1-2 hr |
| 7 | Documentation | 30 min |

**Total estimated: 6-7 hours of development**

---

## Appendix: Current Stack Module Functions

Already implemented in `crates/metal/src/stack/mod.rs`:

- ✅ `deploy_cert_manager()`
- ✅ `deploy_argocd()`
- ✅ `deploy_vault()`
- ✅ `deploy_ingress_nginx()`
- ✅ `deploy_argo_workflows()`
- ✅ `deploy_local_path_provisioner()`
- ✅ `init_vault()`
- ✅ `unseal_vault()`
- ✅ `get_argocd_password()`

Need to add:
- 🔲 `deploy_mayastor()`
- 🔲 `deploy_app_of_apps()`
- 🔲 `install_openlens()`













