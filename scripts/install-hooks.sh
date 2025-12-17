#!/usr/bin/env bash
# =============================================================================
# install-hooks.sh - Install git hooks for local CI with Claude auto-fix
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"
SRC_HOOKS_DIR="$SCRIPT_DIR/hooks"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

install_hooks() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║      Installing Git Hooks with Claude Auto-Fix               ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Check if we're in a git repo
    if [ ! -d "$HOOKS_DIR" ]; then
        log_error "Not a git repository or .git/hooks not found"
        exit 1
    fi
    
    # Install pre-push hook
    local src="$SRC_HOOKS_DIR/pre-push"
    local dst="$HOOKS_DIR/pre-push"
    
    if [ ! -f "$src" ]; then
        log_error "Source hook not found: $src"
        exit 1
    fi
    
    if [ -f "$dst" ]; then
        log_warning "Existing pre-push hook found"
        log_info "Backing up to: $dst.backup"
        cp "$dst" "$dst.backup"
    fi
    
    cp "$src" "$dst"
    chmod +x "$dst"
    log_success "Installed pre-push hook"
    
    # Make local-ci.sh executable
    chmod +x "$SCRIPT_DIR/local-ci.sh"
    log_success "Made local-ci.sh executable"
    
    echo ""
    log_success "Installation complete!"
    echo ""
    echo "The pre-push hook will now run CI checks before every push."
    echo ""
    echo "Usage:"
    echo "  git push                 # Normal push (runs checks + auto-fix)"
    echo "  git push --no-verify     # Bypass all hooks (emergency)"
    echo ""
}

remove_hooks() {
    local dst="$HOOKS_DIR/pre-push"
    
    if [ -f "$dst" ]; then
        rm "$dst"
        log_success "Removed pre-push hook"
    else
        log_info "No pre-push hook found"
    fi
}

case "${1:-}" in
    --remove|-r)
        remove_hooks
        ;;
    --help|-h)
        echo "Usage: install-hooks.sh [--remove]"
        ;;
    *)
        install_hooks
        ;;
esac
















