#!/bin/bash
# Kind Cluster: Load CLI images into Kind node
# Usage: ./kind-load-cli-images.sh [claude|codex|cursor|factory|opencode|all]
#
# Note: Each image is ~12GB, loading takes 5-10 minutes per image
# For local testing, recommend loading only 1-2 images

set -euo pipefail

CLUSTER="${KIND_CLUSTER:-cto-dev}"
REGISTRY="${REGISTRY:-ghcr.io/5dlabs}"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if image exists locally
check_image() {
    local image=$1
    if docker image inspect "$image" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Pull image if not present
pull_image() {
    local image=$1
    if ! check_image "$image"; then
        log_info "Pulling $image (this may take a while)..."
        docker pull "$image"
    else
        log_info "Image $image already present locally"
    fi
}

# Load image into Kind
load_image() {
    local cli=$1
    local image="${REGISTRY}/${cli}:latest"
    
    log_info "Processing ${cli}..."
    
    # Pull if needed
    pull_image "$image"
    
    # Check if already loaded in Kind
    if docker exec "${CLUSTER}-control-plane" crictl images 2>/dev/null | grep -q "${REGISTRY}/${cli}"; then
        log_warn "${cli} already loaded in Kind cluster"
        return 0
    fi
    
    # Load into Kind
    log_info "Loading ${cli} into Kind cluster (this takes 5-10 minutes)..."
    local start_time=$(date +%s)
    
    kind load docker-image "$image" --name "$CLUSTER"
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    log_info "✅ Loaded ${cli} in ${duration}s"
}

# Show available images and sizes
show_status() {
    echo ""
    echo "=== Local Docker Images ==="
    docker images --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}" 2>/dev/null | \
        grep -E "(5dlabs/(claude|codex|cursor|factory|opencode)|REPOSITORY)" || \
        echo "No 5dlabs CLI images found locally"
    
    echo ""
    echo "=== Images in Kind Cluster ($CLUSTER) ==="
    docker exec "${CLUSTER}-control-plane" crictl images 2>/dev/null | \
        grep -E "(5dlabs|IMAGE)" || \
        echo "No 5dlabs images in Kind cluster"
    echo ""
}

# Show usage
usage() {
    echo "Usage: $0 [options] [cli1] [cli2] ..."
    echo ""
    echo "CLI options:"
    echo "  claude    Load Claude CLI image (~12GB)"
    echo "  codex     Load Codex CLI image (~12GB)"
    echo "  cursor    Load Cursor CLI image (~12GB)"
    echo "  factory   Load Factory CLI image (~12GB)"
    echo "  opencode  Load OpenCode CLI image (~12GB)"
    echo "  all       Load all CLI images (~60GB total)"
    echo ""
    echo "Other options:"
    echo "  --status  Show current image status"
    echo "  --help    Show this help"
    echo ""
    echo "Environment variables:"
    echo "  KIND_CLUSTER  Kind cluster name (default: cto-dev)"
    echo "  REGISTRY      Container registry (default: ghcr.io/5dlabs)"
    echo ""
    echo "Examples:"
    echo "  $0 claude                 # Load only Claude CLI"
    echo "  $0 claude codex           # Load Claude and Codex"
    echo "  $0 --status               # Check what's loaded"
    echo ""
    echo "⚠️  Note: Each image is ~12GB. Loading all images requires ~60GB."
    echo "    For local testing, load only what you need."
}

# Main
main() {
    if [[ $# -eq 0 ]]; then
        usage
        exit 0
    fi
    
    # Check Kind cluster exists
    if ! kind get clusters 2>/dev/null | grep -q "^${CLUSTER}$"; then
        log_error "Kind cluster '${CLUSTER}' not found"
        log_info "Available clusters: $(kind get clusters 2>/dev/null | tr '\n' ' ')"
        exit 1
    fi
    
    local clis=()
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            claude|codex|cursor|factory|opencode)
                clis+=("$1")
                shift
                ;;
            all)
                clis=(claude codex cursor factory opencode)
                shift
                ;;
            --status)
                show_status
                exit 0
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    if [[ ${#clis[@]} -eq 0 ]]; then
        log_error "No CLI specified"
        usage
        exit 1
    fi
    
    # Load each CLI
    for cli in "${clis[@]}"; do
        load_image "$cli"
    done
    
    echo ""
    log_info "Done! Verify with: $0 --status"
}

main "$@"



