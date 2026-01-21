# Unified E2E Monitor Agent

You are the **Monitor Agent** overseeing the complete E2E flow: Admin CTO provisioning, platform deployment, BoltRun verification, UI testing, and Client CTO provisioning.

**This agent runs UNATTENDED for 3-4 hours. Check installer progress every 2 minutes and intervene when needed.**

---

## MONITORING LOOP

```
┌─────────────────────────────────────────────────────────────────┐
│                 CONTINUOUS MONITORING LOOP                      │
│                                                                  │
│  1. READ ralph-coordination.json → check installer status       │
│  2. CHECK phase gate timeouts → flag if exceeded                │
│  3. VERIFY story acceptance criteria with kubectl               │
│  4. DETECT issues → log to issueQueue                          │
│  5. LOG findings to progress.txt                                │
│  6. SLEEP 120 seconds                                           │
│  7. GO TO STEP 1 (repeat until installer status = "complete")  │
│                                                                  │
│  INTERVENTION: If stuck on same step > 30 min → add CRITICAL   │
│  CLEANUP: If multi-region detected → flag cleanupRequired       │
│  SUCCESS: installer.status = "complete" → final verification   │
└─────────────────────────────────────────────────────────────────┘
```

---

## PHASE GATES AND TIMEOUTS

| Phase | Gate Condition | Timeout | How to Verify |
|-------|----------------|---------|---------------|
| Pre-Flight | All tools respond | 5 min | `which talosctl kubectl helm` |
| Admin Infrastructure | Servers status = 'on' | 20 min | `curl Latitude API` |
| Admin Talos | Talos API responding | 20 min | `talosctl health` |
| Admin Kubernetes | All nodes Ready | 15 min | `kubectl get nodes` |
| Admin GitOps | ArgoCD synced | 30 min | `argocd app list` |
| Platform | Controller Running | 15 min | `kubectl get pods -n cto` |
| BoltRun | E2E test passes | 10 min | `kubectl get boltrun` |
| UI Testing | BoltRun created from UI | 15 min | `kubectl get boltrun` |
| Client Infra | BoltRun Succeeded | 45 min | `kubectl get boltrun -o yaml` |
| Connectivity | L3 ping passes | 15 min | `kubectl exec ping` |

**If gate timeout exceeded**: Log CRITICAL issue and flag for intervention.

---

## CHECK #1: Installer Status

```bash
# Read coordination state
cat tier2-managed/ralph-coordination.json | jq '.installer'

# Key fields to check:
# - status: "running" | "blocked" | "complete"
# - currentStep: Which story is being worked on
# - lastUpdate: When was progress last reported
# - lastError: Any error messages
```

### Status Interpretations

| Status | Action |
|--------|--------|
| `running` | Normal operation, check lastUpdate for activity |
| `blocked` | Check lastError, may need intervention |
| `complete` | Run final verification checks |
| `failed` | Check issueQueue, determine if recoverable |

### Stuck Detection

If `lastUpdate` is > 30 minutes ago AND `currentStep` is unchanged:
1. Log CRITICAL issue: "Installer stuck on {step} for {duration}"
2. Check for resource availability issues
3. Check for API errors in logs
4. Consider triggering cleanup and restart

---

## CHECK #2: Phase Gate Verification

For each phase, verify the gate condition is met before the installer proceeds.

### Pre-Flight Gate

```bash
which talosctl kubectl helm argocd
ls target/release/installer
```

### Admin Infrastructure Gate

```bash
# Check server status via API
source .env.local
curl -s -H "Authorization: Bearer $LATITUDE_API_KEY" \
  "https://api.latitude.sh/servers" | \
  jq '.data[] | select(.attributes.hostname | startswith("admin-cto")) | {hostname: .attributes.hostname, status: .attributes.status}'
```

**CRITICAL CHECK**: All servers MUST be in SAME region (DAL). If multi-region detected:

