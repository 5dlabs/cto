#!/bin/bash
# submit-play.sh - Submit a Play workflow and return the workflow name
# Usage: submit-play.sh <task-id> <repository> [template]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/../lib/common.sh"

TASK_ID="${1:-}"
REPOSITORY="${2:-}"
TEMPLATE="${3:-play-workflow-template}"
NAMESPACE="${ARGO_NAMESPACE:-argo}"

if [ -z "$TASK_ID" ] || [ -z "$REPOSITORY" ]; then
    log_error "Usage: submit-play.sh <task-id> <repository> [template]"
    exit 1
fi

log_info "Submitting Play workflow for task $TASK_ID"
log_info "Repository: $REPOSITORY"
log_info "Template: $TEMPLATE"

# Submit the workflow
OUTPUT=$(argo submit \
    --from "workflowtemplate/$TEMPLATE" \
    -n "$NAMESPACE" \
    -p "task-id=$TASK_ID" \
    -p "repository=$REPOSITORY" \
    -o json 2>&1) || {
    log_error "Failed to submit workflow: $OUTPUT"
    exit 1
}

# Extract workflow name
WORKFLOW_NAME=$(echo "$OUTPUT" | jq -r '.metadata.name // empty')

if [ -z "$WORKFLOW_NAME" ]; then
    log_error "Failed to get workflow name from submission response"
    echo "$OUTPUT"
    exit 1
fi

log_success "Workflow submitted: $WORKFLOW_NAME"
echo "$WORKFLOW_NAME"

