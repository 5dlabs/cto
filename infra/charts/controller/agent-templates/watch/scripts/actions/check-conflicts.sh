#!/bin/bash
# Check for merge conflicts on a PR and provide resolution guidance
# Usage: check-conflicts.sh --pr-number 123 [--repo 5dlabs/cto] [--rebase]
#
# This script:
# 1. Checks if PR has merge conflicts
# 2. If conflicts exist, lists conflicted files
# 3. Optionally attempts to resolve via rebase
#
# Output: JSON with conflict status and affected files
# Exit: 0 if no conflicts, 1 if conflicts exist

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/github.sh"

# Parse arguments
pr_number=""
repo="${GITHUB_REPO:-5dlabs/cto}"
attempt_rebase="false"
repo_dir="${REPO_DIR:-/workspace/repo}"

while [ $# -gt 0 ]; do
  case "$1" in
    --pr-number) pr_number="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --rebase) attempt_rebase="true"; shift ;;
    --repo-dir) repo_dir="$2"; shift 2 ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$pr_number" ]; then
  log_error "Required: --pr-number"
  exit 1
fi

export GITHUB_REPO="$repo"

log_info "Checking merge conflicts for PR #$pr_number"

# Get PR merge status
pr_info=$(gh pr view "$pr_number" --repo "$repo" --json mergeable,mergeStateStatus,headRefName,baseRefName)

mergeable=$(echo "$pr_info" | jq -r '.mergeable')
merge_state=$(echo "$pr_info" | jq -r '.mergeStateStatus')
head_branch=$(echo "$pr_info" | jq -r '.headRefName')
base_branch=$(echo "$pr_info" | jq -r '.baseRefName')

log_info "Mergeable: $mergeable"
log_info "Merge state: $merge_state"
log_info "Branch: $head_branch -> $base_branch"

# Check for conflicts
if [ "$mergeable" = "CONFLICTING" ]; then
  log_error "PR #$pr_number has merge conflicts!"
  
  # Try to get list of conflicted files by doing a test merge locally
  if [ -d "$repo_dir/.git" ]; then
    log_step "Identifying conflicted files..."
    
    cd "$repo_dir" || exit 1
    
    # Fetch latest
    git fetch origin "$base_branch" "$head_branch" 2>/dev/null || true
    
    # Try merge to find conflicts (don't actually commit)
    if ! git merge-tree "$(git merge-base "origin/$base_branch" "origin/$head_branch")" "origin/$base_branch" "origin/$head_branch" 2>/dev/null | grep -q "^<<<<<<<"; then
      # Alternative: check with git merge --no-commit
      git checkout "origin/$head_branch" -b temp-conflict-check 2>/dev/null || true
      if ! git merge "origin/$base_branch" --no-commit 2>&1 | tee /tmp/merge-output.txt; then
        conflicted_files=$(git diff --name-only --diff-filter=U 2>/dev/null || grep "CONFLICT" /tmp/merge-output.txt | sed 's/.*Merge conflict in //' || echo "unknown")
        git merge --abort 2>/dev/null || true
      fi
      git checkout - 2>/dev/null || true
      git branch -D temp-conflict-check 2>/dev/null || true
    fi
  fi
  
  # Output conflict report
  report=$(jq -n \
    --arg pr_number "$pr_number" \
    --arg mergeable "$mergeable" \
    --arg merge_state "$merge_state" \
    --arg head_branch "$head_branch" \
    --arg base_branch "$base_branch" \
    --arg conflicted_files "${conflicted_files:-unknown}" \
    '{
      has_conflicts: true,
      pr_number: $pr_number,
      mergeable: $mergeable,
      merge_state: $merge_state,
      head_branch: $head_branch,
      base_branch: $base_branch,
      conflicted_files: ($conflicted_files | split("\n")),
      resolution: "Rebase or merge main into branch and resolve conflicts"
    }')
  
  echo "$report" | jq '.'
  
  # Save to workspace
  if [ -n "${WATCH_WORKSPACE:-}" ]; then
    echo "$report" > "$WATCH_WORKSPACE/merge-conflicts.json"
    log_info "Conflict report saved to $WATCH_WORKSPACE/merge-conflicts.json"
  fi
  
  # Optionally attempt rebase
  if [ "$attempt_rebase" = "true" ] && [ -d "$repo_dir/.git" ]; then
    log_step "Attempting to resolve via rebase..."
    
    cd "$repo_dir" || exit 1
    git checkout "$head_branch" 2>/dev/null || git checkout "origin/$head_branch" -b "$head_branch"
    
    if git rebase "origin/$base_branch"; then
      log_success "Rebase successful - pushing..."
      git push origin "$head_branch" --force-with-lease
      log_success "Conflicts resolved via rebase!"
      exit 0
    else
      log_error "Rebase failed - manual resolution needed"
      git rebase --abort
      exit 1
    fi
  fi
  
  exit 1

elif [ "$mergeable" = "UNKNOWN" ]; then
  log_warn "Merge status unknown - GitHub may still be calculating"
  echo '{"has_conflicts": "unknown", "message": "GitHub still calculating merge status"}'
  exit 2

else
  log_success "No merge conflicts - PR is $mergeable ($merge_state)"
  
  report=$(jq -n \
    --arg pr_number "$pr_number" \
    --arg mergeable "$mergeable" \
    --arg merge_state "$merge_state" \
    '{
      has_conflicts: false,
      pr_number: $pr_number,
      mergeable: $mergeable,
      merge_state: $merge_state
    }')
  
  echo "$report" | jq '.'
  exit 0
fi

