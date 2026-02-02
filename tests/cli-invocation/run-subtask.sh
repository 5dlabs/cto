#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

AGENT="${1:-bolt}"
SUBTASK_ID="${2:-task-1.1}"

[[ -f .env ]] && source .env

: "${LINEAR_OAUTH_TOKEN:?LINEAR_OAUTH_TOKEN must be set}"
: "${LINEAR_ISSUE_IDENTIFIER:?LINEAR_ISSUE_IDENTIFIER must be set}"
: "${ANTHROPIC_API_KEY:?ANTHROPIC_API_KEY must be set}"

SUBTASK_DIR="config/task-${AGENT}/subtasks/${SUBTASK_ID}"
[[ ! -d "$SUBTASK_DIR" ]] && { echo "❌ Subtask not found: $SUBTASK_DIR"; exit 1; }

SUBTASK_TITLE=$(head -1 "${SUBTASK_DIR}/prompt.md" | sed 's/^#\s*//')

echo "=============================================="
echo "  Running Subtask: ${SUBTASK_ID}"
echo "  Title: ${SUBTASK_TITLE}"
echo "  Agent: ${AGENT}"
echo "  Linear Issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
echo ""

# Clean workspace
rm -f workspaces/${AGENT}/stream.jsonl workspaces/${AGENT}/prompt.md
mkdir -p workspaces/${AGENT}
cp "${SUBTASK_DIR}/prompt.md" "workspaces/${AGENT}/prompt.md"
[[ -f "${SUBTASK_DIR}/acceptance-criteria.md" ]] && cp "${SUBTASK_DIR}/acceptance-criteria.md" "workspaces/${AGENT}/"

export SUBTASK_ID SUBTASK_NAME="$SUBTASK_TITLE"

echo "🚀 Starting ${AGENT}..."

# Run agent first, then sidecar after (sidecar processes completed stream)
docker compose up ${AGENT} --abort-on-container-exit 2>&1

echo ""
echo "📡 Running sidecar to post results to Linear..."
docker compose up ${AGENT}-sidecar 2>&1

echo ""
echo "=============================================="
echo "  ✅ Subtask Complete!"
echo "  Check Linear: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="

if [[ -f "workspaces/${AGENT}/stream.jsonl" ]]; then
  echo ""
  echo "📊 Stream Summary:"
  wc -l "workspaces/${AGENT}/stream.jsonl"
fi
