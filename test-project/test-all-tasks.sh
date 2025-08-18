#!/bin/bash
# Test the complete minimal project with all 4 tasks

set -e

NAMESPACE="${NAMESPACE:-agent-platform}"
BRANCH="${BRANCH:-feat/minimal-test-project}"

echo "=================================="
echo "🧪 Testing Complete Minimal Project"
echo "=================================="
echo ""
echo "📋 Configuration:"
echo "  Tasks: 1-4 (minimal test tasks)"
echo "  Branch: $BRANCH"
echo "  Namespace: $NAMESPACE"
echo "  Cost: ZERO (no API calls)"
echo ""

# Check if branch exists
if ! git rev-parse --verify $BRANCH >/dev/null 2>&1; then
    echo "⚠️  Branch $BRANCH doesn't exist yet"
    echo "   This test will use the workflow from main branch"
    echo "   But will look for tasks in $BRANCH"
    echo ""
fi

# Submit the workflow
echo "🚀 Submitting test project workflow..."

WORKFLOW_NAME=$(kubectl create -f test-project/minimal-project-workflow.yaml -o jsonpath='{.metadata.name}' 2>/dev/null || echo "")

if [ -z "$WORKFLOW_NAME" ]; then
    echo "❌ Failed to create workflow"
    echo "   Make sure test-project/minimal-project-workflow.yaml exists"
    exit 1
fi

echo "✅ Created workflow: $WORKFLOW_NAME"
echo ""
echo "⏳ Waiting for completion (this should take < 5 minutes)..."
echo ""

# Monitor progress
timeout=300  # 5 minutes
elapsed=0
while [ $elapsed -lt $timeout ]; do
    status=$(kubectl get workflow "$WORKFLOW_NAME" -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
    
    case "$status" in
        "Succeeded")
            echo ""
            echo "🎉 All tasks completed successfully!"
            echo ""
            echo "📊 Summary:"
            kubectl get workflow "$WORKFLOW_NAME" -n "$NAMESPACE" -o jsonpath='{.status.nodes}' | jq -r 'to_entries | .[] | select(.value.type == "Pod") | "  - \(.value.displayName): \(.value.phase)"' 2>/dev/null || true
            exit 0
            ;;
        "Failed"|"Error")
            echo ""
            echo "❌ Workflow failed"
            echo ""
            echo "Check logs with:"
            echo "  kubectl describe workflow $WORKFLOW_NAME -n $NAMESPACE"
            exit 1
            ;;
        "Running")
            echo -n "."
            ;;
        *)
            echo -n "?"
            ;;
    esac
    
    sleep 5
    elapsed=$((elapsed + 5))
done

echo ""
echo "⏱️ Timeout waiting for completion"
echo ""
echo "Check status with:"
echo "  kubectl get workflow $WORKFLOW_NAME -n $NAMESPACE"
exit 1
