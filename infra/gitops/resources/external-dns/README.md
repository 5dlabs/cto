# External DNS with Cloudflare Integration

This configuration sets up external-dns to automatically manage DNS records in Cloudflare for your Kubernetes resources, specifically integrated with NGrok Gateway.

## Overview

External DNS automatically creates and manages DNS records in Cloudflare based on Kubernetes resources like:
- Gateway API resources (HTTPRoute, etc.)
- Services with LoadBalancer type
- Ingress resources

## Configuration

### Cloudflare API Token Setup

1. **Create a Cloudflare API Token**:
   - Go to Cloudflare Dashboard → My Profile → API Tokens
   - Click "Create Token"
   - Use the "Custom token" template
   - Set permissions:
     - Zone: Zone Settings:Read
     - Zone: Zone:Read  
     - Zone: DNS:Edit
   - Set zone resources to include `5dlabs.ai`
   - Copy the generated token

2. **Add the token to your secret management**:
   ```bash
   # Add to Vault at secret/cloudflare
   # Key: cloudflare
   # Property: api_token
   # Value: your_cloudflare_api_token
   ```

### How It Works

1. **Gateway Integration**: The NGrok Gateway has been annotated with:
   ```yaml
   annotations:
     external-dns.alpha.kubernetes.io/hostname: "public.5dlabs.ai,github.public.5dlabs.ai"
   ```

2. **Automatic DNS Management**: External DNS will:
   - Watch for Gateway resources with the annotation
   - Create DNS records in Cloudflare pointing to NGrok's endpoints
   - Enable Cloudflare proxy (orange cloud) for DDoS protection and CDN

3. **Record Ownership**: External DNS uses TXT records to track ownership:
   - Prefix: `external-dns-`
   - Owner ID: `external-dns-5dlabs`

## Usage

### For Gateway API Resources

Add the annotation to any Gateway or HTTPRoute:
```yaml
metadata:
  annotations:
    external-dns.alpha.kubernetes.io/hostname: "api.5dlabs.ai"
```

### For Services

Create a LoadBalancer service with hostname annotation:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-service
  annotations:
    external-dns.alpha.kubernetes.io/hostname: "service.5dlabs.ai"
spec:
  type: LoadBalancer
  # ... rest of service spec
```

### For Ingress Resources

Standard ingress with hostname in spec:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-ingress
spec:
  rules:
  - host: app.5dlabs.ai
    # ... rest of ingress spec
```

## Monitoring

Check external-dns logs:
```bash
kubectl logs -n external-dns deployment/external-dns
```

View managed DNS records:
```bash
kubectl get txt -A | grep external-dns
```

## Security Features

- Uses API tokens (more secure than global API keys)
- Runs as non-root user
- Minimal RBAC permissions
- Only manages records for `5dlabs.ai` domain
- TXT record ownership tracking prevents conflicts

## Integration with NGrok

The setup automatically integrates with your existing NGrok Gateway:
- DNS records point to NGrok's edge endpoints
- Cloudflare proxy provides additional DDoS protection
- SSL/TLS termination happens at Cloudflare edge
- Traffic flows: Internet → Cloudflare → NGrok → Your services
