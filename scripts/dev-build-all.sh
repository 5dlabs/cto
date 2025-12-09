#!/usr/bin/env bash
# =============================================================================
# dev-build-all.sh - Build all core images with caching and smart parallelism
# =============================================================================
# Builds controller, healer, tools, pm, and research images and pushes them
# to the in-cluster registry for fast local development.
#
# Features:
# - BuildKit cache mounts for ~80% faster rebuilds
# - Staggered parallel builds to avoid resource contention on M1
# - Shared Cargo registry cache across all images
#
# Prerequisites:
#   1. Local registry must be deployed: ./scripts/dev-load.sh --setup
#   2. Docker must trust the insecure registry
#   3. Docker BuildKit enabled (default in Docker Desktop)
#
# Usage:
#   ./scripts/dev-build-all.sh              # Build sequentially (safest)
#   ./scripts/dev-build-all.sh --staggered  # Staggered parallel (recommended)
#   ./scripts/dev-build-all.sh --helm       # Build and update Helm
#
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Core images to build (these are the main platform components)
CORE_IMAGES=(
    "controller"
    "healer"
    "tools"
    "pm"
    "research"
)

# Configuration
NAMESPACE="${NAMESPACE:-cto}"
DEV_TAG="${DEV_TAG:-dev-local}"
REGISTRY_NAMESPACE="${REGISTRY_NAMESPACE:-registry}"

# Build mode
BUILD_MODE="sequential"  # sequential, staggered, or parallel
STAGGER_DELAY=30  # seconds between staggered builds
UPDATE_HELM=false
DRY_RUN=false
CUSTOM_TAG=""

print_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build all core platform images with BuildKit caching for fast rebuilds."
    echo ""
    echo "Core images built:"
    for img in "${CORE_IMAGES[@]}"; do
        echo "  - $img"
    done
    echo ""
    echo "Build Modes:"
    echo "  (default)       Sequential builds - safest, uses shared cache"
    echo "  --staggered     Staggered parallel - starts builds 30s apart"
    echo "  --parallel      Full parallel - may fail on resource-limited machines"
    echo ""
    echo "Options:"
    echo "  --staggered     Start builds staggered (recommended for M1)"
    echo "  --parallel      Full parallel builds (may overwhelm M1)"
    echo "  --helm          After building, update Helm to use dev registry"
    echo "  --tag TAG       Use custom tag (default: dev-local)"
    echo "  --dry-run       Show what would be done without executing"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Caching:"
    echo "  First build: ~5-10 min per image (downloads + compiles everything)"
    echo "  Subsequent:  ~1-3 min per image (uses cached dependencies)"
    echo ""
    echo "Examples:"
    echo "  $0                      # Sequential (safest)"
    echo "  $0 --staggered          # Staggered parallel (recommended)"
    echo "  $0 --staggered --helm   # Build + deploy"
    echo ""
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --staggered)
            BUILD_MODE="staggered"
            shift
            ;;
        --parallel)
            BUILD_MODE="parallel"
            shift
            ;;
        --helm)
            UPDATE_HELM=true
            shift
            ;;
        --tag)
            CUSTOM_TAG="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            print_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            print_help
            exit 1
            ;;
    esac
done

TAG="${CUSTOM_TAG:-$DEV_TAG}"

# Check if registry is deployed
check_registry() {
    if ! kubectl get deployment registry -n "$REGISTRY_NAMESPACE" &>/dev/null; then
        echo -e "${RED}Error: Local registry not found${NC}"
        echo ""
        echo "Run setup first:"
        echo "  ./scripts/dev-load.sh --setup"
        echo ""
        exit 1
    fi
    
    if ! kubectl get deployment registry -n "$REGISTRY_NAMESPACE" -o jsonpath='{.status.readyReplicas}' | grep -q "1"; then
        echo -e "${YELLOW}Warning: Registry deployment not ready${NC}"
        echo "Waiting for registry..."
        kubectl rollout status deployment/registry -n "$REGISTRY_NAMESPACE" --timeout=60s
    fi
}

get_registry_url() {
    local node_ip
    node_ip=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}')
    echo "${node_ip}:30500"
}

# Build a single image (runs in subshell for isolation)
build_image() {
    local name="$1"
    local registry_url="$2"
    local tag="$3"
    local log_file="${ROOT_DIR}/.build-${name}.log"
    
    echo -e "${CYAN}━━━ Building: ${name} ━━━${NC}"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  [DRY RUN] ./scripts/dev-load.sh --no-deploy --tag ${tag} ${name}"
        return 0
    fi
    
    # Run build in subshell, capture output
    if (
        export DOCKER_BUILDKIT=1
        "${SCRIPT_DIR}/dev-load.sh" --no-deploy --tag "${tag}" "${name}"
    ) > "$log_file" 2>&1; then
        echo -e "  ${GREEN}✓ ${name} complete${NC}"
        rm -f "$log_file"
        return 0
    else
        echo -e "  ${RED}✗ ${name} failed${NC}"
        echo -e "  ${YELLOW}See log: ${log_file}${NC}"
        return 1
    fi
}

