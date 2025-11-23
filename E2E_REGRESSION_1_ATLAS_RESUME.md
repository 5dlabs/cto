# Regression #1: Atlas Integration Stages Missing from Resume Logic

**Date:** 2025-11-23  
**Found During:** E2E testing after MCP validation, resume, and Atlas integration changes  
**Severity:** HIGH - Workflows cannot resume from Atlas integration stages

---

## Problem Statement

The workflow template added new Atlas integration stages:
- `waiting-atlas-integration`
- `atlas-integration-in-progress`

But the resume logic in `determine-resume-point` doesn't recognize these stages, causing workflows to restart from the beginning instead of resuming at the correct point.

### Evidence

**ConfigMap shows:**
```yaml
data:
  stage: waiting-atlas-integration
  current-task-id: "4"
  workflow-name: play-task-4-jw7lc
```

**Workflow deleted:**
```bash
$ kubectl get workflow play-task-4-jw7lc -n agent-platform
Error from server (NotFound): workflows.argoproj.io "play-task-4-jw7lc" not found
```

**Resume logic MISSING Atlas stages** (line 2238):
```bash
case "$STORED_STAGE" in
  "pending"|"implementation-in-progress") → implementation
  "quality-in-progress") → quality
  "security-in-progress") → security
  "testing-in-progress") → testing
  "waiting-pr-merged") → waiting-merge
  "completed") → implementation
  *) → implementation  # ❌ Atlas stages fall here!
esac
```

---

## Current Stage Flow

```
implementation-in-progress
  ↓
quality-in-progress
  ↓
security-in-progress
  ↓
testing-in-progress
  ↓
waiting-atlas-integration ← MISSING FROM RESUME
  ↓
atlas-integration-in-progress ← MISSING FROM RESUME
  ↓
waiting-pr-merged
  ↓
completed
```

---

## Impact

When a workflow is deleted (or times out) at `waiting-atlas-integration`:

1. ✅ ConfigMap correctly stores stage
2. ✅ New workflow reads ConfigMap
3. ❌ Stage not recognized, falls to default case
4. ❌ Restarts from `implementation` (wastes time/tokens)
5. ❌ Re-runs all completed stages

**User Impact:**
- Lost progress after testing completes
- Wasted compute/API calls re-running implementation/quality/security/testing
- ~30-60 minutes of wasted agent time per resume

---

## Root Cause

Atlas integration was added in recent changes (today per user), but two templates weren't updated:

1. **`determine-resume-point`** template (line 2238-2269)
   - Case statement doesn't include Atlas stages
   
2. **`check-stage-needed`** template (line 2303)
   - STAGES array missing atlas stages:
   ```bash
   STAGES=("implementation" "quality" "security" "testing" "waiting-merge" "completed")
   # Missing: "waiting-atlas-integration" and "atlas-integration-in-progress"
   ```

---

## Fix Required

### 1. Update `determine-resume-point` (line 2238)

Add cases for Atlas stages:

```bash
case "$STORED_STAGE" in
  "pending"|"implementation-in-progress")
    RESUME_STAGE="implementation"
    ;;
  "quality-in-progress")
    RESUME_STAGE="quality"
    ;;
  "security-in-progress")
    RESUME_STAGE="security"
    ;;
  "testing-in-progress")
    RESUME_STAGE="testing"
    ;;
  "waiting-atlas-integration"|"atlas-integration-in-progress")  # ADD THIS
    RESUME_STAGE="waiting-atlas"
    echo "✅ Resuming at: Atlas Integration"
    ;;
  "waiting-pr-merged")
    RESUME_STAGE="waiting-merge"
    ;;
  "completed")
    RESUME_STAGE="implementation"
    ;;
  *)
    echo "⚠️  Unknown stage: $STORED_STAGE, starting from beginning"
    RESUME_STAGE="implementation"
    ;;
esac
```

### 2. Update `check-stage-needed` (line 2303)

Add atlas stages to progression:

```bash
# Stage order (must match workflow progression)
# implementation → quality → security → testing → waiting-atlas → waiting-merge → completed
STAGES=("implementation" "quality" "security" "testing" "waiting-atlas" "waiting-merge" "completed")
```

