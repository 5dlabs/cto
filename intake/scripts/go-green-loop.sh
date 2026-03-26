#!/usr/bin/env bash
# Retry checkpoints until green or max attempts — **one** feedback-loop start, then only iteration-checkpoints.
#
# Usage (from repo root):
#   ./intake/scripts/go-green-loop.sh
#   ./intake/scripts/go-green-loop.sh --bridges-skip
#   ./intake/scripts/go-green-loop.sh --max-attempts 40 --sleep 20
#
# Env:
#   WORKSPACE                  Repo root (default: CTO root from script path)
#   INTAKE_GO_GREEN_MAX_ATTEMPTS   Max checkpoint runs (default: 60)
#   INTAKE_GO_GREEN_SLEEP_SEC      Seconds between failed attempts (default: 15)
#   INTAKE_GO_GREEN_FOREVER=1     Ignore max attempts; loop until SIGINT (use carefully)
#   INTAKE_OP_ENV_FILE           If set, each attempt runs op run with that file (sets INTAKE_OP_WRAPPED; intake/local.env.op is the default when unset — see intake-op-auto.sh)
#   INTAKE_LOOP_NO_SPEAK / INTAKE_LOOP_NO_DISCORD — passed through to feedback-loop-signal on start only
#   Same as preflight/checkpoints: LINEAR_API_KEY, DISCORD_BRIDGE_URL, INTAKE_PREFLIGHT_BRIDGES_SKIP via --bridges-skip, etc.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export WORKSPACE="${WORKSPACE:-$ROOT}"
cd "$ROOT"

if [[ -x "$ROOT/apps/intake-util/intake-util" ]]; then
  export PATH="$ROOT/apps/intake-util:$PATH"
fi

usage() {
  sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

bridges_skip=""
max_attempts="${INTAKE_GO_GREEN_MAX_ATTEMPTS:-60}"
sleep_sec="${INTAKE_GO_GREEN_SLEEP_SEC:-15}"
forever=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bridges-skip) bridges_skip=1; shift ;;
    --max-attempts)
      max_attempts="${2:?}"
      shift 2
      ;;
    --sleep)
      sleep_sec="${2:?}"
      shift 2
      ;;
    -h | --help) usage ;;
    *) echo "go-green-loop.sh: unknown arg: $1" >&2; usage ;;
  esac
done

if [[ -n "$bridges_skip" ]]; then
  export INTAKE_PREFLIGHT_BRIDGES_SKIP=true
fi

if [[ "${INTAKE_GO_GREEN_FOREVER:-}" == "1" || "${INTAKE_GO_GREEN_FOREVER:-}" == "true" ]]; then
  max_attempts=-1
fi

run_iteration() {
  if [[ -n "${INTAKE_OP_ENV_FILE:-}" ]]; then
    if [[ ! -f "$INTAKE_OP_ENV_FILE" ]]; then
      echo "go-green-loop.sh: INTAKE_OP_ENV_FILE not a file: $INTAKE_OP_ENV_FILE" >&2
      return 1
    fi
    command -v op >/dev/null 2>&1 || {
      echo "go-green-loop.sh: op not on PATH but INTAKE_OP_ENV_FILE is set" >&2
      return 1
    }
    op run --env-file="$INTAKE_OP_ENV_FILE" -- env INTAKE_OP_WRAPPED=1 "$ROOT/intake/scripts/iteration-checkpoints.sh"
  else
    "$ROOT/intake/scripts/iteration-checkpoints.sh"
  fi
}

"$ROOT/intake/scripts/feedback-loop-signal.sh" clear 2>/dev/null || true
"$ROOT/intake/scripts/feedback-loop-signal.sh" start --message "go-green-loop (sleep ${sleep_sec}s between tries)"

attempt=1
while true; do
  if [[ "$max_attempts" -ge 0 ]] && (( attempt > max_attempts )); then
    break
  fi
  printf '\n\033[1m=== go-green-loop attempt %s ===\033[0m\n' "$attempt" >&2
  if [[ "$max_attempts" -ge 0 ]]; then
    printf '(max %s attempts, %ss pause on fail)\n' "$max_attempts" "$sleep_sec" >&2
  else
    printf '(INTAKE_GO_GREEN_FOREVER, %ss pause on fail)\n' "$sleep_sec" >&2
  fi

  if run_iteration; then
    printf '\n\033[1mgo-green-loop: CHECKPOINTS GREEN\033[0m\n\n' >&2
    "$ROOT/intake/scripts/feedback-loop-signal.sh" clear 2>/dev/null || true
    exit 0
  fi

  attempt=$((attempt + 1))
  if [[ "$max_attempts" -ge 0 ]] && (( attempt > max_attempts )); then
    break
  fi
  echo "go-green-loop: sleeping ${sleep_sec}s before next attempt..." >&2
  sleep "$sleep_sec"
done

"$ROOT/intake/scripts/feedback-loop-signal.sh" broken --message "go-green-loop: no green after ${max_attempts} attempts (${sleep_sec}s spacing); fix bridges/secrets/kube + op env then re-run"
exit 1
