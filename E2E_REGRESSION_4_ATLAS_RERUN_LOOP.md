# Regression #4: Atlas Stage Re-runs When Resuming from Atlas

**Date:** 2025-11-23  
**Found During:** E2E testing code review  
**Severity:** HIGH - Resume gets stuck in infinite loop  
**Related:** Regression #1 (Atlas resume fix)

---

## Problem Statement

When resuming a workflow from `waiting-atlas-integration` stage, the Atlas-related tasks re-execute instead of being skipped, causing the workflow to:

1. Re-write the stage to `waiting-atlas-integration` (overwriting progress)
2. Re-suspend waiting for Atlas event
3. Get stuck in an infinite loop - never progressing to merge

---

## Root Cause

The `update-to-waiting-atlas` and `wait-for-atlas-integration` tasks have incomplete `when` conditions:

```yaml
# Before (BUGGY):
- name: update-to-waiting-atlas
  when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' != 'waiting-merge'"
  # ❌ Only checks for waiting-merge, not waiting-atlas!

- name: wait-for-atlas-integration
  when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' != 'waiting-merge'"
  # ❌ Will re-run when resuming from waiting-atlas!
```

### The Loop

```
1. ConfigMap has: stage = "waiting-atlas-integration"
2. Workflow created with task-id = 4
3. Resume point determined: "waiting-atlas"
4. Agent-sequence skips ✅ (resume != implementation)
5. update-to-waiting-atlas checks: resume != 'waiting-merge'
   → TRUE! (resume is 'waiting-atlas')
   → Task RUNS ❌
6. Overwrites ConfigMap stage to "waiting-atlas-integration"
7. wait-for-atlas-integration runs
8. Workflow suspends again
9. Repeat forever (never progresses to merge) ♾️
```

---

## Impact

**Symptoms:**
- Workflow appears to resume but gets stuck
- ConfigMap stage never advances past `waiting-atlas-integration`
- Atlas event triggers cause workflow to re-suspend
- Infinite loop prevents completion
- Wastes compute and API calls

**User impact:**
- Cannot complete workflows that reach Atlas stage
- Manual intervention required (delete ConfigMap or patch stage)
- Lost progress and time

---

## Fix

Add `waiting-atlas` to the skip conditions:

```yaml
# Before (INCOMPLETE):
when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' != 'waiting-merge'"

# After (COMPLETE):
when: "'{{tasks.check-main-resume.outputs.parameters.resume-stage}}' != 'waiting-atlas' && 
      '{{tasks.check-main-resume.outputs.parameters.resume-stage}}' != 'waiting-merge'"
```

**Applied to both tasks:**
- `update-to-waiting-atlas` - Skip if resuming at Atlas or merge
- `wait-for-atlas-integration` - Skip if resuming at Atlas or merge

---

## Logic Flow After Fix

### Scenario 1: Fresh Start (resume = 'implementation')
```
✅ initialize-stage runs
✅ update-to-implementation runs
✅ agent-sequence runs (all agents execute)
✅ update-to-waiting-atlas runs (sets stage)
✅ wait-for-atlas-integration runs (suspends)
```

### Scenario 2: Resume from Atlas (resume = 'waiting-atlas')
```
⏭️ initialize-stage skips (resume != implementation)
⏭️ update-to-implementation skips (resume != implementation)
⏭️ agent-sequence skips (resume == waiting-atlas)
⏭️ update-to-waiting-atlas skips (resume == waiting-atlas) ← FIX
⏭️ wait-for-atlas-integration skips (resume == waiting-atlas) ← FIX
✅ update-to-waiting-merge runs (advances to merge!)
✅ wait-merge-to-main runs
```

### Scenario 3: Resume from Merge (resume = 'waiting-merge')
```
⏭️ All stages skip until wait-merge-to-main
✅ wait-merge-to-main runs
✅ complete-task runs
```

---

## Testing

### Test Case: Resume from Atlas Stage

**Setup:**
1. Create workflow that reaches `waiting-atlas-integration`
2. ConfigMap stores: `stage = "waiting-atlas-integration"`
3. Delete workflow
4. Create new workflow with same task-id

**Expected (After Fix):**
```
✅ check-main-resume: resume-stage = "waiting-atlas"
⏭️ initialize-stage: SKIPPED
⏭️ update-to-implementation: SKIPPED
⏭️ agent-sequence: SKIPPED
⏭️ update-to-waiting-atlas: SKIPPED (new!)
⏭️ wait-for-atlas-integration: SKIPPED (new!)
✅ update-to-waiting-merge: RUNS
✅ wait-merge-to-main: RUNS
✅ Workflow progresses forward!
```

**Before Fix (Broken):**
```
✅ check-main-resume: resume-stage = "waiting-atlas"
⏭️ agent-sequence: SKIPPED
❌ update-to-waiting-atlas: RUNS (should skip!)
❌ wait-for-atlas-integration: RUNS (should skip!)
❌ Workflow stuck in loop, never progresses
```

---

## Complete When Conditions

After all fixes, here's the complete set:

| Task | When Condition |
|------|----------------|
| `check-main-resume` | (always runs) |
| `initialize-stage` | `resume == 'implementation'` |
| `update-to-implementation` | `resume == 'implementation'` |
| `agent-sequence` | `resume != 'waiting-atlas' && resume != 'waiting-merge'` |
| `update-to-waiting-atlas` | `resume != 'waiting-atlas' && resume != 'waiting-merge'` ← FIXED |
| `wait-for-atlas-integration` | `resume != 'waiting-atlas' && resume != 'waiting-merge'` ← FIXED |
| `update-to-waiting-merge` | (always runs after dependencies) |
| `wait-merge-to-main` | (always runs after dependencies) |
| `complete-task` | (always runs after dependencies) |

---

## Related Issues

- Regression #1: Atlas stages added to resume logic
- Regression #2: DAG syntax error (mixed depends/dependencies)
- Discovered during: Cursor bot code review
- Part of E2E testing on hotfix branch

---

## Files Changed

- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
  - Line 283: `update-to-waiting-atlas` when condition
  - Line 294: `wait-for-atlas-integration` when condition
