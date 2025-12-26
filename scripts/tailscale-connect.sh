#!/bin/bash
# Connect to Headscale VPN
# Usage: sudo ./tailscale-connect.sh [--local]
#
# By default, connects via Cloudflare tunnel (works from anywhere)
# Use --local to connect via internal NodePort (home LAN only)

set -e

# Default to Cloudflare tunnel for remote access
if [ "$1" = "--local" ]; then
    HEADSCALE_SERVER="http://192.168.1.77:30880"
    echo "Using local NodePort (home LAN only)"
else
    HEADSCALE_SERVER="https://headscale.5dlabs.ai"
    echo "Using Cloudflare tunnel (works from anywhere)"
fi

# Auth key - generate new one if expired:
# kubectl exec -n headscale deploy/headscale -- headscale preauthkeys create --user 1 --reusable --expiration 720h
AUTH_KEY="${HEADSCALE_AUTH_KEY:-9c89bbe40cc457b6381abb301eb362cafb7020aeff4300ad}"
SOCKET="/var/run/tailscale/tailscaled.sock"
STATE="/var/lib/tailscale/tailscaled.state"

echo "=== Tailscale Headscale Connector ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run with sudo: sudo $0"
    exit 1
fi

# Kill any existing tailscaled
echo "Stopping any existing tailscaled..."
pkill -9 tailscaled 2>/dev/null || true
sleep 2

# Create directories if needed
mkdir -p /var/run/tailscale
mkdir -p /var/lib/tailscale

# Start tailscaled in background
echo "Starting tailscaled..."
tailscaled --state="$STATE" --socket="$SOCKET" &
DAEMON_PID=$!
sleep 3

# Check if daemon started
if ! kill -0 $DAEMON_PID 2>/dev/null; then
    echo "ERROR: tailscaled failed to start"
    exit 1
fi

echo "tailscaled running (PID: $DAEMON_PID)"

# Connect to Headscale
echo "Connecting to Headscale at $HEADSCALE_SERVER..."
tailscale --socket="$SOCKET" up \
    --login-server="$HEADSCALE_SERVER" \
    --authkey="$AUTH_KEY" \
    --accept-routes \
    --force-reauth

# Show status
echo ""
echo "=== Connection Status ==="
tailscale --socket="$SOCKET" status

echo ""
echo "=== Testing connectivity ==="
echo "Pinging subnet router (100.64.0.1)..."
ping -c 2 100.64.0.1 || echo "Ping failed - may need a moment to establish"

echo ""
echo "Done! tailscaled is running in background (PID: $DAEMON_PID)"
echo "To check status: tailscale --socket=$SOCKET status"
