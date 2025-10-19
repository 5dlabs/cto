#!/bin/bash
set -euo pipefail

# ============================================================================
# Blaze - Ngrok Ingress Setup Script
# ============================================================================
# Creates Ngrok ingress for live preview URLs of frontend applications
# deployed in Kubernetes staging namespace
# ============================================================================

NAMESPACE="${NAMESPACE:-agent-platform}"
APP_NAME="${1:-}"
SERVICE_NAME="${2:-}"
SERVICE_PORT="${3:-3000}"

if [ -z "$APP_NAME" ] || [ -z "$SERVICE_NAME" ]; then
    echo "Usage: $0 <app-name> <service-name> [port]"
    echo "Example: $0 task-5-frontend task-5-frontend-svc 3000"
    exit 1
fi

echo "🌐 Setting up Ngrok ingress for $APP_NAME..."

# Generate unique Ngrok domain (or use provided one)
NGROK_DOMAIN="${NGROK_DOMAIN:-${APP_NAME}-preview}"

# Create Ngrok Ingress manifest
cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ${APP_NAME}-ingress
  namespace: ${NAMESPACE}
  labels:
    app: ${APP_NAME}
    managed-by: blaze
  annotations:
    k8s.ngrok.com/modules: ngrok-module-set
spec:
  ingressClassName: ngrok
  rules:
    - host: ${NGROK_DOMAIN}.ngrok.app
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: ${SERVICE_NAME}
                port:
                  number: ${SERVICE_PORT}
---
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: NgrokModuleSet
metadata:
  name: ngrok-module-set
  namespace: ${NAMESPACE}
spec:
  modules:
    compression:
      enabled: true
    headers:
      request:
        add:
          x-preview-app: ${APP_NAME}
EOF

echo "⏳ Waiting for Ngrok ingress to be ready..."
kubectl wait --for=condition=Ready \
    ingress/${APP_NAME}-ingress \
    -n ${NAMESPACE} \
    --timeout=60s || true

# Get the actual Ngrok URL
NGROK_URL=$(kubectl get ingress ${APP_NAME}-ingress -n ${NAMESPACE} \
    -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "")

if [ -z "$NGROK_URL" ]; then
    # Fallback: construct URL from domain
    NGROK_URL="https://${NGROK_DOMAIN}.ngrok.app"
fi

echo ""
echo "✅ Ngrok ingress created successfully!"
echo ""
echo "🌐 Live Preview URL:"
echo "   ${NGROK_URL}"
echo ""
echo "📋 Ingress Details:"
echo "   Name: ${APP_NAME}-ingress"
echo "   Namespace: ${NAMESPACE}"
echo "   Service: ${SERVICE_NAME}:${SERVICE_PORT}"
echo ""
echo "💡 This URL will remain active as long as the ingress exists."
echo "   To delete: kubectl delete ingress ${APP_NAME}-ingress -n ${NAMESPACE}"
echo ""

# Output URL in a machine-readable format for PR comments
cat > /tmp/ngrok-url.txt <<EOF
${NGROK_URL}
EOF

echo "📝 URL saved to /tmp/ngrok-url.txt"

