Implement subtask 10001: Create Cloudflare Tunnel CR with route mapping for PM server and frontend

## Objective
Create an accesstunnel or clustertunnel Custom Resource in the sigma-1-dev namespace to expose services via Cloudflare Tunnel. Configure route mappings for /api/* to the PM server service and / to the frontend service (if in scope per D5). TLS terminates at Cloudflare edge. No NGINX or other ingress controller.

## Steps
1. Create `manifests/production/cloudflare-tunnel.yaml` with an `accesstunnel` or `clustertunnel` CR (use whichever CRD is installed on the cluster — check with `kubectl api-resources | grep tunnel`).
2. Set the CR namespace to `sigma-1-dev`. Configure the tunnel name (e.g., `sigma-1-tunnel`).
3. Add ingress rules in the CR spec:
   - Rule 1: path `/api/*` → service `sigma-1-pm-server` on port 8080 (or whatever the PM server service port is).
   - Rule 2: path `/` → service `sigma-1-frontend` on port 3000 (only if D5 includes Tasks 6-9; otherwise omit).
4. Configure TLS settings: `originRequest.noTLSVerify: true` if services use HTTP internally, since TLS terminates at Cloudflare edge.
5. Apply the manifest: `kubectl apply -f manifests/production/cloudflare-tunnel.yaml`.
6. Verify the tunnel CR reaches 'Active' or 'Ready' status.

## Validation
`kubectl get accesstunnel -n sigma-1-dev` (or `clustertunnel`) returns the CR in 'Active' or 'Ready' state. External HTTPS request to the tunnel URL's `/api/health` endpoint returns HTTP 200 with valid TLS certificate.