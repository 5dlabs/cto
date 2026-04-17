#!/bin/bash
# Check RKE2 server configuration and update to advertise public IP

set -euo pipefail

# RKE2 server public IP
SERVER_IP="141.94.213.36"
SERVER_PRIVATE_IP="10.0.0.181"

echo "=== RKE2 Server Configuration Check ==="
echo "Server Public IP: $SERVER_IP"
echo "Server Private IP: $SERVER_PRIVATE_IP"
echo

# Get SSH key from 1Password
echo "Loading SSH private key..."
SSH_KEY=$(op read "op://Automation/OVH GRA9 GPU SSH/private_key")

# Create temp SSH key file
KEY_FILE=$(mktemp)
echo "$SSH_KEY" > "$KEY_FILE"
chmod 600 "$KEY_FILE"
trap "rm -f $KEY_FILE" EXIT

echo "=== Checking RKE2 server configuration ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i "$KEY_FILE" ubuntu@$SERVER_IP << EOF
echo "--- RKE2 server service status ---"
sudo systemctl status rke2-server --no-pager || true

echo
echo "--- RKE2 server config ---"
cat /etc/rancher/rke2/config.yaml 2>/dev/null || echo "Config file not found"

echo
echo "--- Current RKE2 server process args ---"
ps aux | grep rke2 | grep -v grep || true

echo
echo "--- Network interfaces ---"
ip addr show || true
EOF

echo
echo "=== Configuration Analysis ==="
echo "The RKE2 server is currently advertising its private IP ($SERVER_PRIVATE_IP) to agents."
echo "To fix the GPU node connectivity, we need to configure the server to advertise its public IP."
echo
echo "Options:"
echo "1. Add 'advertise-address: $SERVER_IP' to /etc/rancher/rke2/config.yaml"
echo "2. Add 'tls-san:' entries for both public and private IPs"
echo "3. Add 'node-external-ip: $SERVER_IP' to advertise the external IP"
echo
echo "WARNING: Modifying the RKE2 server configuration requires a restart and may"
echo "         temporarily disrupt the cluster. All existing agents will need to"
echo "         reconnect using the new advertised address."
