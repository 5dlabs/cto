#!/bin/bash
set -euo pipefail

# Backwards-compatible shim that points to the new generator location.
THIS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$THIS_DIR/.." && pwd)"

TARGET_SCRIPT="$ROOT_DIR/infra/charts/controller/scripts/generate-templates-configmap.sh"

if [ ! -f "$TARGET_SCRIPT" ]; then
  echo "❌ Unable to locate generator at $TARGET_SCRIPT" >&2
  exit 1
fi

exec "$TARGET_SCRIPT" "$@"
