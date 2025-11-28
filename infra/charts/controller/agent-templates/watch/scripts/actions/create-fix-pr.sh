#!/bin/bash
# Create a fix PR: branch, commit, push, open PR
# Usage: create-fix-pr.sh --task-id 42 --title "fix: description" --body "Details..."
#
# Output: PR number on success

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/git.sh"
source "$SCRIPT_DIR/../lib/github.sh"

# Parse arguments
task_id=""
title=""
body=""
repo_dir="${REPO_DIR:-/workspace/repo}"
repo="${GITHUB_REPO:-5dlabs/cto}"

while [ $# -gt 0 ]; do
  case "$1" in
    --task-id) task_id="$2"; shift 2 ;;
    --title) title="$2"; shift 2 ;;
    --body) body="$2"; shift 2 ;;
    --repo-dir) repo_dir="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$task_id" ] || [ -z "$title" ]; then
  log_error "Required: --task-id and --title"
  exit 1
fi

export GITHUB_REPO="$repo"
cd "$repo_dir"

# Generate branch name
branch_name=$(git_fix_branch_name "$task_id")

log_info "Creating fix PR for task $task_id"
log_info "  Branch: $branch_name"
log_info "  Title: $title"

# Step 1: Create branch
log_step "Creating branch..."
git_create_branch "$branch_name"

# Step 2: Stage and commit
log_step "Committing changes..."
if ! git_commit "$title"; then
  log_error "No changes to commit"
  exit 1
fi

# Step 3: Push
log_step "Pushing branch..."
git_push "$branch_name"

# Step 4: Create PR
log_step "Creating pull request..."
pr_body="${body:-Automated fix for E2E Watch system.

**Task ID**: $task_id
**Branch**: $branch_name

This PR was created automatically by the Remediation Agent.}"

pr_number=$(gh_create_pr "$branch_name" "$title" "$pr_body" "task-$task_id" "e2e-watch")

if [ -z "$pr_number" ]; then
  log_error "Failed to create PR"
  exit 1
fi

# Save PR info for later steps
if [ -n "${WATCH_WORKSPACE:-}" ]; then
  cat > "$WATCH_WORKSPACE/current-pr.json" <<EOF
{
  "pr_number": $pr_number,
  "branch": "$branch_name",
  "task_id": "$task_id",
  "commit_sha": "$(git_current_sha short)"
}
EOF
  log_info "PR info saved to $WATCH_WORKSPACE/current-pr.json"
fi

log_success "PR #$pr_number created"
echo "$pr_number"

