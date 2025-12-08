#!/usr/bin/env bash
# =============================================================================
# dev-load.sh - Load images directly into Talos cluster (no remote registry)
# =============================================================================
# Build locally and push to an in-cluster registry for fast iteration.
# No external network required - everything stays on your local network!
#
# First time setup:
#   ./scripts/dev-load.sh --setup    # Deploy local registry to cluster
#
# Usage:
#   ./scripts/dev-load.sh controller    # Build and load controller
#   ./scripts/dev-load.sh tools         # Build and load tools
#   ./scripts/dev-load.sh --list        # List available images
#
# Options:
#   --setup        Deploy local registry to cluster (one-time)
#   --no-deploy    Build and load only, skip deployment rollout
#   --tag TAG      Use custom tag (default: dev-local)
#   --dry-run      Show what would be done without executing
#
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
NAMESPACE="${NAMESPACE:-cto}"
DEV_TAG="${DEV_TAG:-dev-local}"
REGISTRY_NAMESPACE="${REGISTRY_NAMESPACE:-registry}"
REGISTRY_PORT="${REGISTRY_PORT:-5000}"

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
SETUP_REGISTRY=false
CUSTOM_TAG=""
IMAGE_NAME=""

print_help() {
    echo "Usage: $0 [OPTIONS] IMAGE"
    echo ""
    echo "Build images locally and push to in-cluster registry for fast iteration."
    echo "No external network required - everything stays local!"
    echo ""
    echo "Arguments:"
    echo "  IMAGE           Image to build (controller, tools, healer, pm, etc.)"
    echo ""
    echo "Options:"
    echo "  --setup         Deploy local registry to cluster (one-time setup)"
    echo "  --list          List available images"
    echo "  --no-deploy     Build and load only, skip deployment rollout"
    echo "  --tag TAG       Use custom tag (default: dev-local)"
    echo "  --dry-run       Show what would be done without executing"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "First time setup:"
    echo "  $0 --setup                 # Deploy registry, then:"
    echo "  $0 controller              # Build and deploy"
    echo ""
    echo "Examples:"
    echo "  $0 controller              # Build, push to local registry, deploy"
    echo "  $0 --no-deploy tools       # Build and push only"
    echo ""
    echo "Environment Variables:"
    echo "  NAMESPACE         Kubernetes namespace (default: cto)"
    echo "  DEV_TAG           Default tag (default: dev-local)"
    echo "  REGISTRY_PORT     Local registry port (default: 5000)"
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

setup_registry() {
    echo -e "${BLUE}Setting up local registry in cluster...${NC}"
    echo ""
    
    # Create namespace
    kubectl create namespace "$REGISTRY_NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    # Deploy registry
    cat <<EOF | kubectl apply -f -
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: registry-data
  namespace: ${REGISTRY_NAMESPACE}
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: registry
  namespace: ${REGISTRY_NAMESPACE}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: registry
  template:
    metadata:
      labels:
        app: registry
    spec:
      containers:
        - name: registry
          image: registry:2
          ports:
            - containerPort: 5000
          env:
            - name: REGISTRY_STORAGE_DELETE_ENABLED
              value: "true"
          volumeMounts:
            - name: data
              mountPath: /var/lib/registry
          resources:
            requests:
              memory: 128Mi
              cpu: 100m
            limits:
              memory: 512Mi
              cpu: 500m
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: registry-data
---
apiVersion: v1
kind: Service
metadata:
  name: registry
  namespace: ${REGISTRY_NAMESPACE}
spec:
  type: NodePort
  ports:
    - port: 5000
      targetPort: 5000
      nodePort: 30500
  selector:
    app: registry
EOF

    echo ""
    echo -e "${GREEN}✓ Registry deployed!${NC}"
    echo ""
    echo "Waiting for registry to be ready..."
    kubectl rollout status deployment/registry -n "$REGISTRY_NAMESPACE" --timeout=120s
    
    # Get node IP for local access
    NODE_IP=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}')
    
    echo ""
    echo -e "${GREEN}══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Local Registry Ready!${NC}"
    echo -e "${GREEN}══════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Registry URL: ${NODE_IP}:30500"
    echo ""
    echo -e "  ${YELLOW}Configure Docker to trust this registry:${NC}"
    echo ""
    echo "  For Docker Desktop, add to daemon.json:"
    echo "    {\"insecure-registries\": [\"${NODE_IP}:30500\"]}"
    echo ""
    echo "  Or on macOS, add via Docker Desktop → Settings → Docker Engine"
    echo ""
    echo -e "  ${YELLOW}Then restart Docker and run:${NC}"
    echo "    $0 controller"
    echo ""
}

