# Cloudflare Tunnel Implementation Guide

**Purpose:** Feature parity guide for migrating from NGrok to Cloudflare Tunnel in the CTO platform. This document outlines how to achieve the same architecture for webhooks and application publishing.

**Last Updated:** November 27, 2025

---

## Table of Contents

1. [Feature Comparison Matrix](#feature-comparison-matrix)
2. [Architecture Mapping](#architecture-mapping)
3. [Core Components](#core-components)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Tunnel Configuration](#tunnel-configuration)
6. [Webhook Implementation](#webhook-implementation)
7. [Dynamic Application Publishing](#dynamic-application-publishing)
8. [Security Features](#security-features)
9. [DNS Integration](#dns-integration)
10. [Secrets Management](#secrets-management)
11. [Migration Plan](#migration-plan)
12. [Cost Comparison](#cost-comparison)

---

## Feature Comparison Matrix

| Feature | NGrok | Cloudflare Tunnel | Parity Status |
|---------|-------|-------------------|---------------|
| **Tunneling** |
| Kubernetes Deployment | ✅ Operator + CRDs | ✅ cloudflared deployment | ✅ Available |
| Dynamic Tunnel Creation | ✅ Tunnel CRD | ⚠️ ConfigMap update + restart | 🔄 Different approach |
| Public URL Generation | ✅ `.status.url` | ✅ Static hostname via config | 🔄 Pre-configured |
| **Traffic Routing** |
| Gateway API Support | ✅ Native | ❌ Not supported | ❌ Manual config |
| Ingress Rules | ✅ Via Gateway | ✅ Via ingress config | ✅ Available |
| Path-based Routing | ✅ HTTPRoute | ✅ Ingress rules | ✅ Available |
| Wildcard Hostnames | ✅ Supported | ✅ Supported | ✅ Available |
| **Security** |
| TLS Termination | ✅ Automatic | ✅ Automatic | ✅ Available |
| Rate Limiting | ✅ TrafficPolicy | ✅ WAF Rules (zone-level) | ✅ Available |
| Security Headers | ✅ TrafficPolicy | ✅ Transform Rules | ✅ Available |
| DDoS Protection | ✅ Via NGrok edge | ✅ Native Cloudflare | ✅ Better |
| Access Control | ❌ Basic | ✅ Cloudflare Access | ✅ Better |
| **DNS** |
| Custom Domains | ✅ Domain CRD | ✅ Native (already on CF) | ✅ Simpler |
| Automatic DNS | ✅ external-dns | ✅ Direct config | ✅ Simpler |
| **Operations** |
| Operator Available | ✅ Official | ⚠️ Community (adyanth) | 🔄 Alternative |
| Helm Chart | ✅ Official | ✅ Community | ✅ Available |
| **Cost** |
| Base Cost | 💰 Per endpoint | ✅ Free | ✅ Better |
| Rate Limiting | 💰 Included | ✅ Free (basic) | ✅ Better |

---

## Architecture Mapping

### Current NGrok Architecture

```
Internet
    ↓
Cloudflare DNS (CNAME → ngrok)
    ↓
NGrok Edge Network
    ↓
NGrok Operator (Kubernetes)
    ↓
Gateway API → HTTPRoute → Service
```

### Target Cloudflare Tunnel Architecture

```
Internet
    ↓
Cloudflare Edge (DNS + CDN + WAF)
    ↓
Cloudflare Tunnel (cloudflared)
    ↓
Ingress Rules → Service
```

**Key Difference:** Cloudflare Tunnel removes the NGrok intermediary. Since you already use Cloudflare for DNS, traffic goes directly through Cloudflare's edge to your cluster.

---

## Core Components

### 1. cloudflared Daemon

The `cloudflared` daemon creates outbound-only connections to Cloudflare's edge, eliminating the need for inbound firewall rules.

**Official Docker Image:** `cloudflare/cloudflared:latest`

**How It Works:**
- Runs as a Kubernetes Deployment
- Authenticates via tunnel token (remotely-managed) or credentials file (locally-managed)
- Maintains persistent connections to Cloudflare edge
- Routes traffic based on ingress configuration

### 2. Tunnel Types

#### Remotely-Managed Tunnel (Recommended)

Configuration is stored in Cloudflare dashboard. The daemon only needs a token.

```yaml
# Kubernetes Deployment
env:
  - name: TUNNEL_TOKEN
    valueFrom:
      secretKeyRef:
        name: cloudflare-tunnel-token
        key: token
command:
  - cloudflared
  - tunnel
  - --no-autoupdate
  - run
```

**Pros:**
- Configuration changes don't require pod restarts
- Easier to manage via dashboard/API
- No local config files needed

**Cons:**
- Less GitOps-friendly (config not in repo)
- Requires Cloudflare API for automation

#### Locally-Managed Tunnel

Configuration stored in a ConfigMap. Full GitOps control.

```yaml
# config.yaml in ConfigMap
tunnel: <TUNNEL_UUID>
credentials-file: /etc/cloudflared/credentials.json
ingress:
  - hostname: github.public.5dlabs.ai
    path: /github/webhook
    service: http://github-eventsource-svc.automation:12000
  - hostname: "*.public.5dlabs.ai"
    service: http://nginx-ingress.infra:80
  - service: http_status:404
```

**Pros:**
- Full GitOps control
- Configuration in version control
- More transparent

**Cons:**
- Requires pod restart for config changes
- More complex secret management

### 3. Community Kubernetes Operator

**Project:** [adyanth/cloudflare-operator](https://github.com/adyanth/cloudflare-operator)

**Stars:** 610+ | **License:** Apache-2.0

**Features:**
- CRDs for Tunnel and TunnelBinding
- Automatic DNS record management
- Automatic ConfigMap updates
- Supports HTTP/TCP/UDP

**CRDs Provided:**
- `Tunnel` / `ClusterTunnel` - Creates and manages tunnels
- `TunnelBinding` - Maps services to tunnel hostnames

**Installation:**
```bash
kubectl apply -k 'https://github.com/adyanth/cloudflare-operator.git/config/default?ref=v0.13.1'
```

---

## Kubernetes Deployment

### Option A: Direct cloudflared Deployment (Simple)

```yaml
# cloudflared-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
  namespace: infra
spec:
  replicas: 2
  selector:
    matchLabels:
      app: cloudflared
  template:
    metadata:
      labels:
        app: cloudflared
    spec:
      containers:
        - name: cloudflared
          image: cloudflare/cloudflared:latest
          args:
            - tunnel
            - --no-autoupdate
            - --metrics
            - 0.0.0.0:2000
            - run
          env:
            - name: TUNNEL_TOKEN
              valueFrom:
                secretKeyRef:
                  name: cloudflare-tunnel-token
                  key: token
          livenessProbe:
            httpGet:
              path: /ready
              port: 2000
            initialDelaySeconds: 10
            periodSeconds: 10
          resources:
            limits:
              cpu: 500m
              memory: 256Mi
            requests:
              cpu: 100m
              memory: 128Mi
```

### Option B: Using cloudflare-operator (Recommended for Dynamic)

```yaml
# cloudflare-operator-secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: cloudflare-api
  namespace: cloudflare-operator-system
stringData:
  CLOUDFLARE_API_TOKEN: "<your-api-token>"
  CLOUDFLARE_ACCOUNT_ID: "<your-account-id>"
---
# tunnel.yaml
apiVersion: cloudflare-operator.io/v1alpha2
kind: ClusterTunnel
metadata:
  name: cto-tunnel
spec:
  cloudflareCredentialsRef:
    name: cloudflare-api
  domain: 5dlabs.ai
  size: 2  # Number of replicas
---
# tunnel-binding.yaml
apiVersion: cloudflare-operator.io/v1alpha2
kind: TunnelBinding
metadata:
  name: github-webhooks
  namespace: automation
spec:
  tunnelRef:
    kind: ClusterTunnel
    name: cto-tunnel
  frontend:
    domain: github.public.5dlabs.ai
    path: /github/webhook
  backend:
    protocol: http
    target: github-eventsource-svc:12000
```

---

## Tunnel Configuration

### Ingress Rules Structure

```yaml
# cloudflared-config.yaml (ConfigMap)
tunnel: <TUNNEL-UUID>
credentials-file: /etc/cloudflared/credentials.json

# Origin configuration (applies to all)
originRequest:
  connectTimeout: 30s
  noTLSVerify: false
  http2Origin: true

ingress:
  # GitHub Webhooks - highest priority
  - hostname: github.public.5dlabs.ai
    path: /github/webhook
    service: http://github-eventsource-svc.automation:12000
    originRequest:
      connectTimeout: 10s

  # Application previews (wildcard)
  - hostname: "*.preview.5dlabs.ai"
    service: http://nginx-ingress.infra:80

  # Production applications
  - hostname: "*.public.5dlabs.ai"
    service: http://nginx-ingress.infra:80

  # Catch-all (required)
  - service: http_status:404
```

### Supported Service Types

| Protocol | Service Format | Example |
|----------|---------------|---------|
| HTTP | `http://host:port` | `http://my-service:80` |
| HTTPS | `https://host:port` | `https://my-service:443` |
| TCP | `tcp://host:port` | `tcp://postgres:5432` |
| SSH | `ssh://host:port` | `ssh://ssh-server:22` |
| RDP | `rdp://host:port` | `rdp://windows:3389` |
| Unix Socket | `unix:/path` | `unix:/var/run/app.sock` |
| Hello World (test) | `hello_world` | Built-in test server |
| HTTP Status | `http_status:CODE` | `http_status:404` |

---

## Webhook Implementation

### Matching Current NGrok Webhook Setup

**Current NGrok Flow:**
```
github.public.5dlabs.ai/github/webhook
  → NGrok Edge
  → Gateway API (public-gateway)
  → HTTPRoute
  → github-eventsource-svc:12000
```

**Cloudflare Tunnel Equivalent:**
```
github.public.5dlabs.ai/github/webhook
  → Cloudflare Edge
  → cloudflared (ingress rule match)
  → github-eventsource-svc:12000
```

### Cloudflare Tunnel Config for Webhooks

```yaml
ingress:
  # Exact path match for webhook
  - hostname: github.public.5dlabs.ai
    path: /github/webhook
    service: http://github-eventsource-svc.automation:12000
    originRequest:
      # Keep connection alive for streaming
      keepAliveConnections: 100
      keepAliveTimeout: 90s
```

### Webhook Path Patterns

Cloudflare Tunnel uses Go regex for path matching:

```yaml
ingress:
  # Exact path
  - hostname: api.5dlabs.ai
    path: /webhook
    service: http://webhook-handler:8080

  # Path prefix (regex)
  - hostname: api.5dlabs.ai
    path: ^/api/v[0-9]+/.*
    service: http://api-server:8080

  # File extensions (regex)
  - hostname: static.5dlabs.ai
    path: \.(jpg|png|css|js)$
    service: http://cdn-cache:80
```

---

## Dynamic Application Publishing

### Challenge: Bolt's Dynamic Tunnel Creation

Current Bolt scripts create NGrok `Tunnel` CRDs dynamically:

```bash
# Current Bolt approach
kubectl apply -f - <<EOF
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: Tunnel
metadata:
  name: task-${TASK_ID}-preview
spec:
  forwardsTo: ${service}:${port}
EOF

# Get URL from status
tunnel_url=$(kubectl get tunnel "$name" -o jsonpath='{.status.url}')
```

### Solution A: Use cloudflare-operator TunnelBinding

```bash
# Bolt script modification for cloudflare-operator
kubectl apply -f - <<EOF
apiVersion: cloudflare-operator.io/v1alpha2
kind: TunnelBinding
metadata:
  name: task-${TASK_ID}-preview
  namespace: ${PREVIEW_NAMESPACE}
spec:
  tunnelRef:
    kind: ClusterTunnel
    name: cto-tunnel
  frontend:
    domain: task-${TASK_ID}.preview.5dlabs.ai
  backend:
    protocol: http
    target: ${service}:${port}
EOF

# URL is deterministic (no status check needed)
PREVIEW_URL="https://task-${TASK_ID}.preview.5dlabs.ai"
```

### Solution B: Internal Ingress Controller + Single Tunnel

Route all preview/production traffic through an internal NGINX ingress:

```yaml
# Single tunnel entry
ingress:
  - hostname: "*.preview.5dlabs.ai"
    service: http://ingress-nginx-controller.infra:80
  - hostname: "*.prod.5dlabs.ai"
    service: http://ingress-nginx-controller.infra:80
```

Bolt creates standard Kubernetes Ingress resources:

```bash
# Bolt creates Ingress instead of Tunnel
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: task-${TASK_ID}-preview
  namespace: ${PREVIEW_NAMESPACE}
spec:
  ingressClassName: nginx
  rules:
    - host: task-${TASK_ID}.preview.5dlabs.ai
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: ${service}
                port:
                  number: ${port}
EOF

PREVIEW_URL="https://task-${TASK_ID}.preview.5dlabs.ai"
```

### Solution C: Cloudflare API for Dynamic Hostnames

Use Cloudflare API to add/remove public hostnames:

```bash
# Add hostname via API
curl -X POST "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  -H "Content-Type: application/json" \
  --data '{
    "config": {
      "ingress": [
        {
          "hostname": "task-123.preview.5dlabs.ai",
          "service": "http://my-service:80"
        }
      ]
    }
  }'
```

**Recommendation:** Solution B (Internal Ingress) is most GitOps-friendly and matches existing Kubernetes patterns.

---

## Security Features

### Rate Limiting (Zone-Level WAF Rules)

Create rate limiting rules in Cloudflare Dashboard or via API:

```json
{
  "description": "Webhook Rate Limit",
  "expression": "(http.host eq \"github.public.5dlabs.ai\" and starts_with(http.request.uri.path, \"/github/webhook\"))",
  "action": "block",
  "ratelimit": {
    "characteristics": ["ip.src"],
    "period": 60,
    "requests_per_period": 600,
    "mitigation_timeout": 60
  }
}
```

**Terraform Configuration:**
```hcl
resource "cloudflare_ruleset" "webhook_rate_limit" {
  zone_id = var.zone_id
  name    = "Webhook Rate Limiting"
  kind    = "zone"
  phase   = "http_ratelimit"

  rules {
    action = "block"
    expression = "(http.host eq \"github.public.5dlabs.ai\" and starts_with(http.request.uri.path, \"/github/webhook\"))"
    ratelimit {
      characteristics     = ["ip.src"]
      period             = 60
      requests_per_period = 600
      mitigation_timeout  = 60
    }
  }
}
```

### Security Headers (Transform Rules)

Create Response Header Modification Rules:

```json
{
  "description": "Add Security Headers",
  "expression": "(http.host contains \"5dlabs.ai\")",
  "action": "rewrite",
  "action_parameters": {
    "headers": {
      "Strict-Transport-Security": {
        "operation": "set",
        "value": "max-age=31536000; includeSubDomains"
      },
      "X-Content-Type-Options": {
        "operation": "set",
        "value": "nosniff"
      },
      "X-Frame-Options": {
        "operation": "set",
        "value": "DENY"
      },
      "X-XSS-Protection": {
        "operation": "set",
        "value": "1; mode=block"
      }
    }
  }
}
```

**Terraform Configuration:**
```hcl
resource "cloudflare_ruleset" "security_headers" {
  zone_id = var.zone_id
  name    = "Security Headers"
  kind    = "zone"
  phase   = "http_response_headers_transform"

  rules {
    action = "rewrite"
    expression = "(http.host contains \"5dlabs.ai\")"
    action_parameters {
      headers {
        name      = "Strict-Transport-Security"
        operation = "set"
        value     = "max-age=31536000; includeSubDomains"
      }
      headers {
        name      = "X-Content-Type-Options"
        operation = "set"
        value     = "nosniff"
      }
      headers {
        name      = "X-Frame-Options"
        operation = "set"
        value     = "DENY"
      }
      headers {
        name      = "X-XSS-Protection"
        operation = "set"
        value     = "1; mode=block"
      }
    }
  }
}
```

### Cloudflare Access (Optional Enhancement)

Protect preview environments with authentication:

```yaml
# Access Application for previews
# Configure in Cloudflare Zero Trust dashboard
Application:
  name: Preview Environments
  domain: "*.preview.5dlabs.ai"
  policies:
    - name: Allow Team Members
      decision: allow
      include:
        - emails_ending_in: "@5dlabs.ai"
    - name: Bypass Webhooks
      decision: bypass
      include:
        - path: "/health"
        - path: "/ready"
```

---

## DNS Integration

### Simplified DNS (No external-dns needed)

Since you already use Cloudflare for DNS:

1. **Tunnel creates DNS automatically** when using remotely-managed tunnels
2. **Manual CNAME** for locally-managed tunnels:
   ```
   github.public.5dlabs.ai  CNAME  <tunnel-id>.cfargotunnel.com
   *.preview.5dlabs.ai      CNAME  <tunnel-id>.cfargotunnel.com
   ```

### Terraform DNS Configuration

```hcl
resource "cloudflare_record" "github_webhook" {
  zone_id = var.zone_id
  name    = "github.public"
  value   = "${var.tunnel_id}.cfargotunnel.com"
  type    = "CNAME"
  proxied = true
}

resource "cloudflare_record" "preview_wildcard" {
  zone_id = var.zone_id
  name    = "*.preview"
  value   = "${var.tunnel_id}.cfargotunnel.com"
  type    = "CNAME"
  proxied = true
}
```

---

## Secrets Management

### Vault Secrets Structure

```
secret/
├── cloudflare-tunnel/
│   ├── TUNNEL_TOKEN      # For remotely-managed tunnel
│   └── TUNNEL_ID         # Tunnel UUID
├── cloudflare-api/
│   ├── API_TOKEN         # API token (for operator/automation)
│   └── ACCOUNT_ID        # Account ID
└── github-webhooks/
    └── secret            # (unchanged)
```

### VaultStaticSecret for Cloudflare Tunnel

```yaml
# infra/vault/secrets/cloudflare-tunnel.yaml
apiVersion: secrets.hashicorp.com/v1beta1
kind: VaultStaticSecret
metadata:
  name: cloudflare-tunnel-token
  namespace: infra
spec:
  vaultAuthRef: infra/vault-auth
  mount: secret
  path: cloudflare-tunnel
  type: kv-v2
  refreshAfter: 1h
  destination:
    create: true
    name: cloudflare-tunnel-token
```

---

## Migration Plan

### Phase 1: Preparation (Week 1)

1. **Create Cloudflare Tunnel**
   - Go to Cloudflare One → Networks → Tunnels
   - Create tunnel named `cto-main`
   - Save tunnel token to Vault

2. **Deploy cloudflared (parallel)**
   - Deploy cloudflared alongside NGrok
   - Configure for test hostname only
   - Verify connectivity

3. **Configure Security Rules**
   - Create rate limiting rules
   - Create security header transform rules

### Phase 2: Webhook Migration (Week 2)

1. **Add Webhook Ingress Rule**
   ```yaml
   - hostname: github-cf.public.5dlabs.ai  # New test hostname
     path: /github/webhook
     service: http://github-eventsource-svc.automation:12000
   ```

2. **Test with GitHub Webhook**
   - Add second webhook URL to test repo
   - Verify events flow through

3. **Switch DNS**
   - Update `github.public.5dlabs.ai` CNAME to Cloudflare Tunnel
   - Monitor for issues

### Phase 3: Application Publishing (Week 3)

1. **Update Bolt Scripts**
   - Modify to create Ingress resources
   - Use predictable URL pattern
   - Test preview deployments

2. **Deploy Internal Ingress**
   - Configure NGINX ingress for preview/prod wildcards
   - Add tunnel ingress rules for wildcards

3. **Test End-to-End**
   - Create test PR
   - Verify preview URL works
   - Verify production deployment

### Phase 4: Cleanup (Week 4)

1. **Remove NGrok Resources**
   - Delete NGrok Gateway Application
   - Delete NGrok Operator
   - Remove NGrok secrets from Vault

2. **Update Documentation**
   - Update all references to NGrok
   - Document new patterns

3. **Remove external-dns NGrok Integration**
   - Simplify external-dns config
   - Remove NGrok-specific sources

---

## Cost Comparison

| Cost Item | NGrok (Current) | Cloudflare Tunnel |
|-----------|-----------------|-------------------|
| **Base Tunneling** | $8/endpoint/month (paid) | Free |
| **Custom Domains** | Included in paid | Free (you own DNS) |
| **Rate Limiting** | Included | Free (basic) / WAF (included) |
| **TLS Certificates** | Included | Free (Cloudflare auto) |
| **DDoS Protection** | Basic | Enterprise-grade (free) |
| **Estimated Monthly** | ~$50-100+ | $0 (already on Cloudflare) |

**Note:** You're already paying for Cloudflare for DNS and other services. Cloudflare Tunnel is included at no additional cost.

---

## Files to Create/Modify

### New Files

```
infra/gitops/applications/
├── cloudflared.yaml                    # ArgoCD Application

infra/gitops/resources/cloudflare-tunnel/
├── kustomization.yaml
├── deployment.yaml                     # cloudflared deployment
├── configmap.yaml                      # Tunnel config (if locally-managed)
├── service.yaml                        # Metrics service
└── README.md

infra/vault/secrets/
└── cloudflare-tunnel.yaml              # VaultStaticSecret
```

### Modified Files

```
infra/charts/controller/agent-templates/code/integration/
├── container-bolt.sh.hbs               # Change Tunnel → Ingress
├── container-bolt-preview.sh.hbs       # Change Tunnel → Ingress
└── container-bolt-cleanup.sh.hbs       # Update cleanup logic
```

### Files to Remove (After Migration)

```
infra/gitops/applications/
├── ngrok-operator.yaml
└── ngrok-gateway.yaml

infra/gitops/resources/ngrok-gateway/
└── (entire directory)
```

---

## Summary

### Key Implementation Decisions

1. **Tunnel Type:** Remotely-managed (simpler) or locally-managed (GitOps)
2. **Operator:** Direct deployment or cloudflare-operator
3. **Dynamic Apps:** Internal ingress controller (recommended)
4. **Security:** Zone-level WAF + Transform Rules

### Advantages Over NGrok

- ✅ **Cost:** Free vs. per-endpoint pricing
- ✅ **Performance:** Direct path through Cloudflare (no extra hop)
- ✅ **Security:** Native Cloudflare WAF, Access, DDoS protection
- ✅ **DNS:** Already on Cloudflare, no external-dns complexity
- ✅ **Reliability:** Cloudflare's global network

### Challenges

- 🔄 **Dynamic Tunnels:** Requires different approach (Ingress-based)
- 🔄 **Operator Maturity:** Community vs. official
- 🔄 **Configuration:** No Gateway API support

### Recommendation

**Use Cloudflare Tunnel with internal NGINX Ingress for application publishing.** This provides:
- Standard Kubernetes patterns (Ingress resources)
- GitOps-friendly configuration
- Predictable URL generation
- Full security feature parity

