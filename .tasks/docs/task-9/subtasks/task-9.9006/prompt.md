Implement subtask 9006: Apply Kubernetes network policies to restrict inter-service traffic

## Objective
Define and apply NetworkPolicy resources for all namespaces to enforce least-privilege network access between services, allowing only explicitly required communication paths.

## Steps
1. Document the required communication paths: which services need to reach PostgreSQL, Redis, each other, and external endpoints.
2. Create a default-deny ingress NetworkPolicy for each namespace: `spec.podSelector: {}` with `policyTypes: [Ingress]` and no ingress rules.
3. For each service, create allow-ingress NetworkPolicies that permit traffic only from known consumers (using namespace and pod label selectors).
4. Allow ingress from cloudflared pods to Morgan and web frontend services.
5. Allow egress to DNS (kube-dns on port 53) for all pods.
6. Apply all policies and verify they don't break existing connectivity.
7. Store all NetworkPolicy manifests in the infra repo under a `network-policies/` directory.

## Validation
After applying policies, verify all services can still communicate with their required dependencies (health checks pass, API calls succeed). Attempt a connection from a service that should be blocked (e.g., web frontend directly to database) and confirm it is denied. Run `kubectl get networkpolicy -A` and verify policies exist in all namespaces.