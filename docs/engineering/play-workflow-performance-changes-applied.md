# Play Workflow Performance Optimizations - Changes Applied

**Date**: 2025-10-14
**Priority**: 1 & 2 Items from Remediation Plan

## Summary

Implemented three key optimizations to reduce play workflow completion time by an estimated 50%:

1. **Fast-Path Detection** for Cleo/Tess (saves ~10-20 minutes)
2. **Reduced PR Polling** from 60s to 20s (saves ~40 seconds per check)
3. **Per-Stage Timeout Guards** (prevents runaway executions)

## Changes Applied

### 1. Fast-Path Detection for Quality/Testing Agents

**File**: `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
**Lines**: Added after line 1165 (GitHub token refresh)

**What It Does**:
- Checks if PR already has approvals before running Cleo/Tess
- Skips redundant quality/testing checks if PR is already approved
- Posts explanatory comment to PR
- Only runs on first attempt (ATTEMPT=1)
- Only applies to quality and testing stages (not implementation)

**Logic**:
```bash
if [ -n "${PR_NUMBER:-}" ] && [ "$ATTEMPT" -eq 1 ] && [ "${WORKFLOW_STAGE:-}" != "implementation" ]; then
  APPROVAL_COUNT=$(gh pr view "$PR_NUMBER" --json reviews --jq '[.reviews[] | select(.state == "APPROVED")] | length')
  
  if [ "${APPROVAL_COUNT:-0}" -gt 0 ]; then
    # Skip work, post comment, mark as success
    SUCCESS=1
    break
  fi
fi
```

**Expected Impact**:
- **Cleo**: Saves 10-15 minutes when PR already approved
- **Tess**: Saves 15-20 minutes when PR already approved
- **Use Case**: Multi-task workflows where PR approved by Tess in previous task

**Testing**:
```bash
# Test scenario: Create a PR, approve it manually, then trigger Cleo
# Expected: Cleo should detect approval and skip quality checks in <30 seconds
```

### 2. Reduced PR Detection Polling

**File**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
**Line**: ~535 (check-or-wait-for-pr template)

**Change**:
```yaml
# Before:
attempts=12  # 60 seconds total (12 x 5s)

# After:
attempts=4   # 20 seconds total (4 x 5s)
```

**Rationale**:
- Agents (Rex) create PRs quickly after commits
- 60 seconds was excessive for typical PR creation time
- Most PRs appear within 5-10 seconds
- Faster failure detection allows quicker retry/remediation

**Expected Impact**:
- Saves 40 seconds per PR detection phase
- Faster workflow progression
- Earlier error detection if PR creation fails

**Testing**:
```bash
# Test scenario: Run Rex implementation without creating PR
# Expected: Workflow should fail after ~20 seconds instead of 60 seconds
```

### 3. Per-Stage Timeout Guards

**File**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
**Locations**: 
- Implementation work: line ~270
- Quality work: line ~350
- Testing work: line ~380

**Changes**:
```yaml
# Implementation
- name: implementation-work
  template: agent-coderun
  timeout: 25m  # NEW: Prevent runaway implementation

# Quality  
- name: quality-work
  template: agent-coderun
  timeout: 20m  # NEW: Prevent runaway quality checks

# Testing
- name: testing-work
  template: agent-coderun
  timeout: 30m  # NEW: Prevent runaway testing (E2E takes longer)
```

**Rationale**:
- Prevents agents from running indefinitely
- Forces failure and human review if taking too long
- Provides predictable completion times
- Helps identify stuck agents/models

**Expected Impact**:
- No legitimate workflow should hit these timeouts with optimized retry counts
- Workflows that would run for 90+ minutes now fail gracefully
- Better alerting and debugging for performance issues

**Testing**:
```bash
# Test scenario: Intentionally slow agent (e.g., add long sleep in prompt)
# Expected: Workflow should timeout and fail at the specified duration
```

## Configuration Verification

**File**: `cto-config.json`

Verified that base configuration is already optimal:
- Cleo: `claude-sonnet-4-20250514` (non-thinking) ✅
- Tess: `claude-sonnet-4-20250514` (non-thinking) ✅
- Rex: `claude-sonnet-4-5-20250929` (standard) ✅

**Note**: The slow workflow analyzed was using `sonnet-4.5-thinking` for Cleo, which was likely a test-specific parameter override.

## Expected Performance Improvements

### Before Optimizations
- **Rex**: 10-15 minutes
- **Cleo**: 60-90 minutes (with retries and no fast-path)
- **Tess**: 20-30 minutes
- **Total**: 90-135 minutes

### After Optimizations
- **Rex**: 10-15 minutes (unchanged)
- **Cleo**: 15-25 minutes (fast-path + timeout guards)
- **Tess**: 15-25 minutes (fast-path + timeout guards)
- **Total**: 40-65 minutes

**Expected Reduction**: 50-60% faster completion

## Deployment Steps

### 1. Test Changes Locally (Optional)
```bash
# Render templates to verify syntax
helm template infra/charts/controller --debug | grep -A 10 "timeout:"
```

### 2. Deploy to Cluster
```bash
# The controller chart will automatically pick up template changes
# ArgoCD should sync automatically, or trigger manually:
argocd app sync cto-controller
```

### 3. Monitor First Workflow
```bash
# Watch a workflow execution
kubectl get workflow -n agent-platform -l workflow-type=play-orchestration -w

