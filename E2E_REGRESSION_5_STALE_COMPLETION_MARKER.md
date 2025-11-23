# Regression #5: Stale Completion Marker Prevents Agent Work

**Date:** 2025-11-23  
**Found During:** E2E testing - Task 6 analysis  
**Severity:** HIGH - Agents skip work incorrectly  
**Agent:** Blaze (but affects all agents)

---

## Problem Statement

Blaze found an "Implementation completion marker" from a previous run and exited without doing any work, despite the branch being freshly recreated.

**From the log:**
```
Line 46: ℹ️ No existing PR found; recreating branch from origin/main
Line 50: Switched to a new branch 'feature/task-6-implementation'
...
Line 204: ✅ Implementation completion marker found - Rex already completed
Line 205: ✅ Factory confirmed task completion
Line 209: ⚠️ Failed to push branch feature/task-6-implementation; skipping auto PR creation
Line 212: ✅ Blaze Factory frontend implementation complete
```

---

## What Happened

### Timeline
1. **Previous run** (hours ago):
   - Task 6 ran and completed
   - Created `feature/task-6-implementation` branch
   - Left completion marker: `.implementation_complete` or similar
   - Branch later deleted from remote

2. **Current run**:
   - Blaze detects remote branch is gone
   - Recreates branch from `origin/main` (line 50)
   - **BUT:** Workspace PVC still has old completion marker!
   - Blaze finds marker and thinks work is done
   - Exits without implementing
   - No PR created
   - Workflow stuck at wait-for-pr

---

## Root Cause

**Completion markers persist in PVC workspace across branch recreations.**

**The flow:**
```
1. Branch deleted remotely (line 23-27 show deleted branches)
2. Blaze detects: "upstream is gone" (line 39-41)
3. Blaze recreates branch from main (line 46-52)
4. Blaze should clean completion markers ❌ (doesn't happen)
5. Blaze finds stale marker (line 204)
6. Blaze exits without work
```

---

## Evidence from Log

### Branch Was Recreated
```
Line 39: Your branch is based on 'origin/feature/task-6-implementation', but the upstream is gone.
Line 41: ⚠️ Upstream branch @{u} is gone (deleted from remote)
Line 46: ℹ️ No existing PR found; recreating branch from origin/main
Line 48: Removing frontend/public/
Line 49: Deleted branch feature/task-6-implementation (was a9c9bc6).
Line 50: Switched to a new branch 'feature/task-6-implementation'
```

### Stale Marker Found
```
Line 204: ✅ Implementation completion marker found - Rex already completed
```

### No Work Done
```
Line 205: ✅ Factory confirmed task completion
Line 206: ⚠️ Factory returned non-zero exit code 1, but completion probe passed
```

### Push Failed (No Changes)
```
Line 209: ⚠️ Failed to push branch feature/task-6-implementation; skipping auto PR creation
```

**Why push failed:** Branch was just recreated from main with no changes, nothing to push!

---

## Impact

**User experience:**
- ✅ Task routing correct (frontend → Blaze)
- ✅ Blaze starts up
- ❌ Blaze skips work (finds stale marker)
- ❌ No implementation happens
- ❌ No PR created
- ❌ Workflow stuck at wait-for-pr
- ❌ "Looks like Blaze didn't start" (actually it did, but exited fast)

**Symptoms:**
- Agent completes in ~6 minutes (too fast)
- No PR created
- Workflow stuck waiting for PR
- No actual implementation work done

---

## Fix Required

### Option 1: Clean Markers on Branch Recreation
When branch is recreated from main, clean all completion markers:

```bash
# In container script, after recreating branch:
if [ "$BRANCH_RECREATED" = "true" ]; then
  echo "🧹 Cleaning completion markers after branch recreation..."
  rm -f /workspace/task-${TASK_ID}/.implementation_complete
  rm -f /workspace/task-${TASK_ID}/.agent_done
  rm -f /workspace/task-${TASK_ID}/.quality_complete
  # etc.
fi
```

### Option 2: Verify Marker Validity
Before trusting completion marker, verify it's valid:

```bash
# Check if marker is from same commit
if [ -f ".implementation_complete" ]; then
  MARKER_COMMIT=$(cat .implementation_complete | grep "commit:" | cut -d: -f2)
  CURRENT_COMMIT=$(git rev-parse HEAD)
  if [ "$MARKER_COMMIT" != "$CURRENT_COMMIT" ]; then
    echo "⚠️ Completion marker from different commit, ignoring"
    rm -f .implementation_complete
  fi
fi
```

### Option 3: Check for Actual Work
Before accepting completion marker, verify work exists:

```bash
# For frontend tasks, check if frontend code exists
if [ "$TASK_LANGUAGE" = "typescript" ] && [ ! -d "frontend/src" ]; then
  echo "⚠️ Completion marker exists but no frontend code found"
  rm -f .implementation_complete
fi
```

---

## Immediate Workaround

**For this workflow:**
```bash
# Delete the PVC to clear stale state
kubectl delete pvc -n agent-platform -l task-id=6,agent=blaze

# Or manually delete completion markers in workspace
kubectl exec -n agent-platform <pod> -- rm -f /workspace/task-6/.implementation_complete
```

**Then restart workflow** and Blaze will do actual work.

---

## Where to Fix

**File:** `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`

**Section:** Branch recreation logic (around line 46-52 in the log)

**Add:** Completion marker cleanup after branch recreation

**Example fix:**
```bash
if [ "$RECREATED_FROM_MAIN" = "true" ]; then
  echo "🧹 Cleaning stale completion markers..."
  find /workspace -name ".implementation_complete" -delete
  find /workspace -name ".agent_done" -delete
  echo "✅ Workspace cleaned for fresh implementation"
fi
```

---

## Related Issues

- Workspace persistence (PVC) is good for resume
- But stale markers cause false positives
- Need smarter marker validation or cleanup
- Affects all agents (Rex, Blaze, etc.)

---

## Testing

After fix, verify:
1. Branch recreated from main
2. Completion markers cleaned
3. Agent does actual work
4. PR created successfully
5. Workflow proceeds to quality stage

---

## Temporary Fix

**For now,** manually clean the workspace:
```bash
# Find the PVC
kubectl get pvc -n agent-platform -l service=cto-parallel-test

# Delete it to force fresh workspace
kubectl delete pvc <pvc-name> -n agent-platform
```

Or update the workflow to skip the completion marker check for this run.
