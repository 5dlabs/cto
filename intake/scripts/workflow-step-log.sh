#!/usr/bin/env bash
# Structured JSONL logging for pipeline workflow steps.
# Usage:
#   source intake/scripts/workflow-step-log.sh
#   _step_log start "step-name" "phase"
#   ... do work ...
#   _step_log end "step-name" "phase" $?
#   _step_log error "step-name" "phase" "error message"
#
# Or as a standalone wrapper:
#   intake/scripts/workflow-step-log.sh start "step-name" "phase"

_STEP_LOG_DIR="${WORKSPACE:-.}/.intake/logs"
_STEP_LOG_FILE="$_STEP_LOG_DIR/pipeline.jsonl"

# Associative array for step start times (bash 4+)
declare -gA _STEP_START_TIMES 2>/dev/null || true

_step_log() {
  local action="$1" step_id="$2" phase="${3:-}" extra="${4:-}"
  local ts run_id

  mkdir -p "$_STEP_LOG_DIR" 2>/dev/null || true
  ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  run_id="${INTAKE_RUN_ID:-$(cat "${WORKSPACE:-.}/.intake/run-id.txt" 2>/dev/null || echo unknown)}"

  case "$action" in
    start)
      # Store start time in epoch seconds
      _STEP_START_TIMES["$step_id"]=$(date +%s)
      printf '{"ts":"%s","event":"step_start","run_id":"%s","step_id":"%s","phase":"%s"}\n' \
        "$ts" "$run_id" "$step_id" "$phase" >> "$_STEP_LOG_FILE"
      ;;
    end)
      local exit_code="${extra:-0}"
      local duration_ms=0
      if [[ -n "${_STEP_START_TIMES[$step_id]:-}" ]]; then
        local now=$(date +%s)
        duration_ms=$(( (now - _STEP_START_TIMES[$step_id]) * 1000 ))
        unset '_STEP_START_TIMES[$step_id]'
      fi
      printf '{"ts":"%s","event":"step_end","run_id":"%s","step_id":"%s","phase":"%s","duration_ms":%d,"exit_code":%s}\n' \
        "$ts" "$run_id" "$step_id" "$phase" "$duration_ms" "$exit_code" >> "$_STEP_LOG_FILE"
      ;;
    error)
      local error_msg="${extra:-unknown error}"
      # Sanitize error message for JSON (escape quotes and newlines)
      error_msg=$(printf '%s' "$error_msg" | head -c 500 | sed 's/"/\\"/g' | tr '\n' ' ')
      printf '{"ts":"%s","event":"step_error","run_id":"%s","step_id":"%s","phase":"%s","error":"%s"}\n' \
        "$ts" "$run_id" "$step_id" "$phase" "$error_msg" >> "$_STEP_LOG_FILE"
      ;;
  esac
}

# Allow standalone invocation
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  _step_log "$@"
fi
