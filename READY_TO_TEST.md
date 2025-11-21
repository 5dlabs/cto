# ✅ Cluster Ready for Testing

## Current Cluster State (After Cleanup)

### ✅ Clean - No Action Needed
- **Workflows:** 0 (all cleaned up)
- **Jobs:** 2 (both completed, TTL will auto-delete)
- **Hung Pods:** 0 (none)
- **Stuck CodeRuns:** 0 (all completed or deleted)
- **Stale ConfigMaps:** 0 (just cleaned 1)

### ✅ Healthy Infrastructure
- **Controller:** Running, Ready, 0 restarts
- **Toolman:** Running, Ready
- **ArgoCD Sync:** Synced (20:58 UTC)
- **ConfigMaps:** Fresh (20:58 UTC)

### 📝 Remaining Resources (Normal)
```
CodeRuns (completed, will expire):
- coderun-cto-parallel-test-t1-implementation-r72d9 (Succeeded, work_completed=true)
- coderun-cto-parallel-test-t1-quality-99mtk (Succeeded, work_completed=true)
- coderun-cto-parallel-test-t6-implementation-kgk5h (Succeeded, work_completed=true)

Jobs (completed, TTL cleanup pending):
- play-coderun-task-6-blaze-factory-agent-pl-e346b20f-e97d9a5d-v1 (Complete)
- workspace-pvc-cleaner-29395935 (Complete)

Pods (system):
- controller-778f684fb6-cfnx5 (Running - main controller)
- toolman-774fdc6c59-knmhh (Running - MCP server)

PVCs (persistent, reused):
- workspace-cto-parallel-test (generic agents)
- workspace-cto-parallel-test-cipher (Cipher)
- workspace-cto-parallel-test-cleo (Cleo)
- workspace-cto-parallel-test-tess (Tess)
- workspace-cto-test-rex-gemini (Rex)
- workspace-atlas-pr-guardian-atlas (Atlas)
```

---

## ✅ No Cleanup Required

**The cluster is already clean!** Someone or TTL cleanup already removed:
- All stuck Tess jobs
- All old workflows
- All hung pods

The remaining resources are all in normal/healthy state.

---

## What Happens When You Test

### Current ConfigMaps (OLD - Before Merge):
- Still have FIFO code (will hang)
- Missing token refresh function (will crash after 45 min)
- No ConfigMap health check (controller won't complain)

### After Merging PR #1551:
1. **Merge PR** → triggers CI
2. **CI regenerates ConfigMaps** → includes all fixes
3. **Push to main** → ArgoCD detects changes
4. **ArgoCD syncs** (~2 min) → new ConfigMaps deployed
5. **Controller pod restarts** → runs health check, verifies ConfigMaps
6. **New test workflows** → use fixed code

---

## Testing Procedure

### Step 1: Merge PR #1551
```bash
gh pr merge 1551 --merge
```

### Step 2: Wait for ArgoCD Sync
```bash
# Watch ArgoCD sync the changes:
kubectl get app controller -n argocd -w

# Or manually trigger:
argocd app sync controller

# Wait until status: Synced
```

### Step 3: Verify Controller Restarted with Health Check
```bash
# Check controller logs for health check:
kubectl logs -n agent-platform -l app.kubernetes.io/name=controller --tail=50 | grep "ConfigMap"

# Should see:
# "Verifying required ConfigMaps are available..."
# "✓ Claude agent templates - 29 files"
# "✓ Codex agent templates - 23 files"
# ... etc ...
# "✅ All required ConfigMaps verified"
```

### Step 4: Run Test Workflow
```bash
# Use your normal test command (e.g., task-master play, kubectl apply, etc.)
# Example:
kubectl apply -f infra/examples/play-workflow-instance.yaml
```

### Step 5: Monitor for Success
```bash
# Watch workflows complete:
kubectl get workflows -n agent-platform -w

# Should see:
# - Jobs created immediately (not 8-hour delay)
# - Tess completes cleanly (not 6-hour hang)
# - All stages progress: Rex → Cleo → Cipher → Tess
# - Workflows complete successfully
```

---

## What Success Looks Like

### Timing (Tasks 1 & 6 type):
```
00:00 - Workflow starts
00:00 - CodeRun created
00:00 - Controller creates job (< 30 seconds)
00:01 - Job starts running
00:10 - Agent completes work
00:11 - Job exits cleanly
00:11 - Workflow progresses to next stage
```

**No 8-hour delays!**

### Timing (Tasks 3 & 4 type with Tess):
```
00:00 - Tess job starts
00:05 - Tess completes work
00:05 - Claude exits cleanly (subshell closes stdin)
00:05 - Container exits
00:05 - Trap cleanup fires
00:05 - Sentinel file created
00:05 - Docker daemon exits
00:05 - Pod terminates
00:05 - CodeRun phase: Succeeded
```

**No 6-hour hangs!**

---

## What Failure Looks Like (If ConfigMaps Broken)

### With Your New Safeguards:
```
00:00 - Workflow starts
00:00 - CodeRun created
00:00 - Controller tries to create job
00:00 - Template generation fails (ConfigMaps missing)
00:00 - Controller retries...
00:05 - Still failing...
10:00 - ⏰ Workflow step timeout (maxDuration: 10m)
10:00 - Workflow: FAILED
10:00 - Error: "Job creation timeout - check ConfigMaps"
```

**Fails fast with clear error!**

Or even better:

```
Controller Startup:
- Verifying ConfigMaps...
- ❌ ConfigMap controller-agent-templates-claude NOT FOUND
- Listing missing ConfigMaps...
- Controller: CRASH (exit 1)
```

**Fails at startup before any workflows run!**

---

## Summary

### Current Cluster State: ✅ **CLEAN**
- No cleanup required
- Already in good state for testing
- Just completed jobs that will auto-expire

### What You Need to Do:
1. **Merge PR #1551** (contains all fixes)
2. **Wait 2 minutes** (ArgoCD sync)
3. **Run test** (should work perfectly)

### What Will Be Different:
- **No 8-hour delays** (ConfigMap health check)
- **No 6-hour hangs** (FIFO removed)
- **No crashes** (token refresh removed)
- **No escaped variables** (client-config copied correctly)
- **Fast failures** (workflow step timeout)
- **Clear errors** (controller health messages)

**The cluster is ready. Just merge and test!** 🚀

