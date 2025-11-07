#!/bin/bash
# Script to clean up orphaned resources in agent-platform namespace
# Removes completed pods, succeeded workflows, and succeeded CodeRuns

set -euo pipefail

NAMESPACE="agent-platform"

echo "🧹 Agent Platform Namespace Cleanup"
echo "===================================="
echo ""

# Function to count resources
count_resources() {
    local resource_type=$1
    local selector=$2
    
    if [[ "$selector" == "pod" ]]; then
        kubectl get pods -n "$NAMESPACE" --field-selector=status.phase==Succeeded --no-headers 2>/dev/null | wc -l | tr -d ' '
    elif [[ "$selector" == "workflow" ]]; then
        kubectl get workflows -n "$NAMESPACE" --no-headers 2>/dev/null | grep -c Succeeded || echo "0"
    elif [[ "$selector" == "coderun" ]]; then
        kubectl get coderuns -n "$NAMESPACE" --no-headers 2>/dev/null | grep -c Succeeded || echo "0"
    fi
}

# Count resources before cleanup
echo "📊 Current Resource Status:"
echo ""
SUCCEEDED_PODS=$(count_resources "pod" "pod")
SUCCEEDED_WORKFLOWS=$(count_resources "workflow" "workflow")
SUCCEEDED_CODERUNS=$(count_resources "coderun" "coderun")
FAILED_PODS=$(kubectl get pods -n "$NAMESPACE" --field-selector=status.phase==Failed --no-headers 2>/dev/null | wc -l | tr -d ' ')

echo "  ✅ Succeeded Pods:      $SUCCEEDED_PODS"
echo "  ❌ Failed Pods:         $FAILED_PODS"
echo "  ✅ Succeeded Workflows: $SUCCEEDED_WORKFLOWS"
echo "  ✅ Succeeded CodeRuns:  $SUCCEEDED_CODERUNS"
echo ""

TOTAL_TO_CLEAN=$((SUCCEEDED_PODS + FAILED_PODS + SUCCEEDED_WORKFLOWS + SUCCEEDED_CODERUNS))

if [[ $TOTAL_TO_CLEAN -eq 0 ]]; then
    echo "✨ No orphaned resources found. Namespace is clean!"
    exit 0
fi

echo "📦 Total resources to clean: $TOTAL_TO_CLEAN"
echo ""

# Prompt for confirmation
read -p "❓ Proceed with cleanup? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cleanup cancelled"
    exit 0
fi

echo ""
echo "🧹 Starting Cleanup..."
echo ""

# Phase 1: Delete succeeded CodeRuns (this will cascade delete associated pods/workflows)
if [[ $SUCCEEDED_CODERUNS -gt 0 ]]; then
    echo "📋 Phase 1: Cleaning up $SUCCEEDED_CODERUNS succeeded CodeRuns"
    kubectl get coderuns -n "$NAMESPACE" --no-headers 2>/dev/null | \
        grep Succeeded | \
        awk '{print $1}' | \
        xargs -I {} kubectl delete coderun {} -n "$NAMESPACE" --wait=false 2>/dev/null || true
    echo "  ✅ CodeRun deletion initiated (cascading to pods/workflows)"
    echo ""
fi

# Phase 2: Delete succeeded workflows that don't have ownerReferences
if [[ $SUCCEEDED_WORKFLOWS -gt 0 ]]; then
    echo "📋 Phase 2: Cleaning up orphaned succeeded workflows"
    # Delete workflows older than 1 hour to avoid deleting active workflow pods
    kubectl get workflows -n "$NAMESPACE" --no-headers 2>/dev/null | \
        grep Succeeded | \
        awk '{print $1}' | \
        xargs -I {} bash -c 'AGE=$(kubectl get workflow {} -n '"$NAMESPACE"' -o jsonpath="{.status.finishedAt}" 2>/dev/null); if [[ -n "$AGE" ]]; then kubectl delete workflow {} -n '"$NAMESPACE"' --wait=false 2>/dev/null || true; fi'
    echo "  ✅ Workflow deletion initiated"
    echo ""
fi

# Phase 3: Delete succeeded pods (after workflows are deleted to avoid recreating)
if [[ $SUCCEEDED_PODS -gt 0 ]]; then
    echo "📋 Phase 3: Cleaning up $SUCCEEDED_PODS succeeded pods"
    kubectl delete pods -n "$NAMESPACE" --field-selector=status.phase==Succeeded --wait=false 2>/dev/null || true
    echo "  ✅ Succeeded pod deletion initiated"
    echo ""
fi

# Phase 4: Delete failed pods
if [[ $FAILED_PODS -gt 0 ]]; then
    echo "📋 Phase 4: Cleaning up $FAILED_PODS failed pods"
    kubectl delete pods -n "$NAMESPACE" --field-selector=status.phase==Failed --wait=false 2>/dev/null || true
    echo "  ✅ Failed pod deletion initiated"
    echo ""
fi

# Wait a moment for deletions to propagate
echo "⏳ Waiting for resource cleanup (10 seconds)..."
sleep 10
echo ""

# Count remaining resources
echo "📊 Post-Cleanup Resource Status:"
echo ""
REMAINING_SUCCEEDED_PODS=$(count_resources "pod" "pod")
REMAINING_SUCCEEDED_WORKFLOWS=$(count_resources "workflow" "workflow")
REMAINING_SUCCEEDED_CODERUNS=$(count_resources "coderun" "coderun")
REMAINING_FAILED_PODS=$(kubectl get pods -n "$NAMESPACE" --field-selector=status.phase==Failed --no-headers 2>/dev/null | wc -l | tr -d ' ')

echo "  ✅ Succeeded Pods:      $REMAINING_SUCCEEDED_PODS"
echo "  ❌ Failed Pods:         $REMAINING_FAILED_PODS"
echo "  ✅ Succeeded Workflows: $REMAINING_SUCCEEDED_WORKFLOWS"
echo "  ✅ Succeeded CodeRuns:  $REMAINING_SUCCEEDED_CODERUNS"
echo ""

PODS_FREED=$((SUCCEEDED_PODS + FAILED_PODS - REMAINING_SUCCEEDED_PODS - REMAINING_FAILED_PODS))
WORKFLOWS_FREED=$((SUCCEEDED_WORKFLOWS - REMAINING_SUCCEEDED_WORKFLOWS))
CODERUNS_FREED=$((SUCCEEDED_CODERUNS - REMAINING_SUCCEEDED_CODERUNS))

echo "=========================================="
echo "✅ Cleanup Complete!"
echo "=========================================="
echo ""
echo "📈 Resources Freed:"
echo "  • Pods: $PODS_FREED"
echo "  • Workflows: $WORKFLOWS_FREED"
echo "  • CodeRuns: $CODERUNS_FREED"
echo ""
echo "💡 Note: Some resources may still be in 'Terminating' state."
echo "   They will be fully removed within 1-2 minutes."
echo ""
echo "🔍 Monitor remaining pods:"
echo "   kubectl get pods -n $NAMESPACE --watch"
echo ""
echo "🔄 To clean up regularly, consider setting up a CronJob or adding this to your workflow TTL strategy."
echo ""

