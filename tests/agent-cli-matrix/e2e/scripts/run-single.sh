#!/bin/bash
# Run a single agent/CLI combination E2E test
#
# Usage: ./run-single.sh <agent> <cli>
# Example: ./run-single.sh rex claude

set -euo pipefail

AGENT="${1:-}"
CLI="${2:-}"

if [[ -z "$AGENT" || -z "$CLI" ]]; then
    echo "Usage: $0 <agent> <cli>"
    echo "Example: $0 rex claude"
    echo ""
    echo "Available agents: rex, blaze, grizz, nova, tap, spark, bolt, cipher, cleo, tess, stitch, morgan, atlas"
    echo "Available CLIs: claude, codex, cursor, factory, gemini, opencode"
    exit 1
fi

REPO="5dlabs/cto-test-${AGENT}-${CLI}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/../outputs/${AGENT}-${CLI}"

echo "═══════════════════════════════════════════════════════════════"
echo "  Running E2E Test: ${AGENT} + ${CLI}"
echo "  Repository: ${REPO}"
echo "═══════════════════════════════════════════════════════════════"

# Check if repo exists
if ! gh repo view "$REPO" &>/dev/null; then
    echo "Error: Repository $REPO does not exist"
    echo "Create it with: gh repo create $REPO --public"
    exit 1
fi

# Get the task prompt from scenarios.yaml
SCENARIOS_FILE="${SCRIPT_DIR}/../../scenarios/scenarios.yaml"
if [[ ! -f "$SCENARIOS_FILE" ]]; then
    echo "Error: scenarios.yaml not found at $SCENARIOS_FILE"
    exit 1
fi

echo ""
echo "Step 1: Reset repository and create task..."
# Clone, reset to empty, push
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
gh repo clone "$REPO" . -- --quiet 2>/dev/null || true

# Create minimal README
AGENT_UPPER=$(echo "$AGENT" | tr '[:lower:]' '[:upper:]' | cut -c1)$(echo "$AGENT" | cut -c2-)
cat > README.md << EOF
# ${AGENT_UPPER} Test Repository

This repository is used for E2E testing of the ${AGENT} agent with ${CLI} CLI.

**Do not manually modify this repository.** It is reset before each test run.
EOF

# Create tasks structure
mkdir -p .tasks/tasks

# Get task prompt from scenarios.yaml
TASK_PROMPT=$(python3 -c "
import yaml
import sys
with open('${SCENARIOS_FILE}', 'r') as f:
    scenarios = yaml.safe_load(f)
agent = '${AGENT}'
if agent in scenarios:
    print(scenarios[agent]['scenario']['task_prompt'])
else:
    print('Implement a feature for this agent')
" 2>/dev/null || echo "Implement a feature matching the agent specialty")

TASK_TITLE=$(python3 -c "
import yaml
with open('${SCENARIOS_FILE}', 'r') as f:
    scenarios = yaml.safe_load(f)
agent = '${AGENT}'
if agent in scenarios:
    print(scenarios[agent]['scenario']['title'])
else:
    print('Implement feature')
" 2>/dev/null || echo "Implement feature")

# Create tasks.json
cat > .tasks/tasks/tasks.json << TASKJSON
{
  "tasks": [
    {
      "id": 1,
      "title": "${TASK_TITLE}",
      "description": "E2E test task for ${AGENT} agent",
      "status": "pending",
      "priority": "high",
      "dependencies": []
    }
  ],
  "metadata": {
    "projectName": "cto-test-${AGENT}-${CLI}",
    "version": "1.0.0"
  }
}
TASKJSON

# Create individual task file
mkdir -p .tasks/tasks/task-001
cat > .tasks/tasks/task-001/task.md << TASKMD
# Task 1: ${TASK_TITLE}

## Description
${TASK_PROMPT}

## Acceptance Criteria
- Implementation matches the task requirements
- Code is well-structured and follows best practices
- Tests are included where appropriate
TASKMD

git add -A
git commit -m "Reset for E2E test with task" --allow-empty 2>/dev/null || true
git push origin main --force 2>/dev/null || true
cd - > /dev/null
rm -rf "$TEMP_DIR"

echo "  ✓ Repository reset with task created"

echo ""
echo "Step 2: Submit Play workflow..."
echo "  This will run the agent against the repository."
echo ""
echo "  To submit via MCP, use:"
echo "  mcp_cto_play({"
echo "    repository: \"${REPO}\","
echo "    service: \"cto-test-${AGENT}-${CLI}\","
echo "    cli: \"${CLI}\""
echo "  })"
echo ""
echo "  Or manually via kubectl apply -f coderun.yaml"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Save test metadata
cat > "${OUTPUT_DIR}/test-info.json" << EOF
{
  "agent": "${AGENT}",
  "cli": "${CLI}",
  "repository": "${REPO}",
  "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "pending"
}
EOF

echo ""
echo "Step 3: Monitor and collect output..."
echo "  After the workflow completes, run:"
echo "  ./collect-output.sh ${AGENT} ${CLI}"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Test submitted. Monitor progress in ArgoCD or Argo Workflows."
echo "═══════════════════════════════════════════════════════════════"

