# Pull Request to GitHub Project Automatic Linking

## Problem

Pull requests were not being automatically linked to GitHub Projects, making it difficult to track PR status alongside task issues in the project board.

**Previous Design:**
- Only Issues were added to the project
- PRs linked to issues via "Closes #XXX" in description
- Rationale: "Avoid duplication - one item per task"

**User Request:**
> "Can we make sure when the pull requests get created we link it to the project as well?"

## Solution

**New Design:**
- Both Issues AND PRs are added to the project
- Issues show task overview and tracking
- PRs show implementation details and code review status
- Comprehensive visibility in project board

## Implementation

### **File Modified:** `infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs`

**Location:** Lines 709-721

**Before:**
```bash
# ALWAYS update GitHub Checks API (PRs NOT added to project - linked via issue instead)
local pr_number=$(find_pr_for_task "$task_id")
if [[ -n "$pr_number" ]]; then
  # Create/update GitHub check
  create_or_update_github_check "$task_id" "$pr_number" "$current_agent" "$current_stage" "$workflow_phase"
  
  # NOTE: PRs are NOT added to project board to avoid duplication
  # PRs are linked to issues via "Closes #XXX" in PR description
  # This keeps one item per task in the project (the issue)
fi
```

**After:**
```bash
# ALWAYS update GitHub Checks API and link PRs to project
local pr_number=$(find_pr_for_task "$task_id")
if [[ -n "$pr_number" ]]; then
  # Create/update GitHub check
  create_or_update_github_check "$task_id" "$pr_number" "$current_agent" "$current_stage" "$workflow_phase"
  
  # Link PR to GitHub Project for visibility
  # PRs appear alongside issues in project board for comprehensive tracking
  # Issues show the task overview, PRs show the implementation details
  if ! link_pr_to_project "$project_id" "$pr_number" "$task_id" "$current_stage" 2>/dev/null; then
    log "⚠️  Could not link PR #$pr_number to project (may already be linked or permissions issue)"
  fi
fi
```

## How It Works

### **PR Detection**

Morgan PM monitors workflow progress and detects PRs using `find_pr_for_task()`:

```bash
find_pr_for_task() {
  local task_id="$1"
  
  # Find PR with task label
  local pr_number=$(gh pr list \
    --repo "$REPO_OWNER/$REPO_NAME" \
    --label "task-$task_id" \
    --state open \
    --json number \
    --jq '.[0].number' 2>/dev/null || echo "")
  
  echo "$pr_number"
}
```

### **PR Linking**

When a PR is detected, `link_pr_to_project()` is called:

```bash
link_pr_to_project() {
  local project_id="$1"
  local pr_number="$2"
  local task_id="$3"
  local current_stage="$4"
  
  # Get PR node ID
  local pr_node_id=$(gh pr view "$pr_number" \
    --repo "$REPO_OWNER/$REPO_NAME" \
    --json id --jq '.id' 2>/dev/null || echo "")
  
  # Check if PR already linked to THIS project
  local pr_in_this_project=$(gh pr view "$pr_number" \
    --repo "$REPO_OWNER/$REPO_NAME" \
    --json projectItems --jq --arg pid "$project_id" \
    '.projectItems[] | select(.project.id == $pid) | .id' 2>/dev/null || echo "")
  
  if [[ -n "$pr_in_this_project" ]]; then
    # Already linked - just update stage
    set_project_item_stage "$project_id" "$pr_in_this_project" "$current_stage"
    return 0
  fi
  
  # Add PR to project using GraphQL API
  local pr_item_id=""
  if ! pr_item_id=$(retry_with_backoff 3 "Link PR #$pr_number to project" \
    add_issue_to_project "$project_id" "$pr_node_id" "$pr_number"); then
    return 1
  fi
  
  # Set PR stage to match task stage
  set_project_item_stage "$project_id" "$pr_item_id" "$current_stage"
}
```

### **GraphQL Integration**

Uses the same robust `add_issue_to_project()` function that handles issue linking:

```graphql
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
    }
  }
}
```

**Features:**
- ✅ **Retry logic** - 3 attempts with exponential backoff
- ✅ **Duplicate prevention** - Checks if PR already linked before adding
- ✅ **Stage tracking** - Sets PR stage to match task stage
- ✅ **Error handling** - Graceful failure with logging
- ✅ **Permission checks** - Handles GitHub API permission errors

## Benefits

### **1. Comprehensive Project Visibility**

