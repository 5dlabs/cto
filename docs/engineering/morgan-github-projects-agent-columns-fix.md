# Morgan PM - Agent-Based Columns Fix

**Date:** November 6, 2025  
**Issue:** GitHub Project status not syncing with Kubernetes job status  
**Solution:** Redesigned GitHub Projects board to use agent-based columns instead of generic status columns

---

## The Problem

Morgan PM had **two critical issues**:

### 1. Silent Failures in Field Updates
The `set_project_item_status()` and `set_project_item_agent()` functions returned exit code `0` (success) even when lookups failed, causing silent failures:

```bash
# Old behavior - SILENT FAILURE
set_project_item_status() {
  local field_id=$(get_project_field "$project_id" "Stage")
  
  if [[ -z "$field_id" ]]; then
    return 0  # ❌ Returns success even though it failed!
  fi
  # ... more code
}
```

This meant:
- ✅ Function calls succeeded (exit code 0)
- ❌ GraphQL API calls never executed
- ❌ Project fields never updated
- ❌ No warnings logged (because `||` fallback never triggered)

### 2. Poor UX Design - Generic Status Columns

The original design used generic columns:
- **To Do** | **In Progress** | **In Review** | **Done** | **Blocked**

This had several problems:
- ❌ Can't assign issues to agents (they're not GitHub users)
- ❌ No visual indication of which agent is working
- ❌ Required separate "Current Agent" field that wasn't visible
- ❌ Generic columns don't match the multi-agent workflow

---

## The Solution: Agent-Based Columns + Real Cluster State

**Key Principle: GitHub Projects reflects ACTUAL cluster activity, not theoretical stages**

**New Board Structure:**
```
┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬────────────┐
│ Pending  │   Rex    │   Cleo   │  Cipher  │   Tess   │  Atlas   │   Bolt   │ Complete ✅ │
├──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼────────────┤
│ Task 2   │ Task 1   │ Task 3   │ Task 7   │ Task 4   │ Task 9   │ Task 11  │ Task 5     │
│ Task 8   │ Task 10  │ Task 6   │          │          │          │          │ Task 12    │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴────────────┘
```

**Benefits:**
- ✅ **Immediate visual clarity** - see which agent is working on what
- ✅ **Matches workflow reality** - columns ARE the agent stages
- ✅ **Real cluster state** - detects actual running agents from Kubernetes
- ✅ **Simplified field structure** - one "Stage" field instead of two separate fields
- ✅ **Better UX** - drag tasks between agents to change workflow stage
- ✅ **Self-documenting** - the board tells the story of the multi-agent pipeline
- ✅ **Accurate tracking** - three-tier detection strategy prioritizes actual pod/CodeRun state

---

## Implementation Changes

### Priority: Real Cluster State Over Theoretical Mappings

**Critical Design Principle:**
Morgan now uses a **three-tier detection strategy** to ensure the GitHub Projects board reflects **actual cluster activity**:

1. **Tier 1 (Highest Priority):** Check for RUNNING pods with `task-id` label
2. **Tier 2:** Check active CodeRun resources with agent information
3. **Tier 3 (Fallback Only):** Use stage mapping when cluster state unavailable

```bash
# Example: Detecting actual running agent
get_actual_running_agent() {
  # Strategy 1: RUNNING pods (most accurate)
  kubectl get pods -l "task-id=$task_id" --field-selector=status.phase=Running
  # Returns: "rex" from pod label → Formatted as "Rex (Implementation)"
  
  # Strategy 2: Active CodeRuns
  kubectl get coderuns -l "task-id=$task_id" 
  # Returns: githubApp field from CodeRun spec
  
  # Strategy 3: Workflow stage (fallback)
  # Only used if Strategies 1 and 2 return nothing
}
```

**Why This Matters:**
- ❌ **Old approach:** Mapped workflow stage to agent (theoretical)
- ✅ **New approach:** Detects actual running agent from cluster (reality)
- 📊 **Result:** Board shows WHO is actually working, not who should be working

---

### 1. Fixed Silent Failures with Proper Error Handling

