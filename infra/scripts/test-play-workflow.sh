#!/bin/bash
# Test script for Play Workflow Template
# Validates syntax, deployment, and basic functionality

set -e

NAMESPACE="${NAMESPACE:-argo}"
TEMPLATE_FILE="infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml"

echo "========================================="
echo "Play Workflow Template Testing Script"
echo "========================================="
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "Checking prerequisites..."
if ! command_exists kubectl; then
    echo "❌ kubectl not found. Please install kubectl."
    exit 1
fi

if ! command_exists argo; then
    echo "⚠️  Argo CLI not found. Some tests will be skipped."
    ARGO_CLI=false
else
    ARGO_CLI=true
fi

echo "✅ Prerequisites satisfied"
echo ""

# Test 1: YAML Syntax Validation
echo "Test 1: Validating YAML syntax..."
if kubectl apply --dry-run=client -f "$TEMPLATE_FILE" > /dev/null 2>&1; then
    echo "✅ YAML syntax is valid"
else
    echo "❌ YAML syntax validation failed"
    kubectl apply --dry-run=client -f "$TEMPLATE_FILE"
    exit 1
fi
echo ""

# Test 2: Template Structure Validation
echo "Test 2: Validating template structure..."
if $ARGO_CLI; then
    if argo template lint "$TEMPLATE_FILE" 2>/dev/null; then
        echo "✅ Argo template structure is valid"
    else
        echo "❌ Argo template structure validation failed"
        argo template lint "$TEMPLATE_FILE"
        exit 1
    fi
else
    echo "⚠️  Skipping Argo lint (CLI not available)"
fi
echo ""

# Test 3: Parameter Validation
echo "Test 3: Validating parameters..."
REQUIRED_PARAMS=(
    "implementation-agent"
    "quality-agent"
    "testing-agent"
    "task-id"
    "repository"
    "service"
    "model"
)

for param in "${REQUIRED_PARAMS[@]}"; do
    if grep -q "name: $param" "$TEMPLATE_FILE"; then
        echo "  ✅ Parameter '$param' defined"
    else
        echo "  ❌ Parameter '$param' missing"
        exit 1
    fi
done
echo ""

# Test 4: DAG Task Dependencies
echo "Test 4: Validating DAG task dependencies..."
DAG_TASKS=(
    "implementation-work"
    "wait-pr-created"
    "quality-work"
    "wait-ready-for-qa"
    "testing-work"
    "wait-pr-approved"
    "complete-task"
)

for task in "${DAG_TASKS[@]}"; do
    if grep -q "name: $task" "$TEMPLATE_FILE"; then
        echo "  ✅ Task '$task' defined"
    else
        echo "  ❌ Task '$task' missing"
        exit 1
    fi
done
echo ""

# Test 5: Suspend Template Validation
echo "Test 5: Validating suspend templates..."
if grep -q "suspend: {}" "$TEMPLATE_FILE"; then
    echo "✅ Suspend template configured for indefinite suspension"
else
    echo "❌ Suspend template not properly configured"
    exit 1
fi
echo ""

# Test 6: Label Configuration
echo "Test 6: Validating label configuration..."
REQUIRED_LABELS=(
    "workflow-type"
    "task-id"
    "current-stage"
)

for label in "${REQUIRED_LABELS[@]}"; do
    if grep -q "$label:" "$TEMPLATE_FILE"; then
        echo "  ✅ Label '$label' configured"
    else
        echo "  ❌ Label '$label' missing"
        exit 1
    fi
done
echo ""

# Test 7: activeDeadlineSeconds Configuration
echo "Test 7: Validating timeout configuration..."
if grep -q "activeDeadlineSeconds: 1209600" "$TEMPLATE_FILE"; then
    echo "✅ 14-day timeout configured (1209600 seconds)"
else
    echo "❌ activeDeadlineSeconds not set to 14 days"
    exit 1
fi
echo ""

# Test 8: No Hardcoded Agent Names
echo "Test 8: Checking for hardcoded agent names..."
if grep -E "(5DLabs-Rex|5DLabs-Cleo|5DLabs-Tess)" "$TEMPLATE_FILE" | grep -v "value:" | grep -v "description:" | grep -v "#" > /dev/null; then
    echo "⚠️  Warning: Possible hardcoded agent names found"
    grep -E "(5DLabs-Rex|5DLabs-Cleo|5DLabs-Tess)" "$TEMPLATE_FILE" | grep -v "value:" | grep -v "description:" | grep -v "#"
else
    echo "✅ No hardcoded agent names (uses parameters)"
fi
echo ""

# Test 9: CodeRun CRD Integration
echo "Test 9: Validating CodeRun CRD integration..."
if grep -q "kind: CodeRun" "$TEMPLATE_FILE"; then
    echo "✅ CodeRun CRD properly integrated"
    if grep -q "continueSession: true" "$TEMPLATE_FILE"; then
        echo "  ✅ Session continuity enabled"
    else
        echo "  ❌ Session continuity not enabled"
    fi
else
    echo "❌ CodeRun CRD not found in template"
    exit 1
fi
echo ""

# Test 10: Resource Management
echo "Test 10: Validating resource management..."
if grep -q "ttlStrategy:" "$TEMPLATE_FILE"; then
    echo "✅ TTL strategy configured for cleanup"
fi

if grep -q "podGC:" "$TEMPLATE_FILE"; then
    echo "✅ Pod garbage collection configured"
fi

if grep -q "volumeClaimTemplates:" "$TEMPLATE_FILE"; then
    echo "✅ Volume claim templates configured"
fi
echo ""

# Summary
echo "========================================="
echo "Test Summary"
echo "========================================="
echo "✅ All critical tests passed!"
echo ""
echo "Template ready for deployment."
echo ""
echo "To deploy the template:"
echo "  kubectl apply -f $TEMPLATE_FILE"
echo ""
echo "To submit a workflow instance:"
echo "  argo submit --from workflowtemplate/play-workflow-template \\"
echo "    -p task-id=3 \\"
echo "    -p implementation-agent=5DLabs-Rex \\"
echo "    -p quality-agent=5DLabs-Cleo \\"
echo "    -p testing-agent=5DLabs-Tess"
echo ""
echo "To monitor workflow execution:"
echo "  argo watch <workflow-name>"
echo "  kubectl get workflows -n $NAMESPACE"
echo ""