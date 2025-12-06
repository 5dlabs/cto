#!/bin/bash
# Run a single CodeRun task for one agent/CLI combination
# Captures both the generated code and stdout/logs
#
# Usage: ./run-single-task.sh <agent> <cli>
# Example: ./run-single-task.sh rex claude

set -euo pipefail

AGENT="${1:-}"
CLI="${2:-}"

if [[ -z "$AGENT" || -z "$CLI" ]]; then
    echo "Usage: $0 <agent> <cli>"
    echo "Example: $0 rex claude"
    echo ""
    echo "Available agents: rex, blaze, grizz, nova, tap, spark, bolt, cipher, cleo, tess, morgan, atlas"
    echo "Available CLIs: claude, codex, cursor, factory, gemini, opencode"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCENARIOS_FILE="${SCRIPT_DIR}/../../scenarios/scenarios.yaml"
OUTPUT_DIR="${SCRIPT_DIR}/../outputs/${AGENT}-${CLI}"
REPO="5dlabs/cto-test-${AGENT}-${CLI}"

# Map agent names to GitHub apps
get_github_app() {
    case "$1" in
        rex) echo "5DLabs-Rex" ;;
        blaze) echo "5DLabs-Blaze" ;;
        grizz) echo "5DLabs-Grizz" ;;
        nova) echo "5DLabs-Nova" ;;
        tap) echo "5DLabs-Tap" ;;
        spark) echo "5DLabs-Spark" ;;
        bolt) echo "5DLabs-Bolt" ;;
        cipher) echo "5DLabs-Cipher" ;;
        cleo) echo "5DLabs-Cleo" ;;
        tess) echo "5DLabs-Tess" ;;
        stitch) echo "5DLabs-Stitch" ;;
        morgan) echo "5DLabs-Morgan" ;;
        atlas) echo "5DLabs-Atlas" ;;
        *) echo "5DLabs-$(echo "$1" | sed 's/./\U&/')" ;;
    esac
}

GITHUB_APP=$(get_github_app "$AGENT")

echo "═══════════════════════════════════════════════════════════════"
echo "  Single Task Test: ${AGENT} + ${CLI}"
echo "  Repository: ${REPO}"
echo "  GitHub App: ${GITHUB_APP}"
echo "═══════════════════════════════════════════════════════════════"

# Create output directory
mkdir -p "$OUTPUT_DIR"
mkdir -p "${OUTPUT_DIR}/code"
mkdir -p "${OUTPUT_DIR}/logs"

# Get task info from scenarios.yaml
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

