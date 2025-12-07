#!/usr/bin/env bash
# =============================================================================
# kind-build-all.sh - Build all Docker images for local Kind development
# =============================================================================
# Builds ARM64 images from local source and loads them into Kind cluster.
# No GHCR pulls required - everything is built fresh from the current branch.
#
# Usage:
#   ./scripts/kind-build-all.sh              # Build and load everything
#   ./scripts/kind-build-all.sh --build-only # Build only, don't load to Kind
#   ./scripts/kind-build-all.sh --list       # List what will be built
#   ./scripts/kind-build-all.sh runtime      # Build specific image(s)
#   ./scripts/kind-build-all.sh claude codex # Build multiple specific images
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-cto-dev}"
IMAGE_TAG="${IMAGE_TAG:-kind-local}"
REGISTRY="${REGISTRY:-ghcr.io/5dlabs}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build order matters - dependencies first
declare -A BUILD_ORDER=(
    [1]="runtime"      # Base image for CLI containers
    [2]="controller"   # CTO controller
    [3]="claude"       # Claude CLI
    [4]="codex"        # Codex CLI  
    [5]="cursor"       # Cursor CLI
    [6]="factory"      # Factory CLI
    [7]="gemini"       # Gemini CLI
    [8]="opencode"     # OpenCode CLI
    [9]="tools"        # Tools MCP server
    [10]="openmemory"  # OpenMemory service
    [11]="healer"      # Healer service
)

# Image descriptions
declare -A IMAGE_DESC=(
    [runtime]="Base runtime image with dev tools (Ubuntu 24.04 + Node + Python + Go + Rust)"
    [controller]="CTO Controller - CodeRun reconciliation"
    [claude]="Claude CLI container (Anthropic)"
    [codex]="Codex CLI container (OpenAI)"
    [cursor]="Cursor CLI container"
    [factory]="Factory CLI container"
    [gemini]="Gemini CLI container (Google)"
    [opencode]="OpenCode CLI container"
    [tools]="Tools MCP server (all MCP tools)"
    [openmemory]="OpenMemory service (context/memory management)"
    [healer]="Healer service (CI remediation)"
)

# Estimated sizes (compressed)
declare -A IMAGE_SIZES=(
    [runtime]="~2.5GB"
    [controller]="~100MB"
    [claude]="~500MB (on runtime)"
    [codex]="~500MB (on runtime)"
    [cursor]="~500MB (on runtime)"
    [factory]="~500MB (on runtime)"
    [gemini]="~500MB (on runtime)"
    [opencode]="~500MB (on runtime)"
    [tools]="~1.5GB"
    [openmemory]="~500MB"
    [healer]="~500MB"
)

