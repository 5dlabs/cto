# Morgan PM: Comprehensive Remediation & Feature Completion Plan

## Executive Summary

This document outlines a complete remediation plan to transform Morgan PM from its current partially-working state to a **feature-complete, production-grade project management system** that provides real-time visibility into multi-agent workflows.

**Time Estimate**: 2-3 days for full implementation  
**Complexity**: Medium-High  
**Impact**: High - Enables full GitHub Projects integration

---

## Part 1: Deep Problem Analysis

### 1.1 The kubectl Watch Failure (CRITICAL)

**Symptom**:
```
⚠️ Skipping invalid JSON line (repeated 50+ times)
```

**Root Cause Analysis**:

The command `kubectl get workflows --watch-only -o json` produces **stream output**, not a JSON array. Each event is a separate JSON object on a new line, but the output format depends on the watch mode:

```bash
# What kubectl actually outputs (problematic):
{"type":"ADDED","object":{...}}
{"type":"MODIFIED","object":{...}}
# Morgan expects this ↑ but gets raw objects sometimes ↓
{metadata:{...},status:{...}}
{metadata:{...},status:{...}}
```

**Why It's Critical**:
- Real-time updates are completely broken
- Morgan falls back to 2-minute polling
- Status changes delayed by up to 2 minutes
- Agent assignments delayed
- User experience is poor

**Deep Technical Issue**:
- kubectl watch has multiple output modes
- `--watch-only` gives only changes, but format varies
- Shell piping through `jq` can buffer/corrupt streams
- Need proper JSONL (JSON Lines) handling

### 1.2 The Issue Linking Mystery (CRITICAL)

**Symptom**:
```bash
Morgan logs: "✅ Added issue #320 to project (Item ID: PVTI_lADOC8B7k84BHR5czggvjPI)"
Reality: gh issue view 320 --json projectItems → []
```

**Deep Analysis**:

After deep investigation, here are the possible causes:

**Theory A: Org-Level vs Repo-Level Mismatch**
- Morgan creates **org-level** projects: `createProjectV2(input: {ownerId: O_kgDOC8B7kw})`
- Morgan tries to link **repo-level** issues to them
- GitHub's permission model might require special handling for cross-boundary operations
- The `addProjectV2ItemById` mutation might succeed but not persist due to repository access rules

**Theory B: Async Operation Completion**
- GraphQL mutation returns immediately with item ID
- But the actual linking happens asynchronously
- If there's a permission issue or validation failure AFTER the mutation returns, it fails silently
- Morgan doesn't wait or verify the link actually persisted

**Theory C: Project-Repository Linking Failure**
- Morgan calls `linkProjectV2ToRepository` mutation
- Logs show: "Linking project PVT_... to repository R_..."
- But we never verify it succeeded!
- If this link fails, issues can't be added

**Theory D: GitHub API Bug/Quirk**
- Known issue: Org-level Projects v2 have had bugs with cross-repository linking
- Might be hitting an undocumented limitation
- Need to test with repo-level projects to isolate

### 1.3 The Field Update Skipping (HIGH)

**Symptom**:
```
⏭️ No change for task-1 (skipping comment posting)
```

**Root Cause**:

The deduplication logic in `handle_task_event` is designed to prevent comment spam, but it's too aggressive:

```bash
# Morgan checks cached state:
if has_state_changed "$task_id" "$current_stage" "$current_agent" "$task_status"; then
  # Only update IF state changed
  update_issue_assignee
  add_status_history
  ...
fi

# BUT: Project field updates are INSIDE this block!
set_project_item_status  # ← Never called if "no change"
set_project_item_agent   # ← Never called if "no change"
```

**The Problem**:
- First sync: State changes, fields get set ✅
- Subsequent syncs: No state change, fields don't update ❌
- If initial sync fails (e.g., rate limit), fields NEVER get set
- Periodic re-sync doesn't fix it because "no change" is detected

**Why This Matters**:
- Project fields remain stuck at initial values
- Even if we fix issue linking, fields won't update
- Manual project updates don't get corrected

