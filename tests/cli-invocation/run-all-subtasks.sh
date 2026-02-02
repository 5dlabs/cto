#!/usr/bin/env bash
# =============================================================================
# Run All Subtasks - Each as Separate Agent Session
# =============================================================================
#
# Runs each subtask in task-bolt sequentially, creating separate Linear
# agent sessions for each one.
#
# Usage: ./run-all-subtasks.sh [subtask-pattern]
#   Examples:
#     ./run-all-subtasks.sh           # Run all subtasks
#     ./run-all-subtasks.sh task-1.1  # Run only task-1.1
#     ./run-all-subtasks.sh task-1    # Run all task-1.x subtasks
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Load environment
if [[ -f .env ]]; then
  source .env
fi

# Required env vars
: "${LINEAR_OAUTH_TOKEN:?LINEAR_OAUTH_TOKEN must be set}"
: "${LINEAR_ISSUE_IDENTIFIER:?LINEAR_ISSUE_IDENTIFIER must be set}"
: "${ANTHROPIC_API_KEY:?ANTHROPIC_API_KEY must be set}"

# Pattern filter (optional)
PATTERN="${1:-task-}"

echo "=============================================="
echo "  Running Subtasks Matching: ${PATTERN}*"
echo "  Linear Issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
echo ""

# Find all matching subtasks
SUBTASKS_DIR="config/task-bolt/subtasks"
SUBTASKS=$(ls -d ${SUBTASKS_DIR}/${PATTERN}* 2>/dev/null | sort)

if [[ -z "$SUBTASKS" ]]; then
  echo "❌ No subtasks found matching pattern: ${PATTERN}"
  exit 1
fi

# Count subtasks
TOTAL=$(echo "$SUBTASKS" | wc -l | tr -d ' ')
CURRENT=0

echo "Found ${TOTAL} subtask(s) to run:"
for subtask_dir in $SUBTASKS; do
  subtask_id=$(basename "$subtask_dir")
  title=$(head -1 "${subtask_dir}/prompt.md" | sed 's/^#\s*//')
  echo "  - ${subtask_id}: ${title}"
done
echo ""

# Run each subtask
for subtask_dir in $SUBTASKS; do
  CURRENT=$((CURRENT + 1))
  SUBTASK_ID=$(basename "$subtask_dir")
  SUBTASK_TITLE=$(head -1 "${subtask_dir}/prompt.md" | sed 's/^#\s*//')
  
  echo "=============================================="
  echo "  [${CURRENT}/${TOTAL}] ${SUBTASK_ID}"
  echo "  ${SUBTASK_TITLE}"
  echo "=============================================="
  
  # Clean workspace
  echo "🧹 Cleaning workspace..."
  rm -f workspaces/claude/stream.jsonl
  rm -f workspaces/claude/prompt.md
  rm -f workspaces/claude/debug.jsonl
  
  # Copy prompt to workspace
  echo "📋 Copying prompt from ${subtask_dir}/prompt.md"
  cp "${subtask_dir}/prompt.md" workspaces/claude/prompt.md
  
  # Export subtask info for sidecar
  export SUBTASK_ID
  export SUBTASK_NAME="$SUBTASK_TITLE"
  
  # Run docker-compose with timeout
  echo "🚀 Starting Claude + Sidecar..."
  echo "   (This will create a new agent session in Linear)"
  echo ""
  
  # Run with timeout (10 minutes per subtask)
  timeout 600 docker compose up claude claude-sidecar --abort-on-container-exit 2>&1 || {
    exit_code=$?
    if [[ $exit_code -eq 124 ]]; then
      echo "⏰ Subtask timed out after 10 minutes"
    else
      echo "⚠️  Subtask exited with code: $exit_code"
    fi
  }
  
  # Brief pause between subtasks
  echo ""
  echo "✅ Completed ${SUBTASK_ID}"
  
  if [[ $CURRENT -lt $TOTAL ]]; then
    echo "⏳ Waiting 5 seconds before next subtask..."
    sleep 5
  fi
  echo ""
done

echo "=============================================="
echo "  All Subtasks Complete!"
echo "  Check Linear issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
