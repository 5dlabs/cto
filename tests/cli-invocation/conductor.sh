#!/bin/bash
# conductor.sh - Orchestrate sub-agents and post play-by-play to Linear

set -e

source .env

# Create a fresh issue for this test run
echo "🎯 Creating Linear issue for orchestration test..."
ISSUE_JSON=$(curl -s -X POST https://api.linear.app/graphql \
  -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation CreateIssue($input: IssueCreateInput!) { issueCreate(input: $input) { success issue { id identifier url } } }",
    "variables": {
      "input": {
        "teamId": "9cc787e5-3039-46b3-8fd6-4e0d0d381e74",
        "title": "Full Orchestration Test - '"$(date +%H:%M)"'",
        "description": "Testing conductor + parallel sub-agents + final summary"
      }
    }
  }')

ISSUE_ID=$(echo "$ISSUE_JSON" | jq -r '.data.issueCreate.issue.id')
ISSUE_IDENTIFIER=$(echo "$ISSUE_JSON" | jq -r '.data.issueCreate.issue.identifier')
ISSUE_URL=$(echo "$ISSUE_JSON" | jq -r '.data.issueCreate.issue.url')

echo "✅ Created: $ISSUE_IDENTIFIER"
echo "   URL: $ISSUE_URL"
echo ""

# Update .env
sed -i '' "s/LINEAR_ISSUE_IDENTIFIER=.*/LINEAR_ISSUE_IDENTIFIER=$ISSUE_IDENTIFIER/" .env

# Create conductor session first
echo "⚡ Creating BOLT conductor session..."
CONDUCTOR_SESSION=$(curl -s -X POST https://api.linear.app/graphql \
  -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation CreateConductor($input: AgentSessionCreateOnIssue!) { agentSessionCreateOnIssue(input: $input) { success agentSession { id } } }",
    "variables": {
      "input": {
        "issueId": "'"$ISSUE_ID"'"
      }
    }
  }' | jq -r '.data.agentSessionCreateOnIssue.agentSession.id')

echo "   Session: $CONDUCTOR_SESSION"

# Post conductor init message
post_conductor() {
    local body="$1"
    curl -s -X POST https://api.linear.app/graphql \
      -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
      -H "Content-Type: application/json" \
      -d '{
        "query": "mutation AddActivity($input: AgentActivityCreateInput!) { agentActivityCreate(input: $input) { success } }",
        "variables": {
          "input": {
            "agentSessionId": "'"$CONDUCTOR_SESSION"'",
            "content": {
              "type": "response",
              "body": "'"$body"'"
            }
          }
        }
      }' > /dev/null
}

echo ""
post_conductor "# ⚡ **BOLT** — Mission Control\\n\\nOrchestrating infrastructure deployment across multiple agents...\\n\\n📋 **Mission:** Deploy complete infrastructure\\n\\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Posted conductor init"

# Define sub-agents
AGENTS=("postgres-deployer" "mongo-deployer" "kafka-deployer")
EMOJIS=("🐘" "🍃" "📨")
TITLES=("Deploy PostgreSQL Cluster" "Deploy MongoDB Cluster" "Deploy Kafka Cluster")

# Post delegation messages
echo ""
echo "🚀 Delegating to sub-agents..."
for i in "${!AGENTS[@]}"; do
    agent="${AGENTS[$i]}"
    emoji="${EMOJIS[$i]}"
    title="${TITLES[$i]}"
    
    post_conductor "${emoji} **Delegating** to ${agent} — ${title}"
    echo "   → ${emoji} ${agent}: ${title}"
    sleep 0.5
done

# Start sub-agents in parallel (simulated for now)
echo ""
echo "⏳ Running sub-agents in parallel..."
echo "   (In production, these would run as parallel containers)"

# Simulate sub-agent completions with varied timing
sleep 2
post_conductor "✅ **Postgres Deployer** completed — 32s • \\$0.0614"
echo "   ✅ postgres-deployer completed"

sleep 1
post_conductor "✅ **Mongo Deployer** completed — 28s • \\$0.0512"
echo "   ✅ mongo-deployer completed"

sleep 2
post_conductor "✅ **Kafka Deployer** completed — 45s • \\$0.0723"
echo "   ✅ kafka-deployer completed"

# Post all done message
post_conductor "🎉 **All 3 sub-agents completed successfully!**"
echo ""
echo "🎉 All sub-agents done!"

# Post final summary as issue comment
echo ""
echo "📊 Posting final summary..."
SUMMARY=$(cat << 'EOF'
🎉 **Infrastructure Deployment Complete!**

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⏱️ **Total:** 1m 45s │ 💰 **Total:** $0.1849 │ 🔄 **Turns:** 12
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 **All Deliverables:**
  ✅ postgresql-cluster.yaml
  ✅ mongodb-cluster.yaml
  ✅ kafka-cluster.yaml

🤖 **Sub-Agents (3):**
  • 🐘 Postgres Deployer — 32s, $0.0614, 4 turns
  • 🍃 Mongo Deployer — 28s, $0.0512, 3 turns
  • 📨 Kafka Deployer — 45s, $0.0723, 5 turns

🔧 **Tools Used:** Write (6), Bash (12), Read (9)
📚 **Skills:** kubernetes-operators, storage-operators, argocd-gitops
🧠 **Model:** claude-sonnet-4-5-20250929

🔄 **Iterations:** 1/3 (completed on first attempt)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
*Mission accomplished! All infrastructure deployed.* ⚡
EOF
)

# Escape for JSON
SUMMARY_ESCAPED=$(echo "$SUMMARY" | jq -Rs .)

curl -s -X POST https://api.linear.app/graphql \
  -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation AddComment($input: CommentCreateInput!) { commentCreate(input: $input) { success } }",
    "variables": {
      "input": {
        "issueId": "'"$ISSUE_ID"'",
        "body": '"$SUMMARY_ESCAPED"'
      }
    }
  }' > /dev/null

echo "✅ Final summary posted!"
echo ""
echo "📋 View at: $ISSUE_URL"
echo ""
echo "Done! Check Linear to see the full orchestration flow."
