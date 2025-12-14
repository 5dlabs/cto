#!/usr/bin/env bash
# =============================================================================
# dev-push.sh - Fast iteration cycle for development
# =============================================================================
# Build and push images directly to the Talos cluster, bypassing CI.
#
# Usage:
#   ./scripts/dev-push.sh controller    # Build and deploy controller
#   ./scripts/dev-push.sh tools         # Build and deploy tools
#   ./scripts/dev-push.sh healer        # Build and deploy healer
#   ./scripts/dev-push.sh --list        # List available images
#
# Options:
#   --no-deploy    Build and push only, skip deployment rollout
#   --tag TAG      Use custom tag (default: dev-$USER)
#   --dry-run      Show what would be done without executing
#
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
REGISTRY="${REGISTRY:-ghcr.io}"
ORG="${ORG:-5dlabs}"
NAMESPACE="${NAMESPACE:-cto}"
DEV_TAG="${DEV_TAG:-dev-$(whoami)}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Image configurations: name -> dockerfile:context:deployment
declare -A IMAGES=(
    ["controller"]="infra/images/controller/Dockerfile.kind:.:controller"
    ["tools"]="infra/images/tools/Dockerfile.kind:.:tools"
    ["healer"]="infra/images/healer/Dockerfile.kind:.:healer"
    ["pm"]="infra/images/pm-server/Dockerfile.build:.:pm"
    ["openmemory"]="infra/images/openmemory/Dockerfile:infra/images/openmemory:openmemory"
    ["tweakcn"]="infra/images/tweakcn/Dockerfile:.:tweakcn"
    ["runtime"]="infra/images/runtime/Dockerfile:.:runtime"
    ["claude"]="infra/images/claude/Dockerfile:.:claude-code"
    ["opencode"]="infra/images/opencode/Dockerfile.local:.:opencode"
    ["dexter"]="infra/images/dexter/Dockerfile.local:.:dexter"
)

# Parse arguments
NO_DEPLOY=false
DRY_RUN=false
CUSTOM_TAG=""
IMAGE_NAME=""

print_help() {
    echo "Usage: $0 [OPTIONS] IMAGE"
    echo ""
    echo "Build and push images directly to your Talos cluster for fast iteration."
    echo ""
    echo "Arguments:"
    echo "  IMAGE           Image to build (controller, tools, healer, pm, etc.)"
    echo ""
    echo "Options:"
    echo "  --list          List available images"
    echo "  --no-deploy     Build and push only, skip deployment rollout"
    echo "  --tag TAG       Use custom tag (default: dev-\$USER)"
    echo "  --dry-run       Show what would be done without executing"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 controller              # Build, push, and deploy controller"
    echo "  $0 --no-deploy tools       # Build and push tools only"
    echo "  $0 --tag feature-x healer  # Use custom tag 'feature-x'"
    echo ""
    echo "Environment Variables:"
    echo "  REGISTRY        Container registry (default: ghcr.io)"
    echo "  ORG             Organization/namespace (default: 5dlabs)"
    echo "  NAMESPACE       Kubernetes namespace (default: cto)"
    echo "  DEV_TAG         Default tag (default: dev-\$USER)"
}

list_images() {
    echo -e "${BLUE}Available images:${NC}"
    echo ""
    printf "  %-12s %-50s\n" "NAME" "DOCKERFILE"
    printf "  %-12s %-50s\n" "----" "----------"
    for name in "${!IMAGES[@]}"; do
        IFS=':' read -r dockerfile context deployment <<< "${IMAGES[$name]}"
        printf "  %-12s %-50s\n" "$name" "$dockerfile"
    done | sort
    echo ""
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --list)
            list_images
            exit 0
            ;;
        --no-deploy)
            NO_DEPLOY=true
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
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            print_help
            exit 1
            ;;
        *)
            IMAGE_NAME="$1"
            shift
            ;;
    esac
done

# Check if image name is provided
if [[ -z "$IMAGE_NAME" ]]; then
    echo -e "${RED}Error: No image specified${NC}"
    echo ""
    print_help
    exit 1
fi

# Check if image exists
if [[ -z "${IMAGES[$IMAGE_NAME]:-}" ]]; then
    echo -e "${RED}Error: Unknown image '$IMAGE_NAME'${NC}"
    echo ""
    list_images
    exit 1
