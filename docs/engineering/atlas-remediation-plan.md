# Atlas PR Guardian - Remediation Plan

## Overview

This document outlines the remediation plan for existing open PRs that were not monitored by Atlas PR Guardian due to the sensor filter bug.

## Problem Statement

Between the initial Atlas PR Guardian deployment and the sensor fix (PR #1350), the sensor had a faulty filter expression that caused all `pull_request` events to be discarded. This means:

- **No Atlas CodeRuns were created** for PRs opened during this period
- **No automatic monitoring** occurred (Bugbot, CI, conflicts)
- **No auto-merge** happened, even for PRs that met all criteria

## Affected PRs

Any PR opened in `5dlabs/cto` between:
- **Start**: Initial Atlas sensor deployment (~2025-11-12 02:12:26Z)
- **End**: Sensor fix deployment (PR #1350 merged)

### Known Affected PRs

- **PR #1352**: "chore: update agent templates ConfigMaps"
  - Status: MERGEABLE, CLEAN
  - No CI checks required
  - No reviews required
  - **Should have been auto-merged** but wasn't

## Remediation Strategy

### Approach: Trigger Atlas via Comment Webhook

Since the sensor is now fixed, we can trigger Atlas for existing PRs by:

1. **Adding a comment** to each affected PR
2. **GitHub sends `issue_comment` webhook** to Argo Events
3. **Sensor processes event** with fixed filter expression
4. **Atlas CodeRun is created** for the PR
5. **Atlas evaluates PR** and takes appropriate action

This approach is:
- ✅ **Non-invasive**: No force-push or branch manipulation
- ✅ **Auditable**: Comment provides clear record
- ✅ **Safe**: Atlas will only merge if criteria are met
- ✅ **Automated**: Script handles all open PRs

### Alternative Approaches (Not Recommended)

#### Close/Reopen PRs
- ❌ Loses PR history and context
- ❌ Triggers unnecessary notifications
- ❌ May confuse contributors

#### Manual Merge
- ❌ Doesn't validate Atlas functionality
- ❌ Requires manual intervention for each PR
- ❌ Doesn't scale

#### Force Webhook Replay
- ❌ Complex to implement
- ❌ Requires direct EventSource manipulation
- ❌ Risk of duplicate events

## Implementation

### Remediation Script

**Location**: `scripts/trigger-atlas-for-existing-prs.sh`

**Features**:
- Fetches all open PRs from `5dlabs/cto`
- Checks if Atlas CodeRun already exists
- Adds comment to trigger Atlas (only if needed)
- Provides dry-run mode for testing
- Comprehensive logging and summary

### Usage

#### Dry Run (Recommended First)
```bash
# See what would happen without making changes
DRY_RUN=true ./scripts/trigger-atlas-for-existing-prs.sh
```

#### Execute Remediation
```bash
# Actually trigger Atlas for affected PRs
DRY_RUN=false ./scripts/trigger-atlas-for-existing-prs.sh
```

### Prerequisites

1. **Sensor Fix Deployed**: PR #1350 must be merged and synced
2. **GitHub CLI Authenticated**: `gh auth status` should show authentication
3. **kubectl Access**: Access to agent-platform namespace
4. **Permissions**: Ability to comment on PRs in 5dlabs/cto

## Execution Plan

### Phase 1: Validation (Pre-Remediation)

1. **Verify Sensor Fix**:
   ```bash
   kubectl get sensor atlas-pr-guardian -n argo -o jsonpath='{.spec.dependencies[0].filters.exprs[0].expr}'
   ```
   Should show the fixed expression.

2. **Check Sensor Health**:
   ```bash
   ./scripts/test-atlas-sensor-fix.sh
   ```
   Should show no filtering errors.

3. **Test with New PR** (Optional):
   - Create a test PR
   - Verify Atlas CodeRun is created
   - Verify Atlas behavior

### Phase 2: Dry Run

```bash
DRY_RUN=true ./scripts/trigger-atlas-for-existing-prs.sh
```

**Expected Output**:
```
🔄 Atlas PR Guardian Remediation Script
========================================
Repository: 5dlabs/cto
Dry Run: true

📋 Fetching open PRs...
Found 1 open PR(s)

🔍 Checking PRs for Atlas activation...

PR #1352: chore: update agent templates ConfigMaps
  Author: app/github-actions
  Created: 2025-11-12T20:51:02Z
  Mergeable: MERGEABLE
  ⚠️  No Atlas CodeRun found - triggering activation...
  [DRY RUN] Would comment on PR #1352 to trigger Atlas

========================================
📊 Remediation Summary
========================================
Total PRs checked: 1
Already active: 0
Triggered: 1
Failed: 0

⚠️  This was a DRY RUN - no changes were made
Run with DRY_RUN=false to actually trigger Atlas
```

### Phase 3: Execute Remediation

```bash
DRY_RUN=false ./scripts/trigger-atlas-for-existing-prs.sh
```

**What Happens**:
1. Script adds comment to PR #1352
2. GitHub sends `issue_comment` webhook
3. Atlas sensor receives event
4. Filter expression passes (fixed)
5. Atlas CodeRun created
6. Atlas evaluates PR state
7. Atlas auto-merges (if criteria met)

### Phase 4: Verification

1. **Check CodeRun Creation**:
   ```bash
   kubectl get coderun -n agent-platform -l agent=atlas
   ```
   Should show CodeRuns for remediated PRs.

2. **Monitor Sensor Logs**:
   ```bash
   kubectl logs -f $(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) -n argo
   ```
   Should show successful event processing.

3. **Verify PR Status**:
   ```bash
   gh pr view 1352 --repo 5dlabs/cto
   ```
   Should show Atlas comment and/or merged status.

## Expected Outcomes

### For PR #1352 (Automated Template Update)

**Current State**:
- ✅ Mergeable: CLEAN
- ✅ No CI checks required
- ✅ No reviews required
- ✅ No conflicts
- ✅ Created by GitHub Actions bot

**Expected Outcome**:
1. Script adds comment to PR
2. Atlas CodeRun created within 30 seconds
3. Atlas evaluates PR:
   - No Bugbot comments ✅
   - No CI failures ✅
   - No merge conflicts ✅
   - PR is mergeable ✅
4. **Atlas auto-merges PR** (squash merge)
5. Atlas posts summary comment
6. PR closed as merged

**Timeline**: 1-2 minutes from comment to merge

### For Other PRs (If Any)

PRs that don't meet auto-merge criteria will:
- Get Atlas CodeRun created
- Be monitored by Atlas
- Wait for criteria to be met
- Auto-merge when ready

## Rollback Plan

If remediation causes issues:

1. **Stop Script**: Ctrl+C if still running
2. **Delete Atlas CodeRuns**:
   ```bash
   kubectl delete coderun -n agent-platform -l agent=atlas,role=pr-guardian
   ```
3. **Manual Review**: Review affected PRs manually
4. **Report Issues**: Document any unexpected behavior

## Success Criteria

- ✅ All open PRs have Atlas CodeRuns created
- ✅ No filtering errors in sensor logs
- ✅ PR #1352 auto-merged successfully
- ✅ Atlas comments visible on PRs
- ✅ No duplicate CodeRuns created

## Monitoring

### Key Metrics

1. **CodeRun Creation Rate**:
   ```bash
   kubectl get coderun -n agent-platform -l agent=atlas --sort-by=.metadata.creationTimestamp
   ```

2. **Sensor Event Processing**:
   ```bash
   kubectl logs -n argo $(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) --tail=50
   ```

3. **PR Merge Activity**:
   ```bash
   gh pr list --repo 5dlabs/cto --state merged --limit 10
   ```

### Alert Conditions

- ⚠️ CodeRun not created within 60 seconds of comment
- ⚠️ Filtering errors appear in sensor logs
- ⚠️ Atlas fails to merge eligible PRs
- ⚠️ Multiple CodeRuns created for same PR

## Post-Remediation

### Cleanup

1. **Review Atlas Comments**: Ensure they're appropriate
2. **Document Results**: Update this document with actual outcomes
3. **Archive Logs**: Save sensor logs for reference
4. **Update Runbook**: Add lessons learned

### Future Prevention

1. **Monitoring**: Add alerts for sensor filtering errors
2. **Testing**: Include sensor filter tests in CI
3. **Documentation**: Keep webhook payload examples updated
4. **Validation**: Test sensor changes with real webhooks

## Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Validation | 5 minutes | Pending |
| Dry Run | 2 minutes | Pending |
| Execute | 5 minutes | Pending |
| Verification | 10 minutes | Pending |
| **Total** | **~22 minutes** | Pending |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Duplicate CodeRuns | Low | Script checks before triggering |
| Unexpected Auto-Merge | Low | Atlas only merges if criteria met |
| API Rate Limits | Low | Script processes PRs sequentially |
| Webhook Delays | Low | Wait 60s between checks |
| Script Failure | Low | Dry-run mode available |

## Contact

For issues during remediation:
- **Script Issues**: Check logs, run with `set -x` for debug
- **Sensor Issues**: Check sensor logs, verify fix deployment
- **Atlas Issues**: Check CodeRun logs, verify GitHub App permissions

## References

- **Sensor Fix**: PR #1350
- **Original Issue**: ATLAS_FIX_SUMMARY.md
- **Event Flow**: docs/engineering/atlas-pr-guardian-flow.md
- **Test Script**: scripts/test-atlas-sensor-fix.sh

