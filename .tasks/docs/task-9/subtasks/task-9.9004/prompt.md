Implement subtask 9004: Configure ingress rules and Cloudflare Tunnel for Morgan and web frontend routing

## Objective
Set up Kubernetes ingress resources and Cloudflare Tunnel to route external traffic to the Morgan chatbot service and web frontend, with proper path-based and host-based routing.

## Steps
1. Deploy `cloudflared` as a Deployment in the cluster with a Tunnel token secret.
2. Configure the Cloudflare Tunnel with ingress rules mapping:
   - `app.domain.com` → web frontend service (Next.js)
   - `api.domain.com` → API gateway service
   - `morgan.domain.com` or webhook path → Morgan chatbot service
3. Create corresponding Kubernetes Service resources if not already present.
4. Configure Kubernetes Ingress resources (or use cloudflared's built-in ingress) with proper path routing.
5. Set appropriate timeouts for long-lived connections (e.g., WebSocket for Morgan).
6. Configure health check endpoints for the tunnel.
7. Test all routes resolve correctly from external clients.

## Validation
Verify external HTTP requests to each hostname route to the correct backend service; confirm WebSocket connections to Morgan work through the tunnel; verify health check endpoint returns 200; confirm no direct NodePort/LoadBalancer exposure exists.