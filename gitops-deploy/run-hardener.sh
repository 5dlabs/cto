#!/bin/bash
# GitOps Hardener Agent - runs Droid to improve GitOps configs
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/hardener-prompt.md"
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

CHECK_INTERVAL="${CHECK_INTERVAL:-120}"

# Set kubeconfig explicitly for this cluster
export KUBECONFIG="/tmp/latitude-test/kubeconfig"

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v droid &> /dev/null; then
        echo "Error: droid CLI not found"
        exit 1
    fi
    
    success "All prerequisites met"
}

# Wait for deployer to start
wait_for_deployer() {
    log "Waiting for deployer agent to start..."
    
    while true; do
        if [[ -f "$COORD_FILE" ]]; then
            local status
            status=$(jq -r '.deployer.status // "not_started"' "$COORD_FILE" 2>/dev/null || echo "not_started")
            if [[ "$status" == "running" || "$status" == "completed" ]]; then
                success "Deployer is $status, starting hardener"
                return 0
            fi
        fi
        sleep 5
    done
}

# Update coordination file
update_coord() {
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    if command -v jq &> /dev/null; then
        local tmp
        tmp=$(mktemp)
        local check_count
        check_count=$(jq -r '.hardener.checkCount // 0' "$COORD_FILE" 2>/dev/null || echo "0")
        check_count=$((check_count + 1))
        
        jq --arg ts "$timestamp" \
           --arg pid "$$" \
           --argjson count "$check_count" \
           '.hardener.status = "running" | 
            .hardener.lastCheck = $ts | 
            .hardener.checkCount = $count |
            .hardener.pid = ($pid | tonumber)' \
           "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
    fi
}

# Run a single check
run_hardener_check() {
    local check_num="$1"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    log "Running hardening check #$check_num..."
    update_coord
    
    # Read current state
    local progress_tail
    progress_tail=$(tail -50 "$PROGRESS_FILE" 2>/dev/null || echo "No progress yet")
    
    local coord_state
    coord_state=$(cat "$COORD_FILE" 2>/dev/null || echo "{}")
    
    local apps_status
    apps_status=$(kubectl get applications -n argocd --no-headers 2>/dev/null | head -30 || echo "Cannot fetch apps")
    
    # Build prompt
    local PROMPT_CONTENT
    PROMPT_CONTENT=$(cat "$PROMPT_FILE")
    
    local FULL_PROMPT="$PROMPT_CONTENT

---

## Current Check: #$check_num at $timestamp

### Recent Progress
\`\`\`
$progress_tail
\`\`\`

### Coordination State
\`\`\`json
$coord_state
\`\`\`

### ArgoCD Applications
\`\`\`
$apps_status
\`\`\`

### Your Task This Check

1. **Review progress.txt** - Look for ERROR, WORKAROUND, or manual interventions
2. **Identify patterns** - What could be fixed in GitOps configs?
3. **IMPLEMENT FIXES** - Edit files in infra/gitops/ to prevent future issues
4. **Document** - Update lessons-learned.md with your fixes
5. **Log** - Add your actions to progress.txt

Focus on sync-wave ordering, missing namespaces, and CRD dependencies.
"

    # Run Droid
    cd "$REPO_ROOT"
    droid exec --skip-permissions-unsafe --cwd "$REPO_ROOT" "$FULL_PROMPT" || true
}

# Main loop
main() {
    log "=== GitOps Hardener Agent (Droid) ==="
    log "Repo root: $REPO_ROOT"
    log "Prompt file: $PROMPT_FILE"
    log "Check interval: ${CHECK_INTERVAL}s"
    
    check_prerequisites
    wait_for_deployer
    
    log "Hardener initialized"
    warn "Using --skip-permissions-unsafe - full access mode"
    log "Press Ctrl+C to stop"
    
    local check_count=0
    
    while true; do
        check_count=$((check_count + 1))
        run_hardener_check "$check_count"
        
        log "Sleeping ${CHECK_INTERVAL}s until next check..."
        sleep "$CHECK_INTERVAL"
    done
}

main "$@"