# Build with staggered starts
build_staggered() {
    local registry_url="$1"
    local tag="$2"
    local pids=()
    local names=()
    local failed=0
    
    echo -e "${GREEN}▶ Building images with staggered starts (${STAGGER_DELAY}s delay)...${NC}"
    echo ""
    
    for i in "${!CORE_IMAGES[@]}"; do
        local name="${CORE_IMAGES[$i]}"
        names+=("$name")
        
        # Start build in background
        (build_image "$name" "$registry_url" "$tag") &
        pids+=($!)
        
        # Stagger next build (except for last one)
        if [[ $i -lt $((${#CORE_IMAGES[@]} - 1)) ]]; then
            echo -e "  ${YELLOW}Waiting ${STAGGER_DELAY}s before next build...${NC}"
            sleep "$STAGGER_DELAY"
        fi
    done
    
    echo ""
    echo -e "${BLUE}Waiting for all builds to complete...${NC}"
    
    # Wait for all builds
    for i in "${!pids[@]}"; do
        if ! wait "${pids[$i]}"; then
            ((failed++))
        fi
    done
    
    return $failed
}

# Build sequentially (safest, still fast with cache)
build_sequential() {
    local registry_url="$1"
    local tag="$2"
    local failed=0
    
    echo -e "${GREEN}▶ Building images sequentially (with shared cache)...${NC}"
    echo ""
    
    for name in "${CORE_IMAGES[@]}"; do
        if ! build_image "$name" "$registry_url" "$tag"; then
            ((failed++))
        fi
        echo ""
    done
    
    return $failed
}

# Build fully parallel (may fail on M1)
build_parallel() {
    local registry_url="$1"
    local tag="$2"
    local pids=()
    local failed=0
    
    echo -e "${GREEN}▶ Building all images in parallel...${NC}"
    echo -e "${YELLOW}  Warning: May overwhelm Docker on M1 Macs${NC}"
    echo ""
    
    for name in "${CORE_IMAGES[@]}"; do
        (build_image "$name" "$registry_url" "$tag") &
        pids+=($!)
    done
    
    # Wait for all builds
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            ((failed++))
        fi
    done
    
    return $failed
}

# =============================================================================
# Main
# =============================================================================

echo ""
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Dev Build All - Core Platform Images${NC}"
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo ""

# Check registry
if [[ "$DRY_RUN" != "true" ]]; then
    check_registry
fi

REGISTRY_URL=$(get_registry_url 2>/dev/null || echo "192.168.1.77:30500")

echo -e "  ${YELLOW}Registry:${NC}    ${REGISTRY_URL}"
echo -e "  ${YELLOW}Tag:${NC}         ${TAG}"
echo -e "  ${YELLOW}Build Mode:${NC}  ${BUILD_MODE}"
echo -e "  ${YELLOW}Update Helm:${NC} ${UPDATE_HELM}"
echo -e "  ${YELLOW}Images:${NC}      ${CORE_IMAGES[*]}"
echo ""
echo -e "  ${CYAN}Tip: First build downloads deps (~5-10min). Subsequent builds use cache (~1-3min).${NC}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}[DRY RUN MODE]${NC}"
    echo ""
fi

START_TIME=$(date +%s)
FAILED=0

case "$BUILD_MODE" in
    staggered)
        build_staggered "$REGISTRY_URL" "$TAG" || FAILED=$?
        ;;
    parallel)
        build_parallel "$REGISTRY_URL" "$TAG" || FAILED=$?
        ;;
    *)
        build_sequential "$REGISTRY_URL" "$TAG" || FAILED=$?
        ;;
esac

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
DURATION_MIN=$((DURATION / 60))
DURATION_SEC=$((DURATION % 60))

# Summary
echo ""
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Build Summary${NC}"
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Duration: ${DURATION_MIN}m ${DURATION_SEC}s"
echo ""

if [[ "$FAILED" -gt 0 ]]; then
    echo -e "${RED}${FAILED} image(s) failed to build${NC}"
    echo ""
    echo "Check logs in: ${ROOT_DIR}/.build-*.log"
    exit 1
fi

# Update Helm if requested
if [[ "$UPDATE_HELM" == "true" && "$DRY_RUN" != "true" ]]; then
    echo -e "${GREEN}▶ Updating Helm to use dev registry...${NC}"
    echo ""
    
    HELM_CMD="helm upgrade cto ${ROOT_DIR}/infra/charts/cto -n ${NAMESPACE} --reuse-values --set global.devRegistry.enabled=true --set global.devRegistry.url=${REGISTRY_URL} --set global.devRegistry.tag=${TAG}"
    
    echo "  $ ${HELM_CMD}"
    if eval "${HELM_CMD}"; then
        echo -e "  ${GREEN}✓ Helm updated${NC}"
    else
        echo -e "  ${RED}✗ Helm update failed${NC}"
        exit 1
    fi
    
    echo ""
    echo "  Waiting for rollouts..."
    for name in controller healer tools pm; do
        kubectl rollout status deployment/${name} -n "${NAMESPACE}" --timeout=120s 2>/dev/null || true
    done
fi

echo ""
echo -e "${GREEN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ All core images built successfully!${NC}"
echo -e "${GREEN}══════════════════════════════════════════════════════════════════${NC}"
echo ""

if [[ "$UPDATE_HELM" != "true" ]]; then
    echo -e "  ${YELLOW}Next step: Enable dev registry in Helm${NC}"
    echo ""
    echo "    helm upgrade cto ./infra/charts/cto -n cto --reuse-values \\"
    echo "      --set global.devRegistry.enabled=true \\"
    echo "      --set global.devRegistry.url=${REGISTRY_URL}"
    echo ""
fi

echo -e "  ${YELLOW}To revert to production images:${NC}"
echo "    helm upgrade cto ./infra/charts/cto -n cto --reuse-values \\"
echo "      --set global.devRegistry.enabled=false"
echo ""