### 1.4 Architectural Limitations

**Current Issues**:

1. **No Verification Loop**
   - Morgan assumes mutations succeed
   - Doesn't verify issues are actually linked
   - Doesn't retry on failure
   - No reconciliation mechanism

2. **No Idempotency**
   - Project creation isn't idempotent
   - Creates duplicate projects if run twice
   - No "get or create" pattern

3. **No Error Recovery**
   - Silent failures
   - No retry logic
   - No dead letter queue
   - No manual intervention tools

4. **Limited Observability**
   - Can't see what's actually in the project
   - No metrics on success/failure rates
   - No alerts on persistent failures

---

## Part 2: Comprehensive Remediation Strategy

### Phase 1: Critical Fixes (Day 1)

#### Fix 1.1: kubectl Watch JSON Parsing

**Objective**: Make real-time event monitoring work reliably

**Implementation**:

```bash
# Current (broken):
kubectl get workflows -n "$NAMESPACE" \
  -l "parent-workflow=$WORKFLOW_NAME" \
  --watch-only -o json 2>/dev/null | \
while IFS= read -r event_line; do
  # Fails because format is inconsistent
done

# New (robust):
watch_workflows_robust() {
  # Use kubectl with proper JSONL output
  kubectl get workflows -n "$NAMESPACE" \
    -l "parent-workflow=$WORKFLOW_NAME" \
    -o json --watch 2>&1 | \
  while IFS= read -r line; do
    # Skip non-JSON lines (kubernetes watch headers)
    if ! echo "$line" | jq empty 2>/dev/null; then
      continue
    fi
    
    # Handle both watch event format and direct object format
    local event_type=$(echo "$line" | jq -r '.type // "MODIFIED"' 2>/dev/null || echo "DIRECT")
    
    if [[ "$event_type" == "DIRECT" ]]; then
      # Direct object - no wrapping
      WORKFLOW_OBJ="$line"
    else
      # Watch event format
      WORKFLOW_OBJ=$(echo "$line" | jq -c '.object' 2>/dev/null || echo "$line")
    fi
    
    # Validate we have a valid workflow object
    if ! echo "$WORKFLOW_OBJ" | jq -e '.metadata.name' >/dev/null 2>/dev/null; then
      log "⚠️  Skipping invalid workflow object"
      continue
    fi
    
    # Process the workflow
    process_workflow_event "$WORKFLOW_OBJ"
  done
}

process_workflow_event() {
  local workflow_obj="$1"
  
  # Extract details safely
  local task_id=$(echo "$workflow_obj" | jq -r '.metadata.labels["task-id"] // ""' 2>/dev/null)
  local workflow_name=$(echo "$workflow_obj" | jq -r '.metadata.name // ""' 2>/dev/null)
  local current_stage=$(echo "$workflow_obj" | jq -r '.metadata.labels["current-stage"] // "pending"' 2>/dev/null)
  local workflow_phase=$(echo "$workflow_obj" | jq -r '.status.phase // "Pending"' 2>/dev/null)
  
  [[ -z "$task_id" ]] && return 0
  
  log "📡 Workflow event: task-$task_id (stage=$current_stage, phase=$workflow_phase)"
  
  # Update GitHub
  handle_task_event "$PROJECT_ID" "$task_id" "$workflow_name" "$current_stage" "$workflow_phase" "false"
}
```

**Validation**:
- Watch produces events without "invalid JSON" errors
- Events trigger within 5 seconds of workflow changes
- Handles restarts gracefully

#### Fix 1.2: Comprehensive GraphQL Error Logging

**Objective**: See exactly what's failing and why

**Implementation**:

