#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# ralph-monitor.sh - Monitor Agent for Dual Ralph Self-Healing System
# =============================================================================
# Uses GPT-5.2 (via droid) for systematic gate checking and failure detection.
# Does NOT attempt fixes - writes failures to coordination file for remediation.
# =============================================================================

ROOT_DIR="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONFIG_PATH="${RALPH_CONFIG:-${ROOT_DIR}/lifecycle-test/ralph-cto.json}"
COORDINATION_FILE="${ROOT_DIR}/lifecycle-test/ralph-coordination.json"

# Source the response analyzer library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ralph-response-analyzer.sh
source "${SCRIPT_DIR}/ralph-response-analyzer.sh"

# Parse configuration
MONITOR_COMMAND=$(jq -r '.dualAgent.monitor.command // "droid exec --model gpt-5.2 --auto medium -f"' "$CONFIG_PATH")
MONITOR_PROMPT_PATH="${ROOT_DIR}/$(jq -r '.dualAgent.monitor.promptPath // "lifecycle-test/monitor-prompt.md"' "$CONFIG_PATH")"
POLL_INTERVAL=$(jq -r '.dualAgent.coordination.pollIntervalSeconds // 10' "$CONFIG_PATH")
MAX_RETRY_AFTER_REMEDIATION=$(jq -r '.dualAgent.coordination.maxRetryAfterRemediation // 3' "$CONFIG_PATH")

# Circuit breaker configuration
CB_NO_PROGRESS_THRESHOLD=$(jq -r '.dualAgent.circuitBreaker.noProgressThreshold // 3' "$CONFIG_PATH")
CB_SAME_ERROR_THRESHOLD=$(jq -r '.dualAgent.circuitBreaker.sameErrorThreshold // 5' "$CONFIG_PATH")
CB_RECOVERY_MINUTES=$(jq -r '.dualAgent.circuitBreaker.recoveryMinutes // 5' "$CONFIG_PATH")

# Session configuration
SESSION_EXPIRATION_HOURS=$(jq -r '.dualAgent.session.expirationHours // 24' "$CONFIG_PATH")
SESSION_AUTO_RESET_ON_CB=$(jq -r '.dualAgent.session.autoResetOnCircuitBreak // true' "$CONFIG_PATH")

# Paths from config
STATE_PATH="${ROOT_DIR}/$(jq -r '.paths.statePath // "lifecycle-test/ralph-cto.state.json"' "$CONFIG_PATH")"
PROGRESS_PATH="${ROOT_DIR}/$(jq -r '.paths.progressPath // "lifecycle-test/progress.txt"' "$CONFIG_PATH")"
REPORT_PATH="${ROOT_DIR}/$(jq -r '.paths.reportPath // "lifecycle-test/report.json"' "$CONFIG_PATH")"
OBJECTIVE_PATH="${ROOT_DIR}/$(jq -r '.paths.objectivePath // "lifecycle-test/current-objective.md"' "$CONFIG_PATH")"
LOG_DIR="${ROOT_DIR}/lifecycle-test/ralph-logs"

log() { echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [MONITOR] $*" >&2; }

# Safe atomic update of coordination file with error handling
# Usage: safe_update_coordination '.path = "value"' or pipe jq expression
safe_update_coordination() {
  local jq_expr="$1"
  shift
  local tmp
  tmp="$(mktemp)"
  
  if ! jq "$jq_expr" "$@" "$COORDINATION_FILE" > "$tmp"; then
    log "ERROR: Failed to update coordination file (jq failed)"
    rm -f "$tmp"
    return 1
  fi
  
  if ! mv "$tmp" "$COORDINATION_FILE"; then
    log "ERROR: Failed to update coordination file (mv failed)"
    rm -f "$tmp"
    return 1
  fi
  
  return 0
}

# Portable ISO8601 date parsing (works on both macOS and Linux)
# Returns epoch seconds or 0 on failure
parse_iso8601_to_epoch() {
  local date_str="$1"
  local epoch
  
  # Try macOS BSD date first
  epoch=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$date_str" "+%s" 2>/dev/null) && echo "$epoch" && return 0
  
  # Try GNU date (Linux)
  epoch=$(date -d "$date_str" "+%s" 2>/dev/null) && echo "$epoch" && return 0
  
  # Try Python as last resort (most portable)
  epoch=$(python3 -c "from datetime import datetime; print(int(datetime.fromisoformat('${date_str}'.replace('Z', '+00:00')).timestamp()))" 2>/dev/null) && echo "$epoch" && return 0
  
  # Failed to parse
  log "WARNING: Failed to parse date: $date_str"
  echo "0"
  return 1
}

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
  
  # Ensure circuitBreaker section exists
  if ! jq -e '.circuitBreaker' "$COORDINATION_FILE" >/dev/null 2>&1; then
    local tmp
    tmp="$(mktemp)"
    jq '. + {
      "circuitBreaker": {
        "state": "closed",
        "noProgressCount": 0,
        "sameErrorCount": 0,
        "lastError": null,
        "lastFileChange": null,
        "openedAt": null
      }
    }' "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  fi
  
  # Ensure session section exists
  if ! jq -e '.session' "$COORDINATION_FILE" >/dev/null 2>&1; then
    local tmp
    tmp="$(mktemp)"
    jq '. + {
      "session": {
        "id": null,
        "startedAt": null,
        "lastActivity": null,
        "expirationHours": 24
      }
    }' "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  fi
}

