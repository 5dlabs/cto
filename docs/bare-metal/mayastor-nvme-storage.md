# OpenEBS Mayastor: High-Performance NVMe Storage for Bare Metal

This document describes how to deploy OpenEBS Mayastor for high-performance distributed
NVMe storage on bare metal Talos Kubernetes clusters. This is a core component of our
**private paid offering**, providing vSAN-like distributed storage with enterprise-grade
fault tolerance.

## Overview

Mayastor is a cloud-native storage solution designed specifically for NVMe drives.
It uses NVMe-oF (NVMe over Fabrics) for replication, preserving the low-latency
characteristics of NVMe while providing data redundancy across nodes.

### Key Features

- **Written in Rust** using Intel's SPDK (Storage Performance Development Kit)
- **NVMe-oF native** - replication at the NVMe protocol level
- **Minimal overhead** - designed to preserve NVMe performance (~5-10% overhead)
- **Synchronous replication** with configurable replica count
- **CSI compliant** - standard Kubernetes storage interface
- **CNCF project** with active development
- **vSAN-like experience** - aggregates local NVMe across nodes into distributed pool

### When to Use Mayastor

| Use Case | Recommendation |
|----------|----------------|
| High-performance databases (PostgreSQL, MySQL) | ✅ Excellent |
| AI/ML workloads with fast storage needs | ✅ Excellent |
| Stateful applications needing replication | ✅ Excellent |
| Production workloads requiring HA | ✅ Excellent |
| Simple single-node dev/test | ⚠️ Works, but Local Path is simpler |
| Object storage (S3-like) | ❌ Use MinIO or Rook-Ceph |

## Cluster Requirements

### Minimum Node Configuration for Production

For **vSAN-like fault tolerance and data replication**, the following is required:

| Configuration | Minimum Nodes | Fault Tolerance | Recommendation |
|---------------|---------------|-----------------|----------------|
| `repl: "1"` | 1 | None | Dev/test only |
| `repl: "2"` | 2 | 1 node failure | Not recommended (no quorum) |
| `repl: "3"` | **3** | 1 node failure | ✅ **Production minimum** |
| `repl: "3"` | 4+ | 1-2 node failures | Optimal for rebuild performance |

> **Why 3 nodes?** Like vSAN, Mayastor requires quorum for cluster decisions.
> With 2 nodes, losing 1 means no majority (1 of 2). With 3 nodes, losing 1
> maintains majority (2 of 3), allowing continued operations.

### Production Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Kubernetes Cluster                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐       │
│  │   Worker 1      │   │   Worker 2      │   │   Worker 3      │       │
│  │                 │   │                 │   │                 │       │
│  │  ┌───────────┐  │   │  ┌───────────┐  │   │  ┌───────────┐  │       │
│  │  │ NVMe SSD  │  │   │  │ NVMe SSD  │  │   │  │ NVMe SSD  │  │       │
│  │  │ DiskPool  │  │   │  │ DiskPool  │  │   │  │ DiskPool  │  │       │
│  │  └─────┬─────┘  │   │  └─────┬─────┘  │   │  └─────┬─────┘  │       │
│  │        │        │   │        │        │   │        │        │       │
│  │  ┌─────┴─────┐  │   │  ┌─────┴─────┐  │   │  ┌─────┴─────┐  │       │
│  │  │ I/O Engine│  │   │  │ I/O Engine│  │   │  │ I/O Engine│  │       │
│  │  │  (SPDK)   │  │   │  │  (SPDK)   │  │   │  │  (SPDK)   │  │       │
│  │  └─────┬─────┘  │   │  └─────┬─────┘  │   │  └─────┬─────┘  │       │
│  └────────┼────────┘   └────────┼────────┘   └────────┼────────┘       │
│           │                     │                     │                 │
│           └─────────────────────┼─────────────────────┘                 │
│                                 │                                       │
│                    ┌────────────┴────────────┐                          │
│                    │   NVMe-oF Replication   │                          │
│                    │   (Synchronous 3-way)   │                          │
│                    └─────────────────────────┘                          │
│                                                                         │
│  Write Path: Data written to all 3 replicas before ACK                  │
│  Read Path:  Reads served from nearest/fastest replica                  │
│  Failure:    Automatic rebuild to surviving nodes                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Reference Cluster Configuration

Example development cluster (single-node, no replication):

- **Control Plane**: Intel Mac Mini at `192.168.1.77`
- **Worker Node**: Dell system with NVMe at `192.168.1.72` (`/dev/nvme0n1`)
- **Current Storage**: Local Path Provisioner with 100GB NVMe volume
- **Talos Version**: v1.10.4

