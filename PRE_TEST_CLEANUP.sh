#!/bin/bash
# Pre-Test Cleanup Script
# Run this before testing with fixed code to ensure clean slate

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  PRE-TEST CLEANUP FOR PR #1551"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

NAMESPACE="agent-platform"

# ============================================================================
# STEP 1: Check Current State
# ============================================================================
echo "📊 STEP 1: Checking current state..."
echo ""

echo "Workflows:"
kubectl get workflows -n $NAMESPACE --no-headers | wc -l | xargs echo "  Count:"

echo "CodeRuns:"
kubectl get coderuns -n $NAMESPACE -o custom-columns=NAME:.metadata.name,TASK:.metadata.labels.task-id,STAGE:.metadata.labels.stage,PHASE:.status.phase,COMPLETED:.status.workCompleted 2>/dev/null || echo "  None found"

echo ""
echo "Jobs:"
kubectl get jobs -n $NAMESPACE --no-headers 2>/dev/null | wc -l | xargs echo "  Count:"

echo ""
echo "Pods:"
kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | wc -l | xargs echo "  Count:"

echo ""

# ============================================================================
# STEP 2: Delete Stale CodeRuns (Optional - only if expired)
# ============================================================================
echo "📋 STEP 2: Checking for expired CodeRuns..."
echo ""

# Only delete CodeRuns that have work_completed=true and are expired
EXPIRED_CODERUNS=$(kubectl get coderuns -n $NAMESPACE -o json | jq -r '.items[] | select(.status.workCompleted == true and (.status.expireAt // "" | . != "" and . < now | strftime("%Y-%m-%dT%H:%M:%SZ"))) | .metadata.name' 2>/dev/null || true)

if [ -n "$EXPIRED_CODERUNS" ]; then
  echo "Found expired CodeRuns:"
  echo "$EXPIRED_CODERUNS" | while read -r name; do
    echo "  - $name"
  done
  echo ""
  echo "Deleting expired CodeRuns..."
  echo "$EXPIRED_CODERUNS" | while read -r name; do
    kubectl delete coderun -n $NAMESPACE "$name" --ignore-not-found=true
    echo "  ✓ Deleted $name"
  done
else
  echo "  ✓ No expired CodeRuns found"
fi

echo ""

# ============================================================================
# STEP 3: Delete Stale Per-CodeRun ConfigMaps
# ============================================================================
echo "🗂️  STEP 3: Cleaning up stale per-CodeRun ConfigMaps..."
echo ""

STALE_CONFIGMAPS=$(kubectl get configmaps -n $NAMESPACE -o name | grep "code-agent-platform-coderun" || true)

if [ -n "$STALE_CONFIGMAPS" ]; then
  echo "Found per-CodeRun ConfigMaps:"
  echo "$STALE_CONFIGMAPS" | while read -r cm; do
    echo "  - $(basename $cm)"
  done
  echo ""
  echo "Deleting stale ConfigMaps..."
  echo "$STALE_CONFIGMAPS" | while read -r cm; do
    kubectl delete -n $NAMESPACE "$cm" --ignore-not-found=true
    echo "  ✓ Deleted $(basename $cm)"
  done
else
  echo "  ✓ No stale ConfigMaps found"
fi

echo ""

# ============================================================================
# STEP 4: Verify No Hung Pods/Jobs
# ============================================================================
echo "🔍 STEP 4: Checking for hung pods/jobs..."
echo ""

RUNNING_JOBS=$(kubectl get jobs -n $NAMESPACE -o json | jq -r '.items[] | select(.status.active > 0) | .metadata.name' 2>/dev/null || true)

if [ -n "$RUNNING_JOBS" ]; then
  echo "⚠️  Found running jobs:"
  echo "$RUNNING_JOBS" | while read -r job; do
    START_TIME=$(kubectl get job -n $NAMESPACE "$job" -o jsonpath='{.status.startTime}')
    echo "  - $job (started: $START_TIME)"
  done
  echo ""
  echo "❌ WARNING: Running jobs found. Delete them before testing:"
  echo ""
  echo "$RUNNING_JOBS" | while read -r job; do
    echo "  kubectl delete job -n $NAMESPACE $job"
  done
  echo ""
  exit 1
