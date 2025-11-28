#!/bin/bash
# Kubernetes operations for E2E Watch Remediation Agent
# Requires: kubectl (uses in-cluster auth)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# ============================================================================
# Pod Operations
# ============================================================================

# Get pods by label selector
# Usage: k8s_get_pods NAMESPACE LABEL_SELECTOR
k8s_get_pods() {
  local namespace="$1"
  local selector="$2"
  
  kubectl get pods -n "$namespace" -l "$selector" -o json
}

# Check if all pods matching selector are ready
# Usage: k8s_pods_ready NAMESPACE LABEL_SELECTOR
k8s_pods_ready() {
  local namespace="$1"
  local selector="$2"
  
  local pods
  pods=$(k8s_get_pods "$namespace" "$selector")
  
  local total ready
  total=$(echo "$pods" | jq '.items | length')
  ready=$(echo "$pods" | jq '[.items[] | select(.status.conditions[] | select(.type=="Ready" and .status=="True"))] | length')
  
  if [ "$total" = "0" ]; then
    log_warn "No pods found matching: $selector"
    return 1
  fi
  
  if [ "$ready" = "$total" ]; then
    log_info "All $ready/$total pods ready"
    return 0
  fi
  
  log_info "Pods ready: $ready/$total"
  return 1
}

# Wait for pods to be ready
# Usage: k8s_wait_pods_ready NAMESPACE LABEL_SELECTOR [timeout_seconds]
k8s_wait_pods_ready() {
  local namespace="$1"
  local selector="$2"
  local timeout="${3:-300}"  # 5 min default
  
  log_step "Waiting for pods to be ready: $selector in $namespace"
  
  poll_until "$timeout" 10 "pods ready ($selector)" k8s_pods_ready "$namespace" "$selector"
}

# Get pod names
# Usage: k8s_pod_names NAMESPACE LABEL_SELECTOR
k8s_pod_names() {
  local namespace="$1"
  local selector="$2"
  
  kubectl get pods -n "$namespace" -l "$selector" -o jsonpath='{.items[*].metadata.name}'
}

# Get pod image
# Usage: k8s_pod_image NAMESPACE POD_NAME [CONTAINER_NAME]
k8s_pod_image() {
  local namespace="$1"
  local pod_name="$2"
  local container="${3:-}"
  
  if [ -n "$container" ]; then
    kubectl get pod -n "$namespace" "$pod_name" \
      -o jsonpath="{.spec.containers[?(@.name=='$container')].image}"
  else
    kubectl get pod -n "$namespace" "$pod_name" \
      -o jsonpath='{.spec.containers[0].image}'
  fi
}

# Check if pods have restarted recently (indicating new deployment)
# Usage: k8s_pods_restarted_after NAMESPACE LABEL_SELECTOR TIMESTAMP
k8s_pods_restarted_after() {
  local namespace="$1"
  local selector="$2"
  local after_ts="$3"  # Unix timestamp
  
  local pods
  pods=$(k8s_get_pods "$namespace" "$selector")
  
  # Get the most recent container start time
  local latest_start
  latest_start=$(echo "$pods" | jq -r '
    [.items[].status.containerStatuses[].state.running.startedAt // empty] |
    sort | last // "1970-01-01T00:00:00Z"
  ')
  
  # Convert to unix timestamp
  local start_ts
  start_ts=$(date -d "$latest_start" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$latest_start" +%s 2>/dev/null || echo 0)
  
  if [ "$start_ts" -gt "$after_ts" ]; then
    log_info "Pods restarted at $latest_start (after $after_ts)"
    return 0
  fi
  
  return 1
}

# ============================================================================
# Deployment Operations
# ============================================================================

# Get deployment status
# Usage: k8s_deployment_status NAMESPACE DEPLOYMENT_NAME
k8s_deployment_status() {
  local namespace="$1"
  local deployment="$2"
  
  kubectl get deployment -n "$namespace" "$deployment" -o json
}

# Check if deployment rollout is complete
# Usage: k8s_deployment_ready NAMESPACE DEPLOYMENT_NAME
k8s_deployment_ready() {
  local namespace="$1"
  local deployment="$2"
  
  kubectl rollout status deployment/"$deployment" -n "$namespace" --timeout=1s 2>/dev/null
}

# Wait for deployment rollout
# Usage: k8s_wait_deployment NAMESPACE DEPLOYMENT_NAME [timeout]
k8s_wait_deployment() {
  local namespace="$1"
  local deployment="$2"
  local timeout="${3:-300}"
  
  log_step "Waiting for deployment rollout: $deployment"
  
  kubectl rollout status deployment/"$deployment" -n "$namespace" --timeout="${timeout}s"
}

# ============================================================================
# Debugging
# ============================================================================

# Get recent events for a namespace/selector
# Usage: k8s_recent_events NAMESPACE [LABEL_SELECTOR]
k8s_recent_events() {
  local namespace="$1"
  local selector="${2:-}"
  
  if [ -n "$selector" ]; then
    local pods
    pods=$(k8s_pod_names "$namespace" "$selector")
    
    for pod in $pods; do
      echo "=== Events for pod: $pod ==="
      kubectl get events -n "$namespace" --field-selector "involvedObject.name=$pod" --sort-by='.lastTimestamp' | tail -10
    done
  else
    kubectl get events -n "$namespace" --sort-by='.lastTimestamp' | tail -20
  fi
}

# Get pod logs
# Usage: k8s_pod_logs NAMESPACE LABEL_SELECTOR [LINES]
k8s_pod_logs() {
  local namespace="$1"
  local selector="$2"
  local lines="${3:-100}"
  
  local pods
  pods=$(k8s_pod_names "$namespace" "$selector")
  
  for pod in $pods; do
    echo "=== Logs for pod: $pod ==="
    kubectl logs -n "$namespace" "$pod" --tail="$lines" 2>/dev/null || echo "(no logs)"
  done
}

# Describe pods for debugging
# Usage: k8s_describe_pods NAMESPACE LABEL_SELECTOR
k8s_describe_pods() {
  local namespace="$1"
  local selector="$2"
  
  kubectl describe pods -n "$namespace" -l "$selector"
}