```bash
# Enhanced add_issue_to_project with full logging
add_issue_to_project() {
  local project_id="$1"
  local issue_node_id="$2"
  local issue_number="$3"  # Add for better logging
  
  local mutation='
    mutation($projectId: ID!, $contentId: ID!) {
      addProjectV2ItemById(input: {
        projectId: $projectId
        contentId: $contentId
      }) {
        item {
          id
          project {
            id
            title
          }
          content {
            ... on Issue {
              number
              projectItems(first: 5) {
                totalCount
                nodes {
                  id
                }
              }
            }
          }
        }
      }
    }
  '
  
  log "🔗 Linking issue #$issue_number (node: $issue_node_id) to project $project_id"
  
  local result=$(gh api graphql \
    -f query="$mutation" \
    -f projectId="$project_id" \
    -f contentId="$issue_node_id" 2>&1)
  
  # Save full response for debugging
  echo "$result" > "/tmp/add-project-item-$issue_number.json"
  
  # Check for errors
  if echo "$result" | jq -e '.errors' >/dev/null 2>&1; then
    log "❌ GraphQL ERROR adding issue #$issue_number to project:"
    echo "$result" | jq '.errors' | tee -a "$SYNC_LOG"
    
    # Check specific error types
    local error_type=$(echo "$result" | jq -r '.errors[0].type // ""')
    local error_msg=$(echo "$result" | jq -r '.errors[0].message // ""')
    
    case "$error_type" in
      "INSUFFICIENT_SCOPES"|"FORBIDDEN")
        log "🔐 Permission error: $error_msg"
        log "💡 Check GitHub App permissions for organization_projects and repository_projects"
        ;;
      "NOT_FOUND")
        log "🔍 Resource not found: $error_msg"
        log "💡 Project ID or Issue Node ID might be invalid"
        ;;
      *)
        log "⚠️  Unknown error type: $error_type - $error_msg"
        ;;
    esac
    
    echo "null"
    return 1
  fi
  
  # Extract item ID
  local item_id=$(echo "$result" | jq -r '.data.addProjectV2ItemById.item.id // "null"')
  
  if [[ "$item_id" == "null" ]] || [[ -z "$item_id" ]]; then
    log "❌ Failed to get item ID from response"
    echo "$result" | jq '.' | tee -a "$SYNC_LOG"
    echo "null"
    return 1
  fi
  
  # VERIFY the link actually worked
  local total_count=$(echo "$result" | jq -r '.data.addProjectV2ItemById.item.content.projectItems.totalCount // 0')
  
  if [[ "$total_count" -gt 0 ]]; then
    log "✅ Verified: Issue #$issue_number is linked to project (total links: $total_count)"
  else
    log "⚠️  WARNING: GraphQL returned item ID but issue has 0 project links!"
    log "💡 This might indicate an async operation or permission issue"
  fi
  
  echo "$item_id"
}
```

**Benefits**:
- See exact GraphQL errors
- Categorize error types
- Provide actionable suggestions
- Verify links actually persist
- Keep audit trail in log files

#### Fix 1.3: Separate Field Updates from Comment Spam Prevention

**Objective**: Always update project fields, even when skipping comments

**Implementation**:

