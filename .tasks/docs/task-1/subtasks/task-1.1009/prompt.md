Implement subtask 1009: Deploy Cloudflare Tunnel Deployment and Service

## Objective
Deploy a Cloudflare Tunnel (cloudflared) Deployment and associated Service in the sigma1 namespace for Morgan external access routing.

## Steps
1. Create a Deployment `sigma1-cloudflare-tunnel` in `sigma1` namespace:
   - Image: `cloudflare/cloudflared:latest`
   - Command: `cloudflared tunnel --no-autoupdate run --token=$(TUNNEL_TOKEN)`
   - Mount tunnel token from a Secret (create `sigma1-tunnel-credentials` Secret with TUNNEL_TOKEN).
   - Resource limits: 128Mi memory, 100m CPU.
   - Replicas: 1.
   - Liveness probe: check cloudflared metrics endpoint or process health.
2. Create a Service (ClusterIP) if needed for internal routing, or rely on Cloudflare Tunnel's outbound-only nature.
3. Configure tunnel ingress rules (in Cloudflare dashboard or via config file) to route external domain to internal services.
4. Apply manifests.

## Validation
Cloudflare Tunnel pod is in Running state: `kubectl get pods -n sigma1 -l app=sigma1-cloudflare-tunnel` shows 1/1 Ready. Pod logs show `Connection established` or `Registered tunnel connection`. Cloudflare dashboard shows tunnel as CONNECTED.