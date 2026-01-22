# PR #3903 Debug - Why Conflicts Not Being Resolved

## Current Status

- **PR #3903**: "Add rust lint 7e8b6"
- **Status**: `mergeable: CONFLICTING`, `mergeStateStatus: DIRTY`
- **Base Branch**: `main` (not `develop`)
- **Merger Agent**: Running (PID 24707)
- **Current PR Being Processed**: #3902

## Root Cause

The conflict resolution code was hardcoded to use `develop`:

```bash
# OLD (BROKEN):
git fetch origin develop
git rebase origin/develop
```

But PR #3903 targets `main`, so:
- The agent tries to merge with `develop`
- This doesn't resolve the actual conflicts with `main`
- The PR remains in CONFLICTING/DIRTY state

## Fix Applied

Updated `merger-prompt.md` conflict resolution section to:
1. Get the PR's actual `baseRefName` (could be `develop` or `main`)
2. Use that base branch for conflict resolution
3. Prioritize PRs with conflicts first

## Why It's Not Being Processed

Possible reasons:
1. **Agent is stuck on PR #3902** - May be waiting for CI or approval
2. **Not prioritizing conflicts** - May be processing PRs in order instead of by priority
3. **Agent needs restart** - The prompt changes won't take effect until agent restarts

## Next Steps

1. **Restart the merger agent** to pick up the updated prompt:
   ```bash
   # Kill current agent
   kill 24707
   
   # Restart it
   cd pr-merge-loop
   ./run-merger.sh
   ```

2. **Verify PR #3903 is detected**:
   ```bash
   gh pr list --state open --json number,mergeable,mergeStateStatus | jq '.[] | select(.number == 3903)'
   ```

3. **Check if conflicts are resolved**:
   ```bash
   gh pr view 3903 --json mergeable,mergeStateStatus
   ```

## Expected Behavior After Fix

When the agent processes PR #3903:
1. Detects `mergeable: CONFLICTING` and `mergeStateStatus: DIRTY`
2. Gets `baseRefName: "main"` from the PR
3. Fetches `origin/main` (not `origin/develop`)
4. Merges/rebase with `origin/main` to resolve conflicts
5. Pushes the resolved branch
6. PR becomes `mergeable: MERGEABLE`