## Prerequisites for Mayastor

### Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU cores per node | 2 dedicated | 4 dedicated |
| RAM per node | 4GB | 8GB+ |
| NVMe drives | 1 per node | 2+ per node |
| Network | 1 Gbps | 10 Gbps+ |

### System Requirements

1. **HugePages** - At least 1GB of 2MB HugePages per Mayastor node
2. **Kernel modules** - `nvme_tcp` and `nvmet_tcp` for NVMe-oF
3. **CPU isolation** - Dedicated cores for the I/O engine (optional but recommended)

## Talos Configuration Changes Required

### Worker Node Changes

The following changes are needed in `infra/talos/config/simple/worker.yaml`:

```yaml
machine:
  type: worker
  # ... existing config ...

  # ADD: HugePages for Mayastor SPDK
  sysctls:
    vm.nr_hugepages: "512"  # 1GB of 2MB HugePages (512 * 2MB = 1024MB)

  # ADD: NVMe-oF kernel modules
  kernel:
    modules:
      - name: nvme_tcp
      - name: nvmet_tcp

  # ADD: Label for Mayastor node selection
  nodeLabels:
    openebs.io/engine: mayastor

  # OPTIONAL: CPU isolation for I/O engine
  # Uncomment if you want dedicated cores for storage I/O
  # kubelet:
  #   extraArgs:
  #     system-reserved: "cpu=2"
  #     kube-reserved: "cpu=1"
```

### Control Plane Changes (if running Mayastor there)

For a single-node or dev cluster where control plane also runs storage:

```yaml
machine:
  type: controlplane
  # ... existing config ...

  sysctls:
    vm.nr_hugepages: "512"

  kernel:
    modules:
      - name: nvme_tcp
      - name: nvmet_tcp
```

### Full Worker Configuration Example

Here's the complete updated `worker.yaml` with Mayastor support:

```yaml
---
version: v1alpha1
debug: false
persist: true
machine:
  type: worker
  token: qt315g.8f9e1kb2r4p79efs
  ca:
    crt: <existing-cert>
    key: ""
  certSANs: []
  kubelet:
    image: ghcr.io/siderolabs/kubelet:v1.33.1
    defaultRuntimeSeccompProfileEnabled: true
    disableManifestsDirectory: true
    extraArgs:
      max-pods: "200"
    nodeIP:
      validSubnets:
        - 192.168.1.0/24
  network:
    interfaces:
      - interface: enp4s0
        dhcp: true

  # Mayastor requirements
  sysctls:
    vm.nr_hugepages: "512"  # 1GB of 2MB HugePages

  kernel:
    modules:
      - name: nvme_tcp
      - name: nvmet_tcp

  nodeLabels:
    openebs.io/engine: mayastor

  install:
    disk: /dev/nvme0n1
    image: ghcr.io/siderolabs/installer:v1.10.4
    wipe: true
    extraKernelArgs:
      - talos.install.disk=/dev/nvme0n1

  features:
    rbac: true
    stableHostname: true
    apidCheckExtKeyUsage: true
    diskQuotaSupport: true
    kubePrism:
      enabled: true
      port: 7445
    hostDNS:
      enabled: true
      forwardKubeDNSToHost: true

cluster:
  # ... existing cluster config unchanged ...
```

## Mayastor Deployment

### Step 1: Apply Talos Configuration Changes

```bash
# Apply the updated worker configuration
talosctl apply-config --talosconfig=config/simple/talosconfig \
  --nodes 192.168.1.72 \
  --file config/simple/worker.yaml

# The node will reboot to apply kernel module and sysctl changes
```

### Step 2: Verify HugePages

```bash
# Check HugePages allocation
talosctl -n 192.168.1.72 read /proc/meminfo | grep -i huge

# Expected output:
# HugePages_Total:     512
# HugePages_Free:      512
# HugePages_Rsvd:        0
# HugePages_Surp:        0
# Hugepagesize:       2048 kB
```

### Step 3: Verify Kernel Modules

```bash
# Check loaded modules
talosctl -n 192.168.1.72 read /proc/modules | grep nvme

# Expected output should include:
# nvme_tcp
# nvmet_tcp
```

### Step 4: Install Mayastor via Helm

```bash
# Add the OpenEBS Mayastor Helm repo
helm repo add mayastor https://openebs.github.io/mayastor-extensions/
helm repo update

# Create namespace
kubectl create namespace mayastor

# Install Mayastor
helm install mayastor mayastor/mayastor \
  --namespace mayastor \
  --set etcd.replicaCount=1 \
  --set io_engine.coreList="{2,3}" \
  --set io_engine.hugepages2Mi=1024Mi \
  --set mayastor.nodeSelector."openebs\.io/engine"=mayastor

# Wait for pods to be ready
kubectl -n mayastor wait --for=condition=ready pod --all --timeout=300s
```

