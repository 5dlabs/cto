#!/usr/bin/env bash
# =========================================================================
# Gemini CLI Invocation Test Script
# Combined from: templates/clis/gemini.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/gemini.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               GEMINI CLI INVOCATION                          ║"
echo "═══════════════════════════════════════════════════════════════"

WORK_DIR="${WORK_DIR:-/tmp/gemini-test}"
mkdir -p "$WORK_DIR"

# Build Gemini command  
GEMINI_CMD=("gemini")
GEMINI_CMD+=("--directory" "$WORK_DIR")
GEMINI_CMD+=("--yolo")  # Auto-approve all actions
GEMINI_CMD+=("--json")

echo "✓ YOLO mode enabled (auto-approve)"
echo "✓ Working dir: $WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory.}"

# Stream output
STREAM_OUTPUT="$WORK_DIR/gemini-stream.jsonl"

echo ""
echo "🚀 Gemini Command: gemini --directory $WORK_DIR --yolo --json"
echo ""

# Execute with prompt as argument
"${GEMINI_CMD[@]}" "$USER_PROMPT" 2>&1 | tee "$STREAM_OUTPUT"
GEMINI_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "✓ Gemini execution completed (exit code: $GEMINI_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $GEMINI_EXIT_CODE
