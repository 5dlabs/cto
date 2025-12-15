#!/bin/bash
# Setup WireGuard peer for connecting to the cluster
set -euo pipefail

WG_DIR="${HOME}/.wireguard"
WG_CONF="${WG_DIR}/wg0.conf"
PRIVATE_KEY_FILE="${WG_DIR}/privatekey"
PUBLIC_KEY_FILE="${WG_DIR}/publickey"

# Cluster configuration
# Use talos-2yg-cty - the only node with WireGuard IP assigned
CLUSTER_ENDPOINT="${CLUSTER_ENDPOINT:-192.168.1.99:51820}"  # talos-2yg-cty
CLUSTER_PUBLIC_KEY="${CLUSTER_PUBLIC_KEY:-ddlSIDUQS12QnMYCPxmjRxy7qC6XAf48cAPVvgrwuVk=}"  # talos-2yg-cty

# Peer configuration
PEER_IP="${PEER_IP:-10.5.0.10/32}"  # Your VPN IP
ALLOWED_IPS="${ALLOWED_IPS:-10.4.0.0/16,10.96.0.0/12,10.244.0.0/16}"  # Cluster networks

echo "🔐 Setting up WireGuard peer..."

# Create directory
mkdir -p "${WG_DIR}"
chmod 700 "${WG_DIR}"

# Check if private key exists
if [[ -f "${PRIVATE_KEY_FILE}" ]]; then
    echo "✅ Private key found at ${PRIVATE_KEY_FILE}"
    PRIVATE_KEY=$(cat "${PRIVATE_KEY_FILE}")
    PUBLIC_KEY=$(cat "${PUBLIC_KEY_FILE}")
else
    echo "⚠️  No private key found. Generating new keypair..."
    PRIVATE_KEY=$(wg genkey)
    PUBLIC_KEY=$(echo "${PRIVATE_KEY}" | wg pubkey)
    
    echo "${PRIVATE_KEY}" > "${PRIVATE_KEY_FILE}"
    echo "${PUBLIC_KEY}" > "${PUBLIC_KEY_FILE}"
    chmod 600 "${PRIVATE_KEY_FILE}"
    
    echo ""
    echo "🔑 NEW KEYS GENERATED!"
    echo "   Public Key: ${PUBLIC_KEY}"
    echo ""
    echo "⚠️  IMPORTANT: You need to update the Peer resource in the cluster!"
    echo "   Edit: infra/gitops/manifests/kilo/peers/jonathon-workstation.yaml"
    echo "   Change publicKey to: \"${PUBLIC_KEY}\""
    echo ""
fi

# Create WireGuard config
cat > "${WG_CONF}" << EOF
# WireGuard configuration for connecting to the Talos cluster
# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

[Interface]
# Private key (keep this secret!)
PrivateKey = ${PRIVATE_KEY}
# VPN IP address assigned to this peer
Address = ${PEER_IP}
# Local DNS for cluster services (optional - uses CoreDNS)
# DNS = 10.96.0.10

[Peer]
# Cluster node public key (talos-a43-ee1)
PublicKey = ${CLUSTER_PUBLIC_KEY}
# Cluster node endpoint
Endpoint = ${CLUSTER_ENDPOINT}
# Networks reachable via VPN:
# - 10.4.0.0/16: WireGuard mesh network
# - 10.96.0.0/12: Kubernetes service CIDR
# - 10.244.0.0/16: Pod CIDR (Flannel)
AllowedIPs = ${ALLOWED_IPS}
# Keep NAT mappings alive
PersistentKeepalive = 25
EOF

chmod 600 "${WG_CONF}"

echo ""
echo "✅ WireGuard configuration created at: ${WG_CONF}"
echo ""
echo "📝 Your public key: ${PUBLIC_KEY}"
echo ""
echo "🚀 To connect, run:"
echo "   sudo wg-quick up ${WG_CONF}"
echo ""
echo "🛑 To disconnect, run:"
echo "   sudo wg-quick down ${WG_CONF}"
echo ""
echo "📊 To check status:"
echo "   sudo wg show"

