#!/bin/bash
# Full remediation flow: validate → PR → CI → bugbot → merge → deploy
# Usage: full-remediation-flow.sh --task-id 42 --title "fix: description"
#
# This orchestrates the entire fix-to-deployment cycle with iteration:
# 1. Run local validation (fmt, clippy, test)
# 2. Create PR with fix
# 3. Poll GitHub Actions for completion
# 4. If CI fails → get logs, exit 1 (agent should fix and retry)
# 5. Trigger and check Bugbot
# 6. If Bugbot issues → exit 1 (agent should fix and retry)
# 7. Enable auto-merge and wait
# 8. Wait for ArgoCD sync + pod readiness
#
# Exit: 0 if fully deployed, 1 if any step fails (with details for retry)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"

# Parse arguments
task_id=""
title=""
body=""
repo_dir="${REPO_DIR:-/workspace/repo}"
repo="${GITHUB_REPO:-5dlabs/cto}"
argocd_app="${ARGOCD_APP:-controller}"
controller_ns="${CONTROLLER_NAMESPACE:-cto}"
controller_selector="${CONTROLLER_LABEL:-app=agent-controller}"
branch=""  # Will be set after PR creation

while [ $# -gt 0 ]; do
  case "$1" in
    --task-id) task_id="$2"; shift 2 ;;
    --title) title="$2"; shift 2 ;;
    --body) body="$2"; shift 2 ;;
    --repo-dir) repo_dir="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --argocd-app) argocd_app="$2"; shift 2 ;;
    --namespace) controller_ns="$2"; shift 2 ;;
    --selector) controller_selector="$2"; shift 2 ;;
    --branch) branch="$2"; shift 2 ;;  # For retries
    --pr-number) pr_number="$2"; shift 2 ;;  # For retries
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$task_id" ] || [ -z "$title" ]; then
  log_error "Required: --task-id and --title"
  exit 1
fi

export GITHUB_REPO="$repo"
export REPO_DIR="$repo_dir"
export WATCH_WORKSPACE="${WATCH_WORKSPACE:-/workspace/watch}"

cd "$repo_dir"

log_info "=========================================="
log_info "Starting Full Remediation Flow"
log_info "=========================================="
log_info "Task ID: $task_id"
log_info "Title: $title"
log_info "Repository: $repo"
log_info "ArgoCD App: $argocd_app"
log_info "=========================================="

# ============================================================================
# Phase 1: Local Validation
# ============================================================================
log_info ""
log_info "📋 PHASE 1: Local Validation"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/run-validation.sh" --repo-dir "$repo_dir" --fix; then
  log_error "Local validation failed - fix issues before proceeding"
  exit 1
fi

# ============================================================================
# Phase 2: Create PR (or use existing if retrying)
# ============================================================================
log_info ""
log_info "📝 PHASE 2: Create Pull Request"
log_info "----------------------------------------"

if [ -z "${pr_number:-}" ]; then
  pr_number=$("$SCRIPT_DIR/create-fix-pr.sh" \
    --task-id "$task_id" \
    --title "$title" \
    --body "$body" \
    --repo-dir "$repo_dir" \
    --repo "$repo")
  
  if [ -z "$pr_number" ]; then
    log_error "Failed to create PR"
    exit 1
  fi
  
  log_success "PR #$pr_number created"
else
  log_info "Using existing PR #$pr_number"
  # Push any new fixes
  git push origin HEAD 2>/dev/null || true
fi

# Get branch name from PR if not set
if [ -z "$branch" ]; then
  branch=$(gh pr view "$pr_number" --repo "$repo" --json headRefName -q '.headRefName')
fi

# ============================================================================
# Phase 3: Poll GitHub Actions for Completion
# ============================================================================
log_info ""
log_info "🔄 PHASE 3: Wait for GitHub Actions"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/poll-actions.sh" --branch "$branch" --pr-number "$pr_number" --repo "$repo" --timeout 1800; then
  log_error "GitHub Actions failed - see failure report above"
  log_info ""
  log_info "Failure details saved to: $WATCH_WORKSPACE/action-failures.json"
  log_info "Agent should analyze failures and push fixes, then retry"
  exit 1
fi

log_success "All GitHub Actions passed"

# ============================================================================
# Phase 4: Check for Merge Conflicts
# ============================================================================
log_info ""
log_info "🔀 PHASE 4: Check Merge Conflicts"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/check-conflicts.sh" --pr-number "$pr_number" --repo "$repo" --repo-dir "$repo_dir"; then
  log_error "Merge conflicts detected"
  log_info ""
  log_info "Conflict report saved to: $WATCH_WORKSPACE/merge-conflicts.json"
  log_info "Agent should:"
  log_info "  1. Fetch latest main: git fetch origin main"
  log_info "  2. Rebase: git rebase origin/main"
  log_info "  3. Resolve conflicts manually"
  log_info "  4. Push: git push --force-with-lease"
  log_info ""
  log_info "To retry after resolving, run:"
  log_info "  $0 --task-id $task_id --title \"$title\" --pr-number $pr_number --branch $branch"
  exit 1
fi

log_success "No merge conflicts"

# ============================================================================
# Phase 5: Trigger and Check Bugbot
# ============================================================================
log_info ""
log_info "🐛 PHASE 5: Check Bugbot"
log_info "----------------------------------------"

# Trigger bugbot if needed, wait for it, then check for issues
if ! "$SCRIPT_DIR/check-bugbot.sh" --pr-number "$pr_number" --repo "$repo" --trigger --wait; then
  log_warn "Bugbot found issues that need to be addressed"
  log_info ""
  log_info "Bugbot issues saved to: $WATCH_WORKSPACE/bugbot-issues.json"
  log_info "Agent should address issues and push fixes, then retry"
  log_info ""
  log_info "To retry after fixing, run:"
  log_info "  $0 --task-id $task_id --title \"$title\" --pr-number $pr_number --branch $branch"
  exit 1
fi

log_success "No Bugbot issues"

# ============================================================================
# Phase 6: Enable Auto-Merge and Wait
# ============================================================================
log_info ""
log_info "✅ PHASE 6: Merge Pull Request"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/merge-pr.sh" --pr-number "$pr_number" --repo "$repo" --wait --timeout 600; then
  log_error "Failed to merge PR"
  exit 1
fi

log_success "PR #$pr_number merged"

# ============================================================================
# Phase 7: Wait for Deployment
# ============================================================================
log_info ""
log_info "🚀 PHASE 7: Wait for Deployment"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/poll-deploy.sh" \
    --app "$argocd_app" \
    --namespace "$controller_ns" \
    --selector "$controller_selector" \
    --timeout 600; then
  log_error "Deployment verification failed"
  exit 1
fi

# ============================================================================
# Complete!
# ============================================================================
log_info ""
log_info "=========================================="
log_success "✅ FULL REMEDIATION FLOW COMPLETE"
log_info "=========================================="
log_info "PR #$pr_number merged and deployed"
log_info "Ready for next Monitor iteration"
log_info "=========================================="

# Clean up issue files on success
rm -f "$WATCH_WORKSPACE/action-failures.json" 2>/dev/null || true
rm -f "$WATCH_WORKSPACE/bugbot-issues.json" 2>/dev/null || true

exit 0

