#!/usr/bin/env bash
# Event-driven CodeRun failure watcher
# Immediately detects pod failures and triggers cleanup

set -euo pipefail

NAMESPACE="${1:-cto}"
POLL_INTERVAL=5  # Check every 5 seconds

log() { echo "[$(date '+%H:%M:%S')] $*"; }
log_error() { echo "[$(date '+%H:%M:%S')] ❌ $*" >&2; }
log_success() { echo "[$(date '+%H:%M:%S')] ✅ $*"; }

cleanup_failed_coderuns() {
    log "🧹 Cleaning up failed CodeRuns..."
    kubectl delete coderuns -n "$NAMESPACE" --field-selector=status.phase=Failed --wait=false 2>/dev/null || true
    kubectl delete pods -n "$NAMESPACE" --field-selector=status.phase=Failed --wait=false 2>/dev/null || true
}

check_for_failures() {
    # Check for Error pods
    local error_pods=$(kubectl get pods -n "$NAMESPACE" -o jsonpath='{.items[?(@.status.phase=="Failed")].metadata.name}' 2>/dev/null || echo "")
    
    # Check for Failed CodeRuns
    local failed_coderuns=$(kubectl get coderuns -n "$NAMESPACE" -o jsonpath='{.items[?(@.status.phase=="Failed")].metadata.name}' 2>/dev/null || echo "")
    
    # Check for Error state in pods
    local error_state=$(kubectl get pods -n "$NAMESPACE" -o jsonpath='{range .items[*]}{.metadata.name}{" "}{.status.containerStatuses[*].state.waiting.reason}{"\n"}{end}' 2>/dev/null | grep -E "Error|CrashLoop|ImagePull" || echo "")
    
    if [[ -n "$error_pods" ]] || [[ -n "$failed_coderuns" ]] || [[ -n "$error_state" ]]; then
        log_error "FAILURE DETECTED!"
        [[ -n "$error_pods" ]] && log_error "  Error pods: $error_pods"
        [[ -n "$failed_coderuns" ]] && log_error "  Failed CodeRuns: $failed_coderuns"
        [[ -n "$error_state" ]] && log_error "  Error state: $error_state"
        
        # Get pod logs for failed pods
        for pod in $error_pods; do
            log "📋 Logs from $pod:"
            kubectl logs -n "$NAMESPACE" "$pod" --tail=20 2>/dev/null || true
        done
        
        return 1  # Failure detected
    fi
    
    return 0  # No failures
}

check_intake_progress() {
    # Check for running intake CodeRuns
    local intake_cr=$(kubectl get coderuns -n "$NAMESPACE" -l type=intake -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    
    if [[ -n "$intake_cr" ]]; then
        local phase=$(kubectl get coderun -n "$NAMESPACE" "$intake_cr" -o jsonpath='{.status.phase}' 2>/dev/null || echo "unknown")
        log "📊 Intake CodeRun: $intake_cr - Phase: $phase"
        
        if [[ "$phase" == "Succeeded" ]]; then
            log_success "Intake completed successfully!"
            return 0
        elif [[ "$phase" == "Failed" ]]; then
            log_error "Intake FAILED!"
            # Get pod logs
            local pod=$(kubectl get pods -n "$NAMESPACE" -l coderun="$intake_cr" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
            if [[ -n "$pod" ]]; then
                log "📋 Intake pod logs:"
                kubectl logs -n "$NAMESPACE" "$pod" --tail=30 2>/dev/null || true
            fi
            return 1
        fi
    fi
    
    return 2  # No intake running
}

# Main loop
log "🔍 Starting CodeRun failure watcher for namespace: $NAMESPACE"
log "   Poll interval: ${POLL_INTERVAL}s"

while true; do
    # Check for failures
    if ! check_for_failures; then
        cleanup_failed_coderuns
        log "⏳ Waiting for retry..."
    fi
    
    # Check intake progress
    intake_status=$?
    check_intake_progress || intake_status=$?
    
    if [[ $intake_status -eq 0 ]]; then
        log_success "Intake succeeded! Watcher complete."
        exit 0
    elif [[ $intake_status -eq 1 ]]; then
        cleanup_failed_coderuns
    fi
    
    sleep "$POLL_INTERVAL"
done
