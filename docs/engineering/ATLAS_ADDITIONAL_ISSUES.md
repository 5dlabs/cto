# Atlas Additional Issues Discovered During Deployment

**Date:** November 12, 2025  
**Context:** While deploying the deduplication fix for PR #1366  

---

## Critical Issues Discovered

### 1. ❌ Sensor Configuration Update Did Not Take Effect

**Problem:**
- Applied new sensor configuration with deduplication workflow
- Deleted sensor pod to reload configuration
- New pod stuck in `Pending` state due to cluster capacity
- **Old broken sensor configuration still active in the cluster**

**Evidence:**
```bash
$ kubectl describe sensor atlas-pr-guardian -n argo | grep operation
        operation: create  # OLD CONFIG (broken, no deduplication)
        
# Should show:
#   argoWorkflow:
#     operation: submit  # NEW CONFIG (with deduplication)
```

**Impact:**
- Deduplication fix NOT active
- Old sensor continuing to create duplicate CodeRuns
- PR #1366 (the fix itself!) not being monitored by Atlas
- More duplicates being created as we speak

---

### 2. ❌ Cluster Resource Exhaustion Prevents Sensor Restart

**Problem:**
```
Events:
  Warning  FailedScheduling  0/2 nodes are available: 
    1 Too many pods
    1 node(s) had untolerated taint {node-role.kubernetes.io/control-plane: }
```

**Root Cause:**
- Main worker node (`talos-a43-ee1`) at pod capacity
- Sensor pod cannot schedule to apply new configuration
- This creates a catch-22: Need new sensor to stop duplicates, but can't schedule sensor due to duplicates

**Current State:**
- Deleted all Pending Atlas pods (freed resources)
- Sensor pod still cannot schedule
- Need to identify and remove other non-essential pods

---

### 3. ❌ Atlas Not Monitoring PR #1366

**Problem:**
- PR #1366 has merge conflicts (`mergeable: false`, `mergeableState: dirty`)
- PR #1366 has 0 comments (no Bugbot activity yet, but would come)
- Atlas's role is to resolve these automatically
- **No Atlas CodeRun created for PR #1366**

**Why:**
```bash
$ kubectl get coderuns -n agent-platform -l pr-number=1366
No resources found
```

**Root Cause:**
- Sensor is broken/not running with new config
- PR #1366 webhooks not being processed
- Atlas cannot function until sensor is fixed

---

### 4. ⚠️ Docker Sidecar Shutdown Issue

**User Report:** "Docker sidecar shutdown is not working yet either"

**Investigation Needed:**
- Container scripts have `stop_docker_sidecar()` function
- Function uses sentinel file + pkill/kill
- EXIT trap configured: `trap 'stop_docker_sidecar || true' EXIT`

**Possible Issues:**
1. Sentinel file not being created correctly
2. Docker daemon not responding to signals
3. Grace period too short
4. Pods terminated before sidecar can shut down

**Files to Check:**
- All container scripts under `infra/charts/controller/agent-templates/code/*/container*.sh.hbs`
- Look for pods with containers stuck in terminating

---

## Immediate Actions Required

### Priority 1: Get Sensor Running with New Config

**Options:**

**A. Scale down other deployments temporarily**
```bash
# Identify largest deployments
kubectl get deployments --all-namespaces --sort-by='.spec.replicas'

# Temporarily scale down non-critical services
kubectl scale deployment <name> --replicas=0 -n <namespace>

# Wait for sensor to schedule
kubectl wait --for=condition=Ready pod -n argo -l sensor-name=atlas-pr-guardian

# Scale services back up
kubectl scale deployment <name> --replicas=<original> -n <namespace>
```

**B. Add node capacity (if cluster supports)**
```bash
# Add worker node or increase pod limits
```

**C. Force schedule sensor with priority**
- Modify sensor deployment to use high priority class
- Preempt lower-priority pods

### Priority 2: Manually Trigger Atlas for PR #1366

While sensor is broken, manually create Atlas CodeRun:

```bash
cat <<EOF | kubectl create -f -
apiVersion: agents.platform.5dlabs.ai/v1alpha1
kind: CodeRun
metadata:
  name: coderun-atlas-pr-1366-manual
  namespace: agent-platform
  labels:
    agent: atlas
    role: pr-guardian
    pr-number: "1366"
    repository: "cto"
spec:
  taskId: 0
  service: "atlas-pr-guardian"
  githubApp: "5DLabs-Atlas"
  repositoryUrl: "https://github.com/5dlabs/cto.git"
  docsRepositoryUrl: "https://github.com/5dlabs/cto.git"
  docsProjectDirectory: "docs"
  workingDirectory: "."
  continueSession: true
  overwriteMemory: false
  cliConfig:
    cliType: "claude"
    model: "claude-sonnet-4-20250514"
    maxTokens: 8192
    temperature: 0.3
  env:
    PR_NUMBER: "1366"
    PR_URL: "https://github.com/5dlabs/cto/pull/1366"
    REPOSITORY_FULL_NAME: "5dlabs/cto"
    GUARDIAN_MODE: "active"
    TARGET_REPOSITORY: "5dlabs/cto"
    MERGE_STRATEGY: "squash"
EOF
```

### Priority 3: Investigate Docker Sidecar Issue

```bash
# Find pods with terminating containers
kubectl get pods -n agent-platform -o json \
  | jq -r '.items[] | select(.status.containerStatuses != null) | .metadata.name'

# Check specific pod logs for sidecar shutdown
kubectl logs <pod-name> -n agent-platform -c docker-daemon

# Look for shutdown messages or errors
```

### Priority 4: Update PR #1366

Add to PR description:
- Sensor deployment issue (needs cluster capacity)
- Manual Atlas triggering instructions
- Docker sidecar investigation status

---

## Long-Term Fixes

### 1. Resource Quotas and Limits
- Set resource requests/limits on all pods
- Implement pod priority classes
- Auto-scaling for worker nodes

### 2. Admission Webhook for Deduplication
- Controller-level deduplication (more reliable than sensor)
- Reject duplicate CodeRun creation at API level
- Doesn't rely on sensor being healthy

### 3. Graceful Degradation
- If sensor unhealthy, queue events for later processing
- Manual trigger mechanism for critical PRs
- Alerts when Atlas not functioning

### 4. Docker Sidecar Improvements
- Pre-stop hooks for graceful shutdown
- Longer termination grace period
- Better signal handling

---

## Current Workaround

Until sensor is fixed:

1. **Manually create Atlas CodeRuns** for important PRs
2. **Monitor cluster capacity** and scale down non-essential services
3. **Delete Pending pods** regularly to free resources
4. **Use cleanup script** when duplicates appear

---

## Testing Plan (Once Sensor Fixed)

1. Verify sensor pod Running with new config
2. Check sensor logs show `argoWorkflow` trigger mode
3. Create test PR and verify only 1 CodeRun created
4. Update test PR and verify no duplicate created
5. Test Atlas on PR #1366 (merge conflicts + potential Bugbot comments)

---

## References

- PR #1366: Atlas deduplication fix
- Sensor config: `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`
- Container scripts: `infra/charts/controller/agent-templates/code/*/container*.sh.hbs`
