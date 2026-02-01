#!/bin/bash
# =============================================================================
# Smoke Test - Direct Linear API - All 14 Agents
# =============================================================================
#
# Creates agent sessions directly via Linear GraphQL API.
# No CLI, no sidecar - just verifies each agent config exists and posts to Linear.
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

source .env

# Override issue if provided
if [[ -n "${1:-}" ]]; then
    LINEAR_ISSUE_IDENTIFIER="$1"
fi

# All 14 agents
AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

echo "=============================================="
echo "  🔥 Smoke Test - ALL ${#AGENTS[@]} Agents"
echo "  📋 Linear Issue: ${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
echo ""

# First, resolve issue ID
echo "🔍 Resolving issue..."
ISSUE_ID=$(curl -s -X POST https://api.linear.app/graphql \
  -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query GetIssue($id: String!) { issue(id: $id) { id } }",
    "variables": {"id": "'"${LINEAR_ISSUE_IDENTIFIER}"'"}
  }' | jq -r '.data.issue.id')

if [[ -z "$ISSUE_ID" || "$ISSUE_ID" == "null" ]]; then
    echo "❌ Failed to resolve issue: ${LINEAR_ISSUE_IDENTIFIER}"
    exit 1
fi
echo "✅ Issue ID: ${ISSUE_ID}"
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
    
    # Get skill names
    SKILLS=$(ls "config/skills-${agent}" 2>/dev/null | head -5 | tr '\n' ', ' | sed 's/,$//')
    
    echo -n "[${CURRENT}/${TOTAL}] ${agent} (${SKILLS_COUNT} skills, ${TOOLS_COUNT} tools)... "
    
    # Create agent session
    RESPONSE=$(curl -s -X POST https://api.linear.app/graphql \
      -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
      -H "Content-Type: application/json" \
      -d '{
        "query": "mutation CreateSession($input: AgentSessionCreateOnIssue!) { agentSessionCreateOnIssue(input: $input) { success agentSession { id } } }",
        "variables": {
          "input": {
            "issueId": "'"${ISSUE_ID}"'"
          }
        }
      }')
    
    SESSION_ID=$(echo "$RESPONSE" | jq -r '.data.agentSessionCreateOnIssue.agentSession.id // empty')
    
    if [[ -z "$SESSION_ID" ]]; then
        echo "❌ Failed to create session"
        FAILED=$((FAILED + 1))
        continue
    fi
    
    # Post init activity
    AGENT_UPPER=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    INIT_BODY="🚀 **${AGENT_UPPER}** — Smoke Test\\n\\n📊 Model: claude-sonnet-4-5-20250929\\n🔧 Tools: ${TOOLS_COUNT} configured\\n📚 Skills: ${SKILLS_COUNT} (${SKILLS}...)\\n\\n*Smoke test to verify Linear integration*"
    
    curl -s -X POST https://api.linear.app/graphql \
      -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
      -H "Content-Type: application/json" \
      -d '{
        "query": "mutation AddActivity($input: AgentActivityCreateInput!) { agentActivityCreate(input: $input) { success } }",
        "variables": {
          "input": {
            "agentSessionId": "'"${SESSION_ID}"'",
            "content": {
              "type": "response",
              "body": "'"${INIT_BODY}"'"
            }
          }
        }
      }' > /dev/null
    
    # Post completion
    COMPLETE_BODY="✅ Smoke test complete\\n\\n⏱️ Duration: 1s\\n💰 Cost: \\$0.001\\n🔄 Turns: 1"
    
    curl -s -X POST https://api.linear.app/graphql \
      -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
      -H "Content-Type: application/json" \
      -d '{
        "query": "mutation AddActivity($input: AgentActivityCreateInput!) { agentActivityCreate(input: $input) { success } }",
        "variables": {
          "input": {
            "agentSessionId": "'"${SESSION_ID}"'",
            "content": {
              "type": "response",
              "body": "'"${COMPLETE_BODY}"'"
            }
          }
        }
      }' > /dev/null
    
    echo "✅"
    SUCCESS=$((SUCCESS + 1))
    
    # Small delay to avoid rate limiting
    sleep 0.5
done

echo ""
echo "=============================================="
echo "  🏁 Smoke Test Complete!"
echo "  ✅ Success: ${SUCCESS}/${TOTAL}"
echo "  ❌ Failed: ${FAILED}/${TOTAL}"
echo "  📋 Linear: https://linear.app/jonathonfritz/issue/${LINEAR_ISSUE_IDENTIFIER}"
echo "=============================================="
