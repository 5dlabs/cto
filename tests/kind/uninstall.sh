#!/bin/bash
# Uninstall CTO services from Kind cluster
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Uninstalling CTO Services from Kind ==="

# Uninstall Helm releases
echo "Uninstalling Helm releases..."
helm uninstall tools -n cto 2>/dev/null || echo "  tools not installed"
helm uninstall healer -n cto 2>/dev/null || echo "  healer not installed"
helm uninstall openmemory -n cto 2>/dev/null || echo "  openmemory not installed"

# Remove healer resources
echo ""
echo "Removing healer resources..."
kubectl delete -f "${SCRIPT_DIR}/healer-resources.yaml" --ignore-not-found

echo ""
echo "=== Uninstall Complete ==="
echo ""
echo "Remaining pods in cto namespace:"
kubectl get pods -n cto 2>/dev/null || echo "  (none or namespace doesn't exist)"






