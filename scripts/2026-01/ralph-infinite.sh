#!/usr/bin/env bash
# Ralph Infinite Autonomous Loop
#
# This script wraps ralph-cto.sh in an infinite loop that:
# 1. Runs the lifecycle test phases
# 2. On failure, spawns a remediation agent to investigate and fix
# 3. Cleans up and resets to intake phase
# 4. Loops forever, never stopping
#
# Usage:
#   ./scripts/2026-01/ralph-infinite.sh [--config path] [--no-remediation]
#
#   --config          Path to ralph-cto.json (default: lifecycle-test/ralph-cto.json)
#   --no-remediation  Skip spawning remediation agent on failure (just cleanup and retry)

set -uo pipefail  # Note: no -e, we handle errors manually

ROOT_DIR="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONFIG_PATH="${RALPH_CONFIG:-${ROOT_DIR}/lifecycle-test/ralph-cto.json}"
ENABLE_REMEDIATION=1
INFINITE_LOG="/tmp/ralph-infinite.log"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --config)
      CONFIG_PATH="$2"
      shift 2
      ;;
    --no-remediation)
      ENABLE_REMEDIATION=0
      shift
      ;;
    -h|--help)
      cat <<'EOF'
Usage: scripts/2026-01/ralph-infinite.sh [--config path] [--no-remediation]

  --config          Path to ralph-cto.json (default: lifecycle-test/ralph-cto.json)
  --no-remediation  Skip spawning remediation agent on failure

Ralph runs forever, fixing issues and restarting automatically.
EOF
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
done

# Load config values
json_get() {
  jq -r "$1" "$CONFIG_PATH"
}

# Configuration from ralph-cto.json
STATE_PATH="${ROOT_DIR}/$(json_get '.paths.statePath')"
PROGRESS_PATH="${ROOT_DIR}/$(json_get '.paths.progressPath')"
REPORT_PATH="${ROOT_DIR}/$(json_get '.paths.reportPath')"
LOG_DIR="${ROOT_DIR}/lifecycle-test/ralph-logs"
REMEDIATION_PROMPT="${ROOT_DIR}/$(json_get '.infinite.remediationPromptPath // "lifecycle-test/remediation-prompt.md"')"

# Infinite mode config with defaults
COOLDOWN_SECONDS=$(json_get '.infinite.cooldownSeconds // 30')
SUCCESS_COOLDOWN_SECONDS=$(json_get '.infinite.successCooldownSeconds // 60')
MAX_CONSECUTIVE_FAILURES=$(json_get '.infinite.maxConsecutiveFailures // 10')

# State tracking
consecutive_failures=0
total_cycles=0
total_successes=0
total_failures=0

log() {
  local msg="[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"
  echo "$msg" >&2
  echo "$msg" >> "$INFINITE_LOG"
}

log_separator() {
  log "════════════════════════════════════════════════════════════════════════════════"
}

append_progress() {
  {
    echo ""
    echo "## $(date -u '+%Y-%m-%d %H:%M:%SZ') [INFINITE]"
    echo "$1"
  } >> "$PROGRESS_PATH"
}

# Find the most recent log file for a given phase
find_latest_log() {
  local phase="$1"
  find "$LOG_DIR" -name "${phase}_*" -type f 2>/dev/null | sort -r | head -1
}

# Reset state to intake phase
reset_to_intake() {
  log "Resetting state to intake phase..."
  
  local tmp
  tmp="$(mktemp)"
  jq '.phase = "intake" | .attempts = {} | .completedObjectives = [] | .attendedCompleted = (.attendedCompleted // 0)' \
    "$STATE_PATH" > "$tmp" && mv "$tmp" "$STATE_PATH"
  
  log "State reset complete"
}

# Run global cleanup commands
run_cleanup() {
  log "Running global cleanup..."
  
  # Delete all CodeRuns
  kubectl delete coderuns -n cto --all --wait=false 2>/dev/null || true
  
  # Delete completed/failed pods
  kubectl delete pods -n cto --field-selector=status.phase!=Running --wait=false 2>/dev/null || true
  
  # Delete PVCs for the test service
  kubectl delete pvc -n cto -l service=prd-alerthub-e2e-test --wait=false 2>/dev/null || true
  kubectl delete pvc -n cto workspace-prd-alerthub-e2e-test-morgan --wait=false 2>/dev/null || true
  
  # Wait for cleanup to propagate
  sleep 10
  
  log "Cleanup complete"
}

