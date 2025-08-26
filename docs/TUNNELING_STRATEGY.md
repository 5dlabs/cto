# Webhook Tunneling Strategy: NGrok & Cloudflare Integration

## Executive Summary

For the multi-agent platform to work effectively in both development and production environments, we need reliable webhook delivery from GitHub to our Kubernetes clusters. Rather than reinventing the wheel with lesser alternatives, we'll embrace **NGrok** and **Cloudflare Tunnel** as our primary tunneling solutions - both are battle-tested, reliable, and have generous free tiers.

## Why Keep NGrok and Cloudflare?

### NGrok Advantages
- **Industry Standard**: Widely adopted by developers globally
- **Reliability**: 99.95% uptime SLA on paid tiers
- **Developer Experience**: Excellent CLI, debugging tools, and web inspector
- **Free Tier**: Sufficient for individual developers (1 online ngrok process, random URL)
- **Paid Tier Value**: $8/month gets persistent domains, multiple tunnels
- **OAuth Support**: Built-in OAuth providers for additional security

### Cloudflare Tunnel Advantages
- **Zero Trust Security**: Enterprise-grade security built-in
- **Free Tier**: Generous limits for most use cases
- **CDN Integration**: Automatic DDoS protection and global edge network
- **Production Ready**: Handles production workloads seamlessly
- **Argo Smart Routing**: Optimized routing for better performance

## Implementation Architecture

### Dual-Mode Tunneling System

```yaml
# tunnel-config.yaml
tunneling:
  provider: auto  # auto, ngrok, cloudflare, none
  
  ngrok:
    enabled: true
    authtoken: ${NGROK_AUTHTOKEN}  # Optional for free tier
    region: us  # us, eu, ap, au
    
    # For development (free tier)
    development:
      mode: ephemeral
      
    # For teams (paid tier)  
    production:
      mode: persistent
      domain: agent-webhooks.ngrok.io
      
  cloudflare:
    enabled: true
    
    # For development
    development:
      mode: quick-tunnel  # No account needed!
      
    # For production
    production:
      mode: named-tunnel
      tunnel_name: agent-platform
      domain: webhooks.example.com
      credentials: ${CF_TUNNEL_TOKEN}
      
  # Fallback for air-gapped environments
  direct:
    enabled: false
    ingress_type: nodeport  # or loadbalancer
    external_ip: ""  # Required if enabled
```

## Provider Selection Logic

```go
// pkg/tunnel/manager.go
package tunnel

type TunnelManager struct {
    config   TunnelConfig
    provider TunnelProvider
}

func NewTunnelManager(config TunnelConfig) *TunnelManager {
    provider := selectProvider(config)
    return &TunnelManager{
        config:   config,
        provider: provider,
    }
}

func selectProvider(config TunnelConfig) TunnelProvider {
    switch config.Provider {
    case "auto":
        // Intelligent auto-selection
        if config.IsProduction() {
            if config.Cloudflare.Enabled && config.Cloudflare.HasCredentials() {
                return &CloudflareProvider{config: config.Cloudflare}
            }
            if config.NGrok.Enabled && config.NGrok.HasAuthToken() {
                return &NGrokProvider{config: config.NGrok}
            }
        } else {
            // Development mode
            if config.NGrok.Enabled {
                return &NGrokProvider{config: config.NGrok}
            }
            if config.Cloudflare.Enabled {
                // Use Cloudflare quick tunnels (no auth needed!)
                return &CloudflareQuickTunnelProvider{}
            }
        }
    case "ngrok":
        return &NGrokProvider{config: config.NGrok}
    case "cloudflare":
        return &CloudflareProvider{config: config.Cloudflare}
    case "none":
        return &DirectAccessProvider{config: config.Direct}
    }
    
    // Default fallback
    return &NGrokProvider{config: config.NGrok}
}
```

## Installation Wizard Integration

### Interactive Tunnel Setup

