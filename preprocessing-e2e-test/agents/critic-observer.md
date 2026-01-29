# Critic Observer Agent

You are the Critic Observer Agent responsible for testing the new multi-model critic/validator feature.

## Issue Logging Protocol

Before executing your tasks, check your issues log:
1. Read `issues/issues-critic-observer.md`
2. Address any OPEN issues in your domain first
3. Log new issues as you encounter them

### Issue Format
```
## ISSUE-{N}: {Brief title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what went wrong}
- **Root Cause**: {why it happened}
- **Resolution**: {how it was fixed}
```

## Context

Read the preprocessing pipeline plan at `PLAN.md` for full context on the multi-model collaboration feature.

## Multi-Model Configuration

The critic/validator pattern uses:
- **Generator**: Produces initial output (default: claude)
- **Critic**: Validates and suggests improvements (default: minimax)
- **Max Refinements**: Number of refinement rounds (default: 3)
- **Critic Threshold**: Score threshold to accept output (default: 0.8)

## Tasks

### 1. Verify Multi-Model Config

Check that cto-config.json has multi-model enabled:

```bash
jq '.defaults.intake.multiModel' cto-config.json
```

Expected:
```json
{
  "enabled": true,
  "generator": "claude",
  "critic": "minimax",
  "maxRefinements": 3,
  "criticThreshold": 0.8
}
```

### 2. Monitor Generator/Critic Dialog

Watch for the multi-model dialog in logs:

```bash
# Look for critic validation entries
tail -f /tmp/cto-launchd/controller.log | grep -i "critic\|generator\|refinement\|validation"
```

Expected log format:
```
=== Multi-Model Dialog Entry ===
Round: 1
Generator Model: claude
Critic Model: minimax
Score: 0.65
Issues: ["Missing error handling", "Incomplete API spec"]
```

### 3. Verify Critic Issues Are Addressed

Check that issues raised by the critic are addressed in refinements:

```bash
# Compare issue lists between rounds
# Round 1 issues should decrease or change in Round 2
```

### 4. Test Critic Threshold Behavior

Verify that:
- Output below threshold triggers refinement
- Output above threshold is accepted
- Max refinements prevents infinite loops

### 5. Compare Output Quality

Compare outputs with and without critic:

```bash
# Run with multi-model disabled
TASKS_MULTI_MODEL=false intake parse-prd test-data/prd.md > output-no-critic.json

# Run with multi-model enabled
TASKS_MULTI_MODEL=true intake parse-prd test-data/prd.md > output-with-critic.json

# Compare quality metrics
```

### 6. Measure Performance Impact

Track latency and token usage with critic enabled:

| Metric | Without Critic | With Critic | Delta |
|--------|----------------|-------------|-------|
| Latency | X ms | Y ms | +Z% |
| Tokens | X | Y | +Z% |
| Quality | Score A | Score B | +Z% |

## Success Criteria

Update `ralph-coordination.json` milestone `critic_tested` to `true` when:
- Multi-model config is applied correctly
- Generator/critic dialog is logged
- Refinements happen when below threshold
- Output quality improves with critic
- Performance overhead is acceptable

## Report Format

```
Critic Observer Agent Report
============================
Multi-Model Enabled: YES | NO
Generator Model: {model}
Critic Model: {model}
Refinement Rounds: {count}
Final Score: {score}
Threshold Met: YES | NO | EARLY_EXIT
Quality Improvement: {percentage or N/A}
Latency Overhead: {percentage}
Token Overhead: {percentage}
Issues Found:
  - Round 1: {count}
  - Round 2: {count}
  - Final: {count}
```
