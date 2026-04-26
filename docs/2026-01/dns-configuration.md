# DNS Configuration for Multi-Cluster Setup

This document describes the DNS namespace structure for separating home and Frankfurt clusters.

## DNS Namespace Structure

| Cluster | DNS Pattern | Example URLs |
|---------|-------------|--------------|
| **Frankfurt (Latitude)** | `*.fra.5dlabs.ai` | `argocd.fra.5dlabs.ai`, `workflows.fra.5dlabs.ai` |
| **Home (Local Lab)** | `*.home.5dlabs.ai` | `argocd.home.5dlabs.ai`, `workflows.home.5dlabs.ai` |

## Frankfurt Cluster (`admin-cto`)

**Cluster Details:**
- External API: `https://67.213.113.187:6443`
- Nodes: `admin-cto-cp1` (control plane), `admin-cto-worker1`
- Location: Latitude.sh bare metal
- Ingress Controller: NodePort (HTTP: 30716, HTTPS: 32145)

**DNS Records Required:**

| Hostname | Type | Value | Purpose |
|----------|------|-------|---------|
| `argocd.fra.5dlabs.ai` | A | `67.213.113.187` | ArgoCD web UI |
| `workflows.fra.5dlabs.ai` | A | `67.213.113.187` | Argo Workflows UI |
| `hubble.fra.5dlabs.ai` | A | `67.213.113.187` | Cilium Hubble UI |

**Port Configuration:**
- HTTP traffic → NodePort `:30716`
- HTTPS traffic → NodePort `:32145`

**Access URLs:**
- ArgoCD: `https://argocd.fra.5dlabs.ai:32145`
- Argo Workflows: `https://workflows.fra.5dlabs.ai:32145`
- Hubble UI: `https://hubble.fra.5dlabs.ai:32145`

## Home Cluster (`simple-cluster`)

**Cluster Details:**
- Control Plane: `192.168.1.77`
- Nodes: `talos-evr-4zu` (CP), `talos-irs-cis`, `talos-tq1-f47`
- Location: Local home lab
- Ingress Controller: NodePort (HTTP: 31251, HTTPS: 31981)

**DNS Records Required:**

| Hostname | Type | Value | Purpose |
|----------|------|-------|---------|
| `argocd.home.5dlabs.ai` | A | `192.168.1.77` | ArgoCD web UI |
| `workflows.home.5dlabs.ai` | A | `192.168.1.77` | Argo Workflows UI |
| `hubble.home.5dlabs.ai` | A | `192.168.1.77` | Hubble UI |
| `headscale.home.5dlabs.ai` | A | `192.168.1.77` | Headscale VPN |

**Access URLs:**
- ArgoCD: `https://argocd.home.5dlabs.ai:31981`
- Argo Workflows: `https://workflows.home.5dlabs.ai:31981`
- Hubble UI: `https://hubble.home.5dlabs.ai:31981`
- Headscale: `https://headscale.home.5dlabs.ai:31981`

## Implementation Status

### Frankfurt Cluster ✅
- [x] Ingress manifests created (`platform-ingress-fra.yaml`)
- [x] Certificates configured (`certificates-fra.yaml`)
- [x] Ingresses deployed to cluster
- [ ] DNS records configured in Cloudflare
- [ ] SSL certificates issued by cert-manager

### Home Cluster 📝
- [x] Ingress manifests created (`platform-ingress-home.yaml`)
- [x] Certificates configured (`certificates-home.yaml`)
- [ ] Ingresses deployed to cluster
- [ ] DNS records configured in Cloudflare
- [ ] SSL certificates issued by cert-manager

## Applying Configurations

### Frankfurt Cluster (Current)
```bash
# Already applied manually:
kubectl apply -f infra/gitops/manifests/ingress/platform-ingress-fra.yaml
kubectl apply -f infra/gitops/manifests/ingress/certificates-fra.yaml
```

### Home Cluster
```bash
# Switch to home cluster context
kubectl config use-context admin@simple-cluster

# Apply home ingress configuration
kubectl apply -f infra/gitops/manifests/ingress/platform-ingress-home.yaml
kubectl apply -f infra/gitops/manifests/ingress/certificates-home.yaml

# Update kustomization.yaml for home cluster
# Edit infra/gitops/manifests/ingress/kustomization.yaml:
# - certificates-home.yaml
# - platform-ingress-home.yaml
```

## Cloudflare DNS Configuration

Add the following DNS records in Cloudflare:

### Frankfurt Records (Public)
```
A    argocd.fra      67.213.113.187    Proxied: No
A    workflows.fra   67.213.113.187    Proxied: No
A    hubble.fra      67.213.113.187    Proxied: No
```

### Home Records (Private)
```
A    argocd.home     192.168.1.77      Proxied: No
A    workflows.home  192.168.1.77      Proxied: No
A    hubble.home     192.168.1.77      Proxied: No
A    headscale.home  192.168.1.77      Proxied: No
```

## Verification

### Test Frankfurt Cluster
```bash
# Set context
kubectl config use-context admin@admin-cto

# Check ingress status
kubectl get ingress -A

# Test DNS resolution
nslookup argocd.fra.5dlabs.ai

# Test HTTPS access
curl -k https://argocd.fra.5dlabs.ai:32145
```

### Test Home Cluster
```bash
# Set context
kubectl config use-context admin@simple-cluster

# Check ingress status
kubectl get ingress -A

# Test DNS resolution
nslookup argocd.home.5dlabs.ai

# Test HTTPS access
curl -k https://argocd.home.5dlabs.ai:31981
```

## Future Enhancements

1. **Load Balancer Integration:** Deploy MetalLB or use cloud load balancers to eliminate NodePort access
2. **Cloudflare Tunnel:** Set up tunnels for secure remote access without exposing NodePorts
3. **External DNS:** Configure external-dns operator to automatically manage Cloudflare DNS records
4. **Let's Encrypt:** Replace self-signed certificates with Let's Encrypt for publicly trusted certs

## Troubleshooting

### Ingress Not Working
```bash
# Check ingress controller
kubectl get pods -n ingress-nginx

# Check ingress status
kubectl describe ingress argocd-server -n argocd

# Check certificate
kubectl get certificate -A
kubectl describe certificate argocd-server-tls -n argocd
```

### DNS Not Resolving
```bash
# Check DNS records
nslookup argocd.fra.5dlabs.ai
dig argocd.fra.5dlabs.ai

# Flush DNS cache (macOS)
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
```

### Certificate Issues
```bash
# Check cert-manager
kubectl get pods -n cert-manager

# Check certificate status
kubectl get certificate -A -o wide

# Force certificate renewal
kubectl delete certificate argocd-server-tls -n argocd
kubectl apply -f infra/gitops/manifests/ingress/certificates-fra.yaml
```
