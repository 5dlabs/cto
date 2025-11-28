#!/bin/bash
# Git operations for E2E Watch Remediation Agent

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Default configuration
GIT_USER_EMAIL="${GIT_USER_EMAIL:-automation@5dlabs.io}"
GIT_USER_NAME="${GIT_USER_NAME:-5DLabs Automation}"

# ============================================================================
# Repository Setup
# ============================================================================

# Clone or update repository
# Usage: git_setup_repo REPO_URL TARGET_DIR
git_setup_repo() {
  local repo_url="$1"
  local target_dir="$2"
  
  if [ -d "$target_dir/.git" ]; then
    log_info "Repository exists, fetching latest..."
    cd "$target_dir" || { log_error "Failed to cd to $target_dir"; return 1; }
    git fetch origin main
    git checkout main
    git reset --hard origin/main
  else
    log_step "Cloning repository: $repo_url"
    git clone "https://github.com/$repo_url.git" "$target_dir"
    cd "$target_dir" || { log_error "Failed to cd to $target_dir"; return 1; }
  fi
  
  # Configure git
  git config user.email "$GIT_USER_EMAIL"
  git config user.name "$GIT_USER_NAME"
  
  log_success "Repository ready at $target_dir"
}

# Create a new branch from main
# Usage: git_create_branch BRANCH_NAME
git_create_branch() {
  local branch_name="$1"
  
  log_step "Creating branch: $branch_name"
  
  git checkout main
  git pull origin main
  git checkout -b "$branch_name"
  
  log_success "On branch: $branch_name"
}

# Generate a unique branch name for a fix
# Usage: git_fix_branch_name TASK_ID [PREFIX]
git_fix_branch_name() {
  local task_id="$1"
  local prefix="${2:-fix/watch}"
  
  echo "${prefix}-t${task_id}-$(date +%s)"
}

# ============================================================================
# Commit Operations
# ============================================================================

# Stage all changes
# Usage: git_stage_all
git_stage_all() {
  git add -A
}

# Commit with message
# Usage: git_commit MESSAGE
git_commit() {
  local message="$1"
  
  git_stage_all
  
  if git diff --cached --quiet; then
    log_warn "No changes to commit"
    return 1
  fi
  
  git commit -m "$message"
  log_success "Committed: $message"
}

# Push branch to origin
# Usage: git_push [BRANCH_NAME]
git_push() {
  local branch="${1:-$(git branch --show-current)}"
  
  log_step "Pushing branch: $branch"
  git push origin "$branch"
  log_success "Pushed to origin/$branch"
}

# Get current branch name
# Usage: git_current_branch
git_current_branch() {
  git branch --show-current
}

# Get current commit SHA
# Usage: git_current_sha [SHORT]
git_current_sha() {
  local short="${1:-}"
  
  if [ "$short" = "short" ]; then
    git rev-parse --short HEAD
  else
    git rev-parse HEAD
  fi
}

# ============================================================================
# Validation Operations
# ============================================================================

# Check if working directory is clean
# Usage: git_is_clean
git_is_clean() {
  git diff --quiet && git diff --cached --quiet
}

# Get list of changed files
# Usage: git_changed_files
git_changed_files() {
  git diff --name-only HEAD~1 2>/dev/null || git diff --name-only --cached
}

# ============================================================================
# Full Fix Workflow
# ============================================================================

# Complete fix workflow: branch, commit, push
# Usage: git_fix_and_push TASK_ID COMMIT_MESSAGE
git_fix_and_push() {
  local task_id="$1"
  local message="$2"
  
  local branch
  branch=$(git_fix_branch_name "$task_id")
  
  git_create_branch "$branch"
  git_commit "$message"
  git_push "$branch"
  
  echo "$branch"
}

