#!/bin/bash
# Join GPU node to RKE2 cluster

set -euo pipefail

INSTANCE_IP="${1:-137.74.136.156}"

echo "Joining GPU node $INSTANCE_IP to RKE2 cluster..."

# Get SSH key from 1Password
echo "Loading SSH private key..."
SSH_KEY=$(op read "op://Automation/OVH GRA9 GPU SSH/private_key")

# Get RKE2 join token
echo "Loading RKE2 join token..."
RKE2_TOKEN=$(op read "op://Automation/RKE2 Join Token/token")
# Use public IP of gra9-node-1 (RKE2 server) since GPU node is on public network
RKE2_SERVER=$(op read "op://Automation/RKE2 Join Token/server_url_public" || echo "https://141.94.213.36:9345")

echo "RKE2 Server: $RKE2_SERVER"

# Create temp SSH key file
KEY_FILE=$(mktemp)
echo "$SSH_KEY" > "$KEY_FILE"
chmod 600 "$KEY_FILE"

# SSH command wrapper
ssh_cmd() {
    ssh -i "$KEY_FILE" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null "$@"
}

# Wait for SSH to be available
echo "Waiting for SSH to be available..."
for i in {1..30}; do
    if ssh_cmd -q ubuntu@$INSTANCE_IP exit 2>/dev/null; then
        echo "SSH is ready!"
        break
    fi
    echo "SSH not ready yet, waiting... ($i/30)"
    sleep 10
done

# Install RKE2 agent
echo "Installing RKE2 agent..."

# First, upload the config file
ssh_cmd ubuntu@$INSTANCE_IP "sudo mkdir -p /etc/rancher/rke2"
ssh_cmd ubuntu@$INSTANCE_IP "sudo tee /etc/rancher/rke2/config.yaml" <<EOF
server: https://141.94.213.36:9345
token: $RKE2_TOKEN
EOF

# Run the installer
ssh_cmd ubuntu@$INSTANCE_IP <<'REMOTE_EOF'
set -e
echo "Downloading RKE2 installer..."
curl -sfL https://get.rke2.io | sudo bash -s - agent

echo "Starting RKE2 agent..."
sudo systemctl enable --now rke2-agent

echo "Waiting for RKE2 agent to be ready..."
sleep 10

echo "RKE2 agent status:"
sudo systemctl status rke2-agent --no-pager || true
REMOTE_EOF

# Cleanup
rm -f "$KEY_FILE"

echo ""
echo "GPU node joined to RKE2 cluster!"
echo "Node IP: $INSTANCE_IP"
echo ""
echo "Next steps:"
echo "1. Verify node appears in kubectl: kubectl get nodes"
echo "2. Wait for GPU Operator to label the node"
echo "3. Verify GPU is detected: kubectl describe node <node-name> | grep nvidia.com/gpu"
