Implement subtask 9005: Set up Cloudflare Tunnel ingress for Morgan and website

## Objective
Deploy Cloudflare Tunnel (cloudflared) as a Kubernetes Deployment to expose Morgan (Discord bot dashboard) and the public website without opening inbound ports.

## Steps
1. Create a Cloudflare Tunnel in the Cloudflare Zero Trust dashboard and obtain the tunnel token/credentials. 2. Store the tunnel credentials as a Kubernetes Secret. 3. Create a `cloudflared` Deployment with 2 replicas running the tunnel connector, mounting the credentials secret. 4. Create a ConfigMap with the `cloudflared` config YAML mapping public hostnames to internal Kubernetes service endpoints (e.g., `morgan.example.com` → `http://morgan-service.default.svc.cluster.local:3000`, `www.example.com` → `http://website-service.default.svc.cluster.local:3000`). 5. Configure DNS CNAME records in Cloudflare pointing to the tunnel UUID. 6. Verify both Morgan and website are reachable via their public URLs through the tunnel.

## Validation
Verify `cloudflared` pods are running with 2 ready replicas. Access `https://morgan.example.com` and `https://www.example.com` from an external browser and confirm correct content loads. Kill one `cloudflared` pod and verify access remains uninterrupted through the surviving replica.