```bash
$ agent-platform install

[4/10] Webhook Tunnel Configuration:
  GitHub needs to reach your cluster for webhook delivery.
  
  Select tunneling solution:
  > [1] NGrok (Recommended for development)
    [2] Cloudflare Tunnel (Recommended for production)  
    [3] Auto-select based on environment
    [4] I have a public endpoint (no tunnel needed)
    
  Selection: 1
  
  NGrok Configuration:
  > [1] Use free tier (random URL each time)
    [2] I have an NGrok account (persistent URLs)
    
  Selection: 2
  
  Enter your NGrok auth token (from dashboard.ngrok.com):
  > ************************************
  
  ✓ NGrok configured successfully
  ✓ Testing tunnel connectivity... Success!
  
  Webhook URL will be: https://agent-platform-abc123.ngrok.io/webhooks
```

## NGrok Integration

### Development Mode (Free Tier)

```go
// pkg/tunnel/ngrok_dev.go
package tunnel

import (
    "golang.ngrok.com/ngrok"
    "golang.ngrok.com/ngrok/config"
)

type NGrokDevProvider struct {
    tunnel ngrok.Tunnel
}

func (p *NGrokDevProvider) Start(port int) (string, error) {
    // Start ephemeral tunnel (no auth token needed for basic use)
    tun, err := ngrok.Listen(context.Background(),
        config.HTTPEndpoint(),
        ngrok.WithAuthtokenFromEnv(), // Optional
    )
    
    if err != nil {
        return "", err
    }
    
    p.tunnel = tun
    
    // URL will be something like: https://abc123.ngrok.io
    return tun.URL(), nil
}

func (p *NGrokDevProvider) Stop() error {
    if p.tunnel != nil {
        return p.tunnel.Close()
    }
    return nil
}
```

### Production Mode (Paid Tier)

```go
// pkg/tunnel/ngrok_prod.go
package tunnel

type NGrokProdProvider struct {
    config NGrokConfig
    tunnel ngrok.Tunnel
}

func (p *NGrokProdProvider) Start(port int) (string, error) {
    // Use persistent domain from paid account
    opts := []ngrok.ConnectOption{
        ngrok.WithAuthtoken(p.config.AuthToken),
    }
    
    if p.config.Domain != "" {
        // Paid feature: custom domain
        tun, err := ngrok.Listen(context.Background(),
            config.HTTPEndpoint(
                config.WithDomain(p.config.Domain),
            ),
            opts...,
        )
        p.tunnel = tun
        return fmt.Sprintf("https://%s", p.config.Domain), nil
    }
    
    // Fallback to generated URL
    return p.startEphemeral(port)
}
```

### NGrok Operator Integration

```yaml
# ngrok-operator-config.yaml
apiVersion: v1
kind: Secret
metadata:
  name: ngrok-auth
  namespace: agent-platform
stringData:
  authtoken: ${NGROK_AUTHTOKEN}
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: webhook-ingress
  annotations:
    k8s.ngrok.com/ingress-class: ngrok
spec:
  ingressClassName: ngrok
  rules:
  - host: agent-webhooks.ngrok.io  # Your persistent domain
    http:
      paths:
      - path: /webhooks
        pathType: Prefix
        backend:
          service:
            name: webhook-processor
            port:
              number: 8080
```

## Cloudflare Tunnel Integration

### Development Mode (Quick Tunnels)

```go
// pkg/tunnel/cloudflare_dev.go
package tunnel

type CloudflareQuickTunnelProvider struct {
    process *exec.Cmd
    url     string
}

func (p *CloudflareQuickTunnelProvider) Start(port int) (string, error) {
    // Quick tunnel - no auth required!
    cmd := exec.Command("cloudflared", "tunnel", "--url", 
        fmt.Sprintf("http://localhost:%d", port))
    
    // Capture output to get tunnel URL
    output, err := cmd.StdoutPipe()
    if err != nil {
        return "", err
    }
    
    if err := cmd.Start(); err != nil {
        return "", err
    }
    
    p.process = cmd
    
    // Parse output for tunnel URL
    scanner := bufio.NewScanner(output)
    for scanner.Scan() {
        line := scanner.Text()
        if strings.Contains(line, "https://") && strings.Contains(line, ".trycloudflare.com") {
            p.url = extractURL(line)
            return p.url, nil
        }
    }
    
    return "", fmt.Errorf("failed to get tunnel URL")
}
```

