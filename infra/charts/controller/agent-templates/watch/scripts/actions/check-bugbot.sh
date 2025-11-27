#!/bin/bash
# Check for bug-bot comments on a PR
# Usage: check-bugbot.sh --pr-number 123 [--repo 5dlabs/cto] [--wait]
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

while [ $# -gt 0 ]; do
  case "$1" in
    --pr-number) pr_number="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --wait) wait_stable="true"; shift ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$pr_number" ]; then
  log_error "Required: --pr-number"
  exit 1
fi

export GITHUB_REPO="$repo"

log_info "Checking bug-bot comments on PR #$pr_number"

# Optionally wait for bug-bot to finish
if [ "$wait_stable" = "true" ]; then
  gh_wait_bugbot_complete "$pr_number" 60
fi

# Get and parse bug-bot issues
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

