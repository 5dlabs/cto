#!/usr/bin/env bash
# Run ordered checkpoints 2–3 from docs/intake-discord-feedback-loop.md (terminal-side).
# Checkpoint 1 and 5 are Discord visual — use browser MCP or human.
#
# Usage: from CTO repo root:
#   WORKSPACE=$PWD ./intake/scripts/iteration-checkpoints.sh
# If intake/local.env.op exists, automatically re-execs under `op run` (see intake/local.env.op.example).
#
# Exits non-zero on first failed checkpoint.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
"$ROOT/intake/scripts/ensure-local-env-op.sh" || true
# shellcheck source=intake/scripts/intake-op-auto.sh
source "$ROOT/intake/scripts/intake-op-auto.sh"
intake_op_auto_wrap "${BASH_SOURCE[0]}" "$@"
export WORKSPACE="${WORKSPACE:-$ROOT}"
cd "$ROOT"

echo "=== Checkpoint 2: pipeline-preflight.sh ===" >&2
"$ROOT/intake/scripts/pipeline-preflight.sh"

echo "=== Checkpoint 3: Linear viewer (no body printed) ===" >&2
code=$(curl -sS -o /tmp/linear_viewer_$$.json -w '%{http_code}' \
  -H "Authorization: Bearer ${LINEAR_API_KEY:?LINEAR_API_KEY required}" \
  -H "Content-Type: application/json" \
  -d '{"query":"{ viewer { id } }"}' \
  https://api.linear.app/graphql)
if [[ "$code" != "200" ]]; then
  echo "Linear GraphQL HTTP $code (see /tmp/linear_viewer_$$.json)" >&2
  echo "Hint: mint a fresh runtime token via PM and export LINEAR_API_KEY from Kubernetes, or use the temporary local.env.op fallback if you are still on the old bootstrap path." >&2
  exit 1
fi
if grep -q 'AUTHENTICATION_ERROR\|"errors"' /tmp/linear_viewer_$$.json 2>/dev/null; then
  echo "Linear returned GraphQL errors (see /tmp/linear_viewer_$$.json)" >&2
  echo "Hint: re-mint the runtime token via PM; if you are still using the local.env.op fallback, the cached pointer may be stale." >&2
  exit 1
fi
rm -f /tmp/linear_viewer_$$.json
echo "Checkpoints 2–3 OK. Next: Discord snapshot (checkpoint 1/5) then lobster run (checkpoint 4)." >&2
