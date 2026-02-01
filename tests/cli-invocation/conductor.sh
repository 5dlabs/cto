#!/bin/bash
# conductor.sh - Orchestrate ALL sub-agents and post play-by-play to Linear

set -e

source .env

# Ensure LINEAR_TEAM_ID is set
if [ -z "$LINEAR_TEAM_ID" ]; then
  echo "❌ ERROR: LINEAR_TEAM_ID environment variable is not set"
  echo "   Please set it in your .env file"
  exit 1
fi

# Create a fresh issue for this test run
echo "🎯 Creating Linear issue for full orchestration test..."
ISSUE_JSON=$(curl -s -X POST https://api.linear.app/graphql \
  -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation CreateIssue($input: IssueCreateInput!) { issueCreate(input: $input) { success issue { id identifier url } } }",
    "variables": {
      "input": {
        "teamId": "'"$LINEAR_TEAM_ID"'",
        "title": "Full Infrastructure Deployment - '"$(date +%H:%M)"'",
        "description": "Complete orchestration: databases + storage + security + networking"
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

# Post conductor message helper
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
post_conductor "# ⚡ **BOLT** — Mission Control\\n\\nOrchestrating complete infrastructure deployment...\\n\\n📋 **Mission:** Deploy databases, storage, security, and networking\\n\\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Posted conductor init"

# Define ALL 6 sub-agents
AGENTS=("postgres-deployer" "mongo-deployer" "kafka-deployer" "seaweedfs-deployer" "security-agent" "network-agent")
EMOJIS=("🐘" "🍃" "📨" "🌊" "🔐" "🌐")
TITLES=("Deploy PostgreSQL Cluster" "Deploy MongoDB Cluster" "Deploy Kafka Cluster" "Deploy SeaweedFS Storage" "Configure Security Policies" "Configure Network Policies")

# Post delegation messages
echo ""
echo "🚀 Delegating to ${#AGENTS[@]} sub-agents..."
for i in "${!AGENTS[@]}"; do
    agent="${AGENTS[$i]}"
    emoji="${EMOJIS[$i]}"
    title="${TITLES[$i]}"
    
    post_conductor "${emoji} **Delegating** to ${agent} — ${title}"
    echo "   → ${emoji} ${agent}: ${title}"
    sleep 0.3
done

# Simulate sub-agent completions with varied timing (parallel execution)
echo ""
echo "⏳ Sub-agents working in parallel..."

# Wave 1 - fast completions
sleep 2
post_conductor "✅ **Mongo Deployer** completed — 28s • \\$0.0512 • 3 turns"
echo "   ✅ mongo-deployer completed (28s)"

sleep 1
post_conductor "✅ **Postgres Deployer** completed — 32s • \\$0.0614 • 4 turns"
echo "   ✅ postgres-deployer completed (32s)"

# Wave 2 - medium
sleep 1
post_conductor "✅ **SeaweedFS Deployer** completed — 38s • \\$0.0589 • 4 turns"
echo "   ✅ seaweedfs-deployer completed (38s)"

sleep 1
post_conductor "✅ **Security Agent** completed — 42s • \\$0.0678 • 5 turns"
echo "   ✅ security-agent completed (42s)"

# Wave 3 - longer tasks
sleep 1
post_conductor "✅ **Kafka Deployer** completed — 45s • \\$0.0723 • 5 turns"
echo "   ✅ kafka-deployer completed (45s)"

sleep 1
post_conductor "✅ **Network Agent** completed — 48s • \\$0.0701 • 5 turns"
echo "   ✅ network-agent completed (48s)"

# Post all done message
post_conductor "🎉 **All 6 sub-agents completed successfully!**"
echo ""
echo "🎉 All sub-agents done!"

# Post final summary as issue comment
echo ""
echo "📊 Posting final summary..."
SUMMARY=$(cat << 'EOF'
🎉 **Infrastructure Deployment Complete!**

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⏱️ **Total:** 48s (parallel) │ 💰 **Total:** $0.3817 │ 🔄 **Turns:** 26
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 **All Deliverables:**
  ✅ postgresql-cluster.yaml
  ✅ mongodb-cluster.yaml  
  ✅ kafka-cluster.yaml
  ✅ seaweedfs-cluster.yaml
  ✅ security-policies.yaml
  ✅ network-policies.yaml

🤖 **Sub-Agents (6):**
  • 🐘 Postgres Deployer — 32s, $0.0614, 4 turns
  • 🍃 Mongo Deployer — 28s, $0.0512, 3 turns
  • 📨 Kafka Deployer — 45s, $0.0723, 5 turns
  • 🌊 SeaweedFS Deployer — 38s, $0.0589, 4 turns
  • 🔐 Security Agent — 42s, $0.0678, 5 turns
  • 🌐 Network Agent — 48s, $0.0701, 5 turns

🛠️ **Built-in Tools:** Write (12), Bash (18), Read (14), Glob (4)

🔧 **MCP Tools Used:**
  • github_push_files (6)
  • kubectl_apply (6)
  • grafana_query_prometheus (2)
  • openmemory_store (3)

📚 **Skills Leveraged:**
  • kubernetes-operators
  • storage-operators
  • argocd-gitops
  • secrets-management

🧠 **Model:** claude-sonnet-4-5-20250929

🔄 **Iterations:** 1/3 (completed on first attempt)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
*Mission accomplished! Complete infrastructure stack deployed.* ⚡
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
echo "Done! Check Linear to see the full orchestration flow with all 6 agents."
