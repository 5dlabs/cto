#!/bin/bash
# poll-workflow.sh - Poll an Argo workflow until completion
# Usage: poll-workflow.sh <workflow-name> [interval] [timeout]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/../lib/common.sh"

WORKFLOW_NAME="${1:-}"
INTERVAL="${2:-30}"
TIMEOUT="${3:-0}"  # 0 = no timeout (infinite)
NAMESPACE="${ARGO_NAMESPACE:-argo}"

if [ -z "$WORKFLOW_NAME" ]; then
    log_error "Usage: poll-workflow.sh <workflow-name> [interval] [timeout]"
    exit 1
fi

log_info "Polling workflow: $WORKFLOW_NAME"
log_info "Interval: ${INTERVAL}s, Timeout: ${TIMEOUT}s (0=infinite)"

START_TIME=$(date +%s)
LAST_PHASE=""

while true; do
    # Get workflow status
    STATUS=$(argo get "$WORKFLOW_NAME" -n "$NAMESPACE" -o json 2>&1) || {
        log_warn "Failed to get workflow status, retrying..."
        sleep "$INTERVAL"
        continue
    }

    PHASE=$(echo "$STATUS" | jq -r '.status.phase // "Unknown"')
    MESSAGE=$(echo "$STATUS" | jq -r '.status.message // ""')

    # Log phase changes
    if [ "$PHASE" != "$LAST_PHASE" ]; then
        log_info "Workflow phase: $PHASE"
        if [ -n "$MESSAGE" ]; then
            log_info "Message: $MESSAGE"
        fi
        LAST_PHASE="$PHASE"
    fi

    # Check for terminal states
    case "$PHASE" in
        "Succeeded")
            log_success "Workflow completed successfully"
            echo "SUCCEEDED"
            exit 0
            ;;
        "Failed"|"Error")
            log_error "Workflow failed: $MESSAGE"
            echo "FAILED"
            exit 1
            ;;
        "Running"|"Pending")
            # Still in progress
            ;;
        *)
            log_warn "Unknown workflow phase: $PHASE"
            ;;
    esac

    # Check timeout
    if [ "$TIMEOUT" -gt 0 ]; then
        ELAPSED=$(($(date +%s) - START_TIME))
        if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
            log_error "Timeout waiting for workflow completion"
            echo "TIMEOUT"
            exit 1
        fi
    fi

    sleep "$INTERVAL"
done

