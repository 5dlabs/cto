# PR to GitHub Project Linking - Architecture Fix

## Problem Identified

There were **two separate, conflicting systems** attempting to link Pull
Requests to GitHub Projects:

### 1. Container Scripts (Incorrect Implementation)

**Location**: All agent container scripts (Rex, Blaze, Codex, Cursor,
Factory, OpenCode)

**Method**:

```bash
gh pr edit "$PR_NUMBER" --add-project "$PROJECT_TITLE"
```

**Issues**:

- Uses project **title string** (e.g., `"service-name - TaskMaster
  Workflow"`)
- Simple CLI command that relies on GitHub searching by name
- Redundant with Morgan PM's authoritative implementation
- Race condition: tries to link before Morgan creates project
- Often fails silently with "may not exist yet or PR already linked"
- Created inconsistency in project management approach

### 2. Morgan PM (Correct Implementation)

**Location**: `morgan-pm.sh.hbs`

**Method**:

```bash
link_pr_to_project "$project_id" "$pr_number" "$task_id" "$current_stage"
```

**Correct Approach**:

- Uses GraphQL **node ID** (e.g., `PVT_kwDOABCDEF`)
- Precise GraphQL API operations via `add_issue_to_project()`
- Checks if PR already exists in project before adding
- Sets proper stage on project board
- Part of event-driven workflow architecture
- Single source of truth for project management

## Type Mismatch Explanation

The user reported: "_The `link_pr_to_project` function is called with
`$project_id` (a GraphQL node ID), but the container scripts that create PRs
use `gh pr edit --add-project "$PROJECT_TITLE"` which expects a project
title string._"

This was **not** a bug in Morgan PM - Morgan correctly uses GraphQL node IDs.
The issue was that container scripts were attempting to do Morgan's job using
an inferior method.

## Root Cause

Recent PR added PR linking to container scripts, creating redundancy:

```diff
+    # Link PR to GitHub Project (Morgan PM creates project for task management)
+    echo "🔗 Linking PR to GitHub Project..."
+    PROJECT_TITLE="{{service}} - TaskMaster Workflow"
+    
+    if gh pr edit "$PR_NUMBER" --add-project "$PROJECT_TITLE" >/dev/null 2>&1; then
+      echo "✅ Linked PR #$PR_NUMBER to project: $PROJECT_TITLE"
+    else
+      echo "⚠️ Could not link PR to project (project may not exist yet or PR
+            already linked)"
+      echo "   Morgan PM will create/link the project during workflow execution"
+    fi
```

**Note the irony**: The comment admits "Morgan PM will create/link the project"
yet the code tries to do it anyway!

## Solution

**Removed all PR linking code from container scripts**. Morgan PM is the
single source of truth.

### Files Fixed

- `container-rex.sh.hbs`
- `container-blaze.sh.hbs`
- `container-rex-remediation.sh.hbs`
- `codex/container-base.sh.hbs`
- `cursor/container-base.sh.hbs`
- `factory/container-base.sh.hbs`
- `opencode/container-base.sh.hbs`

### Change Applied

Replaced redundant PR linking code with simple informational message:

```bash
echo "ℹ️  Morgan PM will link this PR to the GitHub Project during workflow execution"
```

## Architecture Benefits

### Single Responsibility

- **Container Scripts**: Create PRs with proper labels
- **Morgan PM**: Manage GitHub Projects, link items, update stages

### Event-Driven Workflow

1. Container creates PR with correlation labels (`task-*`, `service-*`,
   `run-*`)
2. GitHub webhook fires PR creation event
3. Morgan PM receives event via Argo Events
4. Morgan PM links PR to project using GraphQL node ID
5. Morgan PM sets proper stage on project board
6. Morgan PM maintains synchronization throughout workflow lifecycle

### GraphQL vs CLI

**Why GraphQL node IDs are superior**:

- **Precise**: No ambiguity with multiple projects having similar names
- **Atomic**: Can check existence before adding (prevents duplicates)
- **Metadata-rich**: Can set stage, custom fields, etc. in same operation
- **Event-driven**: Fits naturally into webhook-based architecture
- **Resilient**: Handles project creation timing properly

**Why `gh pr edit --add-project` was problematic**:

- **String matching**: Searches by title, could match wrong project
- **Race conditions**: Fails if project doesn't exist yet
- **Limited metadata**: Can't set stage or custom fields
- **Error-prone**: Silent failures with generic error messages
- **Redundant**: Duplicates Morgan PM's responsibilities

## Verification

After this fix:

1. ✅ Container scripts focus on PR creation only
2. ✅ Morgan PM has exclusive responsibility for project linking
3. ✅ No race conditions between systems
4. ✅ Consistent use of GraphQL APIs for project operations
5. ✅ Clear separation of concerns
6. ✅ Event-driven architecture preserved

## Testing Recommendations

1. Create test workflow with Morgan PM enabled
2. Verify PR gets created by implementation agent (Rex/Blaze)
3. Confirm Morgan PM links PR to project (check GitHub Projects board)
4. Validate PR appears with correct stage
5. Ensure no duplicate linking attempts in logs

## Related Documentation

- `PLAY_WORKFLOW_READINESS_ANALYSIS.md` - Overall workflow architecture
- `MORGAN_PM_SUMMARY.md` - Morgan PM responsibilities and design
- GitHub Projects V2 API docs - GraphQL operations reference