fi

# Use custom tag if provided
TAG="${CUSTOM_TAG:-$DEV_TAG}"

# Parse image config
IFS=':' read -r DOCKERFILE CONTEXT DEPLOYMENT <<< "${IMAGES[$IMAGE_NAME]}"

# Determine full image name
FULL_IMAGE="${REGISTRY}/${ORG}/${IMAGE_NAME}:${TAG}"

echo ""
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Dev Push: ${IMAGE_NAME}${NC}"
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${YELLOW}Image:${NC}      ${FULL_IMAGE}"
echo -e "  ${YELLOW}Dockerfile:${NC} ${DOCKERFILE}"
echo -e "  ${YELLOW}Context:${NC}    ${CONTEXT}"
echo -e "  ${YELLOW}Deployment:${NC} ${DEPLOYMENT}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}[DRY RUN] Would execute the following:${NC}"
    echo ""
fi

# Step 1: Build the image
echo -e "${GREEN}▶ Step 1/3: Building image...${NC}"
BUILD_CMD="docker build --platform linux/amd64 -t ${FULL_IMAGE} -f ${ROOT_DIR}/${DOCKERFILE} ${ROOT_DIR}/${CONTEXT}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo "  $ ${BUILD_CMD}"
else
    echo "  $ ${BUILD_CMD}"
    cd "${ROOT_DIR}"
    eval "${BUILD_CMD}"
    echo -e "  ${GREEN}✓ Build complete${NC}"
fi
echo ""

# Step 2: Push to registry
echo -e "${GREEN}▶ Step 2/3: Pushing to registry...${NC}"
PUSH_CMD="docker push ${FULL_IMAGE}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo "  $ ${PUSH_CMD}"
else
    echo "  $ ${PUSH_CMD}"
    eval "${PUSH_CMD}"
    echo -e "  ${GREEN}✓ Push complete${NC}"
fi
echo ""

# Step 3: Deploy (unless --no-deploy)
if [[ "$NO_DEPLOY" == "false" ]]; then
    echo -e "${GREEN}▶ Step 3/3: Rolling out deployment...${NC}"
    
    # Patch the deployment to use the new image and force a rollout
    PATCH_CMD="kubectl set image deployment/${DEPLOYMENT} ${DEPLOYMENT}=${FULL_IMAGE} -n ${NAMESPACE}"
    ROLLOUT_CMD="kubectl rollout restart deployment/${DEPLOYMENT} -n ${NAMESPACE}"
    WAIT_CMD="kubectl rollout status deployment/${DEPLOYMENT} -n ${NAMESPACE} --timeout=120s"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  $ ${PATCH_CMD}"
        echo "  $ ${ROLLOUT_CMD}"
        echo "  $ ${WAIT_CMD}"
    else
        echo "  $ ${PATCH_CMD}"
        eval "${PATCH_CMD}" 2>/dev/null || true  # May fail if container name differs
        
        echo "  $ ${ROLLOUT_CMD}"
        eval "${ROLLOUT_CMD}"
        
        echo "  Waiting for rollout to complete..."
        if eval "${WAIT_CMD}"; then
            echo -e "  ${GREEN}✓ Deployment rolled out successfully${NC}"
        else
            echo -e "  ${YELLOW}⚠ Rollout may still be in progress${NC}"
        fi
    fi
else
    echo -e "${YELLOW}▶ Step 3/3: Skipping deployment (--no-deploy)${NC}"
fi

echo ""
echo -e "${GREEN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Done!${NC}"
echo -e "${GREEN}══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Image:     ${FULL_IMAGE}"
if [[ "$NO_DEPLOY" == "false" ]]; then
    echo -e "  Deployed:  deployment/${DEPLOYMENT} in ${NAMESPACE}"
    echo ""
    echo -e "  ${YELLOW}View logs:${NC} kubectl logs -f deployment/${DEPLOYMENT} -n ${NAMESPACE}"
    echo -e "  ${YELLOW}Port forward:${NC} kubectl port-forward deployment/${DEPLOYMENT} -n ${NAMESPACE} 8080:8080"
fi
echo ""
echo -e "  ${YELLOW}To revert to production image:${NC}"
echo "    argocd app sync cto --force"
echo ""


