#!/usr/bin/env bash
# =============================================================================
# Setup Cluster BuildKit Builder
# =============================================================================
# Configures docker buildx to use the BuildKit instance running on the cluster.
# This enables fast remote builds with persistent cargo cache.
#
# Usage:
#   ./scripts/setup-cluster-builder.sh [--check|--remove]
#
# Requirements:
#   - Docker with buildx support (Docker Desktop, OrbStack, etc.)
#   - Network access to the cluster (192.168.1.x)
#   - BuildKit deployed on cluster (helm upgrade cto ./infra/charts/cto)
#
set -euo pipefail

# Configuration
BUILDER_NAME="cluster-builder"
BUILDKIT_HOST="${BUILDKIT_HOST:-192.168.1.72}"
BUILDKIT_PORT="${BUILDKIT_PORT:-30501}"
REGISTRY_URL="${REGISTRY_URL:-192.168.1.72:30500}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_connectivity() {
    log_info "Checking connectivity to BuildKit at ${BUILDKIT_HOST}:${BUILDKIT_PORT}..."
    if nc -zv -w 3 "${BUILDKIT_HOST}" "${BUILDKIT_PORT}" 2>&1 | grep -q "succeeded"; then
        log_success "BuildKit is reachable"
        return 0
    else
        log_error "Cannot reach BuildKit at ${BUILDKIT_HOST}:${BUILDKIT_PORT}"
        echo "Make sure BuildKit is deployed: helm upgrade cto ./infra/charts/cto --set buildkit.enabled=true"
        return 1
    fi
}

check_registry() {
    log_info "Checking connectivity to registry at ${REGISTRY_URL}..."
    if curl -s -m 5 "http://${REGISTRY_URL}/v2/_catalog" >/dev/null 2>&1; then
        log_success "Registry is reachable"
        return 0
    else
        log_warn "Cannot reach registry at ${REGISTRY_URL}"
        return 1
    fi
}

setup_builder() {
    log_info "Setting up cluster builder..."
    
    # Check if builder already exists
    if docker buildx inspect "${BUILDER_NAME}" >/dev/null 2>&1; then
        log_warn "Builder '${BUILDER_NAME}' already exists, removing..."
        docker buildx rm "${BUILDER_NAME}" 2>/dev/null || true
    fi
    
    # Create the remote builder
    log_info "Creating remote builder '${BUILDER_NAME}'..."
    docker buildx create \
        --name "${BUILDER_NAME}" \
        --driver remote \
        "tcp://${BUILDKIT_HOST}:${BUILDKIT_PORT}"
    
    # Set as default builder
    log_info "Setting '${BUILDER_NAME}' as default builder..."
    docker buildx use "${BUILDER_NAME}"
    
    # Bootstrap the builder
    log_info "Bootstrapping builder..."
    docker buildx inspect --bootstrap "${BUILDER_NAME}"
    
    log_success "Cluster builder configured successfully!"
    echo ""
    echo "Usage:"
    echo "  # Build and push to local registry"
    echo "  docker buildx build --push -t ${REGISTRY_URL}/cto/myimage:latest ."
    echo ""
    echo "  # Build for linux/amd64 (cross-compile)"
    echo "  docker buildx build --platform linux/amd64 --push -t ${REGISTRY_URL}/cto/myimage:latest ."
    echo ""
    echo "  # Switch back to local builder"
    echo "  docker buildx use default"
}

remove_builder() {
    log_info "Removing cluster builder..."
    if docker buildx inspect "${BUILDER_NAME}" >/dev/null 2>&1; then
        docker buildx rm "${BUILDER_NAME}"
        log_success "Builder '${BUILDER_NAME}' removed"
    else
        log_warn "Builder '${BUILDER_NAME}' does not exist"
    fi
    
    # Switch to default builder
    docker buildx use default 2>/dev/null || true
}

show_status() {
    echo "=== BuildKit Cluster Builder Status ==="
    echo ""
    
    # Check connectivity
    check_connectivity || true
    check_registry || true
    echo ""
    
    # Show current builder
    echo "Current builder:"
    docker buildx ls 2>/dev/null | head -10 || echo "  (docker buildx not available)"
    echo ""
    
    # Check if our builder is active
    if docker buildx inspect "${BUILDER_NAME}" >/dev/null 2>&1; then
        log_success "Cluster builder '${BUILDER_NAME}' is configured"
        echo ""
        docker buildx inspect "${BUILDER_NAME}" 2>/dev/null | head -20
    else
        log_warn "Cluster builder '${BUILDER_NAME}' is not configured"
        echo "Run: $0 to set it up"
    fi
}

# Main
case "${1:-setup}" in
    --check|check|status)
        show_status
        ;;
    --remove|remove)
        remove_builder
        ;;
    --help|help|-h)
        echo "Usage: $0 [--check|--remove|--help]"
        echo ""
        echo "Commands:"
        echo "  (default)   Setup the cluster builder"
        echo "  --check     Show current status"
        echo "  --remove    Remove the cluster builder"
        echo ""
        echo "Environment variables:"
        echo "  BUILDKIT_HOST  BuildKit host (default: 192.168.1.72)"
        echo "  BUILDKIT_PORT  BuildKit port (default: 30501)"
        echo "  REGISTRY_URL   Registry URL (default: 192.168.1.72:30500)"
        ;;
    *)
        check_connectivity
        check_registry || true
        setup_builder
        ;;
esac
