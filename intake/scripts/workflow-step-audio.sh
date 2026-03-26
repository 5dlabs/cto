#!/usr/bin/env bash
set -euo pipefail

WORKFLOW="${1:-workflow}"
STEP="${2:-step}"
ENABLED="${3:-false}"
LEVEL="${4:-normal}"
EXTRA_CONTEXT="${5:-}"
ROLE_OVERRIDE="${6:-}"

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
