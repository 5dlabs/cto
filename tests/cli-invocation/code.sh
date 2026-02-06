#!/usr/bin/env bash
# =========================================================================
# GitHub Copilot Code CLI Invocation Test Script
# Combined from: templates/clis/code.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/code.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               GITHUB COPILOT CODE CLI INVOCATION             ║"
echo "═══════════════════════════════════════════════════════════════"

WORK_DIR="${WORK_DIR:-/tmp/code-test}"
mkdir -p "$WORK_DIR"

# Build Code command (GitHub Copilot CLI)
CODE_CMD=("gh" "copilot" "suggest")
CODE_CMD+=("--shell")

echo "✓ Working dir: $WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory.}"

# Stream output
STREAM_OUTPUT="$WORK_DIR/code-stream.jsonl"

echo ""
echo "🚀 Code Command: gh copilot suggest --shell"
echo ""

# Execute with prompt as argument
cd "$WORK_DIR"
"${CODE_CMD[@]}" "$USER_PROMPT" 2>&1 | tee "$STREAM_OUTPUT"
CODE_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "✓ Code execution completed (exit code: $CODE_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $CODE_EXIT_CODE
