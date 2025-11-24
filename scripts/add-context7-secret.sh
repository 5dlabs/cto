#!/usr/bin/env bash
# Add Context7 API key to the secret store
# Usage: CONTEXT7_API_KEY=your-key ./scripts/add-context7-secret.sh
#    or: ./scripts/add-context7-secret.sh your-key

set -euo pipefail

# Get API key from argument or environment variable
if [ -n "${1:-}" ]; then
    CONTEXT7_API_KEY="$1"
elif [ -z "${CONTEXT7_API_KEY:-}" ]; then
    echo "❌ ERROR: Context7 API key not provided"
    echo ""
    echo "Usage:"
    echo "  CONTEXT7_API_KEY=your-key ./scripts/add-context7-secret.sh"
    echo "  or"
    echo "  ./scripts/add-context7-secret.sh your-key"
    exit 1
fi

echo "Adding Context7 API key to secret store..."

# Check if secret already exists
if kubectl get secret toolman-context7-secrets -n secret-store &>/dev/null; then
    echo "Secret already exists. Updating..."
    kubectl delete secret toolman-context7-secrets -n secret-store
fi

# Create the secret
kubectl create secret generic toolman-context7-secrets \
    --from-literal=CONTEXT7_API_KEY="${CONTEXT7_API_KEY}" \
    -n secret-store

echo "✅ Context7 API key added to secret store successfully!"
echo ""
echo "The External Secrets operator will sync this to the mcp namespace."
echo "Toolman will automatically pick up the secret on next restart."
echo ""
echo "To verify the secret was created:"
echo "  kubectl get secret toolman-context7-secrets -n secret-store"
echo ""
echo "To verify External Secrets synced it:"
echo "  kubectl get secret toolman-context7-secrets -n mcp"
echo ""
echo "To restart Toolman and pick up the new secret:"
echo "  kubectl rollout restart deployment toolman -n agent-platform"

