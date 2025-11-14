#!/bin/bash
# Manual Atlas CodeRun creation for PR monitoring
# Use this when sensor is not functioning

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <pr-number>"
  echo "Example: $0 1366"
  exit 1
fi

PR_NUMBER="$1"

echo "=== Manual Atlas CodeRun Creation ==="
echo "PR Number: #$PR_NUMBER"
echo ""

# Check if CodeRun already exists
EXISTING=$(kubectl get coderuns -n agent-platform \
  -l agent=atlas,pr-number="$PR_NUMBER" \
  -o json 2>/dev/null || echo '{"items":[]}')

COUNT=$(echo "$EXISTING" | jq '.items | length')

if [ "$COUNT" -gt 0 ]; then
  echo "⚠️  Found $COUNT existing CodeRun(s) for PR #$PR_NUMBER:"
  echo "$EXISTING" | jq -r '.items[] | "  - \(.metadata.name) (\(.status.phase // "Unknown"))"'
  echo ""
  read -p "Create another CodeRun anyway? (yes/no): " CONFIRM
  if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted."
    exit 0
  fi
fi

# Fetch PR details from GitHub
echo "Fetching PR details from GitHub..."
PR_DATA=$(gh api "/repos/5dlabs/cto/pulls/$PR_NUMBER" 2>/dev/null || echo "{}")

if [ "$(echo "$PR_DATA" | jq -r '.number')" = "null" ]; then
  echo "❌ PR #$PR_NUMBER not found or gh CLI not configured"
  exit 1
fi

PR_URL=$(echo "$PR_DATA" | jq -r '.html_url')
CLONE_URL=$(echo "$PR_DATA" | jq -r '.head.repo.clone_url')

echo "PR URL: $PR_URL"
echo "Clone URL: $CLONE_URL"
echo ""

# Create CodeRun
echo "Creating Atlas CodeRun..."
cat <<EOF | kubectl create -f - 2>&1 | tee /tmp/atlas-coderun-create.txt
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  generateName: coderun-atlas-pr-$PR_NUMBER-manual-
  namespace: agent-platform
  labels:
    agent: atlas
    role: pr-guardian
    pr-number: "$PR_NUMBER"
    repository: "cto"
    manual-trigger: "true"
spec:
  taskId: 0
  service: "atlas-pr-guardian"
  githubApp: "5DLabs-Atlas"
  model: "claude-sonnet-4-5-20250929"
  repositoryUrl: "$CLONE_URL"
  docsRepositoryUrl: "https://github.com/5dlabs/cto.git"
  docsProjectDirectory: "docs"
  docsBranch: "main"
  workingDirectory: "."
  enableDocker: true
  continueSession: true
  overwriteMemory: false
  cliConfig:
    cliType: "claude"
    model: "claude-sonnet-4-5-20250929"
    maxTokens: 8192
    temperature: 0.3
  env:
    PR_NUMBER: "$PR_NUMBER"
    PR_URL: "$PR_URL"
    REPOSITORY_FULL_NAME: "5dlabs/cto"
    GUARDIAN_MODE: "active"
    TARGET_REPOSITORY: "5dlabs/cto"
    MERGE_STRATEGY: "squash"
EOF

if grep -q "created" /tmp/atlas-coderun-create.txt; then
  CODERUN_NAME=$(grep "created" /tmp/atlas-coderun-create.txt | awk '{print $1}' | cut -d'/' -f2)
  echo ""
  echo "✅ CodeRun created: $CODERUN_NAME"
  echo ""
  echo "Monitor progress:"
  echo "  kubectl get coderun $CODERUN_NAME -n agent-platform -w"
  echo "  kubectl logs -n agent-platform -l job-name=$CODERUN_NAME -f"
else
  echo "❌ CodeRun creation failed:"
  cat /tmp/atlas-coderun-create.txt
  exit 1
fi

rm -f /tmp/atlas-coderun-create.txt
