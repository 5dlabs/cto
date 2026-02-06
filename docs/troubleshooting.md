## CTO Namespace Pods Not Running (OpenMemory, Tools)

OpenMemory and the tools server are both enabled in the CTO Helm chart and in the Argo Application `applications/workloads/cto.yaml`. There was **no Argo or chart change that disables them**; if they are down, the cause is usually runtime/cluster.

### 1. Check Argo and pod status

```bash
# Argo app sync and health
argocd app get cto -n argocd --refresh

# Pods in cto namespace
kubectl get pods -n cto -o wide

# OpenMemory and tools specifically
kubectl get pods -n cto -l app.kubernetes.io/name=openmemory
kubectl get pods -n cto -l app.kubernetes.io/component=tools
```

### 2. Common causes

| Cause | What to check |
|-------|----------------|
| **Image pull** | `kubectl describe pod -n cto -l app.kubernetes.io/name=openmemory` and look for `Failed`/`ErrImagePull`. Ensure `ghcr.io/5dlabs/openmemory:latest` and `ghcr.io/5dlabs/tools:latest` exist and `ghcr-secret` exists in `cto` namespace. |
| **OOM / crash** | `kubectl logs -n cto -l app.kubernetes.io/name=openmemory --tail=100` and same for tools. Tools has 4Gi limit; OpenMemory has 512Mi—increase if OOMKilled. |
| **PVC / storage** | `kubectl get pvc -n cto` — OpenMemory and tools use `mayastor` storageClass. If PVCs are `Pending`, check Mayastor and capacity. |
| **Sync / prune** | Argo may have pruned resources if app was OutOfSync. Run `argocd app sync cto -n argocd` and re-check pods. |

### 3. Quick diagnostic script

```bash
./scripts/diagnose-cto-pods.sh
```

---

## Linear Projects Being Archived Unexpectedly

If you notice Linear projects being archived immediately after creation, check:

### 1. Linear Workspace Automation Rules
Go to: **Linear** → **Settings** → **Workspace** → **Automations**

Look for any rules that:
- Archive projects after a time period
- Archive projects with certain names
- Cleanup duplicate projects

### 2. Production PM Server
If the local dev environment wasn't running when you ran the intake tool, the request may have gone to the production PM server (`pm.5dlabs.ai`) which might have different behavior.

**Fix:** Always run `just preflight` before using MCP tools to ensure:
- Local services are running
- Tunnel is healthy
- `CTO_PM_SERVER_URL` points to dev

### 3. Cleanup Script
To remove old archived test projects:
```bash
just cleanup-test-projects
```

### Verification
Run `just preflight` and ensure:
- ✅ CTO_PM_SERVER_URL points to pm-dev.5dlabs.ai
- ✅ cto-config.json pmServerUrl points to pm-dev.5dlabs.ai
- ✅ Tunnel is healthy
