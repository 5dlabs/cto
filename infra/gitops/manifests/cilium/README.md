# Cilium CNI Configuration

This directory contains the Helm values for deploying Cilium as the CNI
with ClusterMesh support for multi-cluster networking.

## Overview

Cilium provides:
- **eBPF-based networking** - High-performance packet processing
- **Kube-proxy replacement** - eBPF-based service load balancing
- **WireGuard encryption** - Transparent pod-to-pod encryption
- **Hubble** - Network observability and flow visibility
- **ClusterMesh** - Multi-cluster service discovery and load balancing

## Configuration

### Per-Cluster Values

Each cluster in ClusterMesh requires unique values:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `cluster.name` | Unique cluster name | `cto-dal`, `cto-lax` |
| `cluster.id` | Unique ID (1-255) | `1`, `2`, `3` |

### Pod CIDR Allocation

Each cluster must have non-overlapping Pod CIDRs:

| Cluster | Region | cluster.id | Pod CIDR |
|---------|--------|------------|----------|
| cto-dal | DAL | 1 | 10.1.0.0/16 |
| cto-lax | LAX | 2 | 10.2.0.0/16 |
| cto-nyc | NYC | 3 | 10.3.0.0/16 |

## ClusterMesh Setup

After Cilium is installed on all clusters:

1. **Enable ClusterMesh** on each cluster:
   ```bash
   cilium clustermesh enable --context cto-dal
   cilium clustermesh enable --context cto-lax
   cilium clustermesh enable --context cto-nyc
   ```

2. **Connect clusters**:
   ```bash
   cilium clustermesh connect --context cto-dal --destination-context cto-lax
   cilium clustermesh connect --context cto-dal --destination-context cto-nyc
   cilium clustermesh connect --context cto-lax --destination-context cto-nyc
   ```

3. **Verify connectivity**:
   ```bash
   cilium clustermesh status --context cto-dal --wait
   ```

## Global Services

To expose a service across all clusters, add the annotation:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-service
  annotations:
    service.cilium.io/global: "true"
spec:
  # ... service spec
```

## Validation

### Check Cilium Health

```bash
cilium status --context <cluster>
```

### Check ClusterMesh Status

```bash
cilium clustermesh status --context <cluster>
```

### Test Cross-Cluster Connectivity

```bash
# Deploy test pod
kubectl run test --image=nicolaka/netshoot -- sleep infinity

# Ping pod in another cluster
kubectl exec test -- ping <remote-pod-ip>
```

## Troubleshooting

### Cilium Agent Not Ready

```bash
kubectl -n kube-system logs -l k8s-app=cilium --tail=100
```

### ClusterMesh Connection Issues

```bash
# Check clustermesh-apiserver
kubectl -n kube-system logs -l k8s-app=clustermesh-apiserver

# Verify firewall allows UDP/TCP 2379 for etcd
# Verify firewall allows UDP 4240 for VXLAN or WireGuard
```

### WireGuard Not Working

```bash
# Check encryption status
cilium encrypt status
```

## References

- [Cilium Documentation](https://docs.cilium.io/)
- [ClusterMesh Setup Guide](https://docs.cilium.io/en/stable/network/clustermesh/clustermesh/)
- [Cilium on Talos](https://www.talos.dev/v1.9/kubernetes-guides/network/deploying-cilium/)
