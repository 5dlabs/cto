# Morgan PM Issue Linking Investigation

## Problem Statement

Morgan PM successfully creates GitHub Projects and Issues, but **issues are not being linked to projects** despite Morgan receiving successful responses from the GitHub API.

## Current Behavior

### What Works ✅

1. **Project Creation**: Morgan successfully creates org-level projects
   - Project ID: `PVT_kwDOC8B7k84BHR5c`
   - Project Title: "cto-parallel-test - TaskMaster Workflow"
   - Created via GraphQL `createProjectV2` mutation

2. **Issue Creation**: Morgan successfully creates issues
   - Issues #320-329 created successfully
   - All issues have proper labels, descriptions, and metadata

3. **GraphQL Responses**: Morgan receives "success" responses
   - AddProjectV2ItemById returns item IDs like `PVTI_lADOC8B7k84BHR5czggvjPI`
   - No GraphQL errors in the response
   - Morgan logs "✅ Added issue #320 to project"

4. **Status Monitoring**: Morgan watches workflow changes
   - Detects Rex working on tasks 1, 3, 4, 6
   - Posts status update comments to issues
   - Creates GitHub Checks on PRs
   - Syncs every 2 minutes

### What Doesn't Work ❌

1. **Issues NOT Linked to Project**
   ```bash
   gh issue view 320 --repo 5dlabs/cto-parallel-test --json projectItems
   # Returns: []  ← Empty!
   ```

2. **Project Fields NOT Updated**
   - Morgan calls `set_project_item_status` during sync
   - Morgan calls `set_project_item_agent` during sync  
   - BUT: No visible updates in GitHub Projects UI
   - No errors logged

3. **No Event-Driven Updates**
   - Morgan is watching workflows with `kubectl get workflows --watch`
   - But the watch is producing invalid JSON: "⚠️ Skipping invalid JSON line" (repeated)
   - Event loop is not triggering updates

## Deep Investigation

### 1. GraphQL Mutation Analysis

The `addProjectV2ItemById` mutation:

```graphql
mutation($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: {
    projectId: $projectId
    contentId: $contentId
  }) {
    item {
      id
    }
  }
}
```

**Inputs:**
- `projectId`: `PVT_kwDOC8B7k84BHR5c` (org-level project)
- `contentId`: `I_kwDOQKXbrM7V8G1Z` (issue node ID)

**Response:** Returns item ID `PVTI_lADOC8B7k84BHR5czggvjPI`

**But:** Issue's `projectItems` field is empty!

### 2. Possible Root Causes

#### A. Permission Mismatch

**Hypothesis**: Morgan has `organization_projects: admin` permission, but this might not grant access to link **repository issues** to **organization projects**.

**Evidence**:
- Morgan can CREATE org-level projects ✅
- Morgan can CREATE issues ✅  
- Morgan CANNOT link issues to projects ❌

**GitHub's Permission Model**:
- Organization Projects (v2) are separate from Repository Projects
- Linking a repo issue to an org project might require **both**:
  - `organization_projects: write` (Morgan has: admin ✅)
  - `repository_projects: write` (Morgan has: admin ✅)
  - **BUT**: The project must be **linked to the repository** first!

#### B. Project Not Linked to Repository

**Hypothesis**: The org-level project exists, but isn't properly linked to the `cto-parallel-test` repository.

**Evidence from logs**:
```
Linking project PVT_kwDOC8B7k84BHR5c to repository R_kgDOQKXbrA
```

Morgan TRIES to link the project, but we don't see confirmation it succeeded.

**Potential Issue**: The linking mutation might be failing silently.

#### C. API Response Mismatch

**Hypothesis**: GitHub API returns "success" but the operation actually fails due to:
- Rate limiting (silent)
- Internal GitHub error
- Stale cache on GitHub's side

**Evidence**:
- Item IDs are returned
- But items don't appear in issue's `projectItems` array
- This suggests a race condition or async operation that's failing

### 3. Event-Driven Watch Failure

**Critical Issue**: Morgan's event loop is broken!

```
[2025-11-05 04:44:01 UTC] ⚠️  Skipping invalid JSON line
(repeated 50+ times)
```

**Root Cause**: `kubectl get workflows --watch-only` output format issue

**Impact**:
- Real-time updates don't trigger
- Morgan only syncs every 2 minutes (backup mechanism)
- Status changes are delayed
- Project fields updates are delayed or skipped

### 4. Project Field Updates Not Executing

Looking at the sync logs:

```bash
[2025-11-05 05:02:23 UTC] 🔍 Syncing task-1: stage=implementation-in-progress, phase=Running
[2025-11-05 05:02:23 UTC] ⏭️  No change for task-1 (skipping comment posting)
```

