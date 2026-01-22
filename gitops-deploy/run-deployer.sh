#!/bin/bash
# GitOps Deployer Agent - runs Claude to sync ArgoCD applications
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/deployer-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date -u +%Y-%m-%dT%H:%M:%SZ)]${NC} $*"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }

# Set kubeconfig explicitly for this cluster
export KUBECONFIG="/tmp/latitude-test/kubeconfig"

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v claude &> /dev/null; then
        echo "Error: claude CLI not found"
        exit 1
    fi
    
    if ! command -v kubectl &> /dev/null; then
        echo "Error: kubectl not found"
        exit 1
    fi
    
    if [[ ! -f "$KUBECONFIG" ]]; then
        echo "Error: KUBECONFIG not found at $KUBECONFIG"
        exit 1
    fi
    
    log "Using KUBECONFIG: $KUBECONFIG"
    
    # Verify cluster access
    if ! kubectl get nodes &>/dev/null; then
        echo "Error: Cannot access Kubernetes cluster"
        exit 1
    fi
    
    # Verify ArgoCD
    if ! kubectl get pods -n argocd &>/dev/null; then
        echo "Error: ArgoCD namespace not found"
        exit 1
    fi
    
    success "All prerequisites met"
}

# Update coordination file
update_coord() {
    local status="$1"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    # Use jq to update the coordination file
    if command -v jq &> /dev/null; then
        local tmp
        tmp=$(mktemp)
        jq --arg status "$status" \
           --arg ts "$timestamp" \
           --arg pid "$$" \
           '.deployer.status = $status | 
            .deployer.lastUpdate = $ts | 
            .deployer.pid = ($pid | tonumber) |
            .session.lastActivity = $ts' \
           "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
    fi
}

# Log to progress file
log_progress() {
    local msg="$1"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    echo "[$timestamp] $msg" >> "$PROGRESS_FILE"
}

# Main
main() {
    log "=== GitOps Deployer Agent (Claude) ==="
    log "Repo root: $REPO_ROOT"
    log "Prompt file: $PROMPT_FILE"
    
    check_prerequisites
    
    # Generate session ID
    SESSION_ID=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "session-$$")
    
    # Update coordination
    if command -v jq &> /dev/null; then
        local tmp
        tmp=$(mktemp)
        jq --arg id "$SESSION_ID" \
           --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
           '.session.id = $id | .session.startedAt = $ts' \
           "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
    fi
    
    update_coord "running"
    log_progress "=== DEPLOYER AGENT STARTING ==="
    log_progress "Session ID: $SESSION_ID"
    
    # Read the prompt
    PROMPT_CONTENT=$(cat "$PROMPT_FILE")
    
    # Build the full prompt with current state
    APPS_STATUS=$(kubectl get applications -n argocd --no-headers 2>/dev/null | head -20 || echo "No applications yet")
    
    FULL_PROMPT="$PROMPT_CONTENT

---

## Current State

### ArgoCD Applications
\`\`\`
$APPS_STATUS
\`\`\`

### Coordination File
\`\`\`json
$(cat "$COORD_FILE")
\`\`\`

### Your Task

1. Check if platform-project and app-of-apps are applied
2. Wait for applications to be created
3. Sync applications in dependency order
4. Report progress to progress.txt
5. Update ralph-coordination.json with status

Start by checking the current state of ArgoCD applications.
"

    log "Starting Claude..."
    warn "Using --dangerously-skip-permissions - full access mode"
    
    # Run Claude
    cd "$REPO_ROOT"
    claude --dangerously-skip-permissions "$FULL_PROMPT"
    
    update_coord "completed"
    log_progress "=== DEPLOYER AGENT COMPLETED ==="
}

main "$@"
