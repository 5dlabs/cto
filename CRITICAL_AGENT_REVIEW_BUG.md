# CRITICAL: Agent Review Posting Bug - Affects ALL Non-Claude CLIs

## Issue Summary

**TWO critical systemic bugs** affecting Cleo, Cipher, and Tess across Cursor, Factory, OpenCode, and Codex CLIs:

### Bug 1: Duplicate PR Reviews (Inside Retry Loop)
**Status**: ACTIVE BUG - Affects 4 CLIs  
**Evidence**: PR #11 shows Tess approved TWICE

PR reviews are posted INSIDE the `while [ $ATTEMPT -le $MAX_RETRIES ]` loop at lines ~2342-2420.

**Impact**:
- If MAX_RETRIES=2 and agent succeeds on iteration 1: Posts review TWICE
- If MAX_RETRIES=5 and agent takes 3 iterations: Posts review THREE times
- Confuses reviewers and pollutes PR timeline

### Bug 2: No CI Status Checking (Tess Specific)
**Status**: ACTIVE BUG - Affects ALL CLIs  
**Evidence**: PR #11 approved by Tess while CodeQL was FAILING  
**Link**: https://github.com/5dlabs/cto-parallel-test/actions/runs/19519043014/job/55878138862?pr=11

Tess approves immediately after running its own checks (fmt, clippy, test) but NEVER:
- Waits for GitHub Actions CI to complete
- Checks CI status before approving
- Validates that ALL checks are green

**Impact**:
- Tess approves PRs with failing CI
- Broken code can be approved for merge
- Quality gate is bypassed

## Affected Code Locations

### Cursor CLI
- File: `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
- Loop: Line 1553 `while [ $ATTEMPT -le $MAX_RETRIES ]`
- Loop end: Line 1921 `done`
- **Reviews posted INSIDE loop**: Lines 2342-2420
- **No CI checking**: Tess section has no `gh pr checks` validation

### Factory CLI  
- File: `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- Same pattern (lines slightly different)

### OpenCode CLI
- File: `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`
- Same pattern

### Codex CLI
- File: `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`
- Same pattern

### Claude CLI
- **ALREADY FIXED** in PR #1514 (Cipher) - reviews posted AFTER loop
- But still needs CI checking for Tess

## Required Fix

### Part 1: Move PR Reviews Outside Loop (All 4 CLIs)

```bash
# BEFORE (BROKEN):
while [ $ATTEMPT -le $MAX_RETRIES ]; do
  # ... agent runs ...
  if [ $SUCCESS -eq 1 ]; then
    gh pr review --approve  # ← Posted EVERY iteration!
  fi
  break
done

# AFTER (CORRECT):
while [ $ATTEMPT -le $MAX_RETRIES ]; do
  # ... agent runs ...
  # NO review posting here
  break if success
done

# POST-LOOP: Post review ONCE after loop completes
if [ -n "$PR_NUM" ]; then
  if [ $SUCCESS -eq 1 ]; then
    gh pr review --approve  # ← Posted ONCE only!
  else
    gh pr review --request-changes
  fi
fi
```

### Part 2: Add CI Status Checking for Tess (All 5 CLIs)

```bash
# BEFORE Tess approves, check CI:
if [ "$WORKFLOW_STAGE" = "testing" ] && [ $SUCCESS -eq 1 ]; then
  echo "🔍 Checking GitHub CI status before approval..."
  
  # Wait for CI to complete (max 5 minutes)
  CI_TIMEOUT=300
  CI_WAIT=0
  CI_COMPLETE=false
  
  while [ $CI_WAIT -lt $CI_TIMEOUT ]; do
    CI_STATUS=$(gh pr checks "$PR_NUM" --json state,conclusion --jq '[.[] | select(.state != "COMPLETED")] | length')
    if [ "${CI_STATUS:-99}" -eq 0 ]; then
      CI_COMPLETE=true
      break
    fi
    echo "⏳ Waiting for CI to complete... (${CI_WAIT}s / ${CI_TIMEOUT}s)"
    sleep 15
    CI_WAIT=$((CI_WAIT + 15))
  done
  
  if [ "$CI_COMPLETE" = "false" ]; then
    echo "⚠️ CI still running after ${CI_TIMEOUT}s - NOT approving"
    SUCCESS=0
  else
    # Check for failures
    FAILED_CHECKS=$(gh pr checks "$PR_NUM" --json name,conclusion --jq '[.[] | select(.conclusion == "FAILURE" or .conclusion == "CANCELLED")] | length')
    if [ "${FAILED_CHECKS:-0}" -gt 0 ]; then
      echo "❌ CI checks failing - NOT approving"
      SUCCESS=0
    else
      echo "✅ All CI checks passed - safe to approve"
    fi
  fi
fi
```

## Testing Verification

After fix is deployed, verify:

```bash
# 1. Tess should wait for CI
kubectl logs <tess-pod> | grep "Checking GitHub CI status"

# 2. Only ONE approval per run
gh api /repos/5dlabs/cto-parallel-test/pulls/11/reviews --jq '.[] | select(.user.login == "tess-5dlabs[bot]") | .submitted_at' | wc -l
# Should be 1, not 2+

# 3. Tess should NOT approve if CI failing
# Create test PR with failing CI → Tess should REQUEST_CHANGES
```

## Priority

**CRITICAL** - This affects production quality gates and allows broken code to be approved.

## Files to Fix

1. `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
2. `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
3. `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`
4. `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`
5. `infra/charts/controller/agent-templates/code/claude/container.sh.hbs` (CI checking only)
6. Agent system prompts (all CLIs) - add CI checking requirement

