# OpenMemory Deployment Validation Checklist

**Status**: Awaiting deployment after PR #1616 merges  
**PR**: https://github.com/5dlabs/cto/pull/1616

---

## Current Status

✅ **Merged to Main**:
- Complete Helm chart (`infra/charts/openmemory/`)
- Docker configuration (`infra/images/openmemory/`)
- Agent integrations (Rex, Cleo containers)
- Memory functions library (`agent-templates/shared/memory-functions.sh`)
- Documentation (`docs/openmemory-integration-guide.md`)

🔧 **Pending PR #1616**:
- Move OpenMemory app to correct ArgoCD directory
- Fix: `apps/` → `applications/` so app-of-apps can discover it

---

## Deployment Steps

### Step 1: Build Docker Image

```bash
cd infra/images/openmemory
PUSH=true ./build.sh v1.0.0
```

**Requirements**:
- Docker daemon running
- Authenticated to ghcr.io: `echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin`

**Expected Output**:
```
🐳 Building OpenMemory image...
✅ Build complete!
📤 Pushing to registry...
✅ Push complete!
```

### Step 2: Merge PR #1616

Once merged, ArgoCD will automatically:
1. Discover the OpenMemory application
2. Create namespace `cto-system` if needed
3. Deploy OpenMemory resources

### Step 3: Verify ArgoCD Discovery

```bash
# Check ArgoCD picked up the application
kubectl get applications -n argocd | grep openmemory

# Expected output:
# openmemory   Synced   Healthy   https://github.com/5dlabs/cto   main   cto-system
```

### Step 4: Verify Deployment

```bash
# Check pod status
kubectl get pods -n cto-system | grep openmemory

# Expected output:
# openmemory-xxxxx   1/1   Running   0   30s

# Check PVC
kubectl get pvc -n cto-system | grep openmemory

# Expected output:
# openmemory-data   Bound   pvc-xxxxx   20Gi   RWO   longhorn   1m
```

### Step 5: Verify Service

```bash
# Check service
kubectl get svc -n cto-system openmemory

# Expected output:
# NAME         TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)    AGE
# openmemory   ClusterIP   10.43.xxx.xxx  <none>        3000/TCP   1m
```

### Step 6: Test Health Endpoint

```bash
# From within cluster
kubectl exec -n cto-system deploy/openmemory -- curl localhost:3000/health

# Expected output:
# {"status":"ok","database":"connected"}

# Or port-forward locally
kubectl port-forward -n cto-system svc/openmemory 3000:3000 &
curl localhost:3000/health
```

### Step 7: Test Memory Operations

```bash
# Add a test memory
curl -X POST http://localhost:3000/memory/add \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Test memory for validation",
    "metadata": {"agent": "test", "purpose": "validation"}
  }'

# Expected output:
# {"id":"mem_xxxxx","sectors":[...],"waypoints_created":0}

# Query the memory
curl -X POST http://localhost:3000/memory/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "validation test",
    "k": 5
  }'

# Expected output:
# {"matches":[{"id":"mem_xxxxx","content":"Test memory...","score":0.95}]}
```

### Step 8: Verify Agent Integration

```bash
# Check that memory functions are available
kubectl exec -n cto-system deploy/controller -- ls -la /agent-templates/shared/

# Expected to see:
# memory-functions.sh

# Test from an agent container (when next task runs)
# The logs should show:
# 🧠 Initializing OpenMemory integration...
# ✅ OpenMemory connected - loading project context...
```

---

## Validation Checklist

- [ ] Docker image built and pushed to ghcr.io/5dlabs/openmemory:v1.0.0
- [ ] PR #1616 merged to main
- [ ] ArgoCD application appears in `kubectl get applications -n argocd`
- [ ] OpenMemory pod running in cto-system namespace
- [ ] PVC created and bound (20Gi)
- [ ] Service accessible within cluster
- [ ] Health endpoint returns 200 OK
- [ ] Can add memories via API
- [ ] Can query memories via API
- [ ] Memory functions available in agent containers
- [ ] Rex logs show OpenMemory initialization

---

## Expected Timeline

1. **PR #1616 merge**: ~5-10 minutes (review + merge)
2. **ArgoCD sync**: ~1-2 minutes (auto-sync enabled)
3. **Pod startup**: ~2-3 minutes (image pull + container start)
4. **First agent test**: Next task execution

**Total**: ~15 minutes from merge to operational

---

## Troubleshooting

### ArgoCD Not Discovering Application

```bash
# Force app-of-apps to resync
kubectl delete application platform-apps -n argocd
# It will recreate automatically

# Or manually sync
argocd app sync platform-apps
```

### Pod Not Starting

```bash
# Check events
kubectl describe pod -n cto-system -l app.kubernetes.io/name=openmemory

# Check logs
kubectl logs -n cto-system -l app.kubernetes.io/name=openmemory

# Common issues:
# - Image pull failed: Build and push image
# - PVC binding failed: Check Longhorn storage class
# - Permission denied: Check security context settings
```

### Memory Functions Not Available

```bash
# Check ConfigMap was updated
kubectl get cm -n cto-system controller-agent-templates-shared -o yaml

# Should include memory-functions.sh

# If not, sync controller app
argocd app sync controller
```

---

## Success Criteria

OpenMemory is fully operational when:

1. ✅ Pod shows `Running` status
2. ✅ Health endpoint returns `200 OK`
3. ✅ Can add and query memories successfully
4. ✅ Agent logs show memory initialization
5. ✅ First pattern stored successfully during task execution

---

## Next: 2-Week Pilot

Once validation is complete:
1. Run 10-15 tasks through the system
2. Monitor memory query logs
3. Track iteration reduction metrics
4. Measure time-to-completion improvements
5. Analyze cross-agent learning patterns

**Baseline Comparison**: Compare task execution before/after OpenMemory to quantify impact
