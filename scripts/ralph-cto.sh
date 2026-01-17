#!/usr/bin/env bash
set -euo pipefail

# ROOT_DIR can be overridden by WORKSPACE env var (useful for containers)
ROOT_DIR="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONFIG_PATH="${RALPH_CONFIG:-${ROOT_DIR}/lifecycle-test/ralph-cto.json}"
RUN_AGENT=1
START_PHASE=""
ATTENDED_RUN=0
CHECK_COMPLETE=0

usage() {
  cat <<'EOF'
Usage: scripts/ralph-cto.sh [--config path] [--phase id] [--no-run] [--check-complete]

  --config         Path to ralph-cto.json (default: lifecycle-test/ralph-cto.json)
  --phase          Start at a specific phase id (e.g., intake, play)
  --attended       Run in attended mode (required for early loops)
  --no-run         Do not invoke the CLI runner; only write objectives and run gates
  --check-complete Check if lifecycle is complete and exit with status (0=complete, 1=incomplete)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --config)
      CONFIG_PATH="$2"
      shift 2
      ;;
    --phase)
      START_PHASE="$2"
      shift 2
      ;;
    --attended)
      ATTENDED_RUN=1
      shift
      ;;
    --no-run)
      RUN_AGENT=0
      shift
      ;;
    --check-complete)
      CHECK_COMPLETE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

log() { echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*" >&2; }

# Cleanup function for graceful exit
cleanup_on_exit() {
  if [[ -n "${MONITOR_PID:-}" ]] && kill -0 "$MONITOR_PID" 2>/dev/null; then
    log "Cleaning up pod monitor on exit"
    rm -f "$MONITOR_STOP_FILE" "${MONITOR_STOP_FILE}.error"
    kill "$MONITOR_PID" 2>/dev/null || true
  fi
}
trap cleanup_on_exit EXIT

# Handle --check-complete: check if lifecycle is complete and exit
if [[ "$CHECK_COMPLETE" -eq 1 ]]; then
  # Get state path from config or use default
  if [[ -f "$CONFIG_PATH" ]]; then
    STATE_PATH=$(jq -r '.paths.statePath // "lifecycle-test/ralph-cto.state.json"' "$CONFIG_PATH")
    # If RALPH_STATE env is set, use it instead
    STATE_PATH="${RALPH_STATE:-${ROOT_DIR}/${STATE_PATH}}"
  else
    STATE_PATH="${RALPH_STATE:-${ROOT_DIR}/lifecycle-test/ralph-cto.state.json}"
  fi
  
  if [[ ! -f "$STATE_PATH" ]]; then
    log "State file not found: $STATE_PATH"
    echo '{"complete": false, "phase": "unknown", "error": "state file not found"}'
    exit 1
  fi
  
  PHASE=$(jq -r '.phase // "unknown"' "$STATE_PATH")
  COMPLETED=$(jq -r '.completedObjectives // [] | join(",")' "$STATE_PATH")
  
  if [[ "$PHASE" == "done" || "$PHASE" == "complete" ]]; then
    echo "{\"complete\": true, \"phase\": \"$PHASE\", \"completedObjectives\": \"$COMPLETED\"}"
    exit 0
  else
    echo "{\"complete\": false, \"phase\": \"$PHASE\", \"completedObjectives\": \"$COMPLETED\"}"
    exit 1
  fi
fi

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { log "Missing required command: $1"; exit 1; }
}

json_get() {
  jq -r "$1" "$CONFIG_PATH"
}

json_get_raw() {
  jq -c "$1" "$CONFIG_PATH"
}

ensure_file() {
  local path="$1"
  [[ -f "$path" ]] || echo "" > "$path"
}