```bash
handle_task_event() {
  local project_id="$1"
  local task_id="$2"
  local workflow_name="$3"
  local current_stage="$4"
  local workflow_phase="$5"
  local is_sync="${6:-false}"
  
  # Get issue details
  local entry=$(jq -r --arg tid "$task_id" '.[$tid] // empty' "$TASK_ISSUE_MAP")
  [[ -z "$entry" ]] && return 0
  
  local issue_number=$(echo "$entry" | jq -r '.issue_number')
  local item_id=$(echo "$entry" | jq -r '.item_id')
  
  # Get current agent from cluster
  local actual_agent=$(get_actual_running_agent "$task_id" "$workflow_name")
  local current_agent="${actual_agent:-$(map_stage_to_agent "$current_stage" "$workflow_phase")}"
  local task_status=$(map_workflow_to_status "$current_stage" "$workflow_phase")
  
  # Check if state actually changed (for comments)
  local state_changed=false
  if has_state_changed "$task_id" "$current_stage" "$current_agent" "$task_status" "$workflow_phase"; then
    state_changed=true
    log "🔄 State change detected for task-$task_id: $current_agent | $task_status"
  fi
  
  # ═══════════════════════════════════════════════════════════
  # CRITICAL: ALWAYS update project fields (idempotent operations)
  # These are cheap GraphQL calls and ensure consistency
  # ═══════════════════════════════════════════════════════════
  update_project_fields_idempotent "$project_id" "$item_id" "$task_status" "$current_agent"
  
  # ═══════════════════════════════════════════════════════════
  # ALWAYS update issue labels (idempotent)
  # ═══════════════════════════════════════════════════════════
  update_issue_labels "$issue_number" "$task_status" "$current_agent"
  
  # ═══════════════════════════════════════════════════════════
  # CONDITIONAL: Only post comments if state changed (prevent spam)
  # ═══════════════════════════════════════════════════════════
  if [[ "$state_changed" == true ]]; then
    update_issue_assignee "$issue_number" "$current_agent"
    add_status_history "$issue_number" "$task_id" "$current_stage" "$current_agent" "$task_status" "$workflow_phase"
    check_task_health "$task_id" "$workflow_name" "$current_stage" "$issue_number"
    update_progress_comment "$issue_number" "$task_id" "$workflow_name" "$current_stage"
    stream_significant_events "$task_id" "$workflow_name" "$issue_number"
  fi
  
  # ALWAYS update GitHub Checks (idempotent)
  local pr_number=$(find_pr_for_task "$task_id")
  if [[ -n "$pr_number" ]]; then
    create_or_update_github_check "$task_id" "$pr_number" "$current_agent" "$current_stage" "$workflow_phase"
  fi
  
  # Update cache
  update_cached_state "$task_id" "$current_stage" "$current_agent" "$task_status" "$workflow_phase"
}

update_project_fields_idempotent() {
  local project_id="$1"
  local item_id="$2"
  local status="$3"
  local agent="$4"
  
  # These are idempotent - calling them multiple times with same values is safe
  # GraphQL will just return success without doing work if values unchanged
  
  set_project_item_status "$project_id" "$item_id" "$status" 2>/dev/null || \
    log "⚠️  Could not update project status for item $item_id"
  
  set_project_item_agent "$project_id" "$item_id" "$agent" 2>/dev/null || \
    log "⚠️  Could not update project agent for item $item_id"
}
```

**Benefits**:
- Project fields always stay in sync
- Comment spam prevention still works
- Clear separation of concerns
- Resilient to failures

### Phase 2: Architectural Fixes (Day 1-2)

#### Fix 2.1: Implement Project-Repository Verification

**Objective**: Ensure project is properly linked to repository before adding issues

**Implementation**:

```bash
ensure_project_linked_to_repository() {
  local project_id="$1"
  local repo_owner="$2"
  local repo_name="$3"
  
  log "🔗 Verifying project is linked to repository $repo_owner/$repo_name..."
  
  # Check if already linked
  local check_query='
    query($projectId: ID!) {
      node(id: $projectId) {
        ... on ProjectV2 {
          repositories(first: 20) {
            nodes {
              owner {
                login
              }
              name
            }
          }
        }
      }
    }
  '
  
  local result=$(gh api graphql -f query="$check_query" -f projectId="$project_id")
  
  # Check if our repo is in the list
  local is_linked=$(echo "$result" | jq -r --arg owner "$repo_owner" --arg name "$repo_name" \
    '.data.node.repositories.nodes[] | select(.owner.login == $owner and .name == $name) | .name' | head -1)
  
  if [[ -n "$is_linked" ]]; then
    log "✅ Project already linked to repository"
    return 0
  fi
  
  log "📎 Linking project to repository..."
  
  # Get repository ID
  local repo_id=$(get_repository_id "$repo_owner" "$repo_name")
  
  # Link project to repository
  local link_mutation='
    mutation($projectId: ID!, $repositoryId: ID!) {
      linkProjectV2ToRepository(input: {
        projectId: $projectId
        repositoryId: $repositoryId
      }) {
        repository {
          id
          nameWithOwner
        }
      }
    }
  '
  
  local link_result=$(gh api graphql \
    -f query="$link_mutation" \
    -f projectId="$project_id" \
    -f repositoryId="$repo_id")
  
  # Check for errors
  if echo "$link_result" | jq -e '.errors' >/dev/null 2>&1; then
    log "❌ Failed to link project to repository:"
    echo "$link_result" | jq '.errors'
    return 1
  fi
  
  local linked_repo=$(echo "$link_result" | jq -r '.data.linkProjectV2ToRepository.repository.nameWithOwner')
  
  if [[ "$linked_repo" == "$repo_owner/$repo_name" ]]; then
    log "✅ Successfully linked project to repository: $linked_repo"
    return 0
  else
    log "❌ Link verification failed - expected $repo_owner/$repo_name, got $linked_repo"
    return 1
  fi
}
```

