#!/bin/bash
# Poll GitHub CI until all checks pass
# Usage: poll-ci.sh --pr-number 123 [--timeout 1800] [--repo 5dlabs/cto]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/github.sh"

# Parse arguments
pr_number=""
timeout="1800"  # 30 minutes default
repo="${GITHUB_REPO:-5dlabs/cto}"

while [ $# -gt 0 ]; do
  case "$1" in
    --pr-number) pr_number="$2"; shift 2 ;;
    --timeout) timeout="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$pr_number" ]; then
  log_error "Required: --pr-number"
  exit 1
fi

export GITHUB_REPO="$repo"

log_info "Polling CI for PR #$pr_number (timeout: ${timeout}s)"

# Wait for checks to complete
if ! gh_wait_checks_complete "$pr_number" "$timeout"; then
  log_error "Timeout waiting for CI checks"
  gh_get_check_status "$pr_number"
  exit 1
fi

# Verify all passed
if gh_all_checks_passed "$pr_number"; then
  log_success "All CI checks passed for PR #$pr_number"
  exit 0
else
  log_error "Some CI checks failed"
  gh_get_check_status "$pr_number"
  exit 1
fi

