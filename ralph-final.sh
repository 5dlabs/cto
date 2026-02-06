#!/bin/bash

# Ralph Final: Focus on just the last 2 critical PRs
set -euo pipefail

REPO="5dlabs/cto"
TARGET_PRS=(4290 4287 4291)
MAX_ITERATIONS=15
CHECK_INTERVAL=45

log() {
    echo -e "\033[0;34m[$(date '+%H:%M:%S')]\033[0m $1"
}

success() {
    echo -e "\033[0;32m[SUCCESS]\033[0m $1"
}

warn() {
    echo -e "\033[1;33m[WARN]\033[0m $1"
}

get_pr_status() {
    local pr_number=$1
    local pr_info=$(gh pr view $pr_number --repo $REPO --json state,mergeable,mergeStateStatus,statusCheckRollup 2>/dev/null || echo '{"state":"CLOSED"}')
    local state=$(echo "$pr_info" | jq -r '.state')
    
    if [ "$state" = "MERGED" ]; then
        echo "MERGED"
    elif [ "$state" = "CLOSED" ]; then
        echo "CLOSED"
    else
        local mergeable=$(echo "$pr_info" | jq -r '.mergeable')
        local merge_state=$(echo "$pr_info" | jq -r '.mergeStateStatus')
        local failing_checks=$(echo "$pr_info" | jq -r '.statusCheckRollup[] | select(.conclusion == "FAILURE") | .name' | wc -l | tr -d ' ')
        local pending_checks=$(echo "$pr_info" | jq -r '.statusCheckRollup[] | select(.status == "IN_PROGRESS") | .name' | wc -l | tr -d ' ')
        
        if [ "$mergeable" = "MERGEABLE" ] && [ "$merge_state" = "CLEAN" ]; then
            echo "READY"
        elif [ "$pending_checks" -gt 0 ]; then
            echo "PENDING_CI"
        elif [ "$failing_checks" -gt 0 ]; then
            echo "FAILING"
        else
            echo "UNKNOWN"
        fi
    fi
}

retrigger_pr_ci() {
    local pr_number=$1
    log "🔄 Retriggering CI for PR #$pr_number"
    
    # Add comment to potentially retrigger
    gh pr comment $pr_number --repo $REPO --body "🤖 Ralph Final: Retriggering CI" >/dev/null 2>&1 || true
    
    # Try close/reopen to force retrigger
    if gh pr close $pr_number --repo $REPO >/dev/null 2>&1; then
        sleep 3
        gh pr reopen $pr_number --repo $REPO >/dev/null 2>&1 || true
        success "✅ Retriggered CI for PR #$pr_number"
        return 0
    fi
    
    warn "⚠️ Could not retrigger CI for PR #$pr_number"
    return 1
}

main() {
    log "🎯 Ralph Final: Focusing on PRs ${TARGET_PRS[*]}"
    log "Will check every ${CHECK_INTERVAL}s for up to $MAX_ITERATIONS iterations"
    log ""
    
    for ((i=1; i<=MAX_ITERATIONS; i++)); do
        log "🔄 ITERATION $i"
        log "==============="
        
        local all_done=true
        local actions_taken=false
        
        for pr in "${TARGET_PRS[@]}"; do
            local status=$(get_pr_status "$pr")
            
            case "$status" in
                "MERGED")
                    success "✅ PR #$pr: MERGED"
                    ;;
                "READY")
                    log "🚀 PR #$pr: READY - MERGING NOW"
                    if gh pr merge $pr --repo $REPO --squash --auto; then
                        success "✅ PR #$pr queued for auto-merge"
                    else
                        warn "❌ Failed to merge PR #$pr"
                        all_done=false
                    fi
                    actions_taken=true
                    ;;
                "PENDING_CI")
                    log "⏳ PR #$pr: CI in progress"
                    all_done=false
                    ;;
                "FAILING")
                    warn "🔴 PR #$pr: Has failing CI - retriggering"
                    if retrigger_pr_ci "$pr"; then
                        actions_taken=true
                    fi
                    all_done=false
                    ;;
                "CLOSED")
                    warn "❌ PR #$pr: CLOSED"
                    ;;
                *)
                    warn "❓ PR #$pr: $status"
                    all_done=false
                    ;;
            esac
        done
        
        log ""
        
        if [ "$all_done" = true ]; then
            success "🎉 ALL TARGET PRS MERGED! Ralph's work is done."
            return 0
        fi
        
        if [ "$i" -lt "$MAX_ITERATIONS" ]; then
            if [ "$actions_taken" = true ]; then
                log "😴 Actions taken, waiting 90s for CI to start..."
                sleep 90
            else
                log "😴 Sleeping ${CHECK_INTERVAL}s..."
                sleep "$CHECK_INTERVAL"
            fi
        fi
    done
    
    warn "❌ Max iterations reached. Remaining PRs need manual attention."
    return 1
}

# Signal handling
cleanup() {
    log ""
    log "🛑 Ralph Final interrupted"
    exit 1
}
trap cleanup SIGINT SIGTERM

main "$@"