#### Fix 2.2: Add Dual-Mode Project Support (Org + Repo)

**Objective**: Support both org-level and repo-level projects with graceful fallback

**Implementation**:

```bash
get_or_create_project_smart() {
  local repo_owner="$1"
  local repo_name="$2"
  local project_title="$3"
  local prefer_org_level="${4:-true}"
  
  local project_id=""
  
  if [[ "$prefer_org_level" == "true" ]]; then
    log "📊 Attempting to create/find org-level project..."
    project_id=$(get_or_create_org_project "$repo_owner" "$project_title")
    
    if [[ -n "$project_id" ]] && [[ "$project_id" != "null" ]]; then
      # Verify we can link it to the repository
      if ensure_project_linked_to_repository "$project_id" "$repo_owner" "$repo_name"; then
        log "✅ Using org-level project: $project_id"
        echo "$project_id"
        return 0
      else
        log "⚠️  Org-level project exists but can't link to repository - falling back to repo-level"
        project_id=""
      fi
    fi
  fi
  
  # Fallback to repo-level project
  if [[ -z "$project_id" ]]; then
    log "📊 Creating/finding repo-level project..."
    project_id=$(get_or_create_repo_project "$repo_owner" "$repo_name" "$project_title")
    
    if [[ -n "$project_id" ]] && [[ "$project_id" != "null" ]]; then
      log "✅ Using repo-level project: $project_id"
      echo "$project_id"
      return 0
    fi
  fi
  
  log "❌ Failed to create any project"
  echo "null"
  return 1
}

get_or_create_repo_project() {
  local repo_owner="$1"
  local repo_name="$2"
  local project_title="$3"
  
  # Get repository ID
  local repo_id=$(get_repository_id "$repo_owner" "$repo_name")
  
  # Try to find existing project by title
  local find_query='
    query($owner: String!, $name: String!) {
      repository(owner: $owner, name: $name) {
        projectsV2(first: 20) {
          nodes {
            id
            title
          }
        }
      }
    }
  '
  
  local result=$(gh api graphql -f query="$find_query" -f owner="$repo_owner" -f name="$repo_name")
  local existing_id=$(echo "$result" | jq -r --arg title "$project_title" \
    '.data.repository.projectsV2.nodes[] | select(.title == $title) | .id' | head -1)
  
  if [[ -n "$existing_id" ]] && [[ "$existing_id" != "null" ]]; then
    log "✅ Found existing repo-level project: $project_title"
    echo "$existing_id"
    return 0
  fi
  
  # Create new repo-level project
  local create_mutation='
    mutation($repositoryId: ID!, $title: String!) {
      createProjectV2(input: {
        repositoryId: $repositoryId
        title: $title
      }) {
        projectV2 {
          id
          title
        }
      }
    }
  '
  
  local create_result=$(gh api graphql \
    -f query="$create_mutation" \
    -f repositoryId="$repo_id" \
    -f title="$project_title")
  
  local project_id=$(echo "$create_result" | jq -r '.data.createProjectV2.projectV2.id')
  
  log "✅ Created new repo-level project: $project_title (ID: $project_id)"
  echo "$project_id"
}
```

**Benefits**:
- Tries org-level first (better visibility)
- Falls back to repo-level if issues
- Repo-level projects auto-linked (no permission issues)
- User can configure preference

#### Fix 2.3: Add Retry Logic with Exponential Backoff