```bash
# New behavior - LOUD FAILURES with detailed logging
set_project_item_stage() {
  local project_id="$1"
  local item_id="$2"
  local stage="$3"
  
  log "🔍 Setting project item stage: item=$item_id, stage='$stage'"
  
  local field_id=$(get_project_field "$project_id" "Stage")
  
  if [[ -z "$field_id" ]] || [[ "$field_id" == "null" ]]; then
    log "❌ FAILED: Stage field not found in project $project_id"
    log "💡 Action: Run setup_custom_fields() to initialize project fields"
    return 1  # ✅ Returns failure!
  fi
  
  log "✅ Found Stage field: $field_id"
  
  local option_id=$(get_field_option_id "$project_id" "$field_id" "$stage")
  
  if [[ -z "$option_id" ]] || [[ "$option_id" == "null" ]]; then
    log "❌ FAILED: Stage option '$stage' not found in field"
    log "💡 Available options: Pending, Rex (Implementation), Cleo (Quality), etc."
    return 1  # ✅ Returns failure!
  fi
  
  log "✅ Found option ID for '$stage': $option_id"
  
  if update_project_item_field "$project_id" "$item_id" "$field_id" "$option_id"; then
    log "✅ Successfully updated stage to: $stage"
    return 0
  else
    log "❌ Failed to update project item field"
    return 1
  fi
}
```

**Key improvements:**
- Returns `1` (failure) when lookups fail
- Comprehensive logging at each step
- Actionable error messages
- Verifies GraphQL responses

### 2. Updated Custom Fields Setup

```bash
setup_custom_fields() {
  local project_id="$1"
  
  # Create "Stage" field - agent-based workflow stages that map to board columns
  create_single_select_field "$project_id" "Stage" \
    "Pending" \
    "Rex (Implementation)" \
    "Cleo (Quality)" \
    "Cipher (Security)" \
    "Tess (QA)" \
    "Atlas (Integration)" \
    "Bolt (Deployment)" \
    "Complete ✅"
  
  # Create "Task ID" field (text)
  create_text_field "$project_id" "Task ID" || true
  
  # Create "Priority" field for filtering
  create_single_select_field "$project_id" "Priority" \
    "High" "Medium" "Low" || true
}
```

**Changes:**
- ❌ Removed: "Current Agent" field (redundant)
- ✅ Updated: "Stage" field now contains agent names
- ✅ Added: "Priority" field for filtering

### 3. Unified Status and Agent Mapping

```bash
map_workflow_to_status() {
  # Status and agent are now the same thing!
  local stage="$1"
  local phase="$2"
  
  map_stage_to_agent "$stage" "$phase"
}

map_stage_to_agent() {
  local stage="$1"
  local phase="$2"
  
  # Handle completed workflows first
  if [[ "$phase" == "Succeeded" ]]; then
    echo "Complete ✅"
    return 0
  fi
  
  case "$stage" in
    "pending"|"waiting-pr-created")
      echo "Rex (Implementation)"
      ;;
    "implementation"|"implementation-in-progress")
      echo "Rex (Implementation)"
      ;;
    "quality-in-progress"|"waiting-ready-for-qa")
      echo "Cleo (Quality)"
      ;;
    # ... more agents ...
    *)
      echo "Pending"  # Default to Pending for unknown stages
      ;;
  esac
}
```

### 4. Updated Event Handler

```bash
handle_task_event() {
  # ... get task details ...
  
  local current_agent=$(get_actual_running_agent "$task_id" "$workflow_name")
  current_agent="${current_agent:-$(map_stage_to_agent "$current_stage" "$workflow_phase")}"
  
  # CRITICAL: ALWAYS update project stage (agent column)
  set_project_item_stage "$project_id" "$item_id" "$current_agent" || \
    log "⚠️  Could not update project stage for item $item_id to $current_agent"
  
  # ... rest of event handling ...
}
```

**Changes:**
- ❌ Removed: Separate `set_project_item_status()` and `set_project_item_agent()` calls
- ✅ Unified: Single `set_project_item_stage()` call with agent name
- ✅ Better logging: Shows exactly what stage/agent update failed

### 5. Enhanced GraphQL Error Handling

