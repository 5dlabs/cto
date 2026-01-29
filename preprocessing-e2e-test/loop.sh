#!/bin/bash
# Preprocessing Pipeline E2E Test - Ralph Loop with MiniMax Swarm
#
# Usage: ./loop.sh
#
# Runs claudesp-minimax (or minimax) swarm in a loop until all milestones are met
# or manual intervention is needed. When failback.active is true, uses Claude Opus instead.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Fetch MiniMax API key from 1Password if not set
if [[ -z "${MINIMAX_API_KEY:-}" ]]; then
    log_info "Fetching MiniMax API key from 1Password..."
    if command -v op &> /dev/null; then
        export MINIMAX_API_KEY=$(op read 'op://Development/MiniMax API Key/credential' 2>/dev/null || true)
        if [[ -z "$MINIMAX_API_KEY" ]]; then
            log_error "Failed to fetch MiniMax API key from 1Password"
            log_warn "Set MINIMAX_API_KEY environment variable manually"
            exit 1
        fi
        log_success "MiniMax API key loaded"
    else
        log_error "1Password CLI (op) not installed and MINIMAX_API_KEY not set"
        exit 1
    fi
fi

# Prefer claudesp-minimax (sneakpeek); fall back to minimax
SWARM_CMD=""
if command -v claudesp-minimax &> /dev/null; then
    SWARM_CMD="claudesp-minimax"
elif command -v minimax &> /dev/null; then
    SWARM_CMD="minimax"
else
    log_error "Neither claudesp-minimax nor minimax found"
    log_info "Install with: npx @realmikekelly/claude-sneakpeek quick --provider minimax --api-key \$MINIMAX_API_KEY --name claudesp-minimax"
    exit 1
fi
log_info "Using swarm binary: $SWARM_CMD"

# Verify required files exist
for required_file in "swarm-coordinator.md" "agents.json" "ralph-coordination.json"; do
    if [[ ! -f "$required_file" ]]; then
        log_error "Required file missing: $required_file"
        exit 1
    fi
done

# Create output directory
mkdir -p output

# Check for completion
check_complete() {
    if [[ -f ".complete" ]]; then
        return 0
    fi
    
    # Check all milestones
    local all_complete=$(jq -r '
        .milestones | to_entries | map(.value) | all
    ' ralph-coordination.json)
    
    if [[ "$all_complete" == "true" ]]; then
        local open_issues=$(jq -r '.issues_count.open' ralph-coordination.json)
        if [[ "$open_issues" == "0" ]]; then
            return 0
        fi
    fi
    
    return 1
}

# Main loop (infinite - no MAX_ITERATIONS cap)
ITERATION=0

log_info "Starting Ralph loop for preprocessing pipeline E2E test"
log_info "Running in infinite loop mode (no iteration cap)"
log_info "Working directory: $SCRIPT_DIR"
echo ""

while true; do
    ITERATION=$((ITERATION + 1))
    
    echo ""
    echo "=============================================="
    log_info "Ralph Loop Iteration $ITERATION"
    echo "=============================================="
    
    # Update coordination file with iteration info
    jq --arg iter "$ITERATION" --arg time "$(date -Iseconds)" \
        '.iteration = ($iter | tonumber) | .started_at = $time | .status = "running"' \
        ralph-coordination.json > tmp.json && mv tmp.json ralph-coordination.json
    
    # Show current milestone status
    log_info "Current milestones:"
    jq -r '.milestones | to_entries[] | "  \(.key): \(.value)"' ralph-coordination.json
    
    # Show open issues count
    open_issues=$(jq -r '.issues_count.open' ralph-coordination.json)
    if [[ "$open_issues" != "0" ]]; then
        log_warn "Open issues: $open_issues"
    fi

    # Check failback state: use Claude Opus when failback is active
    FAILBACK_ACTIVE=$(jq -r '.failback.active // false' ralph-coordination.json)
    COORDINATOR_PROMPT="$(cat swarm-coordinator.md)"
    AGENTS_JSON="$(cat agents.json)"

    if [[ "$FAILBACK_ACTIVE" == "true" ]]; then
        log_warn "Failback active: using Claude Opus instead of MiniMax"
        if ! command -v claude &> /dev/null; then
            log_error "Failback active but claude binary not found. Install Claude CLI or set failback.active to false."
            exit 1
        fi
        claude \
            --dangerously-skip-permissions \
            --permission-mode delegate \
            --agents "$AGENTS_JSON" \
            --add-dir "$SCRIPT_DIR" \
            --verbose \
            "$COORDINATOR_PROMPT" \
            2>&1 | tee -a "output/iteration-${ITERATION}.log"
    else
        log_info "Running $SWARM_CMD swarm..."
        $SWARM_CMD \
            --dangerously-skip-permissions \
            --permission-mode delegate \
            --agents "$AGENTS_JSON" \
            --add-dir "$SCRIPT_DIR" \
            --verbose \
            "$COORDINATOR_PROMPT" \
            2>&1 | tee -a "output/iteration-${ITERATION}.log"
    fi

    EXIT_CODE=${PIPESTATUS[0]}

    if [[ $EXIT_CODE -ne 0 ]]; then
        log_warn "Swarm exited with code $EXIT_CODE"
    fi
    
    # Check completion criteria
    if check_complete; then
        log_success "All milestones complete and no open issues!"
        log_success "Test completed successfully at iteration $ITERATION"
        
        # Update status
        jq '.status = "complete"' ralph-coordination.json > tmp.json && mv tmp.json ralph-coordination.json
        
        # Create completion marker
        touch .complete
        
        # Print final summary
        echo ""
        echo "=============================================="
        log_success "FINAL SUMMARY"
        echo "=============================================="
        jq '.' ralph-coordination.json
        
        break
    fi
    
    # Brief pause between iterations
    log_info "Pausing before next iteration..."
    sleep 5
done

log_info "Ralph loop ended after $ITERATION iterations"
