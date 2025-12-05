# Atlas Pod Proliferation - Root Cause & Fix

**Date:** November 12, 2025  
**Issue:** 100+ duplicate Atlas pods/CodeRuns created  
**Impact:** Resource exhaustion, pods stuck in Pending state  
**Status:** ✅ Fix Ready for Deployment

---

## Root Cause Analysis

### The Problem

Atlas PR Guardian sensor was creating **multiple CodeRuns per PR**, leading to:
- **100+ pods** in cto namespace
- **73 CodeRuns** for ~15 PRs (5-8 duplicates per PR)
- **Many pods stuck in Pending** (resource exhaustion)
- **Rapid-fire creation** (3+ CodeRuns per second during event storms)

### Why This Happened

1. **No Deduplication Logic**
   - Sensor uses `operation: create` directly on CodeRun resource
   - Every GitHub webhook event triggers a new CodeRun
   - No check for existing CodeRuns for the same PR

2. **Event Storm from GitHub**
   - Single PR action triggers multiple webhook events:
     - `pull_request` (opened, synchronize, ready_for_review)
     - `issue_comment` (created)
     - `pull_request_review` (submitted)
   - Each event → new CodeRun

3. **Design Flaw**
   - Documentation says "controller should implement deduplication"
   - But controller has no such logic
   - Sensor creates CodeRuns blindly

### Evidence from Logs

```
23:38:27 - Created object (triggered by pr-events)
23:38:27 - Created object (triggered by pr-events)  
23:38:27 - Created object (triggered by pr-events)  [3 within same second!]
```

**PR #1029 alone had 8 CodeRuns:**
- `coderun-atlas-pr-2k47b`
- `coderun-atlas-pr-2r8fb`
- `coderun-atlas-pr-4kh7c`
- `coderun-atlas-pr-5vwbn`
- ... (4 more)

---

## The Fix

### 1. Sensor Deduplication Workflow

**Changed:** Direct CodeRun creation → Deduplication workflow

**Before (Broken):**
```yaml
triggers:
  - template:
      k8s:
        operation: create  # Always creates, no checks!
        source:
          resource:
            apiVersion: agents.platform.5dlabs.ai/v1alpha1
            kind: CodeRun
            # ...
```

**After (Fixed):**
```yaml
triggers:
  - template:
      argoWorkflow:
        operation: submit  # Run deduplication workflow first
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            spec:
              templates:
                - name: deduplicate-and-create
                  script:
                    source: |
                      # Check for existing CodeRuns
                      EXISTING=$(kubectl get coderuns -n cto \
                        -l agent=atlas,pr-number="$PR_NUMBER" ...)
                      
                      # If Running: Skip creation
                      # If Completed/Failed: Create new
                      # If none: Create new
```

### 2. Deduplication Logic

```bash
# Check for existing CodeRuns for this PR
EXISTING=$(kubectl get coderuns -n cto \
  -l agent=atlas,pr-number="$PR_NUMBER" -o json)

COUNT=$(echo "$EXISTING" | jq '.items | length')

if [ "$COUNT" -gt 0 ]; then
  # Check if any are Running
  RUNNING=$(echo "$EXISTING" | jq -r '.items[] | select(.status.phase == "Running") | .metadata.name')
  
  if [ -n "$RUNNING" ]; then
    echo "✅ CodeRun already running: $RUNNING"
    echo "   Session continuity will handle this event"
    exit 0  # Skip creation!
  fi
fi

# Create new CodeRun only if none Running
```

### 3. Benefits

- **One Atlas per PR**: No more duplicates
- **Session Continuity Works**: Running CodeRun handles all events for that PR
- **Resource Efficiency**: No more pod storms
- **Idempotent**: Safe to trigger multiple times

---

## Remediation Steps

### Step 1: Clean Up Existing Duplicates

```bash
# Run cleanup script (interactive, requires confirmation)
./scripts/cleanup-duplicate-atlas-coderuns.sh
```

**What it does:**
- Groups CodeRuns by PR number
- Keeps oldest CodeRun per PR
- Deletes all duplicates
- Shows summary before confirmation

**Expected Result:**
- Before: 73 Atlas CodeRuns
- After: ~15 Atlas CodeRuns (one per PR)

### Step 2: Apply Fixed Sensor

```bash
# Backup current sensor
kubectl get sensor atlas-pr-guardian -n argo -o yaml > atlas-pr-guardian-backup.yaml

# Apply fixed sensor
kubectl apply -f infra/gitops/resources/sensors/atlas-pr-guardian-sensor-fixed.yaml

# Restart sensor pod to reload configuration
kubectl delete pod -n argo -l sensor-name=atlas-pr-guardian

# Wait for new pod to start
kubectl wait --for=condition=Ready pod -n argo -l sensor-name=atlas-pr-guardian --timeout=60s
```