### 3. Update workflow DAG skip logic

Need to ensure workflow skips correctly when resuming at `waiting-atlas`:

```yaml
- - name: should-run-atlas
    template: check-stage-needed
    arguments:
      parameters:
        - name: resume-stage
          value: "{{steps.check-resume-point.outputs.parameters.resume-stage}}"
        - name: current-stage
          value: "waiting-atlas"

- - name: wait-for-atlas-integration
    when: "'{{steps.should-run-atlas.outputs.parameters.should-run}}' == 'true'"
    template: suspend-for-event
```

---

## Test Plan

1. **Setup:** Create workflow that reaches `waiting-atlas-integration`
2. **Delete workflow:** Simulate timeout/manual deletion
3. **Verify ConfigMap:** Confirm stage stored correctly
4. **Create new workflow:** Same task-id, same repository
5. **Expected:** Workflow resumes at `wait-for-atlas-integration` step
6. **Verify:** Implementation/quality/security/testing are skipped

---

## Files Affected

- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
  - Line 2238: `determine-resume-point` case statement
  - Line 2303: `check-stage-needed` STAGES array
  - Lines 260-290: Atlas integration DAG steps (need skip logic)

---

## Related Issues

- This is related to Atlas integration rework mentioned in git diff
- Part of recent changes to workflow stage progression
- Compounds with resume functionality testing

---

## Fix Applied ✅

### Changes Made

1. **Updated `determine-resume-point` template** (line 2255-2258)
   - Added case for `waiting-atlas-integration` and `atlas-integration-in-progress`
   - Maps to `RESUME_STAGE="waiting-atlas"`

2. **Updated `check-stage-needed` template** (line 2307)
   - Added `waiting-atlas` to STAGES array
   - Updated progression comment to include Atlas stage

3. **Updated main DAG** (lines 230-303)
   - Added `check-main-resume` as first task in main DAG
   - Added conditional execution to `agent-sequence` (skip if resuming at atlas or later)
   - Added `depends` expressions to handle skipped tasks properly
   - Atlas stages now check resume point and skip if already past that stage

### Key Changes

```yaml
# Resume point determination now includes:
"waiting-atlas-integration"|"atlas-integration-in-progress")
  RESUME_STAGE="waiting-atlas"
  echo "✅ Resuming at: Atlas Integration"
  ;;

# Stage progression updated:
STAGES=("implementation" "quality" "security" "testing" "waiting-atlas" "waiting-merge" "completed")

# Main DAG now checks resume point first:
- name: check-main-resume
  template: determine-resume-point

# Agent sequence skips if resuming at atlas:
- name: agent-sequence
  when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' != 'waiting-atlas' && ..."

# Atlas stages use depends to handle skipped predecessors:
- name: update-to-waiting-atlas
  depends: "(agent-sequence.Succeeded || agent-sequence.Skipped) && check-main-resume.Succeeded"
```

## Critical Follow-Up Fix

### Issue: Stage Updates Overwrite Resume State

After the initial fix, discovered that `initialize-stage` and `update-to-implementation` were running unconditionally, overwriting the ConfigMap's resume state with `pending` → `implementation-in-progress` before skipping `agent-sequence`.

**Impact:** Resume point lost, workflow thinks it's at implementation stage even when resuming at Atlas/merge.

**Fix Applied:**
```yaml
# Only initialize stage when starting fresh
- name: initialize-stage
  when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' == 'implementation'"

# Only update to implementation when starting fresh  
- name: update-to-implementation
  when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' == 'implementation'"
  depends: "(initialize-stage.Succeeded || initialize-stage.Skipped) && ..."

# Agent sequence handles skipped dependencies
- name: agent-sequence
  depends: "(update-to-implementation.Succeeded || update-to-implementation.Skipped) && ..."
```

## Next Steps

1. ✅ **DONE:** Applied fix to resume logic
2. ✅ **DONE:** Fixed stage update overwrite issue
3. **TODO:** Test resume from each stage (including atlas stages)
4. **TODO:** Verify stage transition validation still works  
5. **TODO:** Document Atlas integration stage semantics
6. **TODO:** Check if parallel-execution workflow template needs same fix
