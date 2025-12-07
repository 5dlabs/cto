#!/bin/bash
# Test script to validate Atlas PR Guardian sensor fix

set -euo pipefail

echo "🔍 Testing Atlas PR Guardian Sensor Fix"
echo "========================================"
echo

# Check if sensor is deployed
echo "1. Checking if Atlas PR Guardian sensor is deployed..."
if kubectl get sensor atlas-pr-guardian -n argo &>/dev/null; then
    echo "   ✅ Sensor is deployed"
else
    echo "   ❌ Sensor not found"
    exit 1
fi

# Check sensor status
echo
echo "2. Checking sensor status..."
STATUS=$(kubectl get sensor atlas-pr-guardian -n argo -o jsonpath='{.status.conditions[?(@.type=="Deployed")].status}')
if [ "$STATUS" = "True" ]; then
    echo "   ✅ Sensor is deployed and healthy"
else
    echo "   ❌ Sensor deployment status: $STATUS"
    exit 1
fi

# Check for recent filtering errors in sensor logs
echo
echo "3. Checking sensor logs for filtering errors (last 100 lines)..."
POD=$(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1)
if [ -z "$POD" ]; then
    echo "   ❌ No sensor pod found"
    exit 1
fi

ERROR_COUNT=$(kubectl logs "$POD" -n argo --tail=100 2>/dev/null | grep -c "discarded due to filtering error" || true)
if [ "$ERROR_COUNT" -eq 0 ]; then
    echo "   ✅ No filtering errors found in recent logs"
else
    echo "   ⚠️  Found $ERROR_COUNT filtering errors in recent logs"
    echo "   Last error:"
    kubectl logs "$POD" -n argo --tail=100 2>/dev/null | grep "discarded due to filtering error" | tail -1
fi

# Check for recent CodeRuns created by Atlas
echo
echo "4. Checking for Atlas PR Guardian CodeRuns..."
CODERUN_COUNT=$(kubectl get coderun -n cto -l agent=atlas,role=pr-guardian --no-headers 2>/dev/null | wc -l)
echo "   Found $CODERUN_COUNT Atlas PR Guardian CodeRuns"

# Display sensor configuration
echo
echo "5. Current sensor filter expression:"
kubectl get sensor atlas-pr-guardian -n argo -o jsonpath='{.spec.dependencies[0].filters.exprs[0].expr}' | sed 's/^/   /'

echo
echo
echo "✅ Test complete!"
echo
echo "Expected behavior after fix:"
echo "  - pull_request events: Should pass filter (PR by definition)"
echo "  - pull_request_review events: Should pass filter (PR review by definition)"
echo "  - issue_comment events: Should only pass if comment is on a PR"
echo "  - Other events: Should be filtered out"
echo
echo "To test the fix:"
echo "  1. Apply the updated sensor: kubectl apply -f infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml"
echo "  2. Wait for ArgoCD to sync (or manually sync)"
echo "  3. Create a test PR in 5dlabs/cto"
echo "  4. Check if Atlas CodeRun is created: kubectl get coderun -n cto -l agent=atlas"
echo "  5. Monitor sensor logs: kubectl logs -f \$(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) -n argo"

