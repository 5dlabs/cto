Implement subtask 9005: Set up Cloudflare Tunnel ingress for Morgan and web frontend

## Objective
Configure Cloudflare Tunnel (cloudflared) as the ingress mechanism for Morgan backend and the web frontend, replacing or supplementing any existing ingress controller, with secure routing and DNS configuration.

## Steps
1. Create a Cloudflare Tunnel via the dashboard or `cloudflared tunnel create`.
2. Deploy the `cloudflared` connector as a Kubernetes Deployment (2 replicas for HA) in the infra namespace.
3. Store the tunnel credentials as a Kubernetes Secret.
4. Configure the tunnel's `config.yaml` (via ConfigMap) with ingress rules:
   - `morgan.example.com` → `http://morgan-service.<namespace>.svc.cluster.local:<port>`
   - `app.example.com` → `http://web-frontend-service.<namespace>.svc.cluster.local:<port>`
   - Catch-all rule returning 404.
5. Create CNAME DNS records in Cloudflare pointing to the tunnel ID.
6. Verify connectivity by accessing both URLs externally.
7. Ensure `cloudflared` has appropriate RBAC (ServiceAccount, Role) in the cluster.

## Validation
Access `morgan.example.com` and `app.example.com` from outside the cluster and verify correct responses. Verify tunnel is healthy via `cloudflared tunnel info`. Kill one cloudflared pod and verify the other maintains connectivity. Verify no direct NodePort/LoadBalancer exposure exists for these services.