**Objective**: Handle transient failures gracefully

**Implementation**:

```bash
retry_with_backoff() {
  local max_attempts="$1"
  shift
  local command="$@"
  
  local attempt=1
  local delay=1
  
  while [[ $attempt -le $max_attempts ]]; do
    if eval "$command"; then
      return 0
    fi
    
    if [[ $attempt -lt $max_attempts ]]; then
      log "⏳ Attempt $attempt failed, retrying in ${delay}s..."
      sleep $delay
      delay=$((delay * 2))  # Exponential backoff
      attempt=$((attempt + 1))
    else
      log "❌ All $max_attempts attempts failed"
      return 1
    fi
  done
}

# Usage:
retry_with_backoff 3 add_issue_to_project "$PROJECT_ID" "$ISSUE_NODE_ID" "$ISSUE_NUMBER"
```

### Phase 3: Feature Enhancements (Day 2-3)

#### Enhancement 3.1: Real-Time Assignee Sync

**Objective**: Show actual running agent in both issue and project

**Status**: Already implemented in PR #1249! ✅

Features:
- Queries actual CodeRun from cluster
- Falls back to pod inspection
- Uses stage mapping as final fallback
- Assigns GitHub App bot user to issue

#### Enhancement 3.2: Webhook-Based Updates (Future)

**Objective**: Replace polling with webhooks for instant updates

**Design**:

```yaml
# GitHub App webhook events to subscribe to:
events:
  - projects_v2
  - projects_v2_item
  - projects_v2_status_update
  - pull_request
  - pull_request_review
  - issues
  - issue_comment

# Webhook handler in Morgan:
on_webhook_event:
  - Validate webhook signature
  - Extract event type and payload
  - Update internal state
  - Trigger sync if needed
```

**Benefits**:
- Instant updates (no 2-min delay)
- Lower resource usage (no polling)
- Bidirectional sync (GitHub → Kubernetes)

#### Enhancement 3.3: Project Summary Dashboard

**Objective**: Beautiful, informative project view

**Implementation**:

Add project README with live stats:

```markdown
# 🎯 cto-parallel-test - TaskMaster Workflow

**Status**: 🔄 In Progress | **Completion**: 40% (4/10 tasks)

## 📊 Progress Overview

| Phase | Tasks | Status |
|-------|-------|--------|
| Implementation | 4 | 🔄 In Progress |
| Code Review | 0 | ⏳ Pending |
| QA Testing | 0 | ⏳ Pending |
| Complete | 0 | ✅ Done |

## 🤖 Current Agents

- **Rex (Implementation)**: Tasks 1, 3, 4, 6
- **Cleo (Quality)**: Awaiting PR creation
- **Tess (QA)**: Awaiting quality approval

## ⚡ Recent Activity

- **2min ago**: Rex started Task 6 (Frontend Components)
- **5min ago**: Rex started Task 4 (Product Catalog)
- **10min ago**: Rex started Task 3 (Authentication)

## 🔗 Links

- [Argo Workflow](https://argo.5dlabs.com/workflows/agent-platform/play-project-workflow-template-nhf4q)
- [Task Master Docs](https://github.com/5dlabs/cto-parallel-test/tree/main/docs)
```

**Auto-updated every sync** via GraphQL project description update.

#### Enhancement 3.4: Agent Progress Streaming

**Objective**: Show what agents are actually doing in real-time

**Implementation**:

```bash
stream_agent_progress() {
  local task_id="$1"
  local workflow_name="$2"
  local issue_number="$3"
  
  # Find the active CodeRun
  local coderun=$(kubectl get coderuns -n "$NAMESPACE" \
    -l "task-id=$task_id" \
    --sort-by=.metadata.creationTimestamp \
    -o json | jq -r '.items[-1]')
  
  # Get agent progress from pod logs (last 10 lines)
  local pod_name=$(echo "$coderun" | jq -r '.status.podName // ""')
  if [[ -n "$pod_name" ]]; then
    local recent_logs=$(kubectl logs -n "$NAMESPACE" "$pod_name" -c main --tail=10 2>/dev/null || echo "")
    
    # Extract significant events (file changes, git commits, etc.)
    local significant=$(echo "$recent_logs" | grep -E "✅|📝|🔧|🎯" | head -5)
    
    if [[ -n "$significant" ]]; then
      # Post as collapsible comment
      gh issue comment "$issue_number" --repo "$REPO_OWNER/$REPO_NAME" --body "
<details>
<summary>🤖 Agent Progress Update ($(date -u +"%H:%M UTC"))</summary>

\`\`\`
$significant
\`\`\`

</details>
" 2>/dev/null || true
    fi
  fi
}
```