log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_header() { echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; echo -e "${BLUE}  $1${NC}"; echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"; }

list_images() {
    log_header "Images to Build"
    echo "Build order (dependencies first):"
    echo ""
    for i in $(seq 1 ${#BUILD_ORDER[@]}); do
        img="${BUILD_ORDER[$i]}"
        printf "  %2d. %-12s %s\n" "$i" "$img" "${IMAGE_SIZES[$img]}"
        printf "      └─ %s\n" "${IMAGE_DESC[$img]}"
    done
    echo ""
    echo "Total estimated disk space: ~8-10GB (compressed)"
    echo "Kind loaded images will be larger (~15-20GB)"
}

check_prerequisites() {
    log_header "Checking Prerequisites"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker Desktop."
        exit 1
    fi
    log_success "Docker found: $(docker --version | head -1)"
    
    # Check Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon not running. Please start Docker Desktop."
        exit 1
    fi
    log_success "Docker daemon running"
    
    # Check Kind
    if ! command -v kind &> /dev/null; then
        log_error "Kind not found. Install with: brew install kind"
        exit 1
    fi
    log_success "Kind found: $(kind --version)"
    
    # Check Kind cluster exists
    if ! kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
        log_warn "Kind cluster '${KIND_CLUSTER_NAME}' not found"
        read -p "Create it now? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            kind create cluster --name "${KIND_CLUSTER_NAME}"
        else
            log_error "Kind cluster required. Create with: kind create cluster --name ${KIND_CLUSTER_NAME}"
            exit 1
        fi
    fi
    log_success "Kind cluster '${KIND_CLUSTER_NAME}' exists"
    
    # Check architecture
    local arch
    arch=$(uname -m)
    if [[ "$arch" != "arm64" ]]; then
        log_warn "Non-ARM architecture detected: $arch"
        log_warn "Images will be built for $arch"
    fi
    log_success "Architecture: $arch"
    
    # Check disk space
    local available_gb
    available_gb=$(df -g "${PROJECT_ROOT}" | awk 'NR==2 {print $4}')
    if [[ "$available_gb" -lt 30 ]]; then
        log_warn "Only ${available_gb}GB available. Recommend at least 30GB."
    else
        log_success "Disk space: ${available_gb}GB available"
    fi
}

build_image() {
    local name="$1"
    local image_dir="${PROJECT_ROOT}/infra/images/${name}"
    local full_tag="${REGISTRY}/${name}:${IMAGE_TAG}"
    local dockerfile="Dockerfile"
    
    # Use .kind Dockerfile if it exists
    if [[ -f "${image_dir}/Dockerfile.kind" ]]; then
        dockerfile="Dockerfile.kind"
    fi
    
    if [[ ! -d "$image_dir" ]]; then
        log_error "Image directory not found: $image_dir"
        return 1
    fi
    
    log_info "Building ${name} (${IMAGE_SIZES[$name]:-unknown size})..."
    log_info "  Dockerfile: ${image_dir}/${dockerfile}"
    log_info "  Tag: ${full_tag}"
    
    local build_args=""
    
    # Special handling for CLI images - they need the runtime base
    case "$name" in
        claude|codex|cursor|factory|gemini|opencode)
            build_args="--build-arg BASE_IMAGE=${REGISTRY}/runtime:${IMAGE_TAG}"
            ;;
        controller)
            # Controller builds from project root with different context
            docker build \
                -f "${image_dir}/${dockerfile}" \
                -t "${full_tag}" \
                --platform linux/arm64 \
                "${PROJECT_ROOT}" 2>&1 | while read -r line; do echo "    $line"; done
            log_success "Built ${name}"
            return 0
            ;;
    esac
    
    # Standard build
    # shellcheck disable=SC2086
    docker build \
        ${build_args} \
        -f "${image_dir}/${dockerfile}" \
        -t "${full_tag}" \
        --platform linux/arm64 \
        "${image_dir}" 2>&1 | while read -r line; do echo "    $line"; done
    
    log_success "Built ${name}"
}

load_image() {
    local name="$1"
    local full_tag="${REGISTRY}/${name}:${IMAGE_TAG}"
    
    log_info "Loading ${name} into Kind cluster..."
    
    kind load docker-image "${full_tag}" --name "${KIND_CLUSTER_NAME}" 2>&1 | while read -r line; do echo "    $line"; done
    
    log_success "Loaded ${name} into Kind"
}

build_single() {
    local name="$1"
    local start_time
    start_time=$(date +%s)
    
    log_header "Building: ${name}"
    echo "${IMAGE_DESC[$name]:-No description}"
    echo ""
    
    build_image "$name"
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    log_success "Build completed in ${duration}s"
}

load_single() {
    local name="$1"
    load_image "$name"
}

build_all() {
    local build_only="${1:-false}"
    local start_time
    start_time=$(date +%s)
    
    log_header "Building All Images"
    echo "This will build ${#BUILD_ORDER[@]} images for ARM64"
    echo "Estimated time: 15-30 minutes (depending on cache)"
    echo ""
    
    local built=0
    local failed=0
    
    for i in $(seq 1 ${#BUILD_ORDER[@]}); do
        local img="${BUILD_ORDER[$i]}"
        echo ""
        log_info "[$i/${#BUILD_ORDER[@]}] ${img}"
        
        if build_image "$img"; then
            ((built++))
            
            if [[ "$build_only" != "true" ]]; then
                load_image "$img"
            fi
        else
            log_error "Failed to build ${img}"
            ((failed++))
        fi
    done
    
    local end_time
    end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    echo ""
    log_header "Build Summary"
    echo "  Built:  ${built}/${#BUILD_ORDER[@]}"
    echo "  Failed: ${failed}"
    echo "  Time:   ${total_duration}s ($((total_duration / 60))m $((total_duration % 60))s)"
    
    if [[ "$build_only" != "true" ]]; then
        echo ""
        echo "Images loaded into Kind cluster '${KIND_CLUSTER_NAME}'"
        echo "Use imagePullPolicy: Never in deployments"
    fi
}

show_status() {
    log_header "Current Image Status"
    
    echo "Local Docker images:"
    for i in $(seq 1 ${#BUILD_ORDER[@]}); do
        local img="${BUILD_ORDER[$i]}"
        local full_tag="${REGISTRY}/${img}:${IMAGE_TAG}"
        if docker image inspect "$full_tag" &>/dev/null; then
            local size
            size=$(docker image inspect "$full_tag" --format '{{.Size}}' | numfmt --to=iec-i)
            printf "  ${GREEN}✓${NC} %-12s %s\n" "$img" "$size"
        else
            printf "  ${RED}✗${NC} %-12s (not built)\n" "$img"
        fi
    done
    
    echo ""
    echo "Images in Kind cluster '${KIND_CLUSTER_NAME}':"
    if kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
        docker exec "${KIND_CLUSTER_NAME}-control-plane" crictl images 2>/dev/null | grep "5dlabs" | while read -r line; do
            echo "  $line"
        done
    else
        echo "  (cluster not running)"
    fi
}

# =============================================================================
# Main
# =============================================================================

main() {
    cd "${PROJECT_ROOT}"
    
    case "${1:-}" in
        --list|-l)
            list_images
            ;;
        --status|-s)
            show_status
            ;;
        --build-only|-b)
            check_prerequisites
            build_all "true"
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS] [IMAGE...]"
            echo ""
            echo "Options:"
            echo "  --list, -l        List all images that will be built"
            echo "  --status, -s      Show current build/load status"
            echo "  --build-only, -b  Build images but don't load into Kind"
            echo "  --help, -h        Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                     # Build and load all images"
            echo "  $0 runtime claude      # Build specific images"
            echo "  $0 --build-only        # Build all without loading"
            echo ""
            echo "Environment variables:"
            echo "  KIND_CLUSTER_NAME  Kind cluster name (default: cto-dev)"
            echo "  IMAGE_TAG          Image tag (default: kind-local)"
            echo "  REGISTRY           Registry prefix (default: ghcr.io/5dlabs)"
            ;;
        "")
            check_prerequisites
            build_all
            ;;
        *)
            # Build specific images
            check_prerequisites
            for img in "$@"; do
                if [[ -v "IMAGE_DESC[$img]" ]]; then
                    build_single "$img"
                    load_single "$img"
                else
                    log_error "Unknown image: $img"
                    log_info "Available: ${!IMAGE_DESC[*]}"
                    exit 1
                fi
            done
            ;;
    esac
}

main "$@"