```bash
update_project_item_field() {
  local project_id="$1"
  local item_id="$2"
  local field_id="$3"
  local value="$4"
  
  local result=$(gh api graphql \
    -f query="$mutation" \
    -f projectId="$project_id" \
    -f itemId="$item_id" \
    -f fieldId="$field_id" \
    -f value="$value" 2>&1)
  
  # Check for errors
  if echo "$result" | jq -e '.errors' >/dev/null 2>&1; then
    log "❌ GraphQL error updating project item field:"
    echo "$result" | jq '.errors' >&2
    return 1
  fi
  
  # Verify update succeeded
  local updated_item_id=$(echo "$result" | jq -r '.data.updateProjectV2ItemFieldValue.projectV2Item.id // ""')
  
  if [[ -z "$updated_item_id" ]] || [[ "$updated_item_id" == "null" ]]; then
    log "⚠️  Update returned but no item ID confirmed"
    return 1
  fi
  
  return 0
}
```

**Improvements:**
- Captures and logs GraphQL errors
- Verifies response contains expected data
- Returns proper exit codes

---

## Board View Configuration

```bash
create_default_board_view() {
  # ... create board view ...
  
  echo "💡 To group by agents in the UI:" >&2
  echo "   1. Open the project board" >&2
  echo "   2. Click 'Group by' → Select 'Stage'" >&2
  echo "   3. Columns will show: Pending | Rex | Cleo | Cipher | Tess | Atlas | Bolt | Complete ✅" >&2
}
```

**Note:** GitHub Projects V2 API doesn't support setting `groupByFieldId` during board creation via GraphQL. Users need to manually configure grouping by "Stage" field in the UI on first visit.

---

## What This Fixes

### Before:
```
❌ Silent failures - no errors logged
❌ Project fields never updated
❌ Generic columns (To Do, In Progress, etc.)
❌ No agent visibility in board view
❌ Required separate "Current Agent" field
```

### After:
```
✅ Loud failures with actionable error messages
✅ Comprehensive logging at each step
✅ Agent-based columns (Rex, Cleo, Tess, etc.)
✅ Immediate visual clarity of workflow
✅ Single "Stage" field unifies agent and status
✅ Proper error handling and return codes
✅ GraphQL error detection and logging
```

---

## Additional UX Improvements

### Link PRs to Project

PRs are now automatically linked to the GitHub Project when detected, appearing in the project board alongside their related issues:

```bash
link_pr_to_project() {
  # Get PR node ID
  local pr_node_id=$(gh pr view "$pr_number" --json id --jq '.id')
  
  # Check if already linked (idempotent)
  if [[ "$pr_project_items" -gt 0 ]]; then
    return 0
  fi
  
  # Add PR to project
  add_issue_to_project "$project_id" "$pr_node_id" "$pr_number"
  
  # Set PR stage to match task stage
  set_project_item_stage "$project_id" "$pr_item_id" "$current_stage"
}
```

**Benefits:**
- ✅ PRs appear in project board's right-side "Projects" field
- ✅ PRs move through agent columns as work progresses
- ✅ Both issue and PR tracked in same board for complete visibility
- ✅ Idempotent - safe to call multiple times

---

## Testing the Fix

### 1. Deploy the Changes

```bash
# Create feature branch
git checkout main
git pull origin main
git checkout -b fix/morgan-agent-columns

# Commit changes
git add infra/charts/controller/agent-templates/pm/
git commit -m "fix(morgan): agent-based columns and proper error handling

- Replace generic status columns with agent-based workflow columns
- Fix silent failures in set_project_item_status/agent functions
- Add comprehensive error logging and proper return codes
- Unify status and agent into single Stage field
- Improve GraphQL error handling and response validation

Closes #XXX"

# Push and create PR
git push -u origin fix/morgan-agent-columns
gh pr create --title "Fix Morgan GitHub Projects sync with agent-based columns"
```

### 2. Verify After Merge

