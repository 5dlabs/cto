#!/bin/bash
# End-to-End Test for Linear Input Routing
# Tests the two-way communication flow: Linear → PM Server → Sidecar → Agent
#
# Prerequisites:
# - kubectl configured for the cluster
# - A running CodeRun with Linear integration
# - PM server running

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
NC='\033[0m'

NAMESPACE="${NAMESPACE:-cto}"
PM_SERVICE="${PM_SERVICE:-pm-server}"
PM_PORT="${PM_PORT:-8080}"

echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}║       Linear Input Routing E2E Test                         ║${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# =============================================================================
# Helper Functions
# =============================================================================

log_step() {
    echo -e "\n${BLUE}▶ $1${NC}"
}

log_success() {
    echo -e "  ${GREEN}✓ $1${NC}"
}

log_warning() {
    echo -e "  ${YELLOW}⚠ $1${NC}"
}

log_error() {
    echo -e "  ${RED}✗ $1${NC}"
}

log_info() {
    echo -e "  ${CYAN}ℹ $1${NC}"
}

# =============================================================================
# Test 1: Verify CodeRun pods have the expected labels
# =============================================================================

test_pod_labels() {
    log_step "Test 1: Verify CodeRun pods have Linear session labels"
    
    # Find pods with linear-session label
    local pods=$(kubectl get pods -n "$NAMESPACE" -l linear-session -o json 2>/dev/null)
    local pod_count=$(echo "$pods" | jq '.items | length')
    
    if [[ "$pod_count" -eq 0 ]]; then
        log_warning "No pods found with linear-session label"
        log_info "Creating a test CodeRun is needed to run the full test"
        
        # List all running CodeRuns to help debugging
        echo -e "\n  Current CodeRuns in namespace '$NAMESPACE':"
        kubectl get coderuns -n "$NAMESPACE" 2>/dev/null || echo "  None found"
        return 1
    fi
    
    log_success "Found $pod_count pod(s) with linear-session label"
    
    # Check for expected labels on each pod
    echo "$pods" | jq -r '.items[] | "\(.metadata.name): linear-session=\(.metadata.labels["linear-session"]) linear-issue=\(.metadata.labels["cto.5dlabs.io/linear-issue"] // "N/A") agent-type=\(.metadata.labels["cto.5dlabs.io/agent-type"] // "N/A")"' | while read -r line; do
        log_info "$line"
    done
    
    return 0
}

# =============================================================================
# Test 2: Verify PM server is running and accessible
# =============================================================================

test_pm_server() {
    log_step "Test 2: Verify PM server is running and accessible"
    
    # Check if PM server pod is running
    local pm_pod=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=pm-server -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [[ -z "$pm_pod" ]]; then
        log_error "PM server pod not found in namespace '$NAMESPACE'"
        return 1
    fi
    
    log_success "PM server pod found: $pm_pod"
    
    # Check health endpoint via port-forward (in background)
    kubectl port-forward -n "$NAMESPACE" "svc/$PM_SERVICE" "$PM_PORT:$PM_PORT" &>/dev/null &
    local pf_pid=$!
    sleep 2
    
    local health_response=$(curl -s "http://localhost:$PM_PORT/health" 2>/dev/null || echo "")
    
    kill $pf_pid 2>/dev/null || true
    
    if [[ -z "$health_response" ]]; then
        log_error "PM server health check failed"
        return 1
    fi
    
    log_success "PM server health check passed: $health_response"
    return 0
}

# =============================================================================
# Test 3: Test input routing to a running agent
# =============================================================================

test_input_routing() {
    log_step "Test 3: Test input routing to running agent"
    
    # Find a running pod with linear-session label
    local pod_info=$(kubectl get pods -n "$NAMESPACE" -l linear-session --field-selector=status.phase=Running -o json 2>/dev/null | jq -r '.items[0] // empty')
    
    if [[ -z "$pod_info" ]]; then
        log_warning "No running pods with linear-session label found"
        return 1
    fi
    
    local pod_name=$(echo "$pod_info" | jq -r '.metadata.name')
    local session_id=$(echo "$pod_info" | jq -r '.metadata.labels["linear-session"]')
    local pod_ip=$(echo "$pod_info" | jq -r '.status.podIP')
    
    log_info "Testing with pod: $pod_name"
    log_info "Session ID: $session_id"
    log_info "Pod IP: $pod_ip"
    
    # Test 3a: Test via PM server API
    log_info "Testing via PM server /api/sessions/{session}/input endpoint..."
    
    kubectl port-forward -n "$NAMESPACE" "svc/$PM_SERVICE" "$PM_PORT:$PM_PORT" &>/dev/null &
    local pf_pid=$!
    sleep 2
    
    local test_message="[E2E Test] Testing input routing at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    local response=$(curl -s -X POST "http://localhost:$PM_PORT/api/sessions/$session_id/input" \
        -H "Content-Type: application/json" \
        -d "{\"text\": \"$test_message\"}" 2>/dev/null || echo '{"status":"error","error":"curl failed"}')
    
    kill $pf_pid 2>/dev/null || true
    
    local status=$(echo "$response" | jq -r '.status // "unknown"')
    if [[ "$status" == "ok" ]]; then
        local agents_notified=$(echo "$response" | jq -r '.agents_notified // 0')
        log_success "PM server routed message to $agents_notified agent(s)"
    else
        log_error "PM server routing failed: $response"
    fi
    
    # Test 3b: Test direct HTTP to sidecar (if pod IP available)
    if [[ "$pod_ip" != "null" && -n "$pod_ip" ]]; then
        log_info "Testing direct HTTP to sidecar at $pod_ip:8080..."
        
        # Use kubectl exec to curl the sidecar from within the cluster
        local sidecar_response=$(kubectl exec -n "$NAMESPACE" "$pod_name" -c linear-sidecar -- \
            curl -s -X POST "http://localhost:8080/input" \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"Direct sidecar test\"}" 2>/dev/null || echo "")
        
        if [[ -n "$sidecar_response" ]]; then
            log_success "Sidecar responded: $sidecar_response"
        else
            log_warning "Could not reach sidecar directly (may not have curl)"
        fi
    fi
    
    return 0
}