### Step 3: Verify Fix

```bash
# Monitor Atlas CodeRun creation
watch kubectl get coderuns -n cto -l agent=atlas

# Trigger a test PR event (e.g., open a test PR)
# Observe that only ONE CodeRun is created

# Check sensor logs for deduplication messages
kubectl logs -n argo -l sensor-name=atlas-pr-guardian --tail=50 -f
```

**Expected Log Output:**
```
=== Atlas PR Guardian Deduplication ===
PR Number: 1234
Found 1 existing CodeRun(s) for PR #1234
✅ CodeRun already running: coderun-atlas-pr-abc123
   Session continuity will handle this event
```

### Step 4: Monitor Resource Usage

```bash
# Check pod count stabilizes
kubectl get pods -n cto -l agent=atlas | wc -l
# Should be ~15-20 (one per open PR)

# Check no pods stuck in Pending
kubectl get pods -n cto -l agent=atlas --field-selector=status.phase=Pending
# Should be empty or very few

# Monitor for 10 minutes
watch kubectl get coderuns -n cto -l agent=atlas
```

---

## Testing Plan

### Test Case 1: New PR
1. Open a new PR in 5dlabs/cto
2. Verify exactly 1 Atlas CodeRun created
3. Check sensor logs show deduplication check

**Expected:**
- ✅ One CodeRun created
- ✅ Sensor logs: "Found 0 existing CodeRun(s)"

### Test Case 2: PR Update
1. Push new commit to existing PR
2. Verify NO new CodeRun created
3. Check sensor logs show "CodeRun already running"

**Expected:**
- ✅ No new CodeRun
- ✅ Sensor logs: "CodeRun already running: coderun-atlas-pr-xyz"

### Test Case 3: PR Comment
1. Add comment to existing PR
2. Verify NO new CodeRun created
3. Existing CodeRun handles comment event

**Expected:**
- ✅ No new CodeRun
- ✅ Existing CodeRun logs show comment processing

### Test Case 4: Multiple Events
1. Perform multiple actions quickly:
   - Push commit
   - Add comment
   - Add label
2. Verify only 1 CodeRun exists

**Expected:**
- ✅ Deduplication prevents duplicates
- ✅ All events handled by single CodeRun

---

## Rollback Plan

If the fix causes issues:

```bash
# Restore original sensor
kubectl apply -f atlas-pr-guardian-backup.yaml

# Restart sensor pod
kubectl delete pod -n argo -l sensor-name=atlas-pr-guardian

# Verify restoration
kubectl describe sensor atlas-pr-guardian -n argo
```

---

## Future Improvements

### 1. Controller-Level Deduplication
- Add admission webhook to controller
- Reject CodeRun creation if duplicate detected
- More robust than sensor-level logic

### 2. Rate Limiting
- Limit CodeRun creation rate per repository
- Prevent event storms from overwhelming cluster

### 3. TTL-Based Cleanup
- Add TTL to CodeRuns: delete after 24 hours if PR closed
- Automatic cleanup of stale resources

### 4. Metrics & Alerting
- Track CodeRun creation rate
- Alert if >3 CodeRuns per PR
- Dashboard showing Atlas activity per PR

---

## Files Changed

### New Files
- `scripts/cleanup-duplicate-atlas-coderuns.sh` - Cleanup script
- `infra/gitops/resources/sensors/atlas-pr-guardian-sensor-fixed.yaml` - Fixed sensor
- `docs/engineering/ATLAS_POD_PROLIFERATION_FIX.md` - This document

### Modified Files
- None (fix is in separate file to allow testing before replacing original)

### To Replace (After Testing)
- `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml` → Replace with `-fixed` version

---

## Success Criteria

✅ **Fix is successful when:**
1. No more than 1 Atlas CodeRun per open PR
2. No pods stuck in Pending state
3. Sensor logs show deduplication working
4. Resource usage stable (<30 Atlas pods total)
5. Atlas still responds to PR events correctly

---

## Related Issues

- **Original Implementation:** PR #1357, #1359, #1362, #1363, #1365
- **Documentation:** `docs/engineering/atlas-pr-guardian.md`
- **Sensor Deployment:** `infra/gitops/applications/atlas-pr-guardian-sensor.yaml`

---

## References

- [Argo Events Sensor Best Practices](https://argoproj.github.io/argo-events/sensors/more-about-sensors/)
- [Kubernetes Resource Quotas](https://kubernetes.io/docs/concepts/policy/resource-quotas/)
- [GitHub Webhook Events](https://docs.github.com/en/webhooks-and-events/webhooks/webhook-events-and-payloads)

---

**Status:** Ready for deployment  
**Next Step:** Run cleanup script and apply fixed sensor  
**ETA:** 15 minutes to complete remediation
