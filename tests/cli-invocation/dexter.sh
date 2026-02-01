#!/usr/bin/env bash
# =========================================================================
# Dexter CLI Invocation Test Script
# Combined from: templates/clis/dexter.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/dexter.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               DEXTER CLI INVOCATION                          ║"
echo "═══════════════════════════════════════════════════════════════"

WORK_DIR="${WORK_DIR:-/tmp/dexter-test}"
mkdir -p "$WORK_DIR"

# Build Dexter command
DEXTER_CMD=("dexter" "run")
DEXTER_CMD+=("--directory" "$WORK_DIR")
DEXTER_CMD+=("--auto")
DEXTER_CMD+=("--json")

echo "✓ Auto mode enabled"
echo "✓ Working dir: $WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory.}"

# Stream output
STREAM_OUTPUT="$WORK_DIR/dexter-stream.json"

echo ""
echo "🚀 Dexter Command: dexter run --directory $WORK_DIR --auto --json"
echo ""

# Execute with prompt as argument
"${DEXTER_CMD[@]}" "$USER_PROMPT" 2>&1 | tee "$STREAM_OUTPUT"
DEXTER_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "✓ Dexter execution completed (exit code: $DEXTER_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $DEXTER_EXIT_CODE
