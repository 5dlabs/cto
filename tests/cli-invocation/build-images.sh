#!/usr/bin/env bash
# =============================================================================
# Build Docker Images for CLI Integration Tests
# =============================================================================
#
# Usage:
#   ./build-images.sh all       # Build all images
#   ./build-images.sh sidecar   # Build just sidecar
#   ./build-images.sh claude    # Build just Claude CLI image
#
# =============================================================================

set -euo pipefail

cd "$(dirname "$0")/../.."  # Go to repo root

build_sidecar() {
    echo "=== Building Linear Sidecar (status-sync) ==="
    docker build \
        --target local \
        -f infra/images/linear-sidecar/Dockerfile \
        -t cto-linear-sidecar:local \
        .
    echo "✓ Sidecar built: cto-linear-sidecar:local"
}

build_claude() {
    echo "=== Building Claude CLI Image ==="
    docker build \
        -f infra/images/claude/Dockerfile \
        -t cto-claude:local \
        .
    echo "✓ Claude built: cto-claude:local"
}

build_factory() {
    echo "=== Building Factory CLI Image ==="
    docker build \
        -f infra/images/factory/Dockerfile \
        -t cto-factory:local \
        .
    echo "✓ Factory built: cto-factory:local"
}

build_codex() {
    echo "=== Building Codex CLI Image ==="
    docker build \
        -f infra/images/codex/Dockerfile \
        -t cto-codex:local \
        .
    echo "✓ Codex built: cto-codex:local"
}

build_all() {
    build_sidecar
    build_claude
    # Add more as needed
    # build_factory
    # build_codex
}

# Parse arguments
case "${1:-all}" in
    all)
        build_all
        ;;
    sidecar)
        build_sidecar
        ;;
    claude)
        build_claude
        ;;
    factory)
        build_factory
        ;;
    codex)
        build_codex
        ;;
    *)
        echo "Usage: $0 {all|sidecar|claude|factory|codex}"
        exit 1
        ;;
esac

echo ""
echo "=== Build Complete ==="
docker images | grep "cto-.*:local" | head -10
