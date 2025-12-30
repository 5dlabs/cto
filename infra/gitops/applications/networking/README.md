# Networking Applications

This directory contains ArgoCD applications for cluster networking and remote access.

## Components

| Application | Description | Status |
|-------------|-------------|--------|
| **kilo** | WireGuard-based VPN mesh (node-to-node + external peers) | Active |
| **platform-ingress** | Ingress resources for ArgoCD, Workflows, Hubble | Active |
| **networking-apps** | App-of-apps that deploys all networking components | Active |
| **headscale** | Self-hosted Tailscale control server | Disabled |
| **tailscale-subnet-router** | Exposes cluster networks to VPN clients | Disabled |

## DNS Records

The following DNS records are automatically created by external-dns:

| Hostname | Service | Cloudflare Proxied |
|----------|---------|-------------------|
| `argocd.5dlabs.ai` | ArgoCD Server | Yes |
| `workflows.5dlabs.ai` | Argo Workflows | Yes |
| `hubble.5dlabs.ai` | Hubble UI | Yes |

## Kilo VPN Setup (Active)

Kilo provides WireGuard-based VPN connectivity to the cluster. It runs as a DaemonSet on all nodes
and creates an encrypted mesh network.

### Network Access

Once connected via Kilo, you can access:

- **Kubernetes API**: `https://192.168.1.77:6443` (control plane node)
- **Kubernetes Services**: `http://10.96.x.x:port` (ClusterIP services)
- **Pod IPs**: `http://10.244.x.x:port` (direct pod access)
- **Node IPs**: `ssh 192.168.1.77` (SSH to nodes, NodePort services)

### Client Setup (macOS/Linux)

1. **Install WireGuard**:
   ```bash
   # macOS
   brew install wireguard-tools

   # Linux
   sudo apt install wireguard  # Debian/Ubuntu
   sudo dnf install wireguard-tools  # Fedora
   ```

2. **Generate client keys**:
   ```bash
   wg genkey | tee privatekey | wg pubkey > publickey
   cat publickey  # You'll need this for the Peer resource
   ```

3. **Create a Peer resource in the cluster**:
   ```yaml
   apiVersion: kilo.squat.ai/v1alpha1
   kind: Peer
   metadata:
     name: my-laptop
   spec:
     publicKey: "YOUR_PUBLIC_KEY_FROM_STEP_2"
     allowedIPs:
       - 10.5.0.2/32  # Assign a unique IP to this peer
     persistentKeepaliveInterval: 25
   ```

   Apply it:
   ```bash
   kubectl apply -f my-laptop-peer.yaml
   ```

4. **Get the cluster's WireGuard configuration**:
   ```bash
   # Get the public key and endpoint from a node
   kubectl get nodes -o jsonpath='{.items[0].metadata.annotations}' | jq '.["kilo.squat.ai/key"]'
   kubectl get nodes -o jsonpath='{.items[0].metadata.annotations}' | jq '.["kilo.squat.ai/endpoint"]'
   ```

5. **Create client WireGuard config** (`/etc/wireguard/kilo.conf`):
   ```ini
   [Interface]
   # Your private key from step 2
   PrivateKey = YOUR_PRIVATE_KEY
   # The IP you assigned in the Peer resource
   Address = 10.5.0.2/32

   [Peer]
   # Cluster's public key from step 4
   PublicKey = CLUSTER_PUBLIC_KEY
   # Cluster endpoint from step 4 (or use node IP:51820)
   Endpoint = 192.168.1.77:51820
   # Networks to route through the VPN
   AllowedIPs = 10.244.0.0/16, 10.96.0.0/12, 192.168.1.0/24
   PersistentKeepalive = 25
   ```

6. **Connect**:
   ```bash
   sudo wg-quick up kilo
   ```

7. **Verify connection**:
   ```bash
   wg show
   ping 192.168.1.77
   kubectl --server=https://192.168.1.77:6443 get nodes
   ```

8. **Disconnect**:
   ```bash
   sudo wg-quick down kilo
   ```

