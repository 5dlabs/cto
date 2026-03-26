#!/usr/bin/env bash
# Signal intake feedback-loop state: audio + optional Discord + durable JSON for "visual" status in the repo.
#
# Usage:
#   intake/scripts/feedback-loop-signal.sh start|broken|waiting [--message "extra detail"]
#   intake/scripts/feedback-loop-signal.sh show
#   intake/scripts/feedback-loop-signal.sh clear
#
# Env:
#   WORKSPACE           Repo root (default: parent of intake/)
#   INTAKE_LOOP_NO_SPEAK=1   Skip coordinator-speak (terminals only)
#   INTAKE_LOOP_NO_DISCORD=1 Skip bridge-notify
#
# State file (JSON): intake/.feedback-loop-state.json — gitignored; open in editor or `jq .`.

set -euo pipefail
ROOT="${WORKSPACE:-$(cd "$(dirname "$0")/../.." && pwd)}"
STATE="$ROOT/intake/.feedback-loop-state.json"
SPEAK="$ROOT/intake/scripts/coordinator-speak.sh"

# Resolve lobster-voice binary (preferred over coordinator-speak for TTS)
LV_BIN=""
if command -v lobster-voice >/dev/null 2>&1; then
  LV_BIN="lobster-voice"
elif [[ -x "$ROOT/apps/lobster-voice/lobster-voice" ]]; then
  LV_BIN="$ROOT/apps/lobster-voice/lobster-voice"
fi

speak_msg() {
  local msg="$1" level="${2:-normal}"
  if [[ -n "$LV_BIN" ]]; then
    WORKSPACE="$ROOT" "$LV_BIN" speak "$msg" --level "$level" && return 0
  fi
  "$SPEAK" "$msg" "$level"
}

usage() {
  sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

write_state() {
  local status="$1" detail="$2" spoken="$3"
  mkdir -p "$(dirname "$STATE")"
  jq -n \
    --arg status "$status" \
    --arg detail "$detail" \
    --arg spoken "$spoken" \
    --arg since "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{status: $status, since: $since, detail: $detail, last_spoken: $spoken}' >"$STATE"
}

discord_notify() {
  [[ "${INTAKE_LOOP_NO_DISCORD:-}" == "1" ]] && return 0
  command -v intake-util >/dev/null 2>&1 || return 0
  local body="$1" step="$2"
  local meta
  meta=$(jq -nc --arg step "$step" --arg status "$3" '{step: $step, feedback_loop_status: $status}')
  echo "$body" | intake-util bridge-notify --from intake --to deliberation --metadata "$meta" >/dev/null 2>&1 || true
}

banner() {
  local line="$1"
  printf '\n\033[1m%s\033[0m\n\n' "$line" >&2
}

do_start() {
  local extra="${1:-}"
  local spoken="Feedback loop started. Coordinator is running."
  local detail="running"
  [[ -n "$extra" ]] && detail="running — $extra"
  banner "═══ INTAKE FEEDBACK LOOP: STARTED (running) ═══"
  write_state "running" "$detail" "$spoken"
  [[ "${INTAKE_LOOP_NO_SPEAK:-}" != "1" ]] && speak_msg "$spoken" "normal" || true
  local msg="🔄 **Feedback loop started** — coordinator is running checkpoints and intake tests.${extra:+ $extra}"
  discord_notify "$msg" "feedback-loop-start" "running"
}

do_broken() {
  local extra="${1:-}"
  local spoken="Intake feedback loop broken. Check terminal and state file."
  local detail="${extra:-unrecoverable or exceeded retry budget}"
  banner "═══ INTAKE FEEDBACK LOOP: BROKEN ═══"
  write_state "broken" "$detail" "$spoken"
  [[ "${INTAKE_LOOP_NO_SPEAK:-}" != "1" ]] && speak_msg "$spoken" "error" || true
  discord_notify "**Feedback loop broken.** ${detail}" "feedback-loop-broken" "broken"
}

do_waiting() {
  local extra="${1:-}"
  local spoken="Intake feedback loop is waiting for human input."
  local detail="${extra:-approval or emergency response required}"
  banner "═══ INTAKE FEEDBACK LOOP: WAITING FOR HUMAN ═══"
  write_state "awaiting_human" "$detail" "$spoken"
  [[ "${INTAKE_LOOP_NO_SPEAK:-}" != "1" ]] && speak_msg "$spoken" "wait" || true
  discord_notify "**Waiting for human.** ${detail}" "feedback-loop-waiting" "awaiting_human"
}

cmd="${1:-}"
[[ $# -gt 0 ]] && shift
extra=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --message)
      extra="${2:-}"
      shift 2
      ;;
    *)
      extra="${extra:+$extra }$1"
      shift
      ;;
  esac
done

case "$cmd" in
  start) do_start "$extra" ;;
  broken) do_broken "$extra" ;;
  waiting) do_waiting "$extra" ;;
  show) [[ -f "$STATE" ]] && cat "$STATE" || echo '{"status":"none"}' ;;
  clear)
    rm -f "$STATE"
    banner "═══ INTAKE FEEDBACK LOOP: CLEARED ═══"
    ;;
  *) usage ;;
esac
