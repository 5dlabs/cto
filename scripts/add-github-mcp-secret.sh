#!/usr/bin/env bash
# Add GitHub Personal Access Token for GitHub MCP to the secret store
# Usage: GITHUB_TOKEN=your-token ./scripts/add-github-mcp-secret.sh
#    or: ./scripts/add-github-mcp-secret.sh your-token

set -euo pipefail

# Get token from argument or environment variable
if [ -n "${1:-}" ]; then
    GITHUB_TOKEN="$1"
elif [ -z "${GITHUB_TOKEN:-}" ]; then
    echo "❌ ERROR: GitHub token not provided"
    echo ""
    echo "Usage:"
    echo "  GITHUB_TOKEN=your-token ./scripts/add-github-mcp-secret.sh"
    echo "  or"
    echo "  ./scripts/add-github-mcp-secret.sh your-token"
    exit 1
fi

echo "Adding GitHub MCP token to secret store..."

# Check if secret already exists
if kubectl get secret toolman-github-secrets -n secret-store &>/dev/null; then
    echo "Secret already exists. Updating..."
    kubectl delete secret toolman-github-secrets -n secret-store
fi

# Create the secret
kubectl create secret generic toolman-github-secrets \
    --from-literal=GITHUB_PERSONAL_ACCESS_TOKEN="${GITHUB_TOKEN}" \
    -n secret-store

echo "✅ GitHub MCP token added to secret store successfully!"
echo ""
echo "The External Secrets operator will sync this to the mcp namespace."
echo "Toolman will automatically pick up the secret on next restart."
echo ""
echo "To verify the secret was created:"
echo "  kubectl get secret toolman-github-secrets -n secret-store"
echo ""
echo "To verify External Secrets synced it:"
echo "  kubectl get secret toolman-github-secrets -n mcp"
echo ""
echo "To restart Toolman and pick up the new secret:"
echo "  kubectl rollout restart deployment toolman -n agent-platform"

