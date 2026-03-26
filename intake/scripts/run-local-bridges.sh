#!/usr/bin/env bash
# Run discord-bridge (:3200) and linear-bridge (:3100) locally under `op run`.
#
# Default: one terminal — both processes log to intake/.bridge-logs/*.log and this script
# runs `tail -f` on both so you get macOS-style file headers and one scrolling view.
#
# - Linear auth: OAuth access token in LINEAR_API_KEY (not lin_api_*).
# - Team ID: resolved from defaults.linear.teamId in cto-config.json via GraphQL.
# - Discord: DISCORD_BRIDGE_TOKEN from OpenClaw pooled tokens (intake bot).
#
# Usage (repo root):
#   ./intake/scripts/run-local-bridges.sh              # foreground + tail -f (monitor here)
#   ./intake/scripts/run-local-bridges.sh --no-tail   # no tail (logs still written to files)
#   ./intake/scripts/run-local-bridges.sh --detach    # spawn and exit (you tail logs yourself)
#
# Env:
#   INTAKE_OP_ENV_FILE   (default: intake/local.env.op)
#   ACP_ACTIVITY_ENABLED (default: false)
#   DISCORD_DELIBERATION_CHANNEL_ID optional fixed channel for recipient=deliberation
#   INTAKE_BRIDGE_LOG_DIR override log directory

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
ENVF="${INTAKE_OP_ENV_FILE:-$ROOT/intake/local.env.op}"
LOGD="${INTAKE_BRIDGE_LOG_DIR:-$ROOT/intake/.bridge-logs}"

no_tail=""
detach=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-tail) no_tail=1; shift ;;
    --detach) detach=1; shift ;;
    -h | --help)
      sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) echo "run-local-bridges.sh: unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -f "$ENVF" ]] || {
  echo "run-local-bridges.sh: missing $ENVF" >&2
  exit 1
}
command -v op >/dev/null 2>&1 || {
  echo "run-local-bridges.sh: op not on PATH" >&2
  exit 1
}
command -v jq >/dev/null 2>&1 || {
  echo "run-local-bridges.sh: jq required" >&2
  exit 1
}

# Prevent duplicate bridge processes that cause stale interactions and EADDRINUSE.
pkill -f "apps/discord-bridge" 2>/dev/null || true
pkill -f "apps/linear-bridge" 2>/dev/null || true

mkdir -p "$LOGD"
: >"$LOGD/discord-bridge.log"
: >"$LOGD/linear-bridge.log"

export DISCORD_BRIDGE_URL="${DISCORD_BRIDGE_URL:-http://127.0.0.1:3200}"
export LINEAR_BRIDGE_URL="${LINEAR_BRIDGE_URL:-http://127.0.0.1:3100}"
export ACP_ACTIVITY_ENABLED="${ACP_ACTIVITY_ENABLED:-false}"
export HTTP_PORT="${HTTP_PORT:-3200}"
export WEBHOOK_PORT="${WEBHOOK_PORT:-3100}"
# Default to #intake local channel so Lobster/feedback-loop posts are visible where expected.
export DISCORD_DELIBERATION_CHANNEL_ID="${DISCORD_DELIBERATION_CHANNEL_ID:-1471014430065164461}"

TEAM_ID=$(op run --env-file="$ENVF" -- "$ROOT/intake/scripts/linear-resolve-team-id.sh")
export LINEAR_TEAM_ID="$TEAM_ID"
echo "run-local-bridges.sh: LINEAR_TEAM_ID=$LINEAR_TEAM_ID — logs under $LOGD" >&2

cleanup() {
  [[ -n "${TAIL_PID:-}" ]] && kill "$TAIL_PID" 2>/dev/null || true
  [[ -n "${DISCORD_PID:-}" ]] && kill "$DISCORD_PID" 2>/dev/null || true
  [[ -n "${LINEAR_PID:-}" ]] && kill "$LINEAR_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "run-local-bridges.sh: starting discord-bridge (port ${HTTP_PORT}) → $LOGD/discord-bridge.log" >&2
op run --env-file="$ENVF" -- bash -lc "cd \"$ROOT/apps/discord-bridge\" && exec npx --yes tsx src/index.ts" >>"$LOGD/discord-bridge.log" 2>&1 &
DISCORD_PID=$!

sleep 3

if ! curl -sf --max-time 3 "${DISCORD_BRIDGE_URL%/}/health" >/dev/null; then
  echo "run-local-bridges.sh: warning: discord /health not 200 yet — check $LOGD/discord-bridge.log" >&2
fi

echo "run-local-bridges.sh: starting linear-bridge (port ${WEBHOOK_PORT}) → $LOGD/linear-bridge.log" >&2
op run --env-file="$ENVF" -- bash -lc "cd \"$ROOT/apps/linear-bridge\" && exec npx --yes tsx src/index.ts" >>"$LOGD/linear-bridge.log" 2>&1 &
LINEAR_PID=$!

if [[ -n "$detach" ]]; then
  echo "run-local-bridges.sh: detached. PIDs discord=$DISCORD_PID linear=$LINEAR_PID" >&2
  echo "run-local-bridges.sh: monitor: tail -f \"$LOGD/discord-bridge.log\" \"$LOGD/linear-bridge.log\"" >&2
  disown "$DISCORD_PID" 2>/dev/null || true
  disown "$LINEAR_PID" 2>/dev/null || true
  trap - EXIT INT TERM
  exit 0
fi

if [[ -z "$no_tail" ]]; then
  echo "run-local-bridges.sh: monitoring (Ctrl+C stops bridges + tail)…" >&2
  tail -f "$LOGD/discord-bridge.log" "$LOGD/linear-bridge.log" &
  TAIL_PID=$!
else
  echo "run-local-bridges.sh: --no-tail — follow logs in $LOGD" >&2
fi

wait "$DISCORD_PID" "$LINEAR_PID"
