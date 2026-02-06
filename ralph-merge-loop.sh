#!/bin/bash

# Ralph Loop: Automated PR Merge Orchestrator
# Continuously works until all target PRs are merged and CI is green

set -euo pipefail

# Configuration
TARGET_PRS=(4297 4290 4287)  # Priority order (4304 already merged, 4307 was closed)
REPO="5dlabs/cto"
MAX_ITERATIONS=50
SLEEP_BETWEEN_ITERATIONS=30
LOG_FILE="/tmp/ralph-merge-loop.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

# Check if PR exists and get its status
get_pr_status() {
    local pr_number=$1
    
    if ! gh pr view "$pr_number" --repo "$REPO" --json state,mergeable,statusCheckRollup >/dev/null 2>&1; then
        echo "NONEXISTENT"
        return
    fi
    
    local status=$(gh pr view "$pr_number" --repo "$REPO" --json state,mergeable,statusCheckRollup,mergeStateStatus)
    local state=$(echo "$status" | jq -r '.state')
    local mergeable=$(echo "$status" | jq -r '.mergeable')
    local merge_state=$(echo "$status" | jq -r '.mergeStateStatus')
    local failing_checks=$(echo "$status" | jq -r '.statusCheckRollup[] | select(.conclusion == "FAILURE") | .name' | wc -l)
    
    if [ "$state" = "MERGED" ]; then
        echo "MERGED"
    elif [ "$state" = "CLOSED" ]; then
        echo "CLOSED"
    elif [ "$mergeable" = "CONFLICTING" ]; then
        echo "CONFLICTS"
    elif [ "$merge_state" = "BLOCKED" ]; then
        echo "BLOCKED"
    elif [ "$failing_checks" -gt 0 ]; then
        echo "FAILING_CI"
    elif [ "$mergeable" = "MERGEABLE" ] && [ "$merge_state" = "CLEAN" ]; then
        echo "READY_TO_MERGE"
    else
        echo "PENDING"
    fi
}

# Get failing check details
get_failing_checks() {
    local pr_number=$1
    gh pr view "$pr_number" --repo "$REPO" --json statusCheckRollup | \
        jq -r '.statusCheckRollup[] | select(.conclusion == "FAILURE") | "\(.workflowName): \(.name)"'
}

# Check acceptance criteria - all PRs merged and main branch CI green
check_acceptance_criteria() {
    log "🎯 Checking acceptance criteria..."
    
    local all_merged=true
    for pr in "${TARGET_PRS[@]}"; do
        local status=$(get_pr_status "$pr")
        if [ "$status" != "MERGED" ]; then
            all_merged=false
            break
        fi
    done
    
    if [ "$all_merged" = true ]; then
        # Check main branch CI status
        local main_ci_status=$(gh run list --repo "$REPO" --branch main --limit 1 --json conclusion | jq -r '.[0].conclusion')
        if [ "$main_ci_status" = "success" ]; then
            success "✅ ALL ACCEPTANCE CRITERIA MET!"
            success "   - All target PRs merged"
            success "   - Main branch CI green"
            return 0
        else
            warn "Main branch CI not green: $main_ci_status"
            return 1
        fi
    else
        return 1
    fi
}

# Take action based on PR status
take_action() {
    local pr_number=$1
    local status=$2
    
    log "📋 PR #$pr_number status: $status"
    
    case "$status" in
        "MERGED")
            success "✅ PR #$pr_number already merged"
            ;;
        "READY_TO_MERGE")
            log "🚀 Attempting to merge PR #$pr_number..."
            if gh pr merge "$pr_number" --repo "$REPO" --squash --auto; then
                success "✅ PR #$pr_number queued for auto-merge"
            else
                error "❌ Failed to merge PR #$pr_number"
            fi
            ;;
        "FAILING_CI")
            warn "⚠️ PR #$pr_number has failing CI checks:"
            get_failing_checks "$pr_number" | while read -r check; do
                log "   - $check"
            done
            
            # Special handling for specific issues
            if [ "$pr_number" = "4304" ] || [ "$pr_number" = "4290" ] || [ "$pr_number" = "4287" ]; then
                log "💡 Likely waiting for PR #4307 k8s-openapi fix to be merged"
            fi
            ;;
        "CONFLICTS")
            warn "⚠️ PR #$pr_number has merge conflicts - needs manual resolution"
            ;;
        "BLOCKED")
            warn "⚠️ PR #$pr_number is blocked - investigating..."
            # Could add logic to unblock if it's waiting for reviews
            ;;
        "PENDING")
            log "⏳ PR #$pr_number is pending..."
            ;;
        "NONEXISTENT")
            error "❌ PR #$pr_number does not exist"
            ;;
        "CLOSED")
            warn "⚠️ PR #$pr_number was closed"
            ;;
    esac
}

# Main iteration
run_iteration() {
    local iteration=$1
    
    log ""
    log "🔄 RALPH ITERATION $iteration"
    log "================================="
    
    # First check if we're done
    if check_acceptance_criteria; then
        return 0
    fi
    
    # Process each PR in priority order
    for pr in "${TARGET_PRS[@]}"; do
        local status=$(get_pr_status "$pr")
        take_action "$pr" "$status"
    done
    
    log ""
    log "📊 Current Status Summary:"
    for pr in "${TARGET_PRS[@]}"; do
        local status=$(get_pr_status "$pr")
        case "$status" in
            "MERGED") log "   PR #$pr: ✅ MERGED" ;;
            "READY_TO_MERGE") log "   PR #$pr: 🟢 READY TO MERGE" ;;
            "FAILING_CI") log "   PR #$pr: 🔴 FAILING CI" ;;
            "CONFLICTS") log "   PR #$pr: 🟡 MERGE CONFLICTS" ;;
            "BLOCKED") log "   PR #$pr: 🟠 BLOCKED" ;;
            "PENDING") log "   PR #$pr: ⏳ PENDING" ;;
            *) log "   PR #$pr: ❓ $status" ;;
        esac
    done
    
    return 1  # Continue looping
}

# Main loop
main() {
    log "🤖 Ralph Merge Loop Starting..."
    log "Target PRs: ${TARGET_PRS[*]}"
    log "Repository: $REPO"
    log "Max iterations: $MAX_ITERATIONS"
    log ""
    
    # Clear previous log
    > "$LOG_FILE"
    
    for ((i=1; i<=MAX_ITERATIONS; i++)); do
        if run_iteration "$i"; then
            success "🎉 SUCCESS! All PRs merged and CI green!"
            success "Completed in $i iterations"
            return 0
        fi
        
        if [ "$i" -lt "$MAX_ITERATIONS" ]; then
            log "😴 Sleeping ${SLEEP_BETWEEN_ITERATIONS}s before next iteration..."
            sleep "$SLEEP_BETWEEN_ITERATIONS"
        fi
    done
    
    error "❌ Failed to complete after $MAX_ITERATIONS iterations"
    error "Check the logs and intervene manually if needed"
    return 1
}

# Signal handlers
cleanup() {
    log ""
    log "🛑 Ralph Loop interrupted"
    exit 1
}

trap cleanup SIGINT SIGTERM

# Run if called directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi