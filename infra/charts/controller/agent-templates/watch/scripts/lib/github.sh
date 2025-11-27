#!/bin/bash
# GitHub operations for E2E Watch Remediation Agent
# Requires: gh CLI authenticated

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Default repository (can be overridden)
GITHUB_REPO="${GITHUB_REPO:-5dlabs/cto}"

# Bug-bot username (adjust if different)
BUGBOT_USERNAME="${BUGBOT_USERNAME:-5DLabs-Bug-Bot}"

# ============================================================================
# PR Operations
# ============================================================================

# Create a pull request
# Usage: gh_create_pr "branch" "title" "body" [labels...]
gh_create_pr() {
  local branch="$1"
  local title="$2"
  local body="$3"
  shift 3
  local labels=("$@")
  
  require_cmd gh
  
  log_step "Creating PR: $title"
  
  local label_args=()
  for label in "${labels[@]}"; do
    label_args+=("--label" "$label")
  done
  
  local pr_url
  pr_url=$(gh pr create \
    --repo "$GITHUB_REPO" \
    --head "$branch" \
    --base main \
    --title "$title" \
    --body "$body" \
    "${label_args[@]}" \
    --json url -q '.url')
  
  if [ -z "$pr_url" ]; then
    log_error "Failed to create PR"
    return 1
  fi
  
  local pr_number
  pr_number=$(echo "$pr_url" | grep -oE '[0-9]+$')
  
  log_success "PR created: $pr_url (PR #$pr_number)"
  echo "$pr_number"
}

# Get PR status
# Usage: gh_pr_status PR_NUMBER
gh_pr_status() {
  local pr_number="$1"
  
  gh pr view "$pr_number" \
    --repo "$GITHUB_REPO" \
    --json state,mergeable,mergeStateStatus,statusCheckRollup
}

# Check if PR is mergeable
# Usage: gh_pr_is_mergeable PR_NUMBER
gh_pr_is_mergeable() {
  local pr_number="$1"
  
  local status
  status=$(gh pr view "$pr_number" \
    --repo "$GITHUB_REPO" \
    --json mergeable,mergeStateStatus \
    -q '.mergeable + ":" + .mergeStateStatus')
  
  local mergeable="${status%%:*}"
  local merge_state="${status##*:}"
  
  if [ "$mergeable" = "MERGEABLE" ] && [ "$merge_state" = "CLEAN" ]; then
    return 0
  fi
  
  log_warn "PR #$pr_number not mergeable: $mergeable / $merge_state"
  return 1
}

# Merge a PR
# Usage: gh_merge_pr PR_NUMBER [squash|merge|rebase]
gh_merge_pr() {
  local pr_number="$1"
  local method="${2:-squash}"
  
  log_step "Merging PR #$pr_number ($method)"
  
  gh pr merge "$pr_number" \
    --repo "$GITHUB_REPO" \
    "--$method" \
    --delete-branch
  
  log_success "PR #$pr_number merged"
}

# ============================================================================
# CI/Checks Operations
# ============================================================================

# Wait for CI runs to start
# Usage: gh_wait_ci_start BRANCH [timeout_seconds]
gh_wait_ci_start() {
  local branch="$1"
  local timeout="${2:-300}"
  
  log_step "Waiting for CI to start on branch: $branch"
  
  _check_ci_started() {
    local runs
    runs=$(gh run list \
      --repo "$GITHUB_REPO" \
      --branch "$branch" \
      --json status \
      -q 'length')
    
    [ "$runs" -gt 0 ]
  }
  
  poll_until "$timeout" 10 "CI runs to start" _check_ci_started
}

# Wait for all PR checks to complete
# Usage: gh_wait_checks_complete PR_NUMBER [timeout_seconds]
gh_wait_checks_complete() {
  local pr_number="$1"
  local timeout="${2:-1800}"  # 30 min default
  
  log_step "Waiting for PR #$pr_number checks to complete"
  
  _check_all_complete() {
    local checks
    checks=$(gh pr checks "$pr_number" --repo "$GITHUB_REPO" 2>&1 || true)
    
    # No pending checks means all complete
    if ! echo "$checks" | grep -qE "pending|in_progress|queued"; then
      return 0
    fi
    return 1
  }
  
  poll_until "$timeout" 30 "all checks to complete" _check_all_complete
}

