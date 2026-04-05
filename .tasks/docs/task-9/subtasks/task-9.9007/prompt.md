Implement subtask 9007: Implement Cilium default-deny and ingress network policies

## Objective
Create CiliumNetworkPolicy resources to enforce default-deny ingress in the sigma1 namespace, then add allow rules for Cloudflare Tunnel to frontend and backend services, backend to database/cache, Morgan to all backends, and frontend SSR to backend APIs.

## Steps
1. Create default-deny ingress CiliumNetworkPolicy:
   ```yaml
   apiVersion: cilium.io/v2
   kind: CiliumNetworkPolicy
   metadata:
     name: default-deny-ingress
     namespace: sigma1
   spec:
     endpointSelector: {}
     ingress: []
   ```
2. Allow Cloudflare Tunnel to frontend:
   - Create policy allowing ingress from cloudflare-tunnel pod labels to `sigma1-website` on port 3000
3. Allow Cloudflare Tunnel to backend services:
   - Create policy allowing ingress from cloudflare-tunnel pod labels to each backend service on port 8080
   - One policy per service or a combined policy with endpointSelector matching all backend labels
4. Allow backend services to PostgreSQL:
   - Create policy on `sigma1-postgres` allowing ingress from pods with backend service labels on port 5432
5. Allow backend services to Valkey:
   - Create policy on `sigma1-valkey` allowing ingress from backend service labels on port 6379
6. Allow Morgan to all backend services:
   - Create policy allowing ingress from Morgan pod labels to all backend service pods on port 8080
7. Allow frontend SSR to backend APIs:
   - Create policy allowing ingress from `sigma1-website` pod labels to backend services on port 8080
8. Deny backend-to-backend (already covered by default-deny, just verify no extra allow rules break this).

## Validation
After applying all policies: (1) exec into finance pod, `curl equipment-catalog:8080/health` — verify timeout/connection refused (backend-to-backend denied). (2) exec into Morgan pod, `curl finance:8080/health` — verify 200 (Morgan allowed). (3) exec into a backend pod, `curl sigma1-postgres:5432` — verify connection succeeds (backend to DB allowed). (4) Verify frontend still accessible via tunnel.