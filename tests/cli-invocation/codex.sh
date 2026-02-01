#!/usr/bin/env bash
# =========================================================================
# OpenAI Codex CLI Invocation Test Script
# Combined from: templates/clis/codex.sh.hbs
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/codex.sh
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               OPENAI CODEX CLI INVOCATION                    ║"
echo "═══════════════════════════════════════════════════════════════"

CODEX_WORK_DIR="${CODEX_WORK_DIR:-/tmp/codex-test}"
mkdir -p "$CODEX_WORK_DIR"

# Build Codex command
CODEX_CMD=("codex" "exec")
CODEX_CMD+=("--cd" "$CODEX_WORK_DIR")
CODEX_CMD+=("--full-auto")
CODEX_CMD+=("--json")

echo "✓ Full auto mode enabled"
echo "✓ Working dir: $CODEX_WORK_DIR"

# Prompt
USER_PROMPT="${PROMPT:-List files in the current directory and report the count.}"
CODEX_CMD+=("$USER_PROMPT")

# Stream output
STREAM_OUTPUT="$CODEX_WORK_DIR/codex-stream.jsonl"

echo ""
echo "🚀 Codex Command: codex exec --cd $CODEX_WORK_DIR --full-auto --json \"<prompt>\""
echo ""

# Execute
"${CODEX_CMD[@]}" 2>&1 | tee "$STREAM_OUTPUT"
CODEX_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "✓ Codex execution completed (exit code: $CODEX_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"

exit $CODEX_EXIT_CODE
