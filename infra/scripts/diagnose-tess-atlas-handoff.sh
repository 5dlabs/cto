#!/bin/bash
# Diagnostic script for Tess-Atlas handoff issues

set -euo pipefail

echo "=== Tess-Atlas Handoff Diagnostics ==="
echo "Date: $(date)"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find stuck workflows
echo "🔍 Checking for stuck workflows..."
STUCK_WORKFLOWS=$(kubectl get workflows -n agent-platform \
  -l current-stage=waiting-atlas-integration \
  --field-selector status.phase=Running \
  -o json 2>/dev/null | jq -r '.items[] | select((now - (.status.startedAt | fromdateiso8601)) > 1800) | .metadata.name' || true)

if [ -n "$STUCK_WORKFLOWS" ]; then
  echo -e "${RED}❌ Found stuck workflows (>30 minutes at waiting-atlas-integration):${NC}"
  echo "$STUCK_WORKFLOWS"
else
  echo -e "${GREEN}✅ No stuck workflows found${NC}"
fi

echo ""
echo "📊 Workflow Stage Distribution:"
kubectl get workflows -n agent-platform \
  --field-selector status.phase=Running \
  -o custom-columns=NAME:.metadata.name,STAGE:.metadata.labels.current-stage,AGE:.status.startedAt \
  2>/dev/null || echo "No running workflows"

echo ""
echo "🧪 Recent Tess CodeRuns:"
kubectl get coderuns -n agent-platform \
  -l stage=testing \
  --sort-by=.metadata.creationTimestamp \
  -o custom-columns=NAME:.metadata.name,TASK:.metadata.labels.task-id,STATUS:.status.phase,AGE:.metadata.creationTimestamp \
  | tail -5

echo ""
echo "📡 Sensor Health Check:"
# Note: Sensor names don't have '-sensor' suffix in the cluster
for sensor in stage-aware-tess-approval tess-label-fallback atlas-pr-monitor; do
  STATUS=$(kubectl get sensor $sensor -n argo -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "NotFound")
  DEPLOYED=$(kubectl get sensor $sensor -n argo -o jsonpath='{.status.conditions[?(@.type=="Deployed")].status}' 2>/dev/null || echo "NotFound")
  
  if [ "$STATUS" = "True" ]; then
    echo -e "  ${GREEN}✅ $sensor: Ready${NC}"
  elif [ "$DEPLOYED" = "True" ]; then
    echo -e "  ${YELLOW}⚠️ $sensor: Deployed but not Ready${NC}"
  elif [ "$STATUS" = "NotFound" ]; then
    echo -e "  ${RED}❌ $sensor: Not Found${NC}"
  else
    echo -e "  ${YELLOW}⚠️ $sensor: Not Ready (Status: $STATUS)${NC}"
  fi
done

echo ""
echo "🔄 Recent PR Review Events (last hour):"
kubectl logs -n argo \
  -l sensor-name=stage-aware-tess-approval-sensor \
  --since=1h \
  --tail=50 2>/dev/null | grep -E "(PR|review|approved)" | tail -5 || echo "No recent PR review events"

echo ""
echo "🏷️ Recent Label Events (last hour):"
kubectl logs -n argo \
  -l sensor-name=tess-label-fallback \
  --since=1h \
  --tail=50 2>/dev/null | grep -E "(label|approved|fallback)" | tail -5 || echo "No recent label events"

# Check a specific workflow if provided
if [ $# -gt 0 ]; then
  WORKFLOW_NAME="$1"
  echo ""
  echo "=== Detailed Analysis for Workflow: $WORKFLOW_NAME ==="
  
  # Get workflow details
  TASK_ID=$(kubectl get workflow "$WORKFLOW_NAME" -n agent-platform -o jsonpath='{.metadata.labels.task-id}' 2>/dev/null || echo "unknown")
  REPO=$(kubectl get workflow "$WORKFLOW_NAME" -n agent-platform -o jsonpath='{.metadata.labels.repository}' 2>/dev/null | tr '-' '/')
  CURRENT_STAGE=$(kubectl get workflow "$WORKFLOW_NAME" -n agent-platform -o jsonpath='{.metadata.labels.current-stage}' 2>/dev/null || echo "unknown")
  
  echo "Task ID: $TASK_ID"
  echo "Repository: $REPO"
  echo "Current Stage: $CURRENT_STAGE"
  
  # Check for suspended nodes
  echo ""
  echo "Suspended Nodes:"
  kubectl get workflow "$WORKFLOW_NAME" -n agent-platform -o json | \
    jq -r '.status.nodes | to_entries[] | select(.value.type == "Suspend" and .value.phase == "Running") | "  - \(.value.displayName) (ID: \(.key))"' || echo "  None"
  
  # Find associated PR
  if [ "$REPO" != "/" ] && [ "$TASK_ID" != "unknown" ]; then
    echo ""
    echo "Checking GitHub PR status..."
    PR_NUMBER=$(gh pr list --repo "$REPO" --search "task-$TASK_ID" --json number -q '.[0].number' 2>/dev/null || echo "")
    
    if [ -n "$PR_NUMBER" ]; then
      echo "PR #$PR_NUMBER:"
      
      # Check reviews
      echo "  Reviews:"
      gh pr view "$PR_NUMBER" --repo "$REPO" --json reviews 2>/dev/null | \
        jq -r '.reviews[] | "    - \(.author.login): \(.state)"' || echo "    Failed to fetch"
      
      # Check labels
      echo "  Labels:"
      gh pr view "$PR_NUMBER" --repo "$REPO" --json labels 2>/dev/null | \
        jq -r '.labels[] | "    - \(.name)"' || echo "    Failed to fetch"
    else
      echo "  No PR found for task-$TASK_ID"
    fi
  fi
  
  echo ""
  echo "💡 Remediation Suggestions:"
  if [[ "$CURRENT_STAGE" == "waiting-atlas-integration" ]]; then
    echo "  1. Check if Tess submitted PR review: gh pr view <PR#> --repo $REPO --json reviews"
    echo "  2. Check if 'approved' label exists: gh pr view <PR#> --repo $REPO --json labels"
    echo "  3. Resume manually if needed:"
    echo "     kubectl patch workflow $WORKFLOW_NAME -n agent-platform --type='json' -p='[{\"op\":\"replace\",\"path\":\"/spec/suspend\",\"value\":null}]'"
  fi
fi

echo ""
echo "=== Diagnostics Complete ==="
echo "For detailed workflow analysis, run: $0 <workflow-name>"

