#!/bin/bash
# Fix network interface ens8 on GPU node - use static IP instead of DHCP

set -euo pipefail

INSTANCE_IP="${1:-137.74.136.156}"

echo "=== Network fix for GPU node (static IP) ==="
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

echo "=== Step 1: Disable cloud-init network management ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "sudo tee /etc/cloud/cloud.cfg.d/99-disable-network-config.cfg << 'EOF'
network: {config: disabled}
EOF"

echo "=== Step 2: Create netplan config with STATIC IP ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "sudo tee /etc/netplan/60-ens8.yaml << 'EOF'
network:
  version: 2
  ethernets:
    ens8:
      dhcp4: false
      dhcp6: false
      addresses:
        - 10.0.0.159/24
      optional: true
EOF"

echo "=== Step 3: Bring up ens8 manually first ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
      sudo ip link set ens8 up
      sudo ip addr add 10.0.0.159/24 dev ens8
      sleep 2
      ip addr show ens8
    "

echo "=== Step 4: Apply netplan for persistence ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "sudo netplan generate && sudo netplan apply"

echo "=== Step 5: Check interface status ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
      echo '--- Interface Status ---'
      ip addr show ens8
      echo
      echo '--- Routing Table ---'
      ip route | grep 10.0.0 || true
    "

echo "=== Step 6: Test connectivity to RKE2 server ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
      echo 'Testing ports...'
      timeout 5 bash -c '</dev/tcp/10.0.0.181/9345' 2>/dev/null && echo 'Port 9345: REACHABLE' || echo 'Port 9345: UNREACHABLE'
      timeout 5 bash -c '</dev/tcp/10.0.0.181/6443' 2>/dev/null && echo 'Port 6443: REACHABLE' || echo 'Port 6443: UNREACHABLE'
    "

echo "=== Step 7: Update RKE2 config to use private network ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
      sudo tee /etc/rancher/rke2/config.yaml << 'EOF'
server: https://10.0.0.181:9345
token: K10d98a3a0d15168ba535bf7242f143cda36f872042649a2a6314312b51ba5351bd::server:gra9-bootstrap-token-20260312
node-ip: 10.0.0.159
EOF"

echo "=== Step 8: Clear cached kubeconfig and restart RKE2 ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
      sudo rm -f /var/lib/rancher/rke2/agent/kubelet.kubeconfig
      sudo systemctl restart rke2-agent
    "

echo
echo "=== Waiting 20 seconds for RKE2 agent to start ==="
sleep 20

echo "=== Step 9: Check RKE2 agent status ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=30 -o ServerAliveInterval=15 \
    -i "$KEY_FILE" ubuntu@$INSTANCE_IP "
      sudo systemctl status rke2-agent --no-pager || true
      echo
      echo '--- Agent logs (last 20 lines) ---'
      sudo journalctl -u rke2-agent --no-pager -n 20 || true
    "

echo
echo "=== Done ==="
echo "Check node status with: kubectl get nodes"
