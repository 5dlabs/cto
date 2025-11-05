# Morgan PM - GitHub Projects Status Tracking Investigation

**Date:** November 5, 2025  
**Workflow:** `play-project-workflow-template-qldnw`  
**Issue:** Task status not reflecting in GitHub Projects despite Morgan PM running

---

## Executive Summary

Morgan PM is successfully running as a daemon and monitoring workflow progress, but **GitHub Project field values (Stage, Current Agent) are not being updated in real-time**. The root cause is silent failures in the field update functions that return success even when lookups fail.

---

## Investigation Timeline

### 1. Initial Symptoms Observed

**Workflow State:**
- Parent workflow: `play-project-workflow-template-qldnw` (Running)
- Individual task workflows:
  - Task 1: `implementation-in-progress` (Running) - Rex
  - Task 3: `implementation-in-progress` (Running) - Rex
  - Task 4: `quality-in-progress` (Running) - Blaze
  - Task 6: `quality-in-progress` (Running) - Blaze

**Expected Behavior:**
GitHub Project should show task statuses and current agents

**Actual Behavior:**
- GitHub Issues created ✅
- Issue comments posted ✅
- GitHub Checks created ✅
- **Project fields NOT updated** ❌

### 2. Morgan PM Daemon Status Check

```bash
kubectl get pods -n agent-platform -l workflows.argoproj.io/workflow=play-project-workflow-template-qldnw

NAME                                                        READY   STATUS
play-project-workflow-template-qldnw-morgan-project-manager-144363591   2/2     Running
```

✅ **Morgan PM is running successfully**

### 3. Log Analysis

**Evidence of Successful Operations:**

```log
[2025-11-05 15:39:30 UTC] 🔄 State change detected for task-1: Rex (Implementation) | In Progress
[2025-11-05 15:39:37 UTC] 👤 Noting Rex (Implementation) is working on issue #337
https://github.com/5dlabs/cto-parallel-test/issues/337#issuecomment-3491956227
[2025-11-05 15:39:40 UTC] 📝 Added status history for task-1
```

Morgan IS:
- ✅ Detecting state changes
- ✅ Posting issue comments
- ✅ Creating GitHub Checks for PRs
- ✅ Updating issue labels
- ✅ Maintaining task-to-issue mapping

**Missing Evidence:**

No logs for:
- ❌ `set_project_item_status` function calls
- ❌ `set_project_item_agent` function calls
- ❌ GraphQL API calls to update project fields
- ❌ Warning messages about failed updates

### 4. Code Analysis

**Current Implementation** (`handle_task_event` function):

```bash
set_project_item_status "$project_id" "$item_id" "$task_status" 2>/dev/null || \
  log "⚠️  Could not update project status for item $item_id"
  
set_project_item_agent "$project_id" "$item_id" "$current_agent" 2>/dev/null || \
  log "⚠️  Could not update project agent for item $item_id"
```

**Silent Failure Mode** (inside `set_project_item_status`):

```bash
set_project_item_status() {
  local field_id=$(get_project_field "$project_id" "Stage")
  
  if [[ -z "$field_id" ]] || [[ "$field_id" == "null" ]]; then
    return 0  # ⚠️ SILENT SUCCESS even though update failed!
  fi
  
  local option_id=$(get_field_option_id "$project_id" "$field_id" "$status")
  
  if [[ -z "$option_id" ]] || [[ "$option_id" == "null" ]]; then
    return 0  # ⚠️ SILENT SUCCESS even though update failed!
  fi
  
  update_project_item_field "$project_id" "$item_id" "$field_id" "$option_id"
}
```

**Problem:** Functions return exit code 0 (success) even when they can't find required IDs, so the `||` fallback never triggers warning logs.

### 5. Data Verification

**Task-to-Issue Mapping** (`/shared/morgan-pm/task-issue-map.json`):

```json
{
  "issue_number": 337,
  "item_id": "PVTI_lADOC8B7k84BHVJAzggxb60",
  "node_id": "I_kwDOQKXbrM7WFGHr"
}
```

✅ Mapping is correct and persistent

**Project ID:**
- `PVT_kwDOC8B7k84BHVJA` (Organization-level project)
- Successfully created during initialization
- Successfully linked to repository

---

## Root Cause

The `set_project_item_status` and `set_project_item_agent` functions have **silent failure modes**:

1. Functions are called during `handle_task_event` ✅
2. `get_project_field()` lookup may be failing → returns null/empty
3. Function returns 0 (success) instead of non-zero (failure)
4. No warning log is generated because `||` fallback never triggers
5. GraphQL API call is never made
6. Project fields remain unchanged

**Likely Failure Points:**

1. **Field Lookup Issue:** `get_project_field()` not finding "Stage" or "Current Agent" fields
2. **Option Lookup Issue:** `get_field_option_id()` not finding options like "In Progress" or "Rex (Implementation)"
3. **Caching Problem:** Field/option IDs cached during initialization may be stale
4. **GraphQL Schema Mismatch:** Project structure may differ from expectations