TASK_PROMPT=$(python3 -c "
import yaml
with open('${SCENARIOS_FILE}', 'r') as f:
    scenarios = yaml.safe_load(f)
agent = '${AGENT}'
if agent in scenarios:
    print(scenarios[agent]['scenario']['task_prompt'])
else:
    print('Implement a feature matching the agent specialty')
" 2>/dev/null || echo "Implement a feature")

WORKING_DIR=$(python3 -c "
import yaml
with open('${SCENARIOS_FILE}', 'r') as f:
    scenarios = yaml.safe_load(f)
agent = '${AGENT}'
if agent in scenarios:
    print(scenarios[agent]['scenario']['working_directory'])
else:
    print('src')
" 2>/dev/null || echo "src")

echo ""
echo "Task: ${TASK_TITLE}"
echo "Working Directory: ${WORKING_DIR}"
echo ""

# Step 1: Reset repository with task
echo "Step 1: Resetting repository with task..."
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
gh repo clone "$REPO" . -- --quiet 2>/dev/null || {
    echo "Creating repository ${REPO}..."
    gh repo create "$REPO" --public --description "E2E test for ${AGENT} + ${CLI}" -y
    git init
    git remote add origin "https://github.com/${REPO}.git"
}

# Clean the repo
rm -rf ./* .tasks 2>/dev/null || true

# Create README
AGENT_UPPER=$(echo "$AGENT" | tr '[:lower:]' '[:upper:]' | cut -c1)$(echo "$AGENT" | cut -c2-)
CLI_UPPER=$(echo "$CLI" | tr '[:lower:]' '[:upper:]' | cut -c1)$(echo "$CLI" | cut -c2-)
cat > README.md << EOF
# ${AGENT_UPPER} + ${CLI_UPPER} E2E Test

**Task**: ${TASK_TITLE}

This repository tests the ${AGENT} agent with ${CLI} CLI.
EOF

# Create tasks structure
mkdir -p .tasks/tasks/task-001

cat > .tasks/tasks/tasks.json << TASKJSON
{
  "tasks": [
    {
      "id": 1,
      "title": "${TASK_TITLE}",
      "description": "E2E test task",
      "status": "pending",
      "priority": "high",
      "dependencies": []
    }
  ]
}
TASKJSON

cat > .tasks/tasks/task-001/task.md << TASKMD
# Task 1: ${TASK_TITLE}

${TASK_PROMPT}
TASKMD

git add -A
git commit -m "Setup E2E test task" --allow-empty
git push origin main --force 2>/dev/null || git push --set-upstream origin main --force
cd - > /dev/null
rm -rf "$TEMP_DIR"
echo "  ✓ Repository ready"

# Step 2: Create and apply CodeRun
echo ""
echo "Step 2: Creating CodeRun..."

CODERUN_NAME="e2e-${AGENT}-${CLI}-$(date +%s)"
CODERUN_YAML=$(mktemp)

# Create CodeRun YAML (task requirements will be read from the repo)
cat > "$CODERUN_YAML" << 'YAML'
apiVersion: cto.5dlabs.io/v1alpha1
kind: CodeRun
metadata:
  name: CODERUN_NAME_PLACEHOLDER
  namespace: cto
spec:
  runType: implementation
  service: SERVICE_PLACEHOLDER
  repositoryUrl: REPO_URL_PLACEHOLDER
  docsRepositoryUrl: REPO_URL_PLACEHOLDER
  docsBranch: main
  docsProjectDirectory: docs
  workingDirectory: WORKING_DIR_PLACEHOLDER
  model: claude-sonnet-4-20250514
  githubApp: GITHUB_APP_PLACEHOLDER
  taskId: 1
  contextVersion: 1
  continueSession: false
  overwriteMemory: false
  enableDocker: false
  cliConfig:
    cliType: CLI_PLACEHOLDER
    model: claude-sonnet-4-20250514
    maxTokens: 16000
    temperature: 0.7
    settings:
      approvalPolicy: auto-edit
      sandboxMode: full
YAML

# Replace placeholders
sed -i.bak "s|CODERUN_NAME_PLACEHOLDER|${CODERUN_NAME}|g" "$CODERUN_YAML"
sed -i.bak "s|SERVICE_PLACEHOLDER|cto-test-${AGENT}-${CLI}|g" "$CODERUN_YAML"
sed -i.bak "s|REPO_URL_PLACEHOLDER|https://github.com/${REPO}|g" "$CODERUN_YAML"
sed -i.bak "s|WORKING_DIR_PLACEHOLDER|${WORKING_DIR}|g" "$CODERUN_YAML"
sed -i.bak "s|GITHUB_APP_PLACEHOLDER|${GITHUB_APP}|g" "$CODERUN_YAML"
sed -i.bak "s|CLI_PLACEHOLDER|${CLI}|g" "$CODERUN_YAML"
rm -f "${CODERUN_YAML}.bak"

echo "  CodeRun manifest:"
cat "$CODERUN_YAML" | head -30
echo "  ..."
echo ""

# Apply the CodeRun
echo "  Applying CodeRun to cluster..."
kubectl apply -f "$CODERUN_YAML"
rm "$CODERUN_YAML"
echo "  ✓ CodeRun created: ${CODERUN_NAME}"

# Save metadata
cat > "${OUTPUT_DIR}/run-info.json" << EOF
{
  "agent": "${AGENT}",
  "cli": "${CLI}",
  "github_app": "${GITHUB_APP}",
  "repository": "${REPO}",
  "coderun_name": "${CODERUN_NAME}",
  "task_title": "${TASK_TITLE}",
  "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "running"
}
EOF

# Step 3: Wait and monitor
echo ""
echo "Step 3: Monitoring CodeRun..."
echo "  Waiting for pod to start..."

POD_NAME=""
for i in {1..60}; do
    POD_NAME=$(kubectl get pods -n cto -l "cto.5dlabs.io/coderun=${CODERUN_NAME}" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)
    if [[ -n "$POD_NAME" ]]; then
        break
    fi
    sleep 5
done

if [[ -z "$POD_NAME" ]]; then
    echo "  ✗ Pod not found after 5 minutes"
    exit 1
fi

echo "  ✓ Pod started: ${POD_NAME}"
echo ""
echo "  Streaming logs (Ctrl+C to stop watching, task continues in background)..."
echo "  Logs also saved to: ${OUTPUT_DIR}/logs/stdout.log"
echo ""
echo "────────────────────────────────────────────────────────────────"

# Stream logs to both terminal and file
kubectl logs -n cto -f "$POD_NAME" 2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || true

echo "────────────────────────────────────────────────────────────────"
echo ""

# Step 4: Collect results
echo "Step 4: Collecting results..."

# Wait for completion
echo "  Waiting for CodeRun to complete..."
for i in {1..120}; do
    STATUS=$(kubectl get coderun -n cto "$CODERUN_NAME" -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
    if [[ "$STATUS" == "Succeeded" || "$STATUS" == "Failed" ]]; then
        break
    fi
    sleep 10
done

echo "  Final status: ${STATUS}"

# Clone the repo to get generated code
echo "  Cloning generated code..."
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
gh repo clone "$REPO" . -- --quiet

# Find the feature branch
BRANCH=$(git branch -r | grep -E "feature/task-1|feature/e2e" | head -1 | tr -d ' ' | sed 's/origin\///')
if [[ -n "$BRANCH" ]]; then
    git checkout "$BRANCH" 2>/dev/null || true
fi

# Copy all code (excluding .git and .tasks)
rsync -av --exclude='.git' --exclude='.tasks' . "${OUTPUT_DIR}/code/"

# Generate diff
git diff main HEAD > "${OUTPUT_DIR}/code.patch" 2>/dev/null || true

cd - > /dev/null
rm -rf "$TEMP_DIR"

# Update run info
cat > "${OUTPUT_DIR}/run-info.json" << EOF
{
  "agent": "${AGENT}",
  "cli": "${CLI}",
  "github_app": "${GITHUB_APP}",
  "repository": "${REPO}",
  "coderun_name": "${CODERUN_NAME}",
  "task_title": "${TASK_TITLE}",
  "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "completed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "${STATUS}",
  "branch": "${BRANCH:-main}"
}
EOF

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Results saved to: ${OUTPUT_DIR}"
echo ""
echo "  Files:"
find "${OUTPUT_DIR}" -type f | sort | sed 's|^|    |'
echo ""
echo "═══════════════════════════════════════════════════════════════"

