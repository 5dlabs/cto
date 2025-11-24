#!/bin/bash
# Quick E2E Reset - Minimal script for fast iteration
#
# This script is fully portable - works for any developer checking out the repo.
# The template is stored in testing/cto-parallel-test-template/ and the test
# repo is created in a sibling directory to your workspace.
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
NS="agent-platform"
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
<<<<<<< HEAD
      echo ""
      echo "  ⚠️  Failed to delete repository!"
      echo ""
      echo "  This usually means you need one of:"
      echo "    1. The 'delete_repo' scope: gh auth refresh -h github.com -s delete_repo"
      echo "    2. Admin permissions on organization repos (contact organization owner)"
      echo ""
      echo "  Manual deletion: https://github.com/$REPO/settings"
      echo ""
      echo "  NOTE: When you recreate this repo, make sure your GitHub account has"
      echo "        admin/delete permissions to avoid this issue in the future."
      echo ""
      exit 1
    fi
    echo "  ✓ Repository deleted"
  fi
  
  # Create new repository
  # NOTE: The account creating this repo should have admin rights to delete it later
  echo "  Creating fresh repository..."
  if gh repo create $REPO --private --clone=false; then
    echo "  ✓ Repository created"
  else
    echo "  ✗ Failed to create repository"
    exit 1
  fi
  
  # Reset local repo
  rm -rf $LOCAL
  mkdir -p $LOCAL
  
  # Use submodule template if available, otherwise create minimal structure
  if [ -d "$TEMPLATE" ] && [ -f "$TEMPLATE/cto-config.json" ]; then
    echo "  Using submodule template..."
    # Ensure submodule is up to date
    cd "$PROJECT_ROOT"
    git submodule update --init --recursive testing/cto-parallel-test 2>/dev/null || true
    
    # Copy from submodule (excluding .git)
    rsync -av --exclude='.git' "$TEMPLATE/" "$LOCAL/" || \
      cp -r "$TEMPLATE"/* "$LOCAL/" 2>/dev/null || true
    
    # Copy hidden files except .git
    find "$TEMPLATE" -maxdepth 1 -name ".*" ! -name ".git" ! -name "." ! -name ".." -exec cp -r {} "$LOCAL/" \; 2>/dev/null || true
  else
    echo "  Creating minimal structure..."
    cd $LOCAL
=======
      echo "  ⚠️  Failed to delete repository. You may need to grant delete_repo permission:"
      echo "     Run: gh auth refresh -h github.com -s delete_repo"
      echo "  Or delete it manually at: https://github.com/$REPO/settings"
      exit 1
    fi
  fi
  
  # Create new repository
  echo "  Creating fresh repository..."
  gh repo create $REPO --private --clone=false
  
  # Reset local repo
  rm -rf $LOCAL
  mkdir -p $LOCAL
  
  # Use submodule template if available, otherwise create minimal structure
  if [ -d "$TEMPLATE" ] && [ -f "$TEMPLATE/cto-config.json" ]; then
    echo "  Using submodule template..."
    # Ensure submodule is up to date
    cd "$PROJECT_ROOT"
    git submodule update --init --recursive testing/cto-parallel-test 2>/dev/null || true
    
    # Copy from submodule (excluding .git)
    rsync -av --exclude='.git' "$TEMPLATE/" "$LOCAL/" || \
      cp -r "$TEMPLATE"/* "$LOCAL/" 2>/dev/null || true
    
    # Copy hidden files except .git
    find "$TEMPLATE" -maxdepth 1 -name ".*" ! -name ".git" ! -name "." ! -name ".." -exec cp -r {} "$LOCAL/" \; 2>/dev/null || true
  else
    echo "  Creating minimal structure..."
<<<<<<< HEAD
    rm -rf "$LOCAL"
    mkdir -p "$LOCAL"
    cd "$LOCAL"
>>>>>>> dc3c56da (fix: implement ephemeral E2E template workflow to prevent submodule crosstalk)
=======
    cd $LOCAL
>>>>>>> 6b5ca940 (feat: add portable E2E test template to repository)
    
    # Minimal setup
    cat > cto-config.json <<'EOF'
{
  "version": "1.0.0",
  "project": "cto-parallel-test"
}
EOF
    
    mkdir -p docs/.taskmaster/docs
    cat > docs/.taskmaster/docs/test.txt <<'EOF'
# Test PRD

Build a simple test application.
EOF
<<<<<<< HEAD
<<<<<<< HEAD
  fi
  
  # Initialize git and push
  cd $LOCAL
  git init
  git add .
  git commit -m "Reset" || git commit --allow-empty -m "Reset"
  git branch -M main
  git remote add origin git@github.com:${REPO}.git 2>/dev/null || \
    git remote set-url origin git@github.com:${REPO}.git
  git push -u origin main --force
  
=======
    
    # Initialize git and push
    git init
    git add .
    git commit -m "Reset" || git commit --allow-empty -m "Reset"
    git branch -M main
    git remote add origin git@github.com:${REPO}.git 2>/dev/null || \
      git remote set-url origin git@github.com:${REPO}.git
    git push -u origin main --force
  fi
  
>>>>>>> dc3c56da (fix: implement ephemeral E2E template workflow to prevent submodule crosstalk)
=======
  fi
  
  # Initialize git and push
  cd $LOCAL
  git init
  git add .
  git commit -m "Reset" || git commit --allow-empty -m "Reset"
  git branch -M main
  git remote add origin git@github.com:${REPO}.git 2>/dev/null || \
    git remote set-url origin git@github.com:${REPO}.git
  git push -u origin main --force
  
>>>>>>> 6b5ca940 (feat: add portable E2E test template to repository)
  echo "✓ GitHub repository reset"
fi

echo ""
echo "✅ Reset complete!"
echo ""
echo "Run test: cto play --task-id <id>"
echo "Monitor: kubectl logs -f -l workflow -n agent-platform"

<<<<<<< HEAD
<<<<<<< HEAD

=======
>>>>>>> dc3c56da (fix: implement ephemeral E2E template workflow to prevent submodule crosstalk)
=======

>>>>>>> 6b5ca940 (feat: add portable E2E test template to repository)
