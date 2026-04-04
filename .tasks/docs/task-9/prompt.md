Implement task 9: Production Hardening: HA Scaling, CDN, TLS, Ingress, and Network Policies (Bolt - Kubernetes/Helm)

## Goal
Stretch/deferred task: Apply production hardening to the sigma-1 infrastructure including high-availability replica scaling for cto-pm, TLS termination on ingress, network policies restricting inter-namespace traffic, and resource limits/requests on all pods. This task is beyond the PRD's stated acceptance criteria and should only be started after Tasks 1–6 and 8 are complete.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: medium
- Dependencies: 8

## Implementation Plan
Step-by-step implementation:

1. HA scaling for cto-pm:
   - Increase replica count from 1 to at least 2
   - Add PodDisruptionBudget: minAvailable=1
   - Configure anti-affinity rules to spread pods across nodes
   - Verify Elysia handles stateless horizontal scaling (no in-memory session state)

2. Ingress configuration:
   - Create an Ingress resource for the PM server (if external access is needed for the dashboard)
   - TLS termination using cert-manager with Let's Encrypt or organization's CA
   - Rate limiting annotations (e.g., nginx ingress rate-limit: 100 req/min)

3. Network policies:
   - Default deny all ingress/egress for sigma-1 namespace
   - Allow ingress: cto-pm ← ingress controller (port 3000)
   - Allow egress: cto-pm → bots/discord-bridge-http, bots/linear-bridge (in-cluster)
   - Allow egress: cto-pm → Linear API, GitHub API, NOUS API (external, on 443)
   - Allow egress: cto-pm → Hermes (in-cluster, discovered port)
   - Allow egress: external-secrets-operator → backing secret store

4. Resource management:
   - Set resource requests and limits on all pods in sigma-1:
     - cto-pm: requests 256Mi/250m, limits 512Mi/500m (adjust based on observed usage)
   - Configure HPA (Horizontal Pod Autoscaler) for cto-pm: target CPU 70%, min 2, max 5

5. Health probes:
   - Liveness probe: HTTP GET /health, initialDelaySeconds=10, periodSeconds=30
   - Readiness probe: HTTP GET /ready, initialDelaySeconds=5, periodSeconds=10
   - Startup probe: HTTP GET /health, failureThreshold=30, periodSeconds=10

6. CDN (if dashboard is exposed externally):
   - Configure CDN caching for static Next.js assets (if Task 7 is deployed)
   - No caching for API routes

7. Label all production resources with `sigma-1-pipeline: production`.

## Acceptance Criteria
1. `kubectl get pods -n sigma-1 -l app=cto-pm` shows >= 2 running pods distributed across different nodes. 2. PodDisruptionBudget allows voluntary disruption only when >= 1 pod remains: `kubectl get pdb -n sigma-1` shows minAvailable=1. 3. NetworkPolicy is applied: `kubectl get networkpolicy -n sigma-1` lists at least one policy with default-deny and explicit allow rules. 4. Test network policy enforcement: a pod without allowed labels in sigma-1 cannot reach cto-pm (connection timeout). 5. Ingress returns valid TLS certificate: `curl -v https://{ingress-host}` shows TLS handshake with valid cert. 6. HPA is configured: `kubectl get hpa -n sigma-1` shows cto-pm HPA with min=2, max=5, target CPU=70%. 7. All pods have resource requests and limits set: `kubectl describe pod -n sigma-1` shows non-zero values for requests.cpu, requests.memory, limits.cpu, limits.memory.

## Subtasks
- Configure HA replica scaling and anti-affinity for cto-pm Deployment: Update the cto-pm Deployment to run at least 2 replicas with pod anti-affinity rules to spread pods across nodes, ensuring high availability.
- Create PodDisruptionBudget for cto-pm: Define a PodDisruptionBudget resource ensuring at least one cto-pm pod remains available during voluntary disruptions (node drains, upgrades).
- Configure Ingress resource with TLS termination via cert-manager: Create an Ingress resource for the cto-pm service with TLS termination using cert-manager, including a ClusterIssuer or Issuer for certificate provisioning.
- Create default-deny NetworkPolicy for sigma-1 namespace: Apply a default-deny-all NetworkPolicy for both ingress and egress traffic in the sigma-1 namespace, establishing a zero-trust baseline.
- Create NetworkPolicy allow rules for cto-pm ingress traffic: Define NetworkPolicy resources allowing ingress traffic to cto-pm from the ingress controller on port 3000.
- Create NetworkPolicy allow rules for cto-pm egress traffic: Define NetworkPolicy resources allowing egress from cto-pm to in-cluster services (discord-bridge-http, linear-bridge, Hermes) and external APIs (Linear, GitHub, NOUS on port 443).
- Set resource requests and limits on all sigma-1 pods: Configure CPU and memory requests and limits for all pods in the sigma-1 namespace, starting with cto-pm at 256Mi/250m requests and 512Mi/500m limits.
- Configure HorizontalPodAutoscaler for cto-pm: Create an HPA resource targeting the cto-pm Deployment with min 2, max 5 replicas, and target CPU utilization of 70%.
- Configure health probes (liveness, readiness, startup) for cto-pm: Add liveness, readiness, and startup probes to the cto-pm Deployment to enable Kubernetes to detect unhealthy pods and manage rolling updates correctly.
- Configure CDN caching for static dashboard assets: If the Next.js dashboard (Task 7) is deployed and exposed via Ingress, configure CDN caching for static assets while ensuring API routes bypass the cache.
- Label all sigma-1 production resources with pipeline label: Apply the label `sigma-1-pipeline: production` to all resources in the sigma-1 namespace for consistent identification and management.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.