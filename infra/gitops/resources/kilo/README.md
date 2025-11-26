# Kilo - Kubernetes WireGuard VPN

Kilo is an open-source, Kubernetes-native WireGuard mesh network that provides secure VPN access to the cluster.

## Overview

- **License**: Apache 2.0 (fully open source)
- **Protocol**: WireGuard
- **Mode**: Flannel compatibility (runs alongside existing CNI)
- **Management**: kubectl + kgctl CLI

## Architecture

Kilo runs as a DaemonSet on all nodes, creating a WireGuard mesh. External clients (Peers) can connect to access internal cluster services.

```
┌─────────────────────────────────────────────────────┐
│                  Kubernetes Cluster                  │
│  ┌─────────────┐              ┌─────────────┐       │
│  │ Control Node│◄────────────►│ Worker Node │       │
│  │  (kilo pod) │  WireGuard   │  (kilo pod) │       │
│  └──────┬──────┘    Mesh      └──────┬──────┘       │
│         │                            │              │
│         └────────────┬───────────────┘              │
│                      │                              │
└──────────────────────┼──────────────────────────────┘
                       │ UDP 51820
                       ▼
              ┌─────────────────┐
              │ External Peer   │
              │ (WireGuard)     │
              └─────────────────┘
```

## Setup for New Peers

### 1. Generate WireGuard Keys

On your local machine:

```bash
# Generate key pair
wg genkey | tee privatekey | wg pubkey > publickey

# View your public key
cat publickey
```

### 2. Create Peer Resource

Copy the example template and fill in your public key:

```bash
cp examples/developer-workstation.yaml peers/my-device.yaml
# Edit peers/my-device.yaml with your public key
# Then add it to kustomization.yaml resources
```

Example peer config:

```yaml
apiVersion: kilo.squat.ai/v1alpha1
kind: Peer
metadata:
  name: my-device
spec:
  allowedIPs:
    - 10.5.0.2/32  # Unique IP for this peer
  publicKey: "YOUR_PUBLIC_KEY_HERE"
  persistentKeepalive: 25
```

### 3. Generate Client Configuration

Install kgctl:

```bash
# macOS/Linux
go install github.com/squat/kilo/cmd/kgctl@latest

# Or download from releases
# https://github.com/squat/kilo/releases
```

Generate config:

```bash
kgctl showconf peer my-device > wg0.conf
```

### 4. Add Private Key and Connect

Edit `wg0.conf` to add your private key in the `[Interface]` section:

```ini
[Interface]
PrivateKey = YOUR_PRIVATE_KEY_HERE
Address = 10.5.0.2/32
# Add DNS if you want cluster DNS resolution
DNS = 10.96.0.10

[Peer]
# ... generated config ...
```

Connect:

```bash
# Linux
sudo wg-quick up ./wg0.conf

# macOS (with WireGuard app)
# Import the wg0.conf file into the WireGuard app
```

### 5. Test Connection

```bash
# Test access to a cluster service
curl http://doc-server-agent-docs-server.mcp.svc.cluster.local

# Or ping a pod IP
ping 10.244.x.x
```

## Files

| File | Description |
|------|-------------|
| `crds.yaml` | Peer CustomResourceDefinition |
| `rbac.yaml` | ServiceAccount, ClusterRole, ClusterRoleBinding |
| `daemonset.yaml` | Kilo DaemonSet (Flannel compatibility mode) |
| `examples/*.yaml` | Peer templates (not deployed) |
| `peers/*.yaml` | Actual peer configs (add to kustomization.yaml) |

## Useful Commands

```bash
# List all peers
kubectl get peers

# View peer details
kubectl describe peer developer-workstation

# Check Kilo logs
kubectl logs -n kube-system -l app.kubernetes.io/name=kilo

# Generate network graph
kgctl graph | dot -Tsvg > network.svg
```

## Troubleshooting

### Peer Can't Connect

1. Ensure UDP port 51820 is open on the cluster's public IP
2. Check that the peer's public key matches the one in the Peer resource
3. Verify the peer has a unique allowedIP

### No Route to Cluster Services

1. Ensure the WireGuard interface is up: `sudo wg show`
2. Check that routes are configured: `ip route | grep wg`
3. Verify the service CIDR is in the AllowedIPs

### DNS Not Working

Add the cluster's CoreDNS IP to your WireGuard config:

```ini
[Interface]
DNS = 10.96.0.10
```

## References

- [Kilo Documentation](https://kilo.squat.ai/)
- [Kilo GitHub](https://github.com/squat/kilo)
- [WireGuard](https://www.wireguard.com/)