**Missing**: No calls to `set_project_item_status` or `set_project_item_agent`!

**Why**: Morgan's `handle_task_event` function has deduplication logic that skips updates if "no change detected". But this logic might be TOO aggressive and skipping the initial project field setup.

## Recommended Fixes (Priority Order)

### 1. Fix kubectl watch JSON parsing (HIGH)

**Problem**: Event loop is completely broken
**Solution**: Handle both watch output formats properly

```bash
# Current (broken):
kubectl get workflows --watch-only -o json

# Better approach:
kubectl get workflows --watch -o json | jq -c .
# Or use --output-watch-events
```

### 2. Verify project-repository linking (HIGH)

**Problem**: Project might not be linked to repository
**Solution**: Add verification after linking

```bash
# After linking, verify:
gh api graphql -f query='
query($projectId: ID!) {
  node(id: $projectId) {
    ... on ProjectV2 {
      repositories(first: 10) {
        nodes {
          nameWithOwner
        }
      }
    }
  }
}' -f projectId="PVT_kwDOC8B7k84BHR5c"
```

### 3. Add comprehensive error logging (HIGH)

**Problem**: Silent failures - we don't know what's actually failing
**Solution**: Log full GraphQL responses, not just extracted fields

```bash
# Before:
echo "$result" | jq -r '.data.addProjectV2ItemById.item.id'

# After:
echo "$result" | tee /tmp/graphql-response.json | jq -r '.data.addProjectV2ItemById.item.id'
# Check for .errors field
```

### 4. Force initial project field sync (MEDIUM)

**Problem**: Deduplication skips initial field setup
**Solution**: Always set project fields on first sync, ignore cache

```bash
# Add flag to handle_task_event:
if [[ "$is_initial_sync" == "true" ]]; then
  # ALWAYS update project fields, ignore deduplication
  set_project_item_status "$project_id" "$item_id" "$task_status"
  set_project_item_agent "$project_id" "$item_id" "$current_agent"
fi
```

### 5. Use Repository-Level Projects Instead (ALTERNATIVE)

**Problem**: Org-level projects + repo issues might have permission issues
**Solution**: Create repository-level project instead

**Pros**:
- Simpler permission model
- Issues automatically linkable
- No cross-boundary issues

**Cons**:
- One project per repository (might want org-level view)
- Less visibility across organization

## Testing Plan

### Step 1: Verify Current State

```bash
# Check if project is linked to repo
gh api graphql -f query='query { node(id: "PVT_kwDOC8B7k84BHR5c") { ... on ProjectV2 { repositories(first: 10) { nodes { nameWithOwner } } } } }'

# Check if issue is linked to project (from project side)
gh api graphql -f query='query { node(id: "PVTI_lADOC8B7k84BHR5czggvjPI") { ... on ProjectV2Item { content { ... on Issue { number } } } } }'
```

### Step 2: Test Manual Linking

```bash
# Try to manually link issue to project
gh api graphql -f query='
mutation {
  addProjectV2ItemById(input: {
    projectId: "PVT_kwDOC8B7k84BHR5c"
    contentId: "I_kwDOQKXbrM7V8G1Z"
  }) {
    item {
      id
      content {
        ... on Issue {
          number
          projectItems(first: 5) {
            nodes {
              id
            }
          }
        }
      }
    }
  }
}'
```

### Step 3: Test with Repo-Level Project

```bash
# Create a repository-level project instead
gh project create --owner 5dlabs --repo cto-parallel-test --title "Test Repo Project"

# Try linking issue
gh project item-add <PROJECT_NUMBER> --owner 5dlabs --repo cto-parallel-test --url "https://github.com/5dlabs/cto-parallel-test/issues/320"
```

## Next Steps

1. Fix the kubectl watch JSON parsing issue (blocks real-time updates)
2. Add comprehensive GraphQL error logging
3. Test manual issue linking to confirm permissions
4. Consider switching to repository-level projects if org-level continues to fail
5. Add verification step after project-repository linking

## Questions to Answer

1. **Does the Morgan GitHub App have permission to link repo issues to org projects?**
   - Check: `organization_projects: admin` ✅
   - Check: `repository_projects: admin` ✅
   - BUT: Does this combination actually work across boundaries?

2. **Is the project properly linked to the repository?**
   - Need to verify with GraphQL query

3. **Why does the API return success but linking doesn't persist?**
   - Async operation?
   - Cache issue?
   - Permissions rejected after initial validation?

4. **Why is kubectl watch producing invalid JSON?**
   - Output format changed?
   - Need different flags?
   - Buffering issue?

