#!/bin/bash

# Ralph Aggressive: Takes ACTION to fix and merge PRs, not just monitor
set -euo pipefail

REPO="5dlabs/cto"
MAX_ITERATIONS=20
SLEEP_BETWEEN_ITERATIONS=60

log() {
    echo -e "\033[0;34m[$(date '+%H:%M:%S')]\033[0m $1"
}

success() {
    echo -e "\033[0;32m[SUCCESS]\033[0m $1"
}

error() {
    echo -e "\033[0;31m[ERROR]\033[0m $1"
}

warn() {
    echo -e "\033[1;33m[WARN]\033[0m $1"
}

# Aggressively fix PR conflicts by updating branch
fix_conflicts() {
    local pr_number=$1
    log "🔨 FIXING conflicts in PR #$pr_number"
    
    # Try to update the branch using GitHub API
    if gh api repos/5dlabs/cto/pulls/$pr_number/update-branch --method PUT >/dev/null 2>&1; then
        success "✅ Successfully updated PR #$pr_number branch"
        return 0
    else
        error "❌ Could not auto-update PR #$pr_number - needs manual intervention"
        return 1
    fi
}

# Force retrigger CI by adding an empty commit or closing/reopening
retrigger_ci() {
    local pr_number=$1
    log "🔄 RETRIGGERING CI for PR #$pr_number"
    
    # Get PR details
    local pr_info=$(gh pr view $pr_number --repo $REPO --json headRefName,author)
    local branch=$(echo "$pr_info" | jq -r '.headRefName')
    local author=$(echo "$pr_info" | jq -r '.author.login')
    
    log "   Branch: $branch, Author: $author"
    
    # Add a comment to trigger workflows (some workflows trigger on PR comments)
    if gh pr comment $pr_number --repo $REPO --body "🤖 Ralph: Retriggering CI after k8s-openapi fix" >/dev/null 2>&1; then
        success "✅ Added comment to retrigger CI for PR #$pr_number"
        
        # Wait a moment then try to close/reopen to force CI retrigger
        sleep 5
        
        # Close and immediately reopen to force CI retrigger
        log "   Temporarily closing/reopening to force CI..."
        if gh pr close $pr_number --repo $REPO >/dev/null 2>&1; then
            sleep 2
            if gh pr reopen $pr_number --repo $REPO >/dev/null 2>&1; then
                success "✅ Successfully retriggered CI for PR #$pr_number"
                return 0
            fi
        fi
    fi
    
    error "❌ Could not retrigger CI for PR #$pr_number"
    return 1
}

# Get PR status
get_pr_status() {
    local pr_number=$1
    
    if ! gh pr view "$pr_number" --repo "$REPO" >/dev/null 2>&1; then
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
    elif [ "$merge_state" = "BLOCKED" ] && [ "$failing_checks" -gt 0 ]; then
        echo "FAILING_CI"
    elif [ "$mergeable" = "MERGEABLE" ] && [ "$merge_state" = "CLEAN" ]; then
        echo "READY_TO_MERGE"
    else
        echo "PENDING"
    fi
}

# Take aggressive action
take_aggressive_action() {
    local pr_number=$1
    local status=$2
    
    log ""
    log "🎯 PR #$pr_number: $status"
    
    case "$status" in
        "MERGED")
            success "✅ Already merged"
            return 0
            ;;
        "READY_TO_MERGE")
            log "🚀 MERGING immediately..."
            if gh pr merge "$pr_number" --repo "$REPO" --squash --auto; then
                success "✅ Successfully queued for merge"
                return 0
            else
                error "❌ Merge failed"
                return 1
            fi
            ;;
        "CONFLICTS")
            warn "⚠️ Has conflicts - FIXING..."
            if fix_conflicts "$pr_number"; then
                sleep 10  # Wait for the update to process
                return 0  # Will re-check in next iteration
            else
                return 1
            fi
            ;;
        "FAILING_CI")
            warn "⚠️ Has failing CI - RETRIGGERING..."
            if retrigger_ci "$pr_number"; then
                sleep 15  # Wait for CI to start
                return 0  # Will re-check in next iteration  
            else
                return 1
            fi
            ;;
        "PENDING")
            log "⏳ Waiting..."
            return 0
            ;;
        *)
            warn "❓ Unknown status: $status"
            return 1
            ;;
    esac
}

# Main aggressive loop
run_aggressive_iteration() {
    local iteration=$1
    log ""
    log "🤖 RALPH AGGRESSIVE ITERATION $iteration"
    log "========================================="
    
    # Get current open PRs (excluding develop branch and release PRs)
    local target_prs=$(gh pr list --repo "$REPO" --state open --json number,title --jq '.[] | select(.title | test("fix\\(|feat\\(") and (test("develop|release") | not)) | .number')
    
    if [ -z "$target_prs" ]; then
        success "🎉 No target PRs found - all clear!"
        return 0
    fi
    
    log "🎯 Target PRs: $(echo $target_prs | tr '\n' ' ')"
    
    local all_success=true
    for pr in $target_prs; do
        local status=$(get_pr_status "$pr")
        if ! take_aggressive_action "$pr" "$status"; then
            all_success=false
        fi
    done
    
    log ""
    log "📊 Status Summary:"
    for pr in $target_prs; do
        local status=$(get_pr_status "$pr")
        case "$status" in
            "MERGED") log "   PR #$pr: ✅ MERGED" ;;
            "READY_TO_MERGE") log "   PR #$pr: 🟢 READY TO MERGE" ;;
            "FAILING_CI") log "   PR #$pr: 🔴 FAILING CI" ;;
            "CONFLICTS") log "   PR #$pr: 🟡 CONFLICTS" ;;
            "PENDING") log "   PR #$pr: ⏳ PENDING" ;;
            *) log "   PR #$pr: ❓ $status" ;;
        esac
    done
    
    # Check if we're done (all target PRs merged)
    local remaining_count=0
    for pr in $target_prs; do
        local status=$(get_pr_status "$pr")
        if [ "$status" != "MERGED" ]; then
            remaining_count=$((remaining_count + 1))
        fi
    done
    
    if [ "$remaining_count" -eq 0 ]; then
        success "🎉 ALL TARGET PRs MERGED!"
        return 0
    else
        log "🔄 $remaining_count PRs remaining..."
        return 1
    fi
}

# Main function
main() {
    log "🤖 Ralph Aggressive Mode ACTIVATED"
    log "Repo: $REPO"
    log "Max iterations: $MAX_ITERATIONS"
    log ""
    
    for ((i=1; i<=MAX_ITERATIONS; i++)); do
        if run_aggressive_iteration "$i"; then
            success "🎉 SUCCESS! All PRs merged in $i iterations!"
            return 0
        fi
        
        if [ "$i" -lt "$MAX_ITERATIONS" ]; then
            log "😴 Sleeping ${SLEEP_BETWEEN_ITERATIONS}s before next iteration..."
            sleep "$SLEEP_BETWEEN_ITERATIONS"
        fi
    done
    
    error "❌ Failed to complete after $MAX_ITERATIONS iterations"
    return 1
}

# Signal handlers
cleanup() {
    log ""
    log "🛑 Ralph Aggressive interrupted"
    exit 1
}

trap cleanup SIGINT SIGTERM

main "$@"