---

## Fix Applied

### Phase 1: Diagnostic Logging (PR #1254)

Added comprehensive debug logging to both functions:

```bash
set_project_item_status() {
  local project_id="$1"
  local item_id="$2"
  local status="$3"
  
  log "🔍 DEBUG: set_project_item_status called - item=$item_id, status=$status"
  
  local field_id=$(get_project_field "$project_id" "Stage")
  log "🔍 DEBUG: Stage field_id=$field_id"
  
  if [[ -z "$field_id" ]] || [[ "$field_id" == "null" ]]; then
    log "⚠️  DEBUG: Stage field_id is null or empty, returning"
    return 0
  fi
  
  local option_id=$(get_field_option_id "$project_id" "$field_id" "$status")
  log "🔍 DEBUG: Status option_id=$option_id for status=$status"
  
  if [[ -z "$option_id" ]] || [[ "$option_id" == "null" ]]; then
    log "⚠️  DEBUG: Status option_id is null or empty for status=$status, returning"
    return 0
  fi
  
  log "✅ DEBUG: Calling update_project_item_field for status update"
  update_project_item_field "$project_id" "$item_id" "$field_id" "$option_id"
}
```

Same logging added to `set_project_item_agent()`.

### Phase 2: Root Cause Identification (Pending)

After PR #1254 merges and ArgoCD syncs:

1. Restart Morgan PM pod
2. Observe debug logs during next state change
3. Identify exact failure point:
   - Field lookup failing?
   - Option lookup failing?
   - GraphQL call failing?
4. Apply targeted fix based on findings

---

## Expected Debug Output

Once logging is active, we should see one of these scenarios:

### Scenario A: Field Lookup Failure
```log
🔍 DEBUG: set_project_item_status called - item=PVTI_lADOC8B7k84BHVJAzggxb60, status=In Progress
🔍 DEBUG: Stage field_id=null
⚠️  DEBUG: Stage field_id is null or empty, returning
```
**Fix:** Investigate `get_project_field()` - field name mismatch or caching issue

### Scenario B: Option Lookup Failure
```log
🔍 DEBUG: set_project_item_status called - item=PVTI_lADOC8B7k84BHVJAzggxb60, status=In Progress
🔍 DEBUG: Stage field_id=PVTF_<field_id>
🔍 DEBUG: Status option_id=null for status=In Progress
⚠️  DEBUG: Status option_id is null or empty for status=In Progress, returning
```
**Fix:** Investigate `get_field_option_id()` - option name mismatch or need to create option

### Scenario C: Successful Execution
```log
🔍 DEBUG: set_project_item_status called - item=PVTI_lADOC8B7k84BHVJAzggxb60, status=In Progress
🔍 DEBUG: Stage field_id=PVTF_<field_id>
🔍 DEBUG: Status option_id=PVTFO_<option_id> for status=In Progress
✅ DEBUG: Calling update_project_item_field for status update
```
**Next:** Add logging to `update_project_item_field()` to verify GraphQL call

---

## Next Steps

1. ✅ **PR Created:** #1254 with debug logging
2. ⏳ **Waiting:** PR merge + ArgoCD sync
3. ⏳ **Restart:** Morgan PM pod to load new ConfigMap
4. ⏳ **Observe:** Debug logs during next task state change
5. ⏳ **Fix:** Apply targeted remedy based on diagnostic output
6. ⏳ **Verify:** Confirm project fields update correctly
7. ⏳ **Clean Up:** Remove debug logging once root cause is fixed

---

## Related Files

- **Morgan PM Script:** `infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs`
- **Helper Functions:** `infra/charts/controller/agent-templates/pm/github-projects-helpers.sh.hbs`
- **ConfigMap Template:** `infra/charts/controller/templates/agent-templates-pm.yaml`
- **Investigation PR:** #1254

---

## Additional Context

### Why Silent Failures Are Problematic

Morgan's design philosophy: "Always update project fields (idempotent, cheap operations)"

The code expects these calls to either:
1. **Succeed silently** (field already has correct value)
2. **Fail loudly** (API error, permission issue)

Current behavior:
3. **Fail silently** (lookups return null but function returns 0)

This breaks the idempotency contract and makes debugging impossible without logging.

### Design Improvement Recommendations

After fixing the immediate issue, consider:

1. **Return Non-Zero on Lookup Failures:** Functions should return exit code 1 when lookups fail
2. **Cache Validation:** Periodically refresh field/option ID caches
3. **Health Checks:** Add periodic validation that project fields are accessible
4. **Metrics:** Expose Prometheus metrics for project update success/failure rates

---

**Investigation by:** Claude (AI Assistant)  
**Workflow:** play-project-workflow-template-qldnw  
**Status:** Diagnostic logging added, awaiting PR merge  
**PR:** #1254