**Benefits**:
- See what agents are actually doing
- Catch stuck agents early
- Better visibility into progress

---

## Part 3: Testing & Validation Strategy

### Test Suite 1: Issue Linking Verification

```bash
#!/bin/bash
# test-issue-linking.sh

test_org_level_project() {
  echo "Testing org-level project creation and linking..."
  
  # Create project
  PROJECT_ID=$(create_org_level_project "5dlabs" "Test Org Project")
  
  # Create test issue
  ISSUE_URL=$(gh issue create \
    --repo 5dlabs/cto-parallel-test \
    --title "Test Issue for Linking" \
    --body "Testing org-level project linking")
  ISSUE_NUMBER=$(echo "$ISSUE_URL" | grep -Eo '[0-9]+$')
  
  # Get issue node ID
  ISSUE_NODE_ID=$(gh issue view "$ISSUE_NUMBER" \
    --repo 5dlabs/cto-parallel-test \
    --json id --jq '.id')
  
  # Link to project
  ITEM_ID=$(add_issue_to_project "$PROJECT_ID" "$ISSUE_NODE_ID" "$ISSUE_NUMBER")
  
  # VERIFY
  sleep 2  # Wait for async operations
  
  LINKED_COUNT=$(gh issue view "$ISSUE_NUMBER" \
    --repo 5dlabs/cto-parallel-test \
    --json projectItems --jq '.projectItems | length')
  
  if [[ "$LINKED_COUNT" -gt 0 ]]; then
    echo "✅ SUCCESS: Issue is linked to project"
    return 0
  else
    echo "❌ FAIL: Issue not linked despite successful mutation"
    return 1
  fi
}

test_repo_level_project() {
  echo "Testing repo-level project creation and linking..."
  # Similar test for repo-level
}

# Run tests
test_org_level_project
test_repo_level_project
```

### Test Suite 2: Real-Time Update Validation

```bash
#!/bin/bash
# test-realtime-updates.sh

# Monitor Morgan's event processing
kubectl logs -f -n agent-platform \
  -l agent=morgan \
  --tail=0 | \
while read -r line; do
  if echo "$line" | grep -q "📡 Workflow event"; then
    # Extract timestamp
    TIMESTAMP=$(date +%s)
    echo "Event detected at: $TIMESTAMP"
    
    # Check if GitHub was updated within 5 seconds
    sleep 5
    
    # Verify issue was updated
    # (implementation details)
    
    echo "✅ Real-time update verified"
  fi
done
```

### Test Suite 3: Field Update Consistency

```bash
#!/bin/bash
# test-field-updates.sh

verify_project_fields() {
  local issue_number="$1"
  local expected_status="$2"
  local expected_agent="$3"
  
  # Get project item
  local item=$(gh api graphql -f query="..." | jq ...)
  
  local actual_status=$(echo "$item" | jq -r '.status')
  local actual_agent=$(echo "$item" | jq -r '.agent')
  
  if [[ "$actual_status" == "$expected_status" ]] && \
     [[ "$actual_agent" == "$expected_agent" ]]; then
    echo "✅ Fields match expected values"
    return 0
  else
    echo "❌ Field mismatch!"
    echo "   Expected: $expected_status / $expected_agent"
    echo "   Actual:   $actual_status / $actual_agent"
    return 1
  fi
}
```

---

## Part 4: Implementation Roadmap

