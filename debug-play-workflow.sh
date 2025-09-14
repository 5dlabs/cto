#!/bin/bash

echo "🔍 Play Workflow Debug Investigation"
echo "====================================="
echo "Timestamp: $(date)"
echo ""

# Create debug output directory
mkdir -p debug-output
cd debug-output

echo "1. Checking git status and recent commits..."
git log --oneline -10 > git-log.txt
git status > git-status.txt

echo "2. Checking current CodeRun resources..."
kubectl get coderun -n agent-platform -o yaml > current-coderuns.yaml
kubectl get coderun -n agent-platform > coderun-list.txt

echo "3. Checking controller ConfigMap (container scripts)..."
kubectl get configmap controller-claude-templates -n agent-platform -o yaml > controller-templates-configmap.yaml

echo "4. Checking if our validation function is in the deployed ConfigMap..."
kubectl get configmap controller-claude-templates -n agent-platform -o jsonpath='{.data.container\.sh\.hbs}' | grep -c "is_valid_cfg" > validation-function-count.txt || echo "0" > validation-function-count.txt

echo "5. Checking controller configuration (Helm values)..."
kubectl get configmap controller-task-controller-config -n agent-platform -o yaml > controller-config.yaml

echo "6. Checking controller logs..."
kubectl logs -n agent-platform deployment/controller --tail=200 > controller-logs.txt

echo "7. Checking current workflow status..."
kubectl get workflow -n agent-platform -o yaml > current-workflows.yaml

echo "8. Checking if controller debug output exists..."
kubectl logs -n agent-platform deployment/controller --tail=500 | grep -E "DEBUG.*generate_client_config|DEBUG.*github_app|DEBUG.*agents|DEBUG.*tools-config" > controller-debug-logs.txt || echo "No debug output found" > controller-debug-logs.txt

echo "9. Checking agent definitions in Helm values..."
grep -A 50 "agents:" /Users/jonathonfritz/code/work-projects/5dlabs/cto/infra/charts/controller/values.yaml > helm-agents-config.txt

echo "10. Checking if our MCP server changes are active..."
ps aux | grep cto-mcp > mcp-processes.txt

echo ""
echo "Debug information collected in debug-output/ directory:"
echo "======================================================"
ls -la
echo ""
echo "Key files to examine:"
echo "- validation-function-count.txt (should be > 0 if changes deployed)"
echo "- controller-debug-logs.txt (should show our debug output)"
echo "- controller-config.yaml (should show agent definitions)"
echo "- current-coderuns.yaml (shows tools-config annotation)"
echo "- controller-logs.txt (full controller logs)"
echo ""
echo "Run this script and examine the output files!"
