# Fix: PR Discovery and Branch Management Issues

## Problem Statement

Task 7 and Task 8 encountered critical workflow coordination issues:

### Task 7 Issues:
- **Duplicate PRs Created**: PR #84 (`feature/task-7-implementation-20251025210730`) and PR #85 (`feature/task-7-implementation`)
- **Merge Conflicts**: PR #84 marked as "CONFLICTING" and unable to merge
- **Stuck Workflow**: Workflow waiting for PR that has unresolvable conflicts

### Task 8 Issues:
- **Premature Start**: Task 8 (PR #86) started before Task 7's workflow completed
- **No Coordination**: No mechanism preventing parallel task execution

### Root Cause:
When Rex encounters an existing branch that can't be fast-forwarded, the system creates a **timestamped fallback branch** (e.g., `feature/task-7-implementation-20251025210730`). This causes:
1. Duplicate PRs for the same task
2. Workflow PR discovery failures (wrong branch name)
3. Merge conflicts that can't be resolved
4. Workflows stuck indefinitely

## Solution

### 1. Intelligent Branch Handling (Eliminates Timestamped Fallbacks)

**Before:**
```bash
# When branch can't fast-forward:
NEW_BRANCH="${FEATURE_BRANCH}-$(date -u +%Y%m%d%H%M%S)"
git checkout -B "$NEW_BRANCH" "$REMOTE_BASE"
FEATURE_BRANCH="$NEW_BRANCH"
# Result: feature/task-7-implementation-20251025210730
```

**After:**
```bash
# When branch can't fast-forward:
EXISTING_PR=$(gh pr list --state open --head "$FEATURE_BRANCH" --json number)
if [ -n "$EXISTING_PR" ]; then
  # PR exists - continue with it (conflicts handled in PR)
  echo "✅ Found existing PR #$EXISTING_PR"
else
  # No PR - recreate branch fresh
  git branch -D "$FEATURE_BRANCH"
  git checkout -b "$FEATURE_BRANCH" "$REMOTE_BASE"
fi
# Result: Always feature/task-7-implementation
```

**Impact:**
- ✅ No more timestamped branches
- ✅ Single PR per task
- ✅ Cleaner git history
- ✅ Conflicts handled in PR review process

### 2. Enhanced PR Discovery (Handles Labeling Race Conditions)

**Before:**
```bash
# Only searched by labels:
GET /repos/{owner}/{repo}/issues?labels=task-7,run-workflow-name
# Problem: Fails if PR created before labels applied
```

**After:**
```bash
# Step 1: Try labels (fast path)
GET /repos/{owner}/{repo}/issues?labels=task-7,run-workflow-name

# Step 2: Fallback to branch name
if not found:
  GET /repos/{owner}/{repo}/pulls?head=owner:feature/task-7-implementation

# Step 3: Add labels immediately when found
POST /repos/{owner}/{repo}/issues/{pr_number}/labels
```

**Impact:**
- ✅ Discovers PRs regardless of label timing
- ✅ Automatically adds correlation labels
- ✅ Eliminates workflow stuck states
- ✅ Handles timing edge cases

## Files Modified

### Agent Templates (Branch Handling):
- `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/claude/container-rex.sh.hbs`
- `infra/charts/controller/agent-templates/code/claude/container-rex-remediation.sh.hbs`
- `infra/charts/controller/agent-templates/code/claude/container-tess.sh.hbs`

### Workflow Template (PR Discovery):
- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`

## Testing Recommendations

### 1. Test Scenario: Existing Branch with Merge Conflict
```bash
# Setup:
# 1. Create feature/task-X-implementation branch
# 2. Make conflicting changes in main
# 3. Start task workflow

# Expected:
# - Rex checks for existing PR
# - If none: Recreates branch from main
# - If exists: Continues with existing PR
# - No timestamped branch created
```

### 2. Test Scenario: PR Discovery
```bash
# Setup:
# 1. Create PR without labels
# 2. Workflow wait-for-pr step starts

# Expected:
# - First attempt: Search by labels (fails)
# - Fallback: Search by branch name (succeeds)
# - Adds labels: task-X, run-workflow-name, service-name
# - Returns PR URL and number
```

### 3. Test Scenario: Sequential Task Coordination
```bash
# Setup:
# 1. Start task 7
# 2. PR #85 merges successfully
# 3. Start task 8 immediately

# Expected:
# - Task 8 proceeds normally
# - No duplicate PRs
# - Clean workflow progression
```

## Verification Steps

1. **Close duplicate PR #84**:
   ```bash
   gh pr close 84 -R 5dlabs/rust-basic-api --comment "Closing duplicate PR. Fixed via PR #85."
   ```

2. **Terminate stuck workflow**:
   ```bash
   kubectl delete workflow play-workflow-template-82kw9 -n agent-platform
   ```

3. **Merge this PR and sync ArgoCD**

4. **Run new task workflow** and verify:
   - ✅ Single PR created
   - ✅ No timestamped branches
   - ✅ PR discovery succeeds
   - ✅ Labels applied correctly

## Benefits

### Immediate:
- ✅ Eliminates duplicate PR creation
- ✅ Prevents workflow stuck states
- ✅ Resolves Task 7/8 coordination issues

### Long-term:
- ✅ Cleaner git branch management
- ✅ More reliable PR discovery
- ✅ Better workflow state management
- ✅ Reduced manual intervention needed

## Breaking Changes

**None.** This is a backward-compatible fix that improves existing behavior without changing interfaces or contracts.

## Follow-up Considerations

1. **Workflow Coordination**: Consider adding explicit task sequencing to prevent Task N+1 from starting until Task N's workflow fully completes (not just PR merge).

2. **Conflict Resolution**: For PRs with conflicts, consider:
   - Auto-rebase when safe
   - Clear notifications to reviewers
   - Graceful failure paths

3. **Label Management**: Consider making labels required metadata during PR creation rather than post-creation addition.


