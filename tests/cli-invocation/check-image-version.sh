#!/usr/bin/env bash
# =============================================================================
# Check Image Version
#
# Displays the git commit and branch that a Docker image was built from.
# Useful for verifying image-to-source alignment.
#
# Usage:
#   ./check-image-version.sh                        # Check all cto-* images
#   ./check-image-version.sh cto-claude:local       # Check specific image
#   ./check-image-version.sh --current              # Show current git commit
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get current git info
get_current_commit() {
    local commit=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")
    local branch=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    local dirty=$(git -C "$PROJECT_ROOT" diff --quiet 2>/dev/null && echo "" || echo "-dirty")
    echo "commit=${commit}${dirty} branch=${branch}"
}

# Get image labels
check_image() {
    local image="$1"
    
    if ! docker image inspect "$image" >/dev/null 2>&1; then
        echo -e "${RED}✗ ${image}${NC} - not found"
        return 1
    fi
    
    local revision=$(docker image inspect "$image" --format '{{index .Config.Labels "org.opencontainers.image.revision"}}' 2>/dev/null || echo "unlabeled")
    local branch=$(docker image inspect "$image" --format '{{index .Config.Labels "org.opencontainers.image.ref.name"}}' 2>/dev/null || echo "unlabeled")
    local created=$(docker image inspect "$image" --format '{{.Created}}' 2>/dev/null | cut -d'T' -f1,2 | tr 'T' ' ')
    
    if [[ "$revision" == "" || "$revision" == "unlabeled" ]]; then
        echo -e "${YELLOW}? ${image}${NC} - no commit label (built: ${created})"
    else
        echo -e "${GREEN}✓ ${image}${NC} - commit=${revision} branch=${branch} (built: ${created})"
    fi
}

# Main
main() {
    if [[ "${1:-}" == "--current" ]]; then
        echo "Current source: $(get_current_commit)"
        exit 0
    fi
    
    echo ""
    echo "=============================================="
    echo "  Image Version Check"
    echo "=============================================="
    echo ""
    echo "Current source: $(get_current_commit)"
    echo ""
    
    if [[ $# -gt 0 && "$1" != "--current" ]]; then
        # Check specific image(s)
        for image in "$@"; do
            check_image "$image"
        done
    else
        # Check all cto-* images
        echo "Local CTO images:"
        local images=$(docker images --format '{{.Repository}}:{{.Tag}}' | grep "^cto-" | sort)
        if [[ -z "$images" ]]; then
            echo "  (no cto-* images found)"
        else
            for image in $images; do
                check_image "$image"
            done
        fi
    fi
    
    echo ""
    
    # Show build info if available
    if [[ -f "$SCRIPT_DIR/.build-info" ]]; then
        echo "Recent builds from .build-info:"
        tail -5 "$SCRIPT_DIR/.build-info" | while read line; do
            echo "  $line"
        done
    fi
}

main "$@"
