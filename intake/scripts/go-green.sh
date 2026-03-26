#!/usr/bin/env bash
# One-shot: clear loop state → feedback-loop start → iteration-checkpoints (preflight + Linear GraphQL).
#
# Usage (repo root or any cwd; discovers CTO root from script path):
#   ./intake/scripts/go-green.sh
#   ./intake/scripts/go-green.sh --bridges-skip   # skip Discord/Linear bridge /health only (see pipeline-preflight.sh)
#
# Env: same as pipeline-preflight + iteration-checkpoints (LINEAR_API_KEY, DISCORD_BRIDGE_URL, …).
# Checkpoints auto-use `op run` when intake/local.env.op exists (see intake/local.env.op.example).
# Prepends apps/intake-util to PATH when the repo-built binary exists.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export WORKSPACE="${WORKSPACE:-$ROOT}"
cd "$ROOT"

if [[ -x "$ROOT/apps/intake-util/intake-util" ]]; then
  export PATH="$ROOT/apps/intake-util:$PATH"
fi

usage() {
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

bridges_skip=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --bridges-skip) bridges_skip=1; shift ;;
    -h | --help) usage ;;
    *) echo "go-green.sh: unknown arg: $1" >&2; usage ;;
  esac
done

if [[ -n "$bridges_skip" ]]; then
  export INTAKE_PREFLIGHT_BRIDGES_SKIP=true
fi

"$ROOT/intake/scripts/feedback-loop-signal.sh" clear 2>/dev/null || true
"$ROOT/intake/scripts/feedback-loop-signal.sh" start --message "go-green"
"$ROOT/intake/scripts/iteration-checkpoints.sh"
