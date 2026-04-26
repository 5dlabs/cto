#!/usr/bin/env bash
# Build, unit-test, and optionally smoke-test linear-bridge HTTP (health + run registration).
# Requires: Node 20+, npm. For smoke: LINEAR_API_KEY and LINEAR_TEAM_ID (team key or UUID).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/apps/linear-bridge"

echo "==> linear-bridge: npm ci"
npm ci

echo "==> linear-bridge: unit tests"
npm test

echo "==> linear-bridge: TypeScript build"
npm run build

if [[ -z "${LINEAR_API_KEY:-}" ]]; then
  echo ""
  echo "OK: build + tests passed. LINEAR_API_KEY not set — skipping HTTP smoke."
  echo "    Export LINEAR_API_KEY and LINEAR_TEAM_ID (e.g. from cto-config defaults.linear.teamId), then re-run."
  exit 0
fi

PORT="${VERIFY_LINEAR_BRIDGE_PORT:-13100}"
export ACP_ACTIVITY_ENABLED="${ACP_ACTIVITY_ENABLED:-false}"
export AGENT_SESSIONS_ENABLED="${AGENT_SESSIONS_ENABLED:-false}"
export WEBHOOK_PORT="$PORT"
export LINEAR_TEAM_ID="${LINEAR_TEAM_ID:-CTOPA}"

echo ""
echo "==> HTTP smoke on 127.0.0.1:${PORT} (ACP + agent sessions off for minimal deps)"

cleanup() {
  if [[ -n "${PID:-}" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

node dist/index.js &
PID=$!
sleep 2

curl -fsS "http://127.0.0.1:${PORT}/health" | jq .
curl -fsS -X POST "http://127.0.0.1:${PORT}/runs/verify-smoke/register" \
  -H 'Content-Type: application/json' \
  -d '{"agent":"intake","linearSessionId":"verify-session"}' | jq .

echo ""
echo "OK: linear-bridge HTTP health + POST /runs/:id/register succeeded."
