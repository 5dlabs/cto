#!/bin/bash
# ArgoCD operations for E2E Watch Remediation Agent
# Requires: argocd CLI (uses in-cluster auth)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# ============================================================================
# ArgoCD Application Operations
# ============================================================================

# Get application status
# Usage: argocd_app_status APP_NAME
argocd_app_status() {
  local app_name="$1"
  
  argocd app get "$app_name" -o json 2>/dev/null
}

# Get sync status
# Usage: argocd_sync_status APP_NAME
argocd_sync_status() {
  local app_name="$1"
  
  argocd app get "$app_name" -o json | jq -r '.status.sync.status'
}

# Get health status
# Usage: argocd_health_status APP_NAME
argocd_health_status() {
  local app_name="$1"
  
  argocd app get "$app_name" -o json | jq -r '.status.health.status'
}

# Check if app is synced and healthy
# Usage: argocd_is_healthy APP_NAME
argocd_is_healthy() {
  local app_name="$1"
  
  local status
  status=$(argocd app get "$app_name" -o json 2>/dev/null)
  
  if [ -z "$status" ]; then
    log_warn "Could not get status for app: $app_name"
    return 1
  fi
  
  local sync_status health_status
  sync_status=$(echo "$status" | jq -r '.status.sync.status')
  health_status=$(echo "$status" | jq -r '.status.health.status')
  
  if [ "$sync_status" = "Synced" ] && [ "$health_status" = "Healthy" ]; then
    return 0
  fi
  
  log_info "App $app_name: sync=$sync_status, health=$health_status"
  return 1
}

# Trigger a sync
# Usage: argocd_sync APP_NAME [--prune]
argocd_sync() {
  local app_name="$1"
  local prune="${2:-}"
  
  log_step "Triggering sync for app: $app_name"
  
  if [ "$prune" = "--prune" ]; then
    argocd app sync "$app_name" --prune
  else
    argocd app sync "$app_name"
  fi
}

# Wait for app to be synced and healthy
# Usage: argocd_wait_healthy APP_NAME [timeout_seconds]
argocd_wait_healthy() {
  local app_name="$1"
  local timeout="${2:-600}"  # 10 min default
  
  log_step "Waiting for app '$app_name' to be synced and healthy"
  
  poll_until "$timeout" 15 "ArgoCD app $app_name healthy" argocd_is_healthy "$app_name"
}

# Get the current revision/commit
# Usage: argocd_current_revision APP_NAME
argocd_current_revision() {
  local app_name="$1"
  
  argocd app get "$app_name" -o json | jq -r '.status.sync.revision'
}

# Check if app has the expected revision deployed
# Usage: argocd_has_revision APP_NAME EXPECTED_SHA
argocd_has_revision() {
  local app_name="$1"
  local expected_sha="$2"
  
  local current
  current=$(argocd_current_revision "$app_name")
  
  # Check if current revision starts with expected (short SHA comparison)
  if [[ "$current" == "$expected_sha"* ]] || [[ "$expected_sha" == "$current"* ]]; then
    return 0
  fi
  
  log_info "Revision mismatch: current=$current, expected=$expected_sha"
  return 1
}

# Wait for specific revision to be deployed
# Usage: argocd_wait_revision APP_NAME EXPECTED_SHA [timeout_seconds]
argocd_wait_revision() {
  local app_name="$1"
  local expected_sha="$2"
  local timeout="${3:-600}"
  
  log_step "Waiting for revision $expected_sha to be deployed"
  
  _check_revision() {
    argocd_has_revision "$app_name" "$expected_sha" && argocd_is_healthy "$app_name"
  }
  
  poll_until "$timeout" 15 "revision $expected_sha deployed" _check_revision
}

# Get detailed sync/health info for debugging
# Usage: argocd_debug_status APP_NAME
argocd_debug_status() {
  local app_name="$1"
  
  local status
  status=$(argocd app get "$app_name" -o json 2>/dev/null)
  
  echo "=== ArgoCD App Status: $app_name ==="
  echo "$status" | jq '{
    sync: .status.sync,
    health: .status.health,
    conditions: .status.conditions,
    resources: [.status.resources[] | select(.health.status != "Healthy") | {name, kind, health}]
  }'
}

