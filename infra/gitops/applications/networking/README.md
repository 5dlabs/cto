# Networking Applications

This directory contains ArgoCD applications for cluster networking and remote access.

## Components

| Application | Description |
|-------------|-------------|
| **headscale** | Self-hosted Tailscale control server (WireGuard VPN) |
| **tailscale-subnet-router** | Exposes cluster networks to VPN clients |
| **platform-ingress** | Ingress resources for ArgoCD, Workflows, Headscale, Hubble |
| **networking-apps** | App-of-apps that deploys all networking components |

## DNS Records

The following DNS records are automatically created by external-dns:

| Hostname | Service | Cloudflare Proxied |
|----------|---------|-------------------|
| `argocd.5dlabs.ai` | ArgoCD Server | Yes |
| `workflows.5dlabs.ai` | Argo Workflows | Yes |
| `headscale.5dlabs.ai` | Headscale Control | No (direct for WireGuard) |
| `hubble.5dlabs.ai` | Hubble UI | Yes |

## VPN Setup

### Prerequisites

1. Headscale must be deployed and running
2. ingress-nginx must be deployed
3. external-dns must be configured with Cloudflare credentials

### Server Setup

```bash
# Create a user
kubectl exec -n headscale deploy/headscale -- headscale users create admin

# Create a reusable auth key (for subnet router)
kubectl exec -n headscale deploy/headscale -- headscale preauthkeys create --user admin --reusable --expiration 720h

# Store the auth key for subnet router
kubectl create secret generic tailscale-auth -n headscale --from-literal=TS_AUTHKEY=<key-from-above>

# Restart subnet router to pick up the key
kubectl rollout restart deployment/tailscale-subnet-router -n headscale

# Approve the subnet routes in Headscale
kubectl exec -n headscale deploy/headscale -- headscale routes list
kubectl exec -n headscale deploy/headscale -- headscale routes enable -r <route-id>
```

### Client Setup (macOS)

1. Install Tailscale:
   ```bash
   brew install tailscale
   ```

2. Create a preauth key for your client:
   ```bash
   kubectl exec -n headscale deploy/headscale -- headscale preauthkeys create --user admin --expiration 24h
   ```

3. Connect to Headscale:
   ```bash
   sudo tailscale up --login-server https://headscale.5dlabs.ai --authkey <your-key>
   ```

4. Verify connection:
   ```bash
   tailscale status
   ```

### Accessing Cluster Services

Once connected via VPN, you can access:

- **Kubernetes Services**: `curl http://10.96.x.x:port`
- **Pod IPs**: `curl http://10.244.x.x:port`
- **MagicDNS**: `curl http://service.namespace.svc.cluster.local` (if DNS is configured)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Internet                                  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
              ┌─────────────▼─────────────┐
              │    Cloudflare (Proxied)   │
              │  argocd.5dlabs.ai         │
              │  workflows.5dlabs.ai      │
              │  hubble.5dlabs.ai         │
              └─────────────┬─────────────┘
                            │
              ┌─────────────▼─────────────┐
              │    headscale.5dlabs.ai    │◄────── Direct (WireGuard)
              │    (Not proxied)          │
              └─────────────┬─────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                     Kubernetes Cluster                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ ingress-    │  │  Headscale  │  │ Tailscale Subnet Router │  │
│  │   nginx     │  │   Server    │  │  (advertises 10.96/12,  │  │
│  │             │  │             │  │   10.244/16)            │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│                                              │                    │
│                                              ▼                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Cluster Network Access                       │   │
│  │  - Pod CIDR: 10.244.0.0/16                               │   │
│  │  - Service CIDR: 10.96.0.0/12                            │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Headscale not starting
Check logs: `kubectl logs -n headscale deploy/headscale`

### Subnet router not connecting
1. Check if auth key is set: `kubectl get secret tailscale-auth -n headscale`
2. Check logs: `kubectl logs -n headscale deploy/tailscale-subnet-router`

### DNS not resolving
Ensure external-dns has the correct Cloudflare credentials:
`kubectl get secret cloudflare-api-credentials -n external-dns`

### Certificates not issuing
Check cert-manager logs: `kubectl logs -n cert-manager deploy/cert-manager`

