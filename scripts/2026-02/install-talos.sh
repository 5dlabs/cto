#!/bin/bash
# Talos Installation Script for Scaleway Elastic Metal
# Usage: ./install-talos.sh <node1-ip> <node2-ip> [...]

set -e

NODES="$@"
TALOS_VERSION="v1.10.4"
TALOS_DIR="$(cd "$(dirname "$0")/../infra/talos" && pwd)"

echo "☸️  Talos Kubernetes Installation"
echo "================================"
echo "Nodes: $NODES"
echo "Talos Version: $TALOS_VERSION"
echo ""

# Download talosctl
echo "📦 Downloading talosctl..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    ARCH="amd64"
    curl -Lo talosctl "https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/talosctl-darwin-${ARCH}"
else
    ARCH="amd64"
    curl -Lo talosctl "https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/talosctl-linux-${ARCH}"
fi
chmod +x talosctl

# Generate talosconfig
echo "🔑 Generating talosconfig..."
./talosctl gen config talos-cluster "${TALOS_DIR}/config/simple" --force

# Update node IPs in configs
echo "📝 Updating node IPs in Talos configs..."
for i in "${!NODES[@]}"; do
    idx=$((i+1))
    node="${NODES[$i]}"
    echo "  Node $idx: $node"
    
    # Update controlplane config (first node)
    if [[ $idx -eq 1 ]]; then
        sed -i '' "s/192.168.1.77/$node/g" "${TALOS_DIR}/config/simple/controlplane.yaml"
    fi
done

# For Scaleway, we need to use the install API to boot from ISO or use rescue mode
echo ""
echo "💡 Talos Installation Options for Scaleway Elastic Metal:"
echo ""
echo "Option 1: Use Scaleway Rescue Mode"
echo "------------------------------------"
echo "1. Boot server into rescue mode via Scaleway console"
echo "2. SSH into rescue system"
echo "3. Download and run talosctl:"
cat << 'EOF'
   
   # Download Talos kernel/initrd
   curl -LO https://github.com/siderolabs/talos/releases/download/v1.10.4/metal-amd64-amd64.tar.gz
   tar xzf metal-amd64-amd64.tar.gz
   
   # Boot via iPXE or install to disk
   ./metal install -n <server-ip> -t <talos-config.yaml>
   
   # Bootstrap
   ./talosctl bootstrap -n <control-plane-ip>
   
   # Get kubeconfig
   ./talosctl kubeconfig -n <control-plane-ip>
EOF

echo ""
echo "Option 2: Use Existing Ubuntu (Talos-in-Machine)"
echo "-------------------------------------------------"
cat << 'EOF'
   # On each node, run:
   ssh ubuntu@<node-ip>
   
   # Download and run Talos installer
   curl -sL https://tal.dev/install | bash -s -
   
   # From your workstation:
   ./talosctl apply-config --insecure --nodes <node-ip> --file controlplane.yaml
   ./talosctl bootstrap -n <control-plane-ip>
EOF

echo ""
echo "📖 Full instructions: agents/metal/infra/talos/README.md"
