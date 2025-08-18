#!/bin/bash
# Test a single task from the minimal test project

set -e

TASK_ID="${1:-1}"
NAMESPACE="${NAMESPACE:-agent-platform}"

echo "=================================="
echo "🧪 Testing Minimal Task $TASK_ID"
echo "=================================="
echo ""

# Create workflow for single task
cat > /tmp/test-task-${TASK_ID}.yaml <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: test-minimal-task-${TASK_ID}-
  namespace: ${NAMESPACE}
  labels:
    task-id: "${TASK_ID}"
    test-type: "minimal"
    cost: "zero"
spec:
  workflowTemplateRef:
    name: play-workflow-template
  arguments:
    parameters:
      - name: task-id
        value: "${TASK_ID}"
      - name: repository
        value: "5dlabs/cto"
      - name: service
        value: "test-minimal"
      - name: docs-repository
        value: "https://github.com/5dlabs/cto"
      - name: docs-project-directory
        value: "test-project"
      - name: docs-branch
        value: "feat/minimal-test-project"
      - name: implementation-agent
        value: "5DLabs-Rex"
      - name: quality-agent
        value: "5DLabs-Cleo"
      - name: testing-agent
        value: "5DLabs-Tess"
      - name: model
        value: "claude-3-5-haiku-20241022"
EOF

# Submit workflow
WORKFLOW_NAME=$(kubectl create -f /tmp/test-task-${TASK_ID}.yaml -o jsonpath='{.metadata.name}')

echo "✅ Created workflow: $WORKFLOW_NAME"
echo ""
echo "📊 Monitor with:"
echo "  kubectl get workflow $WORKFLOW_NAME -n $NAMESPACE -w"
echo ""
echo "🔍 Check logs with:"
echo "  kubectl logs -f workflow/$WORKFLOW_NAME -n $NAMESPACE"
echo ""
echo "🧹 Clean up with:"
echo "  kubectl delete workflow $WORKFLOW_NAME -n $NAMESPACE"
