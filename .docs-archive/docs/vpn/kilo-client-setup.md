# Kilo VPN Client Setup Guide

This guide explains how to connect your development workstation to the Kubernetes cluster using Kilo, an open-source WireGuard-based VPN.

## Prerequisites

- macOS (Homebrew) or Linux
- `kubectl` configured with cluster access
- Go 1.18+ (for kgctl installation)

## Quick Start

### 1. Install WireGuard and kgctl

**macOS:**
```bash
# Install WireGuard tools
brew install wireguard-tools

# Install kgctl (Kilo CLI)
go install github.com/squat/kilo/cmd/kgctl@latest

# Add Go bin to PATH if not already
export PATH="$PATH:$(go env GOPATH)/bin"
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install wireguard-tools

# Fedora/RHEL
sudo dnf install wireguard-tools

# Install kgctl
go install github.com/squat/kilo/cmd/kgctl@latest
```

### 2. Generate Your WireGuard Keys

```bash
# Create a directory for your WireGuard config
mkdir -p ~/.wireguard
cd ~/.wireguard

# Generate key pair
wg genkey | tee privatekey | wg pubkey > publickey

# Restrict permissions
chmod 600 privatekey

# Display your public key (you'll need this)
cat publickey
```

### 3. Create Your Peer Resource

Create a peer configuration file. Replace `YOUR_PUBLIC_KEY` with your actual public key:

```yaml
# Save as: infra/gitops/resources/kilo/peers/your-name-workstation.yaml
apiVersion: kilo.squat.ai/v1alpha1
kind: Peer
metadata:
  name: your-name-workstation
spec:
  allowedIPs:
    - 10.5.0.XX/32  # Use a unique IP (ask team for next available)
  publicKey: "YOUR_PUBLIC_KEY"
  persistentKeepalive: 25
```

Apply the peer to the cluster:
```bash
kubectl apply -f infra/gitops/resources/kilo/peers/your-name-workstation.yaml
```

> **Note:** Add your peer file to `infra/gitops/resources/kilo/kustomization.yaml` for GitOps-managed deployment.

### 4. Generate Your WireGuard Config

```bash
# Generate the peer configuration
kgctl showconf peer your-name-workstation > ~/.wireguard/kilo-partial.conf

# View the generated config
cat ~/.wireguard/kilo-partial.conf
```

### 5. Create Complete WireGuard Config

Create the full config file at `~/.wireguard/kilo.conf`:

**If on the same LAN as the cluster (192.168.1.x):**
```ini
[Interface]
PrivateKey = <YOUR_PRIVATE_KEY>
Address = 10.5.0.XX/32
# Cluster DNS for service name resolution
DNS = 10.96.0.10

[Peer]
PublicKey = TW6ZDC7d6VR8JrW2vmMpXsSFlQ+y+fhJ+8TLBXxJTFE=
# Only route internal cluster networks (not node IPs since we're on same LAN)
AllowedIPs = 10.244.0.0/16, 10.96.0.0/12, 10.4.0.0/24, 10.5.0.0/24
Endpoint = 192.168.1.72:51820
PersistentKeepalive = 25
```

**If connecting remotely (different network):**
```ini
[Interface]
PrivateKey = <YOUR_PRIVATE_KEY>
Address = 10.5.0.XX/32
DNS = 10.96.0.10

[Peer]
PublicKey = TW6ZDC7d6VR8JrW2vmMpXsSFlQ+y+fhJ+8TLBXxJTFE=
# Route both node IPs and cluster networks
AllowedIPs = 10.244.0.0/16, 10.96.0.0/12, 10.4.0.0/24, 10.5.0.0/24, 192.168.1.72/32, 192.168.1.77/32
# Use public IP/hostname if cluster has external endpoint
Endpoint = <EXTERNAL_ENDPOINT>:51820
PersistentKeepalive = 25
```

