#!/bin/bash
# download-logs.sh - Download all logs from a workflow
# Usage: download-logs.sh <workflow-name> [output-dir]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/../lib/common.sh"

WORKFLOW_NAME="${1:-}"
OUTPUT_DIR="${2:-/workspace/watch/logs}"
NAMESPACE="${ARGO_NAMESPACE:-argo}"
TAIL_LINES="${LOG_TAIL_LINES:-1000}"

if [ -z "$WORKFLOW_NAME" ]; then
    log_error "Usage: download-logs.sh <workflow-name> [output-dir]"
    exit 1
fi

log_info "Downloading logs for workflow: $WORKFLOW_NAME"
log_info "Output directory: $OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR"

# Get workflow details
WORKFLOW_JSON=$(argo get "$WORKFLOW_NAME" -n "$NAMESPACE" -o json 2>&1) || {
    log_error "Failed to get workflow details"
    exit 1
}

# Get all nodes (steps) from the workflow
NODES=$(echo "$WORKFLOW_JSON" | jq -r '.status.nodes // {} | to_entries[] | select(.value.type == "Pod") | .key')

if [ -z "$NODES" ]; then
    log_warn "No pod nodes found in workflow"
    # Still try to get overall workflow logs
    argo logs "$WORKFLOW_NAME" -n "$NAMESPACE" --tail "$TAIL_LINES" > "$OUTPUT_DIR/workflow.log" 2>&1 || true
    exit 0
fi

# Download logs for each node
for NODE_ID in $NODES; do
    NODE_INFO=$(echo "$WORKFLOW_JSON" | jq -r ".status.nodes[\"$NODE_ID\"]")
    NODE_NAME=$(echo "$NODE_INFO" | jq -r '.displayName // .name // "unknown"')
    NODE_PHASE=$(echo "$NODE_INFO" | jq -r '.phase // "Unknown"')
    
    # Sanitize node name for filename
    SAFE_NAME=$(echo "$NODE_NAME" | tr '/' '-' | tr ' ' '_')
    LOG_FILE="$OUTPUT_DIR/${SAFE_NAME}.log"
    
    log_info "Downloading logs for: $NODE_NAME ($NODE_PHASE)"
    
    # Try argo logs first (works for completed pods)
    if argo logs "$WORKFLOW_NAME" -n "$NAMESPACE" --node-name "$NODE_NAME" --tail "$TAIL_LINES" > "$LOG_FILE" 2>&1; then
        log_success "Downloaded: $LOG_FILE"
    else
        # Fallback to kubectl if argo logs fails
        POD_NAME=$(echo "$NODE_INFO" | jq -r '.id // empty')
        if [ -n "$POD_NAME" ]; then
            kubectl logs "$POD_NAME" -n "$NAMESPACE" --tail="$TAIL_LINES" --all-containers=true > "$LOG_FILE" 2>&1 || {
                log_warn "Failed to get logs for $NODE_NAME"
                echo "No logs available for $NODE_NAME" > "$LOG_FILE"
            }
        fi
    fi
    
    # Add metadata header to log file
    {
        echo "# Node: $NODE_NAME"
        echo "# Phase: $NODE_PHASE"
        echo "# Workflow: $WORKFLOW_NAME"
        echo "# Downloaded: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
        echo "---"
        cat "$LOG_FILE"
    } > "$LOG_FILE.tmp" && mv "$LOG_FILE.tmp" "$LOG_FILE"
done

# Create summary file
SUMMARY_FILE="$OUTPUT_DIR/summary.md"
{
    echo "# Workflow Logs Summary"
    echo ""
    echo "**Workflow:** $WORKFLOW_NAME"
    echo "**Downloaded:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo ""
    echo "## Stages"
    echo ""
    for NODE_ID in $NODES; do
        NODE_INFO=$(echo "$WORKFLOW_JSON" | jq -r ".status.nodes[\"$NODE_ID\"]")
        NODE_NAME=$(echo "$NODE_INFO" | jq -r '.displayName // .name // "unknown"')
        NODE_PHASE=$(echo "$NODE_INFO" | jq -r '.phase // "Unknown"')
        SAFE_NAME=$(echo "$NODE_NAME" | tr '/' '-' | tr ' ' '_')
        
        case "$NODE_PHASE" in
            "Succeeded") ICON="✅" ;;
            "Failed"|"Error") ICON="❌" ;;
            "Running") ICON="🔄" ;;
            *) ICON="⚪" ;;
        esac
        
        echo "- $ICON **$NODE_NAME**: $NODE_PHASE (see \`${SAFE_NAME}.log\`)"
    done
} > "$SUMMARY_FILE"

log_success "Logs downloaded to $OUTPUT_DIR"
log_info "Summary: $SUMMARY_FILE"

