#!/bin/bash
# Test the Play Project Workflow

set -e

NAMESPACE="${NAMESPACE:-cto}"

echo "=================================="
echo "🎯 Testing Play Project Workflow"
echo "=================================="
echo ""

# Check if workflow template exists
echo "📋 Checking for play-project-workflow-template..."
if kubectl get workflowtemplate play-project-workflow-template -n "$NAMESPACE" &>/dev/null; then
    echo "✅ Template exists in cluster"
else
    echo "❌ Template not found. Applying from local file..."
    kubectl apply -f infra/charts/controller/templates/workflowtemplates/play-project-workflow-template.yaml
fi

# Create test workflow
echo ""
echo "🚀 Submitting test workflow for tasks 1-3..."

cat > /tmp/test-play-project.yaml <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: test-play-project-
  namespace: $NAMESPACE
  labels:
    workflow-type: project-test
    test-run: "true"
spec:
  workflowTemplateRef:
    name: play-project-workflow-template
  arguments:
    parameters:
      - name: start-from-task
        value: "1"
      - name: max-tasks
        value: "3"
      - name: repository
        value: "5dlabs/cto"
      - name: service
        value: "cto"
      - name: docs-repository
        value: "https://github.com/5dlabs/cto"
      - name: docs-project-directory
        value: "docs"
      - name: docs-branch
        value: "main"
      - name: implementation-agent
        value: "5DLabs-Rex"
      - name: quality-agent
        value: "5DLabs-Cleo"
      - name: testing-agent
        value: "5DLabs-Tess"
      - name: model
        value: "claude-3-5-sonnet-20241022"
EOF

# Submit workflow
WORKFLOW_NAME=$(kubectl create -f /tmp/test-play-project.yaml -o jsonpath='{.metadata.name}')

if [ -z "$WORKFLOW_NAME" ]; then
    echo "❌ Failed to create workflow"
    exit 1
fi

echo "✅ Created workflow: $WORKFLOW_NAME"
echo ""
echo "📊 Monitor with:"
echo "  kubectl get workflow $WORKFLOW_NAME -n $NAMESPACE -w"
echo "  kubectl logs -f workflow/$WORKFLOW_NAME -n $NAMESPACE"
echo ""
echo "🧹 Clean up with:"
echo "  kubectl delete workflow $WORKFLOW_NAME -n $NAMESPACE"
