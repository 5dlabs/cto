#!/bin/bash
# start-watch.sh - Start the E2E Watch loop by creating the initial Monitor CodeRun
# Usage: start-watch.sh <task-id> <repository> [service]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/../lib/common.sh" 2>/dev/null || {
    # Fallback logging if common.sh not available
    log_info() { echo "[INFO] $*"; }
    log_success() { echo "[SUCCESS] $*"; }
    log_error() { echo "[ERROR] $*" >&2; }
}

TASK_ID="${1:-}"
REPOSITORY="${2:-}"
SERVICE="${3:-e2e-watch}"
NAMESPACE="${AGENT_NAMESPACE:-agent-platform}"
MONITOR_AGENT="${MONITOR_AGENT:-5DLabs-Morgan}"
MONITOR_MODEL="${MONITOR_MODEL:-glm-4-plus}"

if [ -z "$TASK_ID" ] || [ -z "$REPOSITORY" ]; then
    log_error "Usage: start-watch.sh <task-id> <repository> [service]"
    log_error "Example: start-watch.sh task-123 5dlabs/cto"
    exit 1
fi

# Generate unique name with timestamp
TIMESTAMP=$(date +%s)
CODERUN_NAME="watch-monitor-${TASK_ID}-${TIMESTAMP}"

log_info "Starting E2E Watch loop"
log_info "Task ID: $TASK_ID"
log_info "Repository: $REPOSITORY"
log_info "Service: $SERVICE"
log_info "Monitor Agent: $MONITOR_AGENT"
log_info "Monitor Model: $MONITOR_MODEL"

# Create the Monitor CodeRun
# Use || to handle failure gracefully with set -e
if ! cat <<EOF | kubectl apply -f -
apiVersion: agents.platform/v1alpha1
kind: CodeRun
metadata:
  name: ${CODERUN_NAME}
  namespace: ${NAMESPACE}
  labels:
    task-id: "${TASK_ID}"
    watch-role: monitor
    watch-iteration: "1"
spec:
  taskId: "${TASK_ID}"
  service: "${SERVICE}"
  repositoryUrl: "https://github.com/${REPOSITORY}"
  githubApp: "${MONITOR_AGENT}"
  model: "${MONITOR_MODEL}"
  cliConfig:
    iteration: 1
    watchMode: true
EOF
then
    log_error "Failed to create Monitor CodeRun"
    exit 1
fi

log_success "Created Monitor CodeRun: $CODERUN_NAME"
log_info "Watch loop started. Monitor agent will:"
log_info "  1. Submit Play workflow"
log_info "  2. Evaluate results against acceptance criteria"
log_info "  3. If issues found → Create Remediation CodeRun"
log_info "  4. Loop continues until success"
log_info ""
log_info "To follow progress:"
log_info "  kubectl logs -f -l task-id=${TASK_ID} -n ${NAMESPACE}"
log_info "  kubectl get coderuns -l task-id=${TASK_ID} -n ${NAMESPACE} -w"

echo "$CODERUN_NAME"