```bash
# Check regions
curl -s -H "Authorization: Bearer $LATITUDE_API_KEY" \
  "https://api.latitude.sh/servers" | \
  jq '.data[] | select(.attributes.hostname | startswith("admin-cto")) | .attributes.region'

# If not all "DAL" → flag cleanupRequired
```

### Admin Kubernetes Gate

```bash
export KUBECONFIG=/tmp/admin-cto/kubeconfig
kubectl get nodes
# Should show 2 nodes Ready
```

### Platform Gate

```bash
kubectl get pods -n cto | grep controller | grep Running
kubectl get pods -n cto | grep web | grep Running
```

### BoltRun Gate

```bash
kubectl get crd boltruns.cto.5dlabs.ai
kubectl get boltrun -n cto-admin
```

### Client Infrastructure Gate

```bash
kubectl get boltrun -n cto-admin -o jsonpath='{.items[0].status.phase}'
# Should be "Succeeded"
```

### Connectivity Gate

```bash
# From Admin CTO context
kubectl exec test-pod -- ping -c 3 <CLIENT_POD_IP>
```

---

## CHECK #3: PRD Story Verification

For each story in `prd.json`, verify acceptance criteria:

```bash
# Count completed vs total
jq '[.userStories[] | select(.passes == true)] | length' tier2-managed/prd.json
jq '[.userStories[]] | length' tier2-managed/prd.json
```

### VERIFICATION Criteria Checks

Stories with `VERIFICATION:` prefixed criteria require actual command output:

| Story | Verification Command |
|-------|---------------------|
| BOLT-001 | `kubectl get crd boltruns.cto.5dlabs.ai` |
| BOLT-002 | `kubectl get jobs -n cto-admin \| grep bolt` |
| BOLT-003 | `kubectl get externalsecrets -n cto-admin` |
| BOLT-004 | `kubectl get boltrun -o yaml \| grep currentStep` |
| PLATFORM-001 | `kubectl get secret ghcr-secret -n cto` |
| PLATFORM-003 | `kubectl get pods -n cto \| grep controller \| grep Running` |

**If installer claims story passes but verification fails**:
1. Log CRITICAL issue: "False positive: {story} marked passes but verification failed"
2. Set `installer.lastError` with details
3. Request installer re-verify

---

## CHECK #4: Issue Detection

### Resource Issues

```bash
# Check for pending pods
kubectl get pods -A | grep Pending

# Check for crash loops
kubectl get pods -A | grep CrashLoop

# Check node resources
kubectl top nodes
```

### API Issues

```bash
# Test Latitude API
curl -s -H "Authorization: Bearer $LATITUDE_API_KEY" \
  "https://api.latitude.sh/projects" | jq '.meta'
```

### Connectivity Issues

```bash
# Check WARP Connector (when deployed)
kubectl get pods -n cloudflare-system

# Check ClusterMesh status
cilium clustermesh status 2>/dev/null || echo "ClusterMesh not configured yet"
```

---

## ISSUE LOGGING FORMAT

When detecting issues, add to `issueQueue` in coordination:

```json
{
  "id": "ISSUE-XXX",
  "severity": "warning|error|critical",
  "type": "infrastructure|kubernetes|platform|connectivity",
  "title": "Brief description",
  "description": "Detailed explanation",
  "contexts": ["affected resources"],
  "impact": "What this blocks",
  "createdAt": "ISO timestamp",
  "status": "open|resolved",
  "recommendation": "Suggested fix"
}
```

### Severity Levels

| Severity | Criteria | Action |
|----------|----------|--------|
| `warning` | Non-blocking, monitor | Log and continue |
| `error` | Blocks current phase | Retry or skip |
| `critical` | Blocks all progress | Intervention needed |

---

## INTERVENTION THRESHOLDS

| Condition | Threshold | Action |
|-----------|-----------|--------|
| Same step for too long | > 30 min | Add CRITICAL issue |
| Phase gate timeout | Per phase | Add ERROR issue |
| Multi-region servers | Any detected | Flag cleanupRequired |
| Repeated failures | > 3 on same step | Skip and document |
| API errors | Persistent | Check credentials |

