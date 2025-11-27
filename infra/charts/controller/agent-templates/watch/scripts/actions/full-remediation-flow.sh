#!/bin/bash
# Full remediation flow: validate → PR → CI → merge → deploy
# Usage: full-remediation-flow.sh --task-id 42 --title "fix: description"
#
# This orchestrates the entire fix-to-deployment cycle:
# 1. Run local validation (fmt, clippy, test)
# 2. Create PR with fix
# 3. Wait for CI to pass
# 4. Check for bug-bot issues
# 5. Merge PR
# 6. Wait for ArgoCD sync + pod readiness
#
# Exit: 0 if fully deployed, 1 if any step fails

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
# Phase 2: Create PR
# ============================================================================
log_info ""
log_info "📝 PHASE 2: Create Pull Request"
log_info "----------------------------------------"

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

# ============================================================================
# Phase 3: Wait for CI
# ============================================================================
log_info ""
log_info "🔄 PHASE 3: Wait for CI Checks"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/poll-ci.sh" --pr-number "$pr_number" --repo "$repo" --timeout 1800; then
  log_error "CI checks failed"
  exit 1
fi

# ============================================================================
# Phase 4: Check Bug-Bot
# ============================================================================
log_info ""
log_info "🐛 PHASE 4: Check for Bug-Bot Issues"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/check-bugbot.sh" --pr-number "$pr_number" --repo "$repo" --wait; then
  log_warn "Bug-bot found issues - these need to be addressed"
  # In a full implementation, the agent would read bugbot-issues.json and make additional fixes
  # For now, we fail and let the outer loop retry
  exit 1
fi

# ============================================================================
# Phase 5: Merge PR
# ============================================================================
log_info ""
log_info "🔀 PHASE 5: Merge Pull Request"
log_info "----------------------------------------"

if ! "$SCRIPT_DIR/merge-pr.sh" --pr-number "$pr_number" --repo "$repo"; then
  log_error "Failed to merge PR"
  exit 1
fi

# ============================================================================
# Phase 6: Wait for Deployment
# ============================================================================
log_info ""
log_info "🚀 PHASE 6: Wait for Deployment"
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

exit 0

