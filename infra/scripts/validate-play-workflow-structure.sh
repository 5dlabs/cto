#!/bin/bash
# Validate Play Workflow Template Structure
# This script validates the template structure without requiring Helm

set -e

TEMPLATE_FILE="infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml"

echo "========================================="
echo "Play Workflow Structure Validation"
echo "========================================="
echo ""

# Test 1: Check Helm conditional and basic structure
echo "Test 1: Validating Helm template structure..."
if head -1 "$TEMPLATE_FILE" | grep -q "{{- if .Values.argo.enabled }}"; then
    echo "✅ Helm conditional found"
else
    echo "❌ Missing Helm conditional"
    exit 1
fi

if tail -1 "$TEMPLATE_FILE" | grep -q "{{- end }}"; then
    echo "✅ Helm conditional properly closed"
else
    echo "❌ Helm conditional not closed"
    exit 1
fi
echo ""

# Test 2: Check WorkflowTemplate definition
echo "Test 2: Validating WorkflowTemplate definition..."
if grep -q "kind: WorkflowTemplate" "$TEMPLATE_FILE"; then
    echo "✅ WorkflowTemplate kind defined"
else
    echo "❌ WorkflowTemplate kind missing"
    exit 1
fi

if grep -q "name: play-workflow-template" "$TEMPLATE_FILE"; then
    echo "✅ Template name defined"
else
    echo "❌ Template name missing"
    exit 1
fi
echo ""

# Test 3: Check required parameters
echo "Test 3: Checking required parameters..."
PARAMS=(
    "implementation-agent"
    "quality-agent"
    "testing-agent"
    "task-id"
    "repository"
    "service"
    "model"
)

for param in "${PARAMS[@]}"; do
    if grep -q "\- name: $param" "$TEMPLATE_FILE"; then
        echo "  ✅ Parameter: $param"
    else
        echo "  ❌ Missing parameter: $param"
        exit 1
    fi
done
echo ""

# Test 4: Check DAG tasks
echo "Test 4: Checking DAG task structure..."
TASKS=(
    "implementation-work"
    "wait-pr-created"
    "quality-work"
    "wait-ready-for-qa"
    "testing-work"
    "wait-pr-approved"
    "complete-task"
)

for task in "${TASKS[@]}"; do
    if grep -q "\- name: $task" "$TEMPLATE_FILE"; then
        echo "  ✅ Task: $task"
    else
        echo "  ❌ Missing task: $task"
        exit 1
    fi
done
echo ""

# Test 5: Check templates
echo "Test 5: Checking template definitions..."
TEMPLATES=(
    "main"
    "agent-coderun"
    "suspend-for-event"
    "task-completion"
    "cleanup-handler"
)

for template in "${TEMPLATES[@]}"; do
    if grep -q "\- name: $template" "$TEMPLATE_FILE"; then
        echo "  ✅ Template: $template"
    else
        echo "  ❌ Missing template: $template"
        exit 1
    fi
done
echo ""

# Test 6: Check suspend configuration
echo "Test 6: Validating suspend configuration..."
if grep -q "suspend: {}" "$TEMPLATE_FILE"; then
    echo "✅ Indefinite suspend configured"
else
    echo "❌ Suspend not properly configured"
    exit 1
fi
echo ""

# Test 7: Check CodeRun integration
echo "Test 7: Validating CodeRun CRD integration..."
if grep -q "kind: CodeRun" "$TEMPLATE_FILE"; then
    echo "✅ CodeRun CRD integrated"
else
    echo "❌ CodeRun CRD missing"
    exit 1
fi

if grep -q "continueSession: true" "$TEMPLATE_FILE"; then
    echo "✅ Session continuity enabled"
else
    echo "❌ Session continuity not enabled"
    exit 1
fi
echo ""

# Test 8: Check workflow configuration
echo "Test 8: Validating workflow configuration..."
if grep -q "activeDeadlineSeconds: 1209600" "$TEMPLATE_FILE"; then
    echo "✅ 14-day timeout configured"
else
    echo "❌ Incorrect timeout configuration"
    exit 1
fi

if grep -q "serviceAccountName: argo-workflow" "$TEMPLATE_FILE"; then
    echo "✅ Service account configured"
else
    echo "❌ Service account missing"
    exit 1
fi
echo ""

# Test 9: Check labeling
echo "Test 9: Validating label configuration..."
LABELS=(
    "workflow-type: play-orchestration"
    "task-id:"
    "current-stage:"
    "github-app:"
)

for label in "${LABELS[@]}"; do
    if grep -q "$label" "$TEMPLATE_FILE"; then
        echo "  ✅ Label: ${label%:}"
    else
        echo "  ❌ Missing label: ${label%:}"
        exit 1
    fi
done
echo ""

# Test 10: Check parameter propagation
echo "Test 10: Validating parameter propagation..."
if grep -q '{{`{{workflow.parameters' "$TEMPLATE_FILE"; then
    echo "✅ Workflow parameter references found"
else
    echo "❌ No workflow parameter references"
    exit 1
fi

if grep -q '{{`{{inputs.parameters' "$TEMPLATE_FILE"; then
    echo "✅ Input parameter references found"
else
    echo "❌ No input parameter references"
    exit 1
fi
echo ""

# Summary
echo "========================================="
echo "Validation Summary"
echo "========================================="
echo "✅ All structure validation tests passed!"
echo ""
echo "The template structure is valid and contains:"
echo "- Proper Helm templating"
echo "- All required parameters"
echo "- Complete DAG task structure"
echo "- Suspend/resume configuration"
echo "- CodeRun CRD integration"
echo "- Proper labeling for correlation"
echo ""
echo "Note: This validates structure only."
echo "Full Helm rendering requires 'helm template' command."
echo ""