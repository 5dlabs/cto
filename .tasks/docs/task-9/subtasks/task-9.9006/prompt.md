Implement subtask 9006: Define and apply Kubernetes NetworkPolicies for inter-service traffic restriction

## Objective
Map all legitimate inter-service communication flows and create NetworkPolicy resources to enforce least-privilege network access between pods.

## Steps
1. Document all required service-to-service communication flows (e.g., backend → PostgreSQL:5432, backend → Redis:6379, Morgan → backend API, cloudflared → web-frontend, cloudflared → Morgan). 2. Create a default-deny ingress NetworkPolicy for the production namespace: deny all ingress unless explicitly allowed. 3. Create per-service NetworkPolicy resources allowing only the documented flows: a) PostgreSQL: allow ingress from backend pods on port 5432. b) Redis: allow ingress from backend pods on port 6379. c) Backend API: allow ingress from cloudflared and Morgan on the API port. d) Web frontend: allow ingress from cloudflared on the HTTP port. e) Morgan: allow ingress from cloudflared on its port, allow egress to backend API. 4. Ensure DNS (kube-dns) egress is allowed for all pods. 5. Apply all NetworkPolicies and test each allowed flow.

## Validation
From a test pod in the namespace, attempt to connect to PostgreSQL — should be blocked. From a backend pod, connect to PostgreSQL — should succeed. Repeat for each service pair: verify authorized connections succeed and unauthorized connections are rejected (timeout/connection refused). Use `kubectl exec` with `nc` or `curl` to test connectivity.