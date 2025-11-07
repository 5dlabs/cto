# Cluster Resource Optimization Summary

## Problem

2-node cluster hitting pod scheduling limits:
- **Control-plane node**: Mac Mini at 192.168.1.77 (tainted, cannot run regular workloads)
- **Worker node**: Dell at 192.168.1.72 (hitting pod capacity limit)

Error: `0/2 nodes are available: 1 Too many pods, 1 node(s) had untolerated taint {node-role.kubernetes.io/control-plane: }`

## Optimizations Implemented

### Phase 1: Removed Unused Databases ✅ (Saves ~6-8 pods)

Deleted:
- `trader-postgres.yaml` (2 instances + 2 poolers = 4 pods)
- `trader-questdb.yaml` (1 pod)
- `test-db/postgres.yaml` (1 pod)
- `test-db.yaml` ArgoCD application

These databases were not used by the core CTO platform.

### Phase 2: Reduced HA Replicas for Dev Environment ✅ (Saves ~5 pods)

**vector-postgres**:
- Before: 2 instances + 2 poolers = 4 pods
- After: 1 instance + 1 pooler = 2 pods
- **Savings: 2 pods**

**redis**:
- Before: 3 redis + 3 sentinels = 6 pods
- After: 1 redis + 1 sentinel = 2 pods
- **Savings: 4 pods**

**Total Phase 1+2 Savings: ~11-13 pods**

## Additional Optional Optimizations (Phase 3)

If you still need more pod capacity, consider temporarily disabling these optional services:

### Monitoring & Observability (Can disable for dev)

**Lowest priority (disable first):**
```bash
# Disable monitoring stack (Grafana, VictoriaMetrics, etc.)
kubectl delete application -n argocd grafana
kubectl delete application -n argocd victoria-metrics
kubectl delete application -n argocd victoria-logs
kubectl delete application -n argocd otel-collector
kubectl delete application -n argocd monitoring-stack
```
**Pod savings: ~5-8 pods**

**Medium priority (disable if needed):**
```bash
# Disable logging collection
kubectl delete application -n argocd fluent-bit
```
**Pod savings: ~1-2 pods (DaemonSet)**

### Network Services (If not actively using)

**Twingate connectors** (if VPN not needed right now):
```bash
kubectl delete application -n argocd twingate-pastoral
kubectl delete application -n argocd twingate-therapeutic
```
**Pod savings: ~2 pods**

### Kubernetes Runners (If not running CI/CD jobs)

```bash
# Disable platform runners if not actively using
kubectl delete application -n argocd platform-runners
```
**Pod savings: Depends on runner count**

## Services That Should NOT Be Disabled

Keep these for core platform functionality:
- `controller` - Core CTO platform controller
- `argo-workflows` - Workflow orchestration (required)
- `argo-events` - Event processing (required)
- `external-secrets` - Secrets management
- `postgres-operator`, `redis-operator` - Database operators
- `doc-server` - Documentation service
- `toolman` - Tool management
- `k8s-mcp` - Kubernetes MCP server
- `ngrok-operator`, `ngrok-gateway` - External access
- `external-dns` - DNS management
- `gateway-api` - Gateway resources

## Alternative Solutions

### Increase Worker Node Pod Limit

If you need all services, consider increasing the pod limit on your worker node:

**Edit Talos worker config** (`infra/talos/config/simple/worker.yaml`):
```yaml
machine:
  kubelet:
    extraArgs:
      max-pods: "200"  # Default is 110
```

Then apply:
```bash
talosctl apply-config --nodes 192.168.1.72 --file infra/talos/config/simple/worker.yaml
```

### Add Another Worker Node

The most scalable solution is adding another worker node to the cluster. This would:
- Double available pod capacity
- Enable true HA for databases
- Allow running monitoring stack without resource concerns

### Use Control-Plane Node for Workloads (Not Recommended)

As a last resort, you could remove the control-plane taint to allow scheduling workloads there, but this is **not recommended** for stability:

```bash
kubectl taint nodes <control-plane-node> node-role.kubernetes.io/control-plane:NoSchedule-
```

## Deployment Instructions

After making these changes, commit and push to trigger ArgoCD sync:

```bash
git add infra/gitops/databases/
git commit -m "fix: optimize database replicas for dev cluster resource constraints"
git push origin fix/workflow-success-condition-syntax
```

ArgoCD will automatically:
1. Remove deleted database applications
2. Scale down vector-postgres and redis replicas
3. Free up pod capacity within 5-10 minutes

## Monitoring Changes

Check pod count after sync:
```bash
# Watch pods being terminated
kubectl get pods -A --watch

# Count total pods on worker node
kubectl get pods -A -o wide | grep 192.168.1.72 | wc -l

# Check for pending pods
kubectl get pods -A | grep Pending
```

## Expected Results

After these optimizations, your worker node should have:
- **11-13 fewer pods** from database reduction
- **5-10 additional pods** available capacity (if disabling monitoring)
- **Sufficient headroom** for CTO platform workflows

The scheduling errors should resolve within 5-10 minutes of ArgoCD syncing the changes.

