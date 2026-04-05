Implement subtask 9005: Enforce Kubernetes network policies to restrict inter-service traffic

## Objective
Define and apply Kubernetes NetworkPolicy resources for all namespaces to enforce least-privilege network access between services, blocking unauthorized pod-to-pod communication.

## Steps
1. Create a default-deny ingress NetworkPolicy for the application namespace.
2. Define allow-list NetworkPolicies for each service:
   - Web frontend → API service (specific port)
   - API service → PostgreSQL (port 5432)
   - API service → Redis/Valkey (port 6379)
   - Morgan → API service, Signal-CLI sidecar
   - Worker services → PostgreSQL, Redis, NATS/message queue
3. Allow DNS egress (port 53) for all pods to kube-dns.
4. Allow Cloudflare Tunnel pod to reach backend services.
5. Block all other inter-pod traffic by default.
6. Label all pods consistently to support selector-based policies.
7. Apply policies and verify with `kubectl describe networkpolicy`.

## Validation
From a test pod, confirm connections to unauthorized services are refused (timeout/reset); confirm authorized service-to-service connections succeed; verify DNS resolution works for all pods; run a network policy audit tool (e.g., `kubectl np-viewer`) to confirm coverage.