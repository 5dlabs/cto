# Installer Coordination Response

Response from the installer crate consolidating all work from both worktrees.

---

## ✅ All Integration Complete

Both crates have been fully integrated and tested. Ready for 2-node cluster deployment.

### Features Consolidated from uey Worktree

| Feature | Status | Location |
|---------|--------|----------|
| `InventoryManager` | ✅ | `metal::inventory` |
| `with_hugepages()` | ✅ | `metal::talos::BootstrapConfig` |
| `with_mayastor_ready()` | ✅ | `metal::talos::BootstrapConfig` |
| Auto-region selection | ✅ | `installer::bare_metal` |

---

## 📋 Installation Flow (22 Steps)

```
Phase 1 - Infrastructure (1-5):
  ValidatingPrerequisites → CreatingServers → WaitingServersReady
  → BootingTalos → WaitingTalosMaintenance

Phase 2 - Talos Bootstrap (6-12):
  GeneratingConfigs → ApplyingCPConfig → WaitingCPInstall → Bootstrapping
  → WaitingKubernetes → ApplyingWorkerConfig → WaitingWorkerJoin

Phase 3 - Platform Stack (13-16):
  DeployingCilium → DeployingBootstrapResources
  → DeployingLocalPathProvisioner → DeployingArgoCD

Phase 4 - GitOps (17-19):
  WaitingArgoCDReady → ApplyingAppOfApps → WaitingGitOpsSync

Phase 5 - Post-Install (20-22):
  ConfiguringStorage → ConfiguringKubeconfig → Complete
```

---

## 🚀 Usage

### Basic 2-Node Cluster

```bash
cto install \
  --cluster-name cto-prod \
  --region MIA2 \
  --nodes 2 \
  --cp-plan c2-small-x86 \
  --worker-plan c2-small-x86
```

### Auto-Region Selection

```bash
cto install \
  --cluster-name cto-prod \
  --auto-region \
  --fallback-regions MIA2,DAL,ASH,LAX \
  --nodes 2
```

### Full Options

```bash
cto install \
  --cluster-name cto-prod \
  --region MIA2 \
  --nodes 2 \
  --cp-plan c2-small-x86 \
  --worker-plan c2-small-x86 \
  --storage-disk /dev/nvme1n1 \
  --storage-replicas 2 \
  --talos-version v1.9.0 \
  --sync-timeout 30 \
  --profile standard
```

---

## 🗃️ Storage Architecture

```
Bootstrap (Direct Helm):
  └── local-path-provisioner  ← For ArgoCD etcd PVCs

GitOps (sync-wave: -10):
  └── Mayastor
      - infra/gitops/applications/storage/mayastor.yaml
      - HugePages required in Talos config

Post-Install:
  └── ConfiguringStorage step
      - Creates DiskPools on each node
      - Creates mayastor-nvme StorageClass as default
```

### Talos HugePages Configuration

The `metal` crate now automatically configures HugePages when using:

```rust
// In talos bootstrap
let config = BootstrapConfig::new("cluster", "10.0.0.1")
    .with_mayastor_ready()  // Cilium CNI + HugePages
    .with_pod_cidr("10.1.0.0/16");

// Or manually:
let config = config.with_hugepages();  // 1024 * 2MB = 2GiB
let config = config.with_hugepages_count(2048);  // 4GiB
```

---

## 📦 New CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `--auto-region` | Auto-select region based on stock | false |
| `--fallback-regions` | Preferred regions for auto-select | MIA2,DAL,ASH,LAX |
| `--storage-disk` | NVMe disk for Mayastor | (uses install-disk) |
| `--storage-replicas` | Mayastor replica count (1-3) | 2 |

---

## 🧪 Test Status

```
metal crate:     24 tests passing
installer crate: 16 tests passing
```

### New Tests Added

- `test_hugepages_config`
- `test_hugepages_custom_count`
- `test_mayastor_ready_config`
- `test_stock_level_*` (inventory)
- `test_plan_availability_*` (inventory)

---

## 📁 Files Changed

### Metal Crate

| File | Change |
|------|--------|
| `src/inventory.rs` | **NEW** - Stock availability & region selection |
| `src/lib.rs` | Export inventory module |
| `src/talos/bootstrap.rs` | Add `with_hugepages()`, `with_mayastor_ready()` |

### Installer Crate

| File | Change |
|------|--------|
| `src/config.rs` | Add `auto_region`, `fallback_regions`, `storage_disk`, `storage_replicas` |
| `src/state.rs` | Add `selected_region`, `ConfiguringStorage`, `ConfiguringKubeconfig` steps |
| `src/commands/install.rs` | Add `--auto-region`, `--fallback-regions`, `--storage-*` flags |
| `src/bare_metal.rs` | Add `select_region()` using `InventoryManager` |
| `src/orchestrator.rs` | Add `configure_storage()`, `configure_kubeconfig()` |

### GitOps

| File | Change |
|------|--------|
| `applications/storage/mayastor.yaml` | **NEW** - Mayastor ArgoCD Application |
| `cluster-config/mayastor-namespace.yaml` | **NEW** - Privileged namespace |
| `app-of-apps.yaml` | Include `applications/storage` |
| `projects/platform-project.yaml` | Add OpenEBS repo & CRDs |

---

## 🎯 Ready for Deployment

The codebase is ready to deploy a 2-node Mayastor cluster:

```bash
# Set credentials
export LATITUDE_API_KEY=xxx
export LATITUDE_PROJECT_ID=xxx

# Deploy
cto install \
  --cluster-name cto-prod \
  --auto-region \
  --nodes 2 \
  --storage-disk /dev/nvme1n1
```

### What Happens

1. **Region Selection**: Checks stock, picks best available region
2. **Server Creation**: Creates 1 CP + 1 worker in selected region
3. **Talos Boot**: iPXE boots Talos with HugePages configured
4. **Bootstrap**: Generates configs, bootstraps Kubernetes
5. **CNI**: Deploys Cilium (kube-proxy replacement)
6. **ArgoCD**: Deploys ArgoCD, applies app-of-apps
7. **GitOps Sync**: Mayastor, cert-manager, and all workloads sync
8. **Storage Setup**: Creates DiskPools and StorageClass
9. **Kubeconfig**: Merges into ~/.kube/config
10. **Complete**: Cluster ready with Lens instructions

---

*Last updated: December 12, 2025*
