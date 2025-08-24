# NGrok Gateway Configuration

This directory contains the modern NGrok Gateway API configuration for 5dlabs.ai infrastructure.

## Overview

Our NGrok setup provides secure, public access to Kubernetes services through:
- **Custom Domains**: `public.5dlabs.ai` and `github.public.5dlabs.ai`
- **TLS Termination**: Automatic HTTPS with NGrok-managed certificates
- **Traffic Policy**: Security headers, rate limiting, and request/response transformation
- **External DNS**: Automatic DNS record management in Cloudflare

## Components

### 1. Gateway (`gateway.yaml`)
- **Protocol**: HTTPS with TLS termination
- **Hostnames**: `public.5dlabs.ai`, `github.public.5dlabs.ai`
- **Traffic Policy**: Applied via annotation for security and performance
- **External DNS**: Automatic DNS record creation

### 2. Traffic Policy (`traffic-policy.yaml`)
- **Rate Limiting**: 10 requests/second with sliding window algorithm
- **Security Headers**: HSTS, XSS protection, content type options
- **Request Headers**: Adds forwarding and NGrok identification headers
- **Response Headers**: Security-focused response headers

### 3. Domain Resources (`domain.yaml`)
- **Custom Domains**: Registers our domains with NGrok
- **TLS Certificates**: Automatic certificate provisioning and renewal

### 4. Gateway Class (`gatewayclass.yaml`)
- **Controller**: `ngrok.com/gateway-controller`
- **Standard Gateway API**: Compatible with Kubernetes Gateway API v1

## Features

### Modern Security
- **HTTPS Everywhere**: All traffic encrypted with TLS 1.2+
- **Security Headers**: HSTS, XSS protection, frame options
- **Rate Limiting**: Protection against abuse and DDoS
- **Content Security**: Prevents MIME type sniffing

### High Availability
- **NGrok Global Network**: Distributed edge locations worldwide
- **Automatic Failover**: Built-in redundancy and health checks
- **Load Balancing**: Traffic distributed across healthy backends

### Developer Experience
- **Gateway API**: Modern, standardized Kubernetes networking
- **Traffic Policy**: Declarative traffic management
- **External DNS**: Automatic DNS record management
- **Observability**: Built-in metrics and logging

## Usage

### HTTPRoute Example
```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: my-service-route
  namespace: default
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: default
  hostnames:
    - "api.public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: /api/v1
      backendRefs:
        - name: my-service
          port: 80
```

### Service Requirements
Services must be accessible within the cluster:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-service
  namespace: default
spec:
  selector:
    app: my-app
  ports:
    - port: 80
      targetPort: 8080
```

## Monitoring

### Gateway Status
```bash
kubectl get gateway public-gateway -n default -o yaml
```

### Traffic Policy Status
```bash
kubectl get ngroktrafficpolicy security-policy -n default -o yaml
```

### Domain Status
```bash
kubectl get domain -n default
```

### NGrok Operator Logs
```bash
kubectl logs -n ngrok-operator deployment/ngrok-operator
```

## Troubleshooting

### Common Issues

1. **SSL Certificate Errors**
   - NGrok handles TLS termination automatically
   - Check domain registration in NGrok dashboard
   - Verify DNS records point to NGrok endpoints

2. **404 Not Found**
   - Verify HTTPRoute configuration
   - Check backend service is running and accessible
   - Confirm path matching rules

3. **Rate Limiting**
   - Check Traffic Policy configuration
   - Monitor request patterns
   - Adjust rate limits if needed

4. **DNS Resolution**
   - Verify external-dns is running
   - Check Cloudflare DNS records
   - Test with different DNS resolvers

### Debugging Commands
```bash
# Check Gateway status
kubectl describe gateway public-gateway -n default

# Check HTTPRoute status
kubectl get httproute -A

# Check NGrok operator status
kubectl get pods -n ngrok-operator

# Check external-dns logs
kubectl logs -n external-dns deployment/external-dns
```

## Security Considerations

- **Custom Domains**: Use your own domains for branding and control
- **Traffic Policy**: Implement security headers and rate limiting
- **Network Policies**: Consider Kubernetes network policies for additional security
- **RBAC**: Limit access to NGrok resources with proper RBAC

## Performance Optimization

- **Regional Selection**: NGrok automatically selects optimal regions
- **Connection Pooling**: Reuse connections for better performance
- **Compression**: Enable gzip compression in Traffic Policy if needed
- **Caching**: Implement caching strategies in your applications

## Migration Notes

This configuration represents a modern, production-ready NGrok setup using:
- Gateway API v1 (stable)
- NGrok Operator v0.19.0
- Traffic Policy for advanced features
- External DNS integration
- Custom domain support

Previous configurations using Ingress resources can be migrated to Gateway API for better feature support and future compatibility.
