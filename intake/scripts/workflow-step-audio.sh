#!/usr/bin/env bash
set -euo pipefail

WORKFLOW="${1:-workflow}"
STEP="${2:-step}"
ENABLED="${3:-false}"
LEVEL="${4:-normal}"
EXTRA_CONTEXT="${5:-}"
ROLE_OVERRIDE="${6:-}"

# --- Observability: always emit step event (even when audio is muted) ---
_STEP_LOG_DIR="${WORKSPACE:-.}/.intake/logs"
_TIMING_FILE="$_STEP_LOG_DIR/.step-timing"
if mkdir -p "$_STEP_LOG_DIR" 2>/dev/null; then
  _ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  _now=$(date +%s)
  _run_id="${INTAKE_RUN_ID:-unknown}"

  # Close out previous step (emit step_end with duration)
  if [[ -f "$_TIMING_FILE" ]]; then
    _prev_step=$(sed -n '1p' "$_TIMING_FILE")
    _prev_phase=$(sed -n '2p' "$_TIMING_FILE")
    _prev_epoch=$(sed -n '3p' "$_TIMING_FILE")
    if [[ -n "$_prev_epoch" ]]; then
      _dur_ms=$(( (_now - _prev_epoch) * 1000 ))
      printf '{"ts":"%s","event":"step_end","run_id":"%s","step_id":"%s","phase":"%s","duration_ms":%d,"exit_code":0}\n' \
        "$_ts" "$_run_id" "$_prev_step" "$_prev_phase" "$_dur_ms" \
        >> "$_STEP_LOG_DIR/pipeline.jsonl" 2>/dev/null || true
    fi
  fi

  # Emit step_start for current step
  _event="step_start"
  case "$LEVEL" in
    error|fail*) _event="step_error" ;;
  esac
  printf '{"ts":"%s","event":"%s","run_id":"%s","step_id":"%s","phase":"%s"}\n' \
    "$_ts" "$_event" "$_run_id" "$STEP" "$WORKFLOW" \
    >> "$_STEP_LOG_DIR/pipeline.jsonl" 2>/dev/null || true

  # Save timing for this step
  printf '%s\n%s\n%s\n' "$STEP" "$WORKFLOW" "$_now" > "$_TIMING_FILE" 2>/dev/null || true
fi
# --- End observability ---

if [[ "$ENABLED" != "true" && "$ENABLED" != "1" ]]; then
  exit 0
fi

if [[ "${INTAKE_AUDIO_MUTE:-}" == "true" ]]; then
  exit 0
fi

ROOT="${WORKSPACE:-$(cd "$(dirname "$0")/../.." && pwd)}"
STEP_LC="$(printf '%s' "$STEP" | tr '[:upper:]' '[:lower:]')"
if [[ "$STEP_LC" == *"fail"* || "$STEP_LC" == *"error"* ]]; then
  LEVEL="error"
fi

LV_BIN=""
if command -v lobster-voice >/dev/null 2>&1; then
  LV_BIN="lobster-voice"
elif [[ -x "$ROOT/apps/lobster-voice/lobster-voice" ]]; then
  LV_BIN="$ROOT/apps/lobster-voice/lobster-voice"
fi

if [[ -n "$LV_BIN" ]]; then
  lv_args=(step "$WORKFLOW" "$STEP" --level "$LEVEL")
  if [[ -n "${project_name:-}" ]]; then
    lv_args+=(--project "$project_name")
  fi
  if [[ -n "$EXTRA_CONTEXT" ]]; then
    lv_args+=(--context "$EXTRA_CONTEXT")
  fi
  WORKSPACE="$ROOT" LOBSTER_VOICE_AUDIO_DEVICE="${INTAKE_AUDIO_DEVICE:-82}" "$LV_BIN" "${lv_args[@]}" 2>&1 && exit 0 || true
  echo "workflow-step-audio: lobster-voice failed, falling back" >&2
fi

# Fallback: coordinator-speak.sh (original behavior)
sanitize_spoken_label() {
  local value="${1:-}"
  value="$(printf '%s' "$value" | sed -E 's/-[0-9]{10,}(-[0-9]{10,})*$//')"
  printf '%s' "$value"
}

parts=()
if [[ -n "${project_name:-}" ]]; then
  SPOKEN_PROJECT="$(sanitize_spoken_label "${project_name}")"
  if [[ -n "$SPOKEN_PROJECT" ]]; then
    parts+=("project ${SPOKEN_PROJECT}")
  fi
fi
if [[ -n "${session_id:-}" ]]; then
  SPOKEN_SESSION="$(sanitize_spoken_label "${session_id}")"
  if [[ -n "$SPOKEN_SESSION" ]]; then
    parts+=("session ${SPOKEN_SESSION}")
  fi
fi
if [[ -n "${decision_id:-}" ]]; then
  parts+=("decision ${decision_id}")
fi
if [[ -n "$EXTRA_CONTEXT" ]]; then
  parts+=("$EXTRA_CONTEXT")
fi
CTX=""
if [[ ${#parts[@]} -gt 0 ]]; then
  CTX=". $(printf '%s. ' "${parts[@]}")"
fi
ROLE_LABEL="$ROLE_OVERRIDE"
if [[ -z "$ROLE_LABEL" ]]; then
  case "$WORKFLOW" in
    pipeline) ROLE_LABEL="Coordinator lobster" ;;
    deliberation) ROLE_LABEL="Debate lobster" ;;
    decision-voting) ROLE_LABEL="Voting lobster" ;;
    intake) ROLE_LABEL="Planner lobster" ;;
    codebase-analysis) ROLE_LABEL="Research lobster" ;;
    task-refinement) ROLE_LABEL="Refinement lobster" ;;
    voting) ROLE_LABEL="Committee lobster" ;;
    *) ROLE_LABEL="Support lobster" ;;
  esac
fi
"$ROOT/intake/scripts/coordinator-speak.sh" "$ROLE_LABEL. $WORKFLOW step $STEP$CTX" "$LEVEL" || true
