# Atlas Task Zero Removal - Implementation Complete

## Overview

This PR removes the confusing "Task 0" default from Atlas sensors, aligning the implementation with the documented architecture where **Atlas is an integration coordinator, not a task implementer**.

## Changes Made

### 1. atlas-pr-monitor-sensor.yaml

**Problem**: Atlas was defaulting to `taskId: "0"` when no task ID was found in PR labels/branch/title.

**Fix**: 
- Removed the `TASK_ID="0"` fallback
- Made `taskId` field conditional in CodeRun spec (only included if task ID is found)
- Updated logging to clarify when Atlas is running as integration coordinator vs. task-specific guardian
- Changed TASK_ID env var to use `${TASK_ID:-none}` fallback

**Impact**: Atlas guardians will no longer show "Task 0" - they'll either show the actual task ID (if PR is part of a task workflow) or omit the taskId field entirely (for standalone PRs).

### 2. atlas-batch-integration-sensor.yaml

**Problem**: Batch integration was using hardcoded `taskId: 999`.

**Fix**:
- Removed `taskId: 999` entirely
- Changed service name from `atlas-integration` to `atlas` for consistency

**Impact**: Batch integration CodeRuns will not have a taskId, correctly reflecting that they coordinate multiple PRs rather than implementing a specific task.

### 3. Service Name Standardization

**Standardized all Atlas CodeRuns to use `service: "atlas"`**:
- Guardian mode: `service: "atlas"` ✅
- Integration mode: `service: "atlas"` ✅
- Tess approval integration: `service: "atlas"` ✅ (already correct)

This ensures consistent template selection in the controller.

## Architecture Alignment

These changes align the code with the documented Atlas architecture:

### Atlas's True Purpose (from ATLAS_REDESIGN_PLAN.md)

**What Atlas Does:**
- Batch integration of multiple approved PRs
- Merge conflict resolution
- Integration testing of combined changes
- Final merge coordination

**What Atlas Does NOT Do:**
- ❌ Implement tasks
- ❌ Have a task ID (unless monitoring a task-specific PR)
- ❌ Run as part of regular task workflows

## Before vs. After

### Before
```yaml
# All Atlas guardians showed Task 0
taskId: "0"
service: "atlas-pr-guardian"  # Inconsistent

# Batch integration showed Task 999
taskId: 999
service: "atlas-integration"  # Inconsistent
```

### After
```yaml
# Atlas guardians only have taskId if PR is task-related
# taskId field omitted for standalone PRs
service: "atlas"  # Consistent

# Batch integration has no taskId
# taskId field omitted
service: "atlas"  # Consistent
```

## Testing

### Verification Steps

1. **Check existing Atlas CodeRuns**:
   ```bash
   kubectl get coderuns -n agent-platform -l agent=atlas -o json | \
     jq '.items[] | {name: .metadata.name, taskId: .spec.taskId, service: .spec.service}'
   ```

2. **Trigger new guardian** (open a PR without task labels):
   ```bash
   # Should create CodeRun without taskId field
   ```

3. **Trigger new guardian** (open a PR with task-1 label):
   ```bash
   # Should create CodeRun with taskId: "1"
   ```

4. **Check sensor logs**:
   ```bash
   kubectl logs -n argo -l sensor-name=atlas-pr-monitor --tail=50
   ```

### Expected Behavior

**Standalone PR** (no task labels):
- Log: "No task ID (integration coordinator)"
- CodeRun: No `taskId` field in spec
- Env: `TASK_ID=none`

**Task-specific PR** (has task-5 label):
- Log: "Task: 5"
- CodeRun: `taskId: "5"` in spec
- Env: `TASK_ID=5`

**Batch Integration**:
- No `taskId` field in spec
- Service: `atlas`

## Deployment

These changes will be automatically deployed via ArgoCD when merged to main:

1. Sensors will be updated with new logic
2. New Atlas CodeRuns will use the corrected configuration
3. Existing running Atlas guardians will continue with old config (graceful)
4. New PR events will trigger guardians with new config

## Benefits

✅ **Clearer UX** - No more confusing "Task 0"  
✅ **Architecture Alignment** - Code matches documentation  
✅ **Consistent Naming** - All Atlas CodeRuns use `service: "atlas"`  
✅ **Correct Semantics** - Atlas only has taskId when actually monitoring a task  
✅ **Production Ready** - Atlas can now be fully tested end-to-end  

## Related Documentation

- [ATLAS_REDESIGN_PLAN.md](./ATLAS_REDESIGN_PLAN.md) - Complete redesign plan
- [docs/atlas-integration-architecture.md](./docs/atlas-integration-architecture.md) - Architecture overview
- [docs/engineering/atlas-pr-guardian.md](./docs/engineering/atlas-pr-guardian.md) - Guardian mode details

## Next Steps

After this PR merges:

1. ✅ Atlas is production-ready for E2E testing
2. Monitor Atlas guardians on real PRs
3. Implement intelligent conflict resolution (currently TODO)
4. Implement intelligent CI failure recovery (currently TODO)
5. Implement Bugbot feedback resolution (currently TODO)

---

**Status**: Ready for Review & Merge  
**Impact**: Low Risk - Cosmetic changes to task ID handling  
**Testing**: Can be verified immediately after deployment

