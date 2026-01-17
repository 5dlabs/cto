# Healer Remediation Status

## Current State (2026-01-16 19:05 PST)

### Issues Created by Healer-Sensor

1. **[#3853](https://github.com/5dlabs/cto/issues/3853)** - CI Failure: Healer CI - fix/invalid-model-names-and-skills-search
   - **Workflow**: Healer CI
   - **Branch**: fix/invalid-model-names-and-skills-search
   - **Commit**: eddc7ca
   - **Created**: 2026-01-17 02:45:18 UTC

2. **[#3854](https://github.com/5dlabs/cto/issues/3854)** - CI Failure: Release Please - main
   - **Workflow**: Release Please
   - **Branch**: main
   - **Commit**: 172b137
   - **Created**: 2026-01-17 02:45:25 UTC

### Remediation CodeRuns Spawned

1. **healer-ci-atlas-7fr6z** (for #3853)
   - **Agent**: Atlas
   - **Model**: claude-opus-4-5-20251101
   - **Status**: **FAILED**
   - **Reason**: "Implementation agent must create a pull request (PR URL not found in status)"
   - **Issue**: Prompt had **empty logs** - Atlas had no context about what failed
   - **Created**: 2026-01-17 02:45:19 UTC
   - **Failed**: 2026-01-17 02:54:09 UTC (~9 minutes)

2. **healer-ci-atlas-xlhvd** (for #3854)
   - **Agent**: Atlas
   - **Model**: claude-opus-4-5-20251101
   - **Status**: **FAILED**
   - **Reason**: Same as above
   - **Created**: 2026-01-17 02:45:26 UTC
   - **Failed**: Similar timeframe

## The Problem

The healer remediation flow is **partially working** but has critical gaps:

### ✅ What's Working

1. **healer-sensor** is running and detecting CI failures
2. Issues are being created in GitHub with proper labels (`healer`, `ci-failure`)
3. Remediation CodeRuns are being spawned automatically
4. Linear issues are being linked to GitHub issues

### ❌ What's Broken

1. **Empty logs in prompts** - The CI failure logs aren't being fetched or are empty
   - Sensor logs show: `Executing gh command: gh run view 21084223190 --log-failed`
   - But the prompt shows: `## Failure Logs\n\`\`\`\n\n\`\`\``

2. **No PRs created** - Atlas CodeRuns complete but don't create PRs
   - Controller marks them as `Failed` because `PR URL not found in status`
   - Without logs, Atlas can't diagnose the issue

3. **No retry mechanism** - Failed remediations don't trigger re-attempts
   - The issues remain open with no further action

## Expected Flow (from docs/heal-play.md)

```
CI Failure Detected
    │
    ▼
GitHub Issue Created (#3853)
    │
    ▼
Remediation CodeRun Spawned (healer-ci-atlas-7fr6z)
    │
    ▼
Atlas Analyzes Logs ← **BROKEN: No logs provided**
    │
    ▼
Atlas Creates Fix PR ← **BROKEN: No PR created**
    │
    ▼
CI Passes on PR
    │
    ▼
PR Auto-Merged
    │
    ▼
Issue Closed
```

## Root Causes

### 1. Log Fetching Issue

The sensor is calling:
```rust
// crates/healer/src/sensors/github_actions.rs:565
fn fetch_workflow_logs(failure: &WorkflowFailure) -> Result<String> {
    let output = Self::execute_with_retry(&[
        "run", "view", &run_id_str,
        "--repo", &failure.repository,
        "--log-failed",
    ])?;
    // ...
}
```

But the logs are coming back empty. Possible reasons:
- The workflow run logs aren't available yet (timing issue)
- `gh run view --log-failed` returns nothing for this workflow
- Logs are too large and get truncated to empty

### 2. PR Creation Failure

Atlas is supposed to:
1. Analyze the logs
2. Fix the issue
3. Create a PR

But without logs, Atlas can't diagnose anything, so it doesn't create a PR.

The controller then fails the CodeRun because:
```rust
// Controller checks for PR URL in status
if pr_url.is_none() {
    return Err("Implementation agent must create a pull request");
}
```

## What Ralph Should Do

Ralph (the lifecycle test agent) should:

1. **Verify the healer remediation flow** by checking:
   - Are issues being created? ✅ Yes
   - Are CodeRuns being spawned? ✅ Yes
   - Are logs being fetched? ❌ No (empty)
   - Are PRs being created? ❌ No
   - Are PRs being merged? ❌ N/A (no PRs)

2. **Fix the log fetching** in `crates/healer/src/sensors/github_actions.rs`:
   - Add fallback to `--log` if `--log-failed` returns empty
   - Add retry logic with delay (logs might not be ready immediately)
   - Add truncation handling for large logs

3. **Add retry mechanism** for failed remediations:
   - If a remediation CodeRun fails without creating a PR
   - Wait 5 minutes and try again with fresh logs
   - Escalate to human after 3 attempts

4. **Update the healer-test PRD** to include these verification steps

## Immediate Actions Needed

1. Check if the CI workflow logs are actually available:
   ```bash
   gh run view 21084223190 --repo 5dlabs/cto --log
   ```

2. If logs are available, fix the log fetching in the sensor

3. Re-trigger remediation for the open issues:
   ```bash
   # Manually spawn a remediation CodeRun with proper logs
   ```

4. Verify the full flow works end-to-end

## Ralph Guardian Status

The Ralph Guardian is now monitoring:
- Ralph's progress on stories
- Progress.txt integrity
- Healer issue count
- Failed remediation CodeRuns

It will alert if:
- Ralph is stuck for >30 minutes
- Progress.txt shrinks (content deletion)
- Healer issues pile up without remediation
- Remediation CodeRuns keep failing
