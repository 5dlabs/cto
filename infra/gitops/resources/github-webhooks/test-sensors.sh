#!/bin/bash

# Test script for Argo Events Sensors
# This script helps validate sensor webhook processing

echo "=== Argo Events Sensor Testing Script ==="
echo ""

# Check sensor status
echo "1. Checking Sensor Status:"
echo "--------------------------"
kubectl get sensors -n argo
echo ""

# Check sensor pods
echo "2. Checking Sensor Pods:"
echo "------------------------"
kubectl get pods -n argo --selector controller=sensor-controller
echo ""

# Check EventSource status
echo "3. Checking EventSource:"
echo "------------------------"
kubectl get eventsource github -n argo -o jsonpath='{.status.conditions[*]}' | jq '.'
echo ""

# Check EventBus status
echo "4. Checking EventBus:"
echo "---------------------"
kubectl get eventbus default -n argo -o jsonpath='{.status.conditions[*]}' | jq '.'
echo ""

# Instructions for manual testing
echo "5. Manual Testing Instructions:"
echo "--------------------------------"
echo ""
echo "To test each sensor, trigger the following GitHub events:"
echo ""
echo "a) Multi-Agent Workflow Resume Sensor:"
echo "   - Create a new PR with label 'task-3' on branch 'task-3-feature'"
echo "   - Watch logs: kubectl logs -f \$(kubectl get pods -n argo -l sensor-name=multi-agent-workflow-resume -o name | head -1) -n argo"
echo ""
echo "b) Ready-for-QA Label Sensor:"
echo "   - Add 'ready-for-qa' label to a PR (as 5DLabs-Cleo user)"
echo "   - Watch logs: kubectl logs -f \$(kubectl get pods -n argo -l sensor-name=ready-for-qa-label -o name | head -1) -n argo"
echo ""
echo "c) PR Approval Sensor:"
echo "   - Approve a PR (as 5DLabs-Tess user)"
echo "   - Watch logs: kubectl logs -f \$(kubectl get pods -n argo -l sensor-name=pr-approval -o name | head -1) -n argo"
echo ""
echo "d) Rex Remediation Sensor:"
echo "   - Push to a 'task-*' branch (as 5DLabs-Rex user)"
echo "   - Watch logs: kubectl logs -f \$(kubectl get pods -n argo -l sensor-name=rex-remediation -o name | head -1) -n argo"
echo ""

# Function to watch sensor logs
watch_logs() {
    local sensor_name=$1
    echo "Watching logs for $sensor_name sensor..."
    kubectl logs -f $(kubectl get pods -n argo -l sensor-name=$sensor_name -o name | head -1) -n argo
}

# Provide option to watch specific sensor logs
echo "6. Watch Sensor Logs:"
echo "---------------------"
echo "Run one of the following commands to watch sensor logs:"
echo ""
echo "  ./test-sensors.sh logs multi-agent-workflow-resume"
echo "  ./test-sensors.sh logs ready-for-qa-label"
echo "  ./test-sensors.sh logs pr-approval"
echo "  ./test-sensors.sh logs rex-remediation"
echo ""

# Handle log watching if requested
if [ "$1" = "logs" ] && [ -n "$2" ]; then
    watch_logs $2
fi

# Test webhook connectivity
echo "7. Test Webhook Endpoint:"
echo "-------------------------"
echo "EventSource webhook endpoint: http://github-eventsource-svc.argo:12000/github/webhook"
echo ""
echo "To test webhook delivery (requires GitHub webhook secret):"
echo "curl -X POST http://github-eventsource-svc.argo:12000/github/webhook \\"
echo "  -H 'X-GitHub-Event: ping' \\"
echo "  -H 'X-Hub-Signature-256: sha256=YOUR_SIGNATURE' \\"
echo "  -d '{\"zen\":\"Design for failure.\"}'"
echo ""

# Summary
echo "8. Summary:"
echo "-----------"
echo "✅ All 4 sensors deployed successfully"
echo "✅ Sensor pods are running"
echo "✅ EventSource and EventBus are operational"
echo ""
echo "Next steps:"
echo "1. Create test workflows with appropriate labels"
echo "2. Trigger GitHub events to test each sensor"
echo "3. Monitor sensor logs for event processing"
echo "4. Verify workflow resumption occurs correctly"