# NGrok Implementation Reference

**Purpose:** Complete documentation of the CTO platform's NGrok implementation for public traffic ingress, webhooks, and application publishing. This serves as a reference for the Cloudflare Tunnel migration.

**Last Updated:** November 27, 2025

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [Webhook Infrastructure](#webhook-infrastructure)
4. [Application Publishing (Bolt)](#application-publishing-bolt)
5. [Gateway API Configuration](#gateway-api-configuration)
6. [Traffic Policies](#traffic-policies)
7. [DNS Integration](#dns-integration)
8. [Secrets & Authentication](#secrets--authentication)
9. [CRDs & Resources](#crds--resources)
10. [Deployment Flow](#deployment-flow)
11. [Migration Considerations](#migration-considerations)

---

## Architecture Overview

### High-Level Traffic Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Internet                                     │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   Cloudflare DNS      │  ← external-dns manages
                    │   (5dlabs.ai zone)    │    CNAME → ngrok endpoints
                    └───────────┬───────────┘
                                │
                    ┌───────────▼───────────┐
                    │   NGrok Edge Network  │  ← TLS termination
                    │   (Global PoPs)       │    Rate limiting
                    └───────────┬───────────┘    Security headers
                                │
                    ┌───────────▼───────────┐
                    │   NGrok Operator      │  ← Kubernetes controller
                    │   (infra namespace)   │    Manages tunnels/gateways
                    └───────────┬───────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────▼───────┐    ┌──────────▼──────────┐    ┌───────▼───────┐
│   Webhooks    │    │  Public Gateway     │    │    Tunnels    │
│   (Argo       │    │  (Gateway API)      │    │   (App CRDs)  │
│   Events)     │    │                     │    │               │
└───────────────┘    └─────────────────────┘    └───────────────┘
```

### Namespaces Involved

| Namespace | Role |
|-----------|------|
| `infra` | NGrok operator deployment, credentials, domain resources |
| `automation` | Gateway, traffic policies, GitHub webhook HTTPRoute |
| `cto-preview-task-*` | Preview deployment tunnels (dynamic) |
| `cto-prod-task-*` | Production deployment tunnels (dynamic) |

---

## Core Components

### 1. NGrok Operator

**ArgoCD Application:** `ngrok-operator`

```yaml
# infra/gitops/applications/ngrok-operator.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: ngrok-operator
  namespace: argocd
spec:
  source:
    repoURL: https://charts.ngrok.com
    chart: ngrok-operator
    targetRevision: 0.21.1
    helm:
      parameters:
        - name: credentials.secret.name
          value: ngrok-operator-credentials
        - name: replicaCount
          value: "2"
  destination:
    namespace: infra
```

**Key Features:**
- Helm chart version: `0.21.1`
- Runs 2 replicas for HA
- Credentials from Vault-synced secret
- Deployed to `infra` namespace

### 2. NGrok Gateway Application

**ArgoCD Application:** `ngrok-gateway`

```yaml
# infra/gitops/applications/ngrok-gateway.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: ngrok-gateway
  namespace: argocd
spec:
  source:
    repoURL: https://github.com/5dlabs/cto
    targetRevision: main
    path: infra/gitops/resources/ngrok-gateway
  destination:
    namespace: default
```

**Resources Managed:**
- `GatewayClass` - Defines ngrok as gateway controller
- `Gateway` - Public gateway for HTTPS traffic
- `NgrokTrafficPolicy` - Security and rate limiting
- `Domain` - Custom domain registration
- HTTPRoute resources (optional, for redirects)

---

## Webhook Infrastructure

### Purpose
GitHub webhooks are the trigger mechanism for all automation in the CTO platform. Every PR, push, comment, and label change flows through this endpoint.

### Components

#### 1. EventSource (Argo Events)

```yaml
# infra/gitops/resources/github-webhooks/eventsource.yaml
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: github
  namespace: automation
spec:
  github:
    org:
      events:
        - "*"  # All GitHub events
      organizations:
        - "5dlabs"
      webhook:
        endpoint: /github/webhook
        port: "12000"
      secret:
        name: github-webhook-secret
        key: secret
```

**Configuration:**
- Listens on port `12000`
- Receives ALL GitHub events (`*`)
- Authenticates with webhook secret from Vault
- Endpoint path: `/github/webhook`

#### 2. Service (Internal)

```yaml
# infra/gitops/resources/github-webhooks/networking/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: github-eventsource-svc
  namespace: automation
spec:
  type: ClusterIP
  selector:
    eventsource-name: github
  ports:
    - name: webhook
      port: 12000
      targetPort: 12000
```

#### 3. HTTPRoute (Gateway API)

```yaml
# infra/gitops/resources/github-webhooks/networking/httproute.yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: github-webhooks
  namespace: automation
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: automation
  hostnames:
    - "github.public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: /github/webhook
      backendRefs:
        - kind: Service
          name: github-eventsource-svc
          port: 12000
```

**Traffic Flow:**
```
GitHub → github.public.5dlabs.ai/github/webhook
       → NGrok Edge (TLS + rate limiting)
       → public-gateway (Gateway API)
       → github-eventsource-svc:12000
       → Argo Events EventSource
       → Sensors trigger workflows
```

### Active Sensors (17 total)

| Sensor | Purpose |
|--------|---------|
| `atlas-batch-integration-sensor` | Batch PR processing |
| `atlas-conflict-monitor-sensor` | Merge conflict detection |
| `atlas-pr-monitor-sensor` | PR lifecycle monitoring |
| `bolt-production-deployment-sensor` | Production deployments |
| `ci-failure-remediation-sensor` | CI failure handling |
| `stage-aware-cleo-approval-sensor` | Cleo code review triggers |
| `stage-aware-pr-created` | New PR handling |
| `stage-aware-pr-merged-sensor` | Post-merge actions |
| `stage-aware-tess-approval-sensor` | QA approval handling |
| `tess-label-fallback-sensor` | Tess labeling fallback |
| `remediation-feedback-sensor` | Remediation feedback loop |
| `play-workflow-*-sensor` | Play project workflows |

---

## Application Publishing (Bolt)

### Overview

Bolt is responsible for making applications publicly accessible. It operates in three modes:

1. **Bolt-Preview** - Creates preview deployments for PRs
2. **Bolt-Production** - Creates production deployments after merge
3. **Bolt-Monitor** - Continuous health monitoring (daemon)

### Tunnel CRD Pattern

Bolt creates `Tunnel` CRDs to expose services:

```yaml
# Created by Bolt-Preview/Bolt-Production scripts
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: Tunnel
metadata:
  name: task-{id}-preview  # or task-{id}-prod
  namespace: cto-preview-task-{id}
  labels:
    task-id: "{id}"
    environment: preview  # or production
    managed-by: bolt
spec:
  forwardsTo: {service-name}:{port}
```

**Tunnel Lifecycle:**
1. ArgoCD deploys application to namespace
2. Bolt detects service in namespace
3. Bolt creates Tunnel CRD
4. NGrok operator provisions tunnel
5. Tunnel URL appears in `.status.url`
6. Bolt posts URL to GitHub PR

### Preview Deployment Flow

```
PR Created/Updated
       ↓
Bolt-Preview Sensor triggers
       ↓
Create namespace: cto-preview-task-{id}
       ↓
Create ArgoCD Application (from PR branch)
       ↓
Wait for sync + healthy status
       ↓
Detect service in namespace
       ↓
Create Tunnel CRD
       ↓
Wait for tunnel URL
       ↓
Post preview URL to PR comment
```

### Production Deployment Flow

```
PR Merged + ready-for-production label
       ↓
Bolt-Production Sensor triggers
       ↓
Cleanup preview deployment
  - Delete preview ArgoCD app
  - Delete preview namespace
  - Delete preview tunnel
       ↓
Create namespace: cto-prod-task-{id}
       ↓
Create ArgoCD Application (from main branch)
       ↓
Wait for sync + healthy status
       ↓
Create production Tunnel CRD
       ↓
Post production URL to PR comment
```

### Bolt Scripts Location

```
infra/charts/controller/agent-templates/code/integration/
├── container-bolt.sh.hbs          # Production deployment
├── container-bolt-preview.sh.hbs  # Preview deployment
├── container-bolt-cleanup.sh.hbs  # Cleanup on PR close
└── container-bolt-monitor.sh.hbs  # Health monitoring
```

---

## Gateway API Configuration

### GatewayClass

```yaml
# infra/gitops/resources/ngrok-gateway/gatewayclass.yaml
apiVersion: gateway.networking.k8s.io/v1
kind: GatewayClass
metadata:
  name: ngrok
spec:
  controllerName: ngrok.com/gateway-controller
```

### Gateway

```yaml
# infra/gitops/resources/ngrok-gateway/gateway.yaml
apiVersion: gateway.networking.k8s.io/v1
kind: Gateway
metadata:
  name: public-gateway
  namespace: automation
  annotations:
    external-dns.alpha.kubernetes.io/hostname: "github.public.5dlabs.ai"
    k8s.ngrok.com/traffic-policy: security-policy
spec:
  gatewayClassName: ngrok
  listeners:
    - name: github
      protocol: HTTPS
      port: 443
      hostname: "github.public.5dlabs.ai"
      allowedRoutes:
        namespaces:
          from: All
```

**Current State (Cost Optimized):**
- Only `github.public.5dlabs.ai` listener active
- Other domains commented out to reduce NGrok endpoint costs
- Can be re-enabled by uncommenting listeners

**Full Capability (When Enabled):**
```yaml
listeners:
  - name: https
    hostname: "public.5dlabs.ai"
  - name: github
    hostname: "github.public.5dlabs.ai"
  - name: root
    hostname: "5dlabs.ai"
  - name: www
    hostname: "www.5dlabs.ai"
```

### Domain Registration

```yaml
# infra/gitops/resources/ngrok-gateway/domain.yaml
apiVersion: ingress.k8s.ngrok.com/v1alpha1
kind: Domain
metadata:
  name: github-public-5dlabs-ai
  namespace: automation
spec:
  domain: "github.public.5dlabs.ai"
```

---

## Traffic Policies

### Security Policy

```yaml
# infra/gitops/resources/ngrok-gateway/traffic-policy.yaml
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: NgrokTrafficPolicy
metadata:
  name: security-policy
  namespace: automation
spec:
  policy:
    on_http_request:
      # Add forwarding headers
      - actions:
          - type: add-headers
            config:
              headers:
                "X-Forwarded-Proto": "https"
                "X-Ngrok-Endpoint": "true"
      # Rate limiting
      - actions:
          - type: rate-limit
            config:
              name: "webhook-rate-limit"
              algorithm: "sliding_window"
              capacity: 600        # 600 requests per window
              rate: "60s"          # 60 second window
              bucket_key:
                - "conn.client_ip"
    
    on_http_response:
      # Security headers
      - actions:
          - type: add-headers
            config:
              headers:
                "X-Content-Type-Options": "nosniff"
                "X-Frame-Options": "DENY"
                "X-XSS-Protection": "1; mode=block"
                "Strict-Transport-Security": "max-age=31536000; includeSubDomains"
```

**Rate Limiting:**
- Algorithm: Sliding window
- Capacity: 600 requests per 60 seconds (10 req/sec average)
- Bucket key: Client IP address

**Security Headers Applied:**
- HSTS with 1 year max-age
- XSS protection enabled
- Frame denial (clickjacking protection)
- Content type sniffing prevention

---

## DNS Integration

### External-DNS Configuration

```yaml
# infra/gitops/applications/external-dns.yaml
source:
  helm:
    values: |
      provider: cloudflare
      domainFilters:
        - 5dlabs.ai
      sources:
        - gateway-httproute
        - gateway-grpcroute
        - service
        - ingress
      policy: upsert-only
      registry: txt
      txtOwnerId: external-dns-5dlabs
      txtPrefix: external-dns-
      cloudflare:
        proxied: true
```

**How It Works:**

1. Gateway resource has annotation:
   ```yaml
   external-dns.alpha.kubernetes.io/hostname: "github.public.5dlabs.ai"
   ```

2. External-DNS watches for Gateway API resources

3. Creates DNS records in Cloudflare:
   ```
   github.public.5dlabs.ai.  CNAME  <ngrok-endpoint>.ngrok-cname.com
   external-dns-github.public.5dlabs.ai.  TXT  "heritage=external-dns..."
   ```

4. Cloudflare proxy (orange cloud) is enabled for DDoS protection

**DNS Propagation:**
- Automatic via external-dns controller
- Update interval: 1 minute
- Records are CNAME pointing to NGrok's global edge

---

## Secrets & Authentication

### Vault Secrets Structure

```
secret/
├── ngrok-credentials/
│   ├── API_KEY      # NGrok API key
│   └── AUTHTOKEN    # NGrok auth token
├── cloudflare/
│   └── api-token    # Cloudflare API token for external-dns
└── github-webhooks/
    └── secret       # GitHub webhook HMAC secret
```

### VaultStaticSecret for NGrok

```yaml
# infra/vault/secrets/infrastructure.yaml
apiVersion: secrets.hashicorp.com/v1beta1
kind: VaultStaticSecret
metadata:
  name: ngrok-operator-credentials
  namespace: infra
spec:
  vaultAuthRef: infra/vault-auth
  mount: secret
  path: ngrok-credentials
  type: kv-v2
  refreshAfter: 1h
  destination:
    create: true
    name: ngrok-operator-credentials
```

### Secret Keys Required

| Secret | Keys | Used By |
|--------|------|---------|
| `ngrok-operator-credentials` | `API_KEY`, `AUTHTOKEN` | NGrok operator |
| `cloudflare-api-credentials` | `api-token` | external-dns |
| `github-webhook-secret` | `secret` | Argo Events EventSource |

---

## CRDs & Resources

### NGrok CRDs

| CRD | API Group | Purpose |
|-----|-----------|---------|
| `Tunnel` | `ngrok.k8s.ngrok.com/v1alpha1` | Creates individual ngrok tunnels |
| `Domain` | `ingress.k8s.ngrok.com/v1alpha1` | Registers custom domains |
| `NgrokTrafficPolicy` | `ngrok.k8s.ngrok.com/v1alpha1` | Defines traffic rules |

### Gateway API CRDs

| CRD | API Group | Purpose |
|-----|-----------|---------|
| `GatewayClass` | `gateway.networking.k8s.io/v1` | Defines controller |
| `Gateway` | `gateway.networking.k8s.io/v1` | Listener configuration |
| `HTTPRoute` | `gateway.networking.k8s.io/v1` | HTTP routing rules |

### Resource Relationships

```
GatewayClass (ngrok)
       │
       └── Gateway (public-gateway)
               │
               ├── Listener (github.public.5dlabs.ai:443)
               │
               ├── NgrokTrafficPolicy (security-policy)
               │
               └── HTTPRoute (github-webhooks)
                       │
                       └── Service (github-eventsource-svc:12000)
```

---

## Deployment Flow

### Full Platform Startup Sequence

```
1. ArgoCD syncs ngrok-operator
   └── Deploys NGrok operator pods to infra namespace
   
2. VaultStaticSecret syncs ngrok-operator-credentials
   └── Creates K8s secret from Vault
   
3. ArgoCD syncs ngrok-gateway
   └── Creates GatewayClass, Gateway, Domain, TrafficPolicy
   
4. ArgoCD syncs external-dns
   └── Creates DNS records in Cloudflare
   
5. ArgoCD syncs github-webhooks
   └── Creates EventSource, Service, HTTPRoute
   
6. GitHub webhooks flow to cluster
   └── Sensors trigger agent workflows
```

### Application Deployment Sequence

```
1. PR created/updated
   └── GitHub webhook → Argo Events
   
2. Sensor triggers Bolt-Preview
   └── Creates preview namespace
   
3. GitOps creates ArgoCD Application
   └── ArgoCD syncs from PR branch
   
4. Application becomes healthy
   └── Bolt creates Tunnel CRD
   
5. NGrok operator provisions tunnel
   └── .status.url populated
   
6. Bolt retrieves URL
   └── Posts to GitHub PR
```

---

## Migration Considerations

### What Cloudflare Tunnel Must Replace

| NGrok Component | Cloudflare Equivalent | Notes |
|-----------------|----------------------|-------|
| NGrok Operator | cloudflared (cloudflare-tunnel-operator?) | Tunnel management |
| `Tunnel` CRD | Cloudflare Tunnel + Ingress Rule | Per-service exposure |
| `Domain` CRD | Cloudflare DNS (automatic) | Already using Cloudflare DNS |
| `NgrokTrafficPolicy` | Cloudflare Access / WAF Rules | Rate limiting, headers |
| `Gateway` | Cloudflare Tunnel routing | Listener configuration |

### API/CRD Compatibility

**Current Bolt Scripts Use:**
```bash
# Create Tunnel CRD
kubectl apply -f - <<EOF
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: Tunnel
metadata:
  name: $tunnel_name
  namespace: $namespace
spec:
  forwardsTo: $service_name:$service_port
EOF

# Get Tunnel URL
kubectl get tunnel "$tunnel_name" -n "$namespace" \
  -o jsonpath='{.status.url}'
```

**Cloudflare Needs:**
- Equivalent CRD or operator
- `.status.url` or equivalent for URL retrieval
- Same namespace-scoped behavior
- Dynamic tunnel creation/deletion

### Webhook Endpoint Requirements

- **Current:** `https://github.public.5dlabs.ai/github/webhook`
- **Must maintain:** Same URL structure (GitHub webhooks configured)
- **Or:** Update all GitHub webhook configurations

### Traffic Policy Features Needed

1. **Rate Limiting**
   - Per-IP rate limiting
   - Sliding window algorithm
   - Configurable capacity/rate

2. **Security Headers**
   - HSTS
   - X-Frame-Options
   - X-Content-Type-Options
   - X-XSS-Protection

3. **Request Headers**
   - X-Forwarded-Proto
   - Custom identification headers

### DNS Considerations

- Already using Cloudflare for DNS
- external-dns currently points CNAMEs to NGrok
- Migration: Point directly to Cloudflare Tunnel endpoints
- Cloudflare proxy already enabled (orange cloud)

### Secrets Migration

| Current Path | New Path (Suggested) |
|--------------|---------------------|
| `secret/ngrok-credentials` | `secret/cloudflare-tunnel` |
| Keys: `API_KEY`, `AUTHTOKEN` | Keys: `TUNNEL_TOKEN`, etc. |

---

## Files Reference

### Core Configuration Files

```
infra/gitops/applications/
├── ngrok-operator.yaml       # Operator ArgoCD app
├── ngrok-gateway.yaml        # Gateway resources ArgoCD app
├── external-dns.yaml         # DNS management
└── github-webhooks.yaml      # Webhook resources

infra/gitops/resources/ngrok-gateway/
├── gatewayclass.yaml         # ngrok GatewayClass
├── gateway.yaml              # Public gateway
├── domain.yaml               # Custom domain registration
├── traffic-policy.yaml       # Security & rate limiting
├── redirect-route.yaml       # Domain redirects (disabled)
├── kustomization.yaml        # Kustomize bundle
└── README.md                 # Gateway documentation

infra/gitops/resources/github-webhooks/
├── eventsource.yaml          # Argo Events EventSource
├── kustomization.yaml        # Kustomize bundle
└── networking/
    ├── httproute.yaml        # Gateway API HTTPRoute
    └── service.yaml          # Internal service

infra/vault/secrets/
└── infrastructure.yaml       # VaultStaticSecret definitions
```

### Bolt Agent Scripts

```
infra/charts/controller/agent-templates/code/integration/
├── container-bolt.sh.hbs          # Production deployment
├── container-bolt-preview.sh.hbs  # Preview deployment
├── container-bolt-cleanup.sh.hbs  # Cleanup script
└── container-bolt-monitor.sh.hbs  # Health monitoring
```

### Documentation

```
docs/
├── bolt-public-deployment-guide.md      # Bolt usage guide
├── bolt-dual-mode-architecture.md       # Bolt architecture
├── engineering/
│   └── modern-application-exposure.md   # HTTPRoute guide
└── cluster-namespace-structure.md       # Namespace reference
```

---

## Summary

The NGrok implementation provides:

1. **Webhook Ingress** - Secure GitHub webhook endpoint via Gateway API
2. **Application Publishing** - Dynamic tunnel creation for preview/production
3. **Security** - Rate limiting, security headers, TLS termination
4. **DNS Integration** - Automatic Cloudflare DNS management
5. **GitOps** - All configuration managed via ArgoCD

**Key Design Patterns:**
- Gateway API for routing (modern Kubernetes standard)
- Operator pattern for tunnel management
- Dynamic namespace-per-deployment isolation
- Vault-based secrets management
- external-dns for automatic DNS updates

**Migration Priority:**
1. Webhook endpoint (critical for all automation)
2. Application publishing (Bolt preview/production)
3. Traffic policies (security features)
4. DNS management (already on Cloudflare)