# =============================================================================
# Session Management Functions
# =============================================================================

# Generate a new session ID
generate_session_id() {
  echo "ralph-$(date +%s)-$$-$(head -c 4 /dev/urandom | xxd -p)"
}

# Get current session ID
get_session_id() {
  jq -r '.session.id // empty' "$COORDINATION_FILE"
}

# Check if session is expired
is_session_expired() {
  local started_at
  started_at=$(jq -r '.session.startedAt // null' "$COORDINATION_FILE")
  
  if [[ "$started_at" == "null" || -z "$started_at" ]]; then
    return 0  # No session, treat as expired
  fi
  
  local now
  local started_epoch
  now=$(date +%s)
  started_epoch=$(parse_iso8601_to_epoch "$started_at")
  
  local elapsed_hours
  elapsed_hours=$(( (now - started_epoch) / 3600 ))
  
  if [[ $elapsed_hours -ge $SESSION_EXPIRATION_HOURS ]]; then
    log "Session expired: ${elapsed_hours}h >= ${SESSION_EXPIRATION_HOURS}h"
    return 0
  fi
  
  return 1
}

# Start a new session
start_session() {
  local session_id
  session_id=$(generate_session_id)
  
  if ! safe_update_coordination \
     '.session.id = $id |
      .session.startedAt = (now | todate) |
      .session.lastActivity = (now | todate) |
      .session.expirationHours = ($hours | tonumber)' \
     --arg id "$session_id" \
     --arg hours "$SESSION_EXPIRATION_HOURS"; then
    log "ERROR: Failed to start session"
    return 1
  fi
  
  # Increment session reset count
  increment_session_resets
  
  log "Started new session: $session_id"
  echo "$session_id"
}

# Update session activity timestamp
update_session_activity() {
  safe_update_coordination '.session.lastActivity = (now | todate)'
}

# Reset session (clear and start new)
reset_session() {
  log "Resetting session..."
  
  # Reset circuit breaker if configured
  if [[ "$SESSION_AUTO_RESET_ON_CB" == "true" ]]; then
    reset_circuit_breaker
  fi
  
  start_session
}

# Increment session reset count in stats
increment_session_resets() {
  safe_update_coordination '.stats.sessionResets = ((.stats.sessionResets // 0) + 1)'
}

# Ensure valid session (start new if needed)
ensure_valid_session() {
  local session_id
  session_id=$(get_session_id)
  
  if [[ -z "$session_id" ]]; then
    log "No session found, starting new session"
    start_session
    return
  fi
  
  if is_session_expired; then
    log "Session expired, starting new session"
    start_session
    return
  fi
  
  # Update activity timestamp
  update_session_activity
}

# =============================================================================
# End Session Management Functions
# =============================================================================

# =============================================================================
# Circuit Breaker Functions
# =============================================================================

# Get circuit breaker state
get_circuit_breaker_state() {
  jq -r '.circuitBreaker.state // "closed"' "$COORDINATION_FILE"
}