```bash
# Wait for ArgoCD to sync
argocd app sync controller

# Delete Morgan PM ConfigMap to force reload
kubectl delete configmap -n agent-platform agent-templates-pm

# Wait for ConfigMap recreation
kubectl wait --for=condition=available -n agent-platform deployment controller --timeout=300s

# Trigger new play workflow
task-master-play --task-id 1

# Watch Morgan PM logs
kubectl logs -n agent-platform -l agent=morgan -f
```

### 3. Expected Log Output (Cluster State Detection)

```log
# Initial setup
✅ Project ID: PVT_kwDOC8B7k84BHVJA
🔧 Setting up custom fields...
✅ Created field 'Stage' (ID: PVTF_...)
✅ Created field 'Task ID' (ID: PVTF_...)
✅ Created field 'Priority' (ID: PVTF_...)
✅ Custom fields setup completed

🔗 Linking issue #123 to project PVT_kwDOC8B7k84BHVJA...
✅ Verified: Issue #123 linked to project (1 project links total)
✅ Added issue #123 to project (Item ID: PVTI_...)

🔍 Mapping TaskMaster status 'pending' → GitHub Stage (Agent) 'Pending'
🔍 Setting project item stage: item=PVTI_..., stage='Pending'
✅ Found Stage field: PVTF_...
✅ Found option ID for 'Pending': PVTFO_...
🔄 Updating project item field...
✅ Successfully updated stage to: Pending

📡 Workflow event: task-1 (MODIFIED) - stage=implementation-in-progress, phase=Running
🔍 Event details: workflow=play-task-1-abc123, namespace=agent-platform

# Real cluster state detection in action:
🔍 Detecting actual running agent for task-1 from cluster...
✅ Found RUNNING pod with agent: rex
✅ Using actual running agent from cluster: Rex (Implementation)

🔄 State change detected for task-1: Rex (Implementation) | Rex (Implementation)
🔍 Setting project item stage: item=PVTI_..., stage='Rex (Implementation)'
✅ Found Stage field: PVTF_...
✅ Found option ID for 'Rex (Implementation)': PVTFO_...
🔄 Updating project item field...
✅ Successfully updated stage to: Rex (Implementation)

# When task moves to Cleo:
📡 Workflow event: task-1 (MODIFIED) - stage=quality-in-progress, phase=Running
🔍 Detecting actual running agent for task-1 from cluster...
✅ Found RUNNING pod with agent: cleo
✅ Using actual running agent from cluster: Cleo (Quality)
🔄 State change detected for task-1: Cleo (Quality) | Cleo (Quality)
✅ Successfully updated stage to: Cleo (Quality)

# Fallback when no running agent detected:
📡 Workflow event: task-2 (MODIFIED) - stage=pending, phase=Pending
🔍 Detecting actual running agent for task-2 from cluster...
⚠️  Could not detect actual running agent from cluster
⚠️  No running agent detected, using stage mapping: Pending
```

**What to Look For:**
- ✅ "Found RUNNING pod with agent" = Real cluster state detected
- ✅ "Using actual running agent from cluster" = Accurate tracking
- ⚠️ "Using stage mapping" = Fallback (should be rare)

### 4. Verify in GitHub UI

1. Navigate to repository Projects tab
2. Open the "Agent Workflow Board"
3. Manually set grouping: Click "Group by" → Select "Stage"
4. Verify columns appear:
   - **Pending** | **Rex (Implementation)** | **Cleo (Quality)** | **Cipher (Security)** | **Tess (QA)** | **Atlas (Integration)** | **Bolt (Deployment)** | **Complete ✅**
5. Watch tasks move between agent columns as workflows progress

---

## Related Issues

- #XXX - Morgan not updating GitHub Projects status
- #YYY - Silent failures in GitHub Projects integration
- #ZZZ - Need better visibility into multi-agent workflow

## Related Documentation

- [morgan-pm-status-tracking-investigation.md](./morgan-pm-status-tracking-investigation.md)
- [morgan-pm-issue-linking-investigation.md](./morgan-pm-issue-linking-investigation.md)
- [morgan-pm-github-projects-integration.md](./morgan-pm-github-projects-integration.md)

---

**Status:** ✅ Fix Complete - Ready for Testing  
**Author:** Claude (AI Assistant)  
**Review:** Pending