# =============================================================================
# Test 4: Verify sidecar container is running and has correct environment
# =============================================================================

test_sidecar_config() {
    log_step "Test 4: Verify sidecar container configuration"
    
    local pod_info=$(kubectl get pods -n "$NAMESPACE" -l linear-session --field-selector=status.phase=Running -o json 2>/dev/null | jq -r '.items[0] // empty')
    
    if [[ -z "$pod_info" ]]; then
        log_warning "No running pods with linear-session label found"
        return 1
    fi
    
    local pod_name=$(echo "$pod_info" | jq -r '.metadata.name')
    
    # Check sidecar container exists
    local sidecar_exists=$(kubectl get pod -n "$NAMESPACE" "$pod_name" -o jsonpath='{.spec.containers[?(@.name=="linear-sidecar")].name}' 2>/dev/null)
    
    if [[ -z "$sidecar_exists" ]]; then
        log_error "linear-sidecar container not found in pod $pod_name"
        return 1
    fi
    
    log_success "linear-sidecar container found in pod"
    
    # Check sidecar environment variables
    local sidecar_env=$(kubectl get pod -n "$NAMESPACE" "$pod_name" -o jsonpath='{.spec.containers[?(@.name=="linear-sidecar")].env}' 2>/dev/null)
    
    # Verify critical env vars
    local env_vars=("LINEAR_SESSION_ID" "HTTP_PORT" "INPUT_FIFO_PATH")
    for var in "${env_vars[@]}"; do
        if echo "$sidecar_env" | grep -q "$var"; then
            log_success "Environment variable $var is set"
        else
            log_warning "Environment variable $var not found"
        fi
    done
    
    # Check sidecar logs for startup
    log_info "Checking sidecar logs..."
    local sidecar_logs=$(kubectl logs -n "$NAMESPACE" "$pod_name" -c linear-sidecar --tail=10 2>/dev/null || echo "")
    if [[ -n "$sidecar_logs" ]]; then
        echo "$sidecar_logs" | head -5 | while read -r line; do
            log_info "Log: $line"
        done
    fi
    
    return 0
}

# =============================================================================
# Main
# =============================================================================

main() {
    local failed=0
    
    # Run tests
    test_pod_labels || failed=$((failed + 1))
    test_pm_server || failed=$((failed + 1))
    test_sidecar_config || failed=$((failed + 1))
    test_input_routing || failed=$((failed + 1))
    
    # Summary
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    if [[ $failed -eq 0 ]]; then
        echo -e "${GREEN}All tests passed! ✓${NC}"
    else
        echo -e "${YELLOW}$failed test(s) had warnings or failures${NC}"
    fi
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    
    return $failed
}

# Run if not sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi

