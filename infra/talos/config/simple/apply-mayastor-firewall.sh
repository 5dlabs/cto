#!/bin/bash
# Apply Mayastor firewall patches to allow required ports
# This script must be run with a configured talosctl context

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PATCH_FILE="$SCRIPT_DIR/mayastor-network-patch.yaml"

# Node IPs
CONTROL_PLANE="10.8.0.1"
WORKER="10.8.0.2"

echo "========================================="
echo "Mayastor Firewall Configuration"
echo "========================================="
echo ""
echo "This script will open the following ports:"
echo "  - 50051/tcp: agent-core gRPC API"
echo "  - 50052/tcp: agent-core HA cluster"
echo "  - 10124/tcp: io-engine NVMe-oF target"
echo ""

# Check if talosctl is available
if ! command -v talosctl &> /dev/null; then
    echo "ERROR: talosctl is not installed or not in PATH"
    exit 1
fi

# Check if talosctl context is configured
if ! talosctl config info &> /dev/null; then
    echo "ERROR: talosctl context is not configured"
    echo ""
    echo "Please configure talosctl with:"
    echo "  export TALOSCONFIG=\$HOME/.talos/config"
    echo "Or ensure config/simple/talosconfig exists and run:"
    echo "  export TALOSCONFIG=$SCRIPT_DIR/talosconfig"
    exit 1
fi

# Check if patch file exists
if [ ! -f "$PATCH_FILE" ]; then
    echo "ERROR: Patch file not found: $PATCH_FILE"
    exit 1
fi

echo "Applying firewall patch to control plane ($CONTROL_PLANE)..."
talosctl patch machineconfig -n "$CONTROL_PLANE" --patch @"$PATCH_FILE"
echo "✓ Control plane patched"
echo ""

echo "Applying firewall patch to worker ($WORKER)..."
talosctl patch machineconfig -n "$WORKER" --patch @"$PATCH_FILE"
echo "✓ Worker patched"
echo ""

echo "========================================="
echo "Firewall patches applied successfully!"
echo "========================================="
echo ""
echo "The nodes do not need to be rebooted - the firewall rules"
echo "are applied immediately."
echo ""
echo "You can verify the Mayastor disk pools with:"
echo "  kubectl get diskpools -n mayastor"
echo ""
echo "Within 20 seconds, the control plane pools should transition"
echo "from 'Creating' to 'Created' status."
