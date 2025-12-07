#!/bin/bash
# Setup Cloudflare Tunnel for Kind local development
#
# This creates a development tunnel separate from production
# Routes: github-webhooks-dev.5dlabs.ai -> Kind cluster
#
# Prerequisites:
#   - 1Password CLI authenticated
#   - Kind cluster running
#   - Cloudflare API token with Tunnel:Edit, DNS:Edit permissions

set -euo pipefail

NAMESPACE="cloudflare-operator-system"
SECRET_NAME="cloudflare-api-credentials"
TUNNEL_NAME="cto-dev"
DEV_HOSTNAME="github-webhooks-dev.5dlabs.ai"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Setting up Cloudflare Tunnel for Kind ===${NC}"

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"
command -v kubectl >/dev/null 2>&1 || { echo -e "${RED}kubectl not found${NC}"; exit 1; }
command -v op >/dev/null 2>&1 || { echo -e "${RED}1Password CLI not found${NC}"; exit 1; }

# Verify Kind context
CONTEXT=$(kubectl config current-context)
if [[ "$CONTEXT" != *"kind"* ]]; then
    echo -e "${RED}WARNING: Current context '$CONTEXT' doesn't look like Kind!${NC}"
    echo "Switch to Kind context or set KUBECONFIG appropriately"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]] || exit 1
fi

# Create namespace
echo -e "\n${YELLOW}Creating namespace...${NC}"
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Get Cloudflare API token from 1Password
echo -e "\n${YELLOW}Getting Cloudflare credentials from 1Password...${NC}"
CF_API_TOKEN=$(op item get "CloudFlare API" --fields label=credential --reveal 2>/dev/null)

if [ -z "$CF_API_TOKEN" ]; then
    echo -e "${RED}Failed to get Cloudflare API token from 1Password${NC}"
    echo "Make sure you have an item called 'CloudFlare API' with a 'credential' field"
    exit 1
fi

# Create Kubernetes secret
echo -e "\n${YELLOW}Creating Cloudflare API secret...${NC}"
kubectl create secret generic $SECRET_NAME \
    --namespace $NAMESPACE \
    --from-literal=CLOUDFLARE_API_TOKEN="$CF_API_TOKEN" \
    --dry-run=client -o yaml | kubectl apply -f -

echo -e "${GREEN}✓ Secret created${NC}"

# Install Cloudflare Operator
echo -e "\n${YELLOW}Installing Cloudflare Operator...${NC}"
kubectl apply -f https://github.com/adyanth/cloudflare-operator/releases/latest/download/release.yaml

# Wait for operator to be ready
echo -e "\n${YELLOW}Waiting for operator to be ready...${NC}"
kubectl wait --for=condition=available deployment/cloudflare-operator-controller-manager \
    -n $NAMESPACE --timeout=120s || true

# Create ClusterTunnel for dev
echo -e "\n${YELLOW}Creating dev ClusterTunnel...${NC}"
cat <<EOF | kubectl apply -f -
apiVersion: networking.cfargotunnel.com/v1alpha2
kind: ClusterTunnel
metadata:
  name: $TUNNEL_NAME
  labels:
    app.kubernetes.io/name: cto-dev-tunnel
    environment: development
spec:
  newTunnel:
    name: $TUNNEL_NAME
  cloudflare:
    domain: 5dlabs.ai
    secret: $SECRET_NAME
    accountId: b73ec19faa187789b3f9d1deb0e0d95f
  fallbackTarget: http_status:404
EOF

echo -e "\n${GREEN}=== Cloudflare Tunnel Setup Complete ===${NC}"
echo ""
echo "Next steps:"
echo "1. Wait for tunnel to connect: kubectl get clustertunnels"
echo "2. Create TunnelBinding for webhooks (after Argo Events is installed)"
echo "3. Update GitHub App webhook URL to: https://$DEV_HOSTNAME/github/webhook"
echo ""
echo "To check tunnel status:"
echo "  kubectl get clustertunnels"
echo "  kubectl get pods -n $NAMESPACE"



