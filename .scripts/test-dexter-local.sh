#!/bin/bash
# =========================================================================
# Test Dexter Agent Locally
#
# Usage:
#   ./scripts/test-dexter-local.sh [query]
#
# Examples:
#   ./scripts/test-dexter-local.sh
#   ./scripts/test-dexter-local.sh "What is Apple's current market cap?"
#
# Environment Variables:
#   OPENAI_API_KEY          - Required (or ANTHROPIC_API_KEY/GOOGLE_API_KEY)
#   FINANCIAL_DATASETS_API_KEY - Recommended for financial research
#   DEXTER_MODEL            - Model to use (default: gpt-4.1)
# =========================================================================
set -euo pipefail

# Check for required API key
if [ -z "${OPENAI_API_KEY:-}" ] && [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${GOOGLE_API_KEY:-}" ]; then
    echo "❌ ERROR: No LLM API key found"
    echo ""
    echo "Set one of the following environment variables:"
    echo "   export OPENAI_API_KEY=sk-..."
    echo "   export ANTHROPIC_API_KEY=sk-..."
    echo "   export GOOGLE_API_KEY=..."
    exit 1
fi

# Default test query
DEFAULT_QUERY="What was Apple's revenue growth over the last 4 quarters?"
TEST_QUERY="${1:-$DEFAULT_QUERY}"

echo "🧪 Testing Dexter Agent"
echo "   Query: $TEST_QUERY"
echo ""

# Check if local image exists
if ! docker image inspect ghcr.io/5dlabs/dexter:local >/dev/null 2>&1; then
    echo "⚠️ Local image not found. Building..."
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    "$SCRIPT_DIR/build-dexter-image.sh"
    echo ""
fi

# Build environment variables for docker
ENV_ARGS=""
[ -n "${OPENAI_API_KEY:-}" ] && ENV_ARGS="$ENV_ARGS -e OPENAI_API_KEY=$OPENAI_API_KEY"
[ -n "${ANTHROPIC_API_KEY:-}" ] && ENV_ARGS="$ENV_ARGS -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY"
[ -n "${GOOGLE_API_KEY:-}" ] && ENV_ARGS="$ENV_ARGS -e GOOGLE_API_KEY=$GOOGLE_API_KEY"
[ -n "${FINANCIAL_DATASETS_API_KEY:-}" ] && ENV_ARGS="$ENV_ARGS -e FINANCIAL_DATASETS_API_KEY=$FINANCIAL_DATASETS_API_KEY"
[ -n "${LANGSMITH_API_KEY:-}" ] && ENV_ARGS="$ENV_ARGS -e LANGSMITH_API_KEY=$LANGSMITH_API_KEY -e LANGSMITH_TRACING=true"
[ -n "${DEXTER_MODEL:-}" ] && ENV_ARGS="$ENV_ARGS -e DEXTER_MODEL=$DEXTER_MODEL"

# Run Dexter in single-query mode with reduced step limits for testing
echo "🚀 Running Dexter..."
echo ""

# shellcheck disable=SC2086
docker run -it --rm \
  $ENV_ARGS \
  -e DEXTER_MAX_STEPS=10 \
  -e DEXTER_MAX_STEPS_PER_TASK=3 \
  ghcr.io/5dlabs/dexter:local \
  bash -c "printf '%s' '$TEST_QUERY' | python3 -c '
import os
import sys
from dexter.agent import Agent

query = sys.stdin.read().strip()
model = os.environ.get(\"DEXTER_MODEL\", \"claude-sonnet-4-20250514\")
max_steps = int(os.environ.get(\"DEXTER_MAX_STEPS\", \"10\"))
max_steps_per_task = int(os.environ.get(\"DEXTER_MAX_STEPS_PER_TASK\", \"3\"))

print(f\"Model: {model}\", file=sys.stderr)
print(f\"Query: {query[:80]}...\", file=sys.stderr)
print(\"\", file=sys.stderr)

agent = Agent(max_steps=max_steps, max_steps_per_task=max_steps_per_task, model=model)
result = agent.run(query)
print(result)
'"

echo ""
echo "✅ Test complete"

