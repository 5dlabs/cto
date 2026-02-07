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

### hostNetwork Pods Cannot Reach ClusterIP Services

**Symptoms:**
- Pods using `hostNetwork: true` (like Mayastor io-engine, metrics-server, etc.) cannot connect to ClusterIP services
- Connection attempts timeout or fail with "Connection refused"
- Cilium service mappings exist but eBPF socket load balancing doesn't work for hostNetwork pods

**Root Cause:**
When `socketLB.hostNamespaceOnly: true` is set in Cilium values, eBPF socket load balancing only applies to the root host network namespace. Pods running with `hostNetwork: true` run in their own network namespaces and cannot benefit from socket load balancing to reach services.

**Fix:**
Set `socketLB.hostNamespaceOnly: false` in `values.yaml` (infra/gitops/manifests/cilium/values.yaml:116):

```yaml
# Enable socket load balancing for better performance
# hostNamespaceOnly must be false to allow hostNetwork pods (like Mayastor io-engine)
# to reach ClusterIP services via eBPF socket load balancing
socketLB:
  enabled: true
  hostNamespaceOnly: false
```

**Verification:**
```bash
# Check ConfigMap value after sync
kubectl get configmap cilium-config -n kube-system -o jsonpath='{.data.bpf-lb-sock-hostns-only}'
# Should output: false

# Test connectivity from hostNetwork pod
kubectl exec -n <namespace> <hostnetwork-pod> -- curl http://<service-name>.<namespace>.svc.cluster.local
```

**Important:** This configuration is critical for:
- Mayastor storage (io-engine and mayastor-agent-ha-node DaemonSets use hostNetwork)
- metrics-server (uses hostNetwork to bypass Cilium pod-to-host hairpin issue)
- Any other infrastructure components that require hostNetwork and need to reach services

## References

- [Cilium Documentation](https://docs.cilium.io/)
- [ClusterMesh Setup Guide](https://docs.cilium.io/en/stable/network/clustermesh/clustermesh/)
- [Cilium on Talos](https://www.talos.dev/v1.9/kubernetes-guides/network/deploying-cilium/)
