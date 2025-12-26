#!/bin/bash
# Install tailscaled as a LaunchDaemon (runs at boot, no sudo needed after)
# Usage: sudo ./install-tailscale-daemon.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLIST_SRC="$SCRIPT_DIR/com.tailscale.tailscaled.plist"
PLIST_DST="/Library/LaunchDaemons/com.tailscale.tailscaled.plist"

echo "=== Installing Tailscale LaunchDaemon ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run with sudo: sudo $0"
    exit 1
fi

# Kill any existing tailscaled
echo "Stopping any existing tailscaled..."
pkill -9 tailscaled 2>/dev/null || true
launchctl unload "$PLIST_DST" 2>/dev/null || true
sleep 2

# Create directories
echo "Creating directories..."
mkdir -p /var/run/tailscale
mkdir -p /var/lib/tailscale

# Install plist
echo "Installing LaunchDaemon..."
cp "$PLIST_SRC" "$PLIST_DST"
chown root:wheel "$PLIST_DST"
chmod 644 "$PLIST_DST"

# Load the daemon
echo "Loading daemon..."
launchctl load "$PLIST_DST"
sleep 3

# Check if running
if pgrep -q tailscaled; then
    echo "✓ tailscaled is running!"
else
    echo "✗ tailscaled failed to start. Check /var/log/tailscaled.log"
    exit 1
fi

echo ""
echo "=== Done! ==="
echo "tailscaled will now start automatically at boot."
echo ""
echo "Connect to Headscale with:"
echo "  tailscale --socket=/var/run/tailscale/tailscaled.sock up --login-server=http://192.168.1.64:30880 --authkey=a41c69117af424f07308e834f01397989a73f4839d642194 --accept-routes"
echo ""
echo "Or create an alias in your ~/.zshrc:"
echo '  alias ts="tailscale --socket=/var/run/tailscale/tailscaled.sock"'
