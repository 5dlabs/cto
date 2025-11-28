#!/bin/bash
# evaluate-criteria.sh - Evaluate workflow results against acceptance criteria
# Usage: evaluate-criteria.sh <workflow-name> [criteria-file]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/../lib/common.sh"

WORKFLOW_NAME="${1:-}"
CRITERIA_FILE="${2:-/workspace/watch/acceptance-criteria.md}"
LOGS_DIR="${3:-/workspace/watch/logs}"
OUTPUT_FILE="${4:-/workspace/watch/evaluation-result.md}"
NAMESPACE="${ARGO_NAMESPACE:-argo}"

if [ -z "$WORKFLOW_NAME" ]; then
    log_error "Usage: evaluate-criteria.sh <workflow-name> [criteria-file] [logs-dir] [output-file]"
    exit 1
fi

log_info "Evaluating workflow: $WORKFLOW_NAME"
log_info "Criteria file: $CRITERIA_FILE"

# Initialize evaluation result
PASSED=true
ISSUES=()

# Get workflow status
WORKFLOW_JSON=$(argo get "$WORKFLOW_NAME" -n "$NAMESPACE" -o json 2>&1) || {
    log_error "Failed to get workflow details"
    PASSED=false
    ISSUES+=("Failed to retrieve workflow status")
}

WORKFLOW_PHASE=$(echo "$WORKFLOW_JSON" | jq -r '.status.phase // "Unknown"')
WORKFLOW_MESSAGE=$(echo "$WORKFLOW_JSON" | jq -r '.status.message // ""')

log_info "Workflow phase: $WORKFLOW_PHASE"

# Criterion 1: Workflow completion
if [ "$WORKFLOW_PHASE" != "Succeeded" ]; then
    PASSED=false
    ISSUES+=("Workflow did not succeed (phase: $WORKFLOW_PHASE, message: $WORKFLOW_MESSAGE)")
fi

# Criterion 2: Check for failed nodes
FAILED_NODES=$(echo "$WORKFLOW_JSON" | jq -r '.status.nodes // {} | to_entries[] | select(.value.phase == "Failed" or .value.phase == "Error") | .value.displayName // .value.name')
if [ -n "$FAILED_NODES" ]; then
    PASSED=false
    while IFS= read -r node; do
        ISSUES+=("Stage failed: $node")
    done <<< "$FAILED_NODES"
fi

# Criterion 3: Check logs for critical errors
if [ -d "$LOGS_DIR" ]; then
    # Look for critical error patterns in logs
    ERROR_PATTERNS=(
        "error\[E"           # Rust compilation errors
        "FAILED"             # Test failures
        "panicked at"        # Rust panics
        "fatal:"             # Git fatal errors
        "CRITICAL"           # Critical log level
        "OOMKilled"          # Out of memory
        "CrashLoopBackOff"   # Pod crash loop
    )
    
    for pattern in "${ERROR_PATTERNS[@]}"; do
        MATCHES=$(grep -r -l "$pattern" "$LOGS_DIR" 2>/dev/null || true)
        if [ -n "$MATCHES" ]; then
            for match in $MATCHES; do
                # Get context around the error
                CONTEXT=$(grep -B2 -A2 "$pattern" "$match" 2>/dev/null | head -20 || true)
                if [ -n "$CONTEXT" ]; then
                    PASSED=false
                    ISSUES+=("Error pattern '$pattern' found in $(basename "$match")")
                fi
            done
        fi
    done
fi

# Criterion 4: Check if PR was created and merged (for Play workflows)
if command -v gh >/dev/null 2>&1; then
    TASK_ID=$(echo "$WORKFLOW_NAME" | sed -E 's/play-([^-]+).*/\1/' || echo "")
    if [ -n "$TASK_ID" ]; then
        PR_STATE=$(gh pr list -l "task-$TASK_ID" --json state --jq '.[0].state // "none"' 2>/dev/null || echo "unknown")
        if [ "$PR_STATE" = "MERGED" ]; then
            log_success "PR for task $TASK_ID is merged"
        elif [ "$PR_STATE" = "OPEN" ]; then
            log_warn "PR for task $TASK_ID is still open (not merged)"
            # This might be OK depending on the workflow stage
        elif [ "$PR_STATE" = "none" ] || [ "$PR_STATE" = "unknown" ]; then
            log_warn "Could not determine PR status for task $TASK_ID"
        fi
    fi
fi

# Generate evaluation report
{
    echo "# Evaluation Report"
    echo ""
    echo "**Workflow:** $WORKFLOW_NAME"
    echo "**Evaluated:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "**Phase:** $WORKFLOW_PHASE"
    echo ""
    
    if [ "$PASSED" = true ]; then
        echo "## Result: ✅ PASSED"
        echo ""
        echo "All acceptance criteria met."
    else
        echo "## Result: ❌ FAILED"
        echo ""
        echo "### Issues Found"
        echo ""
        for issue in "${ISSUES[@]}"; do
            echo "- $issue"
        done
    fi
    
    echo ""
    echo "---"
    echo ""
    echo "## Criteria Evaluated"
    echo ""
    echo "1. Workflow completion status"
    echo "2. No failed stages/nodes"
    echo "3. No critical errors in logs"
    echo "4. PR status (if applicable)"
    
} > "$OUTPUT_FILE"

log_info "Evaluation report written to: $OUTPUT_FILE"

if [ "$PASSED" = true ]; then
    log_success "All acceptance criteria PASSED"
    exit 0
else
    log_error "Acceptance criteria FAILED"
    log_error "Issues found: ${#ISSUES[@]}"
    for issue in "${ISSUES[@]}"; do
        log_error "  - $issue"
    done
    exit 1
fi