init_state() {
  local path="$1"
  if [[ ! -s "$path" ]]; then
    cat > "$path" <<'EOF'
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

state_set_phase() {
  local path="$1"
  local phase="$2"
  local tmp
  tmp="$(mktemp)"
  jq --arg phase "$phase" '.phase = $phase' "$path" > "$tmp" && mv "$tmp" "$path"
}

state_increment_attempt() {
  local path="$1"
  local phase="$2"
  local tmp
  tmp="$(mktemp)"
  jq --arg phase "$phase" '.attempts[$phase] = ((.attempts[$phase] // 0) + 1)' "$path" > "$tmp" && mv "$tmp" "$path"
}

state_increment_attended() {
  local path="$1"
  local tmp
  tmp="$(mktemp)"
  jq '.attendedCompleted = ((.attendedCompleted // 0) + 1)' "$path" > "$tmp" && mv "$tmp" "$path"
}

state_add_completed_objective() {
  local path="$1"
  local objective="$2"
  local tmp
  tmp="$(mktemp)"
  jq --arg objective "$objective" '
    .completedObjectives = (.completedObjectives // []) |
    if (.completedObjectives | index($objective)) then . else .completedObjectives += [$objective] end
  ' "$path" > "$tmp" && mv "$tmp" "$path"
}

state_set_success() {
  local path="$1"
  local tmp
  tmp="$(mktemp)"
  jq '.last_success = (now | todate)' "$path" > "$tmp" && mv "$tmp" "$path"
}
init_json_array() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "[]" > "$path"
  fi
}

append_report() {
  local report_path="$1"
  local entry="$2"
  local tmp
  tmp="$(mktemp)"
  init_json_array "$report_path"
  jq ". + [$entry]" "$report_path" > "$tmp" && mv "$tmp" "$report_path"
}

append_progress() {
  local progress_path="$1"
  local message="$2"
  ensure_file "$progress_path"
  {
    echo ""
    echo "## $(date -u '+%Y-%m-%d %H:%M:%SZ')"
    echo "$message"
  } >> "$progress_path"
}

redact_string() {
  local input="$1"
  local output="$input"
  local pattern

  if [[ "${REDACTION_ENABLED:-false}" != "true" ]]; then
    echo "$input"
    return
  fi

  while read -r pattern; do
    [[ -z "$pattern" ]] && continue
    set +e
    output="$(perl -0pe "s{$pattern}{${REDACTION_REPLACEMENT}}g" <<< "$output")"
    local status=$?
    set -e
    if [[ "$status" -ne 0 ]]; then
      log "Redaction failed for pattern: $pattern"
      output="$input"
    fi
  done < <(echo "${REDACTION_PATTERNS_JSON:-[]}" | jq -r '.[]?')

  echo "$output"
}

redact_file() {
  local file="$1"
  local pattern

  if [[ "${REDACTION_ENABLED:-false}" != "true" ]]; then
    return 0
  fi

  while read -r pattern; do
    [[ -z "$pattern" ]] && continue
    set +e
    perl -0pi -e "s{$pattern}{${REDACTION_REPLACEMENT}}g" "$file"
    local status=$?
    set -e
    if [[ "$status" -ne 0 ]]; then
      log "Redaction failed for pattern: $pattern"
    fi
  done < <(echo "${REDACTION_PATTERNS_JSON:-[]}" | jq -r '.[]?')
}

select_plan_objective() {
  local plan_path="$1"
  local state_path="$2"
  local line
  local id
  local objective

  [[ -f "$plan_path" ]] || return 1

  while IFS= read -r line; do
    # Quote the pattern to prevent [ ] from being interpreted as glob
    line="${line#"- [ ] "}"
    IFS='|' read -r id objective <<< "$line"
    id="$(echo "$id" | xargs)"
    objective="$(echo "${objective:-}" | xargs)"

    [[ -z "$id" ]] && continue
    if jq -e --arg id "$id" '.completedObjectives // [] | index($id)' "$state_path" >/dev/null; then
      continue
    fi

    echo "${id}|${objective}"
    return 0
  done < <(grep -E '^- \[ \] ' "$plan_path" || true)

  return 1
}

# Pod monitoring functions
MONITOR_PID=""
MONITOR_STOP_FILE=""

check_pod_status() {
  local namespace="$1"
  local selector="$2"
  local pod_info
  local pod_name
  local pod_status
  local container_statuses

  # Get pods matching selector
  pod_info=$(kubectl get pods -n "$namespace" -l "$selector" -o json 2>/dev/null || echo '{"items":[]}')
  
  # Check each pod
  while read -r pod; do
    [[ -z "$pod" || "$pod" == "null" ]] && continue
    
    pod_name=$(echo "$pod" | jq -r '.metadata.name // "unknown"')
    pod_status=$(echo "$pod" | jq -r '.status.phase // "Unknown"')
    
    # Check for Error or Failed phase
    if [[ "$pod_status" == "Failed" || "$pod_status" == "Error" ]]; then
      echo "ERROR:$pod_name:Pod in $pod_status state"
      return 1
    fi
    
    # Check container statuses for terminated with non-zero exit code
    container_statuses=$(echo "$pod" | jq -c '.status.containerStatuses // []')
    while read -r container; do
      [[ -z "$container" || "$container" == "null" ]] && continue
      
      local container_name
      local terminated
      local exit_code
      
      container_name=$(echo "$container" | jq -r '.name // "unknown"')
      terminated=$(echo "$container" | jq -c '.state.terminated // null')
      
      if [[ "$terminated" != "null" ]]; then
        exit_code=$(echo "$terminated" | jq -r '.exitCode // 0')
        if [[ "$exit_code" != "0" ]]; then
          local reason
          reason=$(echo "$terminated" | jq -r '.reason // "Unknown"')
          echo "ERROR:$pod_name:Container $container_name exited with code $exit_code ($reason)"
          return 1
        fi
      fi
    done < <(echo "$container_statuses" | jq -c '.[]')
  done < <(echo "$pod_info" | jq -c '.items[]')
  
  return 0
}

start_pod_monitor() {
  local namespace="$1"
  local interval="$2"
  local progress_path="$3"
  local report_path="$4"
  shift 4
  local selectors=("$@")
  
  MONITOR_STOP_FILE=$(mktemp)
  
  (
    while [[ -f "$MONITOR_STOP_FILE" ]]; do
      for selector in "${selectors[@]}"; do
        local result
        result=$(check_pod_status "$namespace" "$selector" 2>&1)
        local status=$?
        
        if [[ $status -ne 0 ]]; then
          local error_msg
          error_msg=$(echo "$result" | grep "^ERROR:" | head -1)
          if [[ -n "$error_msg" ]]; then
            local pod_name
            local error_detail
            pod_name=$(echo "$error_msg" | cut -d: -f2)
            error_detail=$(echo "$error_msg" | cut -d: -f3-)
            
            log "🚨 Pod monitor detected failure: $pod_name - $error_detail"
            
            # Get pod logs for debugging
            local log_snippet
            log_snippet=$(kubectl logs "$pod_name" -n "$namespace" --all-containers --tail=20 2>/dev/null | head -50 || echo "Failed to get logs")
            
            append_progress "$progress_path" "### Pod Monitor Alert
- **Pod**: $pod_name
- **Error**: $error_detail
- **Selector**: $selector
- **Log snippet**:
\`\`\`
$log_snippet
\`\`\`"
            
            append_report "$report_path" "$(jq -n \
              --arg pod "$pod_name" \
              --arg error "$error_detail" \
              --arg selector "$selector" \
              --arg namespace "$namespace" \
              '{timestamp: (now | todate), type: "pod_monitor_alert", pod: $pod, error: $error, selector: $selector, namespace: $namespace}')"
            
            # Signal main process about the failure
            echo "$error_msg" > "${MONITOR_STOP_FILE}.error"
          fi
        fi
      done
      
      sleep "$interval"
    done
  ) &
  
  MONITOR_PID=$!
  log "Started pod monitor (PID: $MONITOR_PID, interval: ${interval}s)"
}

stop_pod_monitor() {
  if [[ -n "$MONITOR_PID" ]] && kill -0 "$MONITOR_PID" 2>/dev/null; then
    log "Stopping pod monitor (PID: $MONITOR_PID)"
    rm -f "$MONITOR_STOP_FILE"
    kill "$MONITOR_PID" 2>/dev/null || true
    wait "$MONITOR_PID" 2>/dev/null || true
    MONITOR_PID=""
  fi
  
  # Check if monitor detected an error
  if [[ -f "${MONITOR_STOP_FILE}.error" ]]; then
    local error
    error=$(cat "${MONITOR_STOP_FILE}.error")
    rm -f "${MONITOR_STOP_FILE}.error"
    echo "$error"
    return 1
  fi
  
  rm -f "$MONITOR_STOP_FILE" "${MONITOR_STOP_FILE}.error"
  return 0
}

get_phase_json() {
  local phase_id="$1"
  jq -c --arg id "$phase_id" '.phases[] | select(.id == $id)' "$CONFIG_PATH"
}

run_command() {
  local name="$1"
  local command="$2"
  local log_dir="$3"
  local log_path
  log_path="${log_dir}/${name}_$(date -u '+%Y%m%d_%H%M%S').log"
  mkdir -p "$log_dir"
  log "Running: $name"
  set +e
  bash -c "$command" >"$log_path" 2>&1
  local status=$?
  set -e
  redact_file "$log_path"
  echo "$log_path|$status"
}

write_objective() {
  local path="$1"
  local phase_id="$2"
  local objective="$3"
  local gates_json="$4"

  {
    echo "# Objective: ${phase_id}"
    echo ""
    echo "$objective"
    echo ""
    echo "## Gates"
    if [[ "$gates_json" == "null" ]]; then
      echo "- None specified"
    else
      echo "$gates_json" | jq -r '.[] | "- " + .name + ": `" + .command + "`"'
    fi
    echo ""
    echo "## Evidence"
    echo "- Record command output in lifecycle-test/report.json"
    echo "- Update lifecycle-test/progress.txt with the outcome"
  } > "$path"
}

run_gates() {
  local phase_id="$1"
  local gates_json="$2"
  local log_dir="$3"
  local report_path="$4"
  local progress_path="$5"

  if [[ "$gates_json" == "null" ]]; then
    return 0
  fi

  local gate
  local gate_name
  local gate_command
  local gate_command_safe
  local gate_optional
  local result
  local log_path
  local status

  while read -r gate; do
    gate_name="$(echo "$gate" | jq -r '.name')"
    gate_command="$(echo "$gate" | jq -r '.command')"
    gate_optional="$(echo "$gate" | jq -r '.optional // false')"
    gate_command_safe="$(redact_string "$gate_command")"

    result="$(run_command "${phase_id}_${gate_name}" "$gate_command" "$log_dir")"
    log_path="${result%%|*}"
    status="${result##*|}"

    if [[ "$status" -ne 0 && "$gate_optional" == "true" ]]; then
      append_report "$report_path" "$(jq -n \
        --arg phase "$phase_id" \
        --arg gate "$gate_name" \
        --arg cmd "$gate_command_safe" \
        --arg log "$log_path" \
        '{timestamp: (now | todate), phase: $phase, step: "gate", gate: $gate, status: "skipped", command: $cmd, logFile: $log}')"
      append_progress "$progress_path" "- Gate skipped (optional): ${gate_name}"
      continue
    fi

    if [[ "$status" -ne 0 ]]; then
      append_report "$report_path" "$(jq -n \
        --arg phase "$phase_id" \
        --arg gate "$gate_name" \
        --arg cmd "$gate_command_safe" \
        --arg log "$log_path" \
        '{timestamp: (now | todate), phase: $phase, step: "gate", gate: $gate, status: "failed", command: $cmd, logFile: $log}')"
      append_progress "$progress_path" "- Gate failed: ${gate_name} (see ${log_path})"
      return 1
    fi

    append_report "$report_path" "$(jq -n \
      --arg phase "$phase_id" \
      --arg gate "$gate_name" \
      --arg cmd "$gate_command_safe" \
      --arg log "$log_path" \
      '{timestamp: (now | todate), phase: $phase, step: "gate", gate: $gate, status: "passed", command: $cmd, logFile: $log}')"
    append_progress "$progress_path" "- Gate passed: ${gate_name}"
  done < <(echo "$gates_json" | jq -c '.[]')
}

run_agent() {
  local exec_prompt="$1"
  local objective="$2"
  local runner_command="$3"
  local run_prompt
  run_prompt="$(mktemp)"

  {
    cat "$exec_prompt"
    echo ""
    cat "$objective"
  } > "$run_prompt"

  log "Invoking CLI runner"
  # Use the prompt file directly to avoid shell interpretation of backticks
  # Claude CLI accepts -p with a file path or reads from stdin with -p -
  cat "$run_prompt" | $runner_command -
  local exit_code=$?
  rm -f "$run_prompt"
  return $exit_code
}

main() {
  local report_path
  local progress_path
  local objective_path
  local exec_prompt_path
  local state_path
  local plan_path
  local log_dir
  local runner_command
  local selected_phase_id
  local selected_objective
  local plan_selection
  local phase_json
  local attended_enabled
  local attended_required
  local attended_completed

  report_path="${ROOT_DIR}/$(json_get '.paths.reportPath')"
  progress_path="${ROOT_DIR}/$(json_get '.paths.progressPath')"
  objective_path="${ROOT_DIR}/$(json_get '.paths.objectivePath')"
  exec_prompt_path="${ROOT_DIR}/$(json_get '.paths.execPromptPath')"
  state_path="${ROOT_DIR}/$(json_get '.paths.statePath')"
  plan_path="$(json_get '.paths.planPath // empty')"
  if [[ -n "$plan_path" ]]; then
    plan_path="${ROOT_DIR}/${plan_path}"
  fi
  log_dir="${ROOT_DIR}/lifecycle-test/ralph-logs"
  runner_command="$(json_get '.runner.command')"

  REDACTION_ENABLED="$(json_get '.redaction.enabled // false')"
  REDACTION_REPLACEMENT="$(json_get '.redaction.replacement // "[REDACTED]"')"
  REDACTION_PATTERNS_JSON="$(json_get_raw '.redaction.patterns // []')"
  if [[ "$REDACTION_ENABLED" == "true" ]] && ! command -v perl >/dev/null 2>&1; then
    log "Redaction enabled but perl not found; disabling redaction"
    REDACTION_ENABLED=false
  fi

  init_json_array "$report_path"
  ensure_file "$progress_path"
  ensure_file "$objective_path"
  ensure_file "$exec_prompt_path"
  ensure_file "$state_path"
  init_state "$state_path"

  while read -r cmd; do
    require_cmd "$cmd"
  done < <(json_get '.preflight.requiredCommands[]')

  while read -r check; do
    local name
    local command
    local command_safe
    local optional
    local result
    local log_path
    local status

    name="$(echo "$check" | jq -r '.name')"
    command="$(echo "$check" | jq -r '.command')"
    optional="$(echo "$check" | jq -r '.optional // false')"
    command_safe="$(redact_string "$command")"
    result="$(run_command "preflight_${name}" "$command" "$log_dir")"
    log_path="${result%%|*}"
    status="${result##*|}"

    if [[ "$status" -ne 0 && "$optional" == "true" ]]; then
      append_report "$report_path" "$(jq -n \
        --arg phase "preflight" \
        --arg gate "$name" \
        --arg cmd "$command_safe" \
        --arg log "$log_path" \
        '{timestamp: (now | todate), phase: $phase, step: "preflight", gate: $gate, status: "skipped", command: $cmd, logFile: $log}')"
      continue
    fi

    if [[ "$status" -ne 0 ]]; then
      append_report "$report_path" "$(jq -n \
        --arg phase "preflight" \
        --arg gate "$name" \
        --arg cmd "$command_safe" \
        --arg log "$log_path" \
        '{timestamp: (now | todate), phase: $phase, step: "preflight", gate: $gate, status: "failed", command: $cmd, logFile: $log}')"
      log "Preflight failed: ${name} (see ${log_path})"
      exit 1
    fi

    append_report "$report_path" "$(jq -n \
      --arg phase "preflight" \
      --arg gate "$name" \
      --arg cmd "$command_safe" \
      --arg log "$log_path" \
      '{timestamp: (now | todate), phase: $phase, step: "preflight", gate: $gate, status: "passed", command: $cmd, logFile: $log}')"
  done < <(json_get '.preflight.checks' | jq -c '.[]')

  local phase_id
  local objective
  local gates_json
  local cleanup_json
  if [[ -n "$START_PHASE" ]]; then
    selected_phase_id="$START_PHASE"
  elif [[ -n "$plan_path" && -f "$plan_path" ]]; then
    if plan_selection="$(select_plan_objective "$plan_path" "$state_path")"; then
      selected_phase_id="${plan_selection%%|*}"
      selected_objective="${plan_selection#*|}"
    fi
  fi

  if [[ -z "$selected_phase_id" ]]; then
    log "No pending objectives found. Exiting."
    exit 0
  fi

  phase_json="$(get_phase_json "$selected_phase_id")"
  if [[ -z "$phase_json" ]]; then
    log "Phase not found in config: ${selected_phase_id}"
    exit 1
  fi

  phase_id="$selected_phase_id"
  objective="$(echo "$phase_json" | jq -r '.objective')"
  if [[ -n "${selected_objective:-}" && "${selected_objective}" != "${phase_id}" ]]; then
    objective="$selected_objective"
  fi
  gates_json="$(echo "$phase_json" | jq -c '.gates // null')"
  cleanup_json="$(echo "$phase_json" | jq -c '.cleanup // null')"

  attended_enabled="$(json_get '.attended.enabled // false')"
  attended_required="$(json_get '.attended.requiredRuns // 0')"
  attended_completed="$(jq -r '.attendedCompleted // 0' "$state_path")"

  state_set_phase "$state_path" "$phase_id"
  write_objective "$objective_path" "$phase_id" "$objective" "$gates_json"
  append_progress "$progress_path" "- Phase started: ${phase_id}"

  # Pod monitoring configuration
  local monitoring_enabled
  local monitoring_interval
  local monitoring_namespace
  local monitoring_selectors
  monitoring_enabled="$(json_get '.monitoring.enabled // false')"
  monitoring_interval="$(json_get '.monitoring.intervalSeconds // 30')"
  monitoring_namespace="$(json_get '.monitoring.namespace // "cto"')"
  monitoring_selectors=()
  while read -r selector; do
    [[ -n "$selector" ]] && monitoring_selectors+=("$selector")
  done < <(json_get '.monitoring.podSelectors[]? // empty')

  if [[ "$RUN_AGENT" -eq 1 ]]; then
    if [[ "$attended_enabled" == "true" && "$attended_completed" -lt "$attended_required" ]]; then
      if [[ "$ATTENDED_RUN" -ne 1 ]]; then
        append_progress "$progress_path" "- Attended run required before unattended loops. Re-run with --attended."
        log "Attended run required before unattended loops. Re-run with --attended."
        exit 0
      fi
      state_increment_attended "$state_path"
    fi
    
    # Start pod monitor if enabled
    if [[ "$monitoring_enabled" == "true" && ${#monitoring_selectors[@]} -gt 0 ]]; then
      start_pod_monitor "$monitoring_namespace" "$monitoring_interval" "$progress_path" "$report_path" "${monitoring_selectors[@]}"
    fi
    
    # Run agent
    local agent_status=0
    run_agent "$exec_prompt_path" "$objective_path" "$runner_command" || agent_status=$?
    
    # Stop pod monitor and check for errors
    if [[ "$monitoring_enabled" == "true" && ${#monitoring_selectors[@]} -gt 0 ]]; then
      local monitor_error
      if ! monitor_error=$(stop_pod_monitor); then
        append_progress "$progress_path" "- Pod monitor detected failure during agent run: $monitor_error"
        log "Pod monitor detected failure: $monitor_error"
        # If agent also failed, report both
        if [[ $agent_status -ne 0 ]]; then
          append_progress "$progress_path" "- Agent also exited with status: $agent_status"
        fi
        exit 1
      fi
    fi
    
    if [[ $agent_status -ne 0 ]]; then
      append_progress "$progress_path" "- Agent exited with status: $agent_status"
      log "Agent exited with status: $agent_status"
      exit 1
    fi
  fi

  if ! run_gates "$phase_id" "$gates_json" "$log_dir" "$report_path" "$progress_path"; then
    append_progress "$progress_path" "- Phase failed: ${phase_id}"
    state_increment_attempt "$state_path" "$phase_id"
    if [[ "$cleanup_json" != "null" ]]; then
      while read -r cmd; do
        run_command "cleanup_${phase_id}" "$cmd" "$log_dir" >/dev/null
      done < <(echo "$cleanup_json" | jq -r '.[]')
    else
      while read -r cmd; do
        run_command "cleanup_global" "$cmd" "$log_dir" >/dev/null
      done < <(json_get '.cleanup.global[]')
    fi
    exit 1
  fi

  state_set_success "$state_path"
  state_add_completed_objective "$state_path" "$phase_id"
  append_progress "$progress_path" "- Phase passed: ${phase_id}"
}

main
