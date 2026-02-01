#!/usr/bin/env bash
# =========================================================================
# OpenCode CLI Invocation Test Script
# Combined from: templates/clis/opencode.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/opencode.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               OPENCODE CLI INVOCATION                        ║"
echo "═══════════════════════════════════════════════════════════════"

WORK_DIR="${WORK_DIR:-/tmp/opencode-test}"
mkdir -p "$WORK_DIR"

# Build OpenCode command
OPENCODE_CMD=("opencode")
OPENCODE_CMD+=("--directory" "$WORK_DIR")
OPENCODE_CMD+=("--auto-approve")
OPENCODE_CMD+=("--json")

echo "✓ Auto-approve enabled"
echo "✓ Working dir: $WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory.}"

# Stream output
STREAM_OUTPUT="$WORK_DIR/opencode-stream.jsonl"

echo ""
echo "🚀 OpenCode Command: opencode --directory $WORK_DIR --auto-approve --json"
echo ""

# Execute with prompt via stdin
echo "$USER_PROMPT" | "${OPENCODE_CMD[@]}" 2>&1 | tee "$STREAM_OUTPUT"
OPENCODE_EXIT_CODE=${PIPESTATUS[1]}

echo ""
echo "✓ OpenCode execution completed (exit code: $OPENCODE_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $OPENCODE_EXIT_CODE
