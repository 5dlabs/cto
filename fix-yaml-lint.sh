#!/bin/bash

# Script to fix common YAML linting issues

echo "Fixing YAML linting issues..."

# Function to add document start marker and newline to files
fix_document_start_and_newline() {
    local file="$1"
    if [[ -f "$file" ]]; then
        # Add document start marker if missing
        if ! grep -q "^---" "$file"; then
            sed -i '' '1i\
---
' "$file"
        fi
        
        # Add newline at end if missing
        if [[ "$(tail -c1 "$file" | wc -l)" -eq 0 ]]; then
            echo "" >> "$file"
        fi
    fi
}

# Function to remove trailing spaces
remove_trailing_spaces() {
    local file="$1"
    if [[ -f "$file" ]]; then
        sed -i '' 's/[[:space:]]*$//' "$file"
    fi
}

# List of files that need document start and newline fixes
files_to_fix=(
    "infra/charts/k8s-mcp/Chart.yaml"
    "infra/charts/k8s-mcp/values.yaml"
    "infra/charts/k8s-mcp/templates/deployment.yaml"
    "infra/charts/k8s-mcp/templates/service.yaml"
    "infra/charts/claude-agent/Chart.yaml"
    "infra/charts/claude-agent/values.yaml"
    "infra/charts/claude-agent/templates/service.yaml"
    "infra/charts/claude-agent/templates/statefulset.yaml"
    "infra/charts/controller/Chart.yaml"
    "infra/charts/controller/values.yaml"
    "infra/charts/controller/crds/coderun-crd.yaml"
    "infra/charts/controller/crds/docsrun-crd.yaml"
    "infra/charts/controller/templates/deployment.yaml"
    "infra/charts/controller/templates/agents-configmap.yaml"
    "infra/charts/controller/templates/ingress.yaml"
    "infra/charts/controller/templates/docsrun-template.yaml"
    "infra/charts/controller/templates/project-intake-template.yaml"
    "infra/charts/controller/templates/service.yaml"
    "infra/charts/controller/templates/task-controller-config.yaml"
    "infra/charts/controller/templates/rbac.yaml"
    "infra/charts/controller/templates/claude-templates-configmap.yaml"
    "infra/charts/controller/templates/namespace.yaml"
    "infra/charts/controller/templates/coderun-template.yaml"
    "infra/charts/controller/templates/serviceaccount.yaml"
    "infra/charts/controller/templates/configmap.yaml"
    "infra/charts/controller/templates/stage-transitions-template.yaml"
    "infra/charts/controller/templates/secret.yaml"
    "infra/charts/controller/templates/workflow-rbac.yaml"
    "infra/charts/controller/templates/workflowtemplates/play-project-workflow-template.yaml"
    "infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml"
    "infra/charts/controller/templates/workflowtemplates/agent-mount-smoke.yaml"
    "infra/charts/rustdocs-mcp/Chart.yaml"
    "infra/charts/rustdocs-mcp/values.yaml"
    "infra/charts/rustdocs-mcp/templates/deployment.yaml"
    "infra/charts/rustdocs-mcp/templates/service.yaml"
    "infra/cluster-config/otel-collector-metrics-service.yaml"
    "infra/cluster-config/otel-prometheus-service.yaml"
    "infra/cluster-config/local-path-config-patch.yaml"
    "infra/cluster-config/talos-local-path-volume.yaml"
    "infra/examples/play-workflow-instance.yaml"
    "infra/examples/test-stage-transitions.yaml"
    "infra/runners/platform-org-runners.yaml"
    "infra/telemetry/telemetry-dashboards/Chart.yaml"
    "infra/telemetry/telemetry-dashboards/dashboards/dashboard-configmap.yaml"
    "infra/telemetry/values/otel-collector.yaml"
    "infra/telemetry/values/victoria-logs.yaml"
    "infra/telemetry/values/grafana.yaml"
    "infra/telemetry/values/claude-code-telemetry.yaml"
    "infra/telemetry/values/victoria-metrics.yaml"
    "infra/telemetry/alerts/claude-code-alerts.yaml"
    "infra/telemetry/alerts/alerting-rules-configmap.yaml"
    "infra/gitops/resources/github-webhooks/merge-to-main-sensor.yaml"
    "infra/gitops/resources/github-webhooks/stage-aware-resume-sensor.yaml"
    "docs/requirements.yaml"
    "docs/references/argo-events/trigger-with-template.yaml"
    "docs/references/argo-events/complete-trigger-parameterization.yaml"
    "docs/references/argo-events/special-workflow-trigger.yaml"
    "docs/references/argo-events/github-eventsource.yaml"
    "docs/references/argo-events/github.yaml"
    "docs/references/argo-events/trigger-standard-k8s-resource.yaml"
    "docs/examples/example-requirements.yaml"
    ".github/kind-config.yaml"
    ".github/secret-scanning.yml"
    ".github/workflows/infrastructure-build.yaml"
    ".github/workflows/controller-ci.yaml"
    ".github/workflows/agents-build.yaml"
    ".github/workflows/release.yaml"
    ".github/workflows/runner-builld.yaml"
    ".github/workflows/infra-ci.yaml"
    ".github/workflows/develop-deploy.yaml"
    ".github/workflows/helm-publish.yaml"
)

# Fix document start and newline issues
for file in "${files_to_fix[@]}"; do
    if [[ -f "$file" ]]; then
        echo "Fixing $file..."
        fix_document_start_and_newline "$file"
        remove_trailing_spaces "$file"
    fi
done

# Fix specific files with more complex issues
echo "Fixing specific files with complex issues..."

# Fix runner-cache files
if [[ -f "infra/runner-cache/cache-pvc.yaml" ]]; then
    remove_trailing_spaces "infra/runner-cache/cache-pvc.yaml"
    echo "" >> "infra/runner-cache/cache-pvc.yaml"
fi

if [[ -f "infra/runner-cache/values.yaml" ]]; then
    # Only add document start marker if missing
    if ! grep -q "^---" "infra/runner-cache/values.yaml"; then
        sed -i '' '1i\
---
' "infra/runner-cache/values.yaml"
    fi
    remove_trailing_spaces "infra/runner-cache/values.yaml"
    echo "" >> "infra/runner-cache/values.yaml"
fi

echo "YAML linting fixes completed!"
