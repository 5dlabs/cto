Implement subtask 9005: Apply Kubernetes network policies to restrict inter-service traffic

## Objective
Define and apply NetworkPolicy resources to enforce least-privilege network access between pods, namespaces, and external endpoints. Only allow traffic paths that are required by the application architecture.

## Steps
1. Audit the service communication graph: identify which services talk to PostgreSQL, Redis, each other, and external endpoints.
2. Create a default-deny-all ingress NetworkPolicy for each namespace to block all traffic by default.
3. Create granular allow NetworkPolicies for each service:
   - Allow API services to reach PostgreSQL and Redis on their specific ports.
   - Allow the web frontend to reach API services.
   - Allow cloudflared to reach web and Morgan services.
   - Allow DNS egress (port 53) for all pods.
   - Allow egress to external APIs (e.g., OpenAI, Cloudflare) as needed.
4. Label all pods and namespaces consistently for policy selectors.
5. Apply all NetworkPolicy manifests.
6. Document the network policy matrix in the infra repo.

## Validation
After applying policies, verify allowed traffic paths work (e.g., API can connect to PostgreSQL, web can reach API). Verify blocked paths fail (e.g., web cannot directly reach PostgreSQL, random pods cannot reach Redis). Use a test pod to attempt unauthorized connections and confirm they are denied. Run `kubectl get networkpolicy -A` to confirm all policies are applied.