#!/bin/bash
# Reboot GPU node and verify RKE2 connection

set -euo pipefail

INSTANCE_IP="${1:-137.74.136.156}"

echo "=== Rebooting GPU node and verifying RKE2 connection ==="
echo "Instance IP: $INSTANCE_IP"
echo

# Get SSH key from 1Password
echo "Loading SSH private key..."
SSH_KEY=$(op read "op://Automation/OVH GRA9 GPU SSH/private_key")

# Create temp SSH key file
KEY_FILE=$(mktemp)
echo "$SSH_KEY" > "$KEY_FILE"
chmod 600 "$KEY_FILE"
trap "rm -f $KEY_FILE" EXIT

echo "=== Rebooting instance ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "sudo reboot" || true

echo "Waiting 30 seconds for reboot..."
sleep 30

echo "=== Checking if instance is back online ==="
until ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=5 -o ServerAliveInterval=5 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "echo 'Instance is up!'"; do
    echo "Waiting for instance to come back online..."
    sleep 5
done

echo "Instance is back online!"
echo

echo "=== Checking network configuration ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        echo '--- Network Interfaces ---'
        ip addr show
        echo
        echo '--- Routing Table ---'
        ip route
        echo
        echo '--- Testing connectivity to RKE2 server ---'
        timeout 5 bash -c '</dev/tcp/10.0.0.181/9345' 2>/dev/null && echo 'Port 9345: REACHABLE' || echo 'Port 9345: UNREACHABLE'
        timeout 5 bash -c '</dev/tcp/10.0.0.181/6443' 2>/dev/null && echo 'Port 6443: REACHABLE' || echo 'Port 6443: UNREACHABLE'
    "

echo
echo "=== Checking RKE2 agent status ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "sudo systemctl status rke2-agent --no-pager || true"

echo
echo "=== Done ==="
