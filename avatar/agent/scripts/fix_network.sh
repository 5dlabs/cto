#!/bin/bash
# Simple network fix for GPU node - bring up ens8 manually

set -euo pipefail

INSTANCE_IP="${1:-137.74.136.156}"

echo "=== Simple network fix for GPU node ==="
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

# Wait for SSH to be available
echo "=== Waiting for SSH to be available ==="
for i in {1..30}; do
    if ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
        -o ConnectTimeout=5 -o ServerAliveInterval=5 \
        -i "$KEY_FILE" ubuntu@$INSTANCE_IP "echo 'SSH ready'" 2>/dev/null; then
        echo "SSH is available!"
        break
    fi
    echo "Attempt $i: SSH not ready yet, waiting..."
    sleep 10
done

echo "=== Step 1: Check current network state ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        echo '--- Interfaces ---'
        ip link show
        echo
        echo '--- ens8 details ---'
        ip addr show ens8 2>/dev/null || echo 'ens8 not configured'
    "

echo "=== Step 2: Bring up ens8 and get DHCP lease ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        sudo ip link set ens8 up || echo 'Failed to bring up ens8'
        sleep 2
        sudo dhclient ens8 -v || echo 'dhclient failed or lease already obtained'
    "

echo "=== Step 3: Check if we got an IP ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        echo '--- ens8 after DHCP ---'
        ip addr show ens8
        echo
        echo '--- Testing connectivity to RKE2 server ---'
        timeout 5 bash -c '</dev/tcp/10.0.0.181/9345' 2>/dev/null && echo 'Port 9345: REACHABLE' || echo 'Port 9345: UNREACHABLE'
        timeout 5 bash -c '</dev/tcp/10.0.0.181/6443' 2>/dev/null && echo 'Port 6443: REACHABLE' || echo 'Port 6443: UNREACHABLE'
    "

echo "=== Step 4: Create persistent netplan config ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        # Disable cloud-init network management
        echo 'network: {config: disabled}' | sudo tee /etc/cloud/cloud.cfg.d/99-disable-network-config.cfg
        
        # Create netplan config for ens8
        sudo tee /etc/netplan/60-ens8.yaml << 'NETPLAN'
network:
  version: 2
  ethernets:
    ens8:
      dhcp4: true
      dhcp6: false
      optional: true
NETPLAN
        
        sudo chmod 644 /etc/netplan/60-ens8.yaml
    "

echo "=== Step 5: Apply netplan ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        sudo netplan generate 2>/dev/null || echo 'netplan generate had issues'
        sudo netplan apply 2>/dev/null || echo 'netplan apply had issues'
    "

echo "=== Step 6: Restart RKE2 agent ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
        sudo systemctl restart rke2-agent
        sleep 5
        sudo systemctl status rke2-agent --no-pager || true
    "

echo
echo "=== Done ==="
echo "Check node status with: kubectl get nodes"