# Spawn remediation agent to investigate and fix the failure
spawn_remediation_agent() {
  local exit_code="$1"
  local phase
  phase=$(jq -r '.phase // "unknown"' "$STATE_PATH")
  
  log "Spawning remediation agent for phase '$phase' (exit code: $exit_code)"
  
  # Find the most recent log file
  local latest_log
  latest_log=$(find "$LOG_DIR" -type f -name "*.log" 2>/dev/null | sort -r | head -1)
  local log_content=""
  if [[ -f "$latest_log" ]]; then
    log_content=$(tail -100 "$latest_log" 2>/dev/null || echo "Could not read log")
  fi
  
  # Get recent progress
  local recent_progress
  recent_progress=$(tail -50 "$PROGRESS_PATH" 2>/dev/null || echo "No progress file")
  
  # Get recent report entries
  local recent_report
  recent_report=$(jq -c '.[-5:]' "$REPORT_PATH" 2>/dev/null || echo "[]")
  
  # Get pod status
  local pod_status
  pod_status=$(kubectl get pods -n cto -o wide 2>/dev/null || echo "Could not get pods")
  
  # Get CodeRun status
  local coderun_status
  coderun_status=$(kubectl get coderuns -n cto -o json 2>/dev/null | jq -c '.items[] | {name: .metadata.name, phase: .status.phase, message: .status.message}' 2>/dev/null || echo "Could not get coderuns")
  
  # Build failure context
  local failure_context
  failure_context=$(cat <<EOF
# Remediation Task

Ralph's lifecycle test failed and needs your help to fix it.

## Failure Context

- **Phase**: ${phase}
- **Exit Code**: ${exit_code}
- **Consecutive Failures**: ${consecutive_failures}
- **Timestamp**: $(date -u '+%Y-%m-%dT%H:%M:%SZ')

## Recent Log Output

\`\`\`
${log_content}
\`\`\`

## Recent Progress

\`\`\`
${recent_progress}
\`\`\`

## Recent Report Entries

\`\`\`json
${recent_report}
\`\`\`

## Current Pod Status

\`\`\`
${pod_status}
\`\`\`

## Current CodeRun Status

\`\`\`
${coderun_status}
\`\`\`

## Your Task

1. **Analyze** the failure context above
2. **Identify** the root cause
3. **Fix** the issue:
   - If it's a code bug, edit the relevant files
   - If it's a config issue, update the config
   - If it's infrastructure, fix or restart services
4. **Rebuild** if you made code changes: \`cargo build --release\`
5. **Restart** services if needed: \`just launchd-restart\`

After you finish, the system will automatically:
- Run cleanup
- Reset to intake phase
- Restart the lifecycle test

## Key Files

- Config: lifecycle-test/ralph-cto.json
- State: lifecycle-test/ralph-cto.state.json
- Progress: lifecycle-test/progress.txt
- Logs: lifecycle-test/ralph-logs/

## Common Issues

1. **Controller bug** - CodeRun status not updating
   - Check: \`kubectl get coderun <name> -n cto -o yaml\`
   - Fix in: crates/controller/

2. **Agent wrong** - Task assigned to wrong agent
   - Check: PR author vs expected agent
   - Fix in: crates/mcp/src/main.rs

3. **Linear sidecar** - Not terminating with main container
   - Check: \`kubectl get pods -n cto\` for stuck sidecars
   - Fix in: crates/controller/src/tasks/code/resources.rs

4. **Webhook missing** - WEBHOOK_CALLBACK_URL not set
   - Check: launchd environment
   - Fix: scripts/2026-01/launchd-setup.sh

5. **Service health** - Local services not running
   - Check: \`curl localhost:808X/health\`
   - Fix: \`just launchd-restart\`

Do your best to fix the issue. If you cannot fix it, document what you found and what you tried.
EOF
  )
  
  # Check if we have a custom remediation prompt
  if [[ -f "$REMEDIATION_PROMPT" ]]; then
    log "Using custom remediation prompt from $REMEDIATION_PROMPT"
    local custom_prompt
    custom_prompt=$(cat "$REMEDIATION_PROMPT")
    failure_context="${custom_prompt}

${failure_context}"
  fi
  
  append_progress "### Remediation Agent Spawned
- Phase: ${phase}
- Exit Code: ${exit_code}
- Consecutive Failures: ${consecutive_failures}"
  
  # Spawn Claude to investigate and fix
  log "Running remediation agent..."
  local remediation_log="${LOG_DIR}/remediation_$(date -u '+%Y%m%d_%H%M%S').log"
  mkdir -p "$LOG_DIR"
  
  if echo "$failure_context" | claude --dangerously-skip-permissions -p - > "$remediation_log" 2>&1; then
    log "Remediation agent completed successfully"
    append_progress "- Remediation agent completed (see ${remediation_log})"
  else
    local rem_exit=$?
    log "Remediation agent exited with code $rem_exit"
    append_progress "- Remediation agent exited with code ${rem_exit} (see ${remediation_log})"
  fi
}

# Send escalation notification to Linear
send_escalation() {
  local message="$1"
  log "ESCALATION: $message"
  append_progress "### ⚠️ ESCALATION
${message}

Ralph has failed ${consecutive_failures} times consecutively without making progress.
Manual intervention may be required."
  
  # TODO: Implement Linear notification via API
  # For now, just log it prominently
}

# Check if lifecycle is complete
check_complete() {
  local result
  result=$("${ROOT_DIR}/scripts/2026-01/ralph-cto.sh" --config "$CONFIG_PATH" --check-complete 2>/dev/null || echo '{"complete": false}')
  echo "$result" | jq -e '.complete == true' >/dev/null 2>&1
}

# Main infinite loop
main() {
  log_separator
  log "Ralph Infinite Autonomous Loop starting"
  log "Config: $CONFIG_PATH"
  log "State: $STATE_PATH"
  log "Remediation: $([ $ENABLE_REMEDIATION -eq 1 ] && echo 'enabled' || echo 'disabled')"
  log "Cooldown: ${COOLDOWN_SECONDS}s (failure), ${SUCCESS_COOLDOWN_SECONDS}s (success)"
  log "Max consecutive failures before escalation: $MAX_CONSECUTIVE_FAILURES"
  log_separator
  
  append_progress "### Ralph Infinite Loop Started
- Remediation: $([ $ENABLE_REMEDIATION -eq 1 ] && echo 'enabled' || echo 'disabled')
- Cooldown: ${COOLDOWN_SECONDS}s / ${SUCCESS_COOLDOWN_SECONDS}s
- Max consecutive failures: ${MAX_CONSECUTIVE_FAILURES}"
  
  while true; do
    total_cycles=$((total_cycles + 1))
    log_separator
    log "Cycle #${total_cycles} starting (successes: ${total_successes}, failures: ${total_failures}, consecutive: ${consecutive_failures})"
    
    # Run one iteration of ralph-cto.sh
    local exit_code=0
    "${ROOT_DIR}/scripts/2026-01/ralph-cto.sh" --config "$CONFIG_PATH" --attended || exit_code=$?
    
    if [[ $exit_code -eq 0 ]]; then
      log "Phase completed successfully"
      consecutive_failures=0
      
      # Check if all phases are complete
      if check_complete; then
        total_successes=$((total_successes + 1))
        log "🎉 Full lifecycle complete! (total successes: ${total_successes})"
        append_progress "### 🎉 Full Lifecycle Complete
- Cycle: #${total_cycles}
- Total Successes: ${total_successes}
- Resetting for next iteration..."
        
        # Reset for next full run
        run_cleanup
        reset_to_intake
        
        log "Cooldown for ${SUCCESS_COOLDOWN_SECONDS}s before next lifecycle..."
        sleep "$SUCCESS_COOLDOWN_SECONDS"
      else
        # Phase passed but not all complete - continue to next phase
        log "Phase passed, continuing to next phase..."
        # Small delay between phases
        sleep 5
      fi
    else
      total_failures=$((total_failures + 1))
      consecutive_failures=$((consecutive_failures + 1))
      
      log "Phase failed (exit code: ${exit_code}, consecutive failures: ${consecutive_failures})"
      append_progress "### Phase Failed
- Exit Code: ${exit_code}
- Consecutive Failures: ${consecutive_failures}
- Total Failures: ${total_failures}"
      
      # Check if we should escalate
      if [[ $consecutive_failures -ge $MAX_CONSECUTIVE_FAILURES ]]; then
        send_escalation "Ralph has failed ${consecutive_failures} times consecutively"
        # Reset counter after escalation but continue trying
        consecutive_failures=0
      fi
      
      # Spawn remediation agent if enabled
      if [[ $ENABLE_REMEDIATION -eq 1 ]]; then
        spawn_remediation_agent "$exit_code"
      else
        log "Remediation disabled, skipping..."
      fi
      
      # Cleanup and reset
      run_cleanup
      reset_to_intake
      
      log "Cooldown for ${COOLDOWN_SECONDS}s before retry..."
      sleep "$COOLDOWN_SECONDS"
    fi
  done
}

# Handle signals for graceful shutdown
shutdown() {
  log_separator
  log "Received shutdown signal"
  log "Final stats: cycles=${total_cycles}, successes=${total_successes}, failures=${total_failures}"
  append_progress "### Ralph Infinite Loop Stopped
- Total Cycles: ${total_cycles}
- Total Successes: ${total_successes}
- Total Failures: ${total_failures}
- Stopped at: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  exit 0
}

trap shutdown SIGINT SIGTERM

main
