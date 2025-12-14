#!/usr/bin/env bash
# =============================================================================
# Setup Cluster BuildKit Builder
# =============================================================================
# Configures docker buildx to use the BuildKit instance running on the cluster.
# This enables fast remote builds with persistent cargo cache.
#
# Usage:
#   ./scripts/setup-cluster-builder.sh [--check|--remove|--auto]
#
# Requirements:
#   - Docker with buildx support (Docker Desktop, OrbStack, etc.)
#   - Network access to the cluster nodes
#   - BuildKit deployed on cluster (ArgoCD syncs this automatically)
#   - kubectl configured with cluster access (for --auto mode)
#
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration - can be overridden via environment variables
BUILDER_NAME="${BUILDER_NAME:-cluster-builder}"
BUILDKIT_PORT="${BUILDKIT_PORT:-30502}"
REGISTRY_PORT="${REGISTRY_PORT:-30500}"

# These can be auto-detected or manually specified
BUILDKIT_HOST="${BUILDKIT_HOST:-}"
REGISTRY_URL="${REGISTRY_URL:-}"

# =============================================================================
# Auto-detection Functions
# =============================================================================

detect_buildkit_host() {
    # Try to find the node where BuildKit is running
    if ! command -v kubectl &>/dev/null; then
        log_warn "kubectl not found, cannot auto-detect BuildKit host"
        return 1
    fi

    # First, try to find the node where the buildkit pod is scheduled
    local buildkit_node
    buildkit_node=$(kubectl get pods -n cto -l app=buildkit -o jsonpath='{.items[0].spec.nodeName}' 2>/dev/null || true)
    
    if [[ -n "$buildkit_node" ]]; then
        # Get the IP of that specific node
        local node_ip
        node_ip=$(kubectl get node "$buildkit_node" -o jsonpath='{.status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null || true)
        if [[ -n "$node_ip" ]]; then
            echo "$node_ip"
            return 0
        fi
    fi
    
    # Fallback: Get any worker node IP (non-control-plane)
    local worker_ip
    worker_ip=$(kubectl get nodes -l '!node-role.kubernetes.io/control-plane' -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null || true)
    
    if [[ -n "$worker_ip" ]]; then
        echo "$worker_ip"
        return 0
    fi
    
    # Last fallback: Get first node IP
    local first_ip
    first_ip=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null || true)
    
    if [[ -n "$first_ip" ]]; then
        echo "$first_ip"
        return 0
    fi
    
    return 1
}

detect_registry_url() {
    # Registry typically runs on the same node as BuildKit or any node
    local host="${1:-}"
    
    if [[ -z "$host" ]]; then
        host=$(detect_buildkit_host) || return 1
    fi
    
    # Validate that host looks like a valid IP or hostname (not a placeholder)
    # Must start with a digit (IP) or letter (hostname), not special chars like '('
    if [[ ! "$host" =~ ^[0-9a-zA-Z] ]]; then
        return 1
    fi
    
    echo "${host}:${REGISTRY_PORT}"
}

show_cluster_info() {
    echo -e "${CYAN}=== Cluster Information ===${NC}"
    echo ""
    
    if ! command -v kubectl &>/dev/null; then
        log_warn "kubectl not found"
        return 1
    fi
    
    # Show nodes
    echo "Nodes:"
    kubectl get nodes -o wide 2>/dev/null | head -10 || echo "  (cannot get nodes)"
    echo ""
    
    # Show BuildKit pod status
    echo "BuildKit Status:"
    kubectl get pods -n cto -l app=buildkit -o wide 2>/dev/null || echo "  (BuildKit not deployed or namespace 'cto' doesn't exist)"
    echo ""
    
    # Show BuildKit service
    echo "BuildKit Service:"
    kubectl get svc -n cto buildkit 2>/dev/null || echo "  (BuildKit service not found)"
    echo ""
}

# =============================================================================
# Core Functions
# =============================================================================

check_connectivity() {
    local host="$1"
    local port="$2"
    
    log_info "Checking connectivity to BuildKit at ${host}:${port}..."
    
    # Try nc first (works on most systems)
    if command -v nc &>/dev/null; then
        if nc -zv -w 3 "${host}" "${port}" 2>&1 | grep -qE "succeeded|open"; then
            log_success "BuildKit is reachable"
            return 0
        fi
    fi
    
    # Fallback to /dev/tcp (bash built-in)
    if timeout 3 bash -c "echo >/dev/tcp/${host}/${port}" 2>/dev/null; then
        log_success "BuildKit is reachable"
        return 0
    fi
    
    log_error "Cannot reach BuildKit at ${host}:${port}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Check BuildKit is deployed: kubectl get pods -n cto -l app=buildkit"
    echo "  2. Check service exists: kubectl get svc -n cto buildkit"
    echo "  3. Verify network access to cluster nodes"
    echo "  4. Try manual override: BUILDKIT_HOST=<ip> $0"
    return 1
}