# Check agent logs for fast-path detection
kubectl logs -n agent-platform -l workflow-stage=quality --tail=100 | grep "FAST-PATH"

# Verify timeouts are applied
kubectl describe workflow <workflow-name> -n agent-platform | grep -i timeout
```

## Validation Checklist

### Fast-Path Detection
- [ ] Cleo detects approved PR and skips work (< 30 seconds)
- [ ] Tess detects approved PR and skips work (< 30 seconds)
- [ ] Fast-path comment posted to PR
- [ ] SUCCESS status set correctly

### Reduced Polling
- [ ] PR detection completes in 5-10 seconds when PR exists
- [ ] PR detection fails after 20 seconds (not 60) when PR missing
- [ ] No race conditions or missed PRs

### Timeout Guards
- [ ] Normal workflows complete well within timeout limits
- [ ] Stuck workflows fail gracefully at timeout
- [ ] Timeout events logged to workflow events
- [ ] Alerts triggered for timeout failures

## Monitoring Metrics

Track these metrics to measure impact:

### Key Performance Indicators
1. **Average Workflow Duration**: Target 40-60 minutes (from 90-135)
2. **Fast-Path Hit Rate**: % of Cleo/Tess executions using fast-path
3. **Timeout Events**: Should be near-zero after optimization
4. **PR Detection Time**: Should be < 10 seconds average

### Grafana Queries (if monitoring enabled)
```promql
# Average workflow duration by stage
avg(workflow_stage_duration_seconds{type="play-orchestration"}) by (stage)

# Fast-path usage
count(workflow_fast_path_used{agent=~"cleo|tess"}) / count(workflow_agent_started{agent=~"cleo|tess"})

# Timeout events
sum(workflow_timeout_events{type="play-orchestration"}) by (stage)
```

## Rollback Plan

If issues arise:

### Fast-Path Detection
```bash
# Revert container-base.sh.hbs
git revert <commit-hash>

# Or: Comment out fast-path logic with {{#if false}}
```

### Reduced Polling
```bash
# Edit play-workflow-template.yaml
# Change: attempts=4 back to attempts=12
```

### Timeout Guards
```bash
# Remove timeout: lines from workflow template
# Agents will run until max-retries hit or completion
```

## Next Steps

### Priority 2 Remaining Items (Future)
1. **Progressive Success Criteria**: Allow Cleo to approve with partial completion
2. **Incremental Context**: Save agent state between retries
3. **Parallel Quality Gates**: Run Cleo + basic Tess concurrently

### Priority 3 Items (Future)
1. **Acceptance Criteria Service**: Dedicated validation service
2. **Agent Performance Metrics**: Full instrumentation
3. **Smart Model Selection**: Cost-optimized retry sequences

## Known Limitations

1. **Fast-Path**: Only detects existing approvals, doesn't prevent redundant work if approval happens mid-execution
2. **Timeout Guards**: May interrupt legitimate long-running operations (e.g., large codebase analysis)
3. **Reduced Polling**: May cause false failures if GitHub API is slow (rare)

## Questions & Troubleshooting

### Q: What if fast-path incorrectly skips work?
A: Fast-path only triggers if PR has APPROVED status. If approval is revoked, agent will run normally on retry.

### Q: What if 20 seconds isn't enough for PR detection?
A: Increase `attempts` back to 6-8 (30-40 seconds). Or investigate why PR creation is slow.

### Q: What if legitimate work hits timeout?
A: Increase timeout value for that stage. Also review why agent is taking so long (model selection, retry count, etc.)

### Q: How do I test these changes without affecting prod?
A: Deploy to staging cluster first, or use label selectors to target specific workflows.

## References

- **Full Analysis**: `docs/engineering/play-workflow-performance-remediation.md`
- **Workflow Template**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
- **Container Scripts**: `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
- **Configuration**: `cto-config.json`
