#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET="$SCRIPT_DIR/generate-templates-configmaps-split.sh"

if [ ! -f "$TARGET" ]; then
  echo "❌ Unable to locate generator at $TARGET" >&2
  exit 1
fi

exec "$TARGET" "$@"
