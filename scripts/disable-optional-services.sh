#!/bin/bash
# Script to disable optional monitoring and telemetry services
# to free up pod capacity in resource-constrained clusters

set -euo pipefail

echo "🔧 Disabling Optional Services for Pod Capacity"
echo ""
echo "This script will disable non-essential services to free up pod capacity."
echo "Services can be re-enabled later by re-syncing ArgoCD applications."
echo ""

# Function to disable an application
disable_app() {
    local app_name=$1
    local description=$2
    
    if kubectl get application -n argocd "$app_name" &>/dev/null; then
        echo "  ⏸️  Disabling $app_name ($description)"
        kubectl delete application -n argocd "$app_name" --wait=false
    else
        echo "  ⏭️  $app_name already disabled"
    fi
}

# Phase 1: Monitoring & Observability (recommended to disable first)
echo "📊 Phase 1: Disabling Monitoring & Observability Stack"
echo "   Estimated pod savings: ~8-12 pods"
echo ""

disable_app "grafana" "metrics visualization"
disable_app "victoria-metrics" "metrics storage"
disable_app "victoria-logs" "log storage"
disable_app "otel-collector" "observability collector"
disable_app "monitoring-stack" "monitoring infrastructure"
disable_app "fluent-bit" "log forwarding (DaemonSet)"

echo ""
echo "✅ Phase 1 complete"
echo ""

# Phase 2: Optional Network Services
read -p "❓ Disable Twingate VPN connectors? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🌐 Phase 2: Disabling VPN Connectors"
    echo "   Estimated pod savings: ~2 pods"
    echo ""
    
    disable_app "twingate-pastoral" "VPN connector"
    disable_app "twingate-therapeutic" "VPN connector"
    
    echo ""
    echo "✅ Phase 2 complete"
    echo ""
else
    echo "⏭️  Skipping Phase 2"
    echo ""
fi

# Phase 3: CI/CD Runners
read -p "❓ Disable platform runners? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🏃 Phase 3: Disabling CI/CD Runners"
    echo "   Estimated pod savings: Variable based on runner count"
    echo ""
    
    disable_app "platform-runners" "Kubernetes runners for CI/CD"
    
    echo ""
    echo "✅ Phase 3 complete"
    echo ""
else
    echo "⏭️  Skipping Phase 3"
    echo ""
fi

echo "=========================================="
echo "✅ Optional Services Disabled Successfully"
echo "=========================================="
echo ""
echo "📋 What happens next:"
echo "   1. ArgoCD will delete the disabled applications"
echo "   2. Kubernetes will terminate associated pods (~2-5 minutes)"
echo "   3. Pod capacity will be freed on worker node"
echo ""
echo "📊 Monitor the cleanup:"
echo "   kubectl get pods -A --watch"
echo ""
echo "🔄 To re-enable services later:"
echo "   1. Re-apply the ArgoCD application YAML files from infra/gitops/applications/"
echo "   2. Or let ArgoCD auto-sync from main branch"
echo ""
echo "💡 Tip: Check current pod count:"
echo "   kubectl get pods -A -o wide | grep 192.168.1.72 | wc -l"
echo ""