# Check if circuit breaker is open (returns 0 if open, 1 if closed/half-open)
is_circuit_breaker_open() {
  local state
  state=$(get_circuit_breaker_state)
  
  if [[ "$state" == "open" ]]; then
    # Check if recovery time has passed for half-open transition
    local opened_at
    opened_at=$(jq -r '.circuitBreaker.openedAt // null' "$COORDINATION_FILE")
    
    if [[ "$opened_at" != "null" ]]; then
      local now
      local opened_epoch
      now=$(date +%s)
      opened_epoch=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$opened_at" "+%s" 2>/dev/null || date -d "$opened_at" "+%s" 2>/dev/null || echo 0)
      
      local elapsed_minutes
      elapsed_minutes=$(( (now - opened_epoch) / 60 ))
      
      if [[ $elapsed_minutes -ge $CB_RECOVERY_MINUTES ]]; then
        log "Circuit breaker recovery time reached, transitioning to half-open"
        set_circuit_breaker_state "half-open"
        return 1
      fi
    fi
    return 0
  fi
  return 1
}

# Set circuit breaker state
set_circuit_breaker_state() {
  local state="$1"
  
  local opened_at="null"
  if [[ "$state" == "open" ]]; then
    opened_at="\"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\""
  fi
  
  if ! safe_update_coordination \
     '.circuitBreaker.state = $state |
      .circuitBreaker.openedAt = $opened_at' \
     --arg state "$state" \
     --argjson opened_at "$opened_at"; then
    log "ERROR: Failed to set circuit breaker state"
    return 1
  fi
  
  log "Circuit breaker state set to: $state"
}

# Update circuit breaker after a failure
update_circuit_breaker_failure() {
  local error_msg="$1"
  
  # Get current state
  local last_error
  local same_error_count
  local no_progress_count
  
  last_error=$(jq -r '.circuitBreaker.lastError // ""' "$COORDINATION_FILE")
  same_error_count=$(jq -r '.circuitBreaker.sameErrorCount // 0' "$COORDINATION_FILE")
  no_progress_count=$(jq -r '.circuitBreaker.noProgressCount // 0' "$COORDINATION_FILE")
  
  # Check if same error
  if [[ "$error_msg" == "$last_error" ]]; then
    same_error_count=$((same_error_count + 1))
  else
    same_error_count=1
  fi
  
  # Increment no progress count
  no_progress_count=$((no_progress_count + 1))
  
  # Update coordination file
  if ! safe_update_coordination \
     '.circuitBreaker.lastError = $error |
      .circuitBreaker.sameErrorCount = $same_count |
      .circuitBreaker.noProgressCount = $no_progress' \
     --arg error "$error_msg" \
     --argjson same_count "$same_error_count" \
     --argjson no_progress "$no_progress_count"; then
    log "ERROR: Failed to update circuit breaker failure state"
    return 1
  fi
  
  log "Circuit breaker: noProgressCount=$no_progress_count, sameErrorCount=$same_error_count"
  
  # Check thresholds
  if [[ $no_progress_count -ge $CB_NO_PROGRESS_THRESHOLD ]]; then
    log "Circuit breaker OPEN: No progress threshold ($CB_NO_PROGRESS_THRESHOLD) reached"
    set_circuit_breaker_state "open"
    increment_circuit_breaker_trips
    return 0
  fi
  
  if [[ $same_error_count -ge $CB_SAME_ERROR_THRESHOLD ]]; then
    log "Circuit breaker OPEN: Same error threshold ($CB_SAME_ERROR_THRESHOLD) reached"
    set_circuit_breaker_state "open"
    increment_circuit_breaker_trips
    return 0
  fi
  
  return 1
}

# Update circuit breaker after success (progress detected)
update_circuit_breaker_success() {
  local state
  state=$(get_circuit_breaker_state)
  
  # If half-open and success, close the circuit
  if [[ "$state" == "half-open" ]]; then
    log "Circuit breaker: Success in half-open state, closing circuit"
    set_circuit_breaker_state "closed"
  fi
  
  # Reset counters
  safe_update_coordination \
     '.circuitBreaker.noProgressCount = 0 |
      .circuitBreaker.sameErrorCount = 0 |
      .circuitBreaker.lastFileChange = (now | todate)'
}

