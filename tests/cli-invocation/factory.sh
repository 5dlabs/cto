#!/usr/bin/env bash
# =========================================================================
# Factory (Droid) CLI Invocation Test Script
# Combined from: templates/clis/factory.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/factory.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               FACTORY CLI INVOCATION                         ║"
echo "═══════════════════════════════════════════════════════════════"

WORK_DIR="${WORK_DIR:-/tmp/factory-test}"
mkdir -p "$WORK_DIR"

# Build Factory command
FACTORY_CMD=("droid" "run")
FACTORY_CMD+=("--directory" "$WORK_DIR")
FACTORY_CMD+=("--non-interactive")
FACTORY_CMD+=("--json")

echo "✓ Non-interactive mode"
echo "✓ Working dir: $WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory.}"

# Stream output
STREAM_OUTPUT="$WORK_DIR/factory-stream.jsonl"

echo ""
echo "🚀 Factory Command: droid run --directory $WORK_DIR --non-interactive --json"
echo ""

# Execute with prompt as argument
"${FACTORY_CMD[@]}" "$USER_PROMPT" 2>&1 | tee "$STREAM_OUTPUT"
FACTORY_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "✓ Factory execution completed (exit code: $FACTORY_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $FACTORY_EXIT_CODE