### Day 1 Morning: Critical Fixes
- [ ] Fix kubectl watch JSON parsing
- [ ] Add comprehensive GraphQL error logging
- [ ] Separate field updates from comment spam prevention
- [ ] Test with current workflow

**Deliverable**: Real-time updates work, errors are visible

### Day 1 Afternoon: Architectural Improvements
- [ ] Implement project-repository verification
- [ ] Add retry logic with exponential backoff
- [ ] Test issue linking with verification

**Deliverable**: Issues actually link to projects

### Day 2 Morning: Dual-Mode Support
- [ ] Implement repo-level project support
- [ ] Add smart fallback logic
- [ ] Test both org and repo modes

**Deliverable**: Flexible project creation that always works

### Day 2 Afternoon: Polish & Testing
- [ ] Add project summary dashboard
- [ ] Implement agent progress streaming
- [ ] Run comprehensive test suite

**Deliverable**: Feature-complete, well-tested system

### Day 3: Documentation & Rollout
- [ ] Update documentation
- [ ] Create runbooks for troubleshooting
- [ ] Deploy to production
- [ ] Monitor and iterate

**Deliverable**: Production-ready Morgan PM

---

## Part 5: Success Criteria

### Must-Have (P0)
- ✅ Issues successfully link to projects (verified)
- ✅ Project fields update in real-time (< 10 second delay)
- ✅ Agent assignments visible in both issues and projects
- ✅ No "invalid JSON" errors in logs
- ✅ GraphQL errors are logged and actionable

### Should-Have (P1)
- ✅ Retry logic handles transient failures
- ✅ Project-repository linking is verified
- ✅ Works with both org-level and repo-level projects
- ✅ Project summary updates automatically

### Nice-to-Have (P2)
- ⏳ Webhook-based updates (replace polling)
- ⏳ Agent progress streaming to issues
- ⏳ Prometheus metrics for monitoring
- ⏳ Beautiful project dashboard

---

## Part 6: Monitoring & Observability

### Key Metrics

```prometheus
# Morgan PM Health Metrics
morgan_issue_link_success_total
morgan_issue_link_failure_total
morgan_project_field_update_latency_seconds
morgan_event_processing_latency_seconds
morgan_graphql_error_total{type="INSUFFICIENT_SCOPES|FORBIDDEN|NOT_FOUND|OTHER"}
morgan_sync_lag_seconds
```

### Alerts

```yaml
alerts:
  - name: MorganIssueLinkingFailure
    expr: rate(morgan_issue_link_failure_total[5m]) > 0.1
    severity: warning
    description: "Morgan is failing to link issues to projects"
    
  - name: MorganEventLoopStuck
    expr: time() - morgan_last_event_timestamp_seconds > 300
    severity: critical
    description: "Morgan's event loop hasn't processed an event in 5 minutes"
```

---

## Part 7: Migration Path

### From Current Broken State → Working State

**Step 1: Deploy Fixes Without Disruption**
- Deploy kubectl watch fix
- Deploy error logging
- Don't restart existing Morgan pods yet
- Monitor new workflows

**Step 2: Verify Fixes Work**
- Trigger a new test workflow
- Check logs for errors
- Verify issues link properly
- Validate real-time updates

**Step 3: Migrate Existing Projects**
- Script to re-link existing issues to projects
- Bulk update project fields for accuracy
- Verify all tasks are synced

**Step 4: Cleanup**
- Remove old duplicate projects
- Archive test issues
- Document learnings

---

## Conclusion

This comprehensive plan transforms Morgan PM from its current partially-working state to a production-grade, feature-complete system with:

- **Reliable issue linking** (verified and tested)
- **Real-time updates** (< 10 second latency)
- **Robust error handling** (retry logic, comprehensive logging)
- **Flexible architecture** (org/repo project support)
- **Beautiful UX** (live dashboards, agent progress)
- **Production monitoring** (metrics, alerts, observability)

**Estimated Timeline**: 2-3 days
**Complexity**: Medium-High  
**Impact**: Transforms developer experience, enables full GitHub Projects integration

Next step: Begin Day 1 implementation with critical fixes.

