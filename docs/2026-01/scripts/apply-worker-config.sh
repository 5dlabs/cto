#!/bin/bash
# Apply Talos worker config to cto-dal-w1
set -euo pipefail

WORKER_IP="72.46.85.113"
WORKER_CONFIG="$HOME/.talos/clusters/cto-dal/worker.yaml"
TALOSCONFIG="$HOME/.talos/clusters/cto-dal/talosconfig"
KUBECONFIG="$HOME/.talos/clusters/cto-dal/kubeconfig"

export TALOSCONFIG KUBECONFIG

echo "🔧 CTO-DAL Worker Node Setup"
echo ""
echo "Worker IP: $WORKER_IP"
echo "Config:    $WORKER_CONFIG"
echo ""

# Check if worker is reachable
echo "Checking if worker is reachable..."
if ! nc -z -w 5 "$WORKER_IP" 50000 2>/dev/null; then
    echo "❌ Worker Talos API (port 50000) is not responding."
    echo ""
    echo "The worker may still be booting. You can:"
    echo "1. Wait a few more minutes and try again"
    echo "2. Check the console via Latitude dashboard"
    echo "3. Verify the server is powered on:"
    echo ""
    echo "   LATITUDE_API_KEY=\$(op item get 'Latitude.sh API' --vault Personal --fields credential --reveal)"
    echo "   curl -H \"Authorization: Bearer \$LATITUDE_API_KEY\" https://api.latitude.sh/servers/sv_X6KG5mxReNyPB | jq '.data.attributes.status'"
    echo ""
    exit 1
fi

echo "✅ Worker is reachable!"
echo ""

# Check if Talos is in maintenance mode (waiting for config)
echo "Checking Talos status..."
if talosctl --nodes "$WORKER_IP" --insecure version 2>&1 | grep -q "Server:"; then
    echo "✅ Talos is running on worker"
else
    echo "⚠️  Cannot verify Talos version, proceeding anyway..."
fi

echo ""
echo "Applying worker config..."
talosctl apply-config --nodes "$WORKER_IP" --file "$WORKER_CONFIG" --insecure

echo ""
echo "✅ Config applied! The worker should now:"
echo "   1. Install Talos to disk"
echo "   2. Reboot"
echo "   3. Join the cluster"
echo ""
echo "Monitor progress with:"
echo "   talosctl --nodes $WORKER_IP dmesg -f"
echo ""
echo "Check cluster status with:"
echo "   kubectl get nodes -o wide"
