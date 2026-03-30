#!/usr/bin/env bash
# Materialize gitignored local.env.op from defaults (same idea as intake/ensure-local-env-op.sh).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEFAULTS="$ROOT/google-oauth.env.op.defaults"
TARGET="$ROOT/local.env.op"
if [[ -f "$TARGET" ]]; then
  echo "Already exists: $TARGET"
  exit 0
fi
if [[ ! -f "$DEFAULTS" ]]; then
  echo "Missing: $DEFAULTS" >&2
  exit 1
fi
cp "$DEFAULTS" "$TARGET"
echo "Created $TARGET — edit op://YOUR_VAULT/... then run:"
echo "  op run --env-file=$TARGET -- npm run dev"
