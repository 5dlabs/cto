#!/bin/bash
# Check for bug-bot comments on a PR, trigger if needed, iterate until resolved
# Usage: check-bugbot.sh --pr-number 123 [--repo 5dlabs/cto] [--trigger] [--wait]
# 
# Options:
#   --trigger    Trigger bugbot with @bugbot run if check is stale/skipped
#   --wait       Wait for bugbot to stabilize before checking
#
# Output: JSON with bug-bot issues if any found
# Exit: 0 if no issues, 1 if issues found (agent should address them)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/github.sh"

# Parse arguments
pr_number=""
repo="${GITHUB_REPO:-5dlabs/cto}"
wait_stable="false"
trigger_bugbot="false"

while [ $# -gt 0 ]; do
  case "$1" in
    --pr-number) pr_number="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --wait) wait_stable="true"; shift ;;
    --trigger) trigger_bugbot="true"; shift ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$pr_number" ]; then
  log_error "Required: --pr-number"
  exit 1
fi

export GITHUB_REPO="$repo"

log_info "Checking bug-bot status on PR #$pr_number"

# ============================================================================
# Step 1: Check if Bugbot has run (look at PR checks)
# ============================================================================
bugbot_check_status() {
  local pr="$1"
  # Look for Cursor Bugbot or similar in checks
  gh pr checks "$pr" --repo "$repo" --json name,state,bucket 2>/dev/null | \
    jq -r '.[] | select(.name | test("bugbot|Bugbot|BUGBOT"; "i")) | .bucket' | head -1
}

bugbot_status=$(bugbot_check_status "$pr_number")
log_info "Bugbot check status: ${bugbot_status:-not found}"

# ============================================================================
# Step 2: Trigger Bugbot if needed
# ============================================================================
if [ "$trigger_bugbot" = "true" ]; then
  case "${bugbot_status:-}" in
    pass|fail)
      log_info "Bugbot already ran (status: $bugbot_status)"
      ;;
    pending)
      log_info "Bugbot is running, waiting..."
      ;;
    skipping|""|neutral)
      log_warn "Bugbot hasn't run or was skipped - triggering with @bugbot run"
      gh pr comment "$pr_number" --repo "$repo" --body "@bugbot run"
      log_info "Waiting 30s for Bugbot to start..."
      sleep 30
      ;;
  esac
fi

# ============================================================================
# Step 3: Wait for Bugbot to complete if requested
# ============================================================================
if [ "$wait_stable" = "true" ]; then
  log_step "Waiting for Bugbot to complete..."
  
  timeout=300  # 5 minutes
  start_time=$(date +%s)
  
  while true; do
    current_time=$(date +%s)
    elapsed=$((current_time - start_time))
    
    if [ $elapsed -gt $timeout ]; then
      log_warn "Timeout waiting for Bugbot - continuing anyway"
      break
    fi
    
    status=$(bugbot_check_status "$pr_number")
    
    case "${status:-}" in
      pass|fail|skipping|neutral)
        log_info "Bugbot completed (status: $status)"
        break
        ;;
      pending)
        echo -n "."
        sleep 10
        ;;
      *)
        log_info "Bugbot status: ${status:-unknown}"
        sleep 10
        ;;
    esac
  done
  echo ""
  
  # Also wait for comments to stabilize
  gh_wait_bugbot_complete "$pr_number" 60
fi

# ============================================================================
# Step 4: Check for Bugbot comments
# ============================================================================
log_step "Checking for bug-bot comments..."

issues=$(gh_parse_bugbot_issues "$pr_number")
has_issues=$(echo "$issues" | jq -r '.has_issues')

if [ "$has_issues" = "true" ]; then
  count=$(echo "$issues" | jq -r '.count')
  log_warn "Bug-bot found $count issue(s) on PR #$pr_number"
  
  # Output the issues for the agent to address
  echo "$issues" | jq '.'
  
  # Also save to a file for the agent
  if [ -n "${WATCH_WORKSPACE:-}" ]; then
    echo "$issues" > "$WATCH_WORKSPACE/bugbot-issues.json"
    log_info "Issues saved to $WATCH_WORKSPACE/bugbot-issues.json"
  fi
  
  exit 1
else
  log_success "No bug-bot issues found on PR #$pr_number"
  echo '{"has_issues": false, "issues": []}'
  exit 0
fi

