#!/bin/bash
# Connect to Headscale VPN
# Usage: sudo ./tailscale-connect.sh

set -e

HEADSCALE_SERVER="http://192.168.1.64:30880"
AUTH_KEY="a41c69117af424f07308e834f01397989a73f4839d642194"
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
