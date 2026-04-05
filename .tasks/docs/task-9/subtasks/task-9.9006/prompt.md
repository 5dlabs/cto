Implement subtask 9006: Apply Kubernetes NetworkPolicies for inter-service access restriction

## Objective
Define and apply NetworkPolicy resources to enforce least-privilege network access between all services, databases, and external egress.

## Steps
1. Create a default-deny NetworkPolicy for all pods in the application namespace: deny all ingress and egress by default. 2. Create ingress NetworkPolicies for PostgreSQL allowing only backend service pods (label-selected) on port 5432. 3. Create ingress NetworkPolicies for Redis allowing only backend service pods on port 6379 and sentinel port 26379. 4. Create ingress NetworkPolicies for each backend service allowing traffic from cloudflared pods and other authorized services. 5. Create egress NetworkPolicies allowing backend services to reach databases, external APIs (Stripe, LinkedIn, etc.), and DNS (port 53). 6. Create egress NetworkPolicies for cloudflared pods to reach backend services. 7. Apply all policies and verify with `kubectl describe networkpolicy`. 8. Test that unauthorized cross-service connections are blocked.

## Validation
Deploy a test pod without authorized labels and attempt to connect to PostgreSQL on port 5432 — verify the connection is refused/timed out. Verify authorized backend pods can still connect to all required services. Run `kubectl exec` from a backend pod to confirm egress to external APIs succeeds while egress to unauthorized internal services is blocked.