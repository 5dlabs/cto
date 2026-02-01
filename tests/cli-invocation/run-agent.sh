#!/usr/bin/env bash
# =============================================================================
# Run Agent - Easy Agent Testing
# =============================================================================
#
# Runs a single agent with proper setup and Linear integration.
#
# Usage:
#   ./run-agent.sh bolt           # Run Bolt agent
#   ./run-agent.sh rex coder      # Run Rex with coder job type
#   ./run-agent.sh blaze          # Run Blaze agent
#
# Environment:
#   LINEAR_ISSUE_IDENTIFIER - The Linear issue to post results to (required)
#   LINEAR_OAUTH_TOKEN      - OAuth token for Linear API (from .env)
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

AGENT="${1:-bolt}"
JOB_TYPE="${2:-coder}"

echo "=============================================="
echo "🚀 Running Agent: ${AGENT}"
echo "   Job Type: ${JOB_TYPE}"
echo "=============================================="
echo ""

# Check .env exists
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found"
    echo "   Copy .env.example to .env and configure secrets"
    exit 1
fi

# Check LINEAR_ISSUE_IDENTIFIER is set
source .env
if [ -z "${LINEAR_ISSUE_IDENTIFIER:-}" ]; then
    echo "❌ Error: LINEAR_ISSUE_IDENTIFIER not set in .env"
    exit 1
fi

# Verify per-agent skills directory exists
if [ ! -d "config/skills-${AGENT}" ]; then
    echo "⚠️  Skills directory not found: config/skills-${AGENT}"
    echo "   Running scaffold-skills.sh..."
    ./scaffold-skills.sh "${AGENT}"
fi

# Verify per-agent client-config exists
if [ ! -f "config/client-config-${AGENT}.json" ]; then
    echo "⚠️  Client config not found: config/client-config-${AGENT}.json"
    echo "   Running scaffold-agents.sh..."
    ./scaffold-agents.sh
fi

# Check skills count
SKILLS_COUNT=$(ls -1 "config/skills-${AGENT}" 2>/dev/null | wc -l | tr -d ' ')
echo "📚 Skills: ${SKILLS_COUNT} loaded from config/skills-${AGENT}/"

# Check tools count
if [ -f "config/client-config-${AGENT}.json" ]; then
    TOOLS_COUNT=$(jq '.remoteTools | length' "config/client-config-${AGENT}.json" 2>/dev/null || echo "0")
    echo "🔧 Tools: ${TOOLS_COUNT} configured in client-config-${AGENT}.json"
fi

echo ""
echo "📋 Linear Issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo ""

# Clean previous workspace
echo "🧹 Cleaning workspace..."
rm -rf "workspaces/${AGENT}/stream.jsonl" "workspaces/${AGENT}/debug.jsonl" 2>/dev/null || true
mkdir -p "workspaces/${AGENT}"

# Run with docker compose profile
echo "🐳 Starting containers..."
echo ""

docker compose --profile "${AGENT}" up --abort-on-container-exit

echo ""
echo "=============================================="
echo "✅ Agent run complete"
echo "   Check Linear issue: https://linear.app/jonathonfritz/issue/${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
