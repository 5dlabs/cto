#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_PATH="$ROOT_DIR/intake/scripts/playwright-intake-observer.mjs"

if ! command -v node >/dev/null 2>&1; then
  echo "Error: node is required but not found on PATH." >&2
  exit 1
fi

for arg in "$@"; do
  if [[ "$arg" == "--help" ]]; then
    exec node "$SCRIPT_PATH" "$@"
  fi
done

if [[ ! -d "$ROOT_DIR/node_modules/playwright" ]]; then
  cat >&2 <<'EOF'
Error: playwright is not installed in this checkout.

Run these once from the repo root:
  npm install
  npx playwright install chromium
EOF
  exit 1
fi

exec node "$SCRIPT_PATH" "$@"