# Get check status summary
# Usage: gh_get_check_status PR_NUMBER
gh_get_check_status() {
  local pr_number="$1"
  
  gh pr checks "$pr_number" --repo "$GITHUB_REPO" --json name,state,conclusion 2>&1
}

# Check if all checks passed
# Usage: gh_all_checks_passed PR_NUMBER
gh_all_checks_passed() {
  local pr_number="$1"
  
  local failed
  failed=$(gh pr checks "$pr_number" \
    --repo "$GITHUB_REPO" \
    --json conclusion \
    -q '[.[] | select(.conclusion != "SUCCESS" and .conclusion != "SKIPPED" and .conclusion != null)] | length')
  
  [ "$failed" = "0" ]
}

# ============================================================================
# Bug-Bot Operations
# ============================================================================

# Get bug-bot comments on a PR
# Usage: gh_get_bugbot_comments PR_NUMBER
# Returns: JSON array of bug-bot comments
gh_get_bugbot_comments() {
  local pr_number="$1"
  
  gh api \
    --paginate \
    "repos/$GITHUB_REPO/issues/$pr_number/comments" \
    --jq "[.[] | select(.user.login == \"$BUGBOT_USERNAME\")]"
}

# Check if PR has any bug-bot comments
# Usage: gh_has_bugbot_comments PR_NUMBER
gh_has_bugbot_comments() {
  local pr_number="$1"
  
  local count
  count=$(gh_get_bugbot_comments "$pr_number" | jq 'length')
  
  [ "$count" -gt 0 ]
}

# Get bug-bot comments as structured issues
# Usage: gh_parse_bugbot_issues PR_NUMBER
# Returns: JSON with parsed issues for agent to address
gh_parse_bugbot_issues() {
  local pr_number="$1"
  
  local comments
  comments=$(gh_get_bugbot_comments "$pr_number")
  
  if [ "$(echo "$comments" | jq 'length')" = "0" ]; then
    echo '{"has_issues": false, "issues": []}'
    return 0
  fi
  
  # Extract and structure the issues
  echo "$comments" | jq '{
    has_issues: true,
    count: length,
    issues: [.[] | {
      id: .id,
      created_at: .created_at,
      body: .body,
      url: .html_url
    }]
  }'
}

# Wait for bug-bot to finish (no new comments for a period)
# Usage: gh_wait_bugbot_complete PR_NUMBER [stability_seconds]
gh_wait_bugbot_complete() {
  local pr_number="$1"
  local stability="${2:-60}"  # Wait 60s with no new comments
  
  log_step "Waiting for bug-bot to finish analyzing PR #$pr_number"
  
  local last_count=-1
  local stable_for=0
  
  while [ $stable_for -lt "$stability" ]; do
    local current_count
    current_count=$(gh_get_bugbot_comments "$pr_number" | jq 'length')
    
    if [ "$current_count" = "$last_count" ]; then
      stable_for=$((stable_for + 10))
    else
      stable_for=0
      last_count="$current_count"
    fi
    
    sleep 10
  done
  
  log_success "Bug-bot appears stable (no new comments for ${stability}s)"
}

# ============================================================================
# Review Comments Operations
# ============================================================================

# Get all review comments on a PR
# Usage: gh_get_review_comments PR_NUMBER
gh_get_review_comments() {
  local pr_number="$1"
  
  gh api \
    --paginate \
    "repos/$GITHUB_REPO/pulls/$pr_number/comments"
}

# Get pending review threads (unresolved)
# Usage: gh_get_unresolved_threads PR_NUMBER
gh_get_unresolved_threads() {
  local pr_number="$1"
  
  gh pr view "$pr_number" \
    --repo "$GITHUB_REPO" \
    --json reviewDecision,reviews,comments \
    --jq '.reviews | [.[] | select(.state == "CHANGES_REQUESTED")]'
}

