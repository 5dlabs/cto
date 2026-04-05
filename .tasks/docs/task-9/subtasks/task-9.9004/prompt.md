Implement subtask 9004: Configure Cloudflare Tunnel ingress with ClusterTunnel CR

## Objective
Create the ClusterTunnel CR to route external traffic from sigma-1.com and api.sigma-1.com to all backend services, including WebSocket support for Morgan.

## Steps
1. Create or update the ClusterTunnel CR for the cloudflare-tunnel-operator:
   - `apiVersion: networking.cfargotunnel.com/v1alpha1`, `kind: ClusterTunnel`
   - Name: `sigma1-tunnel`
   - Configure the Cloudflare API token secret reference
2. Define ingress rules in the ClusterTunnel spec or via separate TunnelBinding CRs:
   - `sigma-1.com` → service `sigma1-website` port 3000 (frontend)
   - `api.sigma-1.com/catalog/*` → service `equipment-catalog` port 8080
   - `api.sigma-1.com/rms/*` → service `rms` port 8080
   - `api.sigma-1.com/finance/*` → service `finance` port 8080
   - `api.sigma-1.com/vetting/*` → service `customer-vetting` port 8080
   - `api.sigma-1.com/social/*` → service `social-engine` port 8080
   - `api.sigma-1.com/ws/*` → service `morgan` port 8080 (ensure WebSocket support is enabled via `originRequest.noTLSVerify: true` and `originRequest.connectTimeout: 30s`)
3. For WebSocket (Morgan), ensure the tunnel configuration includes:
   - `originRequest.httpHostHeader: morgan`
   - Verify WebSocket upgrade headers are passed through
4. Apply the CR and verify the tunnel is created in the Cloudflare dashboard.
5. Verify DNS records are created automatically (CNAME to tunnel UUID).

## Validation
Verify tunnel pod is running: `kubectl get pods -l app=cloudflare-tunnel`. Test each route: `curl -s -o /dev/null -w '%{http_code}' https://sigma-1.com` returns 200; `curl https://api.sigma-1.com/catalog/health/ready` returns 200; repeat for all 6 service paths. Test WebSocket: use wscat to connect to `wss://api.sigma-1.com/ws/` and verify connection is established.