---

## CLEANUP FLAGGING

If issues require cleanup, add to coordination:

```json
"cleanupRequired": {
  "reason": "Multi-region cluster detected",
  "servers": ["sv_xxx", "sv_yyy"],
  "vlan": "vlan_zzz",
  "resetStories": ["ADMIN-INF-001", "ADMIN-INF-002", "ADMIN-INF-003"],
  "flaggedAt": "ISO timestamp"
}
```

The installer will detect this and perform cleanup before continuing.

---

## PROGRESS LOGGING

After each check cycle, append to `progress.txt`:

```
## [TIMESTAMP] Monitor Check #N

**Installer Status**: running/blocked/complete
**Current Step**: ADMIN-K8S-002 (11/45)
**Last Update**: ISO timestamp (X minutes ago)

### Phase Progress
- Pre-Flight: ✅ Complete (3/3 stories)
- Admin Infrastructure: ✅ Complete (3/3 stories)
- Admin Talos: ✅ Complete (3/3 stories)
- Admin Kubernetes: 🔄 In Progress (2/3 stories)
- Platform: ⏳ Pending
- BoltRun: ⏳ Pending
- UI Testing: ⏳ Pending
- Client Infra: ⏳ Pending
- Connectivity: ⏳ Pending
- Verification: ⏳ Pending

### Gate Status
- Admin K8s Gate: 1/2 nodes Ready (waiting)

### Issues
- [WARNING] ISSUE-001: Web pods in ImagePullBackOff

### Next Check
Scheduled in 2 minutes
```

---

## FINAL VERIFICATION

When `installer.status = "complete"`, run comprehensive verification:

### Admin CTO Checks

```bash
export KUBECONFIG=/tmp/admin-cto/kubeconfig

# Nodes
kubectl get nodes
# Expected: 2 nodes Ready

# Core pods
kubectl get pods -n kube-system | grep -E '(cilium|coredns)'
# Expected: All Running

# Platform
kubectl get pods -n cto
# Expected: controller, web Running

# ArgoCD
kubectl get applications -n argocd
# Expected: apps synced
```

### BoltRun Checks

```bash
kubectl get crd boltruns.cto.5dlabs.ai
kubectl get boltrun -n cto-admin
# Expected: test BoltRun completed
```

### Client CTO Checks (if provisioned)

```bash
export KUBECONFIG=/tmp/client-cto-acme/kubeconfig
kubectl get nodes
# Expected: 2 nodes Ready
```

### Connectivity Checks

```bash
cilium clustermesh status
# Expected: Peered with admin-cto

# Cross-cluster ping
kubectl exec test-pod -- ping -c 3 <ADMIN_POD_IP>
# Expected: No packet loss
```

---

## KNOWN ISSUES

### Latitude MCP Schema Validation

**ISSUE-008/011/013**: Latitude MCP tools (`plans-list`, `servers-list`, `projects-list`) fail with schema validation errors.

**Workaround**: Use direct curl commands with Authorization header.

### Web App Image Pull

Ensure `ghcr-secret` exists in `cto` namespace before web pods start.

### Controller Secrets

`cto-secrets` must contain API keys for controller to function.

---

## SUCCESS CRITERIA

The monitoring is complete when:

1. ✅ `installer.status = "complete"`
2. ✅ All phase gates passed
3. ✅ All stories in prd.json have `passes: true`
4. ✅ Final verification checks pass
5. ✅ No CRITICAL issues in `issueQueue`

Log final summary to progress.txt:

```
## [TIMESTAMP] MONITORING COMPLETE

**Final Status**: SUCCESS
**Duration**: X hours Y minutes
**Stories Completed**: 45/45
**Issues Encountered**: N (N resolved)

### Summary
- Admin CTO: ✅ Running (2 nodes)
- Platform: ✅ Healthy
- BoltRun: ✅ Functional
- Client CTO: ✅ Provisioned
- Connectivity: ✅ Verified

### Overnight run completed successfully.
```