### Step 5: Create Disk Pool

Identify an unused NVMe device or partition for Mayastor:

```bash
# List available disks on worker
talosctl -n 192.168.1.72 disks

# Note: You may need a second NVMe or partition the existing one
# Mayastor needs a raw block device, not a mounted filesystem
```

Create a DiskPool resource:

```yaml
# mayastor-diskpool.yaml
apiVersion: openebs.io/v1beta2
kind: DiskPool
metadata:
  name: pool-worker-1
  namespace: mayastor
spec:
  node: talos-worker  # Use actual node name from `kubectl get nodes`
  disks:
    - /dev/nvme1n1  # Second NVMe drive
    # OR use a partition if single drive:
    # - /dev/nvme0n1p3  # Third partition on main NVMe
```

```bash
kubectl apply -f mayastor-diskpool.yaml
```

### Step 6: Create StorageClass

```yaml
# mayastor-storageclass.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: mayastor-nvme
  annotations:
    storageclass.kubernetes.io/is-default-class: "false"
parameters:
  ioTimeout: "30"
  protocol: nvmf
  repl: "1"  # Single replica for single-node, use "2" or "3" for multi-node
  thin: "true"
provisioner: io.openebs.csi-mayastor
reclaimPolicy: Delete
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
```

```bash
kubectl apply -f mayastor-storageclass.yaml
```

## Testing Mayastor

### Create Test PVC

```yaml
# test-pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: mayastor-test-pvc
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: mayastor-nvme
  resources:
    requests:
      storage: 10Gi
```

### Run Performance Test

```yaml
# fio-benchmark.yaml
apiVersion: v1
kind: Pod
metadata:
  name: fio-benchmark
spec:
  containers:
    - name: fio
      image: nixery.dev/shell/fio
      command:
        - sh
        - -c
        - |
          echo "Running 4K random read/write test..."
          fio --name=randrw --ioengine=libaio --iodepth=32 \
              --rw=randrw --bs=4k --direct=1 --size=1G \
              --numjobs=4 --runtime=60 --group_reporting \
              --filename=/data/test.fio
      volumeMounts:
        - name: data
          mountPath: /data
  volumes:
    - name: data
      persistentVolumeClaim:
        claimName: mayastor-test-pvc
  restartPolicy: Never
```

## Comparison: Local Path vs Mayastor

| Feature | Local Path Provisioner | Mayastor |
|---------|----------------------|----------|
| **Setup complexity** | Simple | Moderate |
| **Performance** | Native NVMe | Near-native (~5-10% overhead) |
| **Data replication** | None | Yes (configurable) |
| **Multi-node access** | No | Yes (NVMe-oF) |
| **Snapshots** | No | Yes |
| **Volume expansion** | No | Yes |
| **Resource usage** | Minimal | 2-4 CPU cores, 1GB+ RAM |
| **Best for** | Dev/test, single-node | Production, HA requirements |

## Migration Path

To migrate from Local Path Provisioner to Mayastor:

1. **Parallel operation**: Keep both storage classes available
2. **New workloads**: Deploy with `mayastor-nvme` StorageClass
3. **Existing data**: Use Velero or manual backup/restore
4. **Gradual transition**: Move workloads one at a time

## Troubleshooting

### HugePages Not Available

```bash
# Check if HugePages are configured
talosctl -n 192.168.1.72 read /proc/meminfo | grep HugePages

# If HugePages_Total is 0, the sysctl wasn't applied
# Re-apply config and reboot the node
```

### Kernel Modules Not Loaded

```bash
# Verify modules are loaded
talosctl -n 192.168.1.72 read /proc/modules | grep nvme

# If missing, check Talos machine config has the kernel.modules section
```

### DiskPool Not Ready

```bash
# Check DiskPool status
kubectl -n mayastor get diskpool

# Check for errors
kubectl -n mayastor describe diskpool pool-worker-1

# Common issues:
# - Disk already has partitions/filesystem
# - Disk path incorrect
# - Insufficient permissions
```

### I/O Engine Pods Crashing

```bash
# Check pod logs
kubectl -n mayastor logs -l app=io-engine

# Common issues:
# - Not enough HugePages
# - CPU cores specified in coreList not available
# - Insufficient memory
```

## Alternative: Multiple Storage Classes

For flexibility, you can run both Local Path and Mayastor:

