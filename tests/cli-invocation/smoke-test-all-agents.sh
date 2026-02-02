#!/bin/bash
# =============================================================================
# Smoke Test - All 14 Agents - Quick Minimal Sessions
# =============================================================================
#
# Creates a minimal agent session for each of the 14 agents in Linear.
# Does NOT run full CLI - just posts init + completion to verify sidecar works.
#
# Usage: ./smoke-test-all-agents.sh [issue-id]
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

source .env

# Override issue if provided
if [[ -n "${1:-}" ]]; then
    LINEAR_ISSUE_IDENTIFIER="$1"
    export LINEAR_ISSUE_IDENTIFIER
fi

# All 14 agents
AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

echo "=============================================="
echo "  🔥 Smoke Test - ALL ${#AGENTS[@]} Agents"
echo "  📋 Linear Issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
echo ""

# Count
TOTAL=${#AGENTS[@]}
CURRENT=0
SUCCESS=0
FAILED=0

for agent in "${AGENTS[@]}"; do
    CURRENT=$((CURRENT + 1))
    
    # Get counts
    SKILLS_COUNT=$(ls -1 "config/skills-${agent}" 2>/dev/null | wc -l | tr -d ' ')
    TOOLS_COUNT=$(jq '.remoteTools | length' "config/client-config-${agent}.json" 2>/dev/null || echo "0")
    
    echo "[${CURRENT}/${TOTAL}] ${agent} (${SKILLS_COUNT} skills, ${TOOLS_COUNT} tools)"
    
    # Create workspace and minimal stream.jsonl
    mkdir -p "workspaces/${agent}"
    
    # Create mock init message
    SKILLS_JSON=$(ls "config/skills-${agent}" 2>/dev/null | jq -R -s 'split("\n") | map(select(length > 0))' || echo '[]')
    
    cat > "workspaces/${agent}/stream.jsonl" << EOF
{"type":"system","subtype":"init","cwd":"/workspace","session_id":"smoke-test-${agent}","tools":["Read","Write","Bash","Glob","Grep"],"mcp_servers":[{"name":"cto-tools","status":"ok"}],"model":"claude-sonnet-4-5-20250929","skills":${SKILLS_JSON},"uuid":"smoke-${agent}"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","id":"msg_smoke_${agent}","type":"message","role":"assistant","content":[{"type":"text","text":"Smoke test for ${agent} agent - verifying Linear integration."}],"stop_reason":"end_turn","usage":{"input_tokens":100,"output_tokens":20}},"session_id":"smoke-test-${agent}","uuid":"smoke-msg-${agent}"}
{"type":"result","subtype":"success","is_error":false,"duration_ms":1000,"num_turns":1,"result":"Smoke test complete for ${agent}","session_id":"smoke-test-${agent}","total_cost_usd":0.001,"uuid":"smoke-result-${agent}"}
EOF

    # Run sidecar to post to Linear
    if docker run --rm \
        --network host \
        -e LINEAR_OAUTH_TOKEN="${LINEAR_OAUTH_TOKEN}" \
        -e LINEAR_ISSUE_IDENTIFIER="${LINEAR_ISSUE_IDENTIFIER}" \
        -e CLI_TYPE=claude \
        -e STREAM_FILE=/workspace/stream.jsonl \
        -e CTO_SKILLS_DIR=/cto-skills \
        -e CTO_CONFIG_PATH=/cto-config.json \
        -e CTO_AGENT_NAME="${agent}" \
        -e RUST_LOG=warn \
        -v "${PWD}/workspaces/${agent}:/workspace:ro" \
        -v "${PWD}/../../templates/skills:/cto-skills:ro" \
        -v "${PWD}/../../cto-config.json:/cto-config.json:ro" \
        cto-linear-sidecar:local 2>&1 | grep -q "session created\|Posted"; then
        
        echo "  ✅ Posted to Linear"
        SUCCESS=$((SUCCESS + 1))
    else
        echo "  ❌ Failed to post"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=============================================="
echo "  🏁 Smoke Test Complete!"
echo "  ✅ Success: ${SUCCESS}/${TOTAL}"
echo "  ❌ Failed: ${FAILED}/${TOTAL}"
echo "  📋 Linear: https://linear.app/jonathonfritz/issue/${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
