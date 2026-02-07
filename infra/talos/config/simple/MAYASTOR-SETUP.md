# Mayastor Setup Guide

## Prerequisites

Mayastor requires specific Talos configuration to work properly with hostNetwork pods.

### 1. Apply PodSecurity Exemptions (Control Plane Only)

```bash
talosctl patch machineconfig -n 10.8.0.1 --patch @mayastor-controlplane-patch.yaml
```

### 2. Apply Worker Prerequisites (Worker Nodes Only)

```bash
talosctl patch machineconfig -n 10.8.0.2 --patch @mayastor-worker-patch.yaml
```

This configures:
- Hugepages (2GiB for io-engine DPDK)
- Node label `openebs.io/engine=mayastor`
- NVMe TCP kernel module
- Required kubelet mounts

### 3. Apply Network Firewall Rules (ALL Nodes)

**CRITICAL**: Mayastor components use `hostNetwork: true` and need incoming connections allowed:

```bash
# Control plane
talosctl patch machineconfig -n 10.8.0.1 --patch @mayastor-network-patch.yaml

# Worker(s)
talosctl patch machineconfig -n 10.8.0.2 --patch @mayastor-network-patch.yaml
```

This opens required ports:
- **50051/tcp**: agent-core gRPC API
- **50052/tcp**: agent-core HA cluster
- **10124/tcp**: io-engine NVMe-oF target

Without these firewall rules, io-engine init containers will timeout trying to connect to agent-core service.

## Verification

After applying patches, verify:

```bash
# Check hugepages on worker
talosctl -n 10.8.0.2 read /proc/meminfo | grep HugePages

# Check io-engine pods start
kubectl get pods -n mayastor -l app=io-engine -o wide

# Check agent-core is reachable
kubectl exec -n mayastor <io-engine-pod> -c io-engine -- nc -vz mayastor-agent-core 50051
```

## Troubleshooting

### io-engine stuck in Init:0/2

**Symptom**: Init container times out connecting to `mayastor-agent-core:50051`

**Cause**: Talos firewall blocking port 50051

**Fix**: Apply `mayastor-network-patch.yaml` to all nodes

### No hugepages available

**Symptom**: io-engine pod fails with "Cannot allocate memory"

**Cause**: Hugepages not configured on worker

**Fix**: Apply `mayastor-worker-patch.yaml` and reboot worker node

## Architecture Notes

Mayastor uses `hostNetwork: true` for both agent-core and io-engine for performance:
- agent-core: Runs on worker, listens on host IP (10.8.0.2)
- io-engine: Runs on all nodes with storage, connects to agent-core

This bypasses Cilium's pod networking entirely, so host firewall rules are required.