### Kilo Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Kubernetes Cluster                        │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Node 1     │  │   Node 2     │  │   Node 3     │          │
│  │  (Control)   │  │   (Worker)   │  │   (Worker)   │          │
│  │  192.168.1.77│  │  192.168.1.X │  │  192.168.1.Y │          │
│  │              │  │              │  │              │          │
│  │  ┌────────┐  │  │  ┌────────┐  │  │  ┌────────┐  │          │
│  │  │ Kilo   │  │  │  │ Kilo   │  │  │  │ Kilo   │  │          │
│  │  │ Agent  │◄─┼──┼─►│ Agent  │◄─┼──┼─►│ Agent  │  │          │
│  │  │ :51820 │  │  │  │ :51820 │  │  │  │ :51820 │  │          │
│  │  └────────┘  │  │  └────────┘  │  │  └────────┘  │          │
│  └──────┬───────┘  └──────────────┘  └──────────────┘          │
│         │ WireGuard Tunnel                                      │
│         │                                                        │
│  ┌──────┴─────────────────────────────────────────────────────┐│
│  │              Cluster Network Access                         ││
│  │  - Pod CIDR: 10.244.0.0/16                                 ││
│  │  - Service CIDR: 10.96.0.0/12                              ││
│  │  - Node Network: 192.168.1.0/24 (K8s API, SSH, NodePort)   ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ WireGuard Peer
                              │
               ┌──────────────┴──────────────┐
               │      External Client        │
               │      (Your Laptop)          │
               │                             │
               │  ┌───────────────────────┐  │
               │  │ WireGuard Interface   │  │
               │  │ Address: 10.5.0.2/32  │  │
               │  │ AllowedIPs:           │  │
               │  │  - 10.244.0.0/16      │  │
               │  │  - 10.96.0.0/12       │  │
               │  │  - 192.168.1.0/24     │  │
               │  └───────────────────────┘  │
               └─────────────────────────────┘
```

### Troubleshooting Kilo

**Kilo pods not starting**:
```bash
kubectl logs -n kilo -l app.kubernetes.io/name=kilo
kubectl get pods -n kilo
```

**No WireGuard interface on nodes**:
```bash
# Check if WireGuard kernel module is loaded
kubectl exec -n kilo ds/kilo -- lsmod | grep wireguard
```

**Peer not connecting**:
```bash
# Check peer was created
kubectl get peers

# Check node annotations for WireGuard config
kubectl get nodes -o yaml | grep kilo.squat.ai
```

**Routes not working**:
```bash
# On client, verify routes are added
ip route | grep wg
# or on macOS
netstat -rn | grep utun
```

## HeadScale VPN Setup (Disabled)

HeadScale/Tailscale setup has been disabled in favor of Kilo.
The configuration remains in the manifests for reference.

To re-enable, update `networking-apps.yaml` and remove Kilo.

### Previous HeadScale Setup (Reference Only)

The previous setup used:
- Headscale server on HTTPS/443
- Tailscale subnet router advertising cluster networks
- TLS certificates from cert-manager

## Sync Wave Order

Networking apps use ArgoCD sync waves:

- Wave `-1`: Kilo (provides VPN connectivity)
- Wave `0`: Default (networking-apps)
- Wave `1`: Platform ingress

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Internet                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
              ┌─────────────▼─────────────┐
              │    Cloudflare (Proxied)   │
              │  argocd.5dlabs.ai         │
              │  workflows.5dlabs.ai      │
              │  hubble.5dlabs.ai         │
              └─────────────┬─────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                     Kubernetes Cluster                           │
│                                                                  │
│  ┌─────────────┐  ┌────────────────┐  ┌────────────────────┐   │
│  │ ingress-    │  │    Kilo        │  │   Cilium           │   │
│  │   nginx     │  │  (WireGuard)   │  │   (CNI + eBPF)     │   │
│  │             │  │  :51820/UDP    │  │                    │   │
│  └─────────────┘  └────────────────┘  └────────────────────┘   │
│                           │                                      │
│                           ▼                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Cluster Network Access via VPN               │   │
│  │  - Pod CIDR: 10.244.0.0/16                               │   │
│  │  - Service CIDR: 10.96.0.0/12                            │   │
│  │  - Node Network: 192.168.1.0/24 (K8s API, SSH, NodePort) │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```
