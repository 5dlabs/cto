#!/bin/bash
# Merge a PR after all checks pass
# Usage: merge-pr.sh --pr-number 123 [--repo 5dlabs/cto] [--method squash] [--wait] [--timeout 600]
#
# This script:
# 1. Optionally waits for CI checks to complete (--wait)
# 2. Verifies all CI checks passed
# 3. Checks for bug-bot issues (fails if found)
# 4. Verifies PR is mergeable
# 5. Merges the PR

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/github.sh"

# Parse arguments
pr_number=""
repo="${GITHUB_REPO:-5dlabs/cto}"
method="squash"
wait_for_ci="false"
timeout="1800"  # 30 minutes default

while [ $# -gt 0 ]; do
  case "$1" in
    --pr-number) pr_number="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --method) method="$2"; shift 2 ;;
    --wait) wait_for_ci="true"; shift ;;
    --timeout) timeout="$2"; shift 2 ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$pr_number" ]; then
  log_error "Required: --pr-number"
  exit 1
fi

export GITHUB_REPO="$repo"

log_info "Preparing to merge PR #$pr_number"

# Step 0: Wait for CI if requested
if [ "$wait_for_ci" = "true" ]; then
  log_step "Waiting for CI checks to complete (timeout: ${timeout}s)..."
  if ! gh_wait_checks_complete "$pr_number" "$timeout"; then
    log_error "Timed out waiting for CI checks"
    exit 1
  fi
fi

# Step 1: Verify all CI checks passed
log_step "Verifying CI checks..."
if ! gh_all_checks_passed "$pr_number"; then
  log_error "CI checks not all passing"
  gh_get_check_status "$pr_number"
  exit 1
fi
log_success "All CI checks passed"

# Step 2: Check for bug-bot issues
log_step "Checking for bug-bot issues..."
if gh_has_bugbot_comments "$pr_number"; then
  log_error "Bug-bot has comments on this PR - address them first"
  gh_parse_bugbot_issues "$pr_number" | jq '.'
  exit 1
fi
log_success "No bug-bot issues"

# Step 3: Verify PR is mergeable
log_step "Checking merge status..."
if ! gh_pr_is_mergeable "$pr_number"; then
  log_error "PR is not mergeable - may have conflicts"
  gh_pr_status "$pr_number" | jq '{mergeable, mergeStateStatus}'
  exit 1
fi
log_success "PR is mergeable"

# Step 4: Merge
log_step "Merging PR #$pr_number..."
gh_merge_pr "$pr_number" "$method"

log_success "PR #$pr_number merged successfully"

