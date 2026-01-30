#!/usr/bin/env bash
# =============================================================================
# Build All CLI Images Locally
#
# Builds the image hierarchy for Apple Silicon (arm64):
#   1. Runtime base image (cto-runtime:local)
#   2. All CLI images in parallel (cto-claude:local, cto-codex:local, etc.)
#
# Usage:
#   ./tests/cli-invocation/build-images.sh          # Build all
#   ./tests/cli-invocation/build-images.sh runtime  # Build runtime only
#   ./tests/cli-invocation/build-images.sh claude   # Build runtime + claude
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

log() { echo -e "${BLUE}[BUILD]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# All CLI images
CLIS=(claude codex cursor opencode gemini factory code dexter)

# Build runtime base image (simplified local version)
build_runtime() {
    log "Building runtime base image..."
    docker build \
        --platform linux/arm64 \
        -t cto-runtime:local \
        -f "$SCRIPT_DIR/dockerfiles/runtime.Dockerfile" \
        "$SCRIPT_DIR/dockerfiles"
    success "cto-runtime:local built"
}

# Build a single CLI image (using local Dockerfiles)
build_cli() {
    local cli="$1"
    log "Building $cli image..."
    docker build \
        --platform linux/arm64 \
        -t "cto-$cli:local" \
        -f "$SCRIPT_DIR/dockerfiles/$cli.Dockerfile" \
        "$SCRIPT_DIR/dockerfiles"
    success "cto-$cli:local built"
}

# Build all CLI images in parallel
build_all_clis() {
    log "Building all CLI images in parallel..."
    
    local pids=()
    for cli in "${CLIS[@]}"; do
        (build_cli "$cli") &
        pids+=($!)
    done
    
    # Wait for all builds
    local failed=0
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            ((failed++))
        fi
    done
    
    if [[ $failed -gt 0 ]]; then
        error "$failed CLI builds failed"
        return 1
    fi
    
    success "All CLI images built"
}

# Build linear-sidecar from source
build_sidecar() {
    log "Building linear-sidecar image (compiling from source)..."
    docker build \
        --platform linux/arm64 \
        -t cto-linear-sidecar:local \
        -f "$SCRIPT_DIR/dockerfiles/linear-sidecar.Dockerfile" \
        "$PROJECT_ROOT"
    success "cto-linear-sidecar:local built"
}

# Show all local images
show_images() {
    echo ""
    log "Local CTO images:"
    docker images | grep "cto-" | grep ":local" || echo "  (none found)"
}

# Main
main() {
    cd "$PROJECT_ROOT"
    
    echo ""
    echo "=============================================="
    echo "  Build CTO Images Locally (arm64)"
    echo "=============================================="
    echo ""
    
    local target="${1:-all}"
    
    case "$target" in
        runtime)
            build_runtime
            ;;
        sidecar)
            build_sidecar
            ;;
        claude|codex|cursor|opencode|gemini|factory|code|dexter)
            # Ensure runtime exists
            if ! docker image inspect cto-runtime:local >/dev/null 2>&1; then
                build_runtime
            fi
            build_cli "$target"
            ;;
        all)
            build_runtime
            build_all_clis
            build_sidecar
            ;;
        *)
            echo "Usage: $0 [runtime|sidecar|claude|codex|cursor|opencode|gemini|factory|code|dexter|all]"
            exit 1
            ;;
    esac
    
    show_images
}

main "$@"
