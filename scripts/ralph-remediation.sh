#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# ralph-remediation.sh - Remediation Agent for Dual Ralph Self-Healing System
# =============================================================================
# Uses Claude for investigating and fixing issues detected by the monitor.
# Watches the coordination file for pending issues and resolves them.
# =============================================================================

ROOT_DIR="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONFIG_PATH="${RALPH_CONFIG:-${ROOT_DIR}/lifecycle-test/ralph-cto.json}"
COORDINATION_FILE="${ROOT_DIR}/lifecycle-test/ralph-coordination.json"

# Source the response analyzer library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ralph-response-analyzer.sh
source "${SCRIPT_DIR}/ralph-response-analyzer.sh"

# Parse configuration
REMEDIATION_COMMAND=$(jq -r '.dualAgent.remediation.command // "claude --dangerously-skip-permissions -p"' "$CONFIG_PATH")
REMEDIATION_PROMPT_PATH="${ROOT_DIR}/$(jq -r '.dualAgent.remediation.promptPath // "lifecycle-test/remediation-prompt.md"' "$CONFIG_PATH")"
POLL_INTERVAL=$(jq -r '.dualAgent.coordination.pollIntervalSeconds // 10' "$CONFIG_PATH")

# Exit detection configuration
REQUIRE_EXPLICIT_SIGNAL=$(jq -r '.dualAgent.exitDetection.requireExplicitSignal // true' "$CONFIG_PATH")
COMPLETION_INDICATOR_THRESHOLD=$(jq -r '.dualAgent.exitDetection.completionIndicatorThreshold // 2' "$CONFIG_PATH")

# Paths from config
PROGRESS_PATH="${ROOT_DIR}/$(jq -r '.paths.progressPath // "lifecycle-test/progress.txt"' "$CONFIG_PATH")"
REPORT_PATH="${ROOT_DIR}/$(jq -r '.paths.reportPath // "lifecycle-test/report.json"' "$CONFIG_PATH")"
LOG_DIR="${ROOT_DIR}/lifecycle-test/ralph-logs"

log() { echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [REMEDIATION] $*" >&2; }

# Ensure coordination file exists
ensure_coordination_file() {
  if [[ ! -f "$COORDINATION_FILE" ]]; then
    cat > "$COORDINATION_FILE" <<'EOF'
{
  "monitor": {
    "status": "stopped",
    "currentPhase": null,
    "lastCheck": null,
    "pid": null
  },
  "remediation": {
    "status": "idle",
    "currentIssue": null,
    "pid": null
  },
  "issueQueue": [],
  "stats": {
    "totalIssues": 0,
    "resolved": 0,
    "failed": 0
  }
}
EOF
  fi
}

# Update remediation status in coordination file
update_remediation_status() {
  local status="$1"
  local issue_id="${2:-null}"
  local tmp
  tmp="$(mktemp)"
  jq --arg status "$status" \
     --arg issue "$issue_id" \
     --arg pid "$$" \
     '.remediation.status = $status | 
      .remediation.currentIssue = (if $issue == "null" then null else $issue end) |
      .remediation.pid = ($pid | tonumber)' \
     "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
}

# Get the next pending issue from the queue
get_next_pending_issue() {
  jq -r '[.issueQueue[] | select(.status == "pending")] | sort_by(.timestamp) | first | .id // empty' "$COORDINATION_FILE"
}

# Get issue details
get_issue() {
  local issue_id="$1"
  jq -c --arg id "$issue_id" '.issueQueue[] | select(.id == $id)' "$COORDINATION_FILE"
}

# Claim an issue (set status to claimed)
claim_issue() {
  local issue_id="$1"
  local tmp
  tmp="$(mktemp)"
  jq --arg id "$issue_id" \
     '(.issueQueue[] | select(.id == $id)).status = "claimed" |
      (.issueQueue[] | select(.id == $id)).claimedAt = (now | todate) |
      (.issueQueue[] | select(.id == $id)).claimedBy = "remediation"' \
     "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  log "Claimed issue: $issue_id"
}

# Update issue status
update_issue_status() {
  local issue_id="$1"
  local status="$2"
  local resolution="${3:-}"
  local tmp
  tmp="$(mktemp)"
  
  jq --arg id "$issue_id" \
     --arg status "$status" \
     --arg resolution "$resolution" \
     '(.issueQueue[] | select(.id == $id)).status = $status |
      (.issueQueue[] | select(.id == $id)).resolvedAt = (now | todate) |
      (.issueQueue[] | select(.id == $id)).resolution = $resolution |
      if $status == "resolved" then .stats.resolved += 1
      elif $status == "failed" then .stats.failed += 1
      else . end' \
     "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  
  log "Updated issue $issue_id status to: $status"
}

# Increment retry count for an issue
increment_retry_count() {
  local issue_id="$1"
  local tmp
  tmp="$(mktemp)"
  jq --arg id "$issue_id" \
     '(.issueQueue[] | select(.id == $id)).retryCount += 1' \
     "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
}

# Append progress
append_progress() {
  local message="$1"
  {
    echo ""
    echo "## $(date -u '+%Y-%m-%d %H:%M:%SZ') [REMEDIATION]"
    echo "$message"
  } >> "$PROGRESS_PATH"
}

# Append report
append_report() {
  local entry="$1"
  local tmp
  tmp="$(mktemp)"
  if [[ ! -f "$REPORT_PATH" ]]; then
    echo "[]" > "$REPORT_PATH"
  fi
  jq ". + [$entry]" "$REPORT_PATH" > "$tmp" && mv "$tmp" "$REPORT_PATH"
}

# Build remediation prompt for an issue
build_remediation_prompt() {
  local issue_id="$1"
  local issue_json
  issue_json="$(get_issue "$issue_id")"
  
  local phase gate log_file exit_code diagnostics
  phase=$(echo "$issue_json" | jq -r '.phase')
  gate=$(echo "$issue_json" | jq -r '.gate')
  log_file=$(echo "$issue_json" | jq -r '.logFile')
  exit_code=$(echo "$issue_json" | jq -r '.exitCode')
  diagnostics=$(echo "$issue_json" | jq -r '.diagnostics // {}')
  
  # Get gate command from config
  local gate_command
  gate_command=$(jq -r --arg phase "$phase" --arg gate "$gate" \
    '.phases[] | select(.id == $phase) | .gates[] | select(.name == $gate) | .command' \
    "$CONFIG_PATH")
  
  # Read log file contents (last 100 lines)
  local log_content=""
  if [[ -f "$log_file" ]]; then
    log_content=$(tail -100 "$log_file" 2>/dev/null || echo "Could not read log file")
  fi
  
  # Build the prompt
  cat <<EOF
# Remediation Task

## Issue Details
- **Issue ID**: $issue_id
- **Phase**: $phase
- **Gate**: $gate
- **Exit Code**: $exit_code

## Gate Command
\`\`\`bash
$gate_command
\`\`\`

## Gate Log Output
\`\`\`
$log_content
\`\`\`

## Diagnostics
\`\`\`json
$diagnostics
\`\`\`

## Instructions

You are the REMEDIATION agent. A gate check has failed and you need to:

1. **Analyze** the failure from the log output above
2. **Investigate** the root cause - check related services, logs, configs
3. **Fix** the issue - make code/config changes as needed
4. **Verify** your fix by running the gate command

### Important Guidelines

- Focus on fixing the immediate issue, not refactoring unrelated code
- If the issue is infrastructure-related (pod crashed, service down), check:
  - \`kubectl get pods -n cto\`
  - \`kubectl logs <pod> -n cto\`
  - Service health endpoints
- If the issue is code-related, find and fix the bug
- After making changes, run the gate command to verify the fix works:
  \`\`\`bash
  $gate_command
  \`\`\`
- Document your investigation and fix in lifecycle-test/progress.txt

### Success Criteria

The remediation is successful if the gate command exits with status 0.
EOF
}

# Run remediation for an issue
run_remediation() {
  local issue_id="$1"
  local issue_json
  issue_json="$(get_issue "$issue_id")"
  
  local phase gate
  phase=$(echo "$issue_json" | jq -r '.phase')
  gate=$(echo "$issue_json" | jq -r '.gate')
  
  log "Starting remediation for issue $issue_id (phase=$phase, gate=$gate)"
  append_progress "Starting remediation for $phase/$gate (issue: $issue_id)"
  
  # Build the remediation task
  local task_file
  task_file="$(mktemp)"
  
  # Combine base prompt with issue-specific details
  {
    if [[ -f "$REMEDIATION_PROMPT_PATH" ]]; then
      cat "$REMEDIATION_PROMPT_PATH"
      echo ""
      echo "---"
      echo ""
    fi
    build_remediation_prompt "$issue_id"
  } > "$task_file"
  
  # Run remediation agent and capture output
  log "Invoking remediation agent..."
  local start_time
  start_time=$(date +%s)
  
  local agent_output_file
  agent_output_file="${LOG_DIR}/remediation_output_${issue_id}_$(date -u '+%Y%m%d_%H%M%S').log"
  mkdir -p "$LOG_DIR"
  
  set +e
  # Check if using droid (needs -f file) or claude (needs piped input)
  if [[ "$REMEDIATION_COMMAND" == *"droid"* ]]; then
    # droid uses -f <file> syntax
    $REMEDIATION_COMMAND "$task_file" 2>&1 | tee "$agent_output_file"
  else
    # claude uses piped input with -p -
    cat "$task_file" | $REMEDIATION_COMMAND - 2>&1 | tee "$agent_output_file"
  fi
  local exit_code=$?
  set -e
  
  local elapsed
  elapsed=$(($(date +%s) - start_time))
  
  rm -f "$task_file"
  
  # Analyze agent output for EXIT_SIGNAL and struggle detection
  local agent_output=""
  if [[ -f "$agent_output_file" ]]; then
    agent_output=$(cat "$agent_output_file")
    
    # Check for struggle patterns using library function
    if ra_detect_struggle "$agent_output"; then
      log "Agent appears to be struggling - flagging for attention"
      append_progress "Warning: Agent may be struggling with $phase/$gate"
    fi
    
    # Check exit condition using library function (not used for remediation but for logging)
    local exit_decision
    exit_decision=$(ra_should_exit_gracefully "$agent_output" "$COMPLETION_INDICATOR_THRESHOLD" "$REQUIRE_EXPLICIT_SIGNAL")
    log "Exit decision from output analysis: $exit_decision"
    
    # Log full analysis for debugging
    ra_debug_analysis "$agent_output"
  fi
  
  # Check if remediation succeeded by re-running the gate
  local gate_command
  gate_command=$(jq -r --arg phase "$phase" --arg gate "$gate" \
    '.phases[] | select(.id == $phase) | .gates[] | select(.name == $gate) | .command' \
    "$CONFIG_PATH")
  
  log "Verifying fix by re-running gate: $gate"
  local verify_log
  verify_log="${LOG_DIR}/remediation_verify_${issue_id}_$(date -u '+%Y%m%d_%H%M%S').log"
  mkdir -p "$LOG_DIR"
  
  set +e
  bash -c "$gate_command" >"$verify_log" 2>&1
  local verify_status=$?
  set -e
  
  if [[ $verify_status -eq 0 ]]; then
    log "Remediation successful for $issue_id (gate passed)"
    update_issue_status "$issue_id" "resolved" "Gate passed after remediation"
    append_progress "Remediation successful for $phase/$gate (took ${elapsed}s)"
    append_report "$(jq -n \
      --arg issue "$issue_id" \
      --arg phase "$phase" \
      --arg gate "$gate" \
      --arg elapsed "$elapsed" \
      '{timestamp: (now | todate), type: "remediation", issue: $issue, phase: $phase, gate: $gate, status: "resolved", elapsedSeconds: ($elapsed | tonumber), source: "remediation"}')"
    return 0
  else
    log "Remediation failed for $issue_id (gate still failing)"
    increment_retry_count "$issue_id"
    update_issue_status "$issue_id" "failed" "Gate still failing after remediation attempt"
    append_progress "Remediation failed for $phase/$gate (gate still failing)"
    append_report "$(jq -n \
      --arg issue "$issue_id" \
      --arg phase "$phase" \
      --arg gate "$gate" \
      --arg elapsed "$elapsed" \
      --arg verify_log "$verify_log" \
      '{timestamp: (now | todate), type: "remediation", issue: $issue, phase: $phase, gate: $gate, status: "failed", elapsedSeconds: ($elapsed | tonumber), verifyLog: $verify_log, source: "remediation"}')"
    return 1
  fi
}

# Main remediation loop
main_loop() {
  log "Starting remediation agent (PID: $$)"
  ensure_coordination_file
  
  update_remediation_status "idle"
  append_progress "Remediation agent started (PID: $$)"
  
  while true; do
    # Check for pending issues
    local pending_issue
    pending_issue=$(get_next_pending_issue)
    
    if [[ -n "$pending_issue" ]]; then
      log "Found pending issue: $pending_issue"
      
      # Claim the issue
      claim_issue "$pending_issue"
      update_remediation_status "working" "$pending_issue"
      
      # Run remediation
      set +e
      run_remediation "$pending_issue"
      local result=$?
      set -e
      
      update_remediation_status "idle"
      
      if [[ $result -ne 0 ]]; then
        log "Remediation attempt failed for $pending_issue"
        # The issue status is already updated in run_remediation
        # Continue to next iteration to allow monitor to possibly create a new issue
      fi
    else
      # No pending issues, wait and check again
      update_remediation_status "idle"
    fi
    
    sleep "$POLL_INTERVAL"
  done
}

# Handle signals for graceful shutdown
cleanup() {
  log "Shutting down remediation agent..."
  update_remediation_status "stopped"
  exit 0
}

trap cleanup SIGTERM SIGINT

# Main entry point
main_loop