**Before:**
```
Project Board:
├── Issue #123: Task-1 Implementation
└── (PR #456 not visible - linked only via "Closes #123")
```

**After:**
```
Project Board:
├── Issue #123: Task-1 Implementation  [Stage: Rex (Implementation)]
└── PR #456: Implement feature X       [Stage: Rex (Implementation)]
```

### **2. Better Tracking**

- **Issues** show: Task description, acceptance criteria, comments, assignees
- **PRs** show: Code changes, review status, checks, deployments
- Both appear in same project board with synchronized stages

### **3. Workflow Integration**

Morgan PM automatically:
1. Creates project on workflow start
2. Creates issues for each task
3. Links issues to project
4. **NEW:** Detects PRs when created
5. **NEW:** Links PRs to project
6. Updates both issue and PR stages as workflow progresses
7. Keeps project board synchronized with workflow state

## Timing

**When PR Linking Happens:**

1. **PR Created** by Rex/Blaze/implementation agent
   - PR gets `task-N`, `service-X`, `run-Y` labels
   - Container script attempts basic linking (may fail if project doesn't exist yet)

2. **Morgan PM Monitoring Loop** (every 30 seconds)
   - Detects workflow progress updates
   - Calls `find_pr_for_task()` to check for PRs
   - **NEW:** Calls `link_pr_to_project()` when PR detected
   - Links PR to project with proper GraphQL API
   - Sets PR stage to match task stage

3. **Stage Updates** (throughout workflow)
   - Morgan monitors Rex → Cleo → Cipher → Tess progression
   - Updates both issue and PR stages in project
   - Both items move through project board columns together

## Error Handling

### **Scenarios Handled:**

**Project doesn't exist yet:**
```
⚠️  Could not link PR to project (may already be linked or permissions issue)
```
- Non-blocking error - retry on next monitoring loop
- Project created on first workflow run
- PR linked on subsequent monitoring cycles

**PR already linked:**
```
✅ PR #456 already linked to this project (item ID: abc123)
```
- Skips re-linking
- Updates stage if needed
- No duplicate entries

**Permission issues:**
```
🔐 Permission Error: Insufficient permissions for organization_projects
💡 Action: Check GitHub App permissions
```
- Clear error messages
- Actionable guidance
- Tracks error metrics

## Testing

### **Verify PR Linking**

1. **Start a play workflow:**
```bash
mcp_user-cto_play --task_id=1
```

2. **Wait for PR creation** (~5-10 minutes for Rex to complete)

3. **Check GitHub Project:**
   - Navigate to repository → Projects
   - Open "{service} - TaskMaster Workflow" project
   - Should see BOTH:
     - Issue #X: Task-1 description
     - PR #Y: Feature implementation

4. **Verify synchronization:**
   - Both items should be in same stage column
   - As workflow progresses (Rex → Cleo → Cipher → Tess)
   - Both issue and PR should move together

### **Check Morgan PM Logs**

```bash
# Get Morgan PM pod logs
kubectl logs -n agent-platform -l workflows.argoproj.io/component=morgan-project-manager --tail=100 | grep "Linking PR"

# Expected output:
# 🔗 Linking PR #456 to project for task-1...
# ✅ Added PR #456 to project (Item ID: PVTI_lADO...)
# ✅ Set PR #456 stage to: Rex (Implementation)
```

## Rollback

If issues arise, revert by changing line 718 back to:

```bash
# NOTE: PRs are NOT added to project board to avoid duplication
# PRs are linked to issues via "Closes #XXX" in PR description
```

And remove the `link_pr_to_project()` call.

## Future Enhancements

1. **PR-Specific Fields**
   - Add "Review Status" field (Approved, Changes Requested, Pending)
   - Add "Checks Status" field (Passing, Failing, Pending)
   - Link to preview deployments (for Bolt integration)

2. **Better Visualization**
   - Custom board view showing issue + PR pairs side-by-side
   - Automated grouping by task
   - Color coding by stage/status

3. **Notifications**
   - Slack/Discord notifications when PRs are created and linked
   - Mention relevant team members in project comments

## Summary

✅ **Pull requests are now automatically linked to GitHub Projects**
✅ **Full visibility** - Both issues and PRs visible in project board
✅ **Synchronized tracking** - Stages update together as workflow progresses
✅ **Robust implementation** - Retry logic, duplicate prevention, error handling
✅ **Non-breaking change** - Existing issue linking unchanged

The project board now provides comprehensive tracking of both task planning (issues) and implementation progress (PRs) in one unified view.