# Reset circuit breaker
reset_circuit_breaker() {
  if ! safe_update_coordination \
     '.circuitBreaker = {
        "state": "closed",
        "noProgressCount": 0,
        "sameErrorCount": 0,
        "lastError": null,
        "lastFileChange": (now | todate),
        "openedAt": null
      }'; then
    log "ERROR: Failed to reset circuit breaker"
    return 1
  fi
  
  log "Circuit breaker reset"
}

# Increment circuit breaker trip count in stats
increment_circuit_breaker_trips() {
  safe_update_coordination '.stats.circuitBreakerTrips = ((.stats.circuitBreakerTrips // 0) + 1)'
}

# =============================================================================
# End Circuit Breaker Functions
# =============================================================================

# Update monitor status in coordination file
update_monitor_status() {
  local status="$1"
  local phase="${2:-null}"
  
  safe_update_coordination \
     '.monitor.status = $status | 
      .monitor.currentPhase = (if $phase == "null" then null else $phase end) | 
      .monitor.lastCheck = (now | todate) |
      .monitor.pid = ($pid | tonumber)' \
     --arg status "$status" \
     --arg phase "$phase" \
     --arg pid "$$"
}

# Write failure to issue queue
write_failure_to_queue() {
  local phase="$1"
  local gate="$2"
  local log_file="$3"
  local exit_code="${4:-1}"
  local diagnostics="${5:-{}}"
  
  local issue_id
  issue_id="issue-$(date +%s)-$$"
  
  if ! safe_update_coordination \
     '.issueQueue += [{
        id: $id,
        timestamp: (now | todate),
        phase: $phase,
        gate: $gate,
        exitCode: ($exit | tonumber),
        logFile: $log,
        diagnostics: $diag,
        status: "pending",
        retryCount: 0
      }] | 
      .stats.totalIssues += 1' \
     --arg id "$issue_id" \
     --arg phase "$phase" \
     --arg gate "$gate" \
     --arg log "$log_file" \
     --arg exit "$exit_code" \
     --argjson diag "$diagnostics"; then
    log "ERROR: Failed to write failure to queue"
    return 1
  fi
  
  log "Wrote failure to queue: $issue_id (phase=$phase, gate=$gate)"
  echo "$issue_id"
}

# Check if there's a pending/claimed issue for a gate
has_active_issue_for_gate() {
  local phase="$1"
  local gate="$2"
  jq -e --arg phase "$phase" --arg gate "$gate" \
    '.issueQueue[] | select(.phase == $phase and .gate == $gate and (.status == "pending" or .status == "claimed"))' \
    "$COORDINATION_FILE" >/dev/null 2>&1
}

# Check if remediation resolved the issue
is_issue_resolved() {
  local phase="$1"
  local gate="$2"
  jq -e --arg phase "$phase" --arg gate "$gate" \
    '.issueQueue[] | select(.phase == $phase and .gate == $gate and .status == "resolved")' \
    "$COORDINATION_FILE" >/dev/null 2>&1
}

# Get retry count for a gate
get_gate_retry_count() {
  local phase="$1"
  local gate="$2"
  jq -r --arg phase "$phase" --arg gate "$gate" \
    '[.issueQueue[] | select(.phase == $phase and .gate == $gate)] | map(.retryCount) | max // 0' \
    "$COORDINATION_FILE"
}

# Wait for remediation to complete
wait_for_remediation() {
  local phase="$1"
  local gate="$2"
  local timeout="${3:-600}"
  local start_time
  start_time=$(date +%s)
  
  log "Waiting for remediation of $phase/$gate (timeout: ${timeout}s)"
  
  while true; do
    local elapsed
    elapsed=$(($(date +%s) - start_time))
    
    if [[ $elapsed -ge $timeout ]]; then
      log "Remediation timeout for $phase/$gate after ${elapsed}s"
      return 1
    fi
    
    # Check if issue was resolved
    local issue_status
    issue_status=$(jq -r --arg phase "$phase" --arg gate "$gate" \
      '[.issueQueue[] | select(.phase == $phase and .gate == $gate)] | sort_by(.timestamp) | last | .status // "unknown"' \
      "$COORDINATION_FILE")
    
    case "$issue_status" in
      resolved)
        log "Remediation completed successfully for $phase/$gate"
        return 0
        ;;
      failed)
        log "Remediation failed for $phase/$gate"
        return 1
        ;;
      pending|claimed)
        # Still being worked on
        sleep "$POLL_INTERVAL"
        ;;
      *)
        log "Unknown issue status: $issue_status"
        sleep "$POLL_INTERVAL"
        ;;
    esac
  done
}

