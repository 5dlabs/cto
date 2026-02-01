#!/usr/bin/env bash
# =========================================================================
# Cursor CLI Invocation Test Script
# Combined from: templates/clis/cursor.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/cursor.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               CURSOR CLI INVOCATION                          ║"
echo "═══════════════════════════════════════════════════════════════"

WORK_DIR="${WORK_DIR:-/tmp/cursor-test}"
mkdir -p "$WORK_DIR"

# Build Cursor command
CURSOR_CMD=("cursor" "chat")
CURSOR_CMD+=("--directory" "$WORK_DIR")
CURSOR_CMD+=("--json")

echo "✓ Working dir: $WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory.}"

# Stream output
STREAM_OUTPUT="$WORK_DIR/cursor-stream.jsonl"

echo ""
echo "🚀 Cursor Command: cursor chat --directory $WORK_DIR --json"
echo ""

# Execute with prompt as argument
"${CURSOR_CMD[@]}" "$USER_PROMPT" 2>&1 | tee "$STREAM_OUTPUT"
CURSOR_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "✓ Cursor execution completed (exit code: $CURSOR_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $CURSOR_EXIT_CODE
