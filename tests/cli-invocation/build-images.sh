#!/usr/bin/env bash
# =============================================================================
# Build All CLI Images Locally
#
# Builds the image hierarchy for Apple Silicon (arm64):
#   1. Runtime base image (cto-runtime:local)
#   2. All CLI images in parallel (cto-claude:local, cto-codex:local, etc.)
#
# Images are tagged with both :local and :<commit-hash> for traceability.
# A .build-info file is created to track which commit each image was built from.
#
# Usage:
#   ./tests/cli-invocation/build-images.sh          # Build all
#   ./tests/cli-invocation/build-images.sh runtime  # Build runtime only
#   ./tests/cli-invocation/build-images.sh claude   # Build runtime + claude
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Get git commit info for image tagging
GIT_COMMIT=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
GIT_DIRTY=$(git -C "$PROJECT_ROOT" diff --quiet 2>/dev/null && echo "" || echo "-dirty")
BUILD_TAG="${GIT_COMMIT}${GIT_DIRTY}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[BUILD]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Record build info for traceability
record_build_info() {
    local image="$1"
    local build_info_file="$SCRIPT_DIR/.build-info"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    # Append to build info file
    echo "${image}:local -> commit=${GIT_COMMIT} branch=${GIT_BRANCH} dirty=${GIT_DIRTY:-no} built=${timestamp}" >> "$build_info_file"
}

# All CLI images
CLIS=(claude codex cursor opencode gemini factory code dexter)

# Build runtime base image (simplified local version)
build_runtime() {
    log "Building runtime base image (commit: ${BUILD_TAG})..."
    docker build \
        --platform linux/arm64 \
        --build-arg GIT_COMMIT="$GIT_COMMIT" \
        --build-arg GIT_BRANCH="$GIT_BRANCH" \
        -t "cto-runtime:local" \
        -t "cto-runtime:${BUILD_TAG}" \
        -f "$SCRIPT_DIR/dockerfiles/runtime.Dockerfile" \
        "$SCRIPT_DIR/dockerfiles"
    record_build_info "cto-runtime"
    success "cto-runtime:local (${BUILD_TAG}) built"
}

# Build a single CLI image (using local Dockerfiles)
build_cli() {
    local cli="$1"
    log "Building $cli image (commit: ${BUILD_TAG})..."
    docker build \
        --platform linux/arm64 \
        --build-arg GIT_COMMIT="$GIT_COMMIT" \
        --build-arg GIT_BRANCH="$GIT_BRANCH" \
        -t "cto-$cli:local" \
        -t "cto-$cli:${BUILD_TAG}" \
        -f "$SCRIPT_DIR/dockerfiles/$cli.Dockerfile" \
        "$SCRIPT_DIR/dockerfiles"
    record_build_info "cto-$cli"
    success "cto-$cli:local (${BUILD_TAG}) built"
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
    log "Building linear-sidecar image (commit: ${BUILD_TAG})..."
    docker build \
        --platform linux/arm64 \
        --build-arg GIT_COMMIT="$GIT_COMMIT" \
        --build-arg GIT_BRANCH="$GIT_BRANCH" \
        -t "cto-linear-sidecar:local" \
        -t "cto-linear-sidecar:${BUILD_TAG}" \
        -f "$SCRIPT_DIR/dockerfiles/linear-sidecar.Dockerfile" \
        "$PROJECT_ROOT"
    record_build_info "cto-linear-sidecar"
    success "cto-linear-sidecar:local (${BUILD_TAG}) built"
}

# Show all local images with their tags
show_images() {
    echo ""
    log "Local CTO images:"
    docker images --format "table {{.Repository}}:{{.Tag}}\t{{.ID}}\t{{.CreatedAt}}" | grep "cto-" || echo "  (none found)"
    
    # Show build info if available
    if [[ -f "$SCRIPT_DIR/.build-info" ]]; then
        echo ""
        log "Recent builds (from .build-info):"
        tail -10 "$SCRIPT_DIR/.build-info" | while read line; do
            echo "  $line"
        done
    fi
}

# Main
main() {
    cd "$PROJECT_ROOT"
    
    echo ""
    echo "=============================================="
    echo "  Build CTO Images Locally (arm64)"
    echo "=============================================="
    echo ""
    log "Git commit: ${GIT_COMMIT}"
    log "Git branch: ${GIT_BRANCH}"
    if [[ -n "$GIT_DIRTY" ]]; then
        warn "Working tree has uncommitted changes (images tagged as ${BUILD_TAG})"
    fi
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
