#!/bin/bash
# Poll for deployment completion: ArgoCD sync + pod readiness
# Usage: poll-deploy.sh --app controller [--namespace cto] [--selector app=agent-controller]
#
# This script:
# 1. Waits for ArgoCD app to sync and become healthy
# 2. Waits for pods matching the selector to be ready

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/argocd.sh"
source "$SCRIPT_DIR/../lib/kubernetes.sh"

# Parse arguments
app_name=""
namespace="cto"
selector="app=agent-controller"
timeout="600"  # 10 minutes default

while [ $# -gt 0 ]; do
  case "$1" in
    --app) app_name="$2"; shift 2 ;;
    --namespace) namespace="$2"; shift 2 ;;
    --selector) selector="$2"; shift 2 ;;
    --timeout) timeout="$2"; shift 2 ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$app_name" ]; then
  log_error "Required: --app"
  exit 1
fi

log_info "Waiting for deployment: app=$app_name, namespace=$namespace, selector=$selector"

# Step 1: Wait for ArgoCD sync
log_step "Phase 1: Waiting for ArgoCD app '$app_name' to sync..."
if ! argocd_wait_healthy "$app_name" "$timeout"; then
  log_error "ArgoCD sync failed or timed out"
  argocd_debug_status "$app_name"
  exit 1
fi
log_success "ArgoCD app synced and healthy"

# Step 2: Wait for pods to be ready
log_step "Phase 2: Waiting for pods to be ready..."
if ! k8s_wait_pods_ready "$namespace" "$selector" 300; then
  log_error "Pods not ready after timeout"
  k8s_describe_pods "$namespace" "$selector"
  k8s_recent_events "$namespace" "$selector"
  exit 1
fi
log_success "All pods ready"

# Get current state for verification
log_info "Deployment verification complete:"
echo "  ArgoCD Revision: $(argocd_current_revision "$app_name")"
echo "  Pods: $(k8s_pod_names "$namespace" "$selector")"

log_success "Deployment complete and verified"

