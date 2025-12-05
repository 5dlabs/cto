# Modern Application Exposure Guide

This document explains how to expose applications to the public internet using our modern infrastructure stack with Gateway API, external-dns, and NGrok.

## 🏗️ Infrastructure Overview

Our modern stack provides automatic:
- **DNS Management** via external-dns + Cloudflare
- **SSL/TLS Certificates** via NGrok + Let's Encrypt  
- **Security Headers & Rate Limiting** via NGrok TrafficPolicy
- **Traffic Routing** via Gateway API (replaces Ingress)

## 🚀 Quick Start

### 1. Create Your Application Resources

Standard Kubernetes Deployment and Service:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  namespace: my-app-namespace
spec:
  replicas: 2
  selector:
    matchLabels:
      app: my-app
  template:
    metadata:
      labels:
        app: my-app
    spec:
      containers:
      - name: my-app
        image: my-app:latest
        ports:
        - containerPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: my-app-service
  namespace: my-app-namespace
spec:
  selector:
    app: my-app
  ports:
    - port: 80
      targetPort: 8080
```

### 2. Create HTTPRoute for Public Exposure

```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: my-app-public
  namespace: my-app-namespace
  annotations:
    # Ignore ArgoCD sync differences for status fields
    argocd.argoproj.io/ignore-differences: |
      jsonPointers:
      - /status
      - /metadata/annotations
      - /metadata/managedFields
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: default
  hostnames:
    - "myapp.public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: "/"
      backendRefs:
        - group: ""
          kind: Service
          name: my-app-service
          port: 80
          weight: 1
```

### 3. Deploy and Access

1. **Deploy** your resources via ArgoCD or kubectl
2. **Wait** ~2-3 minutes for DNS propagation
3. **Access** your app at `https://myapp.public.5dlabs.ai`

That's it! 🎉

## 🔧 Advanced Configuration

### Path-Based Routing

Route different paths to different services:

```yaml
spec:
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: "/api"
      backendRefs:
        - name: api-service
          port: 8080
    - matches:
        - path:
            type: PathPrefix
            value: "/web"
      backendRefs:
        - name: web-service
          port: 80
    - matches:
        - path:
            type: Exact
            value: "/health"
      backendRefs:
        - name: health-service
          port: 9090
```

### Header-Based Routing

Route based on HTTP headers:

```yaml
spec:
  rules:
    - matches:
        - headers:
            - name: "x-api-version"
              value: "v2"
      backendRefs:
        - name: api-v2-service
          port: 8080
    - matches:
        - headers:
            - name: "x-client-type"
              value: "mobile"
      backendRefs:
        - name: mobile-optimized-service
          port: 8080
```

### Multiple Hostnames

Expose the same service on multiple domains:

```yaml
spec:
  hostnames:
    - "myapp.public.5dlabs.ai"
    - "app.public.5dlabs.ai"
    - "service.public.5dlabs.ai"
```

### HTTP Method Routing

Route based on HTTP methods:

```yaml
spec:
  rules:
    - matches:
        - method: GET
          path:
            type: PathPrefix
            value: "/api"
      backendRefs:
        - name: read-api-service
          port: 8080
    - matches:
        - method: POST
          path:
            type: PathPrefix
            value: "/api"
      backendRefs:
        - name: write-api-service
          port: 8080
```

## 🛡️ Built-in Security Features

All exposed applications automatically get:

### Security Headers
- `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`

### Rate Limiting
- **10 requests/second** per client IP
- **20 request burst** capacity
- **Sliding window** strategy

### SSL/TLS
- **Automatic Let's Encrypt certificates**
- **TLS 1.3** with modern cipher suites
- **HTTP → HTTPS redirect**

## 🌐 DNS Management

### Automatic DNS Records

When you create an HTTPRoute, external-dns automatically creates:

```
myapp.public.5dlabs.ai.     300  IN  CNAME  <ngrok-endpoint>.ngrok-cname.com
external-dns-myapp.public.5dlabs.ai.  300  IN  TXT  "heritage=external-dns,external-dns/owner=external-dns-5dlabs"
```

### Custom Subdomains

Use any subdomain under `public.5dlabs.ai`:

- `api.public.5dlabs.ai`
- `dashboard.public.5dlabs.ai`
- `webhooks.public.5dlabs.ai`
- `docs.public.5dlabs.ai`

## 📊 Monitoring & Observability

### NGrok Dashboard
- Access NGrok dashboard for traffic analytics
- View request/response metrics
- Monitor SSL certificate status

### ArgoCD Integration
- HTTPRoutes are managed via GitOps
- Automatic sync and health monitoring
- Rollback capabilities

## 🔍 Troubleshooting

### Common Issues

#### HTTPRoute Not Working
```bash
# Check HTTPRoute status
kubectl get httproute my-app-public -n my-app-namespace -o yaml

# Check Gateway status
kubectl get gateway public-gateway -n default -o yaml

# Check external-dns logs
kubectl logs -n external-dns deployment/external-dns
```

#### DNS Not Resolving
```bash
# Check DNS records
dig myapp.public.5dlabs.ai

# Check external-dns events
kubectl get events -n external-dns
```

#### SSL Certificate Issues
```bash
# Test SSL
curl -v https://myapp.public.5dlabs.ai

# Check NGrok status
kubectl get agentendpoint -A
```

### Debug Commands

```bash
# List all HTTPRoutes
kubectl get httproute -A

# Check Gateway listeners
kubectl get gateway public-gateway -n default -o jsonpath='{.status.listeners}'

# View external-dns managed records
kubectl get dnsendpoint -A
```

## 📋 Migration from Ingress

### Old Pattern (Deprecated)
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
    - hosts: [myapp.example.com]
      secretName: myapp-tls
  rules:
    - host: myapp.example.com
      http:
        paths:
        - path: /
          backend:
            service:
              name: my-app-service
              port:
                number: 80
```

### New Pattern (Modern)
```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: my-app-public
spec:
  parentRefs:
    - name: public-gateway
      namespace: default
  hostnames:
    - "myapp.public.5dlabs.ai"
  rules:
    - backendRefs:
        - name: my-app-service
          port: 80
```

## 🎯 Best Practices

### Naming Conventions
- **HTTPRoute names**: `{app-name}-public`, `{app-name}-api`
- **Hostnames**: `{service}.public.5dlabs.ai`
- **Services**: `{app-name}-service`

### Resource Organization
- Keep HTTPRoute in the same namespace as your service
- Use descriptive names and labels
- Add ArgoCD ignore annotations for status fields

### Security Considerations
- All traffic is automatically rate-limited
- Security headers are applied by default
- Use path-based routing to limit exposed endpoints
- Consider additional authentication for sensitive services

## 🔗 Related Documentation

- [Gateway API Specification](https://gateway-api.sigs.k8s.io/)
- [NGrok Kubernetes Operator](https://github.com/ngrok/ngrok-operator)
- [External-DNS Documentation](https://github.com/kubernetes-sigs/external-dns)

## 💡 Examples Repository

For complete working examples, see the charts repository:
- Basic web application exposure
- API service with path routing
- Multi-service applications
- Webhook endpoints

---

**Questions?** Reach out to the platform team or check existing HTTPRoute examples in the `infra/gitops/resources/` directory.