### Production Mode (Named Tunnels)

```yaml
# cloudflare-tunnel-production.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
  namespace: agent-platform
spec:
  replicas: 2  # HA setup
  template:
    spec:
      containers:
      - name: cloudflared
        image: cloudflare/cloudflared:latest
        args:
        - tunnel
        - --no-autoupdate
        - run
        - --token
        - $(TUNNEL_TOKEN)
        env:
        - name: TUNNEL_TOKEN
          valueFrom:
            secretKeyRef:
              name: cloudflare-tunnel
              key: token
---
apiVersion: v1
kind: Service
metadata:
  name: webhook-processor
spec:
  ports:
  - port: 8080
    targetPort: 8080
  selector:
    app: webhook-processor
```

### Cloudflare Zero Trust Configuration

```go
// pkg/tunnel/cloudflare_prod.go
package tunnel

type CloudflareProdProvider struct {
    config CloudflareConfig
}

func (p *CloudflareProdProvider) Configure() error {
    // Set up Zero Trust access policies
    config := CloudflareConfig{
        TunnelName: p.config.TunnelName,
        Routes: []Route{
            {
                Hostname: "webhooks.example.com",
                Service:  "http://webhook-processor:8080",
                Access: AccessPolicy{
                    // Only allow GitHub webhook IPs
                    AllowedIPs: GitHubWebhookIPs,
                    RequireSignedRequests: true,
                },
            },
        },
    }
    
    return p.applyConfig(config)
}
```

## Configuration Profiles

### Solo Developer Profile
```yaml
# Best for individual developers
tunneling:
  provider: ngrok
  ngrok:
    enabled: true
    # No auth token needed for basic use
    development:
      mode: ephemeral
```

### Small Team Profile
```yaml
# Best for small teams with NGrok subscription
tunneling:
  provider: ngrok
  ngrok:
    enabled: true
    authtoken: ${NGROK_AUTHTOKEN}
    production:
      mode: persistent
      domain: team-webhooks.ngrok.io
```

### Enterprise Profile
```yaml
# Best for production deployments
tunneling:
  provider: cloudflare
  cloudflare:
    enabled: true
    production:
      mode: named-tunnel
      tunnel_name: agent-platform-prod
      domain: webhooks.company.com
      credentials: ${CF_TUNNEL_TOKEN}
      zero_trust:
        enabled: true
        allowed_ips: ${GITHUB_WEBHOOK_IPS}
```

### Hybrid Profile
```yaml
# Auto-select based on environment
tunneling:
  provider: auto
  
  # Use NGrok for development
  ngrok:
    enabled: true
    authtoken: ${NGROK_AUTHTOKEN}
    
  # Use Cloudflare for production
  cloudflare:
    enabled: true
    production:
      credentials: ${CF_TUNNEL_TOKEN}
```

## Cost Analysis

### NGrok Pricing
- **Free**: 1 online ngrok process, random URL
- **Personal ($8/month)**: Custom subdomain, 1 reserved domain
- **Pro ($20/month)**: Multiple agents, IP policies
- **Business ($65/month)**: SSO, advanced features

### Cloudflare Pricing
- **Free**: Cloudflare Tunnel included, quick tunnels
- **Pro ($20/month)**: Advanced security features
- **Business ($200/month)**: Enhanced performance
- **Enterprise**: Custom pricing

### Recommended Approach
1. **Development**: NGrok free tier or Cloudflare quick tunnels ($0)
2. **Small Teams**: NGrok Personal ($8/month)
3. **Production**: Cloudflare free tier with named tunnels ($0)
4. **Enterprise**: Cloudflare Business or Enterprise

## Security Considerations

### Webhook Signature Validation
```go
// Always validate GitHub webhook signatures
func validateWebhook(req *http.Request) error {
    signature := req.Header.Get("X-Hub-Signature-256")
    if signature == "" {
        return errors.New("missing signature")
    }
    
    body, _ := ioutil.ReadAll(req.Body)
    expectedSig := computeSignature(body, webhookSecret)
    
    if !hmac.Equal([]byte(signature), []byte(expectedSig)) {
        return errors.New("invalid signature")
    }
    
    return nil
}
```

