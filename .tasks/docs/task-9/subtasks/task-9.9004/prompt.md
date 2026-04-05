Implement subtask 9004: Set up Cloudflare Tunnel ingress for Morgan and web endpoints

## Objective
Deploy and configure Cloudflare Tunnel (cloudflared) as the ingress mechanism for Morgan assistant and web application endpoints, replacing or augmenting any existing in-cluster ingress.

## Steps
1. Create a Cloudflare Tunnel via the dashboard or `cloudflared` CLI and obtain the tunnel token/credentials.
2. Store the tunnel credentials as a Kubernetes Secret.
3. Deploy `cloudflared` as a Deployment in the cluster with the tunnel credentials mounted.
4. Configure the tunnel's `config.yaml` (or via Cloudflare dashboard) with ingress rules mapping public hostnames to internal Kubernetes services (e.g., `morgan.example.com` → `http://morgan-service.namespace.svc:port`, `app.example.com` → `http://web-service.namespace.svc:port`).
5. Add a catch-all rule returning 404.
6. Verify DNS CNAME records point to the tunnel's `.cfargotunnel.com` address.
7. Apply the cloudflared Deployment manifest and verify the tunnel is connected and healthy.

## Validation
Verify cloudflared pod is running and the tunnel shows as 'Healthy' in Cloudflare dashboard. Curl each public hostname and confirm responses are served from the correct backend services. Verify no direct cluster IP/port is exposed to the internet. Test that catch-all returns 404 for undefined routes.