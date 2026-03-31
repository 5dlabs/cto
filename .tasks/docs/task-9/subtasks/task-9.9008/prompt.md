Implement subtask 9008: Apply Kubernetes NetworkPolicy resources for pod-to-pod isolation

## Objective
Create NetworkPolicy resources restricting pod-to-pod communication: frontend can only reach backend, backend can only reach data services and Loki, with a special egress exception for headless browser screenshot capture.

## Steps
1. Create a default-deny NetworkPolicy for the `hermes-production` namespace: deny all ingress and egress by default.
2. Create NetworkPolicy for frontend pods: allow ingress from the ingress controller, allow egress only to backend service on port 3001.
3. Create NetworkPolicy for backend pods:
   - Allow ingress from the ingress controller (for API routes) and from frontend pods.
   - Allow egress to PostgreSQL (5432), Redis (6379), NATS (4222), MinIO (9000), Loki (3100).
   - Allow egress to external URLs for headless browser screenshot capture (port 443 to 0.0.0.0/0, or a restricted CIDR if possible).
   - Allow DNS egress (port 53 to kube-dns).
4. Create NetworkPolicy for data service pods (if not already handled by operators): allow ingress only from backend pods.
5. Deny all inter-namespace traffic: no pods in `hermes-production` can communicate with pods in other namespaces except kube-system (DNS).
6. Label all pods appropriately for NetworkPolicy selectors: `app.kubernetes.io/component: frontend`, `app.kubernetes.io/component: backend`, etc.

## Validation
From a test pod in `hermes-production`, verify: (1) Can reach PostgreSQL on port 5432 via `nc -zv`. (2) Cannot reach the Kubernetes API server on port 6443. (3) Cannot reach pods in other namespaces. From a frontend pod, verify it can reach backend on 3001 but cannot directly reach PostgreSQL on 5432. From backend pod, verify external HTTPS egress works (for screenshot capture).