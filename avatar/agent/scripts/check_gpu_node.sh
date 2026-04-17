#!/bin/bash
# Check GPU node RKE2 status and cluster connectivity

set -euo pipefail

INSTANCE_IP="${1:-137.74.136.156}"

echo "=== GPU Node Diagnostics for $INSTANCE_IP ==="
echo

# Get SSH key from 1Password
echo "Loading SSH private key..."
SSH_KEY=$(op read "op://Automation/OVH GRA9 GPU SSH/private_key")

# Create temp SSH key file
KEY_FILE=$(mktemp)
echo "$SSH_KEY" > "$KEY_FILE"
chmod 600 "$KEY_FILE"
trap "rm -f $KEY_FILE" EXIT

echo "=== Checking RKE2 agent status ==="
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i "$KEY_FILE" ubuntu@$INSTANCE_IP << 'EOF'
echo "--- RKE2 agent service status ---"
sudo systemctl status rke2-agent --no-pager || true

echo
echo "--- RKE2 agent logs (last 50 lines) ---"
sudo journalctl -u rke2-agent -n 50 --no-pager || true

echo
echo "--- RKE2 config ---"
cat /etc/rancher/rke2/config.yaml 2>/dev/null || echo "Config file not found"

echo
echo "--- Network interfaces ---"
ip addr show || true

echo
echo "--- Routing table ---"
ip route || true

echo
echo "--- Can we reach RKE2 server ports? ---"
timeout 5 bash -c "</dev/tcp/141.94.213.36/9345" 2>/dev/null && echo "Port 9345 (supervisor): REACHABLE" || echo "Port 9345 (supervisor): UNREACHABLE"
timeout 5 bash -c "</dev/tcp/141.94.213.36/6443" 2>/dev/null && echo "Port 6443 (apiserver): REACHABLE" || echo "Port 6443 (apiserver): UNREACHABLE"
timeout 5 bash -c "</dev/tcp/10.0.0.181/9345" 2>/dev/null && echo "Port 9345 (private): REACHABLE" || echo "Port 9345 (private): UNREACHABLE"
timeout 5 bash -c "</dev/tcp/10.0.0.181/6443" 2>/dev/null && echo "Port 6443 (private): REACHABLE" || echo "Port 6443 (private): UNREACHABLE"

echo
echo "--- kubelet kubeconfig (sanitized) ---"
if [ -f /var/lib/rancher/rke2/agent/kubelet.kubeconfig ]; then
    sudo grep server /var/lib/rancher/rke2/agent/kubelet.kubeconfig || true
else
    echo "kubelet.kubeconfig not found"
fi
EOF

echo
echo "=== Checking from existing cluster node ==="
# Get an existing node IP from 1Password or use known IP
EXISTING_NODE=$(op read "op://Automation/RKE2 Join Token/server_url" 2>/dev/null | sed 's|https://||' | sed 's/:.*//' || echo "10.0.0.181")

echo "Attempting to check cluster status from $EXISTING_NODE..."