check_registry() {
    local url="$1"
    
    log_info "Checking connectivity to registry at ${url}..."
    if curl -s -m 5 "http://${url}/v2/_catalog" >/dev/null 2>&1; then
        log_success "Registry is reachable"
        return 0
    else
        log_warn "Cannot reach registry at ${url} (this is OK if not using local registry)"
        return 1
    fi
}

setup_builder() {
    local host="$1"
    local port="$2"
    local registry="$3"
    
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
        "tcp://${host}:${port}"
    
    # Set as default builder
    log_info "Setting '${BUILDER_NAME}' as default builder..."
    docker buildx use "${BUILDER_NAME}"
    
    # Bootstrap the builder
    log_info "Bootstrapping builder..."
    docker buildx inspect --bootstrap "${BUILDER_NAME}"
    
    log_success "Cluster builder configured successfully!"
    echo ""
    echo -e "${CYAN}Configuration:${NC}"
    echo "  BuildKit: tcp://${host}:${port}"
    echo "  Registry: ${registry}"
    echo ""
    echo -e "${CYAN}Usage:${NC}"
    echo "  # Build and push to local registry"
    echo "  docker buildx build --push -t ${registry}/cto/myimage:latest ."
    echo ""
    echo "  # Build for linux/amd64 (cross-compile from ARM Mac)"
    echo "  docker buildx build --platform linux/amd64 --push -t ${registry}/cto/myimage:latest ."
    echo ""
    echo "  # Switch back to local builder"
    echo "  docker buildx use default"
    echo ""
    echo "  # Use Tilt for continuous development"
    echo "  tilt up"
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
    echo -e "${CYAN}=== BuildKit Cluster Builder Status ===${NC}"
    echo ""
    
    # Show cluster info first
    show_cluster_info
    
    # Determine hosts
    local host="${BUILDKIT_HOST:-}"
    local registry="${REGISTRY_URL:-}"
    
    if [[ -z "$host" ]]; then
        host=$(detect_buildkit_host) || host="(unknown)"
    fi
    
    if [[ -z "$registry" ]]; then
        registry=$(detect_registry_url "$host") || registry="(unknown)"
    fi
    
    echo -e "${CYAN}Detected Configuration:${NC}"
    echo "  BuildKit Host: ${host}:${BUILDKIT_PORT}"
    echo "  Registry URL:  ${registry}"
    echo ""
    
    # Check connectivity
    if [[ "$host" != "(unknown)" ]]; then
        check_connectivity "$host" "${BUILDKIT_PORT}" || true
        check_registry "$registry" || true
    fi
    echo ""
    
    # Show current builder
    echo "Docker Builders:"
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

show_help() {
    echo "Setup Cluster BuildKit Builder"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  (default)     Auto-detect cluster and setup builder"
    echo "  --check       Show current status and cluster info"
    echo "  --remove      Remove the cluster builder"
    echo "  --help        Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  BUILDKIT_HOST   BuildKit host IP (auto-detected from kubectl if not set)"
    echo "  BUILDKIT_PORT   BuildKit NodePort (default: 30502)"
    echo "  REGISTRY_URL    Registry URL (auto-detected if not set)"
    echo "  REGISTRY_PORT   Registry NodePort (default: 30500)"
    echo "  BUILDER_NAME    Docker buildx builder name (default: cluster-builder)"
    echo ""
    echo "Examples:"
    echo "  # Auto-detect and setup (requires kubectl access to cluster)"
    echo "  $0"
    echo ""
    echo "  # Manual setup for Latitude cluster"
    echo "  BUILDKIT_HOST=147.75.x.x $0"
    echo ""
    echo "  # Check status"
    echo "  $0 --check"
    echo ""
    echo "  # Remove builder"
    echo "  $0 --remove"
}

# =============================================================================
# Main
# =============================================================================

main() {
    local cmd="${1:-setup}"
    
    case "$cmd" in
        --check|check|status)
            show_status
            ;;
        --remove|remove)
            remove_builder
            ;;
        --help|help|-h)
            show_help
            ;;
        setup|*)
            # Auto-detect or use environment variables
            local host="${BUILDKIT_HOST:-}"
            local registry="${REGISTRY_URL:-}"
            
            if [[ -z "$host" ]]; then
                log_info "Auto-detecting BuildKit host from cluster..."
                host=$(detect_buildkit_host) || {
                    log_error "Could not auto-detect BuildKit host"
                    echo ""
                    echo "Please specify manually:"
                    echo "  BUILDKIT_HOST=<node-ip> $0"
                    echo ""
                    echo "Or check cluster status:"
                    echo "  $0 --check"
                    exit 1
                }
                log_success "Detected BuildKit host: ${host}"
            fi
            
            if [[ -z "$registry" ]]; then
                registry="${host}:${REGISTRY_PORT}"
            fi
            
            # Check connectivity and setup
            check_connectivity "$host" "${BUILDKIT_PORT}"
            check_registry "$registry" || true
            setup_builder "$host" "${BUILDKIT_PORT}" "$registry"
            ;;
    esac
}

main "$@"
