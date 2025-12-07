#!/bin/bash
# Deploy CTO services to Kind cluster
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHARTS_DIR="${SCRIPT_DIR}/../../infra/charts"

echo "=== Deploying CTO Services to Kind ==="

# Ensure namespace exists
kubectl create namespace cto --dry-run=client -o yaml | kubectl apply -f -

# 1. Deploy healer resources first (RBAC, PVC, ConfigMap)
echo ""
echo "1. Deploying healer resources..."
kubectl apply -f "${SCRIPT_DIR}/healer-resources.yaml"
echo "   ✓ Healer resources deployed"

# 2. Deploy OpenMemory (tools depends on it)
echo ""
echo "2. Deploying OpenMemory..."
helm upgrade --install openmemory "${CHARTS_DIR}/openmemory" \
  -f "${SCRIPT_DIR}/openmemory-values.yaml" \
  -n cto \
  --wait --timeout 5m
echo "   ✓ OpenMemory deployed"

# 3. Deploy Tools server
echo ""
echo "3. Deploying Tools server..."
helm upgrade --install tools "${CHARTS_DIR}/tools" \
  -f "${SCRIPT_DIR}/tools-values.yaml" \
  -n cto \
  --wait --timeout 5m
echo "   ✓ Tools server deployed"

# 4. Deploy Healer server
echo ""
echo "4. Deploying Healer server..."
helm upgrade --install healer "${CHARTS_DIR}/universal-app" \
  -f "${SCRIPT_DIR}/healer-values.yaml" \
  -n cto \
  --wait --timeout 5m
echo "   ✓ Healer server deployed"

# 5. Deploy Linear/Integrations server (if image exists)
echo ""
echo "5. Deploying Linear/Integrations server..."
if docker images | grep -q "ghcr.io/5dlabs/linear.*kind-local"; then
  # Create placeholder secrets for Linear
  # Note: For real testing, export these env vars with actual values:
  #   export LINEAR_OAUTH_TOKEN="your-token"
  #   export GITHUB_TOKEN="your-github-token"
  kubectl create secret generic linear-secrets \
    --namespace cto \
    --from-literal=LINEAR_OAUTH_CLIENT_ID="${LINEAR_OAUTH_CLIENT_ID:-placeholder}" \
    --from-literal=LINEAR_OAUTH_CLIENT_SECRET="${LINEAR_OAUTH_CLIENT_SECRET:-placeholder}" \
    --from-literal=LINEAR_OAUTH_TOKEN="${LINEAR_OAUTH_TOKEN:-placeholder}" \
    --from-literal=LINEAR_WEBHOOK_SECRET="${LINEAR_WEBHOOK_SECRET:-placeholder}" \
    --from-literal=GITHUB_TOKEN="${GITHUB_TOKEN:-placeholder}" \
    --dry-run=client -o yaml | kubectl apply -f -
  
  helm upgrade --install linear "${CHARTS_DIR}/linear" \
    -f "${SCRIPT_DIR}/linear-values.yaml" \
    -n cto \
    --wait --timeout 5m
  echo "   ✓ Linear/Integrations server deployed"
else
  echo "   ⚠️  Linear image not found (ghcr.io/5dlabs/linear:kind-local)"
  echo "      Build and load with:"
  echo "      cd infra/images/linear && docker build -f Dockerfile.build -t ghcr.io/5dlabs/linear:kind-local ../../../"
  echo "      kind load docker-image ghcr.io/5dlabs/linear:kind-local"
fi

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "Services deployed:"
kubectl get pods -n cto
echo ""
echo "Services:"
kubectl get svc -n cto
echo ""
echo "Port forward commands:"
echo "  kubectl port-forward svc/tools -n cto 3000:3000"
echo "  kubectl port-forward svc/healer -n cto 8080:8080"
echo "  kubectl port-forward svc/openmemory -n cto 8081:8080"
echo "  kubectl port-forward svc/linear -n cto 8082:8081"



