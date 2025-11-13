#!/bin/bash
# Cleanup script for stuck Atlas pods and CodeRuns

set -euo pipefail

echo "🧹 Cleaning up stuck Atlas resources..."
echo

# Delete failed/error Atlas CodeRuns
echo "1. Deleting failed Atlas CodeRuns..."
FAILED_CODERUNS=$(kubectl get coderun -n agent-platform -l agent=atlas --no-headers 2>/dev/null | grep -E "Failed|Error" | awk '{print $1}' || true)
if [ -n "$FAILED_CODERUNS" ]; then
    echo "$FAILED_CODERUNS" | xargs -r kubectl delete coderun -n agent-platform
    echo "  ✅ Deleted failed CodeRuns"
else
    echo "  ℹ️ No failed CodeRuns found"
fi

# Delete pods in Error state
echo
echo "2. Deleting pods in Error state..."
ERROR_PODS=$(kubectl get pods -n agent-platform -l agent=atlas --field-selector=status.phase=Failed --no-headers 2>/dev/null | awk '{print $1}' || true)
if [ -n "$ERROR_PODS" ]; then
    echo "$ERROR_PODS" | xargs -r kubectl delete pod -n agent-platform
    echo "  ✅ Deleted error pods"
else
    echo "  ℹ️ No error pods found"
fi

# Delete pods with Error status (not Failed phase)
echo
echo "3. Deleting pods with Error status..."
kubectl get pods -n agent-platform -l agent=atlas --no-headers 2>/dev/null | grep -E "Error|CrashLoopBackOff|ImagePullBackOff" | awk '{print $1}' | xargs -r kubectl delete pod -n agent-platform || echo "  ℹ️ No error status pods found"

# Delete old Atlas CodeRuns that have been running too long (>10 minutes)
echo
echo "4. Checking for stuck CodeRuns (running >10 minutes)..."
STUCK_CODERUNS=$(kubectl get coderun -n agent-platform -l agent=atlas --no-headers 2>/dev/null | awk '{
    cmd = "date -d \"" $6 "\" +%s 2>/dev/null || date -j -f \"%Y-%m-%dT%H:%M:%SZ\" \"" $6 "\" +%s 2>/dev/null"
    cmd | getline created
    close(cmd)
    now = systime()
    age = now - created
    if (age > 600) print $1
}' || true)

if [ -n "$STUCK_CODERUNS" ]; then
    echo "  Found stuck CodeRuns:"
    echo "$STUCK_CODERUNS"
    echo "  Deleting..."
    echo "$STUCK_CODERUNS" | xargs -r kubectl delete coderun -n agent-platform
    echo "  ✅ Deleted stuck CodeRuns"
else
    echo "  ℹ️ No stuck CodeRuns found"
fi

# Summary
echo
echo "════════════════════════════════════════"
echo "📊 Cleanup Summary"
echo "════════════════════════════════════════"
echo
echo "Remaining Atlas resources:"
kubectl get coderun -n agent-platform -l agent=atlas --no-headers 2>/dev/null | wc -l | xargs echo "  CodeRuns:"
kubectl get pods -n agent-platform -l agent=atlas --no-headers 2>/dev/null | wc -l | xargs echo "  Pods:"
echo
echo "✅ Cleanup complete!"
echo
echo "Next steps:"
echo "1. Merge PR with ConfigMap updates"
echo "2. Wait for ArgoCD sync"
echo "3. Restart controller: kubectl rollout restart deployment controller -n agent-platform"
echo "4. Trigger Atlas: gh pr comment <pr-number> --repo 5dlabs/cto --body 'Atlas test'"

