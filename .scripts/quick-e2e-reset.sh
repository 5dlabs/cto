#!/bin/bash
# Quick E2E Reset - Minimal script for fast iteration
#
# Template workflow: init in template dir -> push to GitHub -> delete template -> clone to test location
#
# IMPORTANT: This script requires the GitHub account to have:
# 1. Admin permissions on the organization repository
# 2. The 'delete_repo' scope for the GitHub CLI
#
# If you get a 403 error when deleting the repo:
#   gh auth refresh -h github.com -s delete_repo
#
# If that doesn't work, you may need organization admin to grant you
# admin rights on repos in the 5dlabs organization.

set -euo pipefail

echo "🔄 Quick E2E Environment Reset"
echo "=============================="

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NS="cto"
REPO="5dlabs/cto-parallel-test"
LOCAL="/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test"
TEMPLATE="${PROJECT_ROOT}/testing/cto-parallel-test"

# 1. Kubernetes cleanup (do this first)
echo "→ Cleaning Kubernetes resources..."
kubectl delete workflows --all -n $NS --force --grace-period=0 2>/dev/null || true
kubectl delete pods --all -n $NS --force --grace-period=0 2>/dev/null || true
kubectl delete configmaps -n $NS --force --grace-period=0 \
    $(kubectl get cm -n $NS -o name | grep -E "play-|test-|coderun-|docsrun-" || true) 2>/dev/null || true
kubectl delete pvc -n $NS --force --grace-period=0 \
    $(kubectl get pvc -n $NS -o name | grep -E "workspace-play-|workspace-test-" || true) 2>/dev/null || true

# 2. GitHub repo reset (only if --github flag is passed)
if [[ "${1:-}" == "--github" ]]; then
  echo "→ Resetting GitHub repository..."
  
  # Check if repo exists and delete it
  if gh repo view $REPO >/dev/null 2>&1; then
    echo "  Deleting existing repository..."
    if ! gh repo delete $REPO --yes 2>&1; then
      echo "  ⚠️  Failed to delete repository. You may need to grant delete_repo permission:"
      echo "     Run: gh auth refresh -h github.com -s delete_repo"
      echo "  Or delete it manually at: https://github.com/$REPO/settings"
      exit 1
    fi
  fi
  
  # Create new repository
  echo "  Creating fresh repository..."
  gh repo create $REPO --private --clone=false
  
  # Use template if available, otherwise create minimal structure
  if [ -d "$TEMPLATE" ] && [ -f "$TEMPLATE/cto-config.json" ]; then
    echo "  Using template from ${TEMPLATE}..."
    
    # Step 1: Initialize git in the template directory
    cd "$TEMPLATE"
    rm -rf .git
    git init
    git add .
    git commit -m "Reset" || git commit --allow-empty -m "Reset"
    git branch -M main
    git remote add origin git@github.com:${REPO}.git 2>/dev/null || \
      git remote set-url origin git@github.com:${REPO}.git
    
    # Step 2: Push to GitHub
    echo "  Pushing template to GitHub..."
    git push -u origin main --force
    
    # Step 3: Delete the template directory (ephemeral)
    echo "  Cleaning up template directory..."
    cd "$PROJECT_ROOT"
    rm -rf "$TEMPLATE"
    
    # Step 4: Clone from GitHub to test location
    echo "  Cloning from GitHub to test location..."
    rm -rf "$LOCAL"
    git clone git@github.com:${REPO}.git "$LOCAL"
    
  else
    echo "  Creating minimal structure..."
    rm -rf "$LOCAL"
    mkdir -p "$LOCAL"
    cd "$LOCAL"
    
    # Minimal setup
    cat > cto-config.json <<'EOF'
{
  "version": "1.0.0",
  "project": "cto-parallel-test"
}
EOF
    
    mkdir -p docs/.tasks/docs
    cat > docs/.tasks/docs/test.txt <<'EOF'
# Test PRD

Build a simple test application.
EOF
    
    # Initialize git and push
    git init
    git add .
    git commit -m "Reset" || git commit --allow-empty -m "Reset"
    git branch -M main
    git remote add origin git@github.com:${REPO}.git 2>/dev/null || \
      git remote set-url origin git@github.com:${REPO}.git
    git push -u origin main --force
  fi
  
  echo "✓ GitHub repository reset"
fi

echo ""
echo "✅ Reset complete!"
echo ""
echo "Run test: cto play --task-id <id>"
echo "Monitor: kubectl logs -f -l workflow -n cto"
