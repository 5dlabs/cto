# Claude Agent Retry Logic Implementation

## Problem
Claude-based agents (Rex, Cleo, Tess) don't have retry logic like Factory does. They run once and exit, regardless of the `maxRetries` configuration.

## Solution
Add retry loops to all three Claude agent templates, similar to Factory's retry mechanism.

## Implementation Plan

### 1. Cleo (Quality Agent)
**Retry until:** Quality checks pass and PR is approved
**Environment variable:** `CLAUDE_MAX_RETRIES` (falls back to `EXECUTION_MAX_RETRIES`)
**Completion criteria:**
- Claude exits successfully (exit code 0)
- Quality checks pass (or PR approved)
- `ready-for-qa` label added

### 2. Rex (Implementation Agent)
**Retry until:** PR is created with working code
**Environment variable:** `CLAUDE_MAX_RETRIES` (falls back to `EXECUTION_MAX_RETRIES`)
**Completion criteria:**
- Claude exits successfully (exit code 0)
- PR exists with task label
- Code compiles

### 3. Tess (Testing Agent)
**Retry until:** Tests pass
**Environment variable:** `CLAUDE_MAX_RETRIES` (falls back to `EXECUTION_MAX_RETRIES`)
**Completion criteria:**
- Claude exits successfully (exit code 0)
- Tests pass
- Deployment successful (if applicable)

## Changes Required

### Common Pattern
```bash
# Read MAX_RETRIES from environment
MAX_RETRIES=${CLAUDE_MAX_RETRIES:-${EXECUTION_MAX_RETRIES:-10}}
ATTEMPT=1
SUCCESS=0

# Retry loop
while [ $ATTEMPT -le $MAX_RETRIES ]; do
  echo "🚀 Attempt $ATTEMPT/$MAX_RETRIES"
  
  # Run Claude
  # ... existing Claude execution code ...
  
  # Check completion criteria
  if [ completion_criteria_met ]; then
    SUCCESS=1
    break
  fi
  
  if [ $ATTEMPT -lt $MAX_RETRIES ]; then
    echo "⚠️ Attempt $ATTEMPT incomplete, retrying..."
    ATTEMPT=$((ATTEMPT + 1))
  else
    break
  fi
done

# Report final status
if [ $SUCCESS -eq 1 ]; then
  echo "✅ Task completed successfully"
  exit 0
else
  echo "❌ Task incomplete after $MAX_RETRIES attempts"
  exit 1
fi
```

## Files to Modify
1. `infra/charts/controller/agent-templates/code/claude/container-cleo.sh.hbs`
2. `infra/charts/controller/agent-templates/code/claude/container-rex.sh.hbs`
3. `infra/charts/controller/agent-templates/code/claude/container-tess.sh.hbs`
4. `infra/charts/controller/agent-templates/code/claude/container-rex-remediation.sh.hbs`

## Testing
- Set `qualityMaxRetries: 2` in cto-config.json
- Run play workflow
- Verify Cleo retries if quality checks don't pass on first attempt
- Check logs show "Attempt 1/2", "Attempt 2/2", etc.

