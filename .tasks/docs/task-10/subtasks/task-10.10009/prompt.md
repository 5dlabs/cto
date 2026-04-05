Implement subtask 10009: Ingress and TLS: configure Cloudflare Tunnel for all public endpoints

## Objective
Configure a Cloudflare Tunnel (cloudflared) deployment to expose all sigma1 public endpoints with TLS termination at the Cloudflare edge.

## Steps
Step-by-step:
1. Create a `cloudflared` Deployment in the `ingress-system` namespace (or sigma1 if preferred).
2. Create a Kubernetes Secret containing the Cloudflare Tunnel credentials JSON (tunnel ID, account tag, tunnel secret).
3. Create a ConfigMap `cloudflared-config` with the ingress rules mapping hostnames to sigma1 services:
   - `api.sigma1.example.com` → `http://equipment-catalog.sigma1.svc.cluster.local:8080`
   - `rms.sigma1.example.com` → `http://rms.sigma1.svc.cluster.local:8080`
   - Similar entries for finance, customer-vetting, and the Next.js frontend.
   - Catch-all `http_status: 404`
4. TLS is terminated at Cloudflare edge; internal traffic is plain HTTP within the cluster.
5. Set replicas: 2 for cloudflared with pod anti-affinity for HA.

## Validation
Deploy cloudflared and verify pods are running. Check Cloudflare dashboard to confirm tunnel is connected. Curl the public hostname (e.g., `curl https://api.sigma1.example.com/health`) from an external machine and verify a valid TLS-terminated response from the correct backend service.