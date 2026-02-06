#!/bin/bash
# =============================================================================
# Run All Agents - Creates agent sessions for all 14 CTO agents
# =============================================================================
#
# Runs each of the 14 agents sequentially, each creating its own Linear session.
# All agents use Claude CLI with their specific skills/tools configuration.
#
# Usage: ./run-all-agents.sh [issue-id]
#   Examples:
#     ./run-all-agents.sh              # Use LINEAR_ISSUE_IDENTIFIER from .env
#     ./run-all-agents.sh CTOPA-2682   # Override with specific issue
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Load environment
source .env

# Override issue if provided
if [[ -n "${1:-}" ]]; then
    LINEAR_ISSUE_IDENTIFIER="$1"
    export LINEAR_ISSUE_IDENTIFIER
fi

# All 14 agents from cto-config.json
AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

echo "=============================================="
echo "  🚀 Running ALL ${#AGENTS[@]} Agents"
echo "  📋 Linear Issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
echo ""

# Verify all agents have skills
echo "📚 Verifying agent configurations..."
for agent in "${AGENTS[@]}"; do
    if [[ ! -d "config/skills-${agent}" ]]; then
        echo "⚠️  Missing skills for ${agent}, running scaffold..."
        ./scaffold-skills.sh "${agent}" 2>/dev/null || true
    fi
    if [[ ! -f "config/client-config-${agent}.json" ]]; then
        echo "⚠️  Missing client-config for ${agent}, running scaffold..."
        ./scaffold-agents.sh 2>/dev/null || true
    fi
done
echo ""

# Count totals
TOTAL=${#AGENTS[@]}
CURRENT=0
SUCCESS=0
FAILED=0

for agent in "${AGENTS[@]}"; do
    CURRENT=$((CURRENT + 1))
    
    # Get counts
    SKILLS_COUNT=$(ls -1 "config/skills-${agent}" 2>/dev/null | wc -l | tr -d ' ')
    TOOLS_COUNT=$(jq '.remoteTools | length' "config/client-config-${agent}.json" 2>/dev/null || echo "0")
    
    AGENT_UPPER=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    echo "=============================================="
    echo "  [${CURRENT}/${TOTAL}] ${AGENT_UPPER}"
    echo "  📚 Skills: ${SKILLS_COUNT} | 🔧 Tools: ${TOOLS_COUNT}"
    echo "=============================================="
    
    # Clean workspace
    rm -rf "workspaces/${agent}" 2>/dev/null || true
    mkdir -p "workspaces/${agent}"
    
    # Copy agent's task prompt
    if [[ -f "config/task-${agent}/prompt.md" ]]; then
        cp "config/task-${agent}/prompt.md" "workspaces/${agent}/prompt.md"
    else
        # Create minimal prompt
        echo "# Agent Test: ${agent}" > "workspaces/${agent}/prompt.md"
        echo "" >> "workspaces/${agent}/prompt.md"
        echo "Say hello and confirm you are ${agent} with your configured skills and tools." >> "workspaces/${agent}/prompt.md"
    fi
    
    # Run via docker compose with agent-specific config
    # Using the bolt pattern but with agent-specific mounts
    echo "🐳 Starting ${agent}..."
    
    if docker run --rm \
        --network host \
        --env-file .env \
        -e CLI_WORK_DIR=/workspace \
        -e MCP_CLIENT_CONFIG=/config/client-config.json \
        -e TOOLS_URL=http://tools.fra.5dlabs.ai/mcp \
        -v "${PWD}/workspaces/${agent}:/workspace" \
        -v "${PWD}/config/claude-settings.json:/home/node/.claude/settings.json:ro" \
        -v "${PWD}/config/skills-${agent}:/home/node/.claude/skills:ro" \
        -v "${PWD}/config/client-config-${agent}.json:/config/client-config.json:ro" \
        -v "${PWD}/scripts/run-claude.sh:/run.sh:ro" \
        -v /var/run/docker.sock:/var/run/docker.sock:ro \
        -w /workspace \
        cto-claude:local \
        bash /run.sh 2>&1; then
        
        echo "✅ ${agent} CLI completed"
        
        # Run sidecar to post to Linear
        echo "📤 Posting ${agent} session to Linear..."
        if docker run --rm \
            --network host \
            -e LINEAR_OAUTH_TOKEN="${LINEAR_OAUTH_TOKEN}" \
            -e LINEAR_ISSUE_IDENTIFIER="${LINEAR_ISSUE_IDENTIFIER}" \
            -e CLI_TYPE=claude \
            -e STREAM_FILE=/workspace/stream.jsonl \
            -e DEBUG_JSONL_PATH=/workspace/debug.jsonl \
            -e CTO_SKILLS_DIR=/cto-skills \
            -e CTO_CONFIG_PATH=/cto-config.json \
            -e CTO_AGENT_NAME="${agent}" \
            -e RUST_LOG=info \
            -v "${PWD}/workspaces/${agent}:/workspace" \
            -v "${PWD}/../../templates/skills:/cto-skills:ro" \
            -v "${PWD}/../../cto-config.json:/cto-config.json:ro" \
            cto-linear-sidecar:local 2>&1; then
            
            echo "✅ ${agent} session posted!"
            SUCCESS=$((SUCCESS + 1))
        else
            echo "❌ ${agent} sidecar failed"
            FAILED=$((FAILED + 1))
        fi
    else
        echo "❌ ${agent} CLI failed or timed out"
        FAILED=$((FAILED + 1))
    fi
    
    echo ""
    
    # Brief pause between agents
    if [[ $CURRENT -lt $TOTAL ]]; then
        sleep 2
    fi
done

echo "=============================================="
echo "  🏁 All Agents Complete!"
echo "  ✅ Success: ${SUCCESS}/${TOTAL}"
echo "  ❌ Failed: ${FAILED}/${TOTAL}"
echo "  📋 Linear: https://linear.app/jonathonfritz/issue/${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
