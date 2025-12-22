#!/usr/bin/env bash
# =============================================================================
# dev-latitude.sh - Run Tilt with Latitude.sh cluster using port-forward
# =============================================================================
set -euo pipefail

KUBECONFIG="${KUBECONFIG:-$HOME/.kube/cto-dal-cluster.yaml}"
export KUBECONFIG

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Starting dev mode for Latitude.sh cluster...${NC}"

# Check context
CONTEXT=$(kubectl config current-context)
echo -e "Context: ${GREEN}$CONTEXT${NC}"

# Start registry port-forward in background
echo -e "${YELLOW}Starting registry port-forward...${NC}"
kubectl port-forward -n registry svc/registry 30500:5000 &
PF_PID=$!
sleep 2

# Verify port-forward works
if curl -s --connect-timeout 3 http://localhost:30500/v2/ > /dev/null; then
    echo -e "${GREEN}Registry accessible at localhost:30500${NC}"
else
    echo -e "${YELLOW}Warning: Registry not responding yet, waiting...${NC}"
    sleep 3
fi

# Set environment for Tilt
export LOCAL_REGISTRY="localhost:30500"
export CTO_DEV_MODE=true

# Trap to clean up port-forward on exit
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    kill $PF_PID 2>/dev/null || true
}
trap cleanup EXIT

# Enable dev mode in ArgoCD
echo -e "${YELLOW}Enabling dev mode in ArgoCD...${NC}"
./scripts/argocd-dev-mode.sh enable 2>/dev/null || echo "ArgoCD dev mode script not available or failed"

# Start Tilt
echo -e "${GREEN}Starting Tilt...${NC}"
echo -e "Registry: ${GREEN}$LOCAL_REGISTRY${NC}"
echo -e "Dev Mode: ${GREEN}$CTO_DEV_MODE${NC}"
echo ""
tilt up
