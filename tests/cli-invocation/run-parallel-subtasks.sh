#!/bin/bash
# run-parallel-subtasks.sh
# Runs all sub-agent tasks in PARALLEL (not sequential)

set -e

source .env

# Ensure LINEAR_TEAM_ID is set
if [ -z "$LINEAR_TEAM_ID" ]; then
  echo "❌ ERROR: LINEAR_TEAM_ID environment variable is not set"
  echo "   Please set it in your .env file"
  exit 1
fi

# Create fresh issue for this test
echo "Creating fresh Linear issue..."
ISSUE=$(curl -s -X POST https://api.linear.app/graphql \
  -H "Authorization: ${LINEAR_OAUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation CreateIssue($input: IssueCreateInput!) { issueCreate(input: $input) { success issue { identifier url } } }",
    "variables": {
      "input": {
        "teamId": "'"$LINEAR_TEAM_ID"'",
        "title": "Parallel Sub-Agent Test - '"$(date +%H:%M)"'",
        "description": "Testing parallel execution of sub-agents with final summary"
      }
    }
  }')

ISSUE_ID=$(echo "$ISSUE" | jq -r '.data.issueCreate.issue.identifier')
ISSUE_URL=$(echo "$ISSUE" | jq -r '.data.issueCreate.issue.url')

echo "✅ Created issue: $ISSUE_ID"
echo "   URL: $ISSUE_URL"

# Update .env with new issue
sed -i '' "s/LINEAR_ISSUE_IDENTIFIER=.*/LINEAR_ISSUE_IDENTIFIER=$ISSUE_ID/" .env

# Subtasks to run in parallel
SUBTASKS=("task-1.1" "task-1.2" "task-1.3")
NAMES=("postgres-deployer" "mongo-deployer" "kafka-deployer")

echo ""
echo "🚀 Starting ${#SUBTASKS[@]} sub-agents in PARALLEL..."
echo ""

# Clean up any existing containers
docker compose down 2>/dev/null || true

# Start each subtask as a separate compose project (parallel)
PIDS=()
for i in "${!SUBTASKS[@]}"; do
    SUBTASK="${SUBTASKS[$i]}"
    NAME="${NAMES[$i]}"
    
    echo "  → Starting $NAME ($SUBTASK)..."
    
    # Copy prompt to workspace
    mkdir -p "workspaces/$NAME"
    cp "config/task-bolt/subtasks/$SUBTASK/prompt.md" "workspaces/$NAME/prompt.md"
    rm -f "workspaces/$NAME/stream.jsonl"
    
    # Run with unique project name for parallel execution
    COMPOSE_PROJECT_NAME="cto-$NAME" \
    SUBTASK_ID="$SUBTASK" \
    SUBTASK_NAME="$NAME" \
    WORKSPACE_PATH="./workspaces/$NAME" \
    docker compose -f docker-compose-parallel.yml up claude claude-sidecar &
    
    PIDS+=($!)
done

echo ""
echo "⏳ Waiting for all sub-agents to complete..."
echo "   PIDs: ${PIDS[*]}"
echo ""

# Wait for all to complete
for pid in "${PIDS[@]}"; do
    wait $pid || true
done

echo ""
echo "✅ All sub-agents completed!"
echo ""
echo "📋 View results at: $ISSUE_URL"
echo ""

# TODO: Post aggregated summary to issue
# This would sum up all costs, tools, etc. from each sub-agent
