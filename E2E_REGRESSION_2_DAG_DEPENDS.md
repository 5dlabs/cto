# Regression #2: Argo DAG Cannot Mix 'depends' and 'dependencies'

**Date:** 2025-11-23  
**Found During:** E2E parallel workflow testing  
**Severity:** CRITICAL - All workflows fail immediately on startup  
**Related:** Regression #1 (Atlas resume fix)

---

## Problem Statement

All workflows fail immediately with error:
```
invalid spec: templates.main cannot use both 'depends' and 'dependencies' in the same DAG template
```

**Log Evidence:**
```
❌ Workflow play-task-1-phvx5 failed
❌ Workflow play-task-3-5kwx9 failed
❌ Workflow play-task-4-5jzf6 failed
❌ Workflow play-task-6-z88f9 failed
```

---

## Root Cause

While fixing Regression #1 (Atlas resume), I added `depends` expressions to handle skipped tasks but kept the original `dependencies` arrays.

**Argo Workflows constraint:** Cannot use both `depends` and `dependencies` in the same DAG task.

### Examples of Mixed Fields (Before Fix)

```yaml
# INVALID - Has both dependencies AND depends
- name: update-to-implementation
  dependencies: [initialize-stage, check-main-resume]  # ❌
  depends: "(initialize-stage.Succeeded || initialize-stage.Skipped) && check-main-resume.Succeeded"  # ❌
  template: update-workflow-stage

# INVALID - Has both
- name: agent-sequence
  dependencies: [update-to-implementation, check-main-resume]  # ❌
  depends: "(update-to-implementation.Succeeded || update-to-implementation.Skipped) && check-main-resume.Succeeded"  # ❌
  template: agent-sequence
```

---

## Impact

**Severity:** CRITICAL - Total workflow failure

- ❌ No workflows can start
- ❌ Parallel execution completely broken
- ❌ Sequential execution also broken
- ❌ Both fresh starts and resumes fail
- ❌ 100% failure rate

**Blast radius:**
- All play workflows (sequential and parallel)
- All tasks in all levels
- Affects all repositories

---

## Fix Applied

Replaced `dependencies` with `depends` throughout the main DAG.

### Before (Mixed - Invalid)
```yaml
- name: initialize-stage
  dependencies: [check-main-resume]  # ❌
  template: update-workflow-stage

- name: update-to-implementation
  dependencies: [initialize-stage, check-main-resume]  # ❌
  depends: "(initialize-stage.Succeeded || ...)"  # Mixed with dependencies
```

### After (Pure depends - Valid)
```yaml
- name: initialize-stage
  depends: "check-main-resume.Succeeded"  # ✅
  template: update-workflow-stage

- name: update-to-implementation
  depends: "(initialize-stage.Succeeded || initialize-stage.Skipped) && check-main-resume.Succeeded"  # ✅
```

### All Tasks Fixed

1. **initialize-stage**
   ```yaml
   depends: "check-main-resume.Succeeded"
   ```

2. **update-to-implementation**
   ```yaml
   depends: "(initialize-stage.Succeeded || initialize-stage.Skipped) && check-main-resume.Succeeded"
   ```

3. **agent-sequence**
   ```yaml
   depends: "(update-to-implementation.Succeeded || update-to-implementation.Skipped) && check-main-resume.Succeeded"
   ```

4. **update-to-waiting-atlas**
   ```yaml
   depends: "(agent-sequence.Succeeded || agent-sequence.Skipped) && check-main-resume.Succeeded"
   ```

5. **wait-for-atlas-integration**
   ```yaml
   depends: "(update-to-waiting-atlas.Succeeded || update-to-waiting-atlas.Skipped) && check-main-resume.Succeeded"
   ```

6. **update-to-waiting-merge**
   ```yaml
   depends: "wait-for-atlas-integration.Succeeded || wait-for-atlas-integration.Skipped"
   ```

7. **wait-merge-to-main**
   ```yaml
   depends: "update-to-waiting-merge.Succeeded"
   ```

8. **complete-task**
   ```yaml
   depends: "wait-merge-to-main.Succeeded"
   ```

---

## Why depends is Better Than dependencies

**`dependencies` (Array-based):**
- Simple dependency list
- Cannot express conditional logic
- All dependencies must succeed
- No support for OR conditions

**`depends` (Expression-based):**
- Boolean expressions
- Supports OR, AND, NOT logic
- Can handle skipped tasks: `task.Succeeded || task.Skipped`
- More flexible for complex workflows

**For resume logic:** We NEED `depends` because:
- Tasks can be skipped based on resume point
- Need `|| task.Skipped` to allow later tasks to run
- Need conditional dependencies based on workflow state

---

## Testing

### Test 1: Fresh Workflow Start
**Expected:**
- ✅ All tasks run in order
- ✅ No spec validation errors
- ✅ Workflow proceeds through all stages

### Test 2: Resume from Atlas
**Expected:**
- ✅ `initialize-stage` skipped (resume != implementation)
- ✅ `update-to-implementation` skipped (resume != implementation)
- ✅ `agent-sequence` skipped (resume == waiting-atlas)
- ✅ `update-to-waiting-atlas` runs (handles skipped dependencies)
- ✅ `wait-for-atlas-integration` runs
- ✅ Workflow suspends waiting for Atlas event

### Test 3: Parallel Execution
**Expected:**
- ✅ Multiple tasks launch in parallel
- ✅ All tasks succeed (no spec errors)
- ✅ Level-based execution works

---

## Prevention

**Lesson learned:** When using Argo DAG tasks:
- ✅ Use EITHER `dependencies` OR `depends` (never both)
- ✅ Prefer `depends` for complex conditional logic
- ✅ Test spec validation before pushing
- ✅ Check Argo documentation for DAG constraints

**Future changes:**
- Always validate workflow specs with `argo lint`
- Test both sequential and parallel workflows
- Verify with simple workflow creation before full E2E

---

## Files Affected

- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
  - Lines 242-325: Main DAG task definitions
  - Converted 8 tasks from `dependencies` to `depends`

---

## Related Issues

- Regression #1: Atlas integration stages missing from resume logic
- Root cause: Incomplete conversion to `depends` syntax when adding resume logic
- Affects: All workflows (sequential and parallel)
- Fixed: Same commit as stage overwrite fix

---

## Validation

```bash
# Check workflow spec is valid
argo template lint infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml

# Test workflow creation
kubectl create -f test-workflow.yaml --dry-run=server

# Verify no mixed fields
grep -E "dependencies|depends" play-workflow-template.yaml
```

**After fix:**
- No tasks should have both `dependencies` and `depends`
- All DAG tasks use only `depends` expressions
- Workflow spec validates successfully