# Run a single gate command
run_gate() {
  local phase_id="$1"
  local gate_name="$2"
  local gate_command="$3"
  local log_path
  log_path="${LOG_DIR}/${phase_id}_${gate_name}_$(date -u '+%Y%m%d_%H%M%S').log"
  mkdir -p "$LOG_DIR"
  
  log "Running gate: $gate_name"
  set +e
  bash -c "$gate_command" >"$log_path" 2>&1
  local status=$?
  set -e
  
  echo "$log_path|$status"
}

# Get phase JSON from config
get_phase_json() {
  local phase_id="$1"
  jq -c --arg id "$phase_id" '.phases[] | select(.id == $id)' "$CONFIG_PATH"
}

# Append progress
append_progress() {
  local message="$1"
  {
    echo ""
    echo "## $(date -u '+%Y-%m-%d %H:%M:%SZ') [MONITOR]"
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

# Initialize state file
init_state() {
  if [[ ! -s "$STATE_PATH" ]]; then
    cat > "$STATE_PATH" <<'EOF'
{
  "phase": "intake",
  "attempts": {},
  "completedObjectives": [],
  "attendedCompleted": 0,
  "last_success": null
}
EOF
  fi
}

# Get current phase from state
get_current_phase() {
  jq -r '.phase // "intake"' "$STATE_PATH"
}

# Set phase in state
state_set_phase() {
  local phase="$1"
  local tmp
  tmp="$(mktemp)"
  jq --arg phase "$phase" '.phase = $phase' "$STATE_PATH" > "$tmp" && mv "$tmp" "$STATE_PATH"
}

# Mark phase as completed in state
state_add_completed_objective() {
  local objective="$1"
  local tmp
  tmp="$(mktemp)"
  jq --arg objective "$objective" '
    .completedObjectives = (.completedObjectives // []) |
    if (.completedObjectives | index($objective)) then . else .completedObjectives += [$objective] end
  ' "$STATE_PATH" > "$tmp" && mv "$tmp" "$STATE_PATH"
}

# Check if phase is completed
is_phase_completed() {
  local phase="$1"
  jq -e --arg phase "$phase" '.completedObjectives | index($phase)' "$STATE_PATH" >/dev/null 2>&1
}

# Run all gates for a phase
run_phase_gates() {
  local phase_id="$1"
  local phase_json
  phase_json="$(get_phase_json "$phase_id")"
  
  if [[ -z "$phase_json" ]]; then
    log "Phase not found: $phase_id"
    return 1
  fi
  
  local gates_json
  gates_json="$(echo "$phase_json" | jq -c '.gates // null')"
  
  if [[ "$gates_json" == "null" ]]; then
    log "No gates for phase: $phase_id"
    return 0
  fi
  
  update_monitor_status "checking" "$phase_id"
  append_progress "Starting gate checks for phase: $phase_id"
  
  while read -r gate; do
    local gate_name
    local gate_command
    local gate_optional
    
    gate_name="$(echo "$gate" | jq -r '.name')"
    gate_command="$(echo "$gate" | jq -r '.command')"
    gate_optional="$(echo "$gate" | jq -r '.optional // false')"
    
    # Check if there's already a pending issue for this gate
    if has_active_issue_for_gate "$phase_id" "$gate_name"; then
      log "Active issue exists for $gate_name, waiting for remediation..."
      
      local remediation_timeout
      remediation_timeout=$(jq -r '.dualAgent.coordination.remediationTimeoutSeconds // 600' "$CONFIG_PATH")
      
      if wait_for_remediation "$phase_id" "$gate_name" "$remediation_timeout"; then
        log "Remediation complete, retrying gate: $gate_name"
      else
        log "Remediation failed or timed out for $gate_name"
        
        # Check retry count
        local retry_count
        retry_count=$(get_gate_retry_count "$phase_id" "$gate_name")
        
        if [[ $retry_count -ge $MAX_RETRY_AFTER_REMEDIATION ]]; then
          log "Max retries ($MAX_RETRY_AFTER_REMEDIATION) reached for $gate_name, skipping phase"
          append_progress "- Gate $gate_name failed after $retry_count remediation attempts"
          return 1
        fi
      fi
    fi
    
    # Run the gate
    local result
    local log_path
    local status
    
    result="$(run_gate "$phase_id" "$gate_name" "$gate_command")"
    log_path="${result%%|*}"
    status="${result##*|}"
    
    if [[ "$status" -ne 0 ]]; then
      if [[ "$gate_optional" == "true" ]]; then
        append_progress "- Gate skipped (optional): $gate_name"
        append_report "$(jq -n \
          --arg phase "$phase_id" \
          --arg gate "$gate_name" \
          --arg log "$log_path" \
          '{timestamp: (now | todate), phase: $phase, step: "gate", gate: $gate, status: "skipped", logFile: $log, source: "monitor"}')"
        continue
      fi
      
      # Gate failed - write to queue and wait
      log "Gate failed: $gate_name"
      append_progress "- Gate failed: $gate_name (writing to remediation queue)"
      
      # Collect diagnostics
      local diagnostics
      diagnostics=$(jq -n \
        --arg log_tail "$(tail -50 "$log_path" 2>/dev/null || echo 'No log available')" \
        --arg command "$gate_command" \
        '{logTail: $log_tail, command: $command}')
      
      write_failure_to_queue "$phase_id" "$gate_name" "$log_path" "$status" "$diagnostics"
      
      append_report "$(jq -n \
        --arg phase "$phase_id" \
        --arg gate "$gate_name" \
        --arg log "$log_path" \
        '{timestamp: (now | todate), phase: $phase, step: "gate", gate: $gate, status: "failed", logFile: $log, source: "monitor", action: "queued_for_remediation"}')"
      
      # Wait for remediation before continuing
      local remediation_timeout
      remediation_timeout=$(jq -r '.dualAgent.coordination.remediationTimeoutSeconds // 600' "$CONFIG_PATH")
      
      if wait_for_remediation "$phase_id" "$gate_name" "$remediation_timeout"; then
        # Retry the gate after successful remediation
        log "Retrying gate after remediation: $gate_name"
        result="$(run_gate "$phase_id" "$gate_name" "$gate_command")"
        log_path="${result%%|*}"
        status="${result##*|}"
        
        if [[ "$status" -ne 0 ]]; then
          log "Gate still failed after remediation: $gate_name"
          append_progress "- Gate still failed after remediation: $gate_name"
          return 1
        fi
        
        log "Gate passed after remediation: $gate_name"
        append_progress "- Gate passed (after remediation): $gate_name"
      else
        log "Remediation failed for $gate_name, aborting phase"
        return 1
      fi
    else
      append_progress "- Gate passed: $gate_name"
      append_report "$(jq -n \
        --arg phase "$phase_id" \
        --arg gate "$gate_name" \
        --arg log "$log_path" \
        '{timestamp: (now | todate), phase: $phase, step: "gate", gate: $gate, status: "passed", logFile: $log, source: "monitor"}')"
    fi
  done < <(echo "$gates_json" | jq -c '.[]')
  
  return 0
}

# Run the agent for a phase
run_monitor_agent() {
  local phase_id="$1"
  local phase_json
  phase_json="$(get_phase_json "$phase_id")"
  
  local objective
  objective="$(echo "$phase_json" | jq -r '.objective // "Complete this phase"')"
  
  # Create objective file
  {
    echo "# Objective: ${phase_id}"
    echo ""
    echo "$objective"
    echo ""
    echo "## Instructions"
    echo ""
    echo "You are the MONITOR agent. Your job is to:"
    echo "1. Execute the objective above"
    echo "2. Verify success by checking gates"
    echo "3. Report any issues clearly"
    echo ""
    echo "Do NOT attempt to fix infrastructure issues - report them and they will be handled."
  } > "$OBJECTIVE_PATH"
  
  # Create combined prompt
  local run_prompt
  run_prompt="$(mktemp)"
  {
    cat "$MONITOR_PROMPT_PATH"
    echo ""
    echo "---"
    echo ""
    cat "$OBJECTIVE_PATH"
  } > "$run_prompt"
  
  log "Invoking monitor agent for phase: $phase_id"
  
  set +e
  # Check if using droid (needs -f file) or claude (needs piped input)
  if [[ "$MONITOR_COMMAND" == *"droid"* ]]; then
    # droid uses -f <file> syntax
    $MONITOR_COMMAND "$run_prompt"
  else
    # claude uses piped input with -p -
    cat "$run_prompt" | $MONITOR_COMMAND -
  fi
  local exit_code=$?
  set -e
  
  rm -f "$run_prompt"
  return $exit_code
}

# Main monitoring loop
main_loop() {
  log "Starting monitor agent (PID: $$)"
  ensure_coordination_file
  init_state
  
  # Ensure valid session
  ensure_valid_session
  local session_id
  session_id=$(get_session_id)
  log "Session: $session_id"
  
  update_monitor_status "running"
  append_progress "Monitor agent started (PID: $$, session: $session_id)"
  
  # Get all phases in order
  local phases
  phases=$(jq -r '.phases[].id' "$CONFIG_PATH")
  
  while true; do
    # Check circuit breaker before each iteration
    if is_circuit_breaker_open; then
      log "Circuit breaker is OPEN - pausing monitor"
      update_monitor_status "paused-circuit-open"
      append_progress "Monitor paused: Circuit breaker open (will retry in ${CB_RECOVERY_MINUTES} minutes)"
      
      # Reset session if configured
      if [[ "$SESSION_AUTO_RESET_ON_CB" == "true" ]]; then
        log "Auto-resetting session due to circuit breaker open"
        reset_session
      fi
      
      sleep 60  # Wait 1 minute before checking again
      continue
    fi
    
    # Update session activity
    update_session_activity
    
    local current_phase
    current_phase=$(get_current_phase)
    
    log "Current phase: $current_phase"
    update_monitor_status "running" "$current_phase"
    
    # Check if phase is already completed
    if is_phase_completed "$current_phase"; then
      log "Phase $current_phase already completed, finding next..."
      
      local found_next=false
      local past_current=false
      
      while read -r phase; do
        if [[ "$past_current" == "true" ]]; then
          if ! is_phase_completed "$phase"; then
            log "Next phase: $phase"
            state_set_phase "$phase"
            current_phase="$phase"
            found_next=true
            break
          fi
        elif [[ "$phase" == "$current_phase" ]]; then
          past_current=true
        fi
      done <<< "$phases"
      
      if [[ "$found_next" != "true" ]]; then
        log "All phases completed!"
        update_monitor_status "completed"
        append_progress "All phases completed successfully!"
        exit 0
      fi
    fi
    
    # Run agent for the phase
    log "Running agent for phase: $current_phase"
    
    set +e
    run_monitor_agent "$current_phase"
    local agent_status=$?
    set -e
    
    if [[ $agent_status -ne 0 ]]; then
      log "Agent exited with status $agent_status for phase $current_phase"
      append_progress "- Agent exited with status: $agent_status"
    fi
    
    # Run gates for the phase
    if run_phase_gates "$current_phase"; then
      log "All gates passed for phase: $current_phase"
      state_add_completed_objective "$current_phase"
      append_progress "Phase completed: $current_phase"
      
      # Update circuit breaker - progress detected
      update_circuit_breaker_success
      
      # Find and set next phase
      local found_next=false
      local past_current=false
      
      while read -r phase; do
        if [[ "$past_current" == "true" ]]; then
          log "Moving to next phase: $phase"
          state_set_phase "$phase"
          found_next=true
          break
        elif [[ "$phase" == "$current_phase" ]]; then
          past_current=true
        fi
      done <<< "$phases"
      
      if [[ "$found_next" != "true" ]]; then
        log "All phases completed!"
        update_monitor_status "completed"
        append_progress "All phases completed successfully!"
        exit 0
      fi
    else
      log "Gates failed for phase: $current_phase"
      
      # Update circuit breaker with failure
      update_circuit_breaker_failure "phase_${current_phase}_gates_failed"
      
      # Continue looping - remediation may fix the issue
    fi
    
    # Brief pause between iterations
    sleep "$POLL_INTERVAL"
  done
}

# Handle signals for graceful shutdown
cleanup() {
  log "Shutting down monitor agent..."
  update_monitor_status "stopped"
  exit 0
}

trap cleanup SIGTERM SIGINT

# Main entry point
main_loop