Replace:
- `<YOUR_PRIVATE_KEY>` with the contents of `~/.wireguard/privatekey`
- `10.5.0.XX` with your assigned IP
- `<EXTERNAL_ENDPOINT>` with the cluster's public IP (if accessing remotely)

### 6. Connect to the VPN

**Recommended: Use the kilo-vpn.sh script:**
```bash
# Connect
./scripts/kilo-vpn.sh connect

# Check status and test connectivity
./scripts/kilo-vpn.sh status

# Disconnect
./scripts/kilo-vpn.sh disconnect
```

**Alternative: Manual wg-quick commands:**
```bash
# Connect
sudo wg-quick up ~/.wireguard/kilo.conf

# Check status
sudo wg show

# Disconnect
sudo wg-quick down ~/.wireguard/kilo.conf
```

> ⚠️ **Warning: Do NOT mix connection methods!**
>
> Using the WireGuard macOS app AND `wg-quick` simultaneously causes route conflicts.
> Each creates a separate tunnel interface (e.g., `utun6` vs `utun7`), and routes
> may point to the wrong interface, breaking connectivity.
>
> **Choose ONE method:**
> - `scripts/kilo-vpn.sh` (recommended for developers)
> - `wg-quick` manually
> - WireGuard.app (for non-developers who prefer GUI)
>
> If you've been using the WireGuard app, **quit it entirely** before using the script.

### 7. Verify Connection

```bash
# Check WireGuard interface status
sudo wg show

# Test connectivity to a cluster service
curl http://10.110.168.146/api/version  # ArgoCD
curl http://10.96.102.139:3000/health   # Tools
```

## Troubleshooting

### "Handshake did not complete"
- Verify the peer was applied: `kubectl get peer your-name-workstation`
- Check your public key matches: `kubectl describe peer your-name-workstation`
- Ensure the endpoint (192.168.1.72:51820) is reachable

### "Route already exists" warnings
These are harmless - some routes may already exist from previous connections.

### Cannot reach cluster services
1. Verify VPN is up: `sudo wg show`
2. Check handshake completed (look for "latest handshake" in output)
3. Ensure AllowedIPs includes the service CIDR (10.96.0.0/12)

### Connection drops frequently
- Ensure `persistentKeepalive: 25` is set in your peer config
- Check for firewall/NAT issues blocking UDP port 51820

## Assigned Peer IPs

| Name | IP Address | Peer Resource |
|------|------------|---------------|
| jonathon-workstation | 10.5.0.10/32 | `peers/jonathon-workstation.yaml` |
| *(add new peers here)* | 10.5.0.XX/32 | - |

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                         │
│  ┌─────────────────┐              ┌─────────────────┐        │
│  │ Control Node    │◄────────────►│ Worker Node     │        │
│  │ 192.168.1.77    │  WireGuard   │ 192.168.1.72    │        │
│  │  (kilo pod)     │    Mesh      │  (kilo pod)     │        │
│  └────────┬────────┘              └────────┬────────┘        │
│           │                                │                  │
│           └───────────────┬────────────────┘                  │
│                           │                                   │
│                    UDP 51820                                  │
└───────────────────────────┼───────────────────────────────────┘
                            │
                   ┌────────▼────────┐
                   │ Your Workstation│
                   │   10.5.0.XX     │
                   │ (WireGuard)     │
                   └─────────────────┘
```

## Useful Commands

```bash
# List all peers in cluster
kubectl get peers

# View peer details
kubectl describe peer your-name-workstation

# Check Kilo logs
kubectl logs -n kube-system -l app.kubernetes.io/name=kilo

# Generate network graph (requires graphviz)
kgctl graph | dot -Tpng > network.png
```

## Security Notes

- **Never commit your private key** - keep it only on your workstation
- **Public keys are safe to commit** - they're used in peer configs
- Regenerate keys if you suspect compromise: `wg genkey | tee privatekey | wg pubkey > publickey`
