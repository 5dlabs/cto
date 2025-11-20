# Systematic Agent Fix Plan - Comprehensive Root Cause Remediation

## Executive Summary

This fix addresses THREE critical systemic issues causing multiple reviews, duplicate agent runs, and quality gate bypasses across ALL agents and ALL CLIs.

## Problem Analysis

### Issue 1: Label Refreshing Creates Infinite Loops 🔄

**Evidence from PR #14**: Cipher posted 8 reviews with 5-6 minute gaps

**Root Cause**:
```bash
# Cipher completion logic (line 2260-2268)
echo "♻️ Refreshing 'ready-for-qa' label to trigger Tess"
gh api -X DELETE .../ready-for-qa    # Remove label
gh api -X POST .../ready-for-qa      # Re-add label
```

**What Happens**:
1. Cleo completes → adds `ready-for-security` label
2. Sensor triggers → Creates Cipher CodeRun #1
3. Cipher #1 completes → REMOVES+RE-ADDS `ready-for-qa` label
4. Sensor sees label event → Creates Cipher CodeRun #2
5. Cipher #2 completes → REMOVES+RE-ADDS `ready-for-qa` label
6. Loop continues... 8 times in PR #14!

### Issue 2: No Deduplication in Sensors

**Problem**: Sensors blindly create CodeRuns for every label event
**Missing**: Check if agent is already Running for this PR

### Issue 3: Tess Approves Without CI Validation

**Evidence from PR #11**: Tess approved while [CodeQL was FAILING](https://github.com/5dlabs/cto-parallel-test/actions/runs/19519043014/job/55878138862?pr=11)

**Missing**: CI status checking before approval

## Comprehensive Fix Strategy

### Phase 1: Remove Label Refreshing (All CLIs)
**Impact**: Prevents infinite loop of agent creation

**Files**:
- `cursor/container-base.sh.hbs`
- `factory/container-base.sh.hbs`
- `opencode/container-base.sh.hbs`
- `codex/container-base.sh.hbs`
- `claude/container.sh.hbs` (if it has this logic)

**Change**:
```bash
# BEFORE (BROKEN):
echo "♻️ Refreshing 'ready-for-qa' label"
gh api -X DELETE .../ready-for-qa
gh api -X POST .../ready-for-qa

# AFTER (FIXED):
# Don't manipulate labels - they're already set by previous agent
# Sensor watches for FIRST label addition, not refreshes
```

### Phase 2: Add Deduplication to Sensors
**Impact**: One agent run per PR per stage

**Files**:
- `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`

**Change**: Add deduplication check in sensor workflow scripts:
```bash
# Before creating CodeRun, check if one exists
PR_NUMBER="$1"
AGENT="cipher"  # or cleo, tess
STAGE="security"

EXISTING=$(kubectl get coderun -n agent-platform \
  -l pr-number="$PR_NUMBER",agent="$AGENT",stage="$STAGE",phase="Running" \
  --no-headers | wc -l)

if [ "$EXISTING" -gt 0 ]; then
  echo "✅ ${AGENT} already running for PR #${PR_NUMBER} - skipping duplicate creation"
  exit 0
fi
```

### Phase 3: Add CI Status Checking for Tess
**Impact**: Tess only approves when ALL CI checks pass

**Files**: All container-base templates

**Change**: Before Tess approval (line ~2342):
```bash
if [ "$WORKFLOW_STAGE" = "testing" ] && [ $SUCCESS -eq 1 ]; then
  echo "🔍 Verifying GitHub CI status before approval..."
  
  # Wait for CI to complete (max 5 minutes)
  CI_TIMEOUT=300
  CI_WAIT=0
  
  while [ $CI_WAIT -lt $CI_TIMEOUT ]; do
    # Check if any checks are still pending
    PENDING=$(gh pr checks "$PR_NUM" --json state --jq '[.[] | select(.state != "COMPLETED")] | length')
    
    if [ "${PENDING:-99}" -eq 0 ]; then
      # All checks completed, now verify they passed
      FAILED=$(gh pr checks "$PR_NUM" --json conclusion --jq '[.[] | select(.conclusion == "FAILURE" or .conclusion == "CANCELLED")] | length')
      
      if [ "${FAILED:-0}" -gt 0 ]; then
        echo "❌ CI checks FAILED - NOT approving"
        SUCCESS=0
      else
        echo "✅ All CI checks PASSED - safe to approve"
      fi
      break
    fi
    
    echo "⏳ Waiting for CI... (${CI_WAIT}s / ${CI_TIMEOUT}s)"
    sleep 15
    CI_WAIT=$((CI_WAIT + 15))
  done
  
  if [ $CI_WAIT -ge $CI_TIMEOUT ]; then
    echo "⚠️ CI checks still running after 5 minutes - NOT approving"
    SUCCESS=0
  fi
fi
```

### Phase 4: Fix Label Logic in Sensors
**Impact**: Labels only trigger ONCE, not on refresh

**Option A**: Sensor tracks which PRs it has already processed
**Option B**: Check for existing Running CodeRuns before creating new ones
**Recommended**: Option B (simpler, more reliable)

## Implementation Order

1. ✅ Remove label refreshing from all container-base templates
2. ✅ Add CI status checking for Tess (all CLIs)
3. ✅ Add deduplication to sensor workflow scripts
4. ✅ Test and verify
5. ✅ Create comprehensive PR

## Expected Outcomes

**Before**:
- Cipher: 8 reviews on PR #14 (REQUEST_CHANGES + APPROVED multiple times)
- Tess: 2 reviews on PR #11 (duplicate APPROVED)
- Tess: Approved while CI failing

**After**:
- ✅ ONE review per agent per PR (total)
- ✅ Tess waits for CI and only approves if green
- ✅ No duplicate CodeRun creation
- ✅ Clean, predictable workflow progression

## Files to Modify

**Container Templates (5 files)**:
1. `cursor/container-base.sh.hbs`
2. `factory/container-base.sh.hbs`
3. `opencode/container-base.sh.hbs`
4. `codex/container-base.sh.hbs`
5. `claude/container.sh.hbs`

**Sensor Workflows (1 file)**:
6. `play-workflow-sensors.yaml` (add deduplication logic)

**System Prompts (if needed)**:
7. `tess-system-prompt.md.hbs` (document CI checking requirement)