check_registry() {
    # Check if registry is deployed
    if ! kubectl get deployment registry -n "$REGISTRY_NAMESPACE" &>/dev/null; then
        echo -e "${RED}Error: Local registry not found${NC}"
        echo ""
        echo "Run setup first:"
        echo "  $0 --setup"
        echo ""
        exit 1
    fi
    
    # Check if registry is ready
    if ! kubectl get deployment registry -n "$REGISTRY_NAMESPACE" -o jsonpath='{.status.readyReplicas}' | grep -q "1"; then
        echo -e "${YELLOW}Warning: Registry deployment not ready${NC}"
        echo "Waiting for registry..."
        kubectl rollout status deployment/registry -n "$REGISTRY_NAMESPACE" --timeout=60s
    fi
}

get_registry_url() {
    # Get node IP
    local node_ip
    node_ip=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}')
    echo "${node_ip}:30500"
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --setup)
            SETUP_REGISTRY=true
            shift
            ;;
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

# Handle setup
if [[ "$SETUP_REGISTRY" == "true" ]]; then
    setup_registry
    exit 0
fi

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

# Check registry is ready
check_registry

# Get registry URL
REGISTRY_URL=$(get_registry_url)

# Use custom tag if provided
TAG="${CUSTOM_TAG:-$DEV_TAG}"

# Parse image config
IFS=':' read -r DOCKERFILE CONTEXT DEPLOYMENT <<< "${IMAGES[$IMAGE_NAME]}"

# Local registry image path
LOCAL_IMAGE="${REGISTRY_URL}/${IMAGE_NAME}:${TAG}"

echo ""
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Dev Load: ${IMAGE_NAME} (local registry)${NC}"
echo -e "${BLUE}══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${YELLOW}Image:${NC}      ${LOCAL_IMAGE}"
echo -e "  ${YELLOW}Dockerfile:${NC} ${DOCKERFILE}"
echo -e "  ${YELLOW}Context:${NC}    ${CONTEXT}"
echo -e "  ${YELLOW}Deployment:${NC} ${DEPLOYMENT}"
echo -e "  ${YELLOW}Registry:${NC}   ${REGISTRY_URL}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}[DRY RUN] Would execute the following:${NC}"
    echo ""
fi

# Step 1: Build the image
echo -e "${GREEN}▶ Step 1/3: Building image...${NC}"
BUILD_CMD="docker build --platform linux/amd64 -t ${LOCAL_IMAGE} -f ${ROOT_DIR}/${DOCKERFILE} ${ROOT_DIR}/${CONTEXT}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo "  $ ${BUILD_CMD}"
else
    echo "  $ ${BUILD_CMD}"
    cd "${ROOT_DIR}"
    eval "${BUILD_CMD}"
    echo -e "  ${GREEN}✓ Build complete${NC}"
fi
echo ""

# Step 2: Push to local registry
echo -e "${GREEN}▶ Step 2/3: Pushing to local registry...${NC}"
PUSH_CMD="docker push ${LOCAL_IMAGE}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo "  $ ${PUSH_CMD}"
else
    echo "  $ ${PUSH_CMD}"
    if ! eval "${PUSH_CMD}"; then
        echo ""
        echo -e "${RED}Push failed! Make sure Docker trusts the insecure registry.${NC}"
        echo ""
        echo "Add to Docker daemon.json:"
        echo "  {\"insecure-registries\": [\"${REGISTRY_URL}\"]}"
        echo ""
        echo "Then restart Docker."
        exit 1
    fi
    echo -e "  ${GREEN}✓ Pushed to local registry${NC}"
fi
echo ""

# Step 3: Deploy (unless --no-deploy)
if [[ "$NO_DEPLOY" == "false" ]]; then
    echo -e "${GREEN}▶ Step 3/3: Rolling out deployment...${NC}"
    
    # Patch the deployment to use the new image and force a rollout
    PATCH_CMD="kubectl set image deployment/${DEPLOYMENT} ${DEPLOYMENT}=${LOCAL_IMAGE} -n ${NAMESPACE}"
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
echo -e "  Image:     ${LOCAL_IMAGE}"
if [[ "$NO_DEPLOY" == "false" ]]; then
    echo -e "  Deployed:  deployment/${DEPLOYMENT} in ${NAMESPACE}"
    echo ""
    echo -e "  ${YELLOW}View logs:${NC} kubectl logs -f deployment/${DEPLOYMENT} -n ${NAMESPACE}"
fi
echo ""
echo -e "  ${YELLOW}To revert to production image:${NC}"
echo "    argocd app sync cto --force"
echo ""