else
  echo "  ✓ No hung jobs found"
fi

echo ""

# ============================================================================
# STEP 5: Clean Workspace PVCs (OPTIONAL - only if needed)
# ============================================================================
echo "💾 STEP 5: Checking workspace PVCs..."
echo ""

kubectl get pvc -n $NAMESPACE | grep workspace || echo "  No workspace PVCs found"

echo ""
echo "ℹ️  NOTE: PVCs are persistent and reused across runs."
echo "   Only delete if you want a completely fresh workspace."
echo ""

# ============================================================================
# STEP 6: Verify Controller is Healthy
# ============================================================================
echo "🏥 STEP 6: Verifying controller health..."
echo ""

CONTROLLER_POD=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=controller -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)

if [ -n "$CONTROLLER_POD" ]; then
  STATUS=$(kubectl get pod -n $NAMESPACE "$CONTROLLER_POD" -o jsonpath='{.status.phase}')
  READY=$(kubectl get pod -n $NAMESPACE "$CONTROLLER_POD" -o jsonpath='{.status.containerStatuses[0].ready}')
  RESTARTS=$(kubectl get pod -n $NAMESPACE "$CONTROLLER_POD" -o jsonpath='{.status.containerStatuses[0].restartCount}')
  
  echo "  Controller pod: $CONTROLLER_POD"
  echo "  Status: $STATUS"
  echo "  Ready: $READY"
  echo "  Restarts: $RESTARTS"
  
  if [ "$STATUS" != "Running" ] || [ "$READY" != "true" ]; then
    echo ""
    echo "❌ Controller is not healthy!"
    echo "   Check logs: kubectl logs -n $NAMESPACE $CONTROLLER_POD"
    exit 1
  else
    echo "  ✓ Controller is healthy"
  fi
else
  echo "  ❌ Controller pod not found!"
  exit 1
fi

echo ""

# ============================================================================
# STEP 7: Verify ConfigMaps are Synced
# ============================================================================
echo "🔄 STEP 7: Verifying ConfigMaps are synced from main..."
echo ""

# Check if ArgoCD has synced recently
ARGOCD_SYNC=$(kubectl get app controller -n argocd -o jsonpath='{.status.sync.status}' 2>/dev/null || echo "unknown")
LAST_SYNC=$(kubectl get app controller -n argocd -o jsonpath='{.status.operationState.finishedAt}' 2>/dev/null || echo "unknown")

echo "  ArgoCD sync status: $ARGOCD_SYNC"
echo "  Last synced: $LAST_SYNC"

if [ "$ARGOCD_SYNC" != "Synced" ]; then
  echo ""
  echo "⚠️  WARNING: ArgoCD not synced. You may be running old ConfigMaps."
  echo "   Wait for sync or manually sync:"
  echo "   argocd app sync controller"
  echo ""
fi

# Check controller ConfigMap age
CLAUDE_CM_AGE=$(kubectl get configmap -n $NAMESPACE controller-agent-templates-claude -o jsonpath='{.metadata.creationTimestamp}' 2>/dev/null || echo "not found")
echo "  controller-agent-templates-claude: $CLAUDE_CM_AGE"

echo ""

# ============================================================================
# SUMMARY
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✅ CLEANUP COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Cluster state:"
echo "  - Workflows: Clean (all deleted)"
echo "  - Jobs: Clean (all deleted)"
echo "  - CodeRuns: $(kubectl get coderuns -n $NAMESPACE --no-headers | wc -l | xargs) remaining (completed)"
echo "  - Controller: Healthy and ready"
echo ""
echo "✅ Ready for testing with PR #1551 changes!"
echo ""
echo "Next steps:"
echo "  1. Merge PR #1551"
echo "  2. Wait for ArgoCD sync (~2 min)"
echo "  3. Run test workflow"
echo ""

