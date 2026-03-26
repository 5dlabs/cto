#!/usr/bin/env bash
# Smoke test: Discord + Linear bridges in parallel (same payloads as intake-util bridge-notify).
#
# Prerequisites:
#   - ./intake/scripts/run-local-bridges.sh --detach   (or cluster bridges reachable)
#   - intake-util on PATH, or set INTAKE_UTIL to the binary path
#
# Env:
#   DISCORD_BRIDGE_URL  (default http://127.0.0.1:3200)
#   LINEAR_BRIDGE_URL   (default http://127.0.0.1:3100)
#
# Usage (repo root):
#   ./intake/scripts/bridge-dual-smoke.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

DISCORD_BRIDGE_URL="${DISCORD_BRIDGE_URL:-http://127.0.0.1:3200}"
LINEAR_BRIDGE_URL="${LINEAR_BRIDGE_URL:-http://127.0.0.1:3100}"
export DISCORD_BRIDGE_URL LINEAR_BRIDGE_URL

INTAKE_UTIL="${INTAKE_UTIL:-intake-util}"
command -v "$INTAKE_UTIL" >/dev/null 2>&1 || INTAKE_UTIL="$ROOT/apps/intake-util/intake-util"
if [[ ! -x "$INTAKE_UTIL" ]] && command -v "$ROOT/apps/intake-util/intake-util" >/dev/null 2>&1; then
  INTAKE_UTIL="$ROOT/apps/intake-util/intake-util"
fi

echo "=== Health ==="
curl -sf "${DISCORD_BRIDGE_URL%/}/health" | jq -c . || { echo "Discord bridge down" >&2; exit 1; }
curl -sf "${LINEAR_BRIDGE_URL%/}/health" | jq -c . || { echo "Linear bridge down" >&2; exit 1; }

echo ""
echo "=== Marker (no session_id) — both should accept; Discord posts to #intake parent ==="
echo "**Smoke:** pipeline marker (no session)" | "$INTAKE_UTIL" bridge-notify --from intake --to deliberation \
  --metadata '{"step":"dual-smoke","project_name":"bridge-dual-smoke"}' | jq -c .

SID="dual-smoke-$(date +%s)"
echo ""
echo "=== Session thread flow (Discord: deliberation-start in parent + link; turn in thread) ==="
echo "Session id: $SID"

echo "**deliberation-start** (marker + thread link in #intake)" | "$INTAKE_UTIL" bridge-notify --from intake --to deliberation \
  --metadata "{\"step\":\"deliberation-start\",\"session_id\":\"$SID\"}" | jq -c .

echo "**optimist turn** (same session — Discord → thread; Linear → agent session issue)" | "$INTAKE_UTIL" bridge-notify --from optimist --to deliberation \
  --metadata "{\"session_id\":\"$SID\",\"speaker\":\"optimist\",\"turn\":\"1\",\"step\":\"dual-smoke-turn\"}" | jq -c .

echo ""
echo "=== Done ==="
echo "Check Discord #intake for deliberation-start + thread link; open thread for optimist line."
echo "Check Linear for comments on the conversation issue (or agent session issue if configured)."
echo "Discord log (local): tail -f \"$ROOT/intake/.bridge-logs/discord-bridge.log\""