```yaml
# Keep local-path as default for simple workloads
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: local-path
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: rancher.io/local-path
# ...

---
# Use mayastor-nvme for high-performance needs
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: mayastor-nvme
  annotations:
    storageclass.kubernetes.io/is-default-class: "false"
provisioner: io.openebs.csi-mayastor
# ...
```

## Resources

- [OpenEBS Mayastor Documentation](https://mayastor.gitbook.io/introduction/)
- [Mayastor GitHub Repository](https://github.com/openebs/mayastor)
- [Talos Linux Storage Guide](https://www.talos.dev/latest/kubernetes-guides/storage/)
- [NVMe-oF Specification](https://nvmexpress.org/developers/nvme-of-specification/)

---

## Production Multi-Node Deployment

This section covers deploying Mayastor across multiple nodes for vSAN-like
distributed storage with full fault tolerance.

### Hardware Requirements per Node

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| CPU | 4 cores | 8+ cores | 2 cores dedicated to I/O engine |
| RAM | 8 GB | 16+ GB | 1-2 GB for HugePages |
| NVMe (OS) | 256 GB | 512 GB | Boot drive for Talos |
| NVMe (Storage) | 512 GB | 1+ TB | Dedicated Mayastor pool |
| Network | 1 Gbps | 10 Gbps | 25 Gbps ideal for heavy workloads |

### Recommended 3-Node Configuration

```yaml
# Node 1: 192.168.1.71
Worker Node 1:
  - OS Disk: /dev/nvme0n1 (256GB) - Talos installation
  - Storage Disk: /dev/nvme1n1 (1TB) - Mayastor DiskPool

# Node 2: 192.168.1.72
Worker Node 2:
  - OS Disk: /dev/nvme0n1 (256GB) - Talos installation
  - Storage Disk: /dev/nvme1n1 (1TB) - Mayastor DiskPool

# Node 3: 192.168.1.73
Worker Node 3:
  - OS Disk: /dev/nvme0n1 (256GB) - Talos installation
  - Storage Disk: /dev/nvme1n1 (1TB) - Mayastor DiskPool
```

### Multi-Node Talos Configuration

Apply to each worker node (`worker-1.yaml`, `worker-2.yaml`, `worker-3.yaml`):

```yaml
---
version: v1alpha1
debug: false
persist: true
machine:
  type: worker
  token: <cluster-token>
  ca:
    crt: <cluster-ca>
    key: ""

  # Mayastor requirements
  sysctls:
    vm.nr_hugepages: "1024"  # 2GB HugePages for production

  kernel:
    modules:
      - name: nvme_tcp
      - name: nvmet_tcp

  nodeLabels:
    openebs.io/engine: mayastor
    topology.kubernetes.io/zone: zone-a  # For topology-aware scheduling

  kubelet:
    image: ghcr.io/siderolabs/kubelet:v1.33.1
    defaultRuntimeSeccompProfileEnabled: true
    disableManifestsDirectory: true
    extraArgs:
      max-pods: "200"

  install:
    disk: /dev/nvme0n1  # OS disk
    image: ghcr.io/siderolabs/installer:v1.10.4
    wipe: false

  features:
    rbac: true
    stableHostname: true
    apidCheckExtKeyUsage: true
    diskQuotaSupport: true
    kubePrism:
      enabled: true
      port: 7445
    hostDNS:
      enabled: true
      forwardKubeDNSToHost: true

cluster:
  # ... cluster config ...
```

### Multi-Node Helm Installation

```bash
# Install Mayastor with production settings
helm install mayastor mayastor/mayastor \
  --namespace mayastor \
  --create-namespace \
  --set etcd.replicaCount=3 \
  --set etcd.persistence.enabled=true \
  --set etcd.persistence.size=2Gi \
  --set io_engine.coreList="{2,3}" \
  --set io_engine.hugepages2Mi=2048Mi \
  --set mayastor.nodeSelector."openebs\.io/engine"=mayastor \
  --set csi.node.nvme.io_timeout=60
```

### Create DiskPools for Each Node

```yaml
# diskpools.yaml
---
apiVersion: openebs.io/v1beta2
kind: DiskPool
metadata:
  name: pool-worker-1
  namespace: mayastor
spec:
  node: worker-1  # kubectl get nodes
  disks:
    - /dev/nvme1n1
---
apiVersion: openebs.io/v1beta2
kind: DiskPool
metadata:
  name: pool-worker-2
  namespace: mayastor
spec:
  node: worker-2
  disks:
    - /dev/nvme1n1
---
apiVersion: openebs.io/v1beta2
kind: DiskPool
metadata:
  name: pool-worker-3
  namespace: mayastor
spec:
  node: worker-3
  disks:
    - /dev/nvme1n1
```

```bash
kubectl apply -f diskpools.yaml

# Verify all pools are online
kubectl -n mayastor get diskpool
# Expected: 3 pools with state "Online"
```

### Production StorageClass (3-way Replication)

```yaml
# mayastor-replicated-storageclass.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: mayastor-nvme-replicated
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
parameters:
  ioTimeout: "60"
  protocol: nvmf
  repl: "3"  # 3-way replication across nodes
  thin: "true"
  stsAffinityGroup: "true"  # Keep StatefulSet replicas on same nodes
provisioner: io.openebs.csi-mayastor
reclaimPolicy: Delete
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
```

### Verify Replication

```bash
# Create a test PVC
kubectl apply -f - <<EOF
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: test-replicated-pvc
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: mayastor-nvme-replicated
  resources:
    requests:
      storage: 10Gi
EOF

# Check volume has 3 replicas
kubectl -n mayastor get volume
kubectl -n mayastor describe volume <volume-id>
# Should show 3 replicas across different nodes
```

### Capacity Planning

| Usable Capacity | Raw Required | Overhead |
|-----------------|--------------|----------|
| 1 TB usable | 3 TB raw | 3x (repl=3) |
| 5 TB usable | 15 TB raw | 3x (repl=3) |
| 10 TB usable | 30 TB raw | 3x (repl=3) |

**Formula**: `Raw Capacity = Usable Capacity × Replica Count`

### Network Configuration

For best performance with NVMe-oF replication:

```yaml
# Dedicated storage network (optional but recommended)
machine:
  network:
    interfaces:
      # Management/Kubernetes network
      - interface: eth0
        dhcp: true
      # Storage network (dedicated for NVMe-oF)
      - interface: eth1
        addresses:
          - 10.10.10.x/24  # Isolated storage VLAN
        mtu: 9000  # Jumbo frames
```

| Network Speed | Recommendation |
|---------------|----------------|
| 1 Gbps | Adequate for light workloads |
| 10 Gbps | Recommended for production |
| 25 Gbps | Optimal for heavy I/O workloads |

### Monitoring and Alerting

Mayastor exposes Prometheus metrics. Add these alerts:

```yaml
# prometheus-alerts.yaml
groups:
  - name: mayastor
    rules:
      - alert: MayastorPoolDegraded
        expr: mayastor_pool_status != 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Mayastor pool {{ $labels.pool }} is degraded"

      - alert: MayastorVolumeNoReplicas
        expr: mayastor_volume_replica_count < 3
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Volume {{ $labels.volume }} has fewer than 3 replicas"

      - alert: MayastorNodeDown
        expr: up{job="mayastor-io-engine"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Mayastor I/O engine down on {{ $labels.node }}"
```

### Failure Scenarios and Recovery

| Scenario | Impact | Recovery |
|----------|--------|----------|
| 1 node failure | No data loss, degraded performance | Automatic rebuild when node returns |
| 2 node failure | **Data at risk** | Restore from backup |
| Disk failure | No data loss if other replicas healthy | Replace disk, rebuild pool |
| Network partition | Possible split-brain | Requires manual intervention |

### Backup Strategy

Even with 3-way replication, maintain off-cluster backups:

```bash
# Install Velero for backup
velero install \
  --provider aws \
  --plugins velero/velero-plugin-for-aws:v1.5.0 \
  --bucket mayastor-backups \
  --backup-location-config region=us-east-1 \
  --use-restic

# Schedule daily backups
velero schedule create daily-backup \
  --schedule="0 2 * * *" \
  --include-namespaces production
```

---

## Cost Estimation

### Hardware Cost per Node (Approximate)

| Component | Budget | Mid-Range | High-End |
|-----------|--------|-----------|----------|
| Server (used/refurb) | $200-400 | $500-800 | $1000+ |
| NVMe OS (256GB) | $30 | $50 | $80 |
| NVMe Storage (1TB) | $80 | $120 | $200 |
| RAM (16GB) | $50 | $80 | $120 |
| 10GbE NIC | $30 | $80 | $150 |
| **Total per node** | **~$400** | **~$800** | **~$1500** |

### 3-Node Cluster Total

| Tier | Total Cost | Usable Storage |
|------|------------|----------------|
| Budget | ~$1,200 | ~1 TB |
| Mid-Range | ~$2,400 | ~1 TB |
| High-End | ~$4,500+ | ~1 TB+ |

*Note: Usable storage = Raw storage ÷ 3 (for 3-way replication)*