### IP Allowlisting (Optional)
```yaml
# For additional security in production
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: webhook-allow-github
spec:
  podSelector:
    matchLabels:
      app: webhook-processor
  ingress:
  - from:
    - ipBlock:
        cidr: 140.82.112.0/20  # GitHub webhook IPs
    - ipBlock:
        cidr: 143.55.64.0/20
    - ipBlock:
        cidr: 192.30.252.0/22
```

## Troubleshooting Guide

### Common Issues and Solutions

#### NGrok: "Tunnel not found"
```bash
# Check if ngrok is running
ngrok status

# Restart tunnel
agent-platform tunnel restart

# Verify auth token
ngrok authtoken ${YOUR_TOKEN}
```

#### Cloudflare: "Tunnel connection failed"
```bash
# Check tunnel status
cloudflared tunnel list

# View tunnel logs
kubectl logs -n agent-platform deployment/cloudflared

# Recreate tunnel
cloudflared tunnel delete agent-platform
cloudflared tunnel create agent-platform
```

#### Webhook delivery failures
```bash
# Test webhook connectivity
agent-platform test webhook

# Check GitHub webhook recent deliveries
agent-platform github webhook-status

# Manually trigger test webhook
curl -X POST https://your-tunnel-url/webhooks/test \
  -H "Content-Type: application/json" \
  -d '{"test": true}'
```

## Migration Path

### From Self-Hosted to NGrok/Cloudflare
```bash
#!/bin/bash
# migrate-to-managed-tunnels.sh

# Remove old tunnel solutions
kubectl delete deployment localtunnel 2>/dev/null
kubectl delete deployment frp 2>/dev/null

# Install based on selection
if [ "$TUNNEL_PROVIDER" = "ngrok" ]; then
    echo "Setting up NGrok..."
    kubectl create secret generic ngrok-auth \
        --from-literal=authtoken=${NGROK_AUTHTOKEN}
    kubectl apply -f tunnels/ngrok/
elif [ "$TUNNEL_PROVIDER" = "cloudflare" ]; then
    echo "Setting up Cloudflare Tunnel..."
    cloudflared tunnel create agent-platform
    kubectl create secret generic cloudflare-tunnel \
        --from-literal=token=${CF_TUNNEL_TOKEN}
    kubectl apply -f tunnels/cloudflare/
fi

echo "Tunnel migration complete"
```

## Platform Integration

### Automatic Webhook URL Updates
```go
// pkg/platform/webhook_manager.go
func (m *WebhookManager) UpdateGitHubWebhooks(tunnelURL string) error {
    for _, agent := range m.agents {
        webhookURL := fmt.Sprintf("%s/webhooks/%s", tunnelURL, agent.Name)
        
        // Update GitHub App webhook URL
        if err := agent.GitHubApp.UpdateWebhookURL(webhookURL); err != nil {
            return fmt.Errorf("failed to update %s webhook: %w", agent.Name, err)
        }
        
        log.Printf("Updated %s webhook URL to: %s", agent.Name, webhookURL)
    }
    
    return nil
}
```

### Health Monitoring
```go
// pkg/monitoring/tunnel_health.go
func (m *TunnelMonitor) CheckHealth() error {
    // Verify tunnel is up
    resp, err := http.Get(m.tunnelURL + "/health")
    if err != nil {
        return fmt.Errorf("tunnel unreachable: %w", err)
    }
    
    if resp.StatusCode != 200 {
        return fmt.Errorf("tunnel unhealthy: status %d", resp.StatusCode)
    }
    
    // Test webhook delivery
    return m.testWebhookDelivery()
}
```

## Conclusion

By embracing NGrok and Cloudflare Tunnel as first-class citizens in our platform:

1. **Reliability**: We leverage battle-tested infrastructure used by thousands of companies
2. **Cost-Effective**: Both offer generous free tiers perfect for development
3. **Developer Experience**: Well-documented, great CLIs, extensive tooling
4. **Production Ready**: Easy path from development to production
5. **Security**: Enterprise-grade security features available when needed

The dual-provider approach gives users flexibility while maintaining simplicity. Developers can start with free tiers and seamlessly upgrade as their